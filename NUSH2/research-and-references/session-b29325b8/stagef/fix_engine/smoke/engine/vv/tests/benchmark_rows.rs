//! Tests of the benchmark rows. Decision S1 keeps code tests and literature
//! benchmarks apart: these tests check that each row is produced, finite and
//! judged against its unchanged limit. They never assert that a benchmark passes.
//! Run with `--nocapture` to print the rows.

use nush2_vv::fixture::{unread, OWN};
use nush2_vv::harness::{literature_rows, unit_rows};
use nush2_vv::props_bench::{SHOMATE_CP_REL, SHOMATE_H_ABS_KJ, SHOMATE_S_REL};
use nush2_vv::report::{csv, Row, Status};
use std::collections::BTreeSet;
use std::sync::OnceLock;

fn literature() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| literature_rows().unwrap())
}

fn units() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| unit_rows(false).unwrap())
}

fn find<'a>(rows: &'a [Row], id: &str) -> &'a Row {
    rows.iter()
        .find(|r| r.test_id == id)
        .unwrap_or_else(|| panic!("{id} is missing"))
}

/// Each row is well formed for its status.
fn well_formed(rows: &[Row]) {
    let mut ids = BTreeSet::new();
    for r in rows {
        assert!(ids.insert(r.test_id.clone()), "{} appears twice", r.test_id);
        assert!(
            !r.source.is_empty() && !r.locator.is_empty(),
            "{}",
            r.test_id
        );
        match r.status {
            Status::Pass | Status::Fail => {
                // A row judged on a range or a trend (XF-1, TAN-TREND) states its
                // limit in the explanation and carries no single tolerance.
                assert!(r.simulated.is_finite(), "{}", r.test_id);
                assert!(r.tolerance.is_none_or(f64::is_finite), "{}", r.test_id);
            }
            Status::Reported => {
                assert!(r.simulated.is_finite(), "{}", r.test_id);
                assert!(r.tolerance.is_none(), "{}", r.test_id);
            }
            Status::NotRun => {
                // A row that cannot run may still carry its frozen limit (KH-1,
                // KH-2), but never a simulated value.
                assert!(r.explanation.starts_with("not_run:"), "{}", r.test_id);
                assert!(!r.simulated.is_finite(), "{}", r.test_id);
                assert!(r.tolerance.is_none_or(f64::is_finite), "{}", r.test_id);
            }
        }
    }
}

#[test]
fn literature_rows_are_well_formed() {
    let rows = literature();
    well_formed(rows);
    print!("{}", csv(rows));
}

/// No test id repeats across the groups the exporter joins. The M6 column rows
/// take minutes in a debug build and are left to the export.
#[test]
fn ids_are_unique_across_groups() {
    let plant = nush2_vv::harness::plant_rows().unwrap();
    let mut ids = BTreeSet::new();
    for r in plant.iter().chain(literature()).chain(units()) {
        assert!(
            ids.insert(r.test_id.as_str()),
            "{} appears twice",
            r.test_id
        );
    }
}

#[test]
fn unit_rows_are_well_formed() {
    let rows = units();
    well_formed(rows);
    print!("{}", csv(rows));
}

/// The Shomate rows exist, are finite, and are judged against the limits of the
/// former M2 test, unchanged: Cp 0.1% and S 0.01% relative, H 0.01 kJ/mol.
#[test]
fn shomate_spot_values() {
    let rows = literature();
    for (id, tol, relative) in [
        ("PROPS-SHOMATE-CP", SHOMATE_CP_REL, true),
        ("PROPS-SHOMATE-S", SHOMATE_S_REL, true),
        ("PROPS-SHOMATE-H", SHOMATE_H_ABS_KJ, false),
    ] {
        let r = find(rows, id);
        let lit = r.literature.expect("a literature value");
        assert!(r.simulated.is_finite() && lit.is_finite(), "{id}");
        assert_eq!(r.tolerance, Some(tol), "{id}");
        assert!(
            r.conditions.starts_with("54 rows"),
            "{id}: {}",
            r.conditions
        );
        let worst = if relative {
            ((r.simulated - lit) / lit).abs()
        } else {
            (r.simulated - lit).abs()
        };
        // The row shows the worst point, so its gap decides the status.
        assert_eq!(r.status, Status::of(worst < tol), "{id}");
        println!("{id}: {:?}, worst {worst:.3e} against {tol:e}", r.status);
    }
}

/// Decision S2: the LTS keeps Zhang K2, and the NIST constant runs as one
/// sensitivity row with no limit.
#[test]
fn lts_nist_row_is_reported() {
    let r = find(units(), "T961-6-CO-NIST");
    assert_eq!(r.status, Status::Reported);
    assert!(r.simulated.is_finite());
    let zhang = find(units(), "T961-6-CO");
    assert!(matches!(zhang.status, Status::Pass | Status::Fail));
    println!(
        "LTS dry CO: Zhang {:.4} ({:?}), NIST {:.4}, printed {:?}",
        zhang.simulated, zhang.status, r.simulated, r.literature
    );
}

/// The JAERI conversion rows keep their unchanged 0.03 limit, and A-2 stays the
/// one calibration row.
#[test]
fn jaeri_rows_keep_their_limit() {
    for (id, data_use) in [
        ("JA-A-2-X", "calibration"),
        ("JA-A-3-X", "independent"),
        ("JA-A-4-X", "independent"),
        ("JA-A-5-X", "independent"),
        // Uncalibrated rows at activity 1.0 beside the fitted ones.
        ("JA-A-2-X-UNCAL", "independent"),
        ("JA-A-3-X-UNCAL", "independent"),
        ("JA-A-4-X-UNCAL", "independent"),
        ("JA-A-5-X-UNCAL", "independent"),
    ] {
        let r = find(units(), id);
        let lit = r.literature.expect("a literature value");
        assert_eq!(r.tolerance, Some(0.03), "{id}");
        assert_eq!(r.data_use, data_use, "{id}");
        assert_eq!(
            r.status,
            Status::of((r.simulated - lit).abs() <= 0.03),
            "{id}"
        );
    }
}

/// The 104 Aronu points and their aggregate are present.
#[test]
fn aronu_rows_are_present() {
    let rows = literature();
    let n = rows
        .iter()
        .filter(|r| r.test_id.starts_with("ARONU_104_"))
        .count();
    assert_eq!(n, 104);
    let agg = find(rows, "AR-1");
    assert!(agg.simulated.is_finite());
    let fix = find(rows, "AR-1-FIXTURE");
    println!(
        "AR-1 {:?} {}; AR-1-FIXTURE {:?}",
        agg.status, agg.simulated, fix.status
    );
}

/// The M6 column rows are well formed, and decision S7 holds for the
/// fixtures that `nush2_units::bench` reads: after every column benchmark has run
/// once, each stored row must have been returned by a lookup. The column rows
/// take about a minute in a debug build.
#[test]
fn column_rows_and_fixtures() {
    let rows = nush2_vv::harness::unit_rows(true).unwrap();
    well_formed(&rows);
    assert!(rows.iter().any(|r| r.test_id == "DICE-SRD"));
    let mut bad = Vec::new();
    for (name, _) in nush2_units::bench::COLUMN_FIXTURES {
        let left = nush2_units::bench::unread_fixture_rows(name).unwrap();
        if !left.is_empty() {
            bad.push(format!(
                "{name}: {} rows unread, first {:?}",
                left.len(),
                &left[..left.len().min(5)]
            ));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// Decision S7: a fixture read by this crate holds only values a benchmark
/// reads. Each stored row must be returned by at least one lookup.
#[test]
fn fixtures_hold_only_read_values() {
    let _ = literature();
    let _ = units();
    let mut bad = Vec::new();
    for (name, _) in OWN {
        let left = unread(name).unwrap();
        if !left.is_empty() {
            bad.push(format!(
                "{name}: {} rows unread, first {:?}",
                left.len(),
                &left[..left.len().min(5)]
            ));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}
