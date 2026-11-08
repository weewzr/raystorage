//! Case B capture train at M6: absorber at 90% per pass, rich/lean exchanger,
//! stripper, lean cooler and lean pump, run on the M4 gas chain.
//!
//! Feed. The absorber gas is the Case 5 knock-out drum vapour from the M4 chain
//! (40 C, 389 psia), scaled on carbon to the Case B reformer tube carbon, the same
//! scaling `tube_case_b::feed_total` uses. Case 6 stream sheets are absent from the
//! local PDF, so Case 5 composition stands in.
//!
//! Inputs held here repeat the parameter file, since this crate cannot read the
//! plant crate: absorber 26.8 bar (`P_ABSORBER`) and 40 C (decision D28),
//! stripper 2.0 bar (`P_STRIPPER`), lean loading 0.25 (`LOADING_LEAN`), rich cap
//! 0.50 (`LOADING_RICH`), 30 mass% MEA (`W_MEA`), pump and motor efficiencies 0.75
//! and 0.9 (`ETA_PUMP`, `ETA_MOTOR`), cooling 0.015 MWe per MW (`K_COOLING`).
//!
//! Assumptions of this module, each stated where it is used: RSP-250 geometry for
//! the packing, a liquid load of 40 m3/m2/h for the diameter, 12 equilibrium
//! stages with Murphree efficiency 1, a 10 K exchanger approach (I.10), a
//! condenser at 40 C, and one enthalpy path temperature of 110 C for every unit
//! of the island (`T_PHI_ISLAND_K`).
//!
//! One path temperature. The liquid enthalpy of `Solvent::h_liquid_w` heats the
//! loaded liquid at a constant 3.6 kJ/kg/K, so two path temperatures give two
//! different enthalpies for one state. The absorber at 50 C and the stripper at
//! 110 C put 2.46 MW (5% of the reboiler duty) into the hand-offs between them.
//! The island now uses the stripper's 110 C for the absorber, the exchanger, the
//! stripper and the cooler, so it closes on energy (`island_energy_residual_w`).
//! The stripper is then the model the stripper benchmarks judge, which also run at
//! 110 C. The cost is in the absorber: its heat of absorption is carried from
//! 110 C down to the column temperature along the constant heat capacity path,
//! which adds the Kirchhoff term of that path to the Gibbs-Helmholtz value.

use crate::absorber::{self, AbsorberResult, AbsorberSpec, GasFilm};
use crate::case5::{self, TrainOpts};
use crate::packing::RSP_250;
use crate::regen;
use crate::solvent::{Solvent, MW_CO2};
use crate::stream::{Evidence, SolventStream, Sp, Stream};
use crate::stripper::{self, StripMode, StripperResult, StripperSpec};
use crate::tube_case_b;
use nush2_mea::aronu::{MW_MEA, MW_WATER};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;

pub const P_ABS_PA: f64 = 26.8e5;
pub const T_LEAN_K: f64 = 313.15;
pub const P_STRIP_PA: f64 = 2.0e5;
pub const ALPHA_LEAN: f64 = 0.25;
pub const ALPHA_RICH_CAP: f64 = 0.50;
pub const W_MEA: f64 = 0.30;
pub const ETA_PUMP: f64 = 0.75;
pub const ETA_MOTOR: f64 = 0.9;
pub const K_COOLING_MWE_PER_MW: f64 = 0.015;
pub const CAPTURE_SPEC: f64 = 0.90;
/// Section 7 limit on packed height, m.
pub const HEIGHT_LIMIT_M: f64 = 40.0;
/// Assumed design liquid load, m3/m2/h, the low end of H.29.
pub const LIQUID_LOAD_M3_M2_H: f64 = 40.0;
/// The other end of H.29, run as a sensitivity.
pub const LIQUID_LOAD_HIGH_M3_M2_H: f64 = 100.0;
pub const APPROACH_K: f64 = 10.0;
pub const T_COND_K: f64 = 313.15;
pub const STAGES: usize = 12;
pub const MURPHREE: f64 = 1.0;
/// Enthalpy path temperature of the stripper benchmarks and of the island, K.
pub const T_PHI_STRIP_K: f64 = 383.15;
/// Enthalpy path temperature of every unit in the Case B island, K. One value
/// keeps the liquid enthalpy a single function across the hand-offs.
pub const T_PHI_ISLAND_K: f64 = T_PHI_STRIP_K;
pub const SLICES: usize = 40;

/// Absorber feed gas for Case B.
pub fn absorber_feed() -> Result<Stream> {
    let rf = case5::reformer(KSource::Zhang)?;
    let train = case5::train(&rf.outlet, &TrainOpts::sized_on_ieaghg(KSource::Zhang)?)?;
    let c5 = case5::reformer_feed()?.elements()?[0];
    let cb = tube_case_b::feed_total()?.elements()?[0];
    let k = cb / c5;
    let mut s = train.drum.vapour.clone();
    s.n = s.n.map(|v| v * k);
    s.tag = "ABS_FEED".into();
    Ok(s)
}

/// Lean solvent sized by H.28 so that 90% capture brings the rich loading to the
/// 0.50 cap.
pub fn lean_solvent(feed: &Stream) -> Result<SolventStream> {
    let co2 = CAPTURE_SPEC * feed.flow(Sp::Co2);
    let m_free = absorber::solvent_rate(co2, W_MEA, ALPHA_RICH_CAP, ALPHA_LEAN)?;
    let n_mea = m_free * W_MEA / MW_MEA;
    Ok(SolventStream {
        tag: "LEAN".into(),
        t_k: T_LEAN_K,
        p_pa: P_ABS_PA,
        n_mea,
        n_h2o: m_free * (1.0 - W_MEA) / MW_WATER,
        n_co2: ALPHA_LEAN * n_mea,
    })
}

fn mass(s: &SolventStream) -> f64 {
    s.n_mea * MW_MEA + s.n_h2o * MW_WATER + s.n_co2 * MW_CO2
}

/// Absorber specification at a given liquid load, height to be solved, with the
/// central MEA diffusivity.
pub fn absorber_spec(lean: &SolventStream, load_m3_m2_h: f64) -> Result<AbsorberSpec> {
    let solvent = Solvent::new(W_MEA, T_PHI_ISLAND_K)?;
    let rho = solvent.density(lean.t_k, lean.n_co2 / lean.n_mea)?;
    Ok(AbsorberSpec {
        tag: "ABS".into(),
        packing: RSP_250,
        diameter_m: absorber::diameter(mass(lean) / rho, load_m3_m2_h)?,
        height_m: 10.0,
        slices: SLICES,
        p_pa: P_ABS_PA,
        gas_film: GasFilm::RochaSyngas(absorber::chinchen_syngas()),
        t_phi_k: T_PHI_ISLAND_K,
        w_mea: W_MEA,
        d_mea: absorber::DUGAS_7M_LEAN,
    })
}

/// Case B absorber at one liquid load: packed height for 90% per pass, or `None`
/// when 40 m falls short.
#[derive(Debug, Clone, PartialEq)]
pub struct AbsorberCase {
    pub load_m3_m2_h: f64,
    pub diameter_m: f64,
    pub height_m: Option<f64>,
    pub result: AbsorberResult,
}

pub fn absorber_case(feed: &Stream, lean: &SolventStream, load: f64) -> Result<AbsorberCase> {
    absorber_case_with(feed, lean, load, absorber::DUGAS_7M_LEAN)
}

/// As `absorber_case`, with a chosen MEA diffusivity.
pub fn absorber_case_with(
    feed: &Stream,
    lean: &SolventStream,
    load: f64,
    d_mea: absorber::MeaDiffusivity,
) -> Result<AbsorberCase> {
    let spec = AbsorberSpec {
        d_mea,
        ..absorber_spec(lean, load)?
    };
    let d = spec.diameter_m;
    let (z, r) =
        absorber::height_for_capture(&spec, feed, lean, CAPTURE_SPEC, 0.5, HEIGHT_LIMIT_M)?;
    Ok(AbsorberCase {
        load_m3_m2_h: load,
        diameter_m: d,
        height_m: z,
        result: r,
    })
}

/// Regeneration of a rich solvent with the exchanger coupled to the reboiler
/// outlet, lean loading fixed.
#[derive(Debug, Clone, PartialEq)]
pub struct Regeneration {
    pub exchanger: regen::RichLean,
    pub stripper: StripperResult,
    pub lean_cooler_w: f64,
    pub srd_gj_t: f64,
    pub split: regen::SrdSplit,
}

pub fn stripper_spec(mode: StripMode) -> StripperSpec {
    StripperSpec {
        tag: "STRIP".into(),
        stages: STAGES,
        murphree: MURPHREE,
        p_top_pa: P_STRIP_PA,
        p_bottom_pa: P_STRIP_PA,
        t_cond_k: T_COND_K,
        t_phi_k: T_PHI_ISLAND_K,
        w_mea: W_MEA,
        mode,
    }
}

/// Rich solvent through the exchanger, the stripper and the lean cooler. The
/// exchanger is sized on the reboiler outlet with the 10 K approach; once the
/// reboiler temperature and the lean mass settle, the lean side takes exactly the
/// duty that heated the rich feed the stripper solved, so no heat is lost at the
/// hand-off. The approach then differs from 10 K by the last step of the loop.
pub fn regenerate(rich: &SolventStream, alpha_lean: f64) -> Result<Regeneration> {
    let spec = stripper_spec(StripMode::LeanLoading(alpha_lean));
    let solvent = Solvent::new(W_MEA, T_PHI_ISLAND_K)?;
    let mut t_reb = stripper::t_reboiler(&solvent, alpha_lean, P_STRIP_PA)?;
    let m_rich = mass(rich);
    let mut m_lean = m_rich - rich.n_mea * (rich.n_co2 / rich.n_mea - alpha_lean) * MW_CO2;
    for _ in 0..30 {
        let x = regen::rich_lean(m_rich, rich.t_k, m_lean, t_reb, APPROACH_K)?;
        let hot = SolventStream {
            t_k: x.t_rich_out_k,
            p_pa: P_STRIP_PA,
            ..rich.clone()
        };
        let s = stripper::stages(&spec, &hot)?;
        let m_out = mass(&s.lean);
        let done = (s.t_reb_k - t_reb).abs() < 1e-3 && (m_out - m_lean).abs() < 1e-9 * m_rich;
        t_reb = s.t_reb_k;
        m_lean = m_out;
        if done {
            let x = regen::rich_lean_at_duty(m_rich, rich.t_k, m_lean, t_reb, x.duty_w)?;
            let cooler = regen::lean_cooler_w(m_lean, x.t_lean_out_k, T_LEAN_K);
            let m_co2 = s.co2_stripped_mol_s * MW_CO2;
            let d_alpha = rich.n_co2 / rich.n_mea - alpha_lean;
            let phi_r = solvent.phi_j_mol_mea(rich.n_co2 / rich.n_mea)?;
            let phi_l = solvent.phi_j_mol_mea(alpha_lean)?;
            let q_des = (phi_r - phi_l) / d_alpha / MW_CO2;
            let q_sens =
                m_rich * crate::solvent::CP_SOLVENT_J_KG_K * (s.t_reb_k - x.t_rich_out_k) / m_co2;
            return Ok(Regeneration {
                exchanger: x,
                lean_cooler_w: cooler,
                srd_gj_t: s.srd_j_kg / 1.0e6,
                split: regen::srd_split(s.srd_j_kg, q_des, q_sens),
                stripper: s,
            });
        }
    }
    Err(EngineError::NotConverged(
        "rich/lean exchanger and reboiler temperature".into(),
    ))
}

/// Energy residual of the capture island, W: what enters (absorber feed gas,
/// reboiler heat, and the lean make-up that turns the cooled stripper lean into
/// the absorber lean) less what leaves (treated gas, CO2 vent, condenser and lean
/// cooler). Every liquid uses the island solvent at `T_PHI_ISLAND_K`, and gases
/// use the ideal-gas enthalpy the columns use.
///
/// eq: A.8
pub fn island_energy_residual_w(
    feed: &Stream,
    lean: &SolventStream,
    abs: &AbsorberResult,
    reg: &Regeneration,
) -> Result<f64> {
    let solvent = Solvent::new(W_MEA, T_PHI_ISLAND_K)?;
    let h_liq = |s: &SolventStream| solvent.h_liquid_w(s.n_mea, s.n_h2o, s.n_co2, s.t_k);
    let cooled = SolventStream {
        t_k: T_LEAN_K,
        ..reg.stripper.lean.clone()
    };
    let make_up = h_liq(lean)? - h_liq(&cooled)?;
    let h_in = crate::solvent::h_gas_w(&feed.n, feed.t_k)? + reg.stripper.q_reb_w + make_up;
    let h_out = crate::solvent::h_gas_w(&abs.gas_out.n, abs.gas_out.t_k)?
        + crate::solvent::h_gas_w(&reg.stripper.vent.n, reg.stripper.vent.t_k)?
        + reg.stripper.q_cond_w
        + reg.lean_cooler_w;
    Ok(h_in - h_out)
}

/// Modelled values of the four M6 terms on Case B, with what fed them.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnTerms {
    /// T02: per-pass capture reached, the approved 0.90 spec.
    pub t02_capture: f64,
    /// T02: packed height for 90%, m; `None` when 40 m falls short.
    pub t02_height_m: Option<f64>,
    /// T02: rich loading leaving the absorber at that height.
    pub t02_rich_loading: f64,
    pub absorber_diameter_m: f64,
    /// Height at the 100 m3/m2/h liquid load, m, sensitivity.
    pub t02_height_high_load_m: Option<f64>,
    /// Height with the second Dugas MEA diffusivity, m, sensitivity.
    pub t02_height_dugas_rich_m: Option<f64>,
    /// T06: modelled SRD, GJ/t, printed beside the approved 3.8.
    pub t06_srd_gj_t: Option<f64>,
    pub t06_approved_gj_t: f64,
    /// T07: lean pump, MWe.
    pub t07_lean_pump_mwe: Option<f64>,
    /// T08: cooling load at the anchored k on lean cooler plus condenser, MWe.
    pub t08_cooling_mwe: Option<f64>,
    pub evidence_t07_t08: Evidence,
    pub feed_co2_kmol_h: f64,
    pub lean_solvent_t_h: f64,
    pub reboiler_mw: Option<f64>,
    pub condenser_mw: Option<f64>,
    pub lean_cooler_mw: Option<f64>,
    /// Rich/lean exchanger duty, MW.
    pub rich_lean_mw: Option<f64>,
    pub reboiler_t_c: Option<f64>,
    pub split_gj_t: Option<regen::SrdSplit>,
    /// Section 7 findings raised by this run, plain text.
    pub findings: Vec<String>,
    /// Failures of a column after the fallback ladder.
    pub failures: Vec<String>,
}

/// Heat duties of the capture island at the Case B design point, MW.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IslandDuties {
    pub reboiler_mw: f64,
    pub condenser_mw: f64,
    pub lean_cooler_mw: f64,
    pub rich_lean_mw: f64,
}

/// The capture island at the Case B design point: its duties and the column
/// profiles of the same run, absorber slices from the bottom of the packing and
/// stripper stages from the top.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignIsland {
    pub duties: IslandDuties,
    pub absorber: Vec<absorber::SliceState>,
    pub stripper: Vec<stripper::StageState>,
}

/// The central absorber case of `column_terms` and its regeneration, without the
/// sensitivity cases, so the island costs one absorber solve. The duties equal
/// the duties `column_terms` reports.
pub fn design_island() -> Result<DesignIsland> {
    let feed = absorber_feed()?;
    let lean = lean_solvent(&feed)?;
    let abs = absorber_case(&feed, &lean, LIQUID_LOAD_M3_M2_H)?;
    let reg = regenerate(&abs.result.rich, ALPHA_LEAN)?;
    Ok(DesignIsland {
        duties: IslandDuties {
            reboiler_mw: reg.stripper.q_reb_w / 1.0e6,
            condenser_mw: reg.stripper.q_cond_w / 1.0e6,
            lean_cooler_mw: reg.lean_cooler_w / 1.0e6,
            rich_lean_mw: reg.exchanger.duty_w / 1.0e6,
        },
        absorber: abs.result.slices,
        stripper: reg.stripper.stages,
    })
}

/// The duties of `design_island`.
pub fn design_duties() -> Result<IslandDuties> {
    design_island().map(|i| i.duties)
}

/// Run Case B and return the M6 term values. A column that fails after the
/// fallback ladder leaves T06 to T08 empty and marks T07 and T08 `GateFailed`.
pub fn column_terms() -> Result<ColumnTerms> {
    let feed = absorber_feed()?;
    let lean = lean_solvent(&feed)?;
    let mut findings = Vec::new();
    let mut failures = Vec::new();
    let central = absorber_case(&feed, &lean, LIQUID_LOAD_M3_M2_H);
    let high = absorber_case(&feed, &lean, LIQUID_LOAD_HIGH_M3_M2_H);
    let mut t = ColumnTerms {
        t02_capture: f64::NAN,
        t02_height_m: None,
        t02_rich_loading: f64::NAN,
        absorber_diameter_m: f64::NAN,
        t02_height_high_load_m: None,
        t02_height_dugas_rich_m: None,
        t06_srd_gj_t: None,
        t06_approved_gj_t: regen::SRD_CARRIED_GJ_T,
        t07_lean_pump_mwe: None,
        t08_cooling_mwe: None,
        evidence_t07_t08: Evidence::NotRun,
        feed_co2_kmol_h: feed.flow(Sp::Co2) * 3.6,
        lean_solvent_t_h: mass(&lean) * 3.6,
        reboiler_mw: None,
        condenser_mw: None,
        lean_cooler_mw: None,
        rich_lean_mw: None,
        reboiler_t_c: None,
        split_gj_t: None,
        findings: Vec::new(),
        failures: Vec::new(),
    };
    match &high {
        Ok(h) => t.t02_height_high_load_m = h.height_m,
        Err(e) => failures.push(format!("absorber at 100 m3/m2/h: {e}")),
    }
    match absorber_case_with(&feed, &lean, LIQUID_LOAD_M3_M2_H, absorber::DUGAS_7M_RICH) {
        Ok(h) => t.t02_height_dugas_rich_m = h.height_m,
        Err(e) => failures.push(format!("absorber with the second Dugas diffusivity: {e}")),
    }
    let abs = match central {
        Ok(a) => a,
        Err(e) => {
            failures.push(format!("absorber at 40 m3/m2/h: {e}"));
            t.evidence_t07_t08 = Evidence::GateFailed;
            t.failures = failures;
            return Ok(t);
        }
    };
    t.t02_capture = abs.result.capture;
    t.t02_height_m = abs.height_m;
    t.t02_rich_loading = abs.result.rich.n_co2 / abs.result.rich.n_mea;
    t.absorber_diameter_m = abs.diameter_m;
    match abs.height_m {
        None => findings.push(format!(
            "90% per pass is not reached within {HEIGHT_LIMIT_M} m: {:.1}% at {HEIGHT_LIMIT_M} m",
            100.0 * abs.result.capture
        )),
        Some(z) if z > HEIGHT_LIMIT_M => findings.push(format!("90% needs {z:.1} m of packing")),
        _ => {}
    }
    if t.t02_rich_loading > ALPHA_RICH_CAP + 1e-6 {
        findings.push(format!(
            "rich loading {:.3} exceeds 0.50",
            t.t02_rich_loading
        ));
    }
    let reg = match regenerate(&abs.result.rich, ALPHA_LEAN) {
        Ok(r) => r,
        Err(e) => {
            failures.push(format!("stripper: {e}"));
            t.evidence_t07_t08 = Evidence::GateFailed;
            t.findings = findings;
            t.failures = failures;
            return Ok(t);
        }
    };
    let srd = reg.srd_gj_t;
    if !(regen::SRD_LOW_GJ_T..=regen::SRD_HIGH_GJ_T).contains(&srd) {
        findings.push(format!(
            "modelled SRD {srd:.2} GJ/t lies outside 3.4 to 4.2; 3.8 is kept"
        ));
    }
    let solvent = Solvent::new(W_MEA, T_PHI_ISLAND_K)?;
    let lean_out = &reg.stripper.lean;
    let rho = solvent.density(T_LEAN_K, lean_out.n_co2 / lean_out.n_mea)?;
    let pump = regen::lean_pump_w(
        mass(lean_out),
        rho,
        P_ABS_PA - P_STRIP_PA,
        ETA_PUMP,
        ETA_MOTOR,
    )?;
    let cooling = K_COOLING_MWE_PER_MW * (reg.lean_cooler_w + reg.stripper.q_cond_w) / 1.0e6;
    t.t06_srd_gj_t = Some(srd);
    t.t07_lean_pump_mwe = Some(pump / 1.0e6);
    t.t08_cooling_mwe = Some(cooling);
    t.reboiler_mw = Some(reg.stripper.q_reb_w / 1.0e6);
    t.condenser_mw = Some(reg.stripper.q_cond_w / 1.0e6);
    t.lean_cooler_mw = Some(reg.lean_cooler_w / 1.0e6);
    t.rich_lean_mw = Some(reg.exchanger.duty_w / 1.0e6);
    t.reboiler_t_c = Some(reg.stripper.t_reb_k - 273.15);
    t.split_gj_t = Some(regen::SrdSplit {
        q_des: reg.split.q_des / 1.0e6,
        q_sens: reg.split.q_sens / 1.0e6,
        q_strip: reg.split.q_strip / 1.0e6,
    });
    t.findings = findings;
    t.failures = failures;
    Ok(t)
}
