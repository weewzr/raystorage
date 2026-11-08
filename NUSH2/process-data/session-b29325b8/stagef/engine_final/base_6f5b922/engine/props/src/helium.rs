//! Helium properties from H. Petersen, Riso Report No. 224, 1970. The report
//! covers 1 to 100 bar and 273 to 1800 K, and gives these equations on printed
//! pp 6 and 7 (PDF pp 10 and 11). Inside the correlations P is in bar and T in K.
//!
//! Class: Empirical. The report sits outside the project library.
//!
//! Stated standard deviations: density 0.03 sqrt(P) %, heat capacity at most
//! 0.5 %, viscosity 0.0015 T %, conductivity 0.0035 T %, Prandtl number 0.004 T %.
//! At 1173 K the transport values therefore carry about 1.8 % and 4.1 %.

use crate::units::BAR_PA;
use nush2_numerics::{EngineError, Result};

/// Eq. 4-4, J/kg/K.
pub const CP_J_KG_K: f64 = 5195.0;
/// Eq. 4-5, J/kg/K.
pub const CV_J_KG_K: f64 = 3117.0;
/// Eq. 4-6.
pub const GAMMA: f64 = 1.6667;

const T_MIN_K: f64 = 273.0;
const T_MAX_K: f64 = 1800.0;
const P_MIN_BAR: f64 = 1.0;
const P_MAX_BAR: f64 = 100.0;

fn check_t(t_k: f64) -> Result<()> {
    if (T_MIN_K..=T_MAX_K).contains(&t_k) {
        Ok(())
    } else {
        Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside the Petersen range {T_MIN_K} to {T_MAX_K} K"),
        })
    }
}

fn bar(p_pa: f64) -> Result<f64> {
    let p = p_pa / BAR_PA;
    if (P_MIN_BAR..=P_MAX_BAR).contains(&p) {
        Ok(p)
    } else {
        Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p} bar lies outside the Petersen range {P_MIN_BAR} to {P_MAX_BAR} bar"),
        })
    }
}

/// Compressibility factor, Eq. 3-18.
pub fn compressibility(t_k: f64, p_pa: f64) -> Result<f64> {
    check_t(t_k)?;
    Ok(1.0 + 0.4446 * bar(p_pa)? / t_k.powf(1.2))
}

/// Density, kg/m3, Eq. 3-19.
pub fn density_kg_m3(t_k: f64, p_pa: f64) -> Result<f64> {
    check_t(t_k)?;
    let p = bar(p_pa)?;
    Ok(48.14 * p / t_k / (1.0 + 0.4446 * p / t_k.powf(1.2)))
}

/// Dynamic viscosity, Pa s, Eq. 6-1. The report takes it as independent of
/// pressure over its whole range.
pub fn viscosity_pa_s(t_k: f64) -> Result<f64> {
    check_t(t_k)?;
    Ok(3.674e-7 * t_k.powf(0.7))
}

/// Thermal conductivity, W/m/K, Eq. 7-1.
pub fn conductivity_w_m_k(t_k: f64, p_pa: f64) -> Result<f64> {
    check_t(t_k)?;
    let p = bar(p_pa)?;
    Ok(2.682e-3 * (1.0 + 1.123e-3 * p) * t_k.powf(0.71 * (1.0 - 2.0e-4 * p)))
}

/// Prandtl number, Eq. 8-1.
pub fn prandtl(t_k: f64, p_pa: f64) -> Result<f64> {
    check_t(t_k)?;
    let p = bar(p_pa)?;
    Ok(0.7117 / (1.0 + 1.123e-3 * p) * t_k.powf(-(0.01 - 1.42e-4 * p)))
}

/// Heat picked up or given up by a helium flow between two temperatures, W.
/// Positive when `t_hot_k` is above `t_cold_k`.
pub fn duty_w(m_kg_s: f64, t_hot_k: f64, t_cold_k: f64) -> Result<f64> {
    check_t(t_hot_k)?;
    check_t(t_cold_k)?;
    if !(m_kg_s >= 0.0 && m_kg_s.is_finite()) {
        return Err(EngineError::Range {
            param: "m_kg_s".into(),
            msg: format!("{m_kg_s} kg/s must be finite and not negative"),
        });
    }
    Ok(m_kg_s * CP_J_KG_K * (t_hot_k - t_cold_k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_is_the_ratio_of_the_heat_capacities() {
        assert!((CP_J_KG_K / CV_J_KG_K - GAMMA).abs() < 1e-4);
    }

    #[test]
    fn prandtl_eq_8_1_matches_the_property_ratio() {
        for (t, p) in [(739.15, 51.5e5), (1173.15, 51.5e5), (300.0, 1.0e5)] {
            let ratio = CP_J_KG_K * viscosity_pa_s(t).unwrap() / conductivity_w_m_k(t, p).unwrap();
            let pr = prandtl(t, p).unwrap();
            assert!((pr / ratio - 1.0).abs() < 1e-3, "{t} {p}");
        }
    }

    #[test]
    fn density_law_reduces_to_the_ideal_gas_with_z() {
        let (t, p) = (900.0, 50.0e5);
        let rho = density_kg_m3(t, p).unwrap();
        let z = compressibility(t, p).unwrap();
        // Eq. 3-2 with the specific gas constant the report uses, 2077.3 J/kg/K.
        assert!((rho - p / (2077.3 * t * z)).abs() / rho < 1e-4);
    }

    #[test]
    fn outside_the_range_is_an_error() {
        assert!(density_kg_m3(250.0, 1.0e6).is_err());
        assert!(density_kg_m3(1900.0, 1.0e6).is_err());
        assert!(density_kg_m3(800.0, 0.5e5).is_err());
        assert!(conductivity_w_m_k(800.0, 150.0e5).is_err());
        assert!(viscosity_pa_s(f64::NAN).is_err());
        assert!(duty_w(-1.0, 900.0, 700.0).is_err());
    }
}
