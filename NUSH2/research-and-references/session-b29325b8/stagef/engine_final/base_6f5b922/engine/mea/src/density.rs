//! Solvent density by the Weiland et al. (1998) correlation, as printed in Plaza
//! 2012 Eqs 3.16 and 3.17 (PDF p71) and Dugas 2009 Eqs 5.17 to 5.20 with Table 5.2
//! (PDF p133). The Hartono 2014 correlation stays a benchmark only, because its
//! unloaded base is not reconstructible from that paper.
//!
//! V_CO2 = 0.04747 mL/mol is the value both theses print. Reading it as 47.47
//! raises AARD against Hartono Table 3 from 0.51% to 4.84% (`mea_thermo.md` s9).

use crate::water;
use nush2_numerics::{EngineError, Result};

/// Quadratic coefficients of the MEA liquid density in Eq 3.17, g/mL with T in K.
const A: f64 = -5.35162e-7;
const B: f64 = -4.51417e-4;
const C: f64 = 1.19451;
/// Water and MEA interaction volume, mL/mol.
const V_STAR: f64 = -1.8218;
/// Dissolved CO2 volume, mL/mol.
const V_CO2: f64 = 0.04747;
/// g/mol. Plaza's subroutine uses 61.08 (PDF p295), while Eq 3.17 and Dugas Table
/// 5.2 print 61.09. The prototype and this crate use 61.08, kept beside
/// `nush2_props::units::MW_MEA` so the density port is unchanged.
const MW_MEA: f64 = 61.08;
/// g/mol, the prototype's value, kept beside `nush2_props::units::MW_H2O`: the same
/// number.
const MW_WATER: f64 = 18.015;
/// g/mol, the prototype's value, kept beside `nush2_props::units::MW_CO2` (44.0095)
/// so the density port is unchanged; the gap is 1.1e-5 relative.
const MW_CO2: f64 = 44.01;

/// Temperature window, K. Checked against Hartono at 20 to 80 C only, so the
/// stripper range above 80 C is extrapolation.
pub const T_MIN_K: f64 = 293.15;
pub const T_MAX_K: f64 = 403.15;

pub(crate) fn check(t_k: f64, w_mea: f64, alpha: f64) -> Result<()> {
    if !(T_MIN_K..=T_MAX_K).contains(&t_k) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_MIN_K} to {T_MAX_K} K"),
        });
    }
    if !(0.0..=0.60).contains(&w_mea) {
        return Err(EngineError::Range {
            param: "w_mea".into(),
            msg: format!("{w_mea} lies outside 0 to 0.60"),
        });
    }
    if !(0.0..=1.0).contains(&alpha) {
        return Err(EngineError::Range {
            param: "alpha".into(),
            msg: format!("{alpha} lies outside 0 to 1"),
        });
    }
    Ok(())
}

/// Density of loaded solvent, kg/m3. `w_mea` is the MEA mass fraction on a CO2-free
/// basis and `alpha` is mol CO2 per mol MEA.
pub fn density_kg_m3(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    check(t_k, w_mea, alpha)?;
    let v_mea = MW_MEA / (A * t_k * t_k + B * t_k + C);
    let v_water = MW_WATER / (water::density_kg_m3(t_k)? / 1000.0);
    let n_mea = w_mea / MW_MEA;
    let n_water = (1.0 - w_mea) / MW_WATER;
    let n_co2 = n_mea * alpha;
    let total = n_mea + n_water + n_co2;
    let (x_mea, x_water, x_co2) = (n_mea / total, n_water / total, n_co2 / total);
    let v = x_mea * v_mea + x_water * v_water + x_co2 * V_CO2 + x_mea * x_water * V_STAR;
    let m = x_mea * MW_MEA + x_water * MW_WATER + x_co2 * MW_CO2;
    Ok(m / v * 1000.0)
}

/// Total MEA concentration, kmol/m3, from the same density.
pub fn c_mea_kmol_m3(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    let rho = density_kg_m3(t_k, w_mea, alpha)?;
    let kg_co2_free_per_kg_solution = 1.0 / (1.0 + w_mea / MW_MEA * alpha * MW_CO2);
    Ok(rho * kg_co2_free_per_kg_solution * w_mea / MW_MEA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thirty_percent_solvent_is_about_five_molar() {
        // Dugas Table 4.1, PDF p91: 7 m = 5.0 M = 30 wt%.
        let c = c_mea_kmol_m3(298.15, 0.30, 0.0).unwrap();
        assert!((c - 4.96).abs() < 0.1, "{c}");
    }

    #[test]
    fn range_is_enforced() {
        assert!(density_kg_m3(250.0, 0.3, 0.2).is_err());
        assert!(density_kg_m3(320.0, 0.8, 0.2).is_err());
        assert!(density_kg_m3(320.0, 0.3, 1.2).is_err());
    }
}
