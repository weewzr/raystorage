//! Heat recovery for the MEA reboiler: the approved order of sources, the
//! extraction branch that covers what the stack leaves, the grade check of each
//! source against the reboiler, and the two heat terms the Level 3 flowsheet
//! gives on real flows (syngas heat above the cut and the shift heat release).
//!
//! Approved order (decision D27): freed Selexol steam, export steam,
//! syngas above 130 C, then reactor steam. ZLD steam, off by default, sits after
//! the syngas and before the reactor, as the anchored port has it.
//!
//! Pinch. The reboiler is an isothermal sink at `T_reb`. A source can serve it
//! only from above `T_reb` (card I.17), so the 93 C hot water loop of the INL
//! steam system cannot (card J.30). The coldest heat the stack hands over is the
//! syngas at its 130 C cut, so the smallest approach of the stack, its pinch,
//! sits there. No minimum approach temperature is published in the library, so
//! the check prints each approach and judges only that it is positive.

use crate::anchored::{Route, Stack};
use nush2_numerics::Result;
use nush2_props::species::{h_j_mol, Sp as PSp};
use nush2_units::cooler;
use nush2_units::stream::{Sp, Stream};
use serde::Serialize;

/// Sources of reboiler heat in the approved order, the reactor last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SourceId {
    Selexol,
    Export,
    Syngas,
    Zld,
    Reactor,
}

impl SourceId {
    pub fn id(self) -> &'static str {
        match self {
            SourceId::Selexol => "SELEXOL",
            SourceId::Export => "EXPORT",
            SourceId::Syngas => "SYNGAS",
            SourceId::Zld => "ZLD",
            SourceId::Reactor => "REACTOR",
        }
    }
}

/// The approved order, decision D27, with ZLD before the reactor.
pub const APPROVED_ORDER: [SourceId; 5] = [
    SourceId::Selexol,
    SourceId::Export,
    SourceId::Syngas,
    SourceId::Zld,
    SourceId::Reactor,
];

/// How the reboiler duty is met.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Allocation {
    /// MW taken from Selexol, export, syngas and ZLD, in that order.
    pub used: [f64; 4],
    /// Sum of the four sources offered.
    pub avail: f64,
    /// Reactor steam, the duty the stack leaves.
    pub gap: f64,
}

impl Allocation {
    /// MW from one source; the reactor's share is the gap.
    pub fn from(&self, s: SourceId) -> f64 {
        match s {
            SourceId::Selexol => self.used[0],
            SourceId::Export => self.used[1],
            SourceId::Syngas => self.used[2],
            SourceId::Zld => self.used[3],
            SourceId::Reactor => self.gap,
        }
    }
}

/// Meet `q_reb` from the stack in the approved order. Each source gives what it
/// has until the duty is met; the reactor covers the rest,
/// `Q_gap = max(0, Q_reb - Q_stack)`.
///
/// eq: J.17
pub fn allocate(q_reb: f64, stack: &Stack) -> Allocation {
    let avail = stack.selexol + stack.export + stack.ex4 + stack.zld;
    let mut left = q_reb;
    let mut used = [0.0; 4];
    for (u, have) in used
        .iter_mut()
        .zip([stack.selexol, stack.export, stack.ex4, stack.zld])
    {
        *u = have.min(left.max(0.0));
        left -= *u;
    }
    Allocation {
        used,
        avail,
        gap: gap(q_reb, avail),
    }
}

/// Unmet reboiler duty after the stack.
///
/// eq: J.18
pub fn gap(q_reb: f64, avail: f64) -> f64 {
    (q_reb - avail).max(0.0)
}

/// Heat the reactor gives the power and steam branch for a reboiler gap and a
/// net electricity demand `e_nuc`. On extraction, `w = min(E, r_w Q_gap)` and
/// `Q_br = Q_gap + w + (E - w) / eta`; direct, `Q_br = Q_gap + E / eta`.
///
/// eq: B.19
/// eq: B.18
pub fn extraction(gap_mw: f64, e_nuc: f64, r_work: f64, eta_p: f64, route: Route) -> f64 {
    match route {
        Route::Extraction => {
            let wx = e_nuc.min(r_work * gap_mw);
            gap_mw + wx + (e_nuc - wx) / eta_p
        }
        Route::Direct => gap_mw + e_nuc / eta_p,
    }
}

/// Grade of one source against the reboiler.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Grade {
    pub source: &'static str,
    /// Coldest temperature at which the source hands over heat, C.
    pub t_c: f64,
    /// `t_c - T_reb`, K.
    pub approach_k: f64,
    pub usable: bool,
}

/// Grade check of the whole stack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Pinch {
    pub t_reb_c: f64,
    pub grades: Vec<Grade>,
    /// Source with the smallest approach among the usable ones.
    pub pinch_source: &'static str,
    pub pinch_approach_k: f64,
}

/// Check each source temperature against the reboiler,
/// `T_source >= T_reb + dT_min`, and find the pinch of the stack. Sources at or
/// below `T_reb` are marked unusable (the hot water loop).
///
/// eq: I.17
pub fn check_grade(t_reb_c: f64, sources: &[(&'static str, f64)]) -> Pinch {
    let grades: Vec<Grade> = sources
        .iter()
        .map(|&(source, t_c)| Grade {
            source,
            t_c,
            approach_k: t_c - t_reb_c,
            usable: t_c > t_reb_c,
        })
        .collect();
    // eq: J.30
    let pinch = grades
        .iter()
        .filter(|g| g.usable)
        .min_by(|a, b| a.approach_k.total_cmp(&b.approach_k));
    Pinch {
        t_reb_c,
        pinch_source: pinch.map_or("", |g| g.source),
        pinch_approach_k: pinch.map_or(f64::NAN, |g| g.approach_k),
        grades,
    }
}

/// Syngas heat released from `ex3_outlet` down to the cut, with condensation,
/// on the flowsheet's own stream, W.
///
/// eq: F.11
pub fn syngas_above_w(ex3_outlet: &Stream, t_cut_k: f64) -> Result<f64> {
    cooler::heat_above(ex3_outlet, t_cut_k)
}

/// Heat released by the shift across one bed, `Q = xi (-Delta_r h_2)`, with the
/// extent from the CO consumed and the reaction enthalpy at the bed inlet
/// temperature, W.
///
/// eq: E.6
/// eq: A.10
pub fn wgs_release_w(inlet: &Stream, outlet: &Stream) -> Result<f64> {
    let xi = inlet.n[Sp::Co as usize] - outlet.n[Sp::Co as usize];
    let t = inlet.t_k;
    let dh =
        h_j_mol(PSp::Co2, t)? + h_j_mol(PSp::H2, t)? - h_j_mol(PSp::Co, t)? - h_j_mol(PSp::H2o, t)?;
    Ok(-xi * dh)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack() -> Stack {
        Stack {
            selexol: 10.0,
            export: 15.0,
            ex4: 7.0,
            zld: 0.0,
        }
    }

    #[test]
    fn eq_j_17_reboiler_order_is_approved() {
        // Partial demand inside Selexol steam: nothing else is touched.
        let a = allocate(6.0, &stack());
        assert_eq!(a.used, [6.0, 0.0, 0.0, 0.0]);
        assert_eq!(a.gap, 0.0);
        // Past Selexol, export goes second, before syngas.
        let a = allocate(20.0, &stack());
        assert_eq!(a.used, [10.0, 10.0, 0.0, 0.0]);
        // Past export, syngas third; the reactor only after the whole stack.
        let a = allocate(30.0, &stack());
        assert_eq!(a.used, [10.0, 15.0, 5.0, 0.0]);
        assert_eq!(a.gap, 0.0);
        let a = allocate(40.0, &stack());
        assert_eq!(a.used, [10.0, 15.0, 7.0, 0.0]);
        assert!((a.gap - 8.0).abs() < 1e-12);
        assert_eq!(a.from(SourceId::Reactor), a.gap);
        let order: Vec<&str> = APPROVED_ORDER.iter().map(|s| s.id()).collect();
        assert_eq!(order, ["SELEXOL", "EXPORT", "SYNGAS", "ZLD", "REACTOR"]);
    }

    #[test]
    fn eq_j_18_gap_is_never_negative() {
        assert_eq!(gap(5.0, 10.0), 0.0);
        assert_eq!(gap(15.0, 10.0), 5.0);
    }

    #[test]
    fn eq_b_19_extraction_saves_heat_against_direct() {
        let ex = extraction(13.8, 20.0, 0.169, 0.397, Route::Extraction);
        let di = extraction(13.8, 20.0, 0.169, 0.397, Route::Direct);
        assert!(ex < di);
        let w = 0.169 * 13.8;
        assert!((ex - (13.8 + w + (20.0 - w) / 0.397)).abs() < 1e-12);
    }

    #[test]
    fn eq_i_17_hot_water_is_too_cold_and_the_cut_is_the_pinch() {
        let p = check_grade(
            121.0,
            &[
                ("SELEXOL", 152.8),
                ("EXPORT", 253.9),
                ("SYNGAS", 130.0),
                ("HOT_WATER", 93.3),
            ],
        );
        assert_eq!(p.pinch_source, "SYNGAS");
        assert!((p.pinch_approach_k - 9.0).abs() < 1e-12);
        assert!(!p.grades[3].usable);
    }

    #[test]
    fn eq_e_06_release_is_positive_for_a_forward_shift() {
        let mut n = [0.0; nush2_units::stream::NSP];
        n[Sp::Co as usize] = 10.0;
        n[Sp::H2o as usize] = 30.0;
        n[Sp::H2 as usize] = 50.0;
        n[Sp::Co2 as usize] = 5.0;
        let a = Stream::gas("IN", 608.15, 30.0e5, n);
        let mut m = n;
        m[Sp::Co as usize] -= 7.0;
        m[Sp::H2o as usize] -= 7.0;
        m[Sp::H2 as usize] += 7.0;
        m[Sp::Co2 as usize] += 7.0;
        let b = Stream::gas("OUT", 680.0, 30.0e5, m);
        let q = wgs_release_w(&a, &b).unwrap();
        // About 39 kJ/mol at the HTS inlet.
        assert!(q / 7.0 > 3.5e4 && q / 7.0 < 4.3e4, "{q}");
    }
}
