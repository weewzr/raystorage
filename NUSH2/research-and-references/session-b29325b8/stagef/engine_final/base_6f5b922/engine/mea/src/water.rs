//! Liquid water properties for the solvent correlations, from IAPWS-IF97 and the
//! IAPWS viscosity release through `seuif97`. The sources for Weiland's correlations
//! use tabulated water values (Dugas PDF p132, Watson 1986) or Aspen's pure component
//! routines (Plaza PDF p295, p297), and neither prints a formula, so no water
//! coefficient is typed here.

use nush2_numerics::{EngineError, Result};
use nush2_props::steam::psat_pa;
use seuif97::{pt, OD, ODV};

const T_MIN_K: f64 = 273.16;
const T_MAX_K: f64 = 443.15;
const ATM_MPA: f64 = 0.101_325;

/// A compressed-liquid state at or just above saturation, in MPa and deg C.
fn liquid_state(t_k: f64) -> Result<(f64, f64)> {
    if !(T_MIN_K..=T_MAX_K).contains(&t_k) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_MIN_K} to {T_MAX_K} K"),
        });
    }
    let p_mpa = (psat_pa(t_k)? * 1.01e-6).max(ATM_MPA);
    Ok((p_mpa, t_k - 273.15))
}

/// Liquid water density, kg/m3.
pub fn density_kg_m3(t_k: f64) -> Result<f64> {
    let (p, t) = liquid_state(t_k)?;
    Ok(pt(p, t, OD))
}

/// Liquid water dynamic viscosity, Pa s.
pub fn viscosity_pa_s(t_k: f64) -> Result<f64> {
    let (p, t) = liquid_state(t_k)?;
    Ok(pt(p, t, ODV))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_density_agrees_with_hartono_table_a1() {
        // Reference column of Hartono 2014 Table A1, PDF p7.
        for (t_c, rho) in [(20.0, 998.203), (40.0, 992.212), (60.0, 983.191)] {
            let got = density_kg_m3(t_c + 273.15).unwrap();
            assert!((got - rho).abs() < 0.05, "{t_c} C gave {got}");
        }
    }

    #[test]
    fn water_viscosity_is_near_the_known_room_value() {
        let mu = viscosity_pa_s(293.15).unwrap();
        assert!((mu - 1.0e-3).abs() < 2.5e-5, "{mu}");
    }

    #[test]
    fn range_is_enforced() {
        assert!(density_kg_m3(200.0).is_err());
        assert!(viscosity_pa_s(500.0).is_err());
    }
}
