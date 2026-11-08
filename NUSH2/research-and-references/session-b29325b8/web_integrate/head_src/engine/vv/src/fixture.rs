//! Benchmark fixtures of `engine/vv/data/benchmarks/`, in the ten-column schema
//! of `docs/VALIDATION_MATRIX.md` section 2. Every lookup marks the rows it
//! returns, so a test can show that each stored value is read by a benchmark.
//!
//! The fixtures read by `nush2_units::bench` (Khan, Salvinder, Tan, Sonderby,
//! DICE, Kwak, Morgan and the Vinjarapu parametric set) are parsed there and are
//! not listed here.

use nush2_mea::validation::split_csv;
use nush2_numerics::{EngineError, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// Fixtures that the harness itself reads, by file name.
pub const OWN: [(&str, &str); 12] = [
    ("tev953.csv", include_str!("../data/benchmarks/tev953.csv")),
    ("tev961.csv", include_str!("../data/benchmarks/tev961.csv")),
    (
        "xu_froment_1989.csv",
        include_str!("../data/benchmarks/xu_froment_1989.csv"),
    ),
    (
        "wgs_chinchen_1984.csv",
        include_str!("../data/benchmarks/wgs_chinchen_1984.csv"),
    ),
    (
        "wgs_kraussler_2017.csv",
        include_str!("../data/benchmarks/wgs_kraussler_2017.csv"),
    ),
    (
        "aronu_2011.csv",
        include_str!("../data/benchmarks/aronu_2011.csv"),
    ),
    (
        "hartono_2014.csv",
        include_str!("../data/benchmarks/hartono_2014.csv"),
    ),
    (
        "versteeg_1988.csv",
        include_str!("../data/benchmarks/versteeg_1988.csv"),
    ),
    (
        "iaea_tecdoc_1236.csv",
        include_str!("../data/benchmarks/iaea_tecdoc_1236.csv"),
    ),
    (
        "brigman_2014.csv",
        include_str!("../data/benchmarks/brigman_2014.csv"),
    ),
    (
        "isa_2017.csv",
        include_str!("../data/benchmarks/isa_2017.csv"),
    ),
    (
        "abdul_halim_2015.csv",
        include_str!("../data/benchmarks/abdul_halim_2015.csv"),
    ),
];

static READS: Mutex<BTreeMap<&'static str, BTreeSet<usize>>> = Mutex::new(BTreeMap::new());

/// One stored value.
#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    /// Zero-based index among the data rows of its file.
    pub index: usize,
    pub source: String,
    pub locator: String,
    pub data_kind: String,
    pub series: String,
    pub quantity: String,
    pub value: f64,
    pub unit: String,
    pub conditions: String,
}

impl Fix {
    /// A `key=value` entry of `conditions` read as a number.
    pub fn cond(&self, key: &str) -> Result<f64> {
        self.cond_text(key)?
            .parse()
            .map_err(|_| EngineError::Parse(format!("{}: {key} is not a number", self.series)))
    }

    /// A `key=value` entry of `conditions` as text.
    pub fn cond_text(&self, key: &str) -> Result<&str> {
        self.conditions
            .split(';')
            .find_map(|kv| kv.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
            .ok_or_else(|| EngineError::Parse(format!("{}: no {key} in conditions", self.series)))
    }

    /// True when `conditions` carries `key=value`.
    pub fn has(&self, key: &str, value: &str) -> bool {
        self.cond_text(key).is_ok_and(|v| v == value)
    }
}

/// One parsed file.
pub struct Fixture {
    name: &'static str,
    rows: Vec<Fix>,
}

impl Fixture {
    /// Parse an embedded fixture by file name.
    pub fn open(name: &'static str) -> Result<Self> {
        let text = OWN
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, t)| *t)
            .ok_or_else(|| EngineError::Parse(format!("no fixture named {name}")))?;
        Self::parse(name, text)
    }

    fn parse(name: &'static str, text: &str) -> Result<Self> {
        let mut rows = Vec::new();
        for (i, line) in text.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let c = split_csv(line);
            if c.len() != 10 {
                return Err(EngineError::Parse(format!(
                    "{name} line {} has {} cells",
                    i + 1,
                    c.len()
                )));
            }
            let value = c[6]
                .parse::<f64>()
                .map_err(|_| EngineError::Parse(format!("{name} line {}: {}", i + 1, c[6])))?;
            rows.push(Fix {
                index: rows.len(),
                source: c[0].clone(),
                locator: c[1].clone(),
                data_kind: c[3].clone(),
                series: c[4].clone(),
                quantity: c[5].clone(),
                value,
                unit: c[7].clone(),
                conditions: c[9].clone(),
            });
        }
        Ok(Self { name, rows })
    }

    fn mark(&self, idx: impl IntoIterator<Item = usize>) {
        if let Ok(mut log) = READS.lock() {
            log.entry(self.name).or_default().extend(idx);
        }
    }

    /// The row with this series and quantity.
    pub fn get(&self, series: &str, quantity: &str) -> Result<&Fix> {
        let r = self
            .rows
            .iter()
            .find(|r| r.series == series && r.quantity == quantity)
            .ok_or_else(|| {
                EngineError::Parse(format!("{} has no {series} {quantity}", self.name))
            })?;
        self.mark([r.index]);
        Ok(r)
    }

    /// The value with this series and quantity.
    pub fn v(&self, series: &str, quantity: &str) -> Result<f64> {
        Ok(self.get(series, quantity)?.value)
    }

    /// Every row that `keep` accepts, in file order.
    pub fn select(&self, keep: impl Fn(&Fix) -> bool) -> Vec<&Fix> {
        let out: Vec<&Fix> = self.rows.iter().filter(|r| keep(r)).collect();
        self.mark(out.iter().map(|r| r.index));
        out
    }

    /// Number of data rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True for a file with no data rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Rows of `name` that no lookup has returned in this process, by data-row index.
pub fn unread(name: &'static str) -> Result<Vec<usize>> {
    let f = Fixture::open(name)?;
    let read = READS
        .lock()
        .map_err(|_| EngineError::Parse("fixture read log is poisoned".into()))?
        .get(name)
        .cloned()
        .unwrap_or_default();
    Ok((0..f.len()).filter(|i| !read.contains(i)).collect())
}
