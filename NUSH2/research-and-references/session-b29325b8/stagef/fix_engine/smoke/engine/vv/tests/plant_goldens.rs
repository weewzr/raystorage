//! Plant golden tests. A failure here is a port error: fix the code, never the
//! golden file.

use nush2_plant::anchored::{Model, Scenario};
use nush2_plant::economics::{CostQuery, Stage};
use nush2_plant::export::{dcf_grid, tables};
use nush2_plant::params::ParamSet;
use nush2_vv::bench::run_checks;
use nush2_vv::golden::{availability, balance_all_cases, ledger_grid, GoldStats};
use nush2_vv::harness::{max_carbon_residual, plant_rows, unread_params, LAC_GAP_LIMIT};

fn assert_gold(st: GoldStats) {
    assert!(st.compared > 0, "nothing compared");
    assert!(
        st.passed(),
        "{} of {} failed, worst error {:e}: {}",
        st.failed,
        st.compared,
        st.max_err,
        st.worst
    );
}

#[test]
fn golden_balance_all_cases() {
    assert_gold(balance_all_cases().unwrap());
}

#[test]
fn golden_ledger_grid() {
    assert_gold(ledger_grid().unwrap());
}

#[test]
fn golden_availability() {
    assert_gold(availability().unwrap());
}

#[test]
fn screening_21_checks() {
    let m = Model::embedded().unwrap();
    let checks = run_checks(&m).unwrap();
    assert_eq!(checks.len(), 21);
    for c in &checks {
        assert!(c.passed(), "{}: got {}, want {}", c.id, c.got, c.target);
    }
}

#[test]
fn headline_entry_values() {
    let m = Model::embedded().unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    assert!((m.h2_b_kgh * m.hours / 1000.0 - 66925.0).abs() < 1.0);
    assert!((b.q_proc - 120.8).abs() < 0.05);
    assert!((b.c.ng * m.s_b - 23.2).abs() < 0.05);
    assert!((b.r - 179.0).abs() < 0.05);
    assert!((m.r_rated - b.r - 31.0).abs() < 0.05);
    assert!((e.a_atm - 578052.0).abs() < 1.0);
}

#[test]
fn headline_cost_pair() {
    let m = Model::embedded().unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let a = m.emissions(&b.c, m.s_b).a_atm;
    for (stage, life, r, want) in [
        (Stage::Boak, 25.0, m.r_rated, 137.7),
        (Stage::Boak, 60.0, m.r_rated, 117.2),
        (Stage::Boak, 25.0, b.r, 113.1),
        (Stage::Boak, 60.0, b.r, 95.5),
        (Stage::Foak, 25.0, m.r_rated, 209.9),
        (Stage::Foak, 60.0, m.r_rated, 179.5),
    ] {
        let q = CostQuery::central(stage, life, m.n_linear, r);
        let got = m.ledger(&b, &q).unwrap().l3() * 1e6 / a;
        assert!((got - want).abs() < 0.05, "got {got}, want {want}");
    }
}

#[test]
fn sizing_walk() {
    let m = Model::embedded().unwrap();
    let walk = m.sizing_walk(m.s_b).unwrap();
    for (got, want) in walk
        .iter()
        .zip([206.0, 140.8, 150.6, 154.3, 165.9, 166.2, 179.0])
    {
        assert!((got - want).abs() < 0.05, "got {got}, want {want}");
    }
}

#[test]
fn eq_l_37_lac_dcf_against_l5() {
    let m = Model::embedded().unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let grid = dcf_grid(&m, &b).unwrap();
    assert_eq!(grid.len(), 12);
    for (d, _) in grid {
        assert!(
            d.gap_rel.abs() <= LAC_GAP_LIMIT,
            "{:?} {} y n {}: LAC {} against L5 {}",
            d.stage,
            d.life_y,
            d.n,
            d.lac,
            d.l5
        );
        assert!(d.lcoh_nush2 > d.lcoh_smr);
    }
}

#[test]
fn carbon_closes_on_every_run() {
    let m = Model::embedded().unwrap();
    assert!(max_carbon_residual(&m).unwrap() < 0.01);
}

#[test]
fn export_byte_identical() {
    let p = ParamSet::embedded().unwrap();
    assert_eq!(tables(&p).unwrap(), tables(&p).unwrap());
}

#[test]
fn params_complete() {
    let p = ParamSet::embedded().unwrap();
    let unread = unread_params(&p).unwrap();
    assert!(unread.is_empty(), "unread ids: {unread:?}");
    for e in p.entries() {
        assert!(!e.class.is_empty() && !e.rigour.is_empty() && !e.source.is_empty());
    }
}

/// Plant rows verify the port, so each one must pass. Literature benchmarks may
/// fail and are tested in `benchmark_rows.rs`.
#[test]
fn every_plant_row_passes() {
    let rows = plant_rows().unwrap();
    assert_eq!(rows.len(), 59);
    let failed: Vec<&str> = rows
        .iter()
        .filter(|r| !r.passed())
        .map(|r| r.test_id.as_str())
        .collect();
    assert!(failed.is_empty(), "failed: {failed:?}");
}
