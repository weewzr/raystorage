//! Named screening terms. Each rigorous model fills its `modelled` slot, and a
//! term leaves its anchored value only in the presets whose rule allows it and
//! only when its gate passed (module `gates`).
//!
//! Presets, plan section 7. `Anchored` uses no model. `Headline` flips T04,
//! T05, T07 and T08 on a passed gate and keeps every other term anchored, for
//! the reason the term table gives. `Modelled` uses every model whose gate
//! passed and which has a flip path into the anchored balance.
//!
//! A flip enters the balance through the registry input the term replaces
//! (T01 `PHI_RECYCLE`, T02 `ETA_MEA`, T05 `Q_EX4_ABOVE_130C`, T06
//! `SRD_CENTRAL`, T10 `CIRC_CENTRAL_DP_SEC_KPA`) or, for a term with no single
//! input (T04, T07, T08), through a factor equal to the modelled over the
//! anchored value at Case B REC_F, carried to every run. T03, T09, T11, T12 and
//! T13 have no flip path: their modelled slots are cross-checks on another
//! basis.

use crate::anchored::{Balance, Model, Scenario};
use crate::gates::{self, GateState, Gates};
use crate::params::ParamSet;
use crate::Preset;
use nush2_numerics::{EngineError, Result};
use nush2_units::bench::GasTerms;
use nush2_units::columns::ColumnTerms;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Source {
    Anchored,
    Modelled,
}

/// What the Headline preset does with a term, plan section 7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HeadlineRule {
    /// Flips when its gate passes, whichever way the number moves.
    Flips,
    /// Stays anchored, for the reason given.
    Stays(&'static str),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Term {
    pub id: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub anchored: f64,
    pub modelled: Option<f64>,
    /// How the modelled slot was computed.
    pub modelled_note: &'static str,
    pub used: Source,
    pub gate: &'static str,
    /// `None` for a term with no gate (T11, T12).
    pub gate_state: Option<GateState>,
    pub headline: HeadlineRule,
}

impl Term {
    /// The value the preset uses.
    pub fn used_value(&self) -> f64 {
        match (self.used, self.modelled) {
            (Source::Modelled, Some(v)) => v,
            _ => self.anchored,
        }
    }
}

fn rule(id: &str) -> HeadlineRule {
    match id {
        "T04" | "T05" | "T07" | "T08" => HeadlineRule::Flips,
        "T01" => HeadlineRule::Stays("tail-gas carbon split sits in the external block"),
        "T02" => HeadlineRule::Stays("approved spec; the absorber reports height and rich loading"),
        "T03" => HeadlineRule::Stays("published, named in the basis"),
        "T06" => HeadlineRule::Stays("approved, range 3.4 to 4.2"),
        "T09" => HeadlineRule::Stays("INL stages are vendor tuned"),
        "T10" => HeadlineRule::Stays("tube bundle is an assumed design"),
        "T13" => HeadlineRule::Stays("published pair"),
        _ => HeadlineRule::Stays("no replacing model"),
    }
}

fn term(
    id: &'static str,
    name: &'static str,
    unit: &'static str,
    v: f64,
    gate: &'static str,
) -> Term {
    Term {
        id,
        name,
        unit,
        anchored: v,
        modelled: None,
        modelled_note: "",
        used: Source::Anchored,
        gate,
        gate_state: None,
        headline: rule(id),
    }
}

fn set(terms: &mut [Term], id: &str, v: Option<f64>, note: &'static str) {
    if let Some(t) = terms.iter_mut().find(|t| t.id == id) {
        if let Some(x) = v.filter(|x| x.is_finite()) {
            t.modelled = Some(x);
            t.modelled_note = note;
        }
    }
}

/// Fill the slots of the terms gated on M4. T05 is carried from Case 5 to the
/// plant on the same CO2 scale as the anchored value.
pub fn fill_m4(terms: &mut [Term], m: &Model, s: f64, g: &GasTerms) {
    set(
        terms,
        "T01",
        Some(g.t01_phi),
        "SMR-EQ, HTS and LTS on TEV-961 Case 5, bed sized to the IEAGHG approach",
    );
    set(
        terms,
        "T05",
        Some(g.t05_case5_mw * t05_scale(m, s)),
        "Case 5 chain from SMR-EQ, 149 to 130 C with condensation, on the plant CO2 scale",
    );
}

/// Scale from the Case 5 syngas term to the plant, the same as the anchored
/// `Q_EX4_ABOVE_130C` takes.
pub fn t05_scale(m: &Model, s: f64) -> f64 {
    m.co2_cap_c6 / m.case5_cap * s
}

/// Fill the slots of the terms gated on M6 from the Case B column run.
pub fn fill_m6(terms: &mut [Term], c: &ColumnTerms) {
    set(
        terms,
        "T02",
        Some(c.t02_capture),
        "absorber solved to 90% per pass on the RSP-250 stand-in geometry",
    );
    set(
        terms,
        "T06",
        c.t06_srd_gj_t,
        "equilibrium-stage stripper and regeneration train on the Case B rich solvent",
    );
    set(
        terms,
        "T07",
        c.t07_lean_pump_mwe,
        "lean pump on the M6 solvent flow and density",
    );
    set(
        terms,
        "T08",
        c.t08_cooling_mwe,
        "lean cooler plus condenser duty at the anchored k",
    );
}

/// Fill the slots of the terms gated on M7.
pub fn fill_m7(terms: &mut [Term], slots: &crate::flowsheet::TermSlots) {
    set(
        terms,
        "T03",
        Some(slots.t03_mw),
        "Level 3 preheat plus reformer at Case B REC_F; upstream preheat incomplete",
    );
    set(
        terms,
        "T04",
        Some(slots.t04_mw),
        "process heat of REC_F less INL_SEL at the same H2, Level 3 flowsheet",
    );
    set(
        terms,
        "T09",
        Some(slots.t09_mw),
        "isothermal ideal-gas limit, MW of shaft work, a cross-check on another basis",
    );
    set(
        terms,
        "T13",
        Some(slots.t13_mmscfd),
        "natural gas for 8,988 kg/h H2 with the S03 tear, Level 3 flowsheet",
    );
}

/// Fill T10 from the Level 4 loop drop.
pub fn fill_m8(terms: &mut [Term], t10_kpa: f64) {
    set(
        terms,
        "T10",
        Some(t10_kpa),
        "smooth-wall Blasius on the HA reformer annulus at 199 tubes plus HTTR-size IHX \
         tubes; a lower bound",
    );
}

/// True when the term has a path into the anchored balance.
pub fn has_flip_path(id: &str) -> bool {
    matches!(
        id,
        "T01" | "T02" | "T04" | "T05" | "T06" | "T07" | "T08" | "T10"
    )
}

/// True when `preset` would use the model of term `id` on gate outcome `g`,
/// before checking that a modelled value exists.
pub fn wants_model(preset: Preset, id: &str, g: Option<GateState>) -> bool {
    let passed = g == Some(GateState::Passed);
    match preset {
        Preset::Anchored => false,
        Preset::Headline => passed && rule(id) == HeadlineRule::Flips,
        Preset::Modelled => passed && has_flip_path(id),
    }
}

/// Set the gate outcome and the source of every term for a preset.
pub fn resolve(terms: &mut [Term], preset: Preset, gates: &Gates) {
    for t in terms.iter_mut() {
        t.gate_state = gates.of(t.id);
        t.used = if wants_model(preset, t.id, t.gate_state) && t.modelled.is_some() {
            Source::Modelled
        } else {
            Source::Anchored
        };
    }
}

/// The thirteen terms of the replacement table, from a balance.
pub fn terms(m: &Model, b: &Balance) -> Vec<Term> {
    let e = &b.e_rows;
    vec![
        term("T01", "Carbon split after shift", "-", m.phi_rec, "M4"),
        term("T02", "Per-pass capture", "-", m.eta_mea, "M6"),
        term("T03", "Process heat", "MW", b.q_proc, "M4"),
        term("T04", "Recycle slip heat", "MW", b.q_recyc, "M7"),
        term("T05", "Syngas above 130 C", "MW", b.stack.ex4, "M4"),
        term("T06", "Specific reboiler duty", "GJ/t", b.srd, "M6"),
        term("T07", "Lean pump", "MWe", e.mea_pump, "M6"),
        term("T08", "MEA cooling load", "MWe", e.mea_cooling, "M6"),
        term("T09", "CO2 compression", "MWe", e.co2comp, "M7"),
        term(
            "T10",
            "Secondary circulator pressure drop",
            "kPa",
            m.circ_dp_central().0,
            "M8",
        ),
        term(
            "T11",
            "Primary circulator pressure drop",
            "kPa",
            m.circ_dp_central().1,
            "none",
        ),
        term("T12", "Losses", "MW", b.q_ihx + b.q_duct + b.qv, "none"),
        term("T13", "Natural gas", "MMSCFD", b.c.ng * b.s, "M7"),
    ]
}

/// The M4 terms, computed once per process: the Case 5 chain is fixed by the
/// printed basis and reads no registry input.
pub fn gas_terms_cached() -> Result<GasTerms> {
    static GAS: OnceLock<Result<GasTerms>> = OnceLock::new();
    GAS.get_or_init(nush2_units::bench::gas_terms).clone()
}

/// The M6 terms, computed once per process; the Case B column run reads no
/// registry input.
pub fn column_terms_cached() -> Result<ColumnTerms> {
    static COLS: OnceLock<Result<ColumnTerms>> = OnceLock::new();
    COLS.get_or_init(nush2_units::bench::column_terms).clone()
}

/// Terms of a preset on the base registry: anchored values from `base`, the
/// modelled slots the preset needs, and the source of each. Slots no flip needs
/// are left to the exporter, which fills all of them for the record.
pub fn for_preset(base: &ParamSet, preset: Preset, gates: &Gates) -> Result<Vec<Term>> {
    let m = Model::new(base)?;
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let mut list = terms(&m, &b);
    let need = |id: &str| wants_model(preset, id, gates.of(id));
    if need("T01") || need("T05") {
        fill_m4(&mut list, &m, m.s_b, &gas_terms_cached()?);
    }
    if ["T02", "T06", "T07", "T08"].iter().any(|id| need(id)) {
        fill_m6(&mut list, &column_terms_cached()?);
    }
    if need("T04") {
        let suite = crate::flowsheet::suite(&m)?;
        fill_m7(&mut list, &suite.terms);
    }
    if need("T10") {
        let l4 = crate::utility::level4(&m, &b, crate::flowsheet::T_REFORMER_OUT_C)?;
        fill_m8(&mut list, l4.t10.total_kpa);
    }
    resolve(&mut list, preset, gates);
    Ok(list)
}

/// Registry with the flips of `terms` applied to `base`. A term the preset uses
/// that has no flip path is an error, so a passed gate can never be dropped
/// silently.
pub fn apply(base: &ParamSet, terms: &[Term]) -> Result<ParamSet> {
    let mut p = base.clone();
    let m = Model::new(base)?;
    for t in terms.iter().filter(|t| t.used == Source::Modelled) {
        let v = t.used_value();
        match t.id {
            "T01" => p.set("PHI_RECYCLE", v)?,
            "T02" => p.set("ETA_MEA", v)?,
            "T05" => p.set("Q_EX4_ABOVE_130C", v / t05_scale(&m, m.s_b))?,
            "T06" => p.set("SRD_CENTRAL", v)?,
            "T10" => p.set("CIRC_CENTRAL_DP_SEC_KPA", v)?,
            "T04" | "T07" | "T08" => {
                if t.anchored == 0.0 {
                    return Err(EngineError::Range {
                        param: t.id.into(),
                        msg: "anchored value is zero, so no factor can carry the model".into(),
                    });
                }
                p.set_term_scale(t.id, v / t.anchored)?
            }
            other => {
                return Err(EngineError::Range {
                    param: other.into(),
                    msg: "this term has no flip path into the anchored balance".into(),
                })
            }
        }
    }
    Ok(p)
}

/// Registry of a preset on the given gates.
pub fn preset_params(base: &ParamSet, preset: Preset, gates: &Gates) -> Result<ParamSet> {
    if preset == Preset::Anchored {
        return Ok(base.clone());
    }
    let list = for_preset(base, preset, gates)?;
    apply(base, &list)
}

/// Gate definitions, for callers that only hold this module.
pub fn gate_defs() -> &'static [gates::GateDef] {
    &gates::GATES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_requires_gate_in_code() {
        let p = ParamSet::embedded().unwrap();
        // No gate passed: every preset equals Anchored.
        let none = Gates {
            states: gates::GATES
                .iter()
                .map(|g| (g.term, GateState::Failed))
                .collect(),
        };
        for preset in [Preset::Headline, Preset::Modelled] {
            let list = for_preset(&p, preset, &none).unwrap();
            assert!(list.iter().all(|t| t.used == Source::Anchored));
            assert_eq!(preset_params(&p, preset, &none).unwrap(), p);
        }
        // Recorded gates: Headline flips T05 alone.
        let rec = Gates::recorded();
        let list = for_preset(&p, Preset::Headline, &rec).unwrap();
        let flipped: Vec<&str> = list
            .iter()
            .filter(|t| t.used == Source::Modelled)
            .map(|t| t.id)
            .collect();
        assert_eq!(flipped, ["T05"]);
        for t in &list {
            if t.used == Source::Modelled {
                assert_eq!(t.gate_state, Some(GateState::Passed), "{}", t.id);
            }
        }
    }

    #[test]
    fn a_flip_reaches_the_balance_and_nothing_else_moves() {
        let p = ParamSet::embedded().unwrap();
        let rec = Gates::recorded();
        let list = for_preset(&p, Preset::Headline, &rec).unwrap();
        let ph = apply(&p, &list).unwrap();
        let m0 = Model::new(&p).unwrap();
        let mh = Model::new(&ph).unwrap();
        let b0 = m0.balance(Scenario::RecF, &m0.opts(m0.s_b)).unwrap();
        let bh = mh.balance(Scenario::RecF, &mh.opts(mh.s_b)).unwrap();
        let t05 = list.iter().find(|t| t.id == "T05").unwrap();
        assert!((bh.stack.ex4 - t05.used_value()).abs() < 1e-9 * bh.stack.ex4);
        assert_eq!(bh.c, b0.c);
        assert_eq!(bh.q_recyc, b0.q_recyc);
        assert_eq!(bh.e_rows.mea_pump, b0.e_rows.mea_pump);
    }

    #[test]
    fn scale_flips_carry_the_case_b_value() {
        let p = ParamSet::embedded().unwrap();
        let m = Model::new(&p).unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let mut list = terms(&m, &b);
        set(&mut list, "T07", Some(1.25 * b.e_rows.mea_pump), "test");
        resolve(&mut list, Preset::Headline, &Gates::all_passed());
        let pm = apply(&p, &list).unwrap();
        let mm = Model::new(&pm).unwrap();
        let bm = mm.balance(Scenario::RecF, &mm.opts(mm.s_b)).unwrap();
        assert!((bm.e_rows.mea_pump / b.e_rows.mea_pump - 1.25).abs() < 1e-12);
        // A term with no flip path that a preset tries to use is refused.
        let mut list = terms(&m, &b);
        set(&mut list, "T09", Some(3.0), "test");
        list[8].used = Source::Modelled;
        assert!(apply(&p, &list).is_err());
    }
}
