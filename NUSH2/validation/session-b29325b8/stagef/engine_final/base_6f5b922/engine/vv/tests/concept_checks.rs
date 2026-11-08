//! Source, record and traceability checks drawn from the OUTRAM PARK concept review
//! (concepts after OUTRAM PARK, AGPL-3.0, reference only; no OUTRAM code). They
//! read the committed tree: the engine sources, `results/Validation_Results.csv`,
//! `math/eq_register.csv` and the published docs and site files.

use nush2_mea::validation::split_csv;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.rs` file under `dir`, recursively, in a stable order.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// Library source of the engine crates: `src` trees only, cut at an in-file test
/// module (`#[cfg(test)]` followed by `mod tests {`), with the wasm test file left out.
fn library_sources(crates: &[&str]) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for c in crates {
        for p in rust_files(&repo().join("engine").join(c).join("src")) {
            if p.file_name().is_some_and(|n| n == "tests.rs") {
                continue;
            }
            let text = fs::read_to_string(&p).unwrap();
            let lines: Vec<String> = text.lines().map(str::to_string).collect();
            let mut keep = lines.len();
            for i in 0..lines.len().saturating_sub(1) {
                if lines[i].trim() == "#[cfg(test)]"
                    && lines[i + 1].trim().starts_with("mod tests {")
                {
                    keep = i;
                    break;
                }
            }
            let rel = p
                .strip_prefix(repo())
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, lines[..keep].to_vec()));
        }
    }
    out
}

const ALL_CRATES: [&str; 8] = [
    "numerics", "props", "mea", "units", "plant", "vv", "cli", "wasm",
];

/// Doc comment block directly above line `i`.
fn doc_above(lines: &[String], i: usize) -> String {
    let mut j = i;
    let mut doc = Vec::new();
    while j > 0 {
        j -= 1;
        let t = lines[j].trim();
        if let Some(d) = t.strip_prefix("///") {
            doc.push(d.trim().to_string());
        } else if t.starts_with("#[") {
            continue;
        } else {
            break;
        }
    }
    doc.reverse();
    doc.join(" ")
}

/// Item 4: one definition per physical constant and per numerical routine. A
/// definition of a listed name with a literal value, or a second routine of a listed
/// name, must carry a doc line that names the copy it sits beside (the words
/// "beside `" and a path). Aliases that read another constant are not definitions.
#[test]
fn second_definitions_name_the_copy_they_sit_beside() {
    let constants = [
        "R_GAS",
        "R",
        "R_KJ_MOL_K",
        "MW_CO2",
        "MW_H2O",
        "MW_WATER",
        "MW_MEA",
        "BAR_PA",
        "ATM_PA",
    ];
    let routines = ["brent", "stencil", "largest_real_root"];
    let canonical = "engine/props/src/units.rs";
    let mut found: BTreeMap<&str, Vec<(String, usize, String)>> = BTreeMap::new();
    for (file, lines) in library_sources(&ALL_CRATES) {
        for (i, l) in lines.iter().enumerate() {
            let t = l.trim_start();
            let t = t
                .strip_prefix("pub(crate) ")
                .or_else(|| t.strip_prefix("pub "))
                .unwrap_or(t);
            for name in constants {
                let head = format!("const {name}: f64 = ");
                if let Some(v) = t.strip_prefix(&head) {
                    if v.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
                        found.entry(name).or_default().push((
                            file.clone(),
                            i,
                            doc_above(&lines, i),
                        ));
                    }
                }
            }
            for name in routines {
                if t.starts_with(&format!("fn {name}(")) || t.starts_with(&format!("fn {name}<")) {
                    found
                        .entry(name)
                        .or_default()
                        .push((file.clone(), i, doc_above(&lines, i)));
                }
            }
        }
    }
    // The gas constants and the water molar masses each form one family.
    fn family(n: &'static str) -> &'static str {
        match n {
            "R_GAS" | "R" | "R_KJ_MOL_K" => "gas constant",
            "MW_H2O" | "MW_WATER" => "water molar mass",
            other => other,
        }
    }
    let mut fam: BTreeMap<&str, Vec<(String, usize, String)>> = BTreeMap::new();
    for (n, v) in found {
        fam.entry(family(n)).or_default().extend(v);
    }
    let mut bad = Vec::new();
    for (name, defs) in &fam {
        println!("{name}: {} definition(s)", defs.len());
        for (file, i, doc) in defs {
            println!("  {file}:{}", i + 1);
            if defs.len() > 1 && file != canonical && !doc.contains("beside `") {
                bad.push(format!("{name} at {file}:{}", i + 1));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "second definitions without a reason: {bad:?}"
    );
}

/// Item 6: every unit operation module names its model class and the reason in its
/// header, and a module that records a `ModelClass` names that class.
#[test]
fn unit_modules_name_their_class() {
    let units = [
        "absorber",
        "stripper",
        "regen",
        "cooler",
        "flash",
        "smr_eq",
        "smr_tube",
        "wgs",
        "circulator",
        "ihx",
    ];
    let mut bad = Vec::new();
    for m in units {
        let p = repo().join(format!("engine/units/src/{m}.rs"));
        let text = fs::read_to_string(&p).unwrap();
        let header: String = text
            .lines()
            .take_while(|l| l.starts_with("//!"))
            .collect::<Vec<_>>()
            .join(" ");
        let named = ["Rigorous", "ReducedOrder", "Empirical"].iter().any(|c| {
            header.contains(&format!("class {c}")) || header.contains(&format!("Class {c}"))
        });
        if !named {
            bad.push(format!("{m}: no class line"));
        }
        for c in ["Rigorous", "ReducedOrder", "Empirical"] {
            if text.contains(&format!("ModelClass::{c}")) && !header.contains(c) {
                bad.push(format!("{m}: records {c} without naming it"));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

/// Item 18 (c): no gas-phase rate in `engine/units/src` mixes a concentration
/// closure with a pressure-basis equilibrium constant.
#[test]
fn no_unit_mixes_concentration_with_pressure_basis_k() {
    let bad: Vec<String> = library_sources(&["units"])
        .into_iter()
        .filter(|(_, lines)| {
            let t = lines.join("\n");
            t.contains("mixture::conc") && t.contains("equilibrium::")
        })
        .map(|(f, _)| f)
        .collect();
    assert!(bad.is_empty(), "{bad:?}");
}

/// Item 33 (b): crates built for wasm32 call no timer, thread or file system API
/// outside their test modules, since compiling for wasm32 does not show that a call
/// runs there.
#[test]
fn wasm_crates_use_no_host_only_api() {
    let banned = [
        "std::time::Instant",
        "Instant::now",
        "SystemTime",
        "thread::spawn",
        "std::fs",
    ];
    let mut bad = Vec::new();
    for (file, lines) in library_sources(&["numerics", "props", "mea", "units", "plant", "wasm"]) {
        for (i, l) in lines.iter().enumerate() {
            if l.trim_start().starts_with("//") {
                continue;
            }
            for b in banned {
                if l.contains(b) {
                    bad.push(format!("{file}:{} {b}", i + 1));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

/// Rows of the committed validation record, keyed by column name.
fn validation_rows() -> Vec<BTreeMap<String, String>> {
    let text = fs::read_to_string(repo().join("results/Validation_Results.csv")).unwrap();
    let mut lines = text.lines();
    let head = split_csv(lines.next().unwrap());
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| head.iter().cloned().zip(split_csv(l)).collect())
        .collect()
}

/// Item 12: every failed row states the measured value and the limit it missed, so
/// a reader can judge it without re-running it.
#[test]
fn failed_rows_state_value_and_limit() {
    let rows = validation_rows();
    assert!(!rows.is_empty());
    let mut bad = Vec::new();
    for r in rows.iter().filter(|r| r["status"] == "fail") {
        let has_limit = !r["tolerance"].is_empty()
            || r["explanation"].contains("limit")
            || r["explanation"].contains("Limit")
            || r["explanation"].contains("Failed rows");
        if r["explanation"].trim().is_empty() || r["simulated"].trim().is_empty() || !has_limit {
            bad.push(r["test_id"].clone());
        }
    }
    println!(
        "{} failed rows checked",
        rows.iter().filter(|r| r["status"] == "fail").count()
    );
    assert!(bad.is_empty(), "{bad:?}");
}

/// Item 36: no published text claims validated status. Every module's author
/// sign-off is "not yet checked by the author" (docs/VALIDATION_MATRIX.md section
/// 21), so any claim phrase fails, whatever the rows say.
#[test]
fn no_published_text_claims_validated_status() {
    let claims = [
        "is validated",
        "are validated",
        "fully validated",
        "validated model",
        "validated engine",
        "validated against",
        "has been validated against",
    ];
    let mut files = vec![repo().join("README.md")];
    for d in ["docs", "frontend"] {
        if let Ok(entries) = fs::read_dir(repo().join(d)) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension()
                    .is_some_and(|x| x == "md" || x == "js" || x == "html" || x == "json")
                {
                    files.push(p);
                }
            }
        }
    }
    files.push(repo().join("engine/wasm/src/registry.rs"));
    files.sort();
    let mut bad = Vec::new();
    for p in &files {
        let Ok(text) = fs::read_to_string(p) else {
            continue;
        };
        let low = text.to_lowercase();
        for c in claims {
            if low.contains(c) {
                bad.push(format!("{}: {c}", p.display()));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

/// Item 35, partly adopted: the register and the code agree in both directions. The
/// test resolves each `engine_fn` of `math/eq_register.csv` to an item of that name
/// in the named crate and module, and counts the cards that carry an "eq: X.n" doc
/// line in the engine. The full rule (every row resolved or marked not built) is
/// not met yet; this pins today's counts so the gap cannot grow, and prints the
/// unresolved list.
#[test]
fn register_and_code_agree_or_the_gap_is_pinned() {
    let text = fs::read_to_string(repo().join("math/eq_register.csv")).unwrap();
    let mut lines = text.lines();
    let head = split_csv(lines.next().unwrap());
    let col = |n: &str| head.iter().position(|h| h == n).unwrap();
    let (c_id, c_fn) = (col("id"), col("engine_fn"));
    let sources = library_sources(&ALL_CRATES);
    let defines = |lines: &[String], item: &str| {
        lines.iter().any(|l| {
            let t = l.trim_start();
            let t = t
                .strip_prefix("pub(crate) ")
                .or_else(|| t.strip_prefix("pub "))
                .unwrap_or(t);
            [
                "fn ", "const ", "struct ", "enum ", "static ", "type ", "mod ",
            ]
            .iter()
            .any(|k| {
                t.strip_prefix(k).is_some_and(|r| {
                    r.starts_with(item)
                        && !r[item.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
                })
            })
        })
    };
    let (mut total, mut resolved) = (0usize, 0usize);
    let mut unresolved = Vec::new();
    for l in lines.filter(|l| !l.trim().is_empty()) {
        let row = split_csv(l);
        let id = row[c_id].clone();
        for path in row[c_fn].split(';').map(str::trim) {
            if path.is_empty() || !path.contains("::") {
                continue;
            }
            total += 1;
            let parts: Vec<&str> = path.split("::").collect();
            let (krate, item) = (parts[0], *parts.last().unwrap());
            let module = if parts.len() > 2 { parts[1] } else { "" };
            let hit = sources.iter().any(|(f, ls)| {
                let in_crate = f.starts_with(&format!("engine/{krate}/src/"));
                let in_module = module.is_empty()
                    || f.ends_with(&format!("/{module}.rs"))
                    || f.contains(&format!("/{module}/"));
                in_crate && in_module && defines(ls, item)
            });
            if hit {
                resolved += 1;
            } else {
                unresolved.push(format!("{id} {path}"));
            }
        }
    }
    let mut cards = BTreeSet::new();
    for (_, ls) in &sources {
        for l in ls {
            if let Some(pos) = l.find("eq: ") {
                let rest = &l[pos + 4..];
                let card: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
                    .collect();
                if card.contains('.') {
                    cards.insert(card.trim_end_matches('.').to_string());
                }
            }
        }
    }
    println!(
        "register engine_fn paths: {total}, resolved {resolved}, unresolved {}; cards with \
         an eq doc line: {}",
        unresolved.len(),
        cards.len()
    );
    for u in &unresolved {
        println!("  unresolved: {u}");
    }
    // Pinned at the counts this test measured when it was added; fewer unresolved
    // paths and more cards are better.
    assert!(
        unresolved.len() <= UNRESOLVED_CEILING,
        "{} unresolved",
        unresolved.len()
    );
    assert!(cards.len() >= EQ_CARDS_FLOOR, "{} cards", cards.len());
}

/// Measured by `register_and_code_agree_or_the_gap_is_pinned` when it was added:
/// 193 of 299 `engine_fn` paths unresolved, 94 cards with an "eq:" doc line.
const UNRESOLVED_CEILING: usize = 193;
const EQ_CARDS_FLOOR: usize = 94;
