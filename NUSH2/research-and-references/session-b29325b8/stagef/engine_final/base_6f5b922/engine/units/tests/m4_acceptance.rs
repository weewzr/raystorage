//! M4 acceptance tests, plan section 12. Limits are the plan's and stay as written.
//! Each test prints what it computed, so a run with `--nocapture` gives the table.

use nush2_props::equilibrium::KSource;
use nush2_props::mixture;
use nush2_units::case5::{self, TrainOpts};
use nush2_units::closes;
use nush2_units::stream::Sp;
use nush2_units::wgs::Pellet;

fn k_c(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8
}

fn mw(mmbtu_h: f64) -> f64 {
    nush2_props::units::mmbtu_h_to_mw(mmbtu_h)
}

fn train() -> case5::Train {
    let raw = case5::raw_syngas().unwrap();
    case5::train(&raw, &TrainOpts::sized_on_ieaghg(KSource::Zhang).unwrap()).unwrap()
}

#[test]
fn case5_reformer_enthalpy() {
    let r = case5::reformer(KSource::Zhang).unwrap();
    closes!(r.closure);
    let want = mw(case5::DUTY_REFORM_1 + case5::DUTY_REFORM_2);
    let got = r.duty_w / 1.0e6;
    println!(
        "case5_reformer_enthalpy: {got:.2} MW against {want:.2} MW, {:+.2}%",
        100.0 * (got / want - 1.0)
    );
    assert!((got / want - 1.0).abs() <= 0.03);
}

#[test]
fn case5_smr_eq_raw_syngas() {
    let r = case5::reformer(KSource::Zhang).unwrap();
    let y = mixture::mole_fractions(&r.outlet.n).unwrap();
    let mut worst: f64 = 0.0;
    for (sp, pct) in case5::RAW_MOL_PCT {
        let got = 100.0 * y[sp as usize];
        println!("  raw {sp:?}: model {got:.2} printed {pct:.1} mol%");
        worst = worst.max((got - pct).abs());
    }
    println!(
        "case5_smr_eq_raw_syngas: worst gap {worst:.2} points, conversion {:.4}",
        r.conversion
    );
    assert!(worst <= 0.5);
}

#[test]
fn wgs_case5_hts() {
    let t = train();
    closes!(t.hts.closure);
    let got_t = t.hts.outlet.t_k - 273.15;
    let want_t = k_c(case5::HTS_OUT_T_F);
    let got_co = 100.0 * t.hts.outlet.dry_fraction(Sp::Co).unwrap();
    println!(
        "wgs_case5_hts: {got_t:.1} C against {want_t:.1} C; dry CO {got_co:.2} against {}; \
         Q/K {:.3}; bed {:.1} m3",
        case5::HTS_OUT_CO_DRY_PCT,
        t.hts.q_over_k,
        t.hts.bed_volume_m3
    );
    assert!((got_t - want_t).abs() <= 10.0);
    assert!((got_co - case5::HTS_OUT_CO_DRY_PCT).abs() <= 0.3);
}

#[test]
fn wgs_case5_lts() {
    let t = train();
    closes!(t.lts.closure);
    let got_t = t.lts.outlet.t_k - 273.15;
    let want_t = k_c(case5::LTS_OUT_T_F);
    let got_co = 100.0 * t.lts.outlet.dry_fraction(Sp::Co).unwrap();
    println!(
        "wgs_case5_lts: {got_t:.1} C against {want_t:.1} C; dry CO {got_co:.3} against {}",
        case5::LTS_OUT_CO_DRY_PCT
    );
    assert!((got_t - want_t).abs() <= 10.0);
    assert!((got_co - case5::LTS_OUT_CO_DRY_PCT).abs() <= 0.1);
}

#[test]
fn cooler_case5_duties() {
    let t = train();
    let rows = [
        ("HPS-GEN1", &t.hps_gen1, case5::DUTY_HPS_GEN1),
        ("WGS-EX1", &t.ex1, case5::DUTY_WGS_EX1),
        ("WGS-EX2", &t.ex2, case5::DUTY_WGS_EX2),
        ("WGS-EX3", &t.ex3, case5::DUTY_WGS_EX3),
        ("WGS-EX4", &t.ex4, case5::DUTY_WGS_EX4),
        ("KO-DRM-1", &t.ko, case5::DUTY_KO_DRM),
    ];
    let mut failed = Vec::new();
    for (name, unit, printed) in rows {
        closes!(unit.closure);
        let got = unit.duty_w / 1.0e6;
        let want = mw(printed);
        let rel = got / want - 1.0;
        println!(
            "cooler_case5_duties {name}: {got:.2} MW against {want:.2} MW, {:+.1}%",
            100.0 * rel
        );
        if rel.abs() > 0.10 {
            failed.push(name);
        }
    }
    assert!(failed.is_empty(), "outside 10%: {failed:?}");
}

#[test]
fn flash_case5_condensate() {
    let t = train();
    closes!(t.drum.closure);
    let liquid = t.drum.liquid.as_ref().unwrap();
    let got = liquid.mass_kg_s().unwrap() * 3.6;
    let want = case5::KO_LIQ_LB_H * nush2_props::units::LB / 1000.0;
    let co2 = 100.0 * t.drum.vapour.dry_fraction(Sp::Co2).unwrap();
    println!(
        "flash_case5_condensate: {got:.2} t/h against {want:.2} t/h, {:+.2}%; \
         dry CO2 {co2:.2}% against {}; dissolved CO2 {:.0} kg/h",
        100.0 * (got / want - 1.0),
        case5::SELEXOL_IN_CO2_PCT,
        liquid.flow(Sp::Co2) * 44.0095e-3 * 3600.0
    );
    assert!((got / want - 1.0).abs() <= 0.03);
    assert!((co2 - case5::SELEXOL_IN_CO2_PCT).abs() <= 0.3);
}

/// Sensitivities of the chain. Printed for the record and only held to closure.
/// The Chinchen space velocity row is the first basis tried for the bed: it missed
/// the printed HTS outlet, and `bench` keeps that row as failed.
#[test]
fn chain_sensitivities_are_reported() {
    let raw = case5::raw_syngas().unwrap();
    let sized = TrainOpts::sized_on_ieaghg(KSource::Zhang).unwrap();
    for (name, o) in [
        ("sized on IEAGHG", sized),
        (
            "sized, pellet S",
            TrainOpts {
                pellet: Pellet::S,
                ..sized
            },
        ),
        (
            "sized, NIST K",
            TrainOpts::sized_on_ieaghg(KSource::NistGibbs).unwrap(),
        ),
        (
            "Chinchen space velocity",
            TrainOpts::chinchen_space_velocity(KSource::Zhang),
        ),
    ] {
        let t = case5::train(&raw, &o).unwrap();
        closes!(t.hts.closure);
        closes!(t.lts.closure);
        println!(
            "{name}: HTS {:.1} C CO {:.2}% Q/K {:.3} bed {:.0} m3; LTS {:.1} C CO {:.3}%; \
             phi {:.4}; above 130 C {:.2} MW",
            t.hts.outlet.t_k - 273.15,
            100.0 * t.hts.outlet.dry_fraction(Sp::Co).unwrap(),
            t.hts.q_over_k,
            t.hts.bed_volume_m3,
            t.lts.outlet.t_k - 273.15,
            100.0 * t.lts.outlet.dry_fraction(Sp::Co).unwrap(),
            case5::phi_after_shift(&t.lts.outlet),
            case5::syngas_above_130c_w(&t).unwrap() / 1.0e6
        );
    }
    let rf = case5::reformer(KSource::Zhang).unwrap();
    let t = case5::train(&rf.outlet, &sized).unwrap();
    println!(
        "from SMR-EQ outlet: phi {:.4}; above 130 C {:.2} MW; HTS {:.1} C; LTS {:.1} C",
        case5::phi_after_shift(&t.lts.outlet),
        case5::syngas_above_130c_w(&t).unwrap() / 1.0e6,
        t.hts.outlet.t_k - 273.15,
        t.lts.outlet.t_k - 273.15
    );
}

/// Values M4 hands to the plant terms, from `bench::gas_terms`. T01 and T05 flip at
/// M8 only after their gates pass there; T03 stays anchored by plan, so only its
/// Case 5 check is printed.
#[test]
fn gas_terms_are_reported() {
    let g = nush2_units::bench::gas_terms().unwrap();
    println!(
        "T01 phi {:.4}; T03 Case 5 reformer check {:.2} MW; T05 Case 5 heat above 130 C {:.2} MW",
        g.t01_phi, g.t03_case5_reformer_mw, g.t05_case5_mw
    );
    assert!(g.t01_phi > 0.0 && g.t01_phi < 1.0);
    assert!(g.t05_case5_mw > 0.0);
}
