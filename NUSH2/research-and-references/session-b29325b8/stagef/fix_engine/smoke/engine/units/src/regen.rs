//! REGEN: rich/lean exchanger, lean cooler, lean pump, and the reboiler duty
//! terms, class ReducedOrder. Solvent heat capacity is the assumed
//! 3.6 kJ/kg/K of `solvent`, not verified against a table (derivation note M21).

use crate::solvent::CP_SOLVENT_J_KG_K;
use nush2_numerics::{EngineError, Result};
use nush2_props::steam::{hfg_j_kg, tsat_k};

/// Approved SRD, GJ/t, and its range (decision D26).
pub const SRD_CARRIED_GJ_T: f64 = 3.8;
pub const SRD_LOW_GJ_T: f64 = 3.4;
pub const SRD_HIGH_GJ_T: f64 = 4.2;

fn positive(name: &str, v: f64) -> Result<f64> {
    if v.is_finite() && v > 0.0 {
        Ok(v)
    } else {
        Err(EngineError::Range {
            param: name.into(),
            msg: format!("{v} must be positive and finite"),
        })
    }
}

/// Exchanger result. Temperatures in K, duty in W.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RichLean {
    pub t_rich_out_k: f64,
    pub t_lean_out_k: f64,
    pub duty_w: f64,
    /// True when the approach sits at the cold end.
    pub cold_end_pinch: bool,
}

/// Counter-current rich/lean exchanger with equal heat capacity per kg on both
/// sides and a minimum approach `approach_k` at whichever end pinches:
/// m_r c_p (T_r,out - T_r,in) = m_l c_p (T_l,in - T_l,out).
///
/// eq: I.10
pub fn rich_lean(
    m_rich_kg_s: f64,
    t_rich_in_k: f64,
    m_lean_kg_s: f64,
    t_lean_in_k: f64,
    approach_k: f64,
) -> Result<RichLean> {
    positive("m_rich_kg_s", m_rich_kg_s)?;
    positive("m_lean_kg_s", m_lean_kg_s)?;
    let span = t_lean_in_k - t_rich_in_k - approach_k;
    positive("hot-side margin", span)?;
    let c_r = m_rich_kg_s * CP_SOLVENT_J_KG_K;
    let c_l = m_lean_kg_s * CP_SOLVENT_J_KG_K;
    let q_cold = c_l * span;
    let q_hot = c_r * span;
    let (duty, cold) = if q_cold <= q_hot {
        (q_cold, true)
    } else {
        (q_hot, false)
    };
    Ok(RichLean {
        t_rich_out_k: t_rich_in_k + duty / c_r,
        t_lean_out_k: t_lean_in_k - duty / c_l,
        duty_w: duty,
        cold_end_pinch: cold,
    })
}

/// Rich/lean exchanger at a given duty, W, as a design sheet prints it.
pub fn rich_lean_at_duty(
    m_rich_kg_s: f64,
    t_rich_in_k: f64,
    m_lean_kg_s: f64,
    t_lean_in_k: f64,
    duty_w: f64,
) -> Result<RichLean> {
    positive("m_rich_kg_s", m_rich_kg_s)?;
    positive("m_lean_kg_s", m_lean_kg_s)?;
    let t_r = t_rich_in_k + duty_w / (m_rich_kg_s * CP_SOLVENT_J_KG_K);
    let t_l = t_lean_in_k - duty_w / (m_lean_kg_s * CP_SOLVENT_J_KG_K);
    Ok(RichLean {
        t_rich_out_k: t_r,
        t_lean_out_k: t_l,
        duty_w,
        cold_end_pinch: (t_l - t_rich_in_k) <= (t_lean_in_k - t_r),
    })
}

/// Saturated steam condensed in the reboiler, kg/s, at steam pressure `p_steam_pa`.
///
/// eq: I.11
pub fn steam(q_reb_w: f64, p_steam_pa: f64) -> Result<f64> {
    Ok(q_reb_w / hfg_j_kg(tsat_k(p_steam_pa)?)?)
}

/// Specific reboiler duty, J/kg CO2.
///
/// eq: I.12
pub fn srd(q_reb_w: f64, m_co2_kg_s: f64) -> Result<f64> {
    Ok(q_reb_w / positive("m_co2_kg_s", m_co2_kg_s)?)
}

/// SRD split into desorption, sensible and stripping-steam parts, J/kg CO2.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SrdSplit {
    pub q_des: f64,
    pub q_sens: f64,
    pub q_strip: f64,
}

/// SRD = q_des + q_sens + q_strip. `q_des` is the heat of desorption per kg CO2 and
/// `q_sens` the heat to raise the rich feed to the reboiler temperature; the
/// remainder is stripping steam.
///
/// eq: I.13
pub fn srd_split(srd_j_kg: f64, q_des_j_kg: f64, q_sens_j_kg: f64) -> SrdSplit {
    SrdSplit {
        q_des: q_des_j_kg,
        q_sens: q_sens_j_kg,
        q_strip: srd_j_kg - q_des_j_kg - q_sens_j_kg,
    }
}

/// Carried SRD and its range, GJ/t: 3.8, 3.4 and 4.2.
///
/// eq: I.14
pub fn srd_input() -> (f64, f64, f64) {
    (SRD_CARRIED_GJ_T, SRD_LOW_GJ_T, SRD_HIGH_GJ_T)
}

/// Stripping intensity, J per kg of rich solvent (Vinjarapu Eqs 16, 17 give it as
/// RD 3600 / m_rich in kJ/kg with kW and kg/h).
///
/// eq: I.15
pub fn intensity(q_reb_w: f64, m_rich_kg_s: f64) -> Result<f64> {
    Ok(q_reb_w / positive("m_rich_kg_s", m_rich_kg_s)?)
}

/// Reboiler area, m2, from RD = U A dT_LMTD.
///
/// eq: I.16
pub fn reboiler_area(q_reb_w: f64, u_w_m2_k: f64, lmtd_k: f64) -> Result<f64> {
    Ok(q_reb_w / positive("u", u_w_m2_k)? / positive("lmtd", lmtd_k)?)
}

/// Reboiler duty, W, from the captured CO2 and an SRD in J/kg.
///
/// eq: J.16
pub fn q_reb(m_co2_kg_s: f64, srd_j_kg: f64) -> f64 {
    m_co2_kg_s * srd_j_kg
}

/// Electric power of the lean pump, W: volumetric flow times the pressure rise over
/// pump and motor efficiencies.
pub fn lean_pump_w(
    m_kg_s: f64,
    rho_kg_m3: f64,
    dp_pa: f64,
    eta_pump: f64,
    eta_motor: f64,
) -> Result<f64> {
    positive("rho", rho_kg_m3)?;
    positive("eta_pump", eta_pump)?;
    positive("eta_motor", eta_motor)?;
    Ok(m_kg_s / rho_kg_m3 * dp_pa / eta_pump / eta_motor)
}

/// Lean cooler duty, W, from the exchanger outlet to the absorber inlet.
pub fn lean_cooler_w(m_lean_kg_s: f64, t_in_k: f64, t_out_k: f64) -> f64 {
    m_lean_kg_s * CP_SOLVENT_J_KG_K * (t_in_k - t_out_k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_i_10_exchanger_pinches_at_the_cold_end_when_rich_is_larger() {
        let x = rich_lean(110.0, 330.0, 100.0, 395.0, 10.0).unwrap();
        assert!(x.cold_end_pinch);
        assert!((x.t_lean_out_k - 340.0).abs() < 1e-9);
        let back = 110.0 * (x.t_rich_out_k - 330.0) - 100.0 * (395.0 - x.t_lean_out_k);
        assert!(back.abs() < 1e-9);
        assert!(395.0 - x.t_rich_out_k > 10.0);
    }

    #[test]
    fn eq_i_12_srd_and_eq_j_16_duty_are_inverse() {
        let s = srd(q_reb(10.0, 3.8e6), 10.0).unwrap();
        assert!((s - 3.8e6).abs() < 1e-6);
    }

    #[test]
    fn eq_i_11_steam_at_4_bar_needs_its_latent_heat() {
        let m = steam(2.0e6, 4.0e5).unwrap();
        assert!(m > 0.8 && m < 1.0, "{m}");
    }
}
