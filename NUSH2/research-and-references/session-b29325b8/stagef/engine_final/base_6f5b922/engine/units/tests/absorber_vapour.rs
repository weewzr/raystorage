//! Vapour fugacity at absorber pressure (cards G.19 and G.21).
//!
//! Concept after OUTRAM PARK (AGPL-3.0), reference only: test a correction in the
//! regime where the model uses it. The SRK coefficient of card G.21 holds CO2 and
//! water only. In the plant absorber the vapour is H2-rich syngas at 26.8 bar, so
//! the binary describes the wrong mixture there.

use nush2_mea::pstar::{poynting, srk_co2_h2o};
use nush2_units::columns::{P_ABS_PA, T_LEAN_K, W_MEA};
use nush2_units::solvent::{Solvent, PHI_CO2_ABSORBER};

/// The coefficient `Solvent::pstar_gas_pa` applies, backed out of its own output:
/// ideal p* times Poynting divided by the corrected p*.
fn applied_phi(s: &Solvent, t_k: f64, alpha: f64, p_pa: f64) -> f64 {
    let ideal = s.pstar_pa(t_k, alpha).unwrap();
    let gas = s.pstar_gas_pa(t_k, alpha, p_pa).unwrap();
    ideal * poynting(t_k, p_pa).unwrap() / gas
}

#[test]
fn eq_g_21_phi_at_absorber_pressure() {
    let s = Solvent::new(W_MEA, 323.15).unwrap();
    // Top (lean 0.25) and bottom (rich 0.50) of the Case B absorber at 40 C.
    for (end, alpha) in [("top", 0.25), ("bottom", 0.50)] {
        let ideal = s.pstar_pa(T_LEAN_K, alpha).unwrap();
        let y = ideal / P_ABS_PA;
        let binary = srk_co2_h2o(T_LEAN_K, P_ABS_PA, y).unwrap();
        let phi = applied_phi(&s, T_LEAN_K, alpha, P_ABS_PA);
        println!(
            "{end}: alpha {alpha}, p* {ideal:.1} Pa, y_CO2 {y:.3e}; CO2-H2O SRK Z {:.4} \
             phi {:.2}; applied phi {phi:.6}",
            binary.z, binary.phi_co2
        );
        // The binary takes a liquid-like root here, which is why the absorber
        // must not use it.
        assert!(binary.z < 0.3, "{end}: Z {}", binary.z);
        assert!(binary.phi_co2 > 50.0, "{end}: phi {}", binary.phi_co2);
        // The coefficient the absorber applies is a vapour value.
        assert!((0.90..=1.05).contains(&phi), "{end}: applied phi {phi}");
    }
}

#[test]
fn eq_g_19_absorber_vapour_is_ideal_with_poynting() {
    assert_eq!(PHI_CO2_ABSORBER, 1.0);
    let s = Solvent::new(W_MEA, 323.15).unwrap();
    for (t, alpha, p) in [
        (313.15, 0.25, P_ABS_PA),
        (313.15, 0.50, P_ABS_PA),
        (299.15, 0.20, 10.5e5),
        (313.15, 0.40, 1.0e5),
    ] {
        let phi = applied_phi(&s, t, alpha, p);
        assert!(
            (phi - PHI_CO2_ABSORBER).abs() < 1e-12,
            "{t} K {p} Pa: {phi}"
        );
    }
}

/// Concept after OUTRAM PARK, reference only: repeated solves share no stale state.
/// A 30 mass% p* and water activity read after a 40 mass% direct solve equal the
/// same reads made fresh on a new thread (the tables are thread-local), bit for bit.
#[test]
fn eq_g_33_solvent_reads_share_no_stale_state() {
    let reads = |s: &Solvent| -> [u64; 4] {
        [
            s.pstar_pa(331.7, 0.437).unwrap().to_bits(),
            s.pstar_pa(313.15, 0.25).unwrap().to_bits(),
            s.a_water(331.7, 0.437).unwrap().to_bits(),
            s.pstar_gas_pa(313.15, 0.50, P_ABS_PA).unwrap().to_bits(),
        ]
    };
    let strong = Solvent::new(0.40, 323.15).unwrap();
    let _ = reads(&strong);
    let after = reads(&Solvent::new(W_MEA, 323.15).unwrap());
    let fresh = std::thread::spawn(move || reads(&Solvent::new(W_MEA, 323.15).unwrap()))
        .join()
        .unwrap();
    assert_eq!(after, fresh);
}

#[test]
fn eq_g_21_binary_is_a_vapour_at_low_pressure() {
    // The Aronu comparison regime keeps a vapour root.
    for p in [1.0e5, 2.0e5, 10.5e5] {
        let st = srk_co2_h2o(313.15, p, 0.05).unwrap();
        assert!(st.z > 0.7, "{p} Pa: Z {}", st.z);
        assert!((0.9..=1.0).contains(&st.phi_co2), "{p} Pa: {}", st.phi_co2);
    }
}
