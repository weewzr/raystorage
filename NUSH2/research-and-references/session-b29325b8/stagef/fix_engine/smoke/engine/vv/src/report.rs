//! Validation_Results.csv writer.

use nush2_plant::export::{esc, num};

/// Outcome of one row. `Pass` and `Fail` exist only for a comparison that ran
/// against a limit; `Reported` is a value printed with no limit; `NotRun` is a
/// benchmark that could not run, with its reason in the explanation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Reported,
    NotRun,
}

impl Status {
    /// `Pass` when `ok`, else `Fail`.
    pub fn of(ok: bool) -> Self {
        if ok {
            Status::Pass
        } else {
            Status::Fail
        }
    }

    /// The word written to the `status` column.
    pub fn label(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Reported => "reported",
            Status::NotRun => "not_run",
        }
    }
}

/// One row. `status` is computed from the comparison that ran.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
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
    /// Limit the row was judged against. `None` for a reported or not-run row.
    pub tolerance: Option<f64>,
    pub status: Status,
    pub data_use: &'static str,
    pub explanation: String,
}

impl Row {
    /// True for a row that ran and met its limit.
    pub fn passed(&self) -> bool {
        self.status == Status::Pass
    }
}

const HEADER: &str = "test_id,level,subject,source,locator,conditions,literature,simulated,\
unit,abs_error,rel_error,tolerance,status,data_use,explanation,commit";

fn sci(v: f64) -> String {
    format!("{v:.3e}")
}

/// A value cell. A value that did not compute is left blank.
fn cell(v: f64) -> String {
    if v.is_finite() {
        num(v)
    } else {
        String::new()
    }
}

fn opt_sci(v: Option<f64>) -> String {
    v.filter(|x| x.is_finite()).map(sci).unwrap_or_default()
}

/// Sorted CSV. `commit` stays blank: a file cannot name the commit that holds it,
/// and a filled column would break the byte-identical export check.
pub fn csv(rows: &[Row]) -> String {
    let mut sorted: Vec<&Row> = rows.iter().collect();
    sorted.sort_by(|a, b| a.test_id.cmp(&b.test_id));
    let mut out = format!("{HEADER}\n");
    for r in sorted {
        let cells = [
            esc(&r.test_id),
            r.level.to_string(),
            esc(&r.subject),
            esc(&r.source),
            esc(&r.locator),
            esc(&r.conditions),
            r.literature.map(cell).unwrap_or_default(),
            cell(r.simulated),
            esc(&r.unit),
            opt_sci(r.abs_error),
            opt_sci(r.rel_error),
            opt_sci(r.tolerance),
            r.status.label().to_string(),
            r.data_use.to_string(),
            esc(&r.explanation),
            String::new(),
        ];
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out
}
