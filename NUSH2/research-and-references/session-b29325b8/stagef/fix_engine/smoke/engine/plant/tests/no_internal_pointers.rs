//! Source and explanation text in the input database and the results tables
//! cites primary sources, decision ids or repository paths. A pointer into a
//! working note that never ships fails here.

use std::path::PathBuf;

/// Names of working notes and keys that never ship, matched without case.
const PATTERNS: [&str; 35] = [
    "plan section",
    "screening script",
    "screening ledger",
    "smr_wgs",
    "critic.md",
    "audit.md",
    "inventory.md",
    "decision s8",
    "item m14",
    "inl.md",
    "econ_lit",
    "htgr_ihx.md",
    "htgr_calc",
    "amine_spec",
    "mea_columns",
    "mea_thermo",
    "mea_calc",
    "final_v1_7",
    "inl_calcs",
    "old repo",
    "old_repo",
    "old-repo",
    "audit_recompute",
    "freeze section",
    "freeze s",
    "screening.py",
    "user decision",
    "user_answers",
    "rest row",
    "deck slide",
    "q8_check",
    "qproc_kcheck",
    "build order section",
    "section 7 drift",
    "private outline",
];

/// Single-letter working keys of the notes: a word such as s6, s9h or Q3.
fn working_key(word: &str) -> bool {
    let mut c = word.chars();
    let Some(first) = c.next() else {
        return false;
    };
    let rest: String = c.collect();
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let tail = &rest[digits.len()..];
    matches!(first, 's' | 'Q')
        && !digits.is_empty()
        && (tail.is_empty() || (tail.len() == 1 && tail.chars().all(|t| t.is_ascii_lowercase())))
}

/// Markdown files a source string may name: the public docs of the repository.
const PUBLIC_MD: [&str; 2] = [
    "docs/final_design_decisions.md",
    "docs/validation_matrix.md",
];

/// A Markdown or Python file named in a line that is not a public repository
/// path: a working note or script that never ships.
fn private_files(low: &str) -> Vec<String> {
    let mut out = Vec::new();
    for word in low.split(|c: char| !(c.is_ascii_alphanumeric() || "_./-".contains(c))) {
        let word = word.trim_end_matches('.');
        if word.ends_with(".md") && !PUBLIC_MD.contains(&word) {
            out.push(word.to_string());
        }
        if word.ends_with(".py") && !word.starts_with("scripts/") {
            out.push(word.to_string());
        }
    }
    out
}

fn hits(name: &str, text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let low = line.to_lowercase();
        for p in PATTERNS {
            if low.contains(p) {
                out.push(format!("{name}:{}: {p}", i + 1));
            }
        }
        for f in private_files(&low) {
            out.push(format!("{name}:{}: file {f}", i + 1));
        }
        for word in line.split(|c: char| !c.is_ascii_alphanumeric()) {
            if working_key(word) {
                out.push(format!("{name}:{}: working key {word}", i + 1));
            }
        }
    }
    out
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_detector_sees_each_pattern() {
    assert!(!hits("x", "econ_lit.md s9h").is_empty());
    assert!(!hits("x", "see the freeze section 4.2").is_empty());
    assert!(!hits("x", "User decision Q3").is_empty());
    assert!(!hits("x", "locator s6").is_empty());
    assert!(!hits("x", "card J.10; plan section 12 carbon_matches_anchor").is_empty());
    assert!(!hits("x", "Verification against the screening script").is_empty());
    assert!(!hits("x", "see htgr_calc2.py and smr_wgs notes").is_empty());
    assert!(!hits("x", "(notes.md s3)").is_empty());
    assert!(!hits("x", "Section 7 drift, Headline preset").is_empty());
    assert!(hits("x", "Drift check, Headline preset").is_empty());
    assert!(hits("x", "TEV-961 Table 2 doc p16 (PDF p17), 5 s spacing").is_empty());
    assert!(hits("x", "engine/vv/data/freeze_values.json").is_empty());
    assert!(hits("x", "decision S11, docs/VALIDATION_MATRIX.md").is_empty());
    assert!(hits("x", "built by scripts/build_numbers.py").is_empty());
    assert!(hits("x", "Khan prints no critical surface tension").is_empty());
}

#[test]
fn parameters_and_results_cite_no_working_note() {
    let mut found = Vec::new();
    // The comment header of the database names no note; its entries are what
    // reach the results, so every line is checked.
    let toml = root().join("engine/data/parameters.toml");
    let text = std::fs::read_to_string(&toml).expect("parameters.toml");
    found.extend(hits("parameters.toml", &text));
    let dir = root().join("results");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("results")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "csv"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 7, "{files:?}");
    for f in files {
        let text = std::fs::read_to_string(&f).expect("results file");
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        found.extend(hits(&name, &text));
    }
    assert!(
        found.is_empty(),
        "{} pointers:\n{}",
        found.len(),
        found.join("\n")
    );
}
