//! Level 3 flowsheet, milestone M7: streams S01 to S21 around the reformer, the
//! shift train, capture, CO2 compression and the tail-gas loop.
//!
//! Units come from `nush2_units` wherever a model exists at this milestone: the
//! equilibrium reformer and the TEV-961 Case 5 train from HPS-GEN1 to the flash
//! drum. The HTS in that train runs at the dry space velocity of the bed that
//! `wgs::hts` sizes on the IEAGHG outlet approach for the printed Case 5 syngas
//! (see [`hts_train_opts`]). The capture columns sit behind [`CaptureColumns`].
//! Its only implementations here are the anchored islands of the screening
//! balance (MEA and INL's Selexol), so M8 can switch to the M6 column models once
//! their gates pass without touching the rest of the flowsheet.
//!
//! PSA is an unsimulated external block (user answer Q4). Its reject S14 crosses
//! the boundary into REC-CRYO, a fixed-assumption block drawn by INL with three
//! outlets (TEV-961 A-16 PDF p40 and A-18 PDF p42). The CH4-rich recycle S03 goes
//! to the reformer and S21 takes H2 back ahead of the PSA, while the nitrogen-rich
//! cut S15 goes to the burner. The carbon split of the cut is held at INL's
//! fraction (card J.7). The H2 return fraction is unpublished, so it is calculated
//! by closure on INL's natural gas and hydrogen pair at Case A INL_SEL and carried
//! to REC_F.
//!
//! Tears: the recycle gas S03 and the lean loading of S18, solved together by
//! Wegstein with q bounded to [-5, 0] after three direct passes, then damped
//! direct substitution if the budget runs out. The PSA and cryogenic H2 loop is
//! linear in H2, so it is solved in closed form inside each pass.
//!
//! Every unit closes elements and enthalpy on its own. Duties marked "by closure"
//! are the balancing heat of a block whose physics is not modelled at M7
//! (capture island, cryogenic unit, CO2 compressor, N2 burner). The envelope
//! energy check therefore tests the bookkeeping across units and says nothing
//! about those blocks.

use crate::anchored::{Case, Model, Scenario};
use crate::carbon::{carbon_kmol_h, carbon_of, n2_kmol_h, CarbonLedger};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;
use nush2_props::units::{st_d_to_kmol_h, BAR_PA, MW_CO2, MW_MEA, R_GAS};
use nush2_units::case5::{self, TrainOpts};
use nush2_units::smr_eq::{self, SmrEqSpec};
use nush2_units::stream::{
    Boundary, Closure, EnergyKind, EnergyStream, Flag, Flowsheet, ModelClass, Port, SolventStream,
    Sp, Stream, StreamTag, UnitRecord, NSP,
};
use serde::Serialize;
use std::collections::BTreeMap;

/// Pa per psi.
pub const PSI_PA: f64 = 6894.757;
/// Molar mass of O2, kg/mol, for the combustion O2 of N2-BURN.
pub const MW_O2_KG_MOL: f64 = 31.9988e-3;
/// Molar mass of water, kg/mol.
const MW_H2O_KG_MOL: f64 = 18.01528e-3;
/// Atoms C, H, O, N of MEA, C2H7NO.
const MEA_ATOMS: [f64; 4] = [2.0, 7.0, 1.0, 1.0];

const H2: usize = Sp::H2 as usize;
const CO: usize = Sp::Co as usize;
const CO2: usize = Sp::Co2 as usize;
const CH4: usize = Sp::Ch4 as usize;
const H2O: usize = Sp::H2o as usize;
const N2: usize = Sp::N2 as usize;

fn f_to_k(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8 + 273.15
}

/// A set stream state printed by INL for TEV-961 Case 5.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SetState {
    pub t_f: f64,
    pub p_psia: f64,
    pub source: &'static str,
}

impl SetState {
    pub fn t_k(&self) -> f64 {
        f_to_k(self.t_f)
    }
    pub fn p_pa(&self) -> f64 {
        self.p_psia * PSI_PA
    }
}

/// S01 at the battery limit after the NG compressor, NG-2. The compressor load is
/// in the reforming electricity row.
pub const NG_IN: SetState = SetState {
    t_f: 135.0,
    p_psia: 460.0,
    source: "TEV-961 Case 5 NG-2, A-55 PDF p79",
};
/// S02 process steam, STEAM-2.
pub const STEAM_IN: SetState = SetState {
    t_f: 752.0,
    p_psia: 612.0,
    source: "TEV-961 Case 5 STEAM-2, A-55 PDF p79",
};
/// S03 recycle after the recycle compressor, RECY-2.
pub const RECYCLE: SetState = SetState {
    t_f: 327.0,
    p_psia: 460.0,
    source: "TEV-961 Case 5 RECY-2, A-55 PDF p79",
};
/// S04 reformer inlet, NG-10, also the stage 1 inlet of A-52.
pub const REFORMER_IN: SetState = SetState {
    t_f: 1000.0,
    p_psia: 445.0,
    source: "TEV-961 Case 5 NG-10, A-55 PDF p79; stage 1 inlet, A-52 PDF p76",
};
/// Reformer outlet pressure, SG-2, psia.
pub const REFORMER_OUT_P_PSIA: f64 = 439.0;
/// HPS-GEN1 outlet pressure, the first pressure after the reformer, psia. The
/// reformer outlet must lie between this and the inlet pressure.
pub const HPS_GEN1_OUT_P_PSIA: f64 = 436.0;
/// Reformer outlet temperature, C: INL's 871 C (1,600 F) basis for Case 6,
/// TEV-961 PDF p14. 825 C and 800 C run as sensitivities (decision D29).
///
/// eq: D.36
pub const T_REFORMER_OUT_C: f64 = 871.0;
/// Reformer outlet sensitivities, C.
pub const T_REFORMER_SENSITIVITIES_C: [f64; 2] = [825.0, 800.0];
/// Steam to carbon molar ratio at the reformer inlet, Case 5 stage 1, A-52 PDF
/// p76. Case 6 states none, so Case 5 stands in. Carbon here is every carbon atom
/// of the feed and recycle, the reading that closes Case 5 water within 0.3%.
///
/// eq: D.5
pub const S_TO_C: f64 = 3.00;
/// S12 leaves at the absorber state, SG-9.
pub const ABSORBER_OUT: SetState = SetState {
    t_f: 104.0,
    p_psia: 389.0,
    source: "TEV-961 Case 5 SG-9, A-56 PDF p80",
};
/// S21 joins the PSA feed, SG-10.
pub const PSA_FEED: SetState = SetState {
    t_f: 131.0,
    p_psia: 386.0,
    source: "TEV-961 Case 5 SG-10, A-56 PDF p80",
};
/// S13 H2 product.
pub const H2_PRODUCT: SetState = SetState {
    t_f: 131.0,
    p_psia: 376.0,
    source: "TEV-961 Case 5 H2, A-56 PDF p80",
};
/// S14 PSA reject and S15 cut, PURGE.
pub const PSA_REJECT: SetState = SetState {
    t_f: 131.0,
    p_psia: 25.0,
    source: "TEV-961 Case 5 PURGE, A-56 PDF p80",
};
/// S19 CO2 leaving capture, CO2-1.
pub const CO2_LOW: SetState = SetState {
    t_f: 104.0,
    p_psia: 30.0,
    source: "TEV-961 Case 5 CO2-1, A-56 PDF p80",
};
/// S20 compressed CO2, CO2-2.
pub const CO2_HIGH: SetState = SetState {
    t_f: 184.0,
    p_psia: 2015.0,
    source: "TEV-961 Case 5 CO2-2, A-56 PDF p80",
};
/// S16 stack, the Case 5 furnace exhaust EX-3.
pub const STACK: SetState = SetState {
    t_f: 300.0,
    p_psia: 15.0,
    source: "TEV-961 Case 5 EX-3, A-55 PDF p79",
};
/// Solvent temperature used for S17 and S18, K. The anchored island has no
/// solvent energy model, so the absorber temperature of D28 stands in.
const SOLVENT_T_K: f64 = 313.15;

/// Natural gas, mol per mol, TEV-961 A-50 PDF p74 and TEV-953 A-5 PDF p39, with
/// the feed lumping: C6 enters as 1.2 mol C5 per mol, O2 joins N2.
pub const NG_MOL_FRACTIONS: [(Sp, f64); 7] = [
    (Sp::Ch4, 0.93571),
    (Sp::C2h6, 0.03749),
    (Sp::C3h8, 0.00920),
    (Sp::Nc4h10, 0.00260),
    (Sp::Nc5h12, 0.00040 + 1.2 * 0.00010),
    (Sp::N2, 0.01190 + 0.00010),
    (Sp::Co2, 0.00250),
];

/// Tear tolerance on the scaled residual. A limit of 1e-7 is the floor; the
/// flowsheet asks for more so the envelope closes to 1e-9 in elements.
pub const TEAR_TOL: f64 = 1.0e-11;
/// Wegstein pass budget before the damped fallback.
pub const WEGSTEIN_BUDGET: usize = 120;
/// Damped direct substitution budget.
pub const DAMPED_BUDGET: usize = 200;
/// Element closure limit of the envelope.
pub const ELEMENT_TOL: f64 = 1.0e-9;
/// Energy closure limit of the envelope, derivation card A.15.
pub const ENERGY_TOL: f64 = 1.0e-6;

/// Natural gas flow of `mmscfd` at the battery limit, S01.
///
/// eq: J.5
pub fn natural_gas(mmscfd: f64) -> Result<Stream> {
    if !(mmscfd > 0.0 && mmscfd.is_finite()) {
        return Err(EngineError::Range {
            param: "ng_mmscfd".into(),
            msg: format!("{mmscfd} MMSCFD must be positive"),
        });
    }
    let mol_s = nush2_props::units::mmscfd_to_kmol_h(mmscfd) / 3.6;
    let mut n = [0.0; NSP];
    for (sp, y) in NG_MOL_FRACTIONS {
        n[sp as usize] += y * mol_s;
    }
    Ok(Stream::gas("S01", NG_IN.t_k(), NG_IN.p_pa(), n))
}

/// Carbon per mol of natural gas in the lumped feed.
pub fn ng_carbon_per_mol() -> f64 {
    let mut n = [0.0; NSP];
    for (sp, y) in NG_MOL_FRACTIONS {
        n[sp as usize] += y;
    }
    carbon_of(&n)
}

/// Share of the tail-gas carbon that INL recycles, from the published Case 6
/// capture through the anchored ledger.
///
/// eq: J.7
pub fn recycle_fraction(m: &Model) -> f64 {
    let c = m.carbon(Scenario::InlSel, m.phi_rec, m.eta_mea);
    c.r / c.g
}

/// Reaction count of a species set: species less the rank of the atom matrix.
///
/// eq: A.6
pub fn reaction_rank(species: &[Sp]) -> Result<usize> {
    let mut a: Vec<[f64; 4]> = Vec::with_capacity(species.len());
    for s in species {
        let mut n = [0.0; NSP];
        n[*s as usize] = 1.0;
        a.push(nush2_props::mixture::elements(&n)?);
    }
    let mut rank = 0;
    for col in 0..4 {
        let pivot = (rank..a.len()).find(|&r| a[r][col].abs() > 1e-12);
        if let Some(p) = pivot {
            a.swap(rank, p);
            let row = a[rank];
            for (r, other) in a.iter_mut().enumerate() {
                if r != rank {
                    let f = other[col] / row[col];
                    for (v, pv) in other.iter_mut().zip(row) {
                        *v -= f * pv;
                    }
                }
            }
            rank += 1;
        }
    }
    Ok(species.len() - rank)
}

/// How the feed is fixed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum FeedBasis {
    /// Natural gas at the battery limit, MMSCFD.
    NaturalGas(f64),
    /// H2 in the PSA product, kg/h. Natural gas is solved.
    Hydrogen(f64),
}

/// Where the tear iteration starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TearStart {
    /// No recycle and the anchored lean loading.
    Zero,
    /// Recycle carbon of the anchored ledger, split CH4 and CO by its CO share.
    Anchored,
    /// Twice the anchored start.
    Double,
}

/// Inputs of one flowsheet run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlowsheetSpec {
    pub case: Case,
    pub scenario: Scenario,
    pub basis: FeedBasis,
    pub t_reformer_out_c: f64,
    pub p_reformer_out_psia: f64,
    pub s_to_c: f64,
    #[serde(skip)]
    pub k_source: KSource,
    /// Share of the PSA reject H2 that REC-CRYO returns ahead of the PSA.
    pub h2_return: f64,
    /// Share of the reject carbon that REC-CRYO recycles to the reformer.
    pub carbon_recycle: f64,
    pub psa_recovery: f64,
    pub ch4_in_product: f64,
    /// Solve S18 with the S03 tear. False fixes the lean loading, the fallback of
    /// the tear solve.
    pub solvent_tear: bool,
    /// Route the reject H2 that REC-CRYO does not return to the PSA into the
    /// recycle S03 instead of the nitrogen-rich cut. False is the default split;
    /// true is the one recorded sensitivity of decision S11.
    pub h2_rest_to_recycle: bool,
}

impl FlowsheetSpec {
    /// Defaults on the model: 871 C, S/C 3.00, INL's carbon split, PSA as
    /// published. `h2_return` must come from [`close_h2_return`].
    pub fn new(
        m: &Model,
        case: Case,
        scenario: Scenario,
        basis: FeedBasis,
        h2_return: f64,
    ) -> Self {
        Self {
            case,
            scenario,
            basis,
            t_reformer_out_c: T_REFORMER_OUT_C,
            p_reformer_out_psia: REFORMER_OUT_P_PSIA,
            s_to_c: S_TO_C,
            k_source: KSource::Zhang,
            h2_return,
            carbon_recycle: recycle_fraction(m),
            psa_recovery: m.psa_rec,
            ch4_in_product: m.ch4_in_h2,
            solvent_tear: true,
            h2_rest_to_recycle: false,
        }
    }

    /// Range checks. Never clamps.
    pub fn validate(&self, m: &Model) -> Result<()> {
        let t_in_c = REFORMER_IN.t_k() - 273.15;
        let t = self.t_reformer_out_c;
        if !(t > t_in_c && t < m.t_he_s && t.is_finite()) {
            return Err(EngineError::Range {
                param: "t_reformer_out_c".into(),
                msg: format!(
                    "{t} C must lie above the reformer inlet {t_in_c:.1} C and below the \
                     secondary helium supply {} C",
                    m.t_he_s
                ),
            });
        }
        let p = self.p_reformer_out_psia;
        if !(HPS_GEN1_OUT_P_PSIA..=REFORMER_IN.p_psia).contains(&p) {
            return Err(EngineError::Range {
                param: "p_reformer_out_psia".into(),
                msg: format!(
                    "{p} psia must lie between the HPS-GEN1 outlet {HPS_GEN1_OUT_P_PSIA} psia \
                     and the reformer inlet {} psia",
                    REFORMER_IN.p_psia
                ),
            });
        }
        if !(self.s_to_c > 0.0 && self.s_to_c.is_finite()) {
            return Err(EngineError::Range {
                param: "s_to_c".into(),
                msg: format!("{} must be positive", self.s_to_c),
            });
        }
        for (name, v) in [
            ("h2_return", self.h2_return),
            ("carbon_recycle", self.carbon_recycle),
        ] {
            if !(0.0..=1.0).contains(&v) {
                return Err(EngineError::Range {
                    param: name.into(),
                    msg: format!("{v} must lie between 0 and 1"),
                });
            }
        }
        for (name, v) in [
            ("psa_recovery", self.psa_recovery),
            ("ch4_in_product", self.ch4_in_product),
        ] {
            if !(v > 0.0 && v < 1.0) {
                return Err(EngineError::Range {
                    param: name.into(),
                    msg: format!("{v} must lie strictly between 0 and 1"),
                });
            }
        }
        match self.scenario {
            Scenario::InlSel | Scenario::RecF => {}
            other => {
                return Err(EngineError::Range {
                    param: "scenario".into(),
                    msg: format!("{} has no Level 3 flowsheet at M7", other.id()),
                })
            }
        }
        match self.basis {
            FeedBasis::NaturalGas(v) | FeedBasis::Hydrogen(v) if v > 0.0 && v.is_finite() => Ok(()),
            _ => Err(EngineError::Range {
                param: "basis".into(),
                msg: "the feed basis must be positive".into(),
            }),
        }
    }
}

/// Outcome of the capture island in one pass.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureOut {
    /// S12.
    pub treated: Stream,
    /// S19.
    pub co2: Stream,
    /// S17, absent for a physical solvent block.
    pub rich: Option<SolventStream>,
    /// S18 entering the absorber in this pass.
    pub lean_in: Option<SolventStream>,
    /// S18 leaving the stripper in this pass.
    pub lean_out: Option<SolventStream>,
    /// Heat into the island, W.
    pub q_in_w: f64,
    /// Electricity into the island, W.
    pub w_in_w: f64,
    pub records: Vec<UnitRecord>,
}

/// Capture columns behind one interface. M8 switches the implementation to the
/// M6 absorber and stripper once their gates pass.
pub trait CaptureColumns {
    /// Short id printed in flags.
    fn id(&self) -> &'static str;
    /// True when S17 and S18 exist and S18 is a tear.
    fn solvent_loop(&self) -> bool;
    /// Lean loading the stripper delivers, the start of the S18 tear.
    fn lean_start(&self) -> f64;
    /// Solve the island on absorber feed `feed` and lean loading `lean_alpha`.
    fn solve(&self, feed: &Stream, lean_alpha: f64) -> Result<CaptureOut>;
}

/// Anchored MEA island: per-pass capture, loadings, SRD and the screening load
/// rows, applied to the flowsheet's own CO2.
#[derive(Debug, Clone, PartialEq)]
pub struct AnchoredMea {
    pub eta: f64,
    pub srd_gj_t: f64,
    pub rich: f64,
    pub lean: f64,
    pub w_mea: f64,
    pub rho_kg_m3: f64,
    pub p_abs_bar: f64,
    pub p_strip_bar: f64,
    pub eta_pump: f64,
    pub eta_motor: f64,
    pub k_cool: f64,
}

impl AnchoredMea {
    pub fn from_model(m: &Model) -> Self {
        Self {
            eta: m.eta_mea,
            srd_gj_t: m.srd.central,
            rich: m.rich,
            lean: m.lean,
            w_mea: m.w_mea,
            rho_kg_m3: m.rho_solv,
            p_abs_bar: m.p_abs,
            p_strip_bar: m.p_strip,
            eta_pump: m.eta_pump,
            eta_motor: m.eta_motor,
            k_cool: m.k_cool,
        }
    }

    /// Reboiler duty for `co2_mol_s` released, W.
    ///
    /// eq: J.16
    pub fn reboiler_w(&self, co2_mol_s: f64) -> f64 {
        co2_mol_s * MW_CO2 * 1.0e-3 * self.srd_gj_t * 1.0e6
    }

    /// Lean pump electricity for `n_mea` mol/s of MEA in circulation, W.
    ///
    /// eq: J.20
    pub fn pump_w(&self, n_mea: f64) -> f64 {
        let solvent_kg_s = n_mea * MW_MEA * 1.0e-3 / self.w_mea;
        solvent_kg_s / self.rho_kg_m3 * (self.p_abs_bar - self.p_strip_bar) * BAR_PA
            / self.eta_pump
            / self.eta_motor
    }

    /// Cooling fans, W.
    ///
    /// eq: J.21
    pub fn fans_w(&self, q_reb_w: f64) -> f64 {
        self.k_cool * q_reb_w
    }
}

fn solvent(tag: &str, p_pa: f64, n_mea: f64, n_h2o: f64, n_co2: f64) -> SolventStream {
    SolventStream {
        tag: tag.to_string(),
        t_k: SOLVENT_T_K,
        p_pa,
        n_mea,
        n_h2o,
        n_co2,
    }
}

impl CaptureColumns for AnchoredMea {
    fn id(&self) -> &'static str {
        "anchored MEA island"
    }
    fn solvent_loop(&self) -> bool {
        true
    }
    fn lean_start(&self) -> f64 {
        self.lean
    }
    fn solve(&self, feed: &Stream, lean_alpha: f64) -> Result<CaptureOut> {
        if !(lean_alpha >= 0.0 && lean_alpha < self.rich) {
            return Err(EngineError::Range {
                param: "lean_alpha".into(),
                msg: format!("{lean_alpha} must lie between 0 and the rich loading"),
            });
        }
        let cap = self.eta * feed.n[CO2];
        let mut gas = feed.n;
        gas[CO2] -= cap;
        let treated = Stream::gas("S12", ABSORBER_OUT.t_k(), ABSORBER_OUT.p_pa(), gas);
        // The absorber sets circulation so the rich loading meets its cap.
        let n_mea = cap / (self.rich - lean_alpha);
        let n_h2o = n_mea * MW_MEA * 1.0e-3 * (1.0 - self.w_mea) / self.w_mea / MW_H2O_KG_MOL;
        let p_abs = self.p_abs_bar * BAR_PA;
        let lean_in = solvent("S18", p_abs, n_mea, n_h2o, lean_alpha * n_mea);
        let rich = solvent("S17", p_abs, n_mea, n_h2o, lean_alpha * n_mea + cap);
        let lean_out = solvent("S18", p_abs, n_mea, n_h2o, self.lean * n_mea);
        let released = rich.n_co2 - lean_out.n_co2;
        let mut c = [0.0; NSP];
        c[CO2] = released;
        let co2 = Stream::gas("S19", CO2_LOW.t_k(), CO2_LOW.p_pa(), c);
        let q_reb = self.reboiler_w(released);
        let w_pump = self.pump_w(n_mea);
        let w_fans = self.fans_w(q_reb);
        let abs_rec = UnitRecord {
            tag: "ABS".into(),
            class: ModelClass::FixedAssumption,
            boundary: Boundary::Inside,
            inlets: vec!["S11".into(), "S18".into()],
            outlets: vec!["S12".into(), "S17".into()],
            duties: Vec::new(),
            closure: solvent_closure(&[feed], &[&lean_in], &[&treated], &[&rich])?,
            flags: vec![
                format!(
                    "anchored per-pass capture {:.3}; gate M6 not passed",
                    self.eta
                ),
                "energy closed on the capture island by a calculated cooling duty; \
                 solvent enthalpy is not modelled at M7"
                    .into(),
            ],
        };
        let strip_rec = UnitRecord {
            tag: "STRIP".into(),
            class: ModelClass::FixedAssumption,
            boundary: Boundary::Inside,
            inlets: vec!["S17".into()],
            outlets: vec!["S18".into(), "S19".into()],
            duties: vec![
                energy("Q_REB", EnergyKind::Heat, q_reb, "STEAM_SYSTEM", "STRIP"),
                energy("W_LEAN_PUMP", EnergyKind::Electric, w_pump, "GRID", "STRIP"),
                energy("W_MEA_FANS", EnergyKind::Electric, w_fans, "GRID", "STRIP"),
            ],
            closure: solvent_closure(&[], &[&rich], &[&co2], &[&lean_out])?,
            flags: vec![format!(
                "anchored SRD {:.2} GJ/t and lean loading {:.2}; gate M6 not passed",
                self.srd_gj_t, self.lean
            )],
        };
        Ok(CaptureOut {
            treated,
            co2,
            rich: Some(rich),
            lean_in: Some(lean_in),
            lean_out: Some(lean_out),
            q_in_w: q_reb,
            w_in_w: w_pump + w_fans,
            records: vec![abs_rec, strip_rec],
        })
    }
}

/// INL's Selexol block as published: 99% capture and the purification row.
#[derive(Debug, Clone, PartialEq)]
pub struct AnchoredSelexol {
    pub eta: f64,
    /// Electricity, W.
    pub w_e: f64,
}

impl CaptureColumns for AnchoredSelexol {
    fn id(&self) -> &'static str {
        "INL Selexol block"
    }
    fn solvent_loop(&self) -> bool {
        false
    }
    fn lean_start(&self) -> f64 {
        0.0
    }
    fn solve(&self, feed: &Stream, _lean_alpha: f64) -> Result<CaptureOut> {
        let cap = self.eta * feed.n[CO2];
        let mut gas = feed.n;
        gas[CO2] -= cap;
        let treated = Stream::gas("S12", ABSORBER_OUT.t_k(), ABSORBER_OUT.p_pa(), gas);
        let mut c = [0.0; NSP];
        c[CO2] = cap;
        let co2 = Stream::gas("S19", CO2_LOW.t_k(), CO2_LOW.p_pa(), c);
        let rec = UnitRecord {
            tag: "SELEXOL".into(),
            class: ModelClass::FixedAssumption,
            boundary: Boundary::Inside,
            inlets: vec!["S11".into()],
            outlets: vec!["S12".into(), "S19".into()],
            duties: vec![energy(
                "W_SELEXOL",
                EnergyKind::Electric,
                self.w_e,
                "GRID",
                "SELEXOL",
            )],
            closure: solvent_closure(&[feed], &[], &[&treated, &co2], &[])?,
            flags: vec![
                "INL separation block with a fixed split, TEV-961 A-53 PDF p77; steam use \
                 not modelled"
                    .into(),
            ],
        };
        Ok(CaptureOut {
            treated,
            co2,
            rich: None,
            lean_in: None,
            lean_out: None,
            q_in_w: 0.0,
            w_in_w: self.w_e,
            records: vec![rec],
        })
    }
}

fn energy(tag: &str, kind: EnergyKind, w: f64, from: &str, to: &str) -> EnergyStream {
    EnergyStream {
        tag: tag.to_string(),
        kind,
        w,
        t_hot_k: None,
        t_cold_k: None,
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// Element flows C, H, O, N of a gas stream, mol/s. An empty stream has none.
fn gas_elements(s: &Stream) -> Result<[f64; 4]> {
    if s.n.iter().all(|v| *v == 0.0) {
        return Ok([0.0; 4]);
    }
    s.elements()
}

/// Enthalpy flow of a gas stream, W. An empty stream carries none.
fn gas_h(s: &Stream) -> Result<f64> {
    if s.n.iter().all(|v| *v == 0.0) {
        return Ok(0.0);
    }
    s.h_w()
}

/// Element flows C, H, O, N of a solvent stream, mol/s.
fn solvent_elements(s: &SolventStream) -> [f64; 4] {
    let mut e = [0.0; 4];
    for k in 0..4 {
        e[k] = MEA_ATOMS[k] * s.n_mea;
    }
    e[0] += s.n_co2;
    e[2] += 2.0 * s.n_co2 + s.n_h2o;
    e[1] += 2.0 * s.n_h2o;
    e
}

fn rel_gap(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    let mut r = [0.0; 4];
    for k in 0..4 {
        let scale = a[k].abs().max(b[k].abs());
        r[k] = if scale > 0.0 {
            (a[k] - b[k]).abs() / scale
        } else {
            0.0
        };
    }
    r
}

/// Element closure of a unit with gas and solvent streams; energy is closed at
/// the island level and reported there.
fn solvent_closure(
    gas_in: &[&Stream],
    solv_in: &[&SolventStream],
    gas_out: &[&Stream],
    solv_out: &[&SolventStream],
) -> Result<Closure> {
    let mut a = [0.0; 4];
    let mut b = [0.0; 4];
    for s in gas_in {
        add4(&mut a, &gas_elements(s)?);
    }
    for s in solv_in {
        add4(&mut a, &solvent_elements(s));
    }
    for s in gas_out {
        add4(&mut b, &gas_elements(s)?);
    }
    for s in solv_out {
        add4(&mut b, &solvent_elements(s));
    }
    Ok(Closure {
        element_rel: rel_gap(&a, &b),
        energy_rel: 0.0,
    })
}

fn add4(a: &mut [f64; 4], b: &[f64; 4]) {
    for k in 0..4 {
        a[k] += b[k];
    }
}

/// Closure of a gas block with a net heat and work input, `q_in_w`.
fn block_closure(ins: &[&Stream], outs: &[&Stream], q_in_w: f64, o2_in: f64) -> Result<Closure> {
    let mut a = [0.0; 4];
    let mut b = [0.0; 4];
    let (mut h_in, mut h_out) = (q_in_w, 0.0);
    for s in ins {
        add4(&mut a, &gas_elements(s)?);
        h_in += gas_h(s)?;
    }
    a[2] += 2.0 * o2_in;
    for s in outs {
        add4(&mut b, &gas_elements(s)?);
        h_out += gas_h(s)?;
    }
    let scale = h_in.abs().max(h_out.abs()).max(1.0);
    Ok(Closure {
        element_rel: rel_gap(&a, &b),
        energy_rel: (h_in - h_out).abs() / scale,
    })
}

/// Where a boundary stream comes from or goes to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BoundaryLink {
    pub stream: StreamTag,
    /// Boundary source, when no unit makes the stream.
    pub from: Option<&'static str>,
    /// Boundary sink, when no unit takes the stream.
    pub to: Option<&'static str>,
}

/// One duty with its sign on the process: positive into the process.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Duty {
    pub tag: String,
    pub unit: String,
    pub kind: EnergyKind,
    pub w_into_process: f64,
    pub by_closure: bool,
}

/// Iteration record of the tears.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TearLog {
    pub passes: usize,
    pub residual: f64,
    pub method: String,
    pub solvent_tear: bool,
    pub history: Vec<f64>,
}

/// One converged run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlowsheetRun {
    pub id: String,
    pub spec: FlowsheetSpec,
    pub ng_mmscfd: f64,
    pub flowsheet: Flowsheet,
    pub boundary: Vec<BoundaryLink>,
    pub duties: Vec<Duty>,
    /// Combustion O2 of the burner, mol/s. Air nitrogen and excess air are not
    /// tracked, because O2 is not a stream species.
    pub air_o2_mol_s: f64,
    pub h2_product_kg_h: f64,
    pub carbon: CarbonLedger,
    pub tear: TearLog,
    pub flags: Vec<Flag>,
}

impl FlowsheetRun {
    /// Gas stream by tag.
    pub fn gas(&self, tag: &str) -> Result<&Stream> {
        match self.flowsheet.streams.get(tag) {
            Some(Port::Gas(s)) => Ok(s),
            _ => Err(EngineError::Range {
                param: "stream".into(),
                msg: format!("{tag} is not a gas stream of run {}", self.id),
            }),
        }
    }

    /// Solvent stream by tag.
    pub fn solvent(&self, tag: &str) -> Option<&SolventStream> {
        match self.flowsheet.streams.get(tag) {
            Some(Port::Solvent(s)) => Some(s),
            _ => None,
        }
    }

    /// Duty by tag, W into the process.
    pub fn duty(&self, tag: &str) -> Option<f64> {
        self.duties
            .iter()
            .find(|d| d.tag == tag)
            .map(|d| d.w_into_process)
    }

    /// Process heat: feed preheat plus reformer duty, MW. The replacing model of
    /// T03 names this sum.
    pub fn process_heat_mw(&self) -> f64 {
        (self.duty("Q_FEED_PREHEAT").unwrap_or(0.0) + self.duty("Q_SMR").unwrap_or(0.0)) / 1.0e6
    }

    /// Captured CO2, kmol/h.
    pub fn captured_kmol_h(&self) -> f64 {
        self.carbon.captured
    }
}

/// Fixed context of one solve.
struct Ctx<'a> {
    spec: &'a FlowsheetSpec,
    capture: &'a dyn CaptureColumns,
    train: TrainOpts,
    /// Cryogenic load per mol/s of S01, W s/mol.
    cryo_w_per_ng: f64,
    /// CO2 compression per mol/s captured, W s/mol.
    comp_w_per_co2: f64,
}

/// Everything one pass produces.
#[derive(Debug, Clone)]
struct PassOut {
    streams: BTreeMap<StreamTag, Port>,
    units: Vec<UnitRecord>,
    duties: Vec<Duty>,
    air_o2: f64,
    tear_out: Vec<f64>,
    h2_product_mol_s: f64,
}

fn retag(t: &str) -> String {
    match t {
        "SG-3" => "S07A",
        "SG-6" => "S09A",
        "SG-7" => "S09B",
        "SG-7B" => "S09C",
        "SG-8" => "S11",
        other => other,
    }
    .to_string()
}

fn retag_record(mut r: UnitRecord) -> UnitRecord {
    r.inlets = r.inlets.iter().map(|t| retag(t)).collect();
    r.outlets = r.outlets.iter().map(|t| retag(t)).collect();
    r
}

fn retag_stream(s: &Stream) -> Stream {
    let mut s = s.clone();
    s.tag = retag(&s.tag);
    s
}

fn put(streams: &mut BTreeMap<StreamTag, Port>, s: Stream) {
    streams.insert(s.tag.clone(), Port::Gas(s));
}

fn cooler_duty(unit: &str, w_removed: f64) -> Duty {
    Duty {
        tag: format!("Q_{unit}"),
        unit: unit.to_string(),
        kind: EnergyKind::Heat,
        w_into_process: -w_removed,
        by_closure: false,
    }
}

/// One pass of the flowsheet at tear input `x` (S03 flows then lean loading).
///
/// eq: D.5
fn pass(ctx: &Ctx<'_>, s01: &Stream, x: &[f64]) -> Result<PassOut> {
    let spec = ctx.spec;
    let mut streams = BTreeMap::new();
    let mut units = Vec::new();
    let mut duties = Vec::new();

    // Recycle in, S03.
    let mut n03 = [0.0; NSP];
    n03.copy_from_slice(&x[..NSP]);
    let s03_in = Stream::gas("S03", RECYCLE.t_k(), RECYCLE.p_pa(), n03);
    let lean_alpha = x[NSP];

    // Process steam, S02, on S/C over every carbon atom of feed and recycle.
    let carbon_in = carbon_of(&s01.n) + carbon_of(&s03_in.n);
    let mut n02 = [0.0; NSP];
    n02[H2O] = spec.s_to_c * carbon_in;
    let s02 = Stream::gas("S02", STEAM_IN.t_k(), STEAM_IN.p_pa(), n02);

    // FEED-PREHEAT: mixer and heater to the reformer inlet, S04.
    let mut n04 = [0.0; NSP];
    for (i, v) in n04.iter_mut().enumerate() {
        *v = s01.n[i] + s02.n[i] + s03_in.n[i];
    }
    let s04 = Stream::gas("S04", REFORMER_IN.t_k(), REFORMER_IN.p_pa(), n04);
    let q_pre = s04.h_w()? - gas_h(s01)? - s02.h_w()? - gas_h(&s03_in)?;
    units.push(UnitRecord {
        tag: "FEED-PREHEAT".into(),
        class: ModelClass::Rigorous,
        boundary: Boundary::Inside,
        inlets: vec!["S01".into(), "S02".into(), "S03".into()],
        outlets: vec!["S04".into()],
        duties: vec![energy(
            "Q_FEED_PREHEAT",
            EnergyKind::Heat,
            q_pre,
            "HEAT_SUPPLY",
            "FEED-PREHEAT",
        )],
        closure: block_closure(&[s01, &s02, &s03_in], &[&s04], q_pre, 0.0)?,
        flags: vec!["mixer and heater to the printed reformer inlet state".into()],
    });
    duties.push(Duty {
        tag: "Q_FEED_PREHEAT".into(),
        unit: "FEED-PREHEAT".into(),
        kind: EnergyKind::Heat,
        w_into_process: q_pre,
        by_closure: false,
    });

    // SMR-EQ to S05.
    let smr = smr_eq::solve(
        &s04,
        &SmrEqSpec {
            tag: "SMR-EQ".into(),
            t_out_k: spec.t_reformer_out_c + 273.15,
            p_out_pa: spec.p_reformer_out_psia * PSI_PA,
            k_source: spec.k_source,
            outlet_tag: "S05".into(),
        },
    )?;
    duties.push(Duty {
        tag: "Q_SMR".into(),
        unit: "SMR-EQ".into(),
        kind: EnergyKind::Heat,
        w_into_process: smr.duty_w,
        by_closure: false,
    });
    units.push(smr.record.clone());

    // Shift and cooling chain to the flash, TEV-961 Case 5 set states.
    let t = case5::train(&smr.outlet, &ctx.train)?;
    for c in [&t.hps_gen1, &t.ex1, &t.ex2, &t.ex3, &t.ex4, &t.ko] {
        duties.push(cooler_duty(&c.record.tag, c.duty_w));
        units.push(retag_record(c.record.clone()));
        put(&mut streams, retag_stream(&c.outlet));
    }
    for b in [&t.hts, &t.lts] {
        units.push(retag_record(b.record.clone()));
        put(&mut streams, retag_stream(&b.outlet));
    }
    units.push(retag_record(t.drum.record.clone()));
    let s11 = retag_stream(&t.drum.vapour);
    let s10 = match &t.drum.liquid {
        Some(l) => retag_stream(l),
        None => {
            return Err(EngineError::Range {
                param: "flash".into(),
                msg: "no condensate forms at the absorber state".into(),
            })
        }
    };
    put(&mut streams, s10);

    // Capture island.
    let cap = ctx.capture.solve(&s11, lean_alpha)?;
    let q_cap_cool = gas_h(&s11)? + cap.q_in_w + cap.w_in_w - cap.treated.h_w()? - gas_h(&cap.co2)?;
    put(&mut streams, s11);
    for r in &cap.records {
        units.push(r.clone());
    }
    if let Some(r) = &cap.rich {
        streams.insert("S17".into(), Port::Solvent(r.clone()));
    }
    if let Some(l) = &cap.lean_out {
        streams.insert("S18".into(), Port::Solvent(l.clone()));
    }
    duties.push(Duty {
        tag: "Q_CAPTURE".into(),
        unit: "CAPTURE".into(),
        kind: EnergyKind::Heat,
        w_into_process: cap.q_in_w,
        by_closure: false,
    });
    duties.push(Duty {
        tag: "W_CAPTURE".into(),
        unit: "CAPTURE".into(),
        kind: EnergyKind::Electric,
        w_into_process: cap.w_in_w,
        by_closure: false,
    });
    duties.push(Duty {
        tag: "Q_CAPTURE_COOLING".into(),
        unit: "CAPTURE".into(),
        kind: EnergyKind::Heat,
        w_into_process: -q_cap_cool,
        by_closure: true,
    });

    // CO2 compression, S19 to S20, on card J.22 with the flowsheet's CO2.
    let s19 = cap.co2.clone();
    let mut n20 = [0.0; NSP];
    n20[CO2] = s19.n[CO2];
    let s20 = Stream::gas("S20", CO2_HIGH.t_k(), CO2_HIGH.p_pa(), n20);
    let w_comp = ctx.comp_w_per_co2 * s19.n[CO2];
    let q_comp_cool = gas_h(&s19)? + w_comp - gas_h(&s20)?;
    units.push(UnitRecord {
        tag: "CO2-COMP".into(),
        class: ModelClass::Empirical,
        boundary: Boundary::Inside,
        inlets: vec!["S19".into()],
        outlets: vec!["S20".into()],
        duties: vec![
            energy(
                "W_CO2COMP",
                EnergyKind::Electric,
                w_comp,
                "GRID",
                "CO2-COMP",
            ),
            energy(
                "Q_CO2COMP_COOLING",
                EnergyKind::Heat,
                q_comp_cool,
                "CO2-COMP",
                "COOLING_WATER",
            ),
        ],
        closure: block_closure(&[&s19], &[&s20], w_comp - q_comp_cool, 0.0)?,
        flags: vec![
            "INL row scaled on captured CO2 (J.22); intercooling heat by closure on \
             ideal-gas enthalpy"
                .into(),
        ],
    });
    duties.push(Duty {
        tag: "W_CO2COMP".into(),
        unit: "CO2-COMP".into(),
        kind: EnergyKind::Electric,
        w_into_process: w_comp,
        by_closure: false,
    });
    duties.push(Duty {
        tag: "Q_CO2COMP_COOLING".into(),
        unit: "CO2-COMP".into(),
        kind: EnergyKind::Heat,
        w_into_process: -q_comp_cool,
        by_closure: true,
    });
    put(&mut streams, s19);
    put(&mut streams, s20);

    // PSA and the cryogenic H2 return, solved together in closed form on H2.
    let s12 = cap.treated.clone();
    let xi = spec.psa_recovery;
    let r = spec.h2_return;
    let psa = psa_boundary(&s12, xi, spec.ch4_in_product, r)?;
    let (s13, s14, s21) = (psa.product, psa.reject, psa.h2_return);
    let h2_reject = s14.n[H2];
    let h2_prod = s13.n[H2];
    let mut psa_closure = block_closure(&[&s12, &s21], &[&s13, &s14], 0.0, 0.0)?;
    // External block: no energy balance is claimed across it.
    psa_closure.energy_rel = 0.0;
    units.push(UnitRecord {
        tag: "PSA".into(),
        class: ModelClass::ExternalBoundary,
        boundary: Boundary::External,
        inlets: vec!["S12".into(), "S21".into()],
        outlets: vec!["S13".into(), "S14".into()],
        duties: Vec::new(),
        closure: psa_closure,
        flags: vec![format!(
            "unsimulated external block: {:.1}% H2 recovery and {:.1} mol% CH4 in the \
             product are specified and not simulated (Q4, J.12)",
            100.0 * xi,
            100.0 * spec.ch4_in_product
        )],
    });

    // REC-CRYO: three outlets.
    let f = spec.carbon_recycle;
    let mut n03_out = [0.0; NSP];
    let mut n15 = [0.0; NSP];
    for i in 0..NSP {
        let c = crate::carbon::CARBON_ATOMS[i];
        if c > 0.0 {
            n03_out[i] = f * s14.n[i];
            n15[i] = s14.n[i] - n03_out[i];
        } else if i != H2 {
            n15[i] = s14.n[i];
        }
    }
    if spec.h2_rest_to_recycle {
        n03_out[H2] = (1.0 - r) * h2_reject;
    } else {
        n15[H2] = (1.0 - r) * h2_reject;
    }
    let s03_out = Stream::gas("S03", RECYCLE.t_k(), RECYCLE.p_pa(), n03_out);
    let s15 = Stream::gas("S15", PSA_REJECT.t_k(), PSA_REJECT.p_pa(), n15);
    let w_cryo = ctx.cryo_w_per_ng * s01.total();
    let q_cryo_cool = gas_h(&s14)? + w_cryo - gas_h(&s03_out)? - gas_h(&s21)? - gas_h(&s15)?;
    units.push(UnitRecord {
        tag: "REC-CRYO".into(),
        class: ModelClass::FixedAssumption,
        boundary: Boundary::Inside,
        inlets: vec!["S14".into()],
        outlets: vec!["S03".into(), "S15".into(), "S21".into()],
        duties: vec![
            energy("W_CRYO", EnergyKind::Electric, w_cryo, "GRID", "REC-CRYO"),
            energy(
                "Q_CRYO_COOLING",
                EnergyKind::Heat,
                q_cryo_cool,
                "REC-CRYO",
                "COOLING_WATER",
            ),
        ],
        closure: block_closure(&[&s14], &[&s03_out, &s15, &s21], w_cryo - q_cryo_cool, 0.0)?,
        flags: vec![
            format!(
                "carbon split held at INL's recycle fraction {f:.4} (J.7); N2 and water \
                 leave in the cut"
            ),
            format!(
                "H2 return fraction {r:.6} calculated by closure on INL's 34.0 MMSCFD to \
                 130 MMSCFD pair at Case A INL_SEL"
            ),
            "load is the INL Purge Recovery row (J.23) scaled on fresh gas; heat by closure".into(),
        ],
    });
    duties.push(Duty {
        tag: "W_CRYO".into(),
        unit: "REC-CRYO".into(),
        kind: EnergyKind::Electric,
        w_into_process: w_cryo,
        by_closure: false,
    });
    duties.push(Duty {
        tag: "Q_CRYO_COOLING".into(),
        unit: "REC-CRYO".into(),
        kind: EnergyKind::Heat,
        w_into_process: -q_cryo_cool,
        by_closure: true,
    });

    // N2-BURN: complete combustion of the cut with stoichiometric O2.
    let el = gas_elements(&s15)?;
    let mut n16 = [0.0; NSP];
    n16[CO2] = el[0];
    n16[H2O] = el[1] / 2.0;
    n16[N2] = el[3] / 2.0;
    let o2 = (2.0 * el[0] + el[1] / 2.0 - el[2]) / 2.0;
    if o2 < 0.0 {
        return Err(EngineError::Range {
            param: "S15".into(),
            msg: "the cut holds more oxygen than its combustion products".into(),
        });
    }
    let s16 = Stream::gas("S16", STACK.t_k(), STACK.p_pa(), n16);
    // Air enters at 298.15 K, where O2 carries no formation enthalpy.
    let q_burn = gas_h(&s15)? - gas_h(&s16)?;
    units.push(UnitRecord {
        tag: "N2-BURN".into(),
        class: ModelClass::ReducedOrder,
        boundary: Boundary::Inside,
        inlets: vec!["S15".into()],
        outlets: vec!["S16".into()],
        duties: vec![energy(
            "Q_N2_BURN",
            EnergyKind::Heat,
            q_burn,
            "N2-BURN",
            "STEAM_SYSTEM",
        )],
        closure: block_closure(&[&s15], &[&s16], -q_burn, o2)?,
        flags: vec![
            "complete combustion with stoichiometric O2 from air at 298.15 K; air N2 and \
             excess air not tracked; heat released to steam raising by closure"
                .into(),
        ],
    });
    duties.push(Duty {
        tag: "Q_N2_BURN".into(),
        unit: "N2-BURN".into(),
        kind: EnergyKind::Heat,
        w_into_process: -q_burn,
        by_closure: true,
    });

    let mut tear_out = s03_out.n.to_vec();
    tear_out.push(match &cap.lean_out {
        Some(l) if ctx.capture.solvent_loop() && spec.solvent_tear => l.n_co2 / l.n_mea,
        _ => lean_alpha,
    });

    put(&mut streams, s01.clone());
    put(&mut streams, s02);
    put(&mut streams, s03_out);
    put(&mut streams, s04);
    put(&mut streams, smr.outlet.clone());
    put(&mut streams, s12);
    put(&mut streams, s13);
    put(&mut streams, s14);
    put(&mut streams, s15);
    put(&mut streams, s16);
    put(&mut streams, s21);
    // The S03 the preheater used this pass is kept for the closure checks.
    streams.insert("S03_IN".into(), Port::Gas(s03_in));
    if let Some(l) = &cap.lean_in {
        streams.insert("S18_IN".into(), Port::Solvent(l.clone()));
    }

    Ok(PassOut {
        streams,
        units,
        duties,
        air_o2: o2,
        tear_out,
        h2_product_mol_s: h2_prod,
    })
}

/// Outlets of the PSA block with the cryogenic H2 return closed around it.
#[derive(Debug, Clone, PartialEq)]
pub struct PsaOut {
    /// S13, H2 at the specified recovery with CH4 at the specified purity.
    pub product: Stream,
    /// S14, everything else in the PSA feed.
    pub reject: Stream,
    /// S21, the share `h2_return` of the reject H2, back ahead of the PSA.
    pub h2_return: Stream,
}

/// PSA as an unsimulated external block: the product takes `xi` of the H2 fed
/// and CH4 at mole fraction `y_ch4`, the rest is reject. REC-CRYO returns the
/// share `h2_return` of the reject H2 to the PSA feed, so with S12 holding
/// F mol/s of H2 the PSA sees F / (1 - r (1 - xi)). The loop is linear in H2
/// and needs no tear.
///
/// eq: J.12
pub fn psa_boundary(s12: &Stream, xi: f64, y_ch4: f64, h2_return: f64) -> Result<PsaOut> {
    let r = h2_return;
    let h2_feed = s12.n[H2] / (1.0 - r * (1.0 - xi));
    let h2_reject = (1.0 - xi) * h2_feed;
    let mut n21 = [0.0; NSP];
    n21[H2] = r * h2_reject;
    let h2_prod = xi * h2_feed;
    let ch4_prod = h2_prod * y_ch4 / (1.0 - y_ch4);
    if ch4_prod > s12.n[CH4] {
        return Err(EngineError::Range {
            param: "ch4_in_product".into(),
            msg: format!(
                "the product needs {ch4_prod} mol/s CH4 and the PSA feed holds {}",
                s12.n[CH4]
            ),
        });
    }
    let mut n13 = [0.0; NSP];
    n13[H2] = h2_prod;
    n13[CH4] = ch4_prod;
    let mut n14 = s12.n;
    n14[H2] = h2_reject;
    n14[CH4] -= ch4_prod;
    Ok(PsaOut {
        product: Stream::gas("S13", H2_PRODUCT.t_k(), H2_PRODUCT.p_pa(), n13),
        reject: Stream::gas("S14", PSA_REJECT.t_k(), PSA_REJECT.p_pa(), n14),
        h2_return: Stream::gas("S21", PSA_FEED.t_k(), PSA_FEED.p_pa(), n21),
    })
}

fn unreachable_stream<T>(tag: &str) -> Result<T> {
    Err(EngineError::Range {
        param: "stream".into(),
        msg: format!("{tag} missing from the pass"),
    })
}

/// Scaled tear residual: flows against the S03 total, loading against itself.
fn tear_residual(x: &[f64], g: &[f64]) -> f64 {
    let total: f64 = x[..NSP]
        .iter()
        .sum::<f64>()
        .max(g[..NSP].iter().sum::<f64>());
    let scale = total.max(1.0e-12);
    let mut r: f64 = 0.0;
    for i in 0..NSP {
        r = r.max((g[i] - x[i]).abs() / scale);
    }
    let a = x[NSP].abs().max(1.0e-3);
    r.max((g[NSP] - x[NSP]).abs() / a)
}

/// Tear solve by Wegstein, then damped substitution. Returns the pass whose
/// input met the tolerance.
fn solve_tears(ctx: &Ctx<'_>, s01: &Stream, x0: Vec<f64>) -> Result<(Vec<f64>, PassOut, TearLog)> {
    let mut history = Vec::new();
    let mut x = x0;
    let mut out = pass(ctx, s01, &x)?;
    let mut passes = 1;
    let mut prev: Option<(Vec<f64>, Vec<f64>)> = None;
    while passes < WEGSTEIN_BUDGET {
        let g = out.tear_out.clone();
        let res = tear_residual(&x, &g);
        history.push(res);
        if res < TEAR_TOL {
            return Ok((
                x,
                out,
                TearLog {
                    passes,
                    residual: res,
                    method: "wegstein".into(),
                    solvent_tear: ctx.spec.solvent_tear && ctx.capture.solvent_loop(),
                    history,
                },
            ));
        }
        let next: Vec<f64> = match (&prev, passes <= 3) {
            (Some((xp, gp)), false) => (0..x.len())
                .map(|i| {
                    let dx = x[i] - xp[i];
                    let q = if dx.abs() > 1.0e-300 {
                        let s = (g[i] - gp[i]) / dx;
                        (s / (s - 1.0)).clamp(-5.0, 0.0)
                    } else {
                        0.0
                    };
                    let v = q * x[i] + (1.0 - q) * g[i];
                    if q.is_finite() && v.is_finite() {
                        v
                    } else {
                        g[i]
                    }
                })
                .collect(),
            _ => g.clone(),
        };
        prev = Some((x, g));
        x = next.into_iter().map(|v| v.max(0.0)).collect();
        out = pass(ctx, s01, &x)?;
        passes += 1;
    }
    // Fallback: damped direct substitution.
    for _ in 0..DAMPED_BUDGET {
        let g = out.tear_out.clone();
        let res = tear_residual(&x, &g);
        history.push(res);
        if res < TEAR_TOL {
            return Ok((
                x,
                out,
                TearLog {
                    passes,
                    residual: res,
                    method: "damped direct substitution after the Wegstein budget".into(),
                    solvent_tear: ctx.spec.solvent_tear && ctx.capture.solvent_loop(),
                    history,
                },
            ));
        }
        x = x
            .iter()
            .zip(&g)
            .map(|(a, b)| (0.5 * a + 0.5 * b).max(0.0))
            .collect();
        out = pass(ctx, s01, &x)?;
        passes += 1;
    }
    Err(EngineError::NotConverged(format!(
        "flowsheet tears after {passes} passes, last residual {:.3e}",
        history.last().copied().unwrap_or(f64::NAN)
    )))
}

/// Start vector of the tears at natural gas `ng_mmscfd`.
fn tear_start(
    m: &Model,
    spec: &FlowsheetSpec,
    lean: f64,
    ng_mmscfd: f64,
    start: TearStart,
) -> Vec<f64> {
    let c = m.carbon(
        spec.scenario,
        m.phi_rec,
        match spec.scenario {
            Scenario::InlSel => m.eta_sel,
            _ => m.eta_mea,
        },
    );
    // Recycle carbon per MMSCFD of fresh gas in the anchored ledger, mol/s.
    let r_mol_s = c.r / c.ng * ng_mmscfd / 3.6;
    let k = match start {
        TearStart::Zero => 0.0,
        TearStart::Anchored => 1.0,
        TearStart::Double => 2.0,
    };
    let mut x = vec![0.0; NSP + 1];
    x[CH4] = k * r_mol_s * (1.0 - m.co_share);
    x[CO] = k * r_mol_s * m.co_share;
    x[NSP] = lean;
    x
}

/// HTS basis of the flowsheet: the bed that `wgs::hts` sizes on the IEAGHG outlet
/// approach for the printed Case 5 syngas, held at its dry space velocity. A
/// fixed space velocity keeps the bed per unit of dry gas constant, and the HTS
/// outlet then varies smoothly with its feed, which the tears need. At the Case
/// 5 syngas it returns the M4 gate result by construction.
pub fn hts_train_opts(k_source: KSource) -> Result<TrainOpts> {
    let sized = TrainOpts::sized_on_ieaghg(k_source)?;
    let raw = case5::raw_syngas()?;
    let t = case5::train(&raw, &sized)?;
    let hts_in = &t.hps_gen1.outlet;
    let dry_mol_s = hts_in.total() - hts_in.n[H2O];
    // Dry gas at 20 C and 1 bar, the reference state of `wgs::HtsSize::Ghsv`.
    let dry_m3_h = dry_mol_s * R_GAS * 293.15 / 1.0e5 * 3600.0;
    let ghsv = dry_m3_h / t.hts.bed_volume_m3;
    Ok(TrainOpts {
        hts_size: nush2_units::wgs::HtsSize::Ghsv(ghsv),
        ..sized
    })
}

fn capture_for(m: &Model, spec: &FlowsheetSpec, s: f64) -> Box<dyn CaptureColumns> {
    match spec.scenario {
        Scenario::InlSel => Box::new(AnchoredSelexol {
            eta: m.eta_sel,
            w_e: m.e_rows.purification * s * 1.0e6,
        }),
        _ => Box::new(AnchoredMea::from_model(m)),
    }
}

fn boundary_links() -> Vec<BoundaryLink> {
    vec![
        BoundaryLink {
            stream: "S01".into(),
            from: Some("BATTERY_LIMIT"),
            to: None,
        },
        BoundaryLink {
            stream: "S02".into(),
            from: Some("STEAM_SYSTEM"),
            to: None,
        },
        BoundaryLink {
            stream: "S10".into(),
            from: None,
            to: Some("WATER_TREATMENT"),
        },
        BoundaryLink {
            stream: "S13".into(),
            from: None,
            to: Some("PRODUCT"),
        },
        BoundaryLink {
            stream: "S16".into(),
            from: None,
            to: Some("ATMOSPHERE"),
        },
        BoundaryLink {
            stream: "S20".into(),
            from: None,
            to: Some("STORAGE"),
        },
    ]
}

/// True when a unit or battery-limit node sits inside the dashed line of the
/// slide 4 figure (Figure 2). Inside: the nuclear
/// island, the reformer train with its coolers and flash, MEA capture, the
/// recycle compressor, the cryogenic unit, the nitrogen-rich stack, the CO2
/// compressor and the ship offshore storage. Outside: the PSA, which the figure
/// draws beyond the line, and every battery limit (gas supply, steam supply,
/// water treatment, product, atmosphere, off-site users).
pub fn inside_boundary(node: &str) -> bool {
    !matches!(
        node,
        "PSA"
            | "BATTERY_LIMIT"
            | "STEAM_SYSTEM"
            | "WATER_TREATMENT"
            | "PRODUCT"
            | "ATMOSPHERE"
            | "OFFSITE"
            | ""
    )
}

/// A stream crosses the boundary when exactly one of its ends is inside.
pub fn crosses(from: &str, to: &str) -> bool {
    inside_boundary(from) != inside_boundary(to)
}

/// Source and sink of a stream in a run, from its units and boundary links.
pub fn ends(run: &FlowsheetRun, tag: &str) -> (String, String) {
    let from = run
        .flowsheet
        .units
        .iter()
        .find(|u| u.outlets.iter().any(|o| o == tag))
        .map(|u| u.tag.clone())
        .or_else(|| {
            run.boundary
                .iter()
                .find(|b| b.stream == tag)
                .and_then(|b| b.from.map(str::to_string))
        })
        .unwrap_or_default();
    let to = run
        .flowsheet
        .units
        .iter()
        .find(|u| u.inlets.iter().any(|o| o == tag))
        .map(|u| u.tag.clone())
        .or_else(|| {
            run.boundary
                .iter()
                .find(|b| b.stream == tag)
                .and_then(|b| b.to.map(str::to_string))
        })
        .unwrap_or_default();
    (from, to)
}

/// Whether a stream of a run crosses the boundary, from where its ends sit.
pub fn crosses_boundary(run: &FlowsheetRun, tag: &str) -> bool {
    let (from, to) = ends(run, tag);
    crosses(&from, &to)
}

/// Plain name of a stream tag.
pub fn stream_name(tag: &str) -> &'static str {
    match tag {
        "S01" => "Fresh natural gas",
        "S02" => "Process steam",
        "S03" => "Recycled methane-rich gas",
        "S04" => "Mixed reformer feed",
        "S05" => "Raw syngas, reformer outlet",
        "S06" => "HTS inlet",
        "S07" => "HTS outlet",
        "S07A" => "WGS-EX1 outlet",
        "S08" => "LTS inlet",
        "S09" => "LTS outlet",
        "S09A" => "WGS-EX3 outlet",
        "S09B" => "WGS-EX4 outlet",
        "S09C" => "KO-DRM-1 outlet",
        "S10" => "Process condensate",
        "S11" => "Absorber feed",
        "S12" => "H2-rich gas after capture (PSA_FEED)",
        "S13" => "Purified H2 after external PSA (H2_PRODUCT)",
        "S14" => "PSA reject crossing the boundary (PSA_REJECT_RETURN)",
        "S15" => "Nitrogen-rich cut (N2_RICH_CUT)",
        "S16" => "Stack exhaust",
        "S17" => "Rich solvent",
        "S18" => "Lean solvent",
        "S19" => "CO2 from capture",
        "S20" => "Compressed CO2 (CO2_TO_STORAGE)",
        "S21" => "Cryogenic H2 return ahead of the PSA",
        _ => "",
    }
}

/// Assemble the run from the converged pass.
fn assemble(
    id: &str,
    spec: &FlowsheetSpec,
    ng_mmscfd: f64,
    out: PassOut,
    tear: TearLog,
    capture_id: &str,
) -> Result<FlowsheetRun> {
    let PassOut {
        mut streams,
        units,
        duties,
        air_o2,
        h2_product_mol_s,
        ..
    } = out;
    let s03_in = match streams.remove("S03_IN") {
        Some(Port::Gas(s)) => s,
        _ => unreachable_stream("S03_IN")?,
    };
    streams.remove("S18_IN");
    let g = |t: &str| -> Result<Stream> {
        match streams.get(t) {
            Some(Port::Gas(s)) => Ok(s.clone()),
            _ => unreachable_stream(t),
        }
    };
    let s11 = g("S11")?;
    let carbon = CarbonLedger {
        feed: carbon_kmol_h(&g("S01")?),
        tube: carbon_kmol_h(&g("S04")?),
        co2_abs: s11.n[CO2] * 3.6,
        captured: g("S20")?.n[CO2] * 3.6,
        slip: g("S12")?.n[CO2] * 3.6,
        tail_gas: carbon_kmol_h(&g("S14")?),
        recycled: carbon_kmol_h(&g("S03")?),
        burned: carbon_kmol_h(&g("S15")?),
        product: carbon_kmol_h(&g("S13")?),
        condensate: carbon_kmol_h(&g("S10")?),
        n2_cut: n2_kmol_h(&g("S15")?),
    };
    let mut flags = vec![
        format!("capture columns: {capture_id}"),
        format!(
            "tears S03{} converged in {} passes by {}, residual {:.2e}",
            if tear.solvent_tear { " and S18" } else { "" },
            tear.passes,
            tear.method,
            tear.residual
        ),
        format!(
            "S03 used by the preheater differs from the cryogenic outlet by the tear \
             residual; carbon {:.3e} kmol/h",
            (carbon_kmol_h(&s03_in) - carbon.recycled).abs()
        ),
    ];
    if !tear.solvent_tear && spec.scenario == Scenario::RecF {
        flags.push("fallback: S18 lean loading held fixed, S03 tear only".into());
    }
    Ok(FlowsheetRun {
        id: id.to_string(),
        spec: spec.clone(),
        ng_mmscfd,
        flowsheet: Flowsheet { streams, units },
        boundary: boundary_links(),
        duties,
        air_o2_mol_s: air_o2,
        h2_product_kg_h: h2_product_mol_s * nush2_props::units::MW_H2 * 3.6,
        carbon,
        tear,
        flags,
    })
}

/// Solve one run at a fixed natural gas flow.
fn solve_at_ng(
    m: &Model,
    spec: &FlowsheetSpec,
    ng_mmscfd: f64,
    x0: Option<Vec<f64>>,
    start: TearStart,
) -> Result<(Vec<f64>, PassOut, TearLog)> {
    let s = m.scale(spec.case);
    let capture = capture_for(m, spec, s);
    let ng_anchor = m.ng_c6 * s;
    let s01_anchor = natural_gas(ng_anchor)?;
    let b = m.balance(spec.scenario, &m.opts(s))?;
    let ctx = Ctx {
        spec,
        capture: capture.as_ref(),
        train: hts_train_opts(spec.k_source)?,
        cryo_w_per_ng: b.e_rows.purge * 1.0e6 / s01_anchor.total(),
        comp_w_per_co2: m.e_rows.co2comp * 1.0e6 / (st_d_to_kmol_h(m.co2_cap_c6) / 3.6),
    };
    let s01 = natural_gas(ng_mmscfd)?;
    let x0 = match x0 {
        Some(x) => x,
        None => tear_start(m, spec, capture.lean_start(), ng_mmscfd, start),
    };
    match solve_tears(&ctx, &s01, x0.clone()) {
        // Fallback: S03 alone with the lean loading fixed.
        Err(EngineError::NotConverged(msg)) if spec.solvent_tear && capture.solvent_loop() => {
            let mut fixed = spec.clone();
            fixed.solvent_tear = false;
            let ctx2 = Ctx {
                spec: &fixed,
                ..ctx
            };
            let mut x = x0;
            x[NSP] = capture.lean_start();
            let (x, out, mut log) = solve_tears(&ctx2, &s01, x)?;
            log.method = format!("{}, after the joint tear failed ({msg})", log.method);
            Ok((x, out, log))
        }
        other => other,
    }
}

/// Solve a run on its feed basis. A hydrogen basis is met by scaling the natural
/// gas, since every unit at M7 is set by intensive specifications and the
/// flowsheet is homogeneous of degree one in flow.
pub fn solve(m: &Model, spec: &FlowsheetSpec, id: &str, start: TearStart) -> Result<FlowsheetRun> {
    spec.validate(m)?;
    let capture_id = capture_for(m, spec, m.scale(spec.case)).id();
    match spec.basis {
        FeedBasis::NaturalGas(ng) => {
            let (_, out, tear) = solve_at_ng(m, spec, ng, None, start)?;
            assemble(id, spec, ng, out, tear, capture_id)
        }
        FeedBasis::Hydrogen(kg_h) => {
            let target = kg_h / (nush2_props::units::MW_H2 * 3.6);
            let mut ng = m.ng_c6 * m.scale(spec.case);
            let (mut x, mut out, mut tear) = solve_at_ng(m, spec, ng, None, start)?;
            for _ in 0..6 {
                let ratio = target / out.h2_product_mol_s;
                if (ratio - 1.0).abs() < 1.0e-12 {
                    break;
                }
                ng *= ratio;
                let mut x0: Vec<f64> = x.iter().map(|v| v * ratio).collect();
                x0[NSP] = x[NSP];
                let r = solve_at_ng(m, spec, ng, Some(x0), start)?;
                x = r.0;
                out = r.1;
                tear = r.2;
            }
            let gap = out.h2_product_mol_s / target - 1.0;
            if gap.abs() > 1.0e-9 {
                return Err(EngineError::NotConverged(format!(
                    "H2 basis missed by {gap:.3e} after scaling the natural gas"
                )));
            }
            assemble(id, spec, ng, out, tear, capture_id)
        }
    }
}

/// Outcome of the H2 return closure.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct H2ReturnClosure {
    /// Calculated by closure; INL publishes no value.
    pub h2_return: f64,
    pub ng_mmscfd: f64,
    pub h2_target_kg_h: f64,
    pub h2_at_zero_kg_h: f64,
    pub h2_at_one_kg_h: f64,
    pub evaluations: usize,
}

/// Close the cryogenic H2 return fraction on INL's 34.0 MMSCFD natural gas and
/// 130 MMSCFD (29,000 lb/h) H2 at Case A INL_SEL.
///
/// The target is the 29,000 lb/h of H2 that INL equates with 130 MMSCFD
/// (TEV-953 doc p15), the same value the anchored scale uses, so Case B carries
/// no conversion offset. 130 MMSCFD at 99.9 mol% holds 0.9% less H2 than that.
/// The register holds no card for this closure yet.
pub fn close_h2_return(m: &Model) -> Result<H2ReturnClosure> {
    let ng = m.ng_c6;
    let target = m.h2_a_kgh;
    let mut warm: Option<Vec<f64>> = None;
    let mut evals = 0;
    let mut h2_at = |r: f64, warm: &mut Option<Vec<f64>>| -> Result<f64> {
        let spec = FlowsheetSpec::new(m, Case::A, Scenario::InlSel, FeedBasis::NaturalGas(ng), r);
        spec.validate(m)?;
        let (x, out, _) = solve_at_ng(m, &spec, ng, warm.clone(), TearStart::Anchored)?;
        *warm = Some(x);
        evals += 1;
        Ok(out.h2_product_mol_s * nush2_props::units::MW_H2 * 3.6)
    };
    let (mut a, mut b) = (0.0, 1.0);
    let lo = h2_at(a, &mut warm)?;
    let hi = h2_at(b, &mut warm)?;
    let (mut fa, mut fb) = (lo - target, hi - target);
    if fa * fb > 0.0 {
        return Err(EngineError::NotConverged(format!(
            "no H2 return fraction in [0, 1] closes 34.0 MMSCFD to {target:.1} kg/h H2: \
             {lo:.1} kg/h at 0 and {hi:.1} kg/h at 1"
        )));
    }
    // Illinois regula falsi on a monotone function.
    let mut side = 0;
    let mut r = a;
    for _ in 0..100 {
        r = (a * fb - b * fa) / (fb - fa);
        let fr = h2_at(r, &mut warm)? - target;
        if fr.abs() < 1.0e-10 * target || (b - a).abs() < 1.0e-14 {
            break;
        }
        if fr * fb > 0.0 {
            b = r;
            fb = fr;
            if side == -1 {
                fa /= 2.0;
            }
            side = -1;
        } else {
            a = r;
            fa = fr;
            if side == 1 {
                fb /= 2.0;
            }
            side = 1;
        }
    }
    Ok(H2ReturnClosure {
        h2_return: r,
        ng_mmscfd: ng,
        h2_target_kg_h: target,
        h2_at_zero_kg_h: lo,
        h2_at_one_kg_h: hi,
        evaluations: evals,
    })
}

/// Envelope closure of a run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Envelope {
    /// C, H, O, N, relative to the larger side.
    pub element_rel: [f64; 4],
    /// Mass, relative.
    pub mass_rel: f64,
    /// Energy, relative to the largest term.
    pub energy_rel: f64,
}

/// Streams entering and leaving the envelope that holds every unit inside the
/// boundary. The PSA is outside it, so its feed leaves and its reject enters.
const ENVELOPE_IN: [&str; 3] = ["S01", "S02", "S14"];
const ENVELOPE_OUT: [&str; 5] = ["S10", "S12", "S16", "S20", "S21"];

fn boundary_sets(_run: &FlowsheetRun) -> ([&'static str; 3], [&'static str; 5]) {
    (ENVELOPE_IN, ENVELOPE_OUT)
}

/// Element flows C, H, O, N into and out of the envelope, mol/s. Combustion O2
/// enters as an element flow.
pub fn envelope_elements(run: &FlowsheetRun) -> Result<([f64; 4], [f64; 4])> {
    let (ins, outs) = boundary_sets(run);
    let mut a = [0.0; 4];
    let mut b = [0.0; 4];
    for t in ins {
        add4(&mut a, &gas_elements(run.gas(t)?)?);
    }
    a[2] += 2.0 * run.air_o2_mol_s;
    for t in outs {
        add4(&mut b, &gas_elements(run.gas(t)?)?);
    }
    Ok((a, b))
}

/// Element closure of the envelope that holds every unit inside the boundary,
/// relative to the larger side, C, H, O, N.
///
/// eq: A.5
pub fn check_elements(run: &FlowsheetRun) -> Result<[f64; 4]> {
    let (a, b) = envelope_elements(run)?;
    Ok(rel_gap(&a, &b))
}

/// Mass into and out of the envelope, kg/s.
pub fn envelope_mass(run: &FlowsheetRun) -> Result<(f64, f64)> {
    let (ins, outs) = boundary_sets(run);
    let mut a = run.air_o2_mol_s * MW_O2_KG_MOL;
    let mut b = 0.0;
    for t in ins {
        a += mass_or_zero(run.gas(t)?)?;
    }
    for t in outs {
        b += mass_or_zero(run.gas(t)?)?;
    }
    Ok((a, b))
}

/// Mass closure of the same envelope, relative.
///
/// eq: A.2
pub fn check_mass(run: &FlowsheetRun) -> Result<f64> {
    let (a, b) = envelope_mass(run)?;
    Ok((a - b).abs() / a.max(b))
}

fn mass_or_zero(s: &Stream) -> Result<f64> {
    if s.n.iter().all(|v| *v == 0.0) {
        Ok(0.0)
    } else {
        s.mass_kg_s()
    }
}

/// Formation-basis enthalpy flows of the envelope streams, W, by stream tag.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopeEnthalpy {
    pub inlets: Vec<(&'static str, f64)>,
    pub outlets: Vec<(&'static str, f64)>,
}

/// Enthalpy flows of the streams that enter and leave the envelope, the terms
/// `check_energy` sums with the duties.
///
/// eq: A.8
pub fn envelope_enthalpy(run: &FlowsheetRun) -> Result<EnvelopeEnthalpy> {
    let (ins, outs) = boundary_sets(run);
    let mut e = EnvelopeEnthalpy {
        inlets: Vec::new(),
        outlets: Vec::new(),
    };
    for t in ins {
        e.inlets.push((t, gas_h(run.gas(t)?)?));
    }
    for t in outs {
        e.outlets.push((t, gas_h(run.gas(t)?)?));
    }
    Ok(e)
}

/// Energy closure of the envelope: stream enthalpies on the formation basis plus
/// every duty, heat and electricity, into or out of the process.
///
/// eq: A.8
pub fn check_energy(run: &FlowsheetRun) -> Result<f64> {
    let (ins, outs) = boundary_sets(run);
    let mut h_in = 0.0;
    let mut h_out = 0.0;
    let mut big: f64 = 0.0;
    for t in ins {
        let h = gas_h(run.gas(t)?)?;
        h_in += h;
        big = big.max(h.abs());
    }
    for t in outs {
        let h = gas_h(run.gas(t)?)?;
        h_out += h;
        big = big.max(h.abs());
    }
    let mut q = 0.0;
    for d in &run.duties {
        q += d.w_into_process;
        big = big.max(d.w_into_process.abs());
    }
    Ok((h_in + q - h_out).abs() / big.max(1.0))
}

/// Component balance of the units that hold no reaction: every species in equals
/// every species out. Returns the worst relative gap and the unit.
///
/// eq: A.3
pub fn check_species(run: &FlowsheetRun) -> Result<(f64, String)> {
    let mut worst = (0.0, String::new());
    for u in &run.flowsheet.units {
        if matches!(
            u.tag.as_str(),
            "SMR-EQ" | "WGS-HTS" | "WGS-LTS" | "N2-BURN" | "ABS" | "STRIP" | "SELEXOL"
        ) {
            continue;
        }
        let mut a = [0.0; NSP];
        let mut b = [0.0; NSP];
        for t in &u.inlets {
            // The stored S03 is the cryogenic outlet, which equals the
            // preheater's own S03 within the tear residual.
            let s = run.gas(t)?;
            for (v, x) in a.iter_mut().zip(s.n) {
                *v += x;
            }
        }
        for t in &u.outlets {
            let s = run.gas(t)?;
            for (v, x) in b.iter_mut().zip(s.n) {
                *v += x;
            }
        }
        let scale: f64 = a.iter().sum::<f64>().max(b.iter().sum::<f64>()).max(1e-300);
        for i in 0..NSP {
            let g = (a[i] - b[i]).abs() / scale;
            if g > worst.0 {
                worst = (g, u.tag.clone());
            }
        }
    }
    Ok(worst)
}

/// Each stream has one source and one sink, unit or boundary, and every unit
/// port names a stream that exists. Returns the problems found.
pub fn connectivity(run: &FlowsheetRun) -> Vec<String> {
    let mut problems = Vec::new();
    let mut src: BTreeMap<&str, usize> = BTreeMap::new();
    let mut snk: BTreeMap<&str, usize> = BTreeMap::new();
    for u in &run.flowsheet.units {
        for t in &u.outlets {
            *src.entry(t.as_str()).or_default() += 1;
        }
        for t in &u.inlets {
            *snk.entry(t.as_str()).or_default() += 1;
        }
    }
    for b in &run.boundary {
        if b.from.is_some() {
            *src.entry(b.stream.as_str()).or_default() += 1;
        }
        if b.to.is_some() {
            *snk.entry(b.stream.as_str()).or_default() += 1;
        }
    }
    for tag in run.flowsheet.streams.keys() {
        let s = src.get(tag.as_str()).copied().unwrap_or(0);
        let k = snk.get(tag.as_str()).copied().unwrap_or(0);
        if s != 1 || k != 1 {
            problems.push(format!("{tag}: {s} sources and {k} sinks"));
        }
    }
    for tag in src.keys().chain(snk.keys()) {
        if !run.flowsheet.streams.contains_key(*tag) {
            problems.push(format!("{tag} is named by a unit but absent"));
        }
    }
    problems.sort();
    problems.dedup();
    problems
}

/// Envelope closure of a run.
pub fn envelope(run: &FlowsheetRun) -> Result<Envelope> {
    Ok(Envelope {
        element_rel: check_elements(run)?,
        mass_rel: check_mass(run)?,
        energy_rel: check_energy(run)?,
    })
}

/// The H2 ledger: reformer syngas, gas after capture, product
/// after the PSA, recovery and annual H2, kept as separate lines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct H2Ledger {
    pub reformer_h2_kg_h: f64,
    pub after_capture_h2_kg_h: f64,
    pub psa_feed_h2_kg_h: f64,
    pub product_h2_kg_h: f64,
    pub psa_recovery: f64,
    pub plant_recovery: f64,
    pub annual_t_y: f64,
}

/// H2 ledger of a run.
///
/// eq: J.13
pub fn h2_ledger(m: &Model, run: &FlowsheetRun) -> Result<H2Ledger> {
    let kg = |t: &str| -> Result<f64> { Ok(run.gas(t)?.n[H2] * nush2_props::units::MW_H2 * 3.6) };
    let feed = kg("S12")? + kg("S21")?;
    let product = kg("S13")?;
    Ok(H2Ledger {
        reformer_h2_kg_h: kg("S05")?,
        after_capture_h2_kg_h: kg("S12")?,
        psa_feed_h2_kg_h: feed,
        product_h2_kg_h: product,
        psa_recovery: product / feed,
        plant_recovery: product / kg("S12")?,
        annual_t_y: product * m.hours / 1000.0,
    })
}

/// Shaft work of the infinite-stage intercooled limit of the CO2 compressor:
/// reversible and isothermal at the S19 temperature, ideal gas, S19 to S20
/// pressure, W. Finite stages and machine efficiencies are unpublished, and real
/// gas departure above about 60 bar is not modelled, so this is a cross-check of
/// the INL row and never replaces it. No register card exists for it yet.
///
/// eq: J.22
pub fn co2_compression_isothermal_w(run: &FlowsheetRun) -> Result<f64> {
    let a = run.gas("S19")?;
    let b = run.gas("S20")?;
    Ok(a.n[CO2] * R_GAS * a.t_k * (b.p_pa / a.p_pa).ln())
}

/// Modelled slots of the terms M7 owns, MW, MWe and MMSCFD.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TermSlots {
    /// T03 cross-check: preheat plus reformer duty of Case B REC_F, MW.
    pub t03_mw: f64,
    /// T04: process heat of REC_F less INL_SEL at the same H2, MW.
    pub t04_mw: f64,
    /// T09: isothermal limit of CO2 compression, MW of shaft work.
    pub t09_mw: f64,
    /// T13: natural gas for the Case B H2 basis, MMSCFD.
    pub t13_mmscfd: f64,
}

/// Every run M7 exports, with the closure and the term slots.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suite {
    pub closure: H2ReturnClosure,
    pub runs: Vec<FlowsheetRun>,
    pub terms: TermSlots,
}

impl Suite {
    pub fn run(&self, id: &str) -> Option<&FlowsheetRun> {
        self.runs.iter().find(|r| r.id == id)
    }
}

/// Run ids of the suite, sorted.
pub const SUITE_IDS: [&str; 6] = [
    "FS_A_INL_SEL",
    "FS_A_REC_F",
    "FS_B_INL_SEL",
    "FS_B_REC_F",
    "FS_B_REC_F_T800",
    "FS_B_REC_F_T825",
];

/// Id and spec of the suite run for a case and scenario at 871 C: Case A INL_SEL
/// on INL's natural gas, the others on their hydrogen basis. `None` for a
/// scenario the suite does not run.
pub fn base_spec(
    m: &Model,
    case: Case,
    scn: Scenario,
    h2_return: f64,
) -> Option<(String, FlowsheetSpec)> {
    let basis = match (case, scn) {
        (Case::A, Scenario::InlSel) => FeedBasis::NaturalGas(m.ng_c6),
        (Case::A, Scenario::RecF) => FeedBasis::Hydrogen(m.h2_a_kgh),
        (Case::B, Scenario::InlSel | Scenario::RecF) => FeedBasis::Hydrogen(m.h2_b_kgh),
        _ => return None,
    };
    let tag = match case {
        Case::A => "A",
        Case::B => "B",
    };
    Some((
        format!("FS_{tag}_{}", scn.id()),
        FlowsheetSpec::new(m, case, scn, basis, h2_return),
    ))
}

/// H2 return closure on the embedded registry, computed once per process. The
/// closure runs Case A INL_SEL on INL's natural gas, which reads no input the
/// live controls set; the live reply reuses it, and a test of the wasm crate
/// checks that every control leaves it unchanged.
pub fn base_closure_cached() -> Result<H2ReturnClosure> {
    static CLOSURE: std::sync::OnceLock<Result<H2ReturnClosure>> = std::sync::OnceLock::new();
    CLOSURE
        .get_or_init(|| {
            let p = crate::params::ParamSet::embedded()?;
            close_h2_return(&Model::new(&p)?)
        })
        .clone()
}

/// Close the H2 return, then run Case A and Case B on both scenarios at 871 C
/// and Case B REC_F at 825 C and 800 C (S8), all on the hydrogen basis except
/// the closure run.
pub fn suite(m: &Model) -> Result<Suite> {
    let closure = close_h2_return(m)?;
    let r = closure.h2_return;
    let mut runs = Vec::new();
    let mut b_rec = None;
    for (case, scn) in [
        (Case::A, Scenario::InlSel),
        (Case::A, Scenario::RecF),
        (Case::B, Scenario::InlSel),
        (Case::B, Scenario::RecF),
    ] {
        let (id, spec) = base_spec(m, case, scn, r).ok_or_else(|| EngineError::Range {
            param: "run".into(),
            msg: format!("no Level 3 spec for {case:?} {}", scn.id()),
        })?;
        runs.push(solve(m, &spec, &id, TearStart::Anchored)?);
        if (case, scn) == (Case::B, Scenario::RecF) {
            b_rec = Some(spec);
        }
    }
    let b_rec = b_rec.ok_or_else(|| EngineError::Range {
        param: "run".into(),
        msg: "FS_B_REC_F spec missing".into(),
    })?;
    for t in T_REFORMER_SENSITIVITIES_C {
        let mut spec = b_rec.clone();
        spec.t_reformer_out_c = t;
        let id = format!("FS_B_REC_F_T{t:.0}");
        runs.push(solve(m, &spec, &id, TearStart::Anchored)?);
    }
    runs.sort_by(|a, b| a.id.cmp(&b.id));
    let get = |id: &str| -> Result<&FlowsheetRun> {
        runs.iter()
            .find(|r| r.id == id)
            .ok_or_else(|| EngineError::Range {
                param: "run".into(),
                msg: format!("{id} missing"),
            })
    };
    let b_rec_run = get("FS_B_REC_F")?;
    let terms = TermSlots {
        t03_mw: b_rec_run.process_heat_mw(),
        t04_mw: b_rec_run.process_heat_mw() - get("FS_B_INL_SEL")?.process_heat_mw(),
        t09_mw: co2_compression_isothermal_w(b_rec_run)? / 1.0e6,
        t13_mmscfd: b_rec_run.ng_mmscfd,
    };
    Ok(Suite {
        closure,
        runs,
        terms,
    })
}

/// Relative tolerance of `carbon_matches_anchor`.
pub const CARBON_ANCHOR_TOL: f64 = 0.02;
/// Relative tolerance of the three-start tear test.
pub const THREE_START_TOL: f64 = 1.0e-5;
/// Relative limit on modelled natural gas against 23.2 MMSCFD.
pub const NG_FINDING_TOL: f64 = 0.05;

/// One comparison for `results/Validation_Results.csv`, in the field order of the
/// V&V row. Built only from runs that converged in the same call.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Check {
    pub test_id: String,
    pub level: u8,
    pub subject: String,
    pub source: String,
    pub locator: String,
    pub conditions: String,
    pub literature: Option<f64>,
    pub simulated: f64,
    pub unit: String,
    pub abs_error: Option<f64>,
    pub rel_error: Option<f64>,
    pub tolerance: f64,
    pub pass: bool,
    pub data_use: &'static str,
    pub explanation: String,
}

/// A relative comparison against `reference`.
fn rel_check(id: String, subject: &str, reference: f64, got: f64, tol: f64) -> Check {
    let abs = (got - reference).abs();
    let rel = abs / reference.abs();
    Check {
        test_id: id,
        level: 3,
        subject: subject.to_string(),
        source: String::new(),
        locator: String::new(),
        conditions: String::new(),
        literature: Some(reference),
        simulated: got,
        unit: String::new(),
        abs_error: Some(abs),
        rel_error: Some(rel),
        tolerance: tol,
        pass: rel <= tol,
        data_use: "same_dataset_disclosed",
        explanation: String::new(),
    }
}

/// Modelled carbon lines of Case B REC_F against the anchored ledger, each held
/// to `CARBON_ANCHOR_TOL` relative. A miss stays recorded as failed.
pub fn carbon_vs_anchor(m: &Model, run: &FlowsheetRun) -> Result<Vec<Check>> {
    let s = m.scale(run.spec.case);
    let b = m.balance(run.spec.scenario, &m.opts(s))?;
    let c = b.c;
    let k = &run.carbon;
    let lines = [
        ("FEED", "Feed carbon", c.f * s, k.feed, "J.5"),
        ("TUBE", "Tube carbon", c.t * s, k.tube, "J.8"),
        (
            "CO2_ABS",
            "CO2 at the absorber",
            c.co2_abs * s,
            k.co2_abs,
            "J.6",
        ),
        ("CAPTURED", "Captured CO2", c.cap * s, k.captured, "J.9"),
        ("TAIL_GAS", "Tail-gas carbon", c.g * s, k.tail_gas, "J.10"),
        ("RECYCLED", "Recycled carbon", c.r * s, k.recycled, "J.10"),
        (
            "BURNED",
            "Burned carbon, N2-rich cut",
            c.burn * s,
            k.burned,
            "J.10",
        ),
        (
            "PRODUCT",
            "Carbon in the H2 product",
            c.p * s,
            k.product,
            "J.12",
        ),
    ];
    let s11 = run.gas("S11")?;
    let phi = s11.n[CO2] / (s11.n[CO2] + s11.n[CH4] + s11.n[CO]);
    Ok(lines
        .iter()
        .map(|(id, subject, anchored, got, card)| {
            let mut ch = rel_check(
                format!("FS_CARBON_VS_ANCHOR_{id}"),
                subject,
                *anchored,
                *got,
                CARBON_ANCHOR_TOL,
            );
            ch.source = "Anchored ledger, cards J.5 to J.12".into();
            ch.locator = format!(
                "card {card}; test carbon_matches_anchor, engine/plant/tests/m7_flowsheet.rs"
            );
            ch.conditions = format!(
                "{}, H2 {:.0} kg/h, reformer {:.0} C; tolerance is relative",
                run.id, run.h2_product_kg_h, run.spec.t_reformer_out_c
            );
            ch.unit = "kmol C/h".into();
            ch.explanation = format!(
                "Level 3 flowsheet against the anchored ledger. Carbon split after the shift \
                 is {phi:.4} in the flowsheet against the anchored {:.3}",
                m.phi_rec
            );
            ch
        })
        .collect())
}

/// Largest relative spread of the converged S03 and H2 product over the three
/// tear starts, and the passes each took.
pub fn three_start_spread(m: &Model, spec: &FlowsheetSpec) -> Result<(f64, [usize; 3])> {
    let mut runs = Vec::new();
    for start in [TearStart::Zero, TearStart::Anchored, TearStart::Double] {
        runs.push(solve(m, spec, "THREE_START", start)?);
    }
    let base = &runs[1];
    let s03 = base.gas("S03")?;
    let scale = s03.total();
    let mut spread: f64 = 0.0;
    for r in &runs {
        let x = r.gas("S03")?;
        for i in 0..NSP {
            spread = spread.max((x.n[i] - s03.n[i]).abs() / scale);
        }
        spread = spread.max((r.h2_product_kg_h / base.h2_product_kg_h - 1.0).abs());
        spread = spread.max((r.ng_mmscfd / base.ng_mmscfd - 1.0).abs());
    }
    Ok((
        spread,
        [
            runs[0].tear.passes,
            runs[1].tear.passes,
            runs[2].tear.passes,
        ],
    ))
}

/// Every M7 row for the validation file.
pub fn checks(m: &Model, suite: &Suite) -> Result<Vec<Check>> {
    let mut out = Vec::new();
    let b_rec = suite.run("FS_B_REC_F").ok_or_else(|| EngineError::Range {
        param: "run".into(),
        msg: "FS_B_REC_F missing".into(),
    })?;
    let carbon = carbon_vs_anchor(m, b_rec)?;
    let carbon_failed: Vec<String> = carbon
        .iter()
        .filter(|c| !c.pass)
        .map(|c| c.test_id.replace("FS_CARBON_VS_ANCHOR_", ""))
        .collect();
    out.extend(carbon);

    let ng_ref = m.ng_c6 * m.s_b;
    let mut ng = rel_check(
        "FS_T13_NG_FOR_CASE_B_H2".into(),
        "Natural gas for 8,988 kg/h H2, term T13",
        ng_ref,
        b_rec.ng_mmscfd,
        NG_FINDING_TOL,
    );
    ng.source = "TEV-961 Table 2 doc p16, 34.0 MMSCFD, times the Case B scale".into();
    ng.locator = "Table 2, PDF p17, doc p16".into();
    ng.conditions = "Case B REC_F, 871 C, H2 return carried from the closure".into();
    ng.unit = "MMSCFD".into();
    ng.explanation = "Tolerance is relative. The closure fixes INL_SEL at Case A, so this row \
                      tests the move to MEA at 90% with recycle"
        .into();
    out.push(ng);

    let a_inl = suite
        .run("FS_A_INL_SEL")
        .ok_or_else(|| EngineError::Range {
            param: "run".into(),
            msg: "FS_A_INL_SEL missing".into(),
        })?;
    let mut h2 = rel_check(
        "FS_H2_RETURN_CLOSURE".into(),
        "H2 product at 34.0 MMSCFD, Case A INL_SEL",
        suite.closure.h2_target_kg_h,
        a_inl.h2_product_kg_h,
        1.0e-9,
    );
    h2.source = "TEV-953 doc p15 (PDF p16): 130 MMSCFD = 29,000 lb/h".into();
    h2.locator = "TEV-961 Table 2 doc p16 for 34.0 MMSCFD".into();
    h2.conditions = format!(
        "H2 return fraction {:.6}, calculated by closure",
        suite.closure.h2_return
    );
    h2.unit = "kg/h".into();
    h2.data_use = "calibration";
    h2.explanation = format!(
        "The cryogenic H2 return fraction is unpublished and is solved here so the pair \
         holds; it is carried unchanged to REC_F. Bracket: {:.1} kg/h at 0, {:.1} kg/h at 1",
        suite.closure.h2_at_zero_kg_h, suite.closure.h2_at_one_kg_h
    );
    out.push(h2);

    let mut closure_ok = true;
    for r in &suite.runs {
        let e = envelope(r)?;
        let worst = e.element_rel.iter().cloned().fold(0.0, f64::max);
        for (what, v, tol, card, test, why) in [
            (
                "ELEMENTS",
                worst,
                ELEMENT_TOL,
                "A.5",
                "flowsheet_elements_close",
                "Largest relative C, H, O or N residual over the envelope streams, with \
                 the N2-BURN combustion O2 counted as an input. A code check, so it earns \
                 no validation credit",
            ),
            (
                "ENERGY",
                e.energy_rel,
                ENERGY_TOL,
                "A.8",
                "flowsheet_energy_closes",
                "Holds by construction: the capture island, REC-CRYO, CO2-COMP and N2-BURN \
                 set their duties by closure with no energy model at M7, so the row tests \
                 the bookkeeping and earns no validation credit",
            ),
        ] {
            closure_ok &= v <= tol;
            out.push(Check {
                test_id: format!("FS_ENVELOPE_{what}_{}", r.id),
                level: 3,
                subject: format!("Envelope closure, {}", what.to_lowercase()),
                source: "Code verification".into(),
                locator: format!("card {card}; test {test}, engine/plant/tests/m7_flowsheet.rs"),
                // The pass count is left out: the natural gas secant warm-starts
                // each tear, so the count of the last solve follows round-off and
                // differs between build profiles while the result does not.
                conditions: format!("{}, tears converged", r.id),
                literature: Some(0.0),
                simulated: v,
                unit: "relative".into(),
                abs_error: Some(v),
                rel_error: None,
                tolerance: tol,
                pass: v <= tol,
                data_use: "same_dataset_disclosed",
                explanation: why.into(),
            });
        }
    }

    let spec = FlowsheetSpec::new(
        m,
        Case::B,
        Scenario::RecF,
        FeedBasis::NaturalGas(ng_ref),
        suite.closure.h2_return,
    );
    let (spread, passes) = three_start_spread(m, &spec)?;
    out.push(Check {
        test_id: "FS_TEAR_THREE_STARTS".into(),
        level: 3,
        subject: "Tear answer from three starts".into(),
        source: "Code verification".into(),
        locator: "test tear_converges_from_three_starts, engine/plant/tests/m7_flowsheet.rs".into(),
        conditions: format!(
            "Case B REC_F at {ng_ref:.4} MMSCFD; starts zero, anchored, double; passes {} {} {}",
            passes[0], passes[1], passes[2]
        ),
        literature: Some(0.0),
        simulated: spread,
        unit: "relative".into(),
        abs_error: Some(spread),
        rel_error: None,
        tolerance: THREE_START_TOL,
        pass: spread <= THREE_START_TOL,
        data_use: "same_dataset_disclosed",
        explanation: "Largest relative gap over the S03 species and the feed and product \
                      totals. A code check, so it earns no validation credit"
            .into(),
    });

    let gate_pass = closure_ok && carbon_failed.is_empty() && spread <= THREE_START_TOL;
    out.push(Check {
        test_id: "FS_GATE_M7".into(),
        level: 3,
        subject: "M7 acceptance of the Level 3 flowsheet".into(),
        source: "Level 3 flowsheet acceptance: every envelope closes, every carbon line sits \
                 within 2% of the anchored ledger and the tear answer holds from three starts \
                 (decision S11, docs/VALIDATION_MATRIX.md)"
            .into(),
        locator: "engine/plant/tests/m7_flowsheet.rs".into(),
        conditions: "All flowsheet runs; carbon at Case B REC_F".into(),
        literature: Some(0.0),
        simulated: carbon_failed.len() as f64 + if closure_ok { 0.0 } else { 1.0 },
        unit: "failed parts".into(),
        abs_error: None,
        rel_error: None,
        tolerance: 0.0,
        pass: gate_pass,
        data_use: "same_dataset_disclosed",
        explanation: if carbon_failed.is_empty() {
            "Every part passed".into()
        } else {
            format!(
                "Carbon lines outside 2%: {}. The terms gated on M7 cannot \
                 flip on this gate",
                carbon_failed.join(" ")
            )
        },
    });
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_j_05_lumped_feed_keeps_the_registry_carbon() {
        let m = Model::embedded().unwrap();
        assert!((ng_carbon_per_mol() - m.c_per_mol).abs() < 1e-12);
        let s = natural_gas(34.0).unwrap();
        let want = nush2_props::units::mmscfd_to_kmol_h(34.0) * m.c_per_mol;
        assert!((carbon_kmol_h(&s) / want - 1.0).abs() < 1e-12);
    }

    #[test]
    fn eq_j_07_recycle_fraction_matches_the_golden() {
        let m = Model::embedded().unwrap();
        let f = recycle_fraction(&m);
        assert!((f - 0.752).abs() < 0.0005, "{f}");
    }

    #[test]
    fn eq_a_06_reformer_has_two_independent_reactions() {
        let sp = [Sp::Ch4, Sp::H2o, Sp::Co, Sp::H2, Sp::Co2];
        assert_eq!(reaction_rank(&sp).unwrap(), 2);
        assert_eq!(
            reaction_rank(&[Sp::Co, Sp::H2o, Sp::Co2, Sp::H2]).unwrap(),
            1
        );
    }

    #[test]
    fn eq_j_16_eq_j_20_eq_j_21_anchored_island_rows() {
        let m = Model::embedded().unwrap();
        let mea = AnchoredMea::from_model(&m);
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        // Same captured CO2 as the anchored ledger gives the anchored rows.
        let cap_mol_s = b.c.cap * m.s_b / 3.6;
        let q = mea.reboiler_w(cap_mol_s);
        assert!((q / 1e6 - b.q_reb).abs() < 1e-9, "{} {}", q / 1e6, b.q_reb);
        let n_mea = cap_mol_s / (m.rich - m.lean);
        assert!((mea.pump_w(n_mea) / 1e6 - b.e_rows.mea_pump).abs() < 1e-9);
        assert!((mea.fans_w(q) / 1e6 - b.e_rows.mea_cooling).abs() < 1e-9);
    }

    #[test]
    fn bad_specs_are_refused() {
        let m = Model::embedded().unwrap();
        let ok = FlowsheetSpec::new(
            &m,
            Case::B,
            Scenario::RecF,
            FeedBasis::NaturalGas(23.0),
            0.5,
        );
        assert!(ok.validate(&m).is_ok());
        let mut s = ok.clone();
        s.h2_return = 1.5;
        assert!(s.validate(&m).is_err());
        let mut s = ok.clone();
        s.scenario = Scenario::CombH;
        assert!(s.validate(&m).is_err());
        let mut s = ok;
        s.basis = FeedBasis::Hydrogen(-1.0);
        assert!(s.validate(&m).is_err());
        assert!(natural_gas(0.0).is_err());
    }
}
