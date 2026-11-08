//! Fields of a plant run that the live reply reads: the heat duty above the
//! rated reactor, from the function the export writes as UNMET_THERMAL, and the
//! natural gas of the run's own case, from the function the export writes as
//! RES_NG_MMSCFD. Native tests; the wasm crate tests the reply built on them.

use nush2_plant::anchored::{Case, Model, Scenario};
use nush2_plant::{run, Inputs};
use std::path::PathBuf;

/// Minimal CSV split with quoted fields.
fn split(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

/// Cell `col` of the row whose first cells equal `keys`, in a results file.
fn cell(file: &str, keys: &[&str], col: &str) -> f64 {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../results")
        .join(file);
    let text = std::fs::read_to_string(path).expect("results file");
    let mut lines = text.lines();
    let head = split(lines.next().expect("header"));
    let ix = head.iter().position(|h| h == col).expect("column");
    let row = lines
        .map(split)
        .find(|r| keys.iter().enumerate().all(|(i, k)| r[i] == *k))
        .unwrap_or_else(|| panic!("{file}: no row {keys:?}"));
    row[ix].parse().expect("number")
}

fn rated(rating: f64) -> Inputs {
    let mut i = Inputs::headline();
    i.overrides.insert("REACTOR_RATED_MWTH".into(), rating);
    i
}

#[test]
fn unmet_heat_is_the_export_function() {
    for rating in [100.0, 150.0, 178.0, 210.0, 300.0] {
        let i = rated(rating);
        let sol = run(&i).unwrap();
        let m = Model::new(&i.params().unwrap()).unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        assert_eq!(sol.heat.unmet, m.unmet_thermal(rating, &b), "{rating}");
        let (_, heat_first) = m.shortfall_at(rating, &b);
        assert_eq!(sol.heat.unmet, (heat_first - rating).max(0.0), "{rating}");
        assert!(sol.heat.unmet >= 0.0);
    }
    // At 100 MWth the heat duties alone exceed the rating and the plant runs
    // short of heat as well as of electricity.
    let low = run(&rated(100.0)).unwrap();
    assert!(low.heat.margin < 0.0, "{}", low.heat.margin);
    assert!(low.heat.unmet > 0.0, "{}", low.heat.unmet);
    // At the default rating the run carries the exported row.
    let sol = run(&Inputs::headline()).unwrap();
    let row = cell(
        "Integrated_Energy_Balance.csv",
        &["B_REC_F", "UNMET_THERMAL"],
        "value_mw",
    );
    assert!(
        (sol.heat.unmet - row).abs() <= 1e-6,
        "{} {row}",
        sol.heat.unmet
    );
}

#[test]
fn natural_gas_follows_the_case_of_the_run() {
    for (case, col) in [(Case::B, "case_b"), (Case::A, "case_a_mea")] {
        let mut i = Inputs::headline();
        i.case = case;
        let sol = run(&i).unwrap();
        let want = cell("Final_Design_Basis.csv", &["RES_NG_MMSCFD"], col);
        assert!(
            (sol.ng_mmscfd - want).abs() <= 1e-6 * want,
            "{case:?}: run {} against {col} {want}",
            sol.ng_mmscfd
        );
    }
}
