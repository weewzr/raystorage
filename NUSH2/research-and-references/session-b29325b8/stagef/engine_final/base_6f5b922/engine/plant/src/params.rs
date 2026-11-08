//! Input registry. `engine/data/parameters.toml` is the one source of project
//! inputs. It is embedded at build time so native and wasm builds share one file.
//!
//! The parser accepts the small TOML subset the file uses: `[[param]]` tables,
//! `key = "string"`, `key = number`, comments and blank lines. Anything else is
//! a parse error, so a malformed file can never load silently.

use nush2_numerics::{EngineError, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// The embedded input database.
pub const PARAMETERS_TOML: &str = include_str!("../../data/parameters.toml");

/// One registry entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParamEntry {
    pub id: String,
    pub value: f64,
    pub unit: String,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub group: String,
    pub class: String,
    pub rigour: String,
    pub source: String,
    pub locator: String,
    pub source_status: String,
    pub decision: String,
}

/// Registry keyed by id. Overrides stay in memory and are never written back.
///
/// `term_scales` holds the factors a preset puts on a screening term that has no
/// single registry input (T04, T07, T08). They are not registry entries, start
/// empty, and read as 1.0 when absent, so the embedded set is the Anchored
/// preset.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamSet {
    entries: BTreeMap<String, ParamEntry>,
    term_scales: BTreeMap<String, f64>,
}

const CLASSES: [&str; 4] = ["published", "assumed", "calculated", "user_decision"];
const RIGOURS: [&str; 5] = [
    "rigorous",
    "reduced_order",
    "empirical",
    "fixed_assumption",
    "external_boundary",
];
const STATUSES: [&str; 4] = ["primary", "via_reader", "via_old_repo", "not_verified"];

impl ParamSet {
    /// Parse the embedded file.
    pub fn embedded() -> Result<Self> {
        Self::parse(PARAMETERS_TOML)
    }

    /// Parse the TOML subset described in the module header.
    pub fn parse(text: &str) -> Result<Self> {
        let mut entries = BTreeMap::new();
        let mut cur: Option<BTreeMap<String, Field>> = None;
        for (no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line == "[[param]]" {
                if let Some(fields) = cur.take() {
                    let e = build_entry(fields)?;
                    insert(&mut entries, e)?;
                }
                cur = Some(BTreeMap::new());
                continue;
            }
            let (key, field) = parse_line(line)
                .map_err(|m| EngineError::Parse(format!("parameters.toml line {}: {m}", no + 1)))?;
            match cur.as_mut() {
                Some(fields) => {
                    fields.insert(key, field);
                }
                None => {
                    return Err(EngineError::Parse(format!(
                        "parameters.toml line {}: key outside [[param]]",
                        no + 1
                    )))
                }
            }
        }
        if let Some(fields) = cur.take() {
            let e = build_entry(fields)?;
            insert(&mut entries, e)?;
        }
        Ok(Self {
            entries,
            term_scales: BTreeMap::new(),
        })
    }

    /// Read a value by id.
    pub fn get(&self, id: &str) -> Result<f64> {
        self.entries
            .get(id)
            .map(|e| e.value)
            .ok_or_else(|| EngineError::UnknownParam(id.to_string()))
    }

    /// Change a value in memory. Unknown ids and non-finite values are refused.
    pub fn set(&mut self, id: &str, value: f64) -> Result<()> {
        if !value.is_finite() {
            return Err(EngineError::Range {
                param: id.to_string(),
                msg: "value must be finite".into(),
            });
        }
        match self.entries.get_mut(id) {
            Some(e) => {
                e.value = value;
                Ok(())
            }
            None => Err(EngineError::UnknownParam(id.to_string())),
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &ParamEntry> {
        self.entries.values()
    }

    /// Put a preset factor on a screening term. Non-finite or non-positive
    /// factors are refused.
    pub fn set_term_scale(&mut self, term: &str, factor: f64) -> Result<()> {
        if !(factor > 0.0 && factor.is_finite()) {
            return Err(EngineError::Range {
                param: term.to_string(),
                msg: format!("term factor {factor} must be positive and finite"),
            });
        }
        self.term_scales.insert(term.to_string(), factor);
        Ok(())
    }

    /// Preset factor on a screening term, 1.0 when none is set.
    pub fn term_scale(&self, term: &str) -> f64 {
        self.term_scales.get(term).copied().unwrap_or(1.0)
    }

    /// Terms that carry a factor other than the default.
    pub fn term_scales(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.term_scales.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn insert(entries: &mut BTreeMap<String, ParamEntry>, e: ParamEntry) -> Result<()> {
    let id = e.id.clone();
    if entries.insert(id.clone(), e).is_some() {
        return Err(EngineError::Parse(format!("duplicate parameter id {id}")));
    }
    Ok(())
}

enum Field {
    Text(String),
    Number(f64),
}

fn parse_line(line: &str) -> std::result::Result<(String, Field), String> {
    let (key, rest) = line.split_once('=').ok_or("expected key = value")?;
    let key = key.trim().to_string();
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(format!("bad key {key:?}"));
    }
    let rest = rest.trim();
    if let Some(body) = rest.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = body.chars();
        loop {
            match chars.next() {
                None => return Err("unterminated string".into()),
                Some('"') => break,
                Some('\\') => match chars.next() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    _ => return Err("bad escape".into()),
                },
                Some(c) => out.push(c),
            }
        }
        let tail = chars.as_str().trim();
        if !(tail.is_empty() || tail.starts_with('#')) {
            return Err("text after string".into());
        }
        Ok((key, Field::Text(out)))
    } else {
        let num = rest.split('#').next().unwrap_or("").trim();
        let v: f64 = num.parse().map_err(|_| format!("bad number {num:?}"))?;
        if !v.is_finite() {
            return Err(format!("non-finite number {num:?}"));
        }
        Ok((key, Field::Number(v)))
    }
}

fn build_entry(mut f: BTreeMap<String, Field>) -> Result<ParamEntry> {
    let mut text = |k: &str, required: bool| -> Result<String> {
        match f.remove(k) {
            Some(Field::Text(s)) => Ok(s),
            Some(Field::Number(_)) => Err(EngineError::Parse(format!("{k} must be text"))),
            None if required => Err(EngineError::Parse(format!("missing field {k}"))),
            None => Ok(String::new()),
        }
    };
    let id = text("id", true)?;
    let unit = text("unit", true)?;
    let group = text("group", true)?;
    let class = text("class", true)?;
    let rigour = text("rigour", true)?;
    let source = text("source", true)?;
    let locator = text("locator", true)?;
    let source_status = text("source_status", true)?;
    let decision = text("decision", false)?;
    let mut number = |k: &str| -> Result<Option<f64>> {
        match f.remove(k) {
            Some(Field::Number(v)) => Ok(Some(v)),
            Some(Field::Text(_)) => Err(EngineError::Parse(format!("{k} must be a number"))),
            None => Ok(None),
        }
    };
    let value = number("value")?.ok_or_else(|| EngineError::Parse(format!("{id} has no value")))?;
    let low = number("low")?;
    let high = number("high")?;
    if let Some(k) = f.keys().next() {
        return Err(EngineError::Parse(format!("{id}: unknown field {k}")));
    }
    let bad = |what: &str, v: &str| EngineError::Parse(format!("{id}: bad {what} {v:?}"));
    if !CLASSES.contains(&class.as_str()) {
        return Err(bad("class", &class));
    }
    if !RIGOURS.contains(&rigour.as_str()) {
        return Err(bad("rigour", &rigour));
    }
    if !STATUSES.contains(&source_status.as_str()) {
        return Err(bad("source_status", &source_status));
    }
    if source.is_empty() || locator.is_empty() {
        return Err(bad("source", &source));
    }
    Ok(ParamEntry {
        id,
        value,
        unit,
        low,
        high,
        group,
        class,
        rigour,
        source,
        locator,
        source_status,
        decision,
    })
}

/// Reads values and logs which ids were read, so a test can prove that every
/// registry entry feeds the model.
pub struct Reader<'a> {
    set: &'a ParamSet,
    read: BTreeSet<String>,
}

impl<'a> Reader<'a> {
    pub fn new(set: &'a ParamSet) -> Self {
        Self {
            set,
            read: BTreeSet::new(),
        }
    }

    pub fn get(&mut self, id: &str) -> Result<f64> {
        let v = self.set.get(id)?;
        self.read.insert(id.to_string());
        Ok(v)
    }

    /// Preset factor on a screening term; not a registry read.
    pub fn term_scale(&self, term: &str) -> f64 {
        self.set.term_scale(term)
    }

    pub fn into_read(self) -> BTreeSet<String> {
        self.read
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_file_parses() {
        let p = ParamSet::embedded().unwrap();
        assert!(p.len() > 131);
        assert_eq!(p.get("H2_B").unwrap(), 100000.0);
    }

    #[test]
    fn unknown_id_is_an_error() {
        let p = ParamSet::embedded().unwrap();
        assert!(matches!(p.get("NOPE"), Err(EngineError::UnknownParam(_))));
    }

    #[test]
    fn bad_rows_are_refused() {
        assert!(ParamSet::parse("id = \"X\"").is_err());
        assert!(ParamSet::parse("[[param]]\nid = \"X\"\nvalue = oops").is_err());
        assert!(ParamSet::parse("[[param]]\nid = \"X\"\nvalue = 1.0").is_err());
    }

    #[test]
    fn set_refuses_non_finite() {
        let mut p = ParamSet::embedded().unwrap();
        assert!(p.set("AVAILABILITY", f64::NAN).is_err());
    }
}
