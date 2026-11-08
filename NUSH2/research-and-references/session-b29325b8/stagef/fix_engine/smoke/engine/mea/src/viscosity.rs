//! Solvent viscosity by the Weiland et al. (1998) correlation, as printed in Dugas
//! 2009 Eq 5.16 with Table 5.1 (PDF p132), Plaza 2012 Eq 3.18 (PDF p73, coefficients
//! to p74) and Khan 2011 Eq 7 (PDF p3). The three printings agree. Water viscosity
//! comes from the IAPWS release through `water`. Hartono stays a benchmark only.

use crate::density;
use crate::water;
use nush2_numerics::Result;

/// Table 5.1 of Dugas. a and b are printed as zero, so those terms drop out.
const C: f64 = 21.186;
const D: f64 = 2373.0;
const E: f64 = 0.01015;
const F: f64 = 0.0093;
const G: f64 = -2.2589;

/// Solution viscosity over water viscosity, Khan Eq 7. `w_mea` is a mass fraction
/// on a CO2-free basis here, and the correlation takes it as mass percent.
pub fn ratio_to_water(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    density::check(t_k, w_mea, alpha)?;
    let om = 100.0 * w_mea;
    Ok(((C * om + D) * (alpha * (E * om + F * t_k + G) + 1.0) * om / (t_k * t_k)).exp())
}

/// Solution viscosity, Pa s.
///
/// eq: G.27
pub fn viscosity_pa_s(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    Ok(water::viscosity_pa_s(t_k)? * ratio_to_water(t_k, w_mea, alpha)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_g_27_pure_water_limit() {
        assert!((ratio_to_water(320.0, 0.0, 0.0).unwrap() - 1.0).abs() < 1e-15);
    }

    #[test]
    fn range_is_enforced() {
        assert!(viscosity_pa_s(500.0, 0.3, 0.2).is_err());
    }
}
