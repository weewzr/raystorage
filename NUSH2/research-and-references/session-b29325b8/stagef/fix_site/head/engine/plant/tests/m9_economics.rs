//! M9 tests: the sweeps move the way the physics and the ledger say they must,
//! the levers reach the cost target, the DCF family matches the ledger family,
//! and the Headline balance closes on the split of the power branch.

use nush2_plant::anchored::{Model, Scenario};
use nush2_plant::economics::{CostQuery, Stage};
use nush2_plant::export::{branch_split, dcf_family};
use nush2_plant::params::ParamSet;
use nush2_plant::sens::{
    cofunding_rows, levers, mature_rows, stepdown_rows, sweeps, FsInputs, SensRow, NOT_CONVERGED,
};
use std::sync::OnceLock;

/// The sweeps on the embedded registry, with no flowsheet inputs, so the
/// flowsheet points must show as `not_converged` rows.
fn rows() -> &'static [SensRow] {
    static ROWS: OnceLock<Vec<SensRow>> = OnceLock::new();
    ROWS.get_or_init(|| sweeps(&ParamSet::embedded().unwrap(), &FsInputs::default()).unwrap())
}

/// Results of one sweep and metric, sorted by test value.
fn series(sweep: &str, run: Option<&str>, metric: &str) -> Vec<(f64, f64)> {
    let mut v: Vec<(f64, f64)> = rows()
        .iter()
        .filter(|r| r.sweep_id == sweep && r.metric == metric)
        .filter(|r| run.is_none_or(|id| r.run_id == id))
        .map(|r| (r.test_value, r.result))
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(v.len() >= 2, "{sweep} {metric}: {v:?}");
    v
}

fn rising(v: &[(f64, f64)]) -> bool {
    v.windows(2).all(|w| w[1].1 > w[0].1)
}

fn falling(v: &[(f64, f64)]) -> bool {
    v.windows(2).all(|w| w[1].1 < w[0].1)
}

#[test]
fn sens_trends() {
    // Reactor need rises with the reboiler duty.
    assert!(rising(&series("SRD", None, "R_NEEDED")));
    // Abatement falls as capture falls, and rises with availability.
    assert!(rising(&series("CAPTURE", None, "A_T_Y")));
    assert!(rising(&series("AVAILABILITY", None, "A_T_Y")));
    // Cost falls as availability rises: fixed charges spread over more tonnes.
    for m in ["AC_L5_25", "LAC_DCF_25"] {
        assert!(falling(&series("AVAILABILITY", None, m)), "{m}");
    }
    // Cost rises with T&S, reactor OCC, capture capital and the discount rate.
    for sweep in ["TS", "REACTOR_CAPITAL", "CAPTURE_CAPITAL", "DISCOUNT"] {
        for m in [
            "AC_L3_25",
            "AC_L5_25",
            "AC_L5_60",
            "LAC_DCF_25",
            "LAC_DCF_60",
        ] {
            assert!(rising(&series(sweep, None, m)), "{sweep} {m}");
        }
    }
    // Saved gas is a credit, so dearer gas lowers the cost.
    for m in ["AC_L5_25", "LAC_DCF_25"] {
        assert!(falling(&series("GAS", None, m)), "{m}");
    }
    // At 210 MWth, below the 600 MWth reference, a smaller exponent costs more.
    assert!(falling(&series("SIZE_EXPONENT", None, "AC_L5_25")));
    // A longer life lowers the annual charge on both methods.
    for m in ["AC_L3", "AC_L5", "LAC_DCF", "LCOH_NUSH2"] {
        assert!(falling(&series("LIFE", None, m)), "{m}");
    }
    // Charging the need costs less than charging the rating.
    let charged = series("CHARGED", None, "AC_L5_25");
    assert!(charged[0].1 < charged[1].1);
    // FOAK costs more than BOAK.
    let stage = series("STAGE", None, "AC_L5_25");
    assert!(stage[0].1 > stage[1].1, "{stage:?}");
    // Capture with the backup train adds abatement; unabated firing adds none.
    let backup = series("BACKUP_FIRING", None, "A_EXTRA_T_Y");
    assert_eq!(backup[0].1, 0.0);
    assert!(backup[1].1.abs() < 1e-6, "{backup:?}");
    assert!(backup[2].1 > 0.0 && backup[3].1 > 0.0, "{backup:?}");
    // Ahn's smaller gas saving costs more on both bases, and only the basis
    // where carbon follows the gas loses abatement.
    let one = |run: &str, metric: &str| -> f64 {
        let hits: Vec<f64> = rows()
            .iter()
            .filter(|r| r.sweep_id == "GAS_SAVING_AHN" && r.run_id == run && r.metric == metric)
            .map(|r| r.result)
            .collect();
        assert_eq!(hits.len(), 1, "{run} {metric}");
        hits[0]
    };
    for run in ["B_REC_F_CARBON_HELD", "B_REC_F_CARBON_FOLLOWS"] {
        assert!(one(run, "AC_L5_25") > one("B_REC_F", "AC_L5_25"), "{run}");
    }
    assert_eq!(one("B_REC_F_CARBON_HELD", "A_T_Y"), one("B_REC_F", "A_T_Y"));
    assert!(one("B_REC_F_CARBON_FOLLOWS", "A_T_Y") < one("B_REC_F", "A_T_Y"));
}

#[test]
fn a_sweep_without_its_flowsheet_stays_visible() {
    for sweep in ["REFORMER_T", "LTS_K_SOURCE"] {
        let bad: Vec<&SensRow> = rows()
            .iter()
            .filter(|r| r.sweep_id == sweep && r.status == Some(NOT_CONVERGED))
            .collect();
        assert!(!bad.is_empty(), "{sweep}");
        assert!(bad.iter().all(|r| r.result.is_nan()));
    }
    let vent: Vec<&SensRow> = rows()
        .iter()
        .filter(|r| r.sweep_id == "CONDENSATE_CO2" && r.run_id == "B_REC_F_VENTED")
        .collect();
    assert!(!vent.is_empty());
    assert!(vent.iter().all(|r| r.status == Some(NOT_CONVERGED)));
}

#[test]
fn eq_l_31_levers_reach_the_target() {
    let p = ParamSet::embedded().unwrap();
    let m = Model::new(&p).unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let a = m.emissions(&b.c, m.s_b).a_atm;
    let rows = levers(&p).unwrap();
    let mut checked = 0;
    for r in rows.iter().filter(|r| r.metric.ends_with("_NEEDED")) {
        let f: Vec<&str> = r.run_id.split('_').collect();
        let stage = if f[1] == "BOAK" {
            Stage::Boak
        } else {
            Stage::Foak
        };
        let life: f64 = f[2].trim_end_matches('Y').parse().unwrap();
        let q0 = CostQuery::central(stage, life, m.n_linear, m.r_rated);
        let q = match r.metric {
            "GAS_NEEDED" => CostQuery {
                gas: Some(r.result),
                ..q0
            },
            "OCC_NEEDED" => CostQuery {
                rx_mult: r.result / m.occ_kwt.central,
                ..q0
            },
            "TS_NEEDED" => CostQuery {
                ts: Some(r.result),
                ..q0
            },
            other => panic!("unknown lever {other}"),
        };
        let cost = if f[0] == "L5" {
            m.ledger(&b, &q).unwrap().l5() * 1e6 / a
        } else {
            m.dcf(&b, &q).unwrap().lac
        };
        assert!(
            (cost - m.target_c).abs() < 1e-6,
            "{} {}: {cost}",
            r.run_id,
            r.metric
        );
        checked += 1;
    }
    // L5 and DCF, BOAK and FOAK, 25 y and 60 y, three levers each.
    assert_eq!(checked, 24);
}

/// MATURE_60Y and the co-funding solve land on the mature-scenario target: the
/// solved gas price and the solved share of reactor capital, put back into each
/// method, give the target.
#[test]
fn eq_l_31_mature_and_cofunding_reach_their_target() {
    let p = ParamSet::embedded().unwrap();
    let m = Model::new(&p).unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let a = m.emissions(&b.c, m.s_b).a_atm;
    let cost = |l5: bool, q: CostQuery| {
        if l5 {
            m.ledger(&b, &q).unwrap().l5() * 1e6 / a
        } else {
            m.dcf(&b, &q).unwrap().lac
        }
    };
    let mature = mature_rows(&p).unwrap();
    let q0 = CostQuery::central(Stage::Oak10FromBoak, m.lives[2], m.n_linear, m.r_rated);
    let mut n = 0;
    for r in mature.iter().filter(|r| r.metric == "GAS_NEEDED") {
        let l5 = r.run_id.ends_with("_L5");
        let c = cost(
            l5,
            CostQuery {
                gas: Some(r.result),
                ..q0
            },
        );
        assert!((c - m.target_mature_c).abs() < 1e-6, "{}: {c}", r.run_id);
        n += 1;
    }
    assert_eq!(n, 2);
    let co = cofunding_rows(&p).unwrap();
    let mut n = 0;
    for r in co.iter().filter(|r| r.metric == "COFUND_SHARE") {
        let f: Vec<&str> = r.run_id.split('_').collect();
        let life: f64 = f[2].trim_end_matches('Y').parse().unwrap();
        let q = CostQuery {
            rx_mult: 1.0 - r.result,
            ..CostQuery::central(Stage::Boak, life, m.n_linear, m.r_rated)
        };
        let c = cost(f[0] == "L5", q);
        assert!((c - m.target_mature_c).abs() < 1e-6, "{}: {c}", r.run_id);
        assert!((0.0..1.0).contains(&r.result), "{}", r.run_id);
        n += 1;
    }
    assert_eq!(n, 4);
}

/// The printed MATURE_60Y gas price is the larger solve rounded up to S$0.1/GJ,
/// both methods sit at or below the mature-scenario target there, and each
/// step-down bar equals the cost point it stands for: the ledger grid for the
/// first three bars and the MATURE_60Y point at the printed price for the last.
#[test]
fn eq_l_31_stepdown_bars_equal_their_source_rows() {
    let p = ParamSet::embedded().unwrap();
    let m = Model::new(&p).unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let mature = mature_rows(&p).unwrap();
    let pick = |run: &str, metric: &str, test: Option<f64>| -> f64 {
        let hits: Vec<_> = mature
            .iter()
            .filter(|r| r.run_id == run && r.metric == metric)
            .filter(|r| test.is_none_or(|t| r.test_value == t))
            .collect();
        assert_eq!(hits.len(), 1, "{run} {metric} {test:?}");
        hits[0].result
    };
    let x = pick("MATURE_60Y", "GAS_PRINTED", None);
    let g_l5 = pick("MATURE_60Y_L5", "GAS_NEEDED", None);
    let g_dcf = pick("MATURE_60Y_DCF", "GAS_NEEDED", None);
    assert_eq!(x, (g_l5.max(g_dcf) * 10.0).ceil() / 10.0);
    assert!(x >= g_l5 && x >= g_dcf && x - g_l5.max(g_dcf) < 0.1 + 1e-12);
    let at_x_l5 = pick("MATURE_60Y_L5", "COST_S_PER_T", Some(x));
    let at_x_dcf = pick("MATURE_60Y_DCF", "COST_S_PER_T", Some(x));
    assert!(at_x_l5 <= m.target_mature_c, "{at_x_l5}");
    assert!(at_x_dcf <= m.target_mature_c, "{at_x_dcf}");

    let steps = stepdown_rows(&p).unwrap();
    assert_eq!(steps.len(), 8);
    let cp = |stage: Stage, life: f64| {
        nush2_plant::cost_point(&m, &b, stage, life, m.n_linear, m.r_rated).unwrap()
    };
    for (run, want_l5, want_dcf) in [
        (
            "STEPDOWN_1_BOAK_25Y",
            cp(Stage::Boak, m.lives[0]).ac_l5,
            cp(Stage::Boak, m.lives[0]).dcf.lac,
        ),
        (
            "STEPDOWN_2_10OAK_P46_25Y",
            cp(Stage::Oak10FromBoak, m.lives[0]).ac_l5,
            cp(Stage::Oak10FromBoak, m.lives[0]).dcf.lac,
        ),
        (
            "STEPDOWN_3_10OAK_P46_60Y",
            cp(Stage::Oak10FromBoak, m.lives[2]).ac_l5,
            cp(Stage::Oak10FromBoak, m.lives[2]).dcf.lac,
        ),
        ("STEPDOWN_4_GAS_PRINTED", at_x_l5, at_x_dcf),
    ] {
        for (metric, want) in [("AC_L5", want_l5), ("LAC_DCF", want_dcf)] {
            let r = steps
                .iter()
                .find(|r| r.run_id == run && r.metric == metric)
                .unwrap_or_else(|| panic!("{run} {metric}"));
            assert!(
                (r.result - want).abs() <= 1e-9 * want.abs(),
                "{run} {metric}: {} against {want}",
                r.result
            );
            assert!((r.result - r.base_value - r.delta).abs() < 1e-9);
        }
    }
    // Bar 3 is the MATURE_60Y point at the central gas price.
    let bar3 = |metric: &str| {
        steps
            .iter()
            .find(|r| r.run_id == "STEPDOWN_3_10OAK_P46_60Y" && r.metric == metric)
            .unwrap()
            .result
    };
    let central = m.gas.central;
    let l5_central = pick("MATURE_60Y_L5", "COST_S_PER_T", Some(central));
    let dcf_central = pick("MATURE_60Y_DCF", "COST_S_PER_T", Some(central));
    assert!((bar3("AC_L5") - l5_central).abs() < 1e-9);
    assert!((bar3("LAC_DCF") - dcf_central).abs() < 1e-9);
    // Every bar steps down.
    for metric in ["AC_L5", "LAC_DCF"] {
        let v: Vec<f64> = steps
            .iter()
            .filter(|r| r.metric == metric)
            .map(|r| r.result)
            .collect();
        assert!(v.windows(2).all(|w| w[1] < w[0]), "{metric}: {v:?}");
    }
}

#[test]
fn dcf_family_matches_the_ledger_family() {
    let m = Model::embedded().unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let fam = dcf_family(&m, &b).unwrap();
    // Four stages, three lives, three exponents, rated and need charged.
    assert_eq!(fam.len(), 4 * 3 * 3 * 2);
    for d in &fam {
        let sum: f64 = d.gap_terms.iter().map(|t| t.delta).sum();
        assert!((sum - d.gap_abs).abs() < 1e-9 * d.l5.abs().max(1.0));
        // Build order section 2: a gap over 10% needs a named term in the docs.
        assert!(
            d.gap_rel.abs() <= 0.10,
            "{:?} {} y n {} R {}: LAC {} against L5 {}",
            d.stage,
            d.life_y,
            d.n,
            d.r_charged,
            d.lac,
            d.l5
        );
        assert!(d.lcoh_nush2 > d.lcoh_smr);
    }
}

#[test]
fn eq_b_19_branch_split_adds_up() {
    let m = Model::embedded().unwrap();
    for scn in [Scenario::RecF, Scenario::InlSel] {
        for s in [1.0, m.s_b] {
            let b = m.balance(scn, &m.opts(s)).unwrap();
            let k = branch_split(&m, &b);
            let sum = k.to_reboiler + k.extraction_work + k.to_turbine;
            assert!((sum - b.branch).abs() < 1e-9, "{sum} against {}", b.branch);
            assert!(k.to_turbine >= 0.0 && k.extraction_work >= 0.0);
        }
    }
}
