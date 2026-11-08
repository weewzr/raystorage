//! M7 acceptance, plan sections 12 and 14. Limits are the plan's and stay as
//! written. The suite is solved once and shared. Each test prints what it
//! computed, so a run with `--nocapture` gives the table.
//!
//! `carbon_matches_anchor` follows lead decision S1: the comparison is a row in
//! `results/Validation_Results.csv`, so this test asserts that each row exists
//! with finite values and that its pass flag follows the unchanged 2% tolerance.
//! A miss stays recorded as failed in the row.

use nush2_numerics::EngineError;
use nush2_plant::anchored::{Case, Model, Scenario};
use nush2_plant::flowsheet::{
    self, carbon_vs_anchor, check_elements, check_energy, check_mass, check_species, envelope,
    h2_ledger, three_start_spread, FeedBasis, FlowsheetSpec, Suite, TearStart, CARBON_ANCHOR_TOL,
    ELEMENT_TOL, ENERGY_TOL, THREE_START_TOL,
};
use nush2_units::stream::Sp;
use std::sync::OnceLock;

fn model() -> &'static Model {
    static M: OnceLock<Model> = OnceLock::new();
    M.get_or_init(|| Model::embedded().unwrap())
}

fn suite() -> &'static Suite {
    static S: OnceLock<Suite> = OnceLock::new();
    S.get_or_init(|| flowsheet::suite(model()).unwrap())
}

#[test]
fn flowsheet_elements_close() {
    for r in &suite().runs {
        let e = check_elements(r).unwrap();
        println!("{} envelope C H O N {:?}", r.id, e);
        for (k, v) in e.iter().enumerate() {
            assert!(*v < ELEMENT_TOL, "{} element {k}: {v:e}", r.id);
        }
        for u in &r.flowsheet.units {
            for v in u.closure.element_rel {
                assert!(v < ELEMENT_TOL, "{} unit {}: {v:e}", r.id, u.tag);
            }
        }
    }
}

#[test]
fn flowsheet_energy_closes() {
    for r in &suite().runs {
        let e = check_energy(r).unwrap();
        println!("{} envelope energy {e:e}", r.id);
        assert!(e < ENERGY_TOL, "{}: {e:e}", r.id);
        for u in &r.flowsheet.units {
            assert!(
                u.closure.energy_rel < ENERGY_TOL,
                "{} unit {}: {:e}",
                r.id,
                u.tag,
                u.closure.energy_rel
            );
        }
    }
}

#[test]
fn eq_a_2_envelope_mass_closes() {
    for r in &suite().runs {
        let e = check_mass(r).unwrap();
        assert!(e < ELEMENT_TOL, "{}: {e:e}", r.id);
    }
}

#[test]
fn eq_a_3_units_without_reaction_keep_every_species() {
    for r in &suite().runs {
        let (gap, unit) = check_species(r).unwrap();
        // The preheater reads the stored S03, which is the cryogenic outlet and
        // differs from its own input by the tear residual.
        assert!(gap < 1.0e-9, "{} {unit}: {gap:e}", r.id);
    }
}

#[test]
fn carbon_matches_anchor() {
    let m = model();
    let run = suite().run("FS_B_REC_F").unwrap();
    let rows = carbon_vs_anchor(m, run).unwrap();
    assert_eq!(rows.len(), 8);
    for c in &rows {
        let lit = c.literature.unwrap();
        println!(
            "{}: anchored {:.3} flowsheet {:.3} rel {:+.4} {}",
            c.test_id,
            lit,
            c.simulated,
            (c.simulated - lit) / lit,
            if c.pass { "pass" } else { "FAIL" }
        );
        assert!(c.simulated.is_finite() && lit.is_finite() && lit > 0.0);
        assert_eq!(c.tolerance, CARBON_ANCHOR_TOL);
        assert_eq!(c.tolerance, 0.02);
        let rel = (c.simulated - lit).abs() / lit;
        assert_eq!(c.pass, rel <= 0.02, "{}", c.test_id);
    }
    let all = flowsheet::checks(m, suite()).unwrap();
    for c in &rows {
        assert!(all.iter().any(|a| a == c), "{} missing", c.test_id);
    }
    let gate = all.iter().find(|c| c.test_id == "FS_GATE_M7").unwrap();
    assert_eq!(
        gate.pass,
        all.iter().all(|c| c.pass || c.test_id == "FS_GATE_M7")
    );
}

#[test]
fn tear_converges_from_three_starts() {
    let m = model();
    let r = suite().closure.h2_return;
    for (case, scn) in [(Case::B, Scenario::RecF), (Case::A, Scenario::InlSel)] {
        let ng = m.ng_c6 * m.scale(case);
        let spec = FlowsheetSpec::new(m, case, scn, FeedBasis::NaturalGas(ng), r);
        let (spread, passes) = three_start_spread(m, &spec).unwrap();
        println!(
            "{case:?} {}: spread {spread:e}, passes {passes:?}",
            scn.id()
        );
        assert!(spread < THREE_START_TOL, "{spread:e}");
    }
}

#[test]
fn fallback_with_fixed_lean_loading_runs_and_is_logged() {
    let m = model();
    let mut spec = FlowsheetSpec::new(
        m,
        Case::B,
        Scenario::RecF,
        FeedBasis::NaturalGas(m.ng_c6 * m.s_b),
        suite().closure.h2_return,
    );
    spec.solvent_tear = false;
    let r = flowsheet::solve(m, &spec, "FALLBACK", TearStart::Anchored).unwrap();
    assert!(!r.tear.solvent_tear);
    assert!(
        r.flags.iter().any(|f| f.starts_with("fallback")),
        "{:?}",
        r.flags
    );
    let lean = r.solvent("S18").unwrap();
    assert!((lean.n_co2 / lean.n_mea - m.lean).abs() < 1e-12);
    assert!(check_elements(&r).unwrap().iter().all(|v| *v < ELEMENT_TOL));
}

#[test]
fn connectivity() {
    for r in &suite().runs {
        let p = flowsheet::connectivity(r);
        assert!(p.is_empty(), "{}: {p:?}", r.id);
    }
    // A broken sheet is caught.
    let mut broken = suite().run("FS_B_REC_F").unwrap().clone();
    broken.flowsheet.units.retain(|u| u.tag != "REC-CRYO");
    let p = flowsheet::connectivity(&broken);
    assert!(p.iter().any(|s| s.starts_with("S03")), "{p:?}");
    assert!(p.iter().any(|s| s.starts_with("S14")), "{p:?}");
}

#[test]
fn bounds_enforced() {
    let m = model();
    let base = FlowsheetSpec::new(
        m,
        Case::B,
        Scenario::RecF,
        FeedBasis::NaturalGas(23.0),
        suite().closure.h2_return,
    );
    let range = |s: &FlowsheetSpec| {
        matches!(
            flowsheet::solve(m, s, "BOUND", TearStart::Anchored),
            Err(EngineError::Range { .. })
        )
    };
    for t in [900.0, 950.0, 537.0, f64::NAN] {
        let mut s = base.clone();
        s.t_reformer_out_c = t;
        assert!(range(&s), "T {t}");
    }
    for p in [430.0, 446.0] {
        let mut s = base.clone();
        s.p_reformer_out_psia = p;
        assert!(range(&s), "P {p}");
    }
    let mut s = base.clone();
    s.s_to_c = 0.0;
    assert!(range(&s));
    let mut s = base;
    s.scenario = Scenario::Export;
    assert!(range(&s));
}

#[test]
fn eq_j_12_psa_recovery_and_purity() {
    let m = model();
    for r in &suite().runs {
        let feed = r.gas("S12").unwrap().flow(Sp::H2) + r.gas("S21").unwrap().flow(Sp::H2);
        let p = r.gas("S13").unwrap();
        assert!(
            (p.flow(Sp::H2) / feed - m.psa_rec).abs() < 1e-12,
            "{}",
            r.id
        );
        let y = p.flow(Sp::Ch4) / p.total();
        assert!((y - m.ch4_in_h2).abs() < 1e-12, "{}", r.id);
        assert_eq!(p.flow(Sp::Co) + p.flow(Sp::Co2) + p.flow(Sp::N2), 0.0);
    }
}

#[test]
fn eq_j_13_h2_ledger_lines() {
    let m = model();
    let r = suite().run("FS_B_REC_F").unwrap();
    let l = h2_ledger(m, r).unwrap();
    println!("{l:?}");
    assert!((l.product_h2_kg_h - m.h2_b_kgh).abs() < 1e-6);
    assert!((l.annual_t_y - m.h2_b_kgh * m.hours / 1000.0).abs() < 1e-6);
    assert!((l.psa_recovery - m.psa_rec).abs() < 1e-12);
    assert!(l.plant_recovery > l.psa_recovery && l.plant_recovery < 1.0);
    // The shift adds H2 after the reformer, and the cryogenic return adds more
    // ahead of the PSA.
    assert!(l.after_capture_h2_kg_h > l.reformer_h2_kg_h);
    assert!(l.psa_feed_h2_kg_h > l.after_capture_h2_kg_h);
}

#[test]
fn eq_j_10_cryogenic_split_holds_the_inl_fraction() {
    let f = flowsheet::recycle_fraction(model());
    for r in &suite().runs {
        assert!((r.carbon.recycle_share() - f).abs() < 1e-12, "{}", r.id);
        // The cut is the only outlet of the feed nitrogen.
        let n2_in = r.gas("S01").unwrap().flow(Sp::N2) * 3.6;
        assert!((r.carbon.n2_cut - n2_in).abs() < 1e-9 * n2_in, "{}", r.id);
    }
}

#[test]
fn eq_j_11_flowsheet_carbon_closes() {
    for r in &suite().runs {
        let c = r.carbon.closure().unwrap();
        assert!(c.relative_residual.abs() < ELEMENT_TOL, "{}: {c:?}", r.id);
    }
}

#[test]
fn h2_return_closes_the_inl_pair() {
    let m = model();
    let s = suite();
    let r = s.run("FS_A_INL_SEL").unwrap();
    println!("{:?}", s.closure);
    assert!(s.closure.h2_return > 0.0 && s.closure.h2_return < 1.0);
    assert!((r.ng_mmscfd - m.ng_c6).abs() < 1e-12);
    assert!((r.h2_product_kg_h / m.h2_a_kgh - 1.0).abs() < 1e-9);
    // The same fraction is carried to every other run.
    for x in &s.runs {
        assert_eq!(x.spec.h2_return, s.closure.h2_return);
    }
}

#[test]
fn hydrogen_basis_is_met_on_every_run() {
    let m = model();
    for r in &suite().runs {
        let want = match r.spec.basis {
            FeedBasis::Hydrogen(kg_h) => kg_h,
            FeedBasis::NaturalGas(_) => continue,
        };
        assert!((r.h2_product_kg_h / want - 1.0).abs() < 1e-9, "{}", r.id);
        if r.spec.case == Case::B {
            assert_eq!(want, m.h2_b_kgh);
        }
    }
}

#[test]
fn reformer_temperature_runs_through_the_flowsheet() {
    let s = suite();
    for (id, t) in [
        ("FS_B_REC_F", 871.0),
        ("FS_B_REC_F_T825", 825.0),
        ("FS_B_REC_F_T800", 800.0),
    ] {
        let r = s.run(id).unwrap();
        let s05 = r.gas("S05").unwrap();
        println!(
            "{id}: NG {:.4} MMSCFD, process heat {:.3} MW, captured {:.2} kmol/h, burned {:.2} kmol C/h",
            r.ng_mmscfd,
            r.process_heat_mw(),
            r.carbon.captured,
            r.carbon.burned
        );
        assert_eq!(r.spec.t_reformer_out_c, t);
        assert!((s05.t_k - (t + 273.15)).abs() < 1e-9);
        let e = envelope(r).unwrap();
        assert!(e.energy_rel < ENERGY_TOL);
    }
}

#[test]
fn term_slots_are_filled_and_stay_unused() {
    let m = model();
    let s = suite();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let mut terms = nush2_plant::terms::terms(m, &b);
    nush2_plant::terms::fill_m7(&mut terms, &s.terms);
    println!("{:?}", s.terms);
    for t in &terms {
        // M8 adds the T03 cross-check slot from the same suite.
        let filled = matches!(t.id, "T03" | "T04" | "T09" | "T13");
        assert_eq!(t.modelled.is_some(), filled, "{}", t.id);
        assert_eq!(t.used, nush2_plant::terms::Source::Anchored);
        if let Some(v) = t.modelled {
            assert!(v.is_finite() && v > 0.0, "{}", t.id);
        }
    }
    let r = s.run("FS_B_REC_F").unwrap();
    assert_eq!(s.terms.t13_mmscfd, r.ng_mmscfd);
}

#[test]
fn full_tier_carries_the_flowsheet() {
    let mut inputs = nush2_plant::Inputs::headline();
    inputs.tier = nush2_plant::Tier::Full;
    let sol = nush2_plant::run(&inputs).unwrap();
    let fs = sol.flowsheet.as_ref().unwrap();
    assert!(fs.streams.contains_key("S03") && fs.streams.contains_key("S18"));
    let fast = nush2_plant::run(&nush2_plant::Inputs::headline()).unwrap();
    assert!(fast.flowsheet.is_none());
    // The anchored balance is the same in both tiers.
    assert_eq!(sol.heat, fast.heat);
    assert_eq!(sol.carbon, fast.carbon);
}
