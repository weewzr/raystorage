//! Named screening terms. Each rigorous model fills its `modelled` slot at a
//! later milestone, and a term flips only when its gate passes. At M1 every term
//! is anchored.

use crate::anchored::{Balance, Model};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Source {
    Anchored,
    Modelled,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Term {
    pub id: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub anchored: f64,
    pub modelled: Option<f64>,
    pub used: Source,
    pub gate: &'static str,
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
        used: Source::Anchored,
        gate,
    }
}

/// Fill the modelled slots of the terms gated on M7. `used` stays anchored,
/// because a term flips only at M8 after its gate passes.
pub fn fill_m7(terms: &mut [Term], slots: &crate::flowsheet::TermSlots) {
    for t in terms.iter_mut() {
        t.modelled = match t.id {
            "T04" => Some(slots.t04_mw),
            "T09" => Some(slots.t09_mw),
            "T13" => Some(slots.t13_mmscfd),
            _ => t.modelled,
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
