//! Fixture reader shared by the integration tests.

use std::path::PathBuf;

/// Rows of a fixture as header-keyed cells, skipping `#` comment lines.
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn read(name: &str) -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join(name);
        let text = std::fs::read_to_string(&path).expect("fixture present");
        let mut lines = text.lines().filter(|l| !l.starts_with('#'));
        let header: Vec<String> = lines
            .next()
            .expect("header present")
            .split(',')
            .map(str::to_string)
            .collect();
        let rows = lines
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split(',').map(str::to_string).collect())
            .collect();
        Self { header, rows }
    }

    pub fn col(&self, name: &str) -> usize {
        self.header
            .iter()
            .position(|h| h == name)
            .unwrap_or_else(|| panic!("column {name}"))
    }

    pub fn f(&self, row: &[String], name: &str) -> f64 {
        row[self.col(name)].parse().expect("number")
    }
}

/// Mean of |a/b - 1| in percent over pairs.
pub fn aard(pairs: &[(f64, f64)]) -> f64 {
    100.0 * pairs.iter().map(|(a, b)| (a / b - 1.0).abs()).sum::<f64>() / pairs.len() as f64
}
