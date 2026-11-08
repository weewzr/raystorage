//! Density, viscosity and rate-side checks against Hartono et al. 2014.

mod common;

use common::{aard, Table};
use nush2_mea::aronu::{henry_water_co2_pa_m3_mol, henry_water_n2o_pa_m3_mol};
use nush2_mea::density::{c_mea_kmol_m3, density_kg_m3};
use nush2_mea::kinetics::{
    d_co2_solution_m2_s, e1_of_hatta, e_infinity, enhancement_wellek, hatta, henry_khan_pa_m3_mol,
    k2_hikita, overall_rate, B_STOICH,
};
use nush2_mea::viscosity::{ratio_to_water, viscosity_pa_s};

const W: f64 = 0.30;

#[test]
fn density_weiland() {
    let t = Table::read("hartono_table3_density.csv");
    assert_eq!(t.rows.len(), 24);
    let pairs: Vec<(f64, f64)> = t
        .rows
        .iter()
        .map(|r| {
            let m = density_kg_m3(t.f(r, "t_c") + 273.15, W, t.f(r, "alpha")).unwrap();
            (m, t.f(r, "rho_kg_m3"))
        })
        .collect();
    let a = aard(&pairs);
    let lo = pairs.iter().map(|(m, e)| m / e).fold(f64::MAX, f64::min);
    let hi = pairs.iter().map(|(m, e)| m / e).fold(f64::MIN, f64::max);
    println!("density_weiland: n 24, AARD {a:.2}%, ratio {lo:.3} to {hi:.3}");
    assert!(a <= 1.0, "{a}");
}

#[test]
fn viscosity_weiland() {
    let t = Table::read("hartono_table7_viscosity.csv");
    assert_eq!(t.rows.len(), 24);
    let pairs: Vec<(f64, f64)> = t
        .rows
        .iter()
        .map(|r| {
            let m = viscosity_pa_s(t.f(r, "t_c") + 273.15, W, t.f(r, "alpha")).unwrap() * 1e3;
            (m, t.f(r, "eta_mpa_s"))
        })
        .collect();
    let a = aard(&pairs);
    let lo = pairs.iter().map(|(m, e)| m / e).fold(f64::MAX, f64::min);
    let hi = pairs.iter().map(|(m, e)| m / e).fold(f64::MIN, f64::max);
    println!("viscosity_weiland: n 24, AARD {a:.2}%, ratio {lo:.3} to {hi:.3}");
    assert!(a <= 3.0, "{a}");
}

/// Hartono Table 11 tests Khan Eq 8 through the N2O analogy (Aronu Eq 19, PDF p4).
/// The plan reports this as a deviation, so the bound below only guards the unit.
/// Khan prints no unit for Eq 8, and atm L/mol is the assumption recorded in H.9.
#[test]
fn khan_henry_against_hartono_table11() {
    let t = Table::read("hartono_table11_n2o.csv");
    let mut loaded = Vec::new();
    let mut unloaded = Vec::new();
    for r in &t.rows {
        let (tc, a) = (t.f(r, "t_c"), t.f(r, "alpha"));
        let tk = tc + 273.15;
        let k_n2o = t.f(r, "k_h_n2o_kpa_m3_mol") * 1e3;
        let exp = k_n2o * henry_water_co2_pa_m3_mol(tk) / henry_water_n2o_pa_m3_mol(tk);
        let c = c_mea_kmol_m3(tk, W, a).unwrap();
        let model = henry_khan_pa_m3_mol(tk, c).unwrap();
        if a == 0.0 {
            unloaded.push((model, exp));
        } else {
            loaded.push((model, exp));
        }
    }
    let (la, ua) = (aard(&loaded), aard(&unloaded));
    let ratio = |p: &[(f64, f64)]| {
        (
            p.iter().map(|(m, e)| m / e).fold(f64::MAX, f64::min),
            p.iter().map(|(m, e)| m / e).fold(f64::MIN, f64::max),
        )
    };
    let (ll, lh) = ratio(&loaded);
    let (ul, uh) = ratio(&unloaded);
    println!(
        "khan_henry_against_hartono_table11: deviation, loaded n {} AARD {la:.1}% ratio {ll:.2} to {lh:.2}; unloaded n {} AARD {ua:.1}% ratio {ul:.2} to {uh:.2}",
        loaded.len(),
        unloaded.len()
    );
    assert_eq!(loaded.len() + unloaded.len(), 20);
    assert!(
        ll > 0.2 && lh < 2.0,
        "unit reading is suspect: {ll} to {lh}"
    );
}

/// One worked pass through the closure at an absorber state, with the arithmetic of
/// each step checked by the unit tests in `kinetics`. The film coefficients, area, CO2
/// pressure and the D_B to D_CO2 ratio here are placeholders with no source. They only
/// exercise the path, so this test carries no validation claim.
#[test]
fn rate_closure_runs_from_end_to_end() {
    let (t, a) = (313.15, 0.25);
    let c_total = c_mea_kmol_m3(t, W, a).unwrap();
    let c_free = c_total * (1.0 - 2.0 * a);
    let d = d_co2_solution_m2_s(t, ratio_to_water(t, W, a).unwrap()).unwrap();
    let k2 = k2_hikita(t).unwrap();
    let henry = henry_khan_pa_m3_mol(t, c_total).unwrap();
    let k_l0 = 1.0e-4;
    let ha = hatta(d, k2, c_free, k_l0).unwrap();
    let c_ai = 1.0e4 / henry / 1000.0;
    let e_inf = e_infinity(c_free, d * 0.6, B_STOICH, d, c_ai).unwrap();
    let e = enhancement_wellek(e_inf, e1_of_hatta(ha).unwrap()).unwrap();
    let r = overall_rate(1.0e4, 1.0e-5, 200.0, henry, k_l0, e).unwrap();
    println!("rate_closure: Ha {ha:.1}, E_inf {e_inf:.1}, E {e:.1}, r {r:.2} mol/m3/s");
    assert!(ha > 1.0 && e > 1.0 && e <= ha.min(e_inf) * 1.001);
    assert!(r > 0.0 && r.is_finite());
}
