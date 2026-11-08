//! Anchored plant balance. Algebraic carbon, heat and power balances anchored on
//! published INL cases. This is a screening port and it is verified against its
//! Python parent, so nothing here counts as validation of physics.
//!
//! Two changes from the parent: the reboiler heat order follows the approved
//! sequence (freed Selexol steam, export steam, syngas above 130 C, reactor
//! steam), and no default size exponent exists, so every cost call states `n`.

use crate::heatrec;
use crate::params::{ParamSet, Reader};
use crate::utility::{self, ReactorTerms};
use nush2_numerics::{fixed_point, EngineError, Result};
use nush2_props::units::{
    kmol_h_to_t_h, mmbtu_h_to_mw, mmscfd_to_kmol_h, mw_to_mmbtu_h, st_d_to_kmol_h, DAYS_PER_YEAR,
    GJ_PER_MWH, HOURS_PER_YEAR, LB, LHV_CH4, LHV_CO, LHV_H2, MMBTU_GJ, MW_CO2, MW_H2, MW_MEA, ST,
};
use nush2_units::circulator;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Plant basis. Case B is 100,000 Nm3/h H2, Case A the INL plant at 130 MMSCFD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Case {
    A,
    B,
}

/// Tail-gas scenario. Module header of `carbon` gives the meaning of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Scenario {
    InlSel,
    RecF,
    RecFix,
    CombH,
    CombN,
    Export,
}

impl Scenario {
    pub fn id(self) -> &'static str {
        match self {
            Scenario::InlSel => "INL_SEL",
            Scenario::RecF => "REC_F",
            Scenario::RecFix => "REC_FIX",
            Scenario::CombH => "COMB_H",
            Scenario::CombN => "COMB_N",
            Scenario::Export => "EXPORT",
        }
    }

    fn no_recycle(self) -> bool {
        matches!(self, Scenario::CombH | Scenario::CombN | Scenario::Export)
    }

    fn recycled_mea(self) -> bool {
        matches!(self, Scenario::RecF | Scenario::RecFix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostLevel {
    Low,
    Central,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircKey {
    None,
    Low,
    Central,
    Central5p15,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossKey {
    None,
    Low,
    Central,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// R = gap + w + (E - w)/eta with w = min(E, r gap).
    Extraction,
    /// The gap comes 1:1 from helium with no co-generation.
    Direct,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum VesselMode {
    Zero,
    Area,
    Prop,
}

/// Low, central and high values of one input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tri {
    pub low: f64,
    pub central: f64,
    pub high: f64,
}

impl Tri {
    pub fn at(&self, level: CostLevel) -> f64 {
        match level {
            CostLevel::Low => self.low,
            CostLevel::Central => self.central,
            CostLevel::High => self.high,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CircSet {
    p_mpa: f64,
    dp_sec_kpa: f64,
    dp_prim_kpa: f64,
    eta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LossSet {
    ihx: f64,
    duct: f64,
    vessel: VesselMode,
}

/// Published electricity rows of TEV-961 Table 2, MWe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ERows {
    pub reforming: f64,
    pub purification: f64,
    pub steam: f64,
    pub co2comp: f64,
    pub purge: f64,
    pub cooling: f64,
    pub water: f64,
}

impl ERows {
    pub fn sum(&self) -> f64 {
        self.reforming
            + self.purification
            + self.steam
            + self.co2comp
            + self.purge
            + self.cooling
            + self.water
    }
}

/// Inputs and derived constants. Built from a `ParamSet`, so an override flows
/// through every derived value.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub avail: f64,
    pub hours: f64,
    pub days: f64,
    pub h2_a_kgh: f64,
    pub h2_a_mmscfd: f64,
    pub rho_h2n: f64,
    pub h2_b_nm3h: f64,
    pub h2_b_kgh: f64,
    pub s_b: f64,
    pub ng_c6: f64,
    pub ng_conv: f64,
    pub ng_norec: f64,
    pub ng_tube_cc: f64,
    pub ng_hhv: f64,
    pub ng_cc_total: f64,
    pub ng_ahn_equivalent: f64,
    pub c_per_mol: f64,
    pub co2_cap_c6: f64,
    pub co2_emit_c6: f64,
    pub co2_conv: f64,
    pub co2_htgr_nocc: f64,
    pub co2_cap_convcc: f64,
    pub co2_emit_convcc: f64,
    pub eta_sel: f64,
    pub eta_mea: f64,
    pub ch4_in_h2: f64,
    pub phi_rec: f64,
    pub psa_rec: f64,
    pub q_proc: f64,
    pub q_proc_ahn_b: f64,
    pub he_flow: f64,
    pub t_he_s: f64,
    pub t_he_r: f64,
    pub rot: f64,
    pub ihx_approach: f64,
    pub eta_pwr: f64,
    pub eta_pwr_alt: f64,
    pub e_rows: ERows,
    pub e_psa: f64,
    pub export_c6: f64,
    pub export_c5: f64,
    pub export_conv: f64,
    pub e_conv: f64,
    pub q_selexol_c5: f64,
    pub q_zld_c5: f64,
    pub case5_cap: f64,
    pub q_ex4_above130: f64,
    pub srd: Tri,
    pub rich: f64,
    pub lean: f64,
    pub w_mea: f64,
    pub rho_solv: f64,
    pub p_abs: f64,
    pub p_strip: f64,
    pub eta_pump: f64,
    pub eta_motor: f64,
    pub k_cool: f64,
    pub cp_he: f64,
    circ_low: CircSet,
    circ_central: CircSet,
    circ_central_5p15: CircSet,
    circ_high: CircSet,
    pub t_circ_s: f64,
    pub t_circ_p: f64,
    pub q_vessel_ref: f64,
    pub q_vessel_ref_mw: f64,
    loss_low: LossSet,
    loss_central: LossSet,
    loss_high: LossSet,
    pub loss_high_vessel_fraction: f64,
    pub r_work_low_r: f64,
    pub r_work_central: f64,
    pub eta_boiler: f64,
    pub eta_tg: f64,
    pub co_share: f64,
    pub slip_cp: f64,
    pub slip_dt: f64,
    pub psa_reject_kmol_h: f64,
    pub water_gpm_a: f64,
    pub rate: f64,
    pub lives: [f64; 3],
    pub item_life: f64,
    pub fx: f64,
    pub f2019: f64,
    pub f2014: f64,
    pub gas: Tri,
    pub ts: Tri,
    pub occ_kwt: Tri,
    pub opex: Tri,
    pub fuel_share: f64,
    pub foak: Tri,
    pub lf10: f64,
    pub ref_mwt: f64,
    pub n_team: f64,
    pub n_linear: f64,
    pub n_ngnp: f64,
    pub r_rated: f64,
    pub ecs_kwe: f64,
    pub decom: Tri,
    pub decom_kwe_per_kwt: f64,
    pub sff_rate: Tri,
    pub licence: Tri,
    pub ihx_ref_mw: f64,
    pub ihx_s: f64,
    pub ihx_low_s: f64,
    pub ihx_high_mult: f64,
    pub ihx_life_high: f64,
    pub capt_k: Tri,
    pub fom: Tri,
    pub allin: f64,
    pub recycle_capex_a: Tri,
    pub price_e: f64,
    pub ef_grid: f64,
    pub ef_ng_lhv: f64,
    pub ef_up: f64,
    pub ts_burden: f64,
    pub ef_nuc: f64,
    pub system_boiler_eff: f64,
    pub base_cho: f64,
    pub base_ieaghg: f64,
    pub base_ieaghg_rounded: f64,
    pub base_inl: f64,
    pub idc_years: f64,
    pub target_a: f64,
    pub target_c: f64,
    pub f_c6: f64,
    pub p_c: f64,
    pub phi_nr: f64,
    pub smr_tcr_eurm: f64,
    pub smr_tpc_eurm: f64,
    pub smr_fom_eur_y: f64,
    pub smr_water_eur_y: f64,
    pub smr_chem_eur_y: f64,
    pub smr_availability: f64,
    pub comparator_emit_kg_nm3: f64,
    pub comparator_base_kg_nm3: f64,
    pub comparator_captured_kg_nm3: f64,
    pub comparator_cac_eur: f64,
    pub comparator_overall: f64,
    pub amdea_eta: f64,
    pub amdea_srd: f64,
    pub tcr_tpc: f64,
    pub ex4: Ex4Check,
    pub duty_chain_c5: [[f64; 3]; 6],
    /// Steam grades of the INL steam system, C (TEV-961 A-57 PDF p81): LP steam,
    /// which Selexol and ZLD use, HP steam, which is exported, and the hot water
    /// loop supply.
    pub t_lp_steam_c: f64,
    pub t_hp_steam_c: f64,
    pub t_hot_water_c: f64,
    /// Preset factors on T04, T07 and T08; 1.0 in the Anchored preset.
    pub scale_t04: f64,
    pub scale_t07: f64,
    pub scale_t08: f64,
}

/// Inputs of the independent WGS-EX4 cooling curve check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ex4Check {
    pub dry: f64,
    pub cp_dry: f64,
    pub cp_v: f64,
    pub latent: f64,
    pub p_bar: f64,
    pub t_in: f64,
    pub t_out: f64,
    pub ant_a: f64,
    pub ant_b: f64,
    pub ant_c: f64,
    pub ant_bar: f64,
}

/// Carbon balance at Case A in kmol C/h.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Carbon {
    pub scn: Scenario,
    pub ng: f64,
    pub h2: f64,
    pub h_scale: f64,
    pub p: f64,
    pub f: f64,
    pub t: f64,
    pub co2_abs: f64,
    pub cap: f64,
    pub unconv: f64,
    pub g: f64,
    pub r: f64,
    pub burn: f64,
    pub exported: f64,
    pub slip: f64,
    pub closure: f64,
}

/// Heat sources for the reboiler, MW at the chosen scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Stack {
    pub selexol: f64,
    pub export: f64,
    pub ex4: f64,
    pub zld: f64,
}

/// Process electricity, MWe at the chosen scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EUse {
    pub reforming: f64,
    pub purification: f64,
    pub steam: f64,
    pub co2comp: f64,
    pub purge: f64,
    pub cooling: f64,
    pub water: f64,
    pub mea_pump: f64,
    pub mea_cooling: f64,
}

/// Options of `Model::balance`. `Model::opts` gives the central set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceOpts {
    pub s: f64,
    pub srd: f64,
    pub circ: CircKey,
    pub loss: LossKey,
    pub eta_p: f64,
    pub r_work: f64,
    pub route: Route,
    pub phi: f64,
    pub eta_c: f64,
    pub use_export: bool,
    pub use_selexol: bool,
    pub use_ex4: bool,
    pub use_zld: bool,
    pub capture_energy: bool,
    pub purge_override: Option<f64>,
    /// True when the 39.7% efficiency already nets off primary circulation.
    pub eta_net_of_primary_circ: bool,
}

/// Heat and power balance, MW at the chosen scale.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Balance {
    pub scn: Scenario,
    pub s: f64,
    pub srd: f64,
    pub q_proc: f64,
    pub q_recyc: f64,
    pub q_reb: f64,
    pub avail: f64,
    pub gap: f64,
    pub stack: Stack,
    pub q_ihx: f64,
    pub q_duct: f64,
    pub e_rows: EUse,
    pub e_proc: f64,
    pub r: f64,
    pub q_tg: f64,
    pub lhv: f64,
    pub export_kept: f64,
    pub tg_surplus: f64,
    pub gap_after_tg: f64,
    pub cap_t: f64,
    pub c: Carbon,
    pub w_s: f64,
    pub w_p: f64,
    pub e_circ: f64,
    pub e_total: f64,
    pub e_tg: f64,
    pub e_nuc: f64,
    pub branch: f64,
    pub qv: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SolveState {
    w_s: f64,
    w_p: f64,
    e_circ: f64,
    e_total: f64,
    e_tg: f64,
    e_nuc: f64,
    branch: f64,
    qv: f64,
}

/// Capital recovery factor.
pub fn crf(i: f64, n: f64) -> Result<f64> {
    if i <= 0.0 || n <= 0.0 {
        return Err(EngineError::Range {
            param: "crf".into(),
            msg: format!("rate {i} and life {n} must both be positive"),
        });
    }
    Ok(i / (1.0 - (1.0 + i).powf(-n)))
}

/// Sinking fund factor.
pub fn sff(i: f64, n: f64) -> Result<f64> {
    if i <= 0.0 || n <= 0.0 {
        return Err(EngineError::Range {
            param: "sff".into(),
            msg: format!("rate {i} and life {n} must both be positive"),
        });
    }
    Ok(i / ((1.0 + i).powf(n) - 1.0))
}

/// Hard limits checked before a model is built. Never clamped.
const LIMITS: [(&str, f64, f64); 8] = [
    ("AVAILABILITY", 0.01, 1.0),
    ("DISCOUNT", 1e-4, 1.0),
    ("ETA_MEA", 0.01, 1.0),
    ("PHI_RECYCLE", 0.01, 1.0),
    ("ETA_POWER", 0.01, 1.0),
    ("SRD_CENTRAL", 1e-6, 20.0),
    ("REACTOR_RATED_MWTH", 1.0, 2000.0),
    ("GAS_CENTRAL", -100.0, 1000.0),
];

/// Reject out-of-range inputs with an error naming the parameter.
pub fn validate(p: &ParamSet) -> Result<()> {
    for (id, lo, hi) in LIMITS {
        let v = p.get(id)?;
        if !(lo..=hi).contains(&v) {
            return Err(EngineError::Range {
                param: id.to_string(),
                msg: format!("{v} lies outside {lo} to {hi}"),
            });
        }
    }
    Ok(())
}

impl Model {
    /// Build from a parameter set.
    pub fn new(p: &ParamSet) -> Result<Self> {
        Ok(Self::new_logged(p)?.0)
    }

    /// Build from the embedded file.
    pub fn embedded() -> Result<Self> {
        Self::new(&ParamSet::embedded()?)
    }

    /// Build and return the set of ids the model read.
    pub fn new_logged(p: &ParamSet) -> Result<(Self, BTreeSet<String>)> {
        validate(p)?;
        let mut r = Reader::new(p);
        let m = Self::build(&mut r)?;
        Ok((m, r.into_read()))
    }

    fn build(r: &mut Reader<'_>) -> Result<Self> {
        let avail = r.get("AVAILABILITY")?;
        let hours = avail * HOURS_PER_YEAR;
        let days = avail * DAYS_PER_YEAR;
        let h2_a_kgh = r.get("H2_A")? * LB;
        let h2_a_mmscfd = r.get("H2_A_MMSCFD")?;
        let rho_h2n = r.get("RHO_H2_NORMAL")?;
        let h2_b_nm3h = r.get("H2_B")?;
        let h2_b_kgh = h2_b_nm3h * rho_h2n;
        let s_b = h2_b_kgh / h2_a_kgh;

        let ng_c6 = r.get("NG_CASE6")?;
        let ng_conv = r.get("NG_CONV_NOCC")?;
        let ng_cc_total = r.get("NG_TUBE_CONVCC_TOTAL")?;
        let ng_tube_cc = ng_cc_total * r.get("NG_TUBE_CONVCC_TO_TUBES")?;
        let mut c_per_mol = 0.0;
        for (sp, carbons) in [
            ("CH4", 1.0),
            ("C2", 2.0),
            ("C3", 3.0),
            ("C4", 4.0),
            ("C5", 5.0),
            ("C6", 6.0),
            ("CO2", 1.0),
        ] {
            c_per_mol += r.get(&format!("C_PER_MOL_NG_{sp}"))? * carbons;
        }

        let co2_cap_c6 = r.get("CO2_CAPTURED_C6")?;
        let co2_cap_convcc = r.get("CO2_CAPTURED_CONVCC")?;
        let eta_sel = r.get("ETA_SELEXOL")?;
        let ch4_in_h2 = r.get("CH4_IN_H2")?;

        let e_rows = ERows {
            reforming: r.get("E_REFORMING")?,
            purification: r.get("E_PURIFICATION")?,
            steam: r.get("E_STEAM")?,
            co2comp: r.get("E_CO2COMP")?,
            purge: r.get("E_PURGE")?,
            cooling: r.get("E_COOLING")?,
            water: r.get("E_WATER")?,
        };

        let circ = |r: &mut Reader<'_>, name: &str| -> Result<CircSet> {
            Ok(CircSet {
                p_mpa: r.get(&format!("{name}_P_MPA"))?,
                dp_sec_kpa: r.get(&format!("{name}_DP_SEC_KPA"))?,
                dp_prim_kpa: r.get(&format!("{name}_DP_PRIM_KPA"))?,
                eta: r.get(&format!("{name}_ETA"))?,
            })
        };
        let circ_low = circ(r, "CIRC_LOW")?;
        let circ_central = circ(r, "CIRC_CENTRAL")?;
        let circ_central_5p15 = circ(r, "CIRC_CENTRAL_5P15")?;
        let circ_high = circ(r, "CIRC_HIGH")?;

        let q_proc = r.get("Q_PROCESS")?;
        let loss_low = LossSet {
            ihx: r.get("LOSS_LOW_IHX")?,
            duct: r.get("LOSS_LOW_DUCT")?,
            vessel: VesselMode::Area,
        };
        let loss_central = LossSet {
            ihx: r.get("LOSS_CENTRAL_IHX")?,
            duct: r.get("LOSS_CENTRAL_DUCT")?,
            vessel: VesselMode::Area,
        };
        // The 8.1 MWth allowance is tied to the published 176.8 MWth duty.
        let ihx_low_ref = r.get("IHX_LOW_REF_MWTH")?;
        let loss_high = LossSet {
            ihx: r.get("LOSS_HIGH_IHX_MWTH")? / ihx_low_ref,
            duct: r.get("LOSS_HIGH_DUCT")?,
            vessel: VesselMode::Prop,
        };

        let co_part = r.get("CO_SHARE_OF_UNCONVERTED_CO")?;
        let ch4_part = r.get("CO_SHARE_OF_UNCONVERTED_CH4")?;
        let co_share = co_part / (co_part + ch4_part);

        let rate = r.get("DISCOUNT")?;
        let fx = r.get("FX")?;
        let f2019 = r.get("F_2019USD_TO_2025SGD")?;
        let f2014 = r.get("F_2014EUR_TO_2025SGD")?;
        let foak = Tri {
            low: r.get("FOAK_LOW")?,
            central: r.get("FOAK_CENTRAL")?,
            high: r.get("FOAK_HIGH")?,
        };
        let lf10 = (1.0 - r.get("LEARNING")?).powf(r.get("LEARNING_UNITS")?.log2());

        let n_ngnp = {
            let c600 = r.get("N_NGNP_COST_600")?;
            let c350 = r.get("N_NGNP_COST_350")?;
            let s600 = r.get("N_NGNP_SIZE_600")?;
            let s350 = r.get("N_NGNP_SIZE_350")?;
            1.0 + (c350 / c600).ln() / (s350 / s600).ln()
        };

        let ihx_defl = r.get("DEFLATOR_2012_2025_NUM")? / r.get("DEFLATOR_2012_2025_DEN")?;
        let ihx_s = r.get("IHX_LOOP_COST")? * ihx_defl * fx;
        let ihx_low_s = r.get("IHX_LOW_USD_KWTH")? * 1.0e-3 * ihx_low_ref * ihx_defl * fx;

        let k1a_t_h = r.get("CAPTURE_K_1A_T_H")?;
        let capt_1a = r.get("CAPTURE_K_1A_DTCR_EURM")? * f2014 / k1a_t_h;
        let capt_3 = r.get("CAPTURE_K_3_DTCR_EURM")? * f2014 / r.get("CAPTURE_K_3_T_H")?;
        let capt_k = Tri {
            low: capt_1a * r.get("CAPTURE_K_LOW_FACTOR")?,
            central: capt_1a,
            high: capt_3,
        };
        let fom_central = r.get("CAPTURE_FOM")?;
        let fom = Tri {
            low: fom_central,
            central: fom_central,
            high: r.get("CAPTURE_FOM_HIGH")?,
        };

        let comp_k = r.get("RECYCLE_TRAIN_K_TPC_EURM")?
            / (k1a_t_h * 24.0 / ST * r.get("RECYCLE_TRAIN_K_KW_PER_STD")?);
        let tcr_tpc = r.get("TCR_OVER_TPC")?;
        let recycle_capex_a = Tri {
            low: 0.0,
            central: e_rows.purge * comp_k * tcr_tpc * f2014,
            high: r.get("RECYCLE_TRAIN_HIGH_TPC_EURM")? * tcr_tpc * f2014 / s_b,
        };

        let f_c6 = mmscfd_to_kmol_h(ng_c6) * c_per_mol;
        let p_c = mmscfd_to_kmol_h(h2_a_mmscfd) * ch4_in_h2;
        let phi_nr = (co2_cap_convcc / eta_sel) * ST * 1000.0
            / 24.0
            / MW_CO2
            / (mmscfd_to_kmol_h(ng_tube_cc) * c_per_mol);
        let co2_conv = r.get("CO2_CONV_NOCC")?;
        let base_inl = co2_conv * ST * 365.0 / (h2_a_kgh * 8.76);

        let mut duty_chain_c5 = [[0.0; 3]; 6];
        for (i, tag) in [
            "HPS_GEN1", "WGS_EX1", "WGS_EX2", "WGS_EX3", "WGS_EX4", "KO_DRM_1",
        ]
        .iter()
        .enumerate()
        {
            duty_chain_c5[i] = [
                r.get(&format!("SYNGAS_CHAIN_C5_{tag}_MMBTU_H"))?,
                r.get(&format!("SYNGAS_CHAIN_C5_{tag}_T_IN_C"))?,
                r.get(&format!("SYNGAS_CHAIN_C5_{tag}_T_OUT_C"))?,
            ];
        }

        let comparator_base_kg_nm3 = r.get("BASELINE_IEAGHG_KG_NM3")?;
        let f_to_c = |f: f64| (f - 32.0) / 1.8;

        Ok(Self {
            avail,
            hours,
            days,
            h2_a_kgh,
            h2_a_mmscfd,
            rho_h2n,
            h2_b_nm3h,
            h2_b_kgh,
            s_b,
            ng_c6,
            ng_conv,
            ng_norec: r.get("NG_NORECYCLE")?,
            ng_tube_cc,
            ng_hhv: r.get("NG_HHV")?,
            ng_cc_total,
            ng_ahn_equivalent: ng_conv * r.get("NG_AHN_EQUIVALENT_NG_KG_H")?
                / r.get("NG_AHN_EQUIVALENT_BASE_KG_H")?,
            c_per_mol,
            co2_cap_c6,
            co2_emit_c6: r.get("CO2_EMITTED_C6")?,
            co2_conv,
            co2_htgr_nocc: r.get("CO2_HTGR_NOCC")?,
            co2_cap_convcc,
            co2_emit_convcc: r.get("CO2_EMITTED_CONVCC")?,
            eta_sel,
            eta_mea: r.get("ETA_MEA")?,
            ch4_in_h2,
            phi_rec: r.get("PHI_RECYCLE")?,
            psa_rec: r.get("PSA_H2_RECOVERY")?,
            q_proc,
            q_proc_ahn_b: r.get("Q_PROCESS_AHN_CASE_B")?,
            he_flow: r.get("HE_FLOW")?,
            t_he_s: r.get("T_HE_SUPPLY")?,
            t_he_r: r.get("T_HE_RETURN")?,
            rot: r.get("ROT")?,
            ihx_approach: r.get("IHX_APPROACH_K")?,
            eta_pwr: r.get("ETA_POWER")?,
            eta_pwr_alt: r.get("ETA_POWER_ALT")?,
            e_rows,
            e_psa: r.get("E_PSA")?,
            export_c6: r.get("EXPORT_STEAM_C6")?,
            export_c5: r.get("EXPORT_STEAM_C5")?,
            export_conv: r.get("EXPORT_STEAM_CONV")?,
            e_conv: r.get("E_CONV_NOCC")?,
            q_selexol_c5: mmbtu_h_to_mw(r.get("Q_SELEXOL_C5_MMBTU_H")?),
            q_zld_c5: mmbtu_h_to_mw(r.get("Q_ZLD_C5_MMBTU_H")?),
            case5_cap: r.get("CO2_CAPTURED_C5")?,
            q_ex4_above130: r.get("Q_EX4_ABOVE_130C")?,
            srd: Tri {
                low: r.get("SRD_LOW")?,
                central: r.get("SRD_CENTRAL")?,
                high: r.get("SRD_HIGH")?,
            },
            rich: r.get("LOADING_RICH")?,
            lean: r.get("LOADING_LEAN")?,
            w_mea: r.get("W_MEA")?,
            rho_solv: r.get("RHO_SOLVENT")?,
            p_abs: r.get("P_ABSORBER")?,
            p_strip: r.get("P_STRIPPER")?,
            eta_pump: r.get("ETA_PUMP")?,
            eta_motor: r.get("ETA_MOTOR")?,
            k_cool: r.get("K_COOLING")?,
            cp_he: r.get("CP_HE")?,
            circ_low,
            circ_central,
            circ_central_5p15,
            circ_high,
            t_circ_s: r.get("T_CIRC_SEC_K")?,
            t_circ_p: r.get("T_CIRC_PRIM_K")?,
            q_vessel_ref: r.get("VESSEL_LOSS_HTTR")?,
            q_vessel_ref_mw: r.get("VESSEL_LOSS_REF_MW")?,
            loss_low,
            loss_central,
            loss_high,
            loss_high_vessel_fraction: r.get("LOSS_HIGH_VESSEL_FRACTION")?,
            r_work_low_r: r.get("R_WORK_HIGH")?,
            r_work_central: r.get("R_WORK_CENTRAL")?,
            eta_boiler: r.get("ETA_BOILER")?,
            eta_tg: r.get("ETA_TAILGAS_POWER")?,
            co_share,
            slip_cp: r.get("SLIP_CP_KJ_KMOL_K")?,
            slip_dt: r.get("SLIP_DT_K")?,
            psa_reject_kmol_h: r.get("PSA_REJECT_KMOL_H")?,
            water_gpm_a: r.get("WATER_GPM_CASE_A")?,
            rate,
            lives: [
                r.get("LIVES_SHORT")?,
                r.get("LIVES_MID")?,
                r.get("LIVES_LONG")?,
            ],
            item_life: r.get("ITEM_LIFE_Y")?,
            fx,
            f2019,
            f2014,
            gas: Tri {
                low: r.get("GAS_LOW")?,
                central: r.get("GAS_CENTRAL")?,
                high: r.get("GAS_HIGH")?,
            },
            ts: Tri {
                low: r.get("TS_LOW")?,
                central: r.get("TS_CENTRAL")?,
                high: r.get("TS_HIGH")?,
            },
            occ_kwt: Tri {
                low: r.get("OCC_LOW")?,
                central: r.get("OCC_CENTRAL")?,
                high: r.get("OCC_HIGH")?,
            },
            opex: Tri {
                low: r.get("OPEX_LOW")?,
                central: r.get("OPEX_CENTRAL")?,
                high: r.get("OPEX_HIGH")?,
            },
            fuel_share: r.get("OPEX_FUEL_SHARE_HTGR")?,
            foak,
            lf10,
            ref_mwt: r.get("REACTOR_COST_REF_SIZE")?,
            n_team: r.get("N_TEAM")?,
            n_linear: r.get("N_LINEAR")?,
            n_ngnp,
            r_rated: r.get("REACTOR_RATED_MWTH")?,
            ecs_kwe: r.get("POWER_ISLAND_ADDBACK")?,
            decom: Tri {
                low: r.get("DECOM_LOW")?,
                central: r.get("DECOM_CENTRAL")?,
                high: r.get("DECOM_HIGH")?,
            },
            decom_kwe_per_kwt: r.get("DECOM_KWE_PER_KWT")?,
            sff_rate: Tri {
                low: rate,
                central: rate,
                high: r.get("DECOM_FUND_RATE_HIGH")?,
            },
            licence: Tri {
                low: r.get("LICENCE_LOW")?,
                central: r.get("LICENCE_CENTRAL")?,
                high: r.get("LICENCE_HIGH_NRC_USDM")?
                    + r.get("LICENCE_HIGH_FOURPACK_USDM")?
                        / r.get("LICENCE_HIGH_FOURPACK_UNITS")?,
            },
            ihx_ref_mw: r.get("IHX_REF_MW")?,
            ihx_s,
            ihx_low_s,
            ihx_high_mult: r.get("IHX_HIGH_MULT")?,
            ihx_life_high: r.get("IHX_LIFE_HIGH_H")? / hours,
            capt_k,
            fom,
            allin: r.get("CAPTURE_ALLIN")? * f2019,
            recycle_capex_a,
            price_e: r.get("ELECTRICITY_PRICE")?,
            ef_grid: r.get("GRID_EF")?,
            ef_ng_lhv: r.get("EF_NG_LHV")?,
            ef_up: r.get("EF_UPSTREAM")?,
            ts_burden: r.get("TS_BURDEN")?,
            ef_nuc: r.get("EF_NUCLEAR")?,
            system_boiler_eff: r.get("SYSTEM_BOILER_EFF")?,
            base_cho: r.get("BASELINE_CHO")?,
            base_ieaghg: comparator_base_kg_nm3 / r.get("BASELINE_IEAGHG_RHO")?,
            base_ieaghg_rounded: r.get("BASELINE_IEAGHG_ROUNDED")?,
            base_inl,
            idc_years: r.get("CONSTRUCTION_YEARS")?,
            target_a: r.get("TARGET_ABATEMENT")?,
            target_c: r.get("TARGET_COST")?,
            f_c6,
            p_c,
            phi_nr,
            smr_tcr_eurm: r.get("SMR_ISLAND_TCR_EURM")?,
            smr_tpc_eurm: r.get("SMR_ISLAND_TPC_EURM")?,
            smr_fom_eur_y: r.get("SMR_ISLAND_FOM_EUR_Y")?,
            smr_water_eur_y: r.get("SMR_ISLAND_RAW_WATER_EUR_Y")?,
            smr_chem_eur_y: r.get("SMR_ISLAND_CHEMICALS_EUR_Y")?,
            smr_availability: r.get("SMR_ISLAND_AVAILABILITY")?,
            comparator_emit_kg_nm3: r.get("COMPARATOR_EMIT_CASE3_KG_NM3")?,
            comparator_base_kg_nm3,
            comparator_captured_kg_nm3: r.get("COMPARATOR_CAPTURED_CASE3_KG_NM3")?,
            comparator_cac_eur: r.get("COMPARATOR_CAC_EUR")?,
            comparator_overall: r.get("COMPARATOR_OVERALL_CAPTURE")?,
            amdea_eta: r.get("AMDEA_ETA")?,
            amdea_srd: r.get("AMDEA_SRD")?,
            tcr_tpc,
            ex4: Ex4Check {
                dry: r.get("EX4_CHECK_DRY_KMOL_H")?,
                cp_dry: r.get("EX4_CHECK_CP_DRY")?,
                cp_v: r.get("EX4_CHECK_CP_VAP")?,
                latent: r.get("EX4_CHECK_LATENT")?,
                p_bar: r.get("EX4_CHECK_P_BAR")?,
                t_in: r.get("EX4_CHECK_T_IN_C")?,
                t_out: r.get("EX4_CHECK_T_OUT_C")?,
                ant_a: r.get("ANTOINE_A")?,
                ant_b: r.get("ANTOINE_B")?,
                ant_c: r.get("ANTOINE_C")?,
                ant_bar: r.get("ANTOINE_MMHG_TO_BAR")?,
            },
            duty_chain_c5,
            t_lp_steam_c: f_to_c(r.get("T_LP_STEAM_C5_F")?),
            t_hp_steam_c: f_to_c(r.get("T_HP_STEAM_C5_F")?),
            t_hot_water_c: f_to_c(r.get("T_HOT_WATER_C5_F")?),
            scale_t04: r.term_scale("T04"),
            scale_t07: r.term_scale("T07"),
            scale_t08: r.term_scale("T08"),
        })
    }

    /// Central balance options at scale `s`.
    pub fn opts(&self, s: f64) -> BalanceOpts {
        BalanceOpts {
            s,
            srd: self.srd.central,
            circ: CircKey::Central,
            loss: LossKey::Central,
            eta_p: self.eta_pwr,
            r_work: self.r_work_central,
            route: Route::Extraction,
            phi: self.phi_rec,
            eta_c: self.eta_mea,
            use_export: true,
            use_selexol: true,
            use_ex4: true,
            use_zld: false,
            capture_energy: true,
            purge_override: None,
            eta_net_of_primary_circ: false,
        }
    }

    /// Scale of a case relative to the INL plant.
    pub fn scale(&self, case: Case) -> f64 {
        match case {
            Case::A => 1.0,
            Case::B => self.s_b,
        }
    }

    /// CO2 t/y from kmol/h at scale `s`.
    pub fn tpy(&self, kmol_h: f64, s: f64) -> f64 {
        kmol_h_to_t_h(kmol_h) * self.hours * s
    }

    /// Carbon balance at Case A, kmol C/h.
    ///
    /// INL_SEL: INL Case 6 as published, Selexol 99%.
    /// REC_F: recycle kept, MEA, cryogenic cut keeps the INL split fraction.
    /// REC_FIX: recycle kept, MEA, cryogenic cut carbon held at the INL value.
    /// COMB_H: no recycle, tail gas burned on site, H2 held.
    /// COMB_N: no recycle, tail gas burned on site, natural gas held.
    /// EXPORT: no recycle, tail gas leaves the boundary and is burned elsewhere.
    pub fn carbon(&self, scn: Scenario, phi: f64, eta: f64) -> Carbon {
        let cap6 = st_d_to_kmol_h(self.co2_cap_c6);
        let t6 = (cap6 / self.eta_sel) / phi;
        let g6 = t6 - cap6 - self.p_c;
        let f6 = (t6 - self.f_c6) / g6;
        let mut c = Carbon {
            scn,
            ng: self.ng_c6,
            h2: self.h2_a_mmscfd,
            h_scale: 1.0,
            p: self.p_c,
            f: self.f_c6,
            t: 0.0,
            co2_abs: 0.0,
            cap: 0.0,
            unconv: 0.0,
            g: 0.0,
            r: 0.0,
            burn: 0.0,
            exported: 0.0,
            slip: 0.0,
            closure: 0.0,
        };
        match scn {
            Scenario::InlSel => {
                c.t = t6;
                c.co2_abs = cap6 / self.eta_sel;
                c.cap = cap6;
                c.unconv = t6 * (1.0 - phi);
                c.g = g6;
                c.r = t6 - self.f_c6;
                c.burn = g6 - (t6 - self.f_c6);
            }
            Scenario::RecF => {
                let t = (self.f_c6 - f6 * self.p_c) / (1.0 - f6 * (1.0 - eta * phi));
                let cap = eta * phi * t;
                let g = t - cap - self.p_c;
                c.t = t;
                c.co2_abs = phi * t;
                c.cap = cap;
                c.unconv = t * (1.0 - phi);
                c.g = g;
                c.r = f6 * g;
                c.burn = (1.0 - f6) * g;
            }
            Scenario::RecFix => {
                let burn = self.f_c6 - cap6 - self.p_c;
                let cap = self.f_c6 - self.p_c - burn;
                let t = cap / (eta * phi);
                let g = t - cap - self.p_c;
                c.t = t;
                c.co2_abs = phi * t;
                c.cap = cap;
                c.unconv = t * (1.0 - phi);
                c.g = g;
                c.r = g - burn;
                c.burn = burn;
            }
            Scenario::CombH | Scenario::Export => {
                let f = mmscfd_to_kmol_h(self.ng_norec) * self.c_per_mol;
                let cap = eta * self.phi_nr * f;
                let g = f - cap - self.p_c;
                c.ng = self.ng_norec;
                c.f = f;
                c.t = f;
                c.co2_abs = self.phi_nr * f;
                c.cap = cap;
                c.unconv = f * (1.0 - self.phi_nr);
                c.g = g;
                if scn == Scenario::CombH {
                    c.burn = g;
                } else {
                    c.exported = g;
                }
            }
            Scenario::CombN => {
                let sc = self.ng_c6 / self.ng_norec;
                let f = self.f_c6;
                let p = self.p_c * sc;
                let cap = eta * self.phi_nr * f;
                let g = f - cap - p;
                c.f = f;
                c.t = f;
                c.co2_abs = self.phi_nr * f;
                c.cap = cap;
                c.unconv = f * (1.0 - self.phi_nr);
                c.g = g;
                c.burn = g;
                c.h2 = self.h2_a_mmscfd * sc;
                c.h_scale = sc;
                c.p = p;
            }
        }
        c.slip = c.co2_abs - c.cap;
        c.closure = c.f - c.cap - c.burn - c.exported - c.p - (c.g - c.r - c.burn - c.exported);
        c
    }

    fn circ_set(&self, key: CircKey) -> Option<CircSet> {
        match key {
            CircKey::None => None,
            CircKey::Low => Some(self.circ_low),
            CircKey::Central => Some(self.circ_central),
            CircKey::Central5p15 => Some(self.circ_central_5p15),
            CircKey::High => Some(self.circ_high),
        }
    }

    fn loss_set(&self, key: LossKey) -> LossSet {
        match key {
            LossKey::None => LossSet {
                ihx: 0.0,
                duct: 0.0,
                vessel: VesselMode::Zero,
            },
            LossKey::Low => self.loss_low,
            LossKey::Central => self.loss_central,
            LossKey::High => self.loss_high,
        }
    }

    /// Central secondary and primary circulator pressure drops, kPa.
    pub fn circ_dp_central(&self) -> (f64, f64) {
        (self.circ_central.dp_sec_kpa, self.circ_central.dp_prim_kpa)
    }

    /// Central loop pressure of the circulator budget, MPa.
    pub fn circ_p_central_mpa(&self) -> f64 {
        self.circ_central.p_mpa
    }

    /// Central circulator isentropic efficiency.
    pub fn circ_eta_central(&self) -> f64 {
        self.circ_central.eta
    }

    /// Central duct loss at Case A, MW (card B.14 reference).
    pub fn loss_central_duct(&self) -> f64 {
        self.loss_central.duct
    }

    /// Shaft power of a helium circulator, MW.
    pub fn circ_shaft(&self, md: f64, t_k: f64, p_mpa: f64, dp_kpa: f64, eta: f64) -> f64 {
        circulator::shaft(md, self.cp_he, t_k, p_mpa, dp_kpa, eta)
    }

    fn vessel_loss(&self, mode: VesselMode, r: f64) -> f64 {
        match mode {
            VesselMode::Zero => 0.0,
            VesselMode::Area => self.q_vessel_ref * (r / self.q_vessel_ref_mw).powf(2.0 / 3.0),
            VesselMode::Prop => self.loss_high_vessel_fraction * r,
        }
    }

    /// Tail gas LHV in MW at Case A for the no-recycle scenarios.
    fn tail_gas_lhv(&self, c: &Carbon) -> f64 {
        let h2 = self.h2_a_kgh * c.h_scale / self.psa_rec * (1.0 - self.psa_rec) / MW_H2;
        let unc = c.unconv;
        let q = h2 * LHV_H2 + unc * (1.0 - self.co_share) * LHV_CH4 + unc * self.co_share * LHV_CO;
        q / 3600.0
    }

    /// Heat and power balance, solved for the reactor rating.
    pub fn balance(&self, scn: Scenario, o: &BalanceOpts) -> Result<Balance> {
        let s = o.s;
        let c = self.carbon(scn, o.phi, o.eta_c);
        let inl = self.carbon(Scenario::InlSel, o.phi, o.eta_c);
        let t_ratio = c.t / inl.t;
        let no_recycle = scn.no_recycle();
        // Process heat. No-recycle tube carbon is higher, so duty scales on tube carbon.
        let q_proc = self.q_proc * if no_recycle { t_ratio } else { 1.0 };
        // Sensible heat for CO2 slip recycled through the reformer. RWGS is not modelled.
        let extra_co2 = if scn.recycled_mea() {
            (c.slip - inl.slip).max(0.0)
        } else {
            0.0
        };
        let q_recyc = extra_co2 * self.slip_cp * self.slip_dt / 3600.0 / 1000.0 * self.scale_t04;
        let cap_t = kmol_h_to_t_h(c.cap);
        let mea = scn != Scenario::InlSel;
        let q_reb = if mea && o.capture_energy {
            cap_t * o.srd / GJ_PER_MWH
        } else {
            0.0
        };
        let scale5 = self.co2_cap_c6 / self.case5_cap;
        let export_mm = if no_recycle {
            self.export_c5
        } else {
            self.export_c6
        };
        let flag = |on: bool| if on { 1.0 } else { 0.0 };
        let stack = Stack {
            selexol: self.q_selexol_c5 * scale5 * flag(o.use_selexol),
            export: mmbtu_h_to_mw(export_mm) * flag(o.use_export),
            ex4: self.q_ex4_above130 * scale5 * flag(o.use_ex4),
            zld: self.q_zld_c5 * scale5 * flag(o.use_zld),
        };
        // Approved order: freed Selexol steam, export steam, syngas, ZLD, reactor.
        let alloc = heatrec::allocate(q_reb, &stack);
        let avail = alloc.avail;
        let (gap, export_kept) = if scn == Scenario::InlSel {
            (0.0, self.export_c6)
        } else {
            let gap = if o.capture_energy { alloc.gap } else { 0.0 };
            (
                gap,
                export_mm - mw_to_mmbtu_h(alloc.from(heatrec::SourceId::Export)),
            )
        };

        let mut e = EUse {
            reforming: self.e_rows.reforming * c.ng / self.ng_c6,
            purification: if scn == Scenario::InlSel {
                self.e_rows.purification
            } else {
                self.e_psa
            },
            steam: self.e_rows.steam * if no_recycle { t_ratio } else { 1.0 },
            co2comp: if o.capture_energy {
                self.e_rows.co2comp * c.cap / st_d_to_kmol_h(self.co2_cap_c6)
            } else {
                0.0
            },
            purge: 0.0,
            cooling: self.e_rows.cooling,
            water: self.e_rows.water,
            mea_pump: 0.0,
            mea_cooling: 0.0,
        };
        if scn.recycled_mea() {
            e.purge = self.e_rows.purge * (1.0 + extra_co2 / self.psa_reject_kmol_h);
        } else if scn == Scenario::InlSel {
            e.purge = self.e_rows.purge;
        }
        if let Some(v) = o.purge_override {
            e.purge = v;
        }
        if mea && o.capture_energy {
            let solv_kg_s = c.cap / (self.rich - self.lean) * MW_MEA / self.w_mea / 3600.0;
            e.mea_pump = solv_kg_s / self.rho_solv * (self.p_abs - self.p_strip) * 1e5
                / self.eta_pump
                / self.eta_motor
                / 1e6
                * self.scale_t07;
            e.mea_cooling = self.k_cool * q_reb * self.scale_t08;
        }
        let e_proc = e.reforming
            + e.purification
            + e.steam
            + e.co2comp
            + e.purge
            + e.cooling
            + e.water
            + e.mea_pump
            + e.mea_cooling;

        // Scale extensive process items.
        let q_proc_s = q_proc * s;
        let q_recyc_s = q_recyc * s;
        let e_proc_s = e_proc * s;
        let q_reb_s = q_reb * s;
        let gap_s = gap * s;
        let avail_s = avail * s;

        let mut lhv = 0.0;
        let mut q_tg = 0.0;
        if matches!(scn, Scenario::CombH | Scenario::CombN) {
            lhv = self.tail_gas_lhv(&c);
            q_tg = lhv * self.eta_boiler * s;
        }
        if scn == Scenario::Export {
            lhv = self.tail_gas_lhv(&c);
        }
        let gap_after_tg = (gap_s - q_tg).max(0.0);
        let tg_left = (q_tg - gap_s).max(0.0);

        let l = self.loss_set(o.loss);
        let q_ihx = l.ihx * (q_proc_s + q_recyc_s);
        let q_duct = l.duct * s;
        let circ_par = self.circ_set(o.circ);

        let solve = |rg: f64| -> (f64, SolveState) {
            let (w_s, w_p) = match circ_par {
                None => (0.0, 0.0),
                Some(cp) => {
                    let md_s = self.he_flow * (q_proc_s + q_recyc_s) / self.q_proc;
                    let md_p =
                        rg * 1e6 / self.cp_he / (self.rot - (self.t_he_r + self.ihx_approach));
                    let w_s = self.circ_shaft(md_s, self.t_circ_s, cp.p_mpa, cp.dp_sec_kpa, cp.eta);
                    let w_p = if o.eta_net_of_primary_circ {
                        0.0
                    } else {
                        self.circ_shaft(md_p, self.t_circ_p, cp.p_mpa, cp.dp_prim_kpa, cp.eta)
                    };
                    (w_s, w_p)
                }
            };
            let e_circ = circulator::electric(w_s, w_p, self.eta_motor);
            let e_total = utility::e_total(e_proc_s, e_circ);
            let e_tg = e_total.min(tg_left * self.eta_tg);
            let e_nuc = e_total - e_tg;
            let branch = heatrec::extraction(gap_after_tg, e_nuc, o.r_work, o.eta_p, o.route);
            let qv = self.vessel_loss(l.vessel, rg);
            let rn = utility::reactor_required(&ReactorTerms {
                q_proc: q_proc_s,
                q_recyc: q_recyc_s,
                q_ihx,
                q_duct,
                qv,
                branch,
                w_s,
                w_p,
            });
            (
                rn,
                SolveState {
                    w_s,
                    w_p,
                    e_circ,
                    e_total,
                    e_tg,
                    e_nuc,
                    branch,
                    qv,
                },
            )
        };
        let (rg, det) = fixed_point(220.0 * s, 1e-9, 200, solve)?;
        let tg_surplus = (tg_left - det.e_tg / self.eta_tg).max(0.0);

        Ok(Balance {
            scn,
            s,
            srd: o.srd,
            q_proc: q_proc_s,
            q_recyc: q_recyc_s,
            q_reb: q_reb_s,
            avail: avail_s,
            gap: gap_s,
            stack: Stack {
                selexol: stack.selexol * s,
                export: stack.export * s,
                ex4: stack.ex4 * s,
                zld: stack.zld * s,
            },
            q_ihx,
            q_duct,
            e_rows: EUse {
                reforming: e.reforming * s,
                purification: e.purification * s,
                steam: e.steam * s,
                co2comp: e.co2comp * s,
                purge: e.purge * s,
                cooling: e.cooling * s,
                water: e.water * s,
                mea_pump: e.mea_pump * s,
                mea_cooling: e.mea_cooling * s,
            },
            e_proc: e_proc_s,
            r: rg,
            q_tg,
            lhv: lhv * s,
            export_kept: export_kept * s,
            tg_surplus,
            gap_after_tg,
            cap_t: cap_t * s,
            c,
            w_s: det.w_s,
            w_p: det.w_p,
            e_circ: det.e_circ,
            e_total: det.e_total,
            e_tg: det.e_tg,
            e_nuc: det.e_nuc,
            branch: det.branch,
            qv: det.qv,
        })
    }

    /// MWe import needed if the reactor is capped at `r_fixed`, heat duties first.
    /// Returns the import and the heat-first demand.
    pub fn shortfall_at(&self, r_fixed: f64, b: &Balance) -> (f64, f64) {
        let heat_first =
            b.q_proc + b.q_recyc + b.q_ihx + b.q_duct + b.qv + b.gap_after_tg - (b.w_s + b.w_p);
        (
            utility::import(b.e_nuc, r_fixed, heat_first, self.eta_pwr),
            heat_first,
        )
    }

    /// Antoine saturation pressure in bar, valid from 99 to 374 C.
    pub fn psat_bar(&self, tc: f64) -> f64 {
        10f64.powf(self.ex4.ant_a - self.ex4.ant_b / (self.ex4.ant_c + tc)) * self.ex4.ant_bar
    }

    /// MW from 149 C to `tmin` on the Case 5 gas, an independent check of the
    /// recoverable heat in the WGS-EX4 cooler.
    pub fn ex4_heat_above(&self, tmin: f64) -> f64 {
        let x = &self.ex4;
        let nv = |t: f64| {
            let y = self.psat_bar(t) / x.p_bar;
            x.dry * y / (1.0 - y)
        };
        let n0 = nv(x.t_in);
        let n1 = nv(tmin);
        (x.dry * x.cp_dry * (x.t_in - tmin)
            + 0.5 * (n0 + n1) * x.cp_v * (x.t_in - tmin)
            + (n0 - n1) * nush2_props::units::MW_H2O * x.latent)
            / 3600.0
            / 1000.0
    }

    /// Scalar summary of Case A or B on the headline scenario, keyed as the
    /// freeze tables are.
    pub fn case_summary(&self, scn: Scenario, s: f64) -> Result<BTreeMap<&'static str, f64>> {
        let o = self.opts(s);
        let b = self.balance(scn, &o)?;
        let c = b.c;
        let e = self.emissions(&c, s);
        let h2_kgh = self.h2_a_kgh * s;
        let h2_t_y = h2_kgh * self.hours / 1000.0;
        let gj_per_mmscf = self.ng_hhv * MMBTU_GJ;
        let mut k: BTreeMap<&'static str, f64> = BTreeMap::new();
        k.insert("s", s);
        k.insert("h2_kg_h", h2_kgh);
        k.insert("h2_t_y", h2_t_y);
        k.insert("h2_nm3_h", h2_kgh / self.rho_h2n);
        k.insert("hours", self.hours);
        k.insert("days", self.days);
        k.insert("ng_mmscfd", self.ng_c6 * s);
        k.insert("ng_unabated_mmscfd", self.ng_conv * s);
        k.insert("ng_norecycle_mmscfd", self.ng_norec * s);
        k.insert("ng_saved_mmscfd", (self.ng_conv - self.ng_c6) * s);
        k.insert(
            "ng_gj_d_saved",
            (self.ng_conv - self.ng_c6) * s * gj_per_mmscf,
        );
        k.insert(
            "gas_saving_SM_y",
            (self.ng_conv - self.ng_c6) * s * gj_per_mmscf * self.days * self.gas.central / 1e6,
        );
        k.insert("feed_C_kmol_h", self.f_c6 * s);
        k.insert("tube_C_kmol_h", c.t * s);
        k.insert("co2_abs_kmol_h", c.co2_abs * s);
        k.insert("cap_kmol_h", c.cap * s);
        k.insert("slip_kmol_h", c.slip * s);
        k.insert("tailgas_C_kmol_h", c.g * s);
        k.insert("recycled_C_kmol_h", c.r * s);
        k.insert("burned_C_kmol_h", c.burn * s);
        k.insert("product_C_kmol_h", c.p * s);
        k.insert("cap_t_h", b.cap_t);
        k.insert("baseline_t_y", e.baseline);
        k.insert("captured_t_y", e.captured);
        k.insert("atm_t_y", e.atm);
        k.insert("slip_t_y", self.tpy(c.slip, s));
        k.insert("tailgas_t_y", self.tpy(c.g, s));
        k.insert("recycled_t_y", self.tpy(c.r, s));
        k.insert("A_t_y", e.a_atm);
        k.insert("plant_capture", e.captured / (e.captured + e.atm));
        k.insert("reduction", e.a_atm / e.baseline);
        k.insert("ci_kg_kg", e.atm / h2_t_y);
        k.insert("baseline_kg_kg", e.baseline / h2_t_y);
        k.insert("never_generated_t_y", e.baseline - e.captured - e.atm);
        k.insert("A_cho", self.base_cho * h2_t_y - e.atm);
        k.insert("A_ieaghg", self.base_ieaghg * h2_t_y - e.atm);
        k.insert("A_900", self.base_ieaghg_rounded * h2_t_y - e.atm);
        k.insert("base_cho", self.base_cho * h2_t_y);
        k.insert("base_ieaghg", self.base_ieaghg * h2_t_y);
        k.insert("q_proc", b.q_proc);
        k.insert("q_recyc", b.q_recyc);
        k.insert("q_reb", b.q_reb);
        k.insert("avail", b.avail);
        k.insert("gap", b.gap);
        k.insert("q_ihx", b.q_ihx);
        k.insert("q_duct", b.q_duct);
        k.insert("qv", b.qv);
        k.insert("w_s", b.w_s);
        k.insert("w_p", b.w_p);
        k.insert("e_proc", b.e_proc);
        k.insert("e_circ", b.e_circ);
        k.insert("e_total", b.e_total);
        k.insert("branch", b.branch);
        k.insert("R", b.r);
        k.insert("circ_sec_MWe", b.w_s / self.eta_motor);
        k.insert("circ_prim_MWe", b.w_p / self.eta_motor);
        k.insert(
            "he_sec_kg_s",
            self.he_flow * (b.q_proc + b.q_recyc) / self.q_proc,
        );
        let dt_prim = self.rot - (self.t_he_r + self.ihx_approach);
        k.insert("he_prim_kg_s_at_R", b.r * 1e6 / self.cp_he / dt_prim);
        k.insert(
            "he_prim_kg_s_at_210",
            self.r_rated * 1e6 / self.cp_he / dt_prim,
        );
        k.insert(
            "solvent_t_h",
            c.cap * s / (self.rich - self.lean) * MW_MEA / self.w_mea / 1000.0,
        );
        k.insert("water_gpm", self.water_gpm_a * s);
        k.insert("export_steam_used_all", b.export_kept);
        k.insert("margin_210", self.r_rated - b.r);
        let best = BalanceOpts {
            srd: self.srd.low,
            circ: CircKey::Low,
            loss: LossKey::Low,
            r_work: self.r_work_low_r,
            use_zld: true,
            eta_p: self.eta_pwr_alt,
            ..o
        };
        k.insert("R_best", self.balance(scn, &best)?.r);
        let high = BalanceOpts {
            srd: self.srd.high,
            circ: CircKey::High,
            loss: LossKey::High,
            route: Route::Direct,
            ..o
        };
        k.insert("R_high", self.balance(scn, &high)?.r);
        k.insert("psa_h2_feed_kg_h", h2_kgh / self.psa_rec);
        k.insert("psa_h2_reject_kg_h", h2_kgh / self.psa_rec - h2_kgh);
        k.insert("psa_h2_feed_kmol_h", h2_kgh / MW_H2 / self.psa_rec);
        Ok(k)
    }

    /// Sizing walk at scale `s`, Case B form of freeze section 5.
    pub fn sizing_walk(&self, s: f64) -> Result<[f64; 7]> {
        let e_all = self.e_rows.sum();
        let e_no_purge = e_all - self.e_rows.purge;
        let o = self.opts(s);
        let no_circ = BalanceOpts {
            circ: CircKey::None,
            ..o
        };
        let zero_reb = BalanceOpts { srd: 1e-4, ..o };
        Ok([
            self.q_proc + e_no_purge / self.eta_pwr,
            self.q_proc * s + e_no_purge * s / self.eta_pwr,
            self.q_proc * s + e_all * s / self.eta_pwr,
            self.balance(Scenario::InlSel, &no_circ)?.r,
            self.balance(Scenario::InlSel, &o)?.r,
            self.balance(Scenario::RecF, &zero_reb)?.r,
            self.balance(Scenario::RecF, &o)?.r,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn availability_out_of_range_names_the_parameter() {
        let mut p = ParamSet::embedded().unwrap();
        p.set("AVAILABILITY", 1.5).unwrap();
        match Model::new(&p) {
            Err(EngineError::Range { param, .. }) => assert_eq!(param, "AVAILABILITY"),
            other => panic!("expected a range error, got {other:?}"),
        }
    }

    #[test]
    fn crf_refuses_zero_life() {
        assert!(crf(0.08, 0.0).is_err());
        assert!(sff(0.0, 25.0).is_err());
    }

    #[test]
    fn reboiler_order_is_approved() {
        // Partial demand: SRD low enough that Selexol steam covers part of the
        // reboiler. Export must go next, before syngas heat.
        let m = Model::embedded().unwrap();
        let mut o = m.opts(1.0);
        let c = m.carbon(Scenario::RecF, o.phi, o.eta_c);
        let cap_t = kmol_h_to_t_h(c.cap);
        let selexol = m.q_selexol_c5 * m.co2_cap_c6 / m.case5_cap;
        // Demand equals Selexol steam plus 5 MW.
        o.srd = (selexol + 5.0) * GJ_PER_MWH / cap_t;
        let b = m.balance(Scenario::RecF, &o).unwrap();
        let used = m.export_c6 - b.export_kept;
        assert!(
            (mmbtu_h_to_mw(used) - 5.0).abs() < 1e-9,
            "export used {used}"
        );
        assert_eq!(b.gap, 0.0);
        // Demand beyond Selexol and export spills to syngas, then the reactor.
        o.srd = (selexol + b.stack.export + 2.0) * GJ_PER_MWH / cap_t;
        let b = m.balance(Scenario::RecF, &o).unwrap();
        assert!(b.export_kept.abs() < 1e-9);
        assert_eq!(b.gap, 0.0);
        o.srd = (selexol + b.stack.export + b.stack.ex4 + 3.0) * GJ_PER_MWH / cap_t;
        let b = m.balance(Scenario::RecF, &o).unwrap();
        assert!((b.gap - 3.0).abs() < 1e-9, "gap {}", b.gap);
    }

    #[test]
    fn eta_net_of_primary_circ_removes_the_primary_row() {
        let m = Model::embedded().unwrap();
        let mut o = m.opts(m.s_b);
        let base = m.balance(Scenario::RecF, &o).unwrap();
        o.eta_net_of_primary_circ = true;
        let net = m.balance(Scenario::RecF, &o).unwrap();
        assert_eq!(net.w_p, 0.0);
        assert!(net.r < base.r);
    }
}
