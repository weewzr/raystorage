//! The 104 published `pCO2 model` values of Aronu et al. as a reproducible check, with
//! rows in the `Validation_Results.csv` schema. The targets live in
//! `tests/data/aronu_104_targets.csv`. `tests/data/aronu_104_rows.csv` holds a stored
//! run, and `tests/fixture_104.rs` compares it with a fresh run.
//!
//! `nush2_vv` has two routes to these rows: read the stored file through
//! `include_str!("../../mea/tests/data/aronu_104_rows.csv")`, or depend on this crate
//! and call `run_model_points` with `rows_csv`.
//!
//! The page prints one model row more than the 104. Corrigendum Table 3 at 60 C (PDF
//! p16) shows 487.652 kPa and loading 0.784 twice, at total pressures 505.3 and 671.9
//! kPa. The set keeps the 671.9 kPa row, which sits in loading order, and omits the
//! other. The page leaves open which row is the repeat, and the port agrees better with
//! the omitted row, as `repeated_table3_row` prints.

use crate::pstar::{pco2_gas_kpa, pstar_kpa};
use nush2_numerics::{EngineError, Result};

const TARGETS: &str = include_str!("../tests/data/aronu_104_targets.csv");

/// Every point must fall within this relative deviation.
pub const POINT_TOLERANCE: f64 = 0.015;
/// AARD limit in percent for the whole set.
pub const AARD_LIMIT_PCT: f64 = 0.30;
/// Header of `Validation_Results.csv`.
pub const CSV_HEADER: &str = "test_id,level,subject,source,locator,conditions,literature,\
simulated,unit,abs_error,rel_error,tolerance,status,data_use,explanation,commit";

const SOURCE: &str = "Aronu et al. 2011, Chem. Eng. Sci. 66, 6393-6406, corrigendum 78, 246-247";
const SUBJECT: &str = "Published pCO2 model value, extended UNIQUAC port";
const EXPLANATION: &str = "Code verification against the paper's own model column, with SRK and \
Poynting applied at the tabulated total pressure, so this row validates no physics. Thomsen \
activity forms, A(T), b, z and v_inf come from general knowledge of the cited framework and \
from no local source. The reproduction supports them indirectly";

/// One row of the published model columns.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelPoint {
    pub table: String,
    pub w_mea: f64,
    pub t_k: f64,
    pub ptot_kpa: f64,
    pub pco2_model_kpa: f64,
    pub alpha: f64,
    pub locator: String,
}

/// One point after it ran.
#[derive(Debug, Clone, PartialEq)]
pub struct PointResult {
    pub point: ModelPoint,
    /// p* before gas corrections, kPa.
    pub ideal_kpa: f64,
    /// p* after SRK and Poynting, kPa. This is the compared value.
    pub corrected_kpa: f64,
}

impl PointResult {
    /// Signed relative deviation of the corrected value.
    pub fn rel_error(&self) -> f64 {
        self.corrected_kpa / self.point.pco2_model_kpa - 1.0
    }

    /// True when this point meets `POINT_TOLERANCE`.
    pub fn pass(&self) -> bool {
        self.rel_error().abs() <= POINT_TOLERANCE
    }
}

/// Aggregate of a run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub n: usize,
    pub aard_pct: f64,
    pub aard_ideal_pct: f64,
    pub max_abs_dev_pct: f64,
    pub failed_points: usize,
}

/// Split one CSV line, honouring double quotes.
pub fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn float(cell: Option<&String>, line: usize) -> Result<f64> {
    cell.and_then(|c| c.parse::<f64>().ok())
        .ok_or_else(|| EngineError::Parse(format!("bad number on targets line {line}")))
}

/// The 104 targets, parsed from the embedded fixture.
pub fn model_points() -> Result<Vec<ModelPoint>> {
    let mut out = Vec::new();
    for (i, line) in TARGETS.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let c = split_csv(line);
        if c.len() != 7 {
            return Err(EngineError::Parse(format!(
                "targets line {i} has {} cells",
                c.len()
            )));
        }
        out.push(ModelPoint {
            table: c[0].clone(),
            w_mea: float(c.get(1), i)?,
            t_k: float(c.get(2), i)?,
            ptot_kpa: float(c.get(3), i)?,
            pco2_model_kpa: float(c.get(4), i)?,
            alpha: float(c.get(5), i)?,
            locator: c[6].clone(),
        });
    }
    Ok(out)
}

/// Run every target through the model.
pub fn run_model_points() -> Result<Vec<PointResult>> {
    model_points()?
        .into_iter()
        .map(|point| {
            let ideal_kpa = pstar_kpa(point.t_k, point.w_mea, point.alpha)?;
            let corrected_kpa = pco2_gas_kpa(point.t_k, point.w_mea, point.alpha, point.ptot_kpa)?;
            Ok(PointResult {
                point,
                ideal_kpa,
                corrected_kpa,
            })
        })
        .collect()
}

/// Summarise a run. Returns a parse error for an empty run.
pub fn summarise(rows: &[PointResult]) -> Result<Summary> {
    if rows.is_empty() {
        return Err(EngineError::Parse("empty run".into()));
    }
    let n = rows.len();
    let mean = |f: &dyn Fn(&PointResult) -> f64| rows.iter().map(f).sum::<f64>() / n as f64;
    let aard = 100.0 * mean(&|r| r.rel_error().abs());
    let ideal = 100.0 * mean(&|r| (r.ideal_kpa / r.point.pco2_model_kpa - 1.0).abs());
    let worst = rows.iter().map(|r| r.rel_error().abs()).fold(0.0, f64::max);
    Ok(Summary {
        n,
        aard_pct: aard,
        aard_ideal_pct: ideal,
        max_abs_dev_pct: 100.0 * worst,
        failed_points: rows.iter().filter(|r| !r.pass()).count(),
    })
}

fn esc(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Rows in the `Validation_Results.csv` schema, one per point, `commit` left blank as
/// the exporter leaves it. Status comes from the comparison that ran.
pub fn rows_csv(rows: &[PointResult]) -> String {
    let mut out = format!("{CSV_HEADER}\n");
    for (i, r) in rows.iter().enumerate() {
        let p = &r.point;
        let conditions = format!(
            "{:.0} mass% MEA, {:.2} K, loading {:.3}, total pressure {} kPa",
            p.w_mea * 100.0,
            p.t_k,
            p.alpha,
            p.ptot_kpa
        );
        let cells = [
            format!("ARONU_104_{:03}", i + 1),
            "1".to_string(),
            esc(SUBJECT),
            esc(SOURCE),
            esc(&p.locator),
            esc(&conditions),
            format!("{:.6}", p.pco2_model_kpa),
            format!("{:.6}", r.corrected_kpa),
            "kPa".to_string(),
            format!("{:.3e}", (r.corrected_kpa - p.pco2_model_kpa).abs()),
            format!("{:.3e}", r.rel_error().abs()),
            format!("{POINT_TOLERANCE:.3e}"),
            if r.pass() { "pass" } else { "fail" }.to_string(),
            "same_dataset_disclosed".to_string(),
            esc(EXPLANATION),
            String::new(),
        ];
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_parse_to_104_rows() {
        let p = model_points().unwrap();
        assert_eq!(p.len(), 104);
        assert_eq!(p.iter().filter(|r| r.w_mea == 0.15).count(), 35);
        assert_eq!(p.iter().filter(|r| r.w_mea == 0.30).count(), 17);
        assert_eq!(p.iter().filter(|r| r.w_mea == 0.45).count(), 38);
        assert_eq!(p.iter().filter(|r| r.w_mea == 0.60).count(), 14);
    }

    #[test]
    fn quoted_cells_keep_their_commas() {
        assert_eq!(split_csv("\"a, b\",c").len(), 2);
    }

    #[test]
    fn escaping_quotes_cells_that_need_it() {
        assert_eq!(esc("plain"), "plain");
        assert_eq!(esc("a,b"), "\"a,b\"");
    }
}
