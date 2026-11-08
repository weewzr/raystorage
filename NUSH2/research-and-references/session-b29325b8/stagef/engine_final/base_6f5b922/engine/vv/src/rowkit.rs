//! Builders for validation rows, shared by the benchmark modules.

use crate::fixture::Fix;
use crate::report::{Row, Status};

pub(crate) const INDEPENDENT: &str = "independent";
pub(crate) const CALIBRATION: &str = "calibration";
pub(crate) const SAME: &str = "same_dataset_disclosed";

/// Identity of a row: everything except the numbers.
#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub id: String,
    pub level: u8,
    pub subject: String,
    pub source: String,
    pub locator: String,
    pub conditions: String,
    pub unit: String,
    pub data_use: &'static str,
}

impl Meta {
    /// Identity whose source and locator come from a fixture row.
    pub fn at(fix: &Fix, id: impl Into<String>, level: u8, subject: &str, unit: &str) -> Self {
        Meta {
            id: id.into(),
            level,
            subject: subject.to_string(),
            source: fix.source.clone(),
            locator: fix.locator.clone(),
            conditions: String::new(),
            unit: unit.to_string(),
            data_use: INDEPENDENT,
        }
    }

    /// Identity with a typed source and locator.
    pub fn new(
        id: impl Into<String>,
        level: u8,
        subject: &str,
        source: &str,
        locator: &str,
    ) -> Self {
        Meta {
            id: id.into(),
            level,
            subject: subject.to_string(),
            source: source.to_string(),
            locator: locator.to_string(),
            conditions: String::new(),
            unit: String::new(),
            data_use: INDEPENDENT,
        }
    }

    pub fn cond(mut self, conditions: impl Into<String>) -> Self {
        self.conditions = conditions.into();
        self
    }

    pub fn locator(mut self, locator: &str) -> Self {
        self.locator = locator.to_string();
        self
    }

    pub fn unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    pub fn use_(mut self, data_use: &'static str) -> Self {
        self.data_use = data_use;
        self
    }

    fn row(
        self,
        lit: Option<f64>,
        sim: f64,
        tol: Option<f64>,
        status: Status,
        note: String,
    ) -> Row {
        let abs = lit.map(|l| (sim - l).abs()).filter(|a| a.is_finite());
        let rel = match (abs, lit) {
            (Some(a), Some(l)) if l != 0.0 => Some(a / l.abs()),
            _ => None,
        };
        Row {
            test_id: self.id,
            level: self.level,
            subject: self.subject,
            source: self.source,
            locator: self.locator,
            conditions: self.conditions,
            literature: lit,
            simulated: sim,
            unit: self.unit,
            abs_error: abs,
            rel_error: rel,
            tolerance: tol,
            status,
            data_use: self.data_use,
            explanation: note,
        }
    }

    /// Judged on |simulated - literature| <= `tol`. A value that did not compute
    /// is written as not run.
    pub fn judged(self, lit: f64, sim: f64, tol: f64, note: impl Into<String>) -> Row {
        let note = note.into();
        if !sim.is_finite() {
            return self.not_run(Some(lit), &note);
        }
        let ok = (sim - lit).abs() <= tol;
        self.row(Some(lit), sim, Some(tol), Status::of(ok), note)
    }

    /// Judged on a pass flag computed by the caller against `tol`, for limits that
    /// are not an absolute gap (AARD, a strict bound, a count).
    pub fn flagged(
        self,
        lit: Option<f64>,
        sim: f64,
        tol: f64,
        ok: bool,
        note: impl Into<String>,
    ) -> Row {
        let note = note.into();
        if !sim.is_finite() {
            return self.not_run(lit, &note);
        }
        self.row(lit, sim, Some(tol), Status::of(ok), note)
    }

    /// Printed with no limit.
    pub fn reported(self, lit: Option<f64>, sim: f64, note: impl Into<String>) -> Row {
        let note = note.into();
        if !sim.is_finite() {
            return self.not_run(lit, &note);
        }
        self.row(lit, sim, None, Status::Reported, note)
    }

    /// Could not run, with the reason.
    pub fn not_run(self, lit: Option<f64>, reason: &str) -> Row {
        let note = if reason.starts_with("not_run:") {
            reason.to_string()
        } else {
            format!("not_run: {reason}")
        };
        self.row(lit, f64::NAN, None, Status::NotRun, note)
    }
}

/// Mean absolute relative deviation in percent of (model, data) pairs.
pub(crate) fn aard_pct(pairs: &[(f64, f64)]) -> f64 {
    if pairs.is_empty() {
        return f64::NAN;
    }
    100.0 * pairs.iter().map(|(m, d)| ((m - d) / d).abs()).sum::<f64>() / pairs.len() as f64
}

/// The pair with the largest relative deviation, as (model, data).
pub(crate) fn worst(pairs: &[(f64, f64)]) -> Option<(f64, f64)> {
    pairs.iter().copied().max_by(|a, b| {
        ((a.0 - a.1) / a.1)
            .abs()
            .total_cmp(&((b.0 - b.1) / b.1).abs())
    })
}

/// Degrees Fahrenheit to kelvin.
pub(crate) fn f_to_k(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8 + 273.15
}

/// Pa per psi, the value `nush2_units::case5` uses.
pub(crate) const PSI_PA: f64 = 6894.757;
