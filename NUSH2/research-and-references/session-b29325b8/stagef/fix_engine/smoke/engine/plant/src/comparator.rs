//! One labelled SMR plus CCS comparator: IEAGHG 2017-02 Case 3, MEA on flue gas,
//! at its own 100,000 Nm3/h and this study's availability. Gas price and
//! availability differ from the ledger, so the row is indicative.

use crate::anchored::Model;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Comparator {
    pub a_t_y: f64,
    pub cac_published: f64,
    pub ts_adj: f64,
    pub cac_t60: f64,
    pub cac_t232: f64,
    pub cac_t112: f64,
    pub overall_capture: f64,
}

impl Model {
    /// eq: K.21
    /// eq: L.32
    pub fn comparator(&self) -> Comparator {
        let avoided = self.comparator_base_kg_nm3 - self.comparator_emit_kg_nm3;
        let captured = self.comparator_captured_kg_nm3;
        let cac_sgd = self.comparator_cac_eur * self.f2014;
        let ts_adj = (self.ts.central - self.ts.low) * captured / avoided;
        Comparator {
            a_t_y: avoided * self.h2_b_nm3h * self.hours / 1000.0,
            cac_published: cac_sgd,
            ts_adj,
            cac_t60: cac_sgd + ts_adj,
            cac_t232: cac_sgd,
            cac_t112: cac_sgd + (self.ts.high - self.ts.low) * captured / avoided,
            overall_capture: self.comparator_overall,
        }
    }
}
