//! IAPWS-IF97 access through the `seuif97` crate.
//!
//! The crate works in deg C, MPa and kJ. This module works in K, Pa and J.
//! The crate signals a bad state with a large negative sentinel, so every value
//! passes through `checked` on its way out.

use nush2_numerics::{EngineError, Result};
use seuif97::{pt, px, tx, OCP, OH, OP, OS, OT, OU, OV, OW};

/// K at 0 deg C.
const KELVIN: f64 = 273.15;
/// Lower limit of IF97 Regions 1 and 2, K.
const T_MIN_K: f64 = 273.15;
/// Upper limit of IF97 Region 2, K. Region 5 is outside this project.
const T_MAX_K: f64 = 1073.15;
/// Upper limit of IF97 Region 1, K.
const T_MAX_LIQUID_K: f64 = 623.15;
/// Upper limit of IF97 Regions 1 to 3, Pa.
const P_MAX_PA: f64 = 100.0e6;
/// Below the triple-point pressure, so vapour is the only fluid state down to
/// 273.15 K and it behaves as an ideal gas. The species layer reads the
/// temperature dependence of H2O vapour here.
pub const P_DILUTE_PA: f64 = 100.0;
/// IF97 Region 2, named explicitly because `seuif97` refuses a region lookup
/// below the triple-point pressure while Eq. (15) itself holds down to p > 0.
const REGION_2: i32 = 2;

fn checked(what: &str, v: f64) -> Result<f64> {
    if v.is_finite() && v > -1000.0 {
        Ok(v)
    } else {
        Err(EngineError::Range {
            param: what.into(),
            msg: format!("IF97 returned the invalid value {v}"),
        })
    }
}

fn check_t(t_k: f64, hi: f64) -> Result<()> {
    if (T_MIN_K..=hi).contains(&t_k) {
        Ok(())
    } else {
        Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_MIN_K} to {hi} K"),
        })
    }
}

/// Saturation pressure in Pa at temperature `t_k`, valid from 273.16 K to 647.096 K.
pub fn psat_pa(t_k: f64) -> Result<f64> {
    if !(273.16..=647.096).contains(&t_k) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside 273.16 to 647.096 K"),
        });
    }
    // seuif97 works in deg C and MPa. Quality 0 selects the saturated liquid line.
    Ok(tx(t_k - KELVIN, 0.0, OP) * 1.0e6)
}

/// Saturation temperature in K at pressure `p_pa`.
pub fn tsat_k(p_pa: f64) -> Result<f64> {
    if !(611.2..=22.064e6).contains(&p_pa) {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} Pa lies outside 611.2 Pa to 22.064 MPa"),
        });
    }
    Ok(px(p_pa / 1.0e6, 0.0, OT) + KELVIN)
}

/// Single-phase state at (T, p). All values are per kg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub v_m3_kg: f64,
    pub h_j_kg: f64,
    pub u_j_kg: f64,
    pub s_j_kg_k: f64,
    pub cp_j_kg_k: f64,
    pub w_m_s: f64,
}

/// Properties at (T, p), with the IF97 region chosen by the crate. Covers 273.15
/// to 1073.15 K and the triple-point pressure to 100 MPa.
pub fn state(t_k: f64, p_pa: f64) -> Result<State> {
    check_t(t_k, T_MAX_K)?;
    if !(p_pa > 0.0 && p_pa <= P_MAX_PA) {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} Pa lies outside 0 to {P_MAX_PA} Pa"),
        });
    }
    let (p, t) = (p_pa / 1.0e6, t_k - KELVIN);
    Ok(State {
        v_m3_kg: checked("IF97 v", pt(p, t, OV))?,
        h_j_kg: checked("IF97 h", pt(p, t, OH))? * 1.0e3,
        u_j_kg: checked("IF97 u", pt(p, t, OU))? * 1.0e3,
        s_j_kg_k: checked("IF97 s", pt(p, t, OS))? * 1.0e3,
        cp_j_kg_k: checked("IF97 cp", pt(p, t, OCP))? * 1.0e3,
        w_m_s: checked("IF97 w", pt(p, t, OW))?,
    })
}

/// Specific enthalpy of liquid water at (T, p), J/kg. A pressure below
/// saturation at `t_k` is an error, since the water would be vapour.
pub fn h_liquid_j_kg(t_k: f64, p_pa: f64) -> Result<f64> {
    check_t(t_k, T_MAX_LIQUID_K)?;
    let ps = psat_pa(t_k)?;
    if p_pa < ps {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} Pa lies below saturation {ps} Pa at {t_k} K"),
        });
    }
    Ok(state(t_k, p_pa)?.h_j_kg)
}

/// Latent heat on the saturation line at `t_k`, J/kg.
pub fn hfg_j_kg(t_k: f64) -> Result<f64> {
    psat_pa(t_k)?;
    let t = t_k - KELVIN;
    let hg = checked("IF97 saturated h", tx(t, 1.0, OH))?;
    let hf = checked("IF97 saturated h", tx(t, 0.0, OH))?;
    Ok((hg - hf) * 1.0e3)
}

/// Enthalpy of vapour at `P_DILUTE_PA`, J/kg, on the IF97 datum.
pub fn h_dilute_j_kg(t_k: f64) -> Result<f64> {
    check_t(t_k, T_MAX_K)?;
    let v = pt(P_DILUTE_PA / 1.0e6, t_k - KELVIN, (OH, REGION_2));
    Ok(checked("IF97 dilute h", v)? * 1.0e3)
}

/// Entropy of vapour at `P_DILUTE_PA`, J/kg/K, on the IF97 datum.
pub fn s_dilute_j_kg_k(t_k: f64) -> Result<f64> {
    check_t(t_k, T_MAX_K)?;
    let v = pt(P_DILUTE_PA / 1.0e6, t_k - KELVIN, (OS, REGION_2));
    Ok(checked("IF97 dilute s", v)? * 1.0e3)
}

/// Heat capacity of vapour at `P_DILUTE_PA`, J/kg/K.
pub fn cp_dilute_j_kg_k(t_k: f64) -> Result<f64> {
    check_t(t_k, T_MAX_K)?;
    let v = pt(P_DILUTE_PA / 1.0e6, t_k - KELVIN, (OCP, REGION_2));
    Ok(checked("IF97 dilute cp", v)? * 1.0e3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boiling_point_at_one_atmosphere() {
        // IF97 gives 373.124 K at 101.325 kPa.
        let t = tsat_k(101_325.0).unwrap();
        assert!((t - 373.124).abs() < 0.01, "got {t}");
    }

    #[test]
    fn saturation_round_trip() {
        let p = psat_pa(450.0).unwrap();
        let t = tsat_k(p).unwrap();
        assert!((t - 450.0).abs() < 1e-3, "got {t}");
    }

    #[test]
    fn out_of_range_is_an_error() {
        assert!(psat_pa(100.0).is_err());
        assert!(tsat_k(1.0).is_err());
        assert!(state(200.0, 1.0e5).is_err());
        assert!(state(500.0, 0.0).is_err());
        assert!(state(f64::NAN, 1.0e5).is_err());
        assert!(h_dilute_j_kg(1200.0).is_err());
    }

    #[test]
    fn liquid_enthalpy_refuses_vapour_states() {
        assert!(h_liquid_j_kg(400.0, 1.0e5).is_err());
        assert!(h_liquid_j_kg(400.0, 3.0e5).is_ok());
        assert!(h_liquid_j_kg(700.0, 30.0e6).is_err());
    }
}
