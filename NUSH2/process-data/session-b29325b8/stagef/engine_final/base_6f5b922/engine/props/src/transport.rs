//! Viscosity and thermal conductivity of wet syngas near 40 bar.
//!
//! Class: Empirical. A two-point fit through maker design values for the product
//! gas at S/C 3.5, printed in JAERI-Tech 2005-031:
//!
//! | Appendix | Page | Mean T | Viscosity | Conductivity |
//! |---|---|---|---|---|
//! | F-2 | PDF p185, printed p171 | 359 C | 23.550 uPa s | 0.12409 W/m/K |
//! | G-1 | PDF p187, printed p173 | 483 C | 27.850 uPa s | 0.14990 W/m/K |
//!
//! The fit is a power law in absolute temperature through both points, the
//! usual form for a gas. Two points cannot test that form, and they fix a single
//! composition, so the result carries no composition or pressure dependence.
//! Any temperature outside 359 C to 483 C returns a flag beside the value.
//! Helium transport lives in `helium`.

use nush2_numerics::{EngineError, Result};

/// Lower fit point, K.
pub const T_LOW_K: f64 = 359.0 + 273.15;
/// Upper fit point, K.
pub const T_HIGH_K: f64 = 483.0 + 273.15;
const MU_LOW: f64 = 23.550e-6;
const MU_HIGH: f64 = 27.850e-6;
const K_LOW: f64 = 0.12409;
const K_HIGH: f64 = 0.14990;
/// Values beyond these limits are refused, K.
const T_REFUSED_BELOW_K: f64 = 273.15;
const T_REFUSED_ABOVE_K: f64 = 1300.0;

/// Model class of this fit, in the terms of the engine plan.
pub const MODEL_CLASS: &str = "Empirical";

/// Where a temperature sits against the fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitFlag {
    Inside,
    BelowFit,
    AboveFit,
}

/// Syngas transport values at one temperature.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyngasTransport {
    pub mu_pa_s: f64,
    pub k_w_m_k: f64,
    pub flag: FitFlag,
}

fn power_law(t_k: f64, low: f64, high: f64) -> f64 {
    let n = (high / low).ln() / (T_HIGH_K / T_LOW_K).ln();
    low * (t_k / T_LOW_K).powf(n)
}

/// Syngas viscosity and conductivity at `t_k`.
pub fn syngas(t_k: f64) -> Result<SyngasTransport> {
    if !(T_REFUSED_BELOW_K..=T_REFUSED_ABOVE_K).contains(&t_k) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_REFUSED_BELOW_K} to {T_REFUSED_ABOVE_K} K"),
        });
    }
    let flag = if t_k > T_HIGH_K {
        FitFlag::AboveFit
    } else if t_k < T_LOW_K {
        FitFlag::BelowFit
    } else {
        FitFlag::Inside
    };
    Ok(SyngasTransport {
        mu_pa_s: power_law(t_k, MU_LOW, MU_HIGH),
        k_w_m_k: power_law(t_k, K_LOW, K_HIGH),
        flag,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_returns_the_printed_values_at_both_points() {
        let a = syngas(T_LOW_K).unwrap();
        let b = syngas(T_HIGH_K).unwrap();
        assert!((a.mu_pa_s / MU_LOW - 1.0).abs() < 1e-12);
        assert!((b.mu_pa_s / MU_HIGH - 1.0).abs() < 1e-12);
        assert!((a.k_w_m_k / K_LOW - 1.0).abs() < 1e-12);
        assert!((b.k_w_m_k / K_HIGH - 1.0).abs() < 1e-12);
    }

    #[test]
    fn printed_prandtl_numbers_follow_from_the_readings() {
        // F-2 and G-1 print Cp 2618.8 and 2692.0 J/kg/K, and Pr 0.49700 and 0.50015.
        assert!((2618.8 * MU_LOW / K_LOW - 0.49700).abs() < 5e-5);
        assert!((2692.0 * MU_HIGH / K_HIGH - 0.50015).abs() < 5e-5);
    }
}
