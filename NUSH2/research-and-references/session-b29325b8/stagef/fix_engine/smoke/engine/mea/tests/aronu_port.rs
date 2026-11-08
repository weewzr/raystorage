//! Port acceptance for the Aronu model. Printed lines
//! with the value that each check found use `--nocapture`.

mod common;

use common::{aard, Table};
use nush2_mea::aronu::{
    henry_water_co2_pa_m3_mol, henry_water_n2o_pa_m3_mol, ln_gamma_inf_mea, Thermo, I_CO2, I_H2O,
    I_H3O, I_HCO3, I_MEA, I_MEACOO, I_MEAH, NS, SPECIES,
};
use nush2_mea::enthalpy::dh_abs_j_mol;
use nush2_mea::pstar::{
    a_water, alpha_at, henry_apparent_pa_m3_mol, pco2_gas_kpa, pstar_kpa, pstar_lookup_kpa,
    pstar_table, water_partial_pressure_pa, PStarTable, TABLE_NA, TABLE_NT,
};
use nush2_mea::speciate::{speciate, speciate_on, KMeaBasis, Speciation};
use nush2_mea::validation::{
    model_points, run_model_points, summarise, AARD_LIMIT_PCT, POINT_TOLERANCE,
};

const PCT_30: f64 = 0.30;

#[test]
fn port_golden_60_points() {
    let t = Table::read("aronu_port_golden.csv");
    assert_eq!(t.rows.len(), 60);
    let mut worst = 0.0_f64;
    let mut worst_at = String::new();
    let mut check = |got: f64, want: f64, what: String| {
        let dev = (got - want).abs() / want.abs().max(1e-300);
        if dev > worst {
            worst = dev;
            worst_at = what;
        }
    };
    for r in &t.rows {
        let (w, tk, a) = (t.f(r, "w_mea"), t.f(r, "T_K"), t.f(r, "alpha"));
        let s = speciate(tk, w, a).unwrap();
        let tag = format!("w {w} T {tk} a {a}");
        check(
            nush2_mea::pstar::pstar_of(&s),
            t.f(r, "pco2_kPa_ideal"),
            format!("p* at {tag}"),
        );
        check(
            ln_gamma_inf_mea(tk).unwrap(),
            t.f(r, "lngamma_inf_MEA"),
            format!("ln gamma inf at {tag}"),
        );
        for (i, name) in [
            "H2O", "MEA", "CO2", "H3O", "MEAH", "OH", "HCO3", "CO3", "MEACOO",
        ]
        .iter()
        .enumerate()
        {
            check(
                s.x[i],
                t.f(r, &format!("x_{name}")),
                format!("x {name} at {tag}"),
            );
            check(
                s.ln_gamma[i],
                t.f(r, &format!("lng_{name}")),
                format!("ln gamma {name} at {tag}"),
            );
        }
    }
    println!("port_golden_60_points: worst relative deviation {worst:.3e} at {worst_at}");
    assert!(worst <= 1e-6, "worst {worst:.3e} at {worst_at}");
}

#[test]
fn aronu_104_model_columns() {
    let rows = run_model_points().unwrap();
    let s = summarise(&rows).unwrap();
    for w in [0.15, 0.30, 0.45, 0.60] {
        let sub: Vec<_> = rows
            .iter()
            .filter(|r| r.point.w_mea == w)
            .cloned()
            .collect();
        let t = summarise(&sub).unwrap();
        println!(
            "aronu_104 w {w:.2}: n {}, AARD corrected {:.3}%, AARD ideal gas {:.2}%, worst {:.3}%",
            t.n, t.aard_pct, t.aard_ideal_pct, t.max_abs_dev_pct
        );
    }
    println!(
        "aronu_104 all: n {}, AARD corrected {:.3}%, AARD ideal gas {:.2}%, worst {:.3}%, failed {}",
        s.n, s.aard_pct, s.aard_ideal_pct, s.max_abs_dev_pct, s.failed_points
    );
    assert_eq!(s.n, 104);
    assert_eq!(s.failed_points, 0, "tolerance {POINT_TOLERANCE}");
    assert!(s.aard_pct <= AARD_LIMIT_PCT, "AARD {}", s.aard_pct);
}

#[test]
fn eq_g_07_k_mea_shift_is_required() {
    // 17 model-column points at 30 mass%, as derivation card G.7 states.
    let pts: Vec<_> = model_points()
        .unwrap()
        .into_iter()
        .filter(|p| p.w_mea == PCT_30)
        .collect();
    assert_eq!(pts.len(), 17);
    let run = |basis| -> Vec<(f64, f64)> {
        pts.iter()
            .map(|p| {
                let s = speciate_on(p.t_k, p.w_mea, p.alpha, basis).unwrap();
                (nush2_mea::pstar::pstar_of(&s), p.pco2_model_kpa)
            })
            .collect()
    };
    let printed = aard(&run(KMeaBasis::AsPrinted));
    let shifted = aard(&run(KMeaBasis::PureMea));
    println!("k_mea_shift_is_required: AARD as printed {printed:.1}%, shifted {shifted:.2}%");
    assert!((printed - 58.8).abs() < 0.5, "as printed {printed}");
    assert!((shifted - 1.04).abs() < 0.1, "shifted {shifted}");
}

#[test]
fn gamma_inf() {
    let a = ln_gamma_inf_mea(313.15).unwrap();
    let b = ln_gamma_inf_mea(393.15).unwrap();
    println!("gamma_inf: {a:.4} at 313.15 K, {b:.4} at 393.15 K");
    assert!((a + 1.279).abs() <= 0.002, "{a}");
    assert!((b + 0.397).abs() <= 0.002, "{b}");
}

#[test]
fn residual_grid() {
    let mut worst = 0.0_f64;
    let mut count = 0;
    for w in [0.15, 0.30, 0.45, 0.60] {
        for tc in (40..=120).step_by(10) {
            let t = tc as f64 + 273.15;
            for k in 0..=24 {
                let a = 0.017 + (0.9 - 0.017) * k as f64 / 24.0;
                let s = speciate(t, w, a).unwrap();
                worst = worst.max(s.residual);
                count += 1;
            }
        }
    }
    println!("residual_grid: {count} states, largest residual {worst:.3e}");
    assert!(worst < 1e-9, "{worst}");
}

#[test]
fn rich_end() {
    let a378 = alpha_at(313.15, PCT_30, 378.0).unwrap();
    let a45 = alpha_at(313.15, PCT_30, 45.0).unwrap();
    println!("rich_end: loading {a378:.4} at 378 kPa, {a45:.4} at 45 kPa, 40 C");
    assert!((a378 - 0.741).abs() <= 0.003, "{a378}");
    assert!((a45 - 0.583).abs() <= 0.003, "{a45}");
}

#[test]
fn aronu_table4_data() {
    let t = Table::read("aronu_table4_data.csv");
    assert_eq!(t.rows.len(), 36);
    let pairs: Vec<(f64, f64)> = t
        .rows
        .iter()
        .map(|r| {
            let m = pstar_kpa(t.f(r, "t_k"), PCT_30, t.f(r, "alpha")).unwrap();
            (m, t.f(r, "pco2_kpa"))
        })
        .collect();
    let a = aard(&pairs);
    println!("aronu_table4_data: n 36, AARD {a:.2}%");
    assert!(a <= 25.0, "{a}");
}

#[test]
fn dugas_table44() {
    let t = Table::read("dugas_table44.csv");
    assert_eq!(t.rows.len(), 12);
    let w = 7.0 * 0.06108 / (1.0 + 7.0 * 0.06108);
    let pairs: Vec<(f64, f64)> = t
        .rows
        .iter()
        .map(|r| {
            let m = pstar_kpa(t.f(r, "t_k"), w, t.f(r, "alpha")).unwrap() * 1e3;
            (m, t.f(r, "pstar_pa"))
        })
        .collect();
    let a = aard(&pairs);
    println!("dugas_table44: n 12, AARD {a:.2}%");
    assert!(a <= 12.0, "{a}");
}

/// Largest relative error of the table at 1,000 fixed random states, and where it falls.
fn table_worst(w: f64) -> (f64, f64, f64) {
    let mut worst = (0.0_f64, 0.0, 0.0);
    // Deterministic generator: a constant seed keeps the sample fixed across runs.
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    for _ in 0..1000 {
        let t = 303.15 + 100.0 * next();
        let a = 0.02 + 0.73 * next();
        let direct = pstar_kpa(t, w, a).unwrap();
        let table = pstar_lookup_kpa(t, w, a).unwrap();
        let dev = (table / direct - 1.0).abs();
        if dev > worst.0 {
            worst = (dev, t, a);
        }
    }
    worst
}

#[test]
fn pstar_table_interp() {
    let (worst, t, a) = table_worst(PCT_30);
    println!(
        "pstar_table_interp: worst {:.4}% at T {t:.2} K, loading {a:.4}",
        100.0 * worst
    );
    assert!(worst <= 0.005, "{worst}");
}

/// The gate above covers the design strength. This runs the same sample at the other
/// strengths and only reports them, because the table grid does not reach 0.5% at 45
/// and 60 mass%. The grid stays as built.
#[test]
fn pstar_table_interp_other_strengths() {
    for w in [0.15, 0.45, 0.60] {
        let (worst, t, a) = table_worst(w);
        println!(
            "pstar_table_interp_other_strengths: w {w:.2}, worst {:.4}% at T {t:.2} K, loading {a:.4}",
            100.0 * worst
        );
        assert!(worst.is_finite());
    }
}

/// Corrigendum Table 3 prints 487.652 kPa and loading 0.784 at two total pressures. See
/// the note in `validation`. This reports both readings and pins which one the set holds.
#[test]
fn repeated_table3_row() {
    let held: Vec<_> = model_points()
        .unwrap()
        .into_iter()
        .filter(|p| p.pco2_model_kpa == 487.652)
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].ptot_kpa, 671.9);
    for ptot in [505.3, 671.9] {
        let p = pco2_gas_kpa(333.15, 0.15, 0.784, ptot).unwrap();
        println!(
            "repeated_table3_row: total pressure {ptot} kPa, port {p:.3} kPa, deviation {:+.3}%",
            100.0 * (p / 487.652 - 1.0)
        );
    }
}

#[test]
fn eq_g_08_and_g_09_and_g_10_and_g_11_speciation_balances_close() {
    for t in [313.15, 353.15, 393.15] {
        for a in [0.02, 0.1, 0.25, 0.5, 0.75, 0.9] {
            let s = speciate(t, PCT_30, a).unwrap();
            for (name, e) in ["MEA", "C", "H", "O", "charge"]
                .iter()
                .zip(s.balance_errors())
            {
                assert!(e.abs() < 1e-9, "{name} {e} at {t}, {a}");
            }
            assert!((s.x.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(s.n.iter().all(|v| *v > 0.0), "species {SPECIES:?}");
        }
    }
}

#[test]
fn eq_g_32_range_error_outside_validity() {
    assert!(speciate(250.0, PCT_30, 0.3).is_err());
    assert!(speciate(450.0, PCT_30, 0.3).is_err());
    assert!(speciate(313.15, 0.05, 0.3).is_err());
    assert!(speciate(313.15, 0.70, 0.3).is_err());
    assert!(speciate(313.15, PCT_30, 0.0).is_err());
    assert!(speciate(313.15, PCT_30, 2.0).is_err());
    assert!(speciate(f64::NAN, PCT_30, 0.3).is_err());
    assert!(pstar_lookup_kpa(290.0, PCT_30, 0.3).is_err());
    assert!(pstar_lookup_kpa(313.15, PCT_30, 0.9).is_err());
    assert!(alpha_at(313.15, PCT_30, -1.0).is_err());
    assert!(Thermo::new(313.15).unwrap().ln_gamma(&[-1.0; NS]).is_err());
}

#[test]
fn eq_g_24_heat_of_absorption_regression() {
    // Prototype table, kJ/mol CO2, 30 mass%. Kind: plausibility. A regression
    // on the prototype with no validation claim.
    let want = [
        (0.10, [87.8, 90.0, 92.1, 94.1, 96.2]),
        (0.30, [84.7, 86.3, 88.0, 89.5, 90.5]),
        (0.40, [81.9, 82.6, 83.0, 82.8, 81.3]),
        (0.50, [61.5, 61.9, 61.9, 61.0, 58.9]),
        (0.60, [55.6, 48.3, 44.0, 41.0, 38.8]),
    ];
    let mut worst = 0.0_f64;
    for (a, row) in want {
        for (t_c, v) in [40.0, 60.0, 80.0, 100.0, 120.0].iter().zip(row) {
            let got = dh_abs_j_mol(t_c + 273.15, PCT_30, a).unwrap() / 1000.0;
            worst = worst.max((got - v).abs());
        }
    }
    println!("heat_of_absorption_regression: worst {worst:.3} kJ/mol");
    assert!(worst <= 0.1, "{worst}");
}

fn profile(t: f64) -> Vec<Speciation> {
    (0..=88)
        .map(|k| speciate(t, PCT_30, 0.02 + 0.01 * k as f64).unwrap())
        .collect()
}

#[test]
fn carbamate_peak_near_05() {
    let p = profile(313.15);
    let peak = p
        .iter()
        .max_by(|a, b| a.x[I_MEACOO].total_cmp(&b.x[I_MEACOO]))
        .unwrap();
    let cross = p
        .iter()
        .find(|s| s.x[I_HCO3] > s.x[I_MEACOO])
        .map(|s| s.alpha)
        .unwrap();
    println!(
        "carbamate_peak_near_05: peak at {:.2}, bicarbonate overtakes at {cross:.2}",
        peak.alpha
    );
    assert!((peak.alpha - 0.5).abs() <= 0.05, "{}", peak.alpha);
    assert!((cross - 0.65).abs() <= 0.05, "{cross}");
}

#[test]
fn eq_g_29_free_mea_falls_with_loading_and_water_activity_stays_below_one() {
    let p = profile(313.15);
    assert!(p.windows(2).all(|w| w[1].n[I_MEA] < w[0].n[I_MEA]));
    assert!(p.windows(2).all(|w| w[1].x[I_CO2] > w[0].x[I_CO2]));
    assert!(p.iter().all(|s| s.x[I_H3O] < s.x[I_MEAH]));
    let aw = a_water(313.15, PCT_30, 0.5).unwrap();
    assert!(aw > 0.8 && aw < 1.0, "{aw}");
    assert!(p[0].x[I_H2O] > 0.8);
}

#[test]
fn water_vapour_pressure_stays_below_pure_water() {
    let p = water_partial_pressure_pa(373.15, PCT_30, 0.3).unwrap();
    assert!(p > 0.7e5 && p < 1.013_25e5, "{p}");
}

/// Derivation card G.23: gamma*_CO2 times Aronu Eq 17 against Hartono Table 11 through
/// Eq 19, AARD 3.4% with n 15, against the 2.7% Aronu quotes (PDF p1).
#[test]
fn n2o_analogy_hartono_table11() {
    let t = Table::read("hartono_table11_n2o.csv");
    let pairs: Vec<(f64, f64)> = t
        .rows
        .iter()
        .filter(|r| t.f(r, "alpha") > 0.0)
        .map(|r| {
            let tk = t.f(r, "t_c") + 273.15;
            let k = t.f(r, "k_h_n2o_kpa_m3_mol") * 1e3;
            let exp = henry_water_co2_pa_m3_mol(tk) / henry_water_n2o_pa_m3_mol(tk) * k;
            (
                henry_apparent_pa_m3_mol(tk, PCT_30, t.f(r, "alpha")).unwrap(),
                exp,
            )
        })
        .collect();
    let a = aard(&pairs);
    println!(
        "n2o_analogy_hartono_table11: n {}, AARD {a:.2}%",
        pairs.len()
    );
    assert_eq!(pairs.len(), 15);
    assert!((a - 3.4).abs() < 0.1, "{a}");
}

/// Every state the range check accepts must either converge or return an error, and
/// this grid must leave no error behind.
#[test]
fn solver_covers_the_accepted_range() {
    let (mut ok, mut bad) = (0, Vec::new());
    for w in [0.10, 0.15, 0.20, 0.30, 0.40, 0.45, 0.50, 0.60] {
        for tc in (25..=130).step_by(15) {
            let t = tc as f64 + 273.15;
            for k in 0..=40 {
                // Log spaced from the lowest loading accepted to the highest.
                let a = 0.001 * (1400.0_f64).powf(k as f64 / 40.0);
                let a = a.min(1.4);
                match speciate(t, w, a) {
                    Ok(s) if s.residual < 1e-9 => ok += 1,
                    _ => bad.push((w, tc, a)),
                }
            }
        }
    }
    println!(
        "solver_covers_the_accepted_range: {ok} converged, {} failed",
        bad.len()
    );
    assert!(bad.is_empty(), "{:?}", &bad[..bad.len().min(12)]);
}

#[test]
fn table_nodes_equal_direct_solves_at_each_strength() {
    for w in [0.15, 0.30, 0.45, 0.60] {
        let tab = pstar_table(w).unwrap();
        for (i, j) in [
            (0, 0),
            (5, 20),
            (TABLE_NT - 1, TABLE_NA - 1),
            (TABLE_NT - 1, 0),
        ] {
            let (t, a) = (PStarTable::t_node(i), PStarTable::alpha_node(j));
            let want = pstar_kpa(t, w, a).unwrap();
            assert!((tab.pstar_kpa(t, a).unwrap() / want - 1.0).abs() < 1e-9);
        }
    }
}
