//! Validation_Results.csv writer, freeze section 8 schema.

use nush2_plant::export::{esc, num};

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
    pub tolerance: f64,
    pub pass: bool,
    pub data_use: &'static str,
    pub explanation: String,
}

const HEADER: &str = "test_id,level,subject,source,locator,conditions,literature,simulated,\
unit,abs_error,rel_error,tolerance,status,data_use,explanation,commit";

fn sci(v: f64) -> String {
    format!("{v:.3e}")
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
            r.literature.map(num).unwrap_or_default(),
            num(r.simulated),
            esc(&r.unit),
            r.abs_error.map(sci).unwrap_or_default(),
            r.rel_error.map(sci).unwrap_or_default(),
            sci(r.tolerance),
            if r.pass { "pass" } else { "fail" }.to_string(),
            r.data_use.to_string(),
            esc(&r.explanation),
            String::new(),
        ];
        out.push_str(&cells.join(","));
        out.push('\n');
    }
    out
}
