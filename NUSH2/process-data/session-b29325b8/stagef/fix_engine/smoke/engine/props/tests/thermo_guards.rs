//! Guards on the property layer drawn from the OUTRAM PARK concept review
//! (concepts after OUTRAM PARK, AGPL-3.0, reference only; no OUTRAM code). Each
//! test names the derivation card it checks. Literature numbers come from the
//! fixtures in `engine/vv/data/props`, never typed here.

use nush2_props::equilibrium::{self, KSource};
use nush2_props::helium;
use nush2_props::species::{self, Sp, H2O_BRANCH_K};
use nush2_props::steam;
use nush2_props::transport::{self, FitFlag};
use nush2_props::units::R_GAS;

const JANAF: &str = include_str!("../../vv/data/props/janaf_spot.csv");

/// (s, log_kf) of one species at one temperature from the JANAF fixture.
fn janaf(name: &str, t_k: f64) -> (f64, f64) {
    for l in JANAF.lines() {
        if l.starts_with('#') || l.starts_with("species") || l.trim().is_empty() {
            continue;
        }
        let c: Vec<&str> = l.split(',').map(str::trim).collect();
        if c[0] == name && c[2].parse::<f64>().unwrap() == t_k {
            return (c[4].parse().unwrap(), c[6].parse().unwrap());
        }
    }
    panic!("{name} at {t_k} K is not in the JANAF fixture");
}

/// Reaction sums over (species, coefficient).
fn sum(reaction: &[(Sp, f64)], f: impl Fn(Sp) -> f64) -> f64 {
    reaction.iter().map(|(s, nu)| nu * f(*s)).sum()
}

const R1: [(Sp, f64); 4] = [
    (Sp::Co, 1.0),
    (Sp::H2, 3.0),
    (Sp::Ch4, -1.0),
    (Sp::H2o, -1.0),
];
const R2: [(Sp, f64); 4] = [
    (Sp::Co2, 1.0),
    (Sp::H2, 1.0),
    (Sp::Co, -1.0),
    (Sp::H2o, -1.0),
];

/// K1 carries the heat capacity integrals. Holding dH and dS at their
/// 298.15 K values drops both Kirchhoff integrals. At 1123.15 K the full route must
/// exceed the constant route by more than a factor of 5, so a regression to the
/// constant form fails.
#[test]
fn eq_d_07_k1_carries_heat_capacity_integrals() {
    let t = 1123.15;
    let dh = sum(&R1, |s| species::h_j_mol(s, 298.15).unwrap());
    let ds = sum(&R1, |s| species::s_j_mol_k(s, 298.15).unwrap());
    let k_const = (-(dh - t * ds) / (R_GAS * t)).exp();
    let k_full = equilibrium::k1_nist_bar2(t).unwrap();
    let k_zhang = equilibrium::k1_zhang_bar2(t).unwrap();
    println!(
        "K1 at {t} K: full Gibbs {k_full:.4e}, Zhang {k_zhang:.4e}, constant dH and dS \
         {k_const:.4e}, ratio {:.2}",
        k_full / k_const
    );
    assert!(k_full / k_const > 5.0, "{}", k_full / k_const);
}

/// Reaction enthalpy and entropy at 298.15 K summed from the species
/// table agree with the JANAF fixture: dS from the absolute entropies and dH from
/// dG (log Kf) plus T dS. Limits 0.1 kJ/mol and 0.1 J/mol/K from the review; the
/// fixture's three printed decimals of log Kf carry about 3 J/mol per species.
#[test]
fn eq_a_10_reaction_enthalpy_and_entropy_sum_from_species() {
    let t = 298.15;
    for (name, rxn) in [("R1", &R1), ("R2", &R2)] {
        let ds_janaf = sum(rxn, |s| janaf(s.name(), t).0);
        let dg_janaf = -R_GAS * t * std::f64::consts::LN_10 * sum(rxn, |s| janaf(s.name(), t).1);
        let dh_janaf = dg_janaf + t * ds_janaf;
        let dh = sum(rxn, |s| species::h_j_mol(s, t).unwrap());
        let ds = sum(rxn, |s| species::s_j_mol_k(s, t).unwrap());
        println!(
            "{name}: dH {:.3} kJ/mol against JANAF {:.3}; dS {ds:.3} J/mol/K against {ds_janaf:.3}",
            dh / 1000.0,
            dh_janaf / 1000.0
        );
        assert!((dh - dh_janaf).abs() < 100.0, "{name} dH");
        assert!((ds - ds_janaf).abs() < 0.1, "{name} dS");
    }
}

/// Liquid water enters the formation-basis balance through one
/// constant offset from the IF97 datum, 300 to 450 K, within 1 J/mol.
#[test]
fn eq_a_09_liquid_water_sits_on_one_offset() {
    let m = Sp::H2o.mw_kg_mol();
    let p = 10.0e5;
    let offset = |t: f64| {
        species::h_liquid_water_j_mol(t, p).unwrap() - steam::h_liquid_j_kg(t, p).unwrap() * m
    };
    let base = offset(300.0);
    for k in 0..=15 {
        let t = 300.0 + 10.0 * f64::from(k);
        assert!(
            (offset(t) - base).abs() < 1.0,
            "{t} K: {}",
            offset(t) - base
        );
    }
}

/// Petersen's heat capacity is the monatomic ideal value 5/2 R/M within
/// 0.1%, with R/M the 2077.3 J/kg/K the report uses (Eq. 3-2).
#[test]
fn eq_b_02_cp_is_the_monatomic_ideal_value() {
    let ideal = 2.5 * 2077.3;
    let rel = helium::CP_J_KG_K / ideal - 1.0;
    println!(
        "helium cp {} against 5/2 R/M {ideal:.2}: {rel:+.2e}",
        helium::CP_J_KG_K
    );
    assert!(rel.abs() < 1e-3, "{rel}");
}

/// Each correlation behaves as its doc declares just inside each edge
/// (Ok) and 1e-6 relative outside (Range error, or a flag where the doc says so).
#[test]
fn correlation_edges_follow_the_declared_policy() {
    let (t_lo, t_hi, p_lo, p_hi) = (273.0, 1800.0, 1.0e5, 100.0e5);
    let out = 1.0e-6;
    // Helium: hard Range error outside 273 to 1800 K and 1 to 100 bar.
    for t in [t_lo, t_hi] {
        assert!(helium::density_kg_m3(t, 50.0e5).is_ok(), "{t}");
        assert!(helium::viscosity_pa_s(t).is_ok(), "{t}");
        assert!(helium::conductivity_w_m_k(t, 50.0e5).is_ok(), "{t}");
    }
    for t in [t_lo * (1.0 - out), t_hi * (1.0 + out)] {
        assert!(helium::density_kg_m3(t, 50.0e5).is_err(), "{t}");
        assert!(helium::viscosity_pa_s(t).is_err(), "{t}");
        assert!(helium::conductivity_w_m_k(t, 50.0e5).is_err(), "{t}");
    }
    for p in [p_lo, p_hi] {
        assert!(helium::density_kg_m3(900.0, p).is_ok(), "{p}");
    }
    for p in [p_lo * (1.0 - out), p_hi * (1.0 + out)] {
        assert!(helium::density_kg_m3(900.0, p).is_err(), "{p}");
        assert!(helium::conductivity_w_m_k(900.0, p).is_err(), "{p}");
    }
    // Syngas transport: a flag outside 359 to 483 C, refused below 273.15 K.
    assert_eq!(
        transport::syngas(transport::T_LOW_K).unwrap().flag,
        FitFlag::Inside
    );
    assert_eq!(
        transport::syngas(transport::T_HIGH_K).unwrap().flag,
        FitFlag::Inside
    );
    assert_eq!(
        transport::syngas(transport::T_LOW_K * (1.0 - out))
            .unwrap()
            .flag,
        FitFlag::BelowFit
    );
    assert_eq!(
        transport::syngas(transport::T_HIGH_K * (1.0 + out))
            .unwrap()
            .flag,
        FitFlag::AboveFit
    );
    assert!(transport::syngas(273.15 * (1.0 - out)).is_err());
    // Equilibrium constants: Range error outside 298.15 to 1500 K.
    for src in [KSource::Zhang, KSource::NistGibbs] {
        assert!(src.pair(298.15).is_ok() && src.pair(1500.0).is_ok());
        assert!(src.pair(298.15 * (1.0 - out)).is_err());
        assert!(src.pair(1500.0 * (1.0 + out)).is_err());
    }
}

/// At each pressure where the engine evaluates water, `state` at
/// Tsat - 1 mK returns liquid and at Tsat + 1 mK returns vapour, never the other
/// phase. Liquid is taken as v below 0.005 m3/kg, which separates the two phases
/// at every pressure listed.
#[test]
fn if97_phase_follows_the_side_of_the_saturation_line() {
    for p in [1.0e5, 2.0e5, 4.5e5, 10.0e5, 26.8e5, 40.0e5] {
        let ts = steam::tsat_k(p).unwrap();
        let below = steam::state(ts - 1.0e-3, p).map(|s| s.v_m3_kg);
        let above = steam::state(ts + 1.0e-3, p).map(|s| s.v_m3_kg);
        if let Ok(v) = below {
            assert!(v < 0.005, "{p} Pa below Tsat: v {v}");
        }
        if let Ok(v) = above {
            assert!(v > 0.005, "{p} Pa above Tsat: v {v}");
        }
        assert!(below.is_ok() || above.is_ok(), "{p} Pa: both sides refused");
    }
}

/// The dilute Region 2 path that sets H2O vapour below 500 K holds down
/// to 273.15 K and refuses anything colder.
#[test]
fn dilute_region_2_path_holds_to_its_lower_edge() {
    assert!(steam::h_dilute_j_kg(273.15).is_ok());
    assert!(steam::h_dilute_j_kg(273.15 * (1.0 - 1.0e-6)).is_err());
    assert!(species::h_j_mol(Sp::H2o, 273.15).is_ok());
    assert!(species::h_j_mol(Sp::H2o, H2O_BRANCH_K - 1.0).is_ok());
}
