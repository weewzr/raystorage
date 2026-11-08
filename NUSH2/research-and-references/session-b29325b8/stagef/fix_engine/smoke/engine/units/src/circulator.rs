//! Helium circulators and loop hydraulics on real flows: the helium flows that a
//! duty needs, adiabatic compression across a pressure rise, the motor draw, the
//! temperature rise the shaft work leaves in the gas, the duct loss that follows
//! pipe diameter and a smooth-wall friction drop.
//!
//! Every function takes plain numbers, so the plant can feed it the flows of any
//! preset. No measured circulator shaft power exists in the library; the forms
//! are the cards named on each function.
//!
//! Class ReducedOrder: adiabatic compression on an
//! assumed pressure-drop budget, with no measured shaft power to close it.

use nush2_numerics::{EngineError, Result};

/// Exponent of card B.10, (kappa - 1) / kappa for helium, as the card prints it.
pub const COMPRESSION_EXPONENT: f64 = 0.4;
/// Reynolds range of the Blasius form, card C.16 (smooth tube, turbulent).
pub const BLASIUS_RE_MIN: f64 = 4.0e3;
pub const BLASIUS_RE_MAX: f64 = 1.0e5;

fn positive(param: &str, v: f64) -> Result<f64> {
    if v > 0.0 && v.is_finite() {
        Ok(v)
    } else {
        Err(EngineError::Range {
            param: param.into(),
            msg: format!("{v} must be positive and finite"),
        })
    }
}

/// Secondary helium flow that carries `q_w` between supply and return,
/// `m_s = Q / (c_p (T_sup - T_ret))`, kg/s.
///
/// eq: B.9
pub fn secondary_flow_kg_s(q_w: f64, cp: f64, t_sup_k: f64, t_ret_k: f64) -> Result<f64> {
    positive("q_w", q_w)?;
    positive("cp", cp)?;
    positive("t_sup - t_ret", t_sup_k - t_ret_k)?;
    Ok(q_w / (cp * (t_sup_k - t_ret_k)))
}

/// Primary helium flow for reactor heat `q_r_w` when the primary returns at the
/// secondary return plus the IHX approach,
/// `m_p = Q_R / (c_p (T_ROT - T_s,ret - dT_app))`, kg/s.
///
/// eq: B.9
pub fn primary_flow_kg_s(
    q_r_w: f64,
    cp: f64,
    rot_k: f64,
    t_ret_k: f64,
    approach_k: f64,
) -> Result<f64> {
    positive("q_r_w", q_r_w)?;
    positive("cp", cp)?;
    positive("rot - t_ret - approach", rot_k - t_ret_k - approach_k)?;
    Ok(q_r_w / (cp * (rot_k - t_ret_k - approach_k)))
}

/// Shaft power of one adiabatic circulator, MW, from flow `md` kg/s, inlet
/// temperature `t_k`, loop pressure `p_mpa`, rise `dp_kpa` and isentropic
/// efficiency: `W = m c_p T_1 [(1 + dp/p)^0.4 - 1] / eta`. The arithmetic order
/// is the anchored port's, so the anchored goldens hold.
///
/// eq: B.10
pub fn shaft(md: f64, cp: f64, t_k: f64, p_mpa: f64, dp_kpa: f64, eta: f64) -> f64 {
    md * cp * t_k * ((1.0 + dp_kpa / 1e3 / p_mpa).powf(COMPRESSION_EXPONENT) - 1.0) / eta / 1e6
}

/// Electric draw of both circulators, motors outside the gas,
/// `E = (W_s + W_p) / eta_m`.
///
/// eq: B.12
pub fn electric(w_s_mw: f64, w_p_mw: f64, eta_motor: f64) -> f64 {
    (w_s_mw + w_p_mw) / eta_motor
}

/// Temperature rise the shaft work leaves in the gas, `dT = W / (m c_p)`, K.
///
/// eq: B.6
pub fn rise(w_mw: f64, m_kg_s: f64, cp: f64) -> Result<f64> {
    positive("m_kg_s", m_kg_s)?;
    positive("cp", cp)?;
    Ok(w_mw * 1.0e6 / (m_kg_s * cp))
}

/// Duct loss that follows pipe diameter, `Q = Q_ref (m_s / m_ref)^{1/2}`, MW.
/// Card B.14 takes 0.45 MW at the INL 78.49 kg/s, 90 m duct.
///
/// eq: B.14
pub fn duct_loss(m_s_kg_s: f64, m_ref_kg_s: f64, q_ref_mw: f64) -> Result<f64> {
    positive("m_s_kg_s", m_s_kg_s)?;
    positive("m_ref_kg_s", m_ref_kg_s)?;
    Ok(q_ref_mw * (m_s_kg_s / m_ref_kg_s).sqrt())
}

/// Friction drop on a smooth passage, Darcy form with the Blasius factor,
/// `dp = f_D (L / D_h) rho u^2 / 2`, `f_D = 0.3164 Re^{-1/4}`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Friction {
    pub dp_pa: f64,
    pub re: f64,
    pub f_darcy: f64,
    pub u_m_s: f64,
    /// False when `re` lies outside the Blasius range, so the drop is an
    /// extrapolation.
    pub in_range: bool,
}

/// Drop across one passage of flow area `area_m2` and hydraulic diameter
/// `d_h_m`, length `length_m`, carrying `m_kg_s` at density `rho` and
/// viscosity `mu`.
///
/// eq: C.16
/// eq: A.11
pub fn dp_blasius(
    m_kg_s: f64,
    area_m2: f64,
    d_h_m: f64,
    length_m: f64,
    rho: f64,
    mu: f64,
) -> Result<Friction> {
    positive("m_kg_s", m_kg_s)?;
    positive("area_m2", area_m2)?;
    positive("d_h_m", d_h_m)?;
    positive("length_m", length_m)?;
    positive("rho", rho)?;
    positive("mu", mu)?;
    let u = m_kg_s / (rho * area_m2);
    let re = rho * u * d_h_m / mu;
    let f = 0.3164 * re.powf(-0.25);
    Ok(Friction {
        dp_pa: f * (length_m / d_h_m) * rho * u * u / 2.0,
        re,
        f_darcy: f,
        u_m_s: u,
        in_range: (BLASIUS_RE_MIN..=BLASIUS_RE_MAX).contains(&re),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_b_09_flows_carry_the_duty() {
        // INL Case 6: 176.8 MW across 900 to 466 C at c_p 5190 gives 78.49 kg/s.
        let m = secondary_flow_kg_s(176.8e6, 5190.0, 1173.15, 739.15).unwrap();
        assert!((m - 78.49).abs() < 0.01, "{m}");
        let mp = primary_flow_kg_s(179.0e6, 5195.0, 1198.15, 739.15, 25.0).unwrap();
        assert!((mp * 5195.0 * 434.0 - 179.0e6).abs() < 1.0);
        assert!(secondary_flow_kg_s(1.0, 5195.0, 700.0, 800.0).is_err());
    }

    #[test]
    fn eq_b_10_zero_rise_needs_no_work() {
        assert_eq!(shaft(50.0, 5195.0, 739.0, 7.0, 0.0, 0.75), 0.0);
        let a = shaft(50.0, 5195.0, 739.0, 7.0, 200.0, 0.75);
        let b = shaft(50.0, 5195.0, 739.0, 7.0, 400.0, 0.75);
        assert!(a > 0.0 && b > 1.9 * a && b < 2.0 * a);
    }

    #[test]
    fn eq_b_06_rise_inverts_the_work() {
        let dt = rise(0.222, 2.519, 5195.0).unwrap();
        assert!((dt - 16.96).abs() < 0.05, "{dt}");
    }

    #[test]
    fn eq_b_14_duct_loss_is_the_reference_at_the_reference_flow() {
        assert!((duct_loss(78.49, 78.49, 0.45).unwrap() - 0.45).abs() < 1e-15);
        let half = duct_loss(78.49 / 4.0, 78.49, 0.45).unwrap();
        assert!((half - 0.225).abs() < 1e-12);
    }

    #[test]
    fn eq_c_16_blasius_matches_its_closed_form() {
        // Water-like numbers chosen to land inside the Blasius range.
        let r = dp_blasius(0.5, 1.0e-3, 0.03, 10.0, 1000.0, 1.0e-3).unwrap();
        assert!((r.u_m_s - 0.5).abs() < 1e-12);
        assert!((r.re - 15000.0).abs() < 1e-6);
        let f = 0.3164 / 15000f64.powf(0.25);
        let want = f * (10.0 / 0.03) * 1000.0 * 0.25 / 2.0;
        assert!((r.dp_pa / want - 1.0).abs() < 1e-12);
        assert!(r.in_range);
        assert!(dp_blasius(0.0, 1.0, 1.0, 1.0, 1.0, 1.0).is_err());
    }
}
