//! M6 acceptance tests. Code tests and literature benchmarks are
//! kept apart: each benchmark test checks that its rows are produced, that every
//! row that ran is finite, and that `pass` is computed against the unchanged limit.
//! The benchmark outcome itself is printed and recorded by the harness, never
//! asserted here. Closure, grid and repeat-run checks are code tests and must hold.

use nush2_units::absorber::{self, AbsorberSpec};
use nush2_units::bench::{self, BenchRow};
use nush2_units::closes;
use nush2_units::columns;
use nush2_units::stream::Sp;
use nush2_units::stripper::{self, StripMode};

fn print_rows(rows: &[BenchRow]) {
    for r in rows {
        println!(
            "{} | lit {} | sim {:.4} {} | tol {} | {} | {}",
            r.test_id,
            r.literature.map_or("-".into(), |v| format!("{v:.4}")),
            r.simulated,
            r.unit,
            r.tolerance.map_or("-".into(), |v| format!("{v:.4}")),
            r.status(),
            r.explanation
        );
    }
}

/// Every row that ran is finite; a row with a limit carries a pass flag that
/// equals the comparison against that limit.
fn rows_are_honest(rows: &[BenchRow]) {
    assert!(!rows.is_empty());
    for r in rows {
        if r.status() == "not_run" {
            assert!(
                r.pass.is_none(),
                "{} has no result yet carries a flag",
                r.test_id
            );
            assert!(r.explanation.starts_with("not_run:"), "{}", r.test_id);
            continue;
        }
        assert!(r.simulated.is_finite(), "{}", r.test_id);
        if let (Some(l), Some(t)) = (r.literature, r.tolerance) {
            assert_eq!(r.pass, Some((r.simulated - l).abs() <= t), "{}", r.test_id);
        } else {
            assert!(r.pass.is_none(), "{}", r.test_id);
        }
    }
}

fn find<'a>(rows: &'a [BenchRow], id: &str) -> &'a BenchRow {
    rows.iter()
        .find(|r| r.test_id == id)
        .unwrap_or_else(|| panic!("{id} is missing"))
}

#[test]
fn absorber_khan_table1() {
    let rows = bench::absorber_khan_table1().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    assert_eq!(rows.len(), 5);
    for r in &rows {
        // Frozen limit of KH-1 and KH-2, and the fixture values.
        assert_eq!(r.tolerance, Some(2.0));
        assert_eq!(r.status(), "not_run");
    }
    assert_eq!(find(&rows, "KH-2-I06").literature, Some(99.92));
}

#[test]
fn absorber_salvinder_85pct() {
    let rows = bench::absorber_salvinder_85pct().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    let r = find(&rows, "SA-1");
    assert_eq!(r.literature, Some(85.0));
    assert_eq!(r.tolerance, Some(5.0));
}

#[test]
fn absorber_tan_trend() {
    let rows = bench::absorber_tan_trend().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    let trend = find(&rows, "TAN-TREND");
    assert_eq!(trend.tolerance, Some(0.0));
    let p: Vec<f64> = ["TAN-P0.1", "TAN-P1", "TAN-P5"]
        .iter()
        .map(|id| find(&rows, id).simulated)
        .collect();
    assert_eq!(trend.pass, Some(p[0] < p[1] && p[1] <= p[2] + 1e-9));
}

#[test]
fn absorber_sonderby_profiles() {
    // R3 carries the full loading profile; the harness runs R1 to R5.
    let rows = bench::absorber_sonderby_runs(&["R3"]).unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    assert!(rows.iter().all(|r| r.tolerance.is_none()));
    assert_eq!(
        rows.iter().filter(|r| r.test_id.contains("-TI")).count(),
        10
    );
    assert_eq!(rows.iter().filter(|r| r.test_id.contains("-V")).count(), 10);
}

#[test]
fn absorber_case_b_spec() {
    let rows = bench::absorber_case_b_spec().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    let spec = find(&rows, "ABS-CASEB-SPEC");
    assert_eq!(spec.tolerance, Some(0.0));
    if spec.status() != "not_run" {
        let h = find(&rows, "ABS-CASEB-HEIGHT").simulated;
        let rich = find(&rows, "ABS-CASEB-RICH").simulated;
        let meets = h.is_finite() && h <= columns::HEIGHT_LIMIT_M && rich <= 0.50 + 1e-6;
        assert_eq!(spec.pass, Some(meets));
        // The coefficient applied at both ends is the ideal vapour value of card
        // G.19, and each row names the SRK binary of card G.21 beside it.
        for id in ["ABS-CASEB-PHI-TOP", "ABS-CASEB-PHI-BOTTOM"] {
            let r = find(&rows, id);
            assert!((r.simulated - 1.0).abs() < 1e-9, "{id}: {}", r.simulated);
            assert!(r.explanation.contains("card G.21"), "{id}");
        }
    }
}

#[test]
fn stripper_dice() {
    let rows = bench::stripper_dice().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    let r = find(&rows, "DICE-SRD");
    let lit = r.literature.unwrap();
    // 242 MMBtu/h over the CO2 of streams 21 and 22 is 3.64 GJ/t, limit 10%.
    assert!((lit - 3.64).abs() < 0.005, "{lit}");
    assert!((r.tolerance.unwrap() - 0.1 * lit).abs() < 1e-12);
}

#[test]
fn stripper_kwak() {
    let rows = bench::stripper_kwak().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    let r = find(&rows, "KW-SRD");
    assert_eq!(r.literature, Some(3.92));
    assert!((r.tolerance.unwrap() - 0.392).abs() < 1e-12);
}

#[test]
fn stripper_morgan() {
    let rows = bench::stripper_morgan().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    for k in ["K14", "K15", "K16"] {
        assert_eq!(find(&rows, &format!("MO-{k}-TOUT")).tolerance, Some(1.4));
    }
}

#[test]
fn stripper_vinjarapu() {
    let rows = bench::stripper_vinjarapu().unwrap();
    print_rows(&rows);
    rows_are_honest(&rows);
    assert_eq!(find(&rows, "VI-1").tolerance, Some(10.0));
    assert_eq!(find(&rows, "VI-2").tolerance, Some(15.0));
    assert_eq!(
        rows.iter()
            .filter(|r| r.test_id.starts_with("VI-SRD-"))
            .count(),
        85
    );
    for id in ["VI-3-SS1", "VI-3-SS2", "VI-3-SS3"] {
        assert_eq!(find(&rows, id).tolerance, Some(0.03));
    }
}

#[test]
fn case_b_terms_are_reported() {
    let t = bench::column_terms().unwrap();
    println!("{t:#?}");
    assert!(
        t.failures.is_empty() || t.evidence_t07_t08 == nush2_units::stream::Evidence::GateFailed
    );
    if t.failures.is_empty() {
        assert!(t.t02_capture.is_finite());
        assert!(t.t06_srd_gj_t.is_some_and(f64::is_finite));
        assert!(t.t07_lean_pump_mwe.is_some_and(f64::is_finite));
        assert!(t.t08_cooling_mwe.is_some_and(f64::is_finite));
        // Section 7 limits raise findings, never edits.
        let srd = t.t06_srd_gj_t.unwrap();
        assert_eq!(
            t.findings.iter().any(|f| f.contains("SRD")),
            !(3.4..=4.2).contains(&srd)
        );
        // The island duties the live reply shows are the duties of these terms.
        let d = columns::design_duties().unwrap();
        assert_eq!(Some(d.reboiler_mw), t.reboiler_mw);
        assert_eq!(Some(d.condenser_mw), t.condenser_mw);
        assert_eq!(Some(d.lean_cooler_mw), t.lean_cooler_mw);
        assert_eq!(Some(d.rich_lean_mw), t.rich_lean_mw);
        assert!(d.rich_lean_mw > 0.0 && d.lean_cooler_mw > 0.0 && d.condenser_mw > 0.0);
    }
}

fn case_b_column(height: f64, slices: usize) -> absorber::AbsorberResult {
    let feed = columns::absorber_feed().unwrap();
    let lean = columns::lean_solvent(&feed).unwrap();
    let spec = columns::absorber_spec(&lean, columns::LIQUID_LOAD_M3_M2_H).unwrap();
    absorber::solve(
        &AbsorberSpec {
            height_m: height,
            slices,
            ..spec
        },
        &feed,
        &lean,
        None,
    )
    .unwrap()
}

/// The capture island closes on energy across its hand-offs: the absorber, the
/// exchanger, the stripper and the cooler share one liquid enthalpy. Limit
/// 1e-6 of the reboiler duty, the flowsheet limit of card A.15.
#[test]
fn eq_a_08_capture_island_closes() {
    let feed = columns::absorber_feed().unwrap();
    let lean = columns::lean_solvent(&feed).unwrap();
    let abs = columns::absorber_case(&feed, &lean, columns::LIQUID_LOAD_M3_M2_H).unwrap();
    let reg = columns::regenerate(&abs.result.rich, columns::ALPHA_LEAN).unwrap();
    let r = columns::island_energy_residual_w(&feed, &lean, &abs.result, &reg).unwrap();
    let q_reb = reg.stripper.q_reb_w;
    println!(
        "island residual {:.3e} W on a reboiler duty of {:.6e} W ({:.3e} relative); \
         height {:?} m, SRD {:.4} GJ/t, approach {:.6} K",
        r,
        q_reb,
        r / q_reb,
        abs.height_m,
        reg.srd_gj_t,
        reg.exchanger.t_lean_out_k - abs.result.rich.t_k
    );
    assert!((r / q_reb).abs() < 1e-6);
}

#[test]
fn absorber_closes() {
    let r = case_b_column(15.0, 40);
    println!(
        "closure {:?}, path {:?}, steps {}",
        r.closure, r.path, r.iterations
    );
    closes!(r.closure);
}

#[test]
fn eq_h_24_grid_doubling_moves_capture_little() {
    let a = case_b_column(15.0, 40);
    let b = case_b_column(15.0, 80);
    println!(
        "capture {:.5} on 40 slices, {:.5} on 80",
        a.capture, b.capture
    );
    assert!((a.capture - b.capture).abs() < 0.002);
}

#[test]
fn eq_h_26_repeat_run_is_identical() {
    let a = case_b_column(12.0, 40);
    let b = case_b_column(12.0, 40);
    assert_eq!(a.x, b.x);
    assert_eq!(a.capture, b.capture);
}

#[test]
fn eq_h_27_capture_counts_absorber_inlet_co2() {
    let r = case_b_column(15.0, 40);
    let feed = columns::absorber_feed().unwrap();
    let want = (feed.flow(Sp::Co2) - r.gas_out.flow(Sp::Co2)) / feed.flow(Sp::Co2);
    assert!((r.capture - want).abs() < 1e-12);
    assert!((absorber::capture(2.0, 0.2).unwrap() - 0.9).abs() < 1e-12);
}

#[test]
fn eq_i_03_stripper_closes_and_modes_agree() {
    let feed = columns::absorber_feed().unwrap();
    let lean = columns::lean_solvent(&feed).unwrap();
    let mut rich = lean.clone();
    rich.n_co2 = 0.5 * rich.n_mea;
    rich.t_k = 383.15;
    let a = stripper::stages(&columns::stripper_spec(StripMode::LeanLoading(0.25)), &rich).unwrap();
    closes!(a.closure);
    // The duty found for lean 0.25, fed back, returns lean 0.25.
    let b = stripper::stages(
        &columns::stripper_spec(StripMode::ReboilerDuty(a.q_reb_w)),
        &rich,
    )
    .unwrap();
    closes!(b.closure);
    let back = b.lean.n_co2 / b.lean.n_mea;
    println!("SRD {:.3} GJ/t, lean back {back:.6}", a.srd_j_kg / 1e6);
    assert!((back - 0.25).abs() < 1e-5);
}

#[test]
fn column_gate_fails_on_any_gated_miss() {
    let mut rows = bench::absorber_khan_table1().unwrap();
    // Khan rows lack source data and stay out of the gate.
    assert_eq!(
        bench::column_gate(&rows),
        nush2_units::stream::Evidence::NotRun
    );
    let mut miss = rows[0].clone();
    miss.test_id = "X".into();
    miss.simulated = 1.0;
    miss.pass = Some(false);
    miss.explanation = String::new();
    rows.push(miss);
    assert_eq!(
        bench::column_gate(&rows),
        nush2_units::stream::Evidence::GateFailed
    );
}
