//! Intermediate heat exchanger, helium to helium, counterflow, constant `c_p`.
//!
//! Rating runs on effectiveness and NTU, sizing on the log mean temperature
//! difference. The module count is the UA ratio to the measured HTTR unit, so no
//! single unit of the Case B size is claimed (card C.14). No Nusselt correlation
//! is evaluated, since Tochio prints the shell-side form without its constant
//! (card C.6). Losses are the residual fraction of card B.15.

use nush2_numerics::{EngineError, Result};

/// Measured HTTR IHX conductance at full power, W/K (card C.14, Tochio Figs 8
/// and 10, PDF p7 to p8, read off the figures as about 85 kW/K).
pub const HTTR_UA_W_K: f64 = 85.0e3;
/// Overall coefficients at which the area is printed, W/m2/K (plan section 9).
/// 400 is the order of the HTTR measured sets (card C.5: 428 at 30 MW, 375 at
/// 15 MW); 760 is the plan's upper case.
pub const U_REPORT_W_M2_K: [f64; 2] = [400.0, 760.0];
/// HTTR heat transfer tubes, Tochio Table 2 (PDF p5): 96 tubes, 31.8 mm outer
/// diameter, 3.5 mm wall.
pub const HTTR_TUBES: f64 = 96.0;
pub const HTTR_TUBE_OD_M: f64 = 0.0318;
pub const HTTR_TUBE_WALL_M: f64 = 0.0035;
/// HTTR tube length, the midpoint of the 21.5 to 22.9 m that TECDOC-1236
/// Table 3-4 (PDF p27) prints.
pub const HTTR_TUBE_LENGTH_M: f64 = 22.2;

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

/// Heat given up by the hot stream, `Q_h = m_p c_p (T_h,in - T_h,out)`, W.
///
/// eq: C.1
pub fn hot_side(m_kg_s: f64, cp: f64, t_in_k: f64, t_out_k: f64) -> f64 {
    m_kg_s * cp * (t_in_k - t_out_k)
}

/// Heat taken up by the cold stream, `Q_c = m_s c_p (T_c,out - T_c,in)`, W.
///
/// eq: C.2
pub fn cold_side(m_kg_s: f64, cp: f64, t_in_k: f64, t_out_k: f64) -> f64 {
    m_kg_s * cp * (t_out_k - t_in_k)
}

/// Closure of the exchanger, `Q_h - Q_c - Q_loss`, W. Zero when the loss
/// accounts for the whole imbalance.
///
/// eq: C.3
pub fn closure(q_hot_w: f64, q_cold_w: f64, q_loss_w: f64) -> f64 {
    q_hot_w - q_cold_w - q_loss_w
}

/// Loss charged to the exchanger, `Q_ihx = f_ihx Q`, W.
///
/// eq: B.15
pub fn loss(f_ihx: f64, q_w: f64) -> f64 {
    f_ihx * q_w
}

/// Log mean temperature difference of the two terminal differences, K. Tends to
/// the common difference when they are equal (card C.8).
///
/// eq: C.7
pub fn lmtd(dt1_k: f64, dt2_k: f64) -> Result<f64> {
    positive("dt1_k", dt1_k)?;
    positive("dt2_k", dt2_k)?;
    let r = dt1_k / dt2_k;
    if (r - 1.0).abs() < 1.0e-9 {
        // eq: C.8
        return Ok(0.5 * (dt1_k + dt2_k));
    }
    Ok((dt1_k - dt2_k) / r.ln())
}

/// Effectiveness, `eps = Q / (C_min (T_h,in - T_c,in))`.
///
/// eq: C.9
pub fn effectiveness(q_w: f64, c_min_w_k: f64, t_hot_in_k: f64, t_cold_in_k: f64) -> Result<f64> {
    positive("c_min_w_k", c_min_w_k)?;
    positive("t_hot_in - t_cold_in", t_hot_in_k - t_cold_in_k)?;
    Ok(q_w / (c_min_w_k * (t_hot_in_k - t_cold_in_k)))
}

/// Number of transfer units, `NTU = UA / C_min`.
///
/// eq: C.10
pub fn ntu(ua_w_k: f64, c_min_w_k: f64) -> Result<f64> {
    positive("c_min_w_k", c_min_w_k)?;
    Ok(ua_w_k / c_min_w_k)
}

/// Counterflow effectiveness at `ntu` and capacity ratio `c_star` in [0, 1].
/// The balanced limit `NTU / (1 + NTU)` (card C.12) is taken when `c_star` is
/// within 1e-9 of one.
///
/// eq: C.11
pub fn eps_ntu(ntu: f64, c_star: f64) -> Result<f64> {
    if !(ntu >= 0.0 && ntu.is_finite()) {
        return Err(EngineError::Range {
            param: "ntu".into(),
            msg: format!("{ntu} must be finite and not negative"),
        });
    }
    if !(0.0..=1.0 + 1.0e-12).contains(&c_star) {
        return Err(EngineError::Range {
            param: "c_star".into(),
            msg: format!("{c_star} must lie in [0, 1]"),
        });
    }
    if (1.0 - c_star).abs() < 1.0e-9 {
        // eq: C.12
        return Ok(ntu / (1.0 + ntu));
    }
    let e = (-ntu * (1.0 - c_star)).exp();
    Ok((1.0 - e) / (1.0 - c_star * e))
}

/// Outcome of a rating run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rating {
    pub q_w: f64,
    pub t_hot_out_k: f64,
    pub t_cold_out_k: f64,
    pub eps: f64,
    pub ntu: f64,
    pub c_star: f64,
}

/// Rate a counterflow exchanger of conductance `ua_w_k` from its inlets and the
/// two capacity rates `m c_p`, W/K.
///
/// eq: C.11
pub fn rate(
    t_hot_in_k: f64,
    t_cold_in_k: f64,
    c_hot_w_k: f64,
    c_cold_w_k: f64,
    ua_w_k: f64,
) -> Result<Rating> {
    positive("c_hot_w_k", c_hot_w_k)?;
    positive("c_cold_w_k", c_cold_w_k)?;
    positive("ua_w_k", ua_w_k)?;
    let c_min = c_hot_w_k.min(c_cold_w_k);
    let c_max = c_hot_w_k.max(c_cold_w_k);
    let c_star = c_min / c_max;
    let n = ntu(ua_w_k, c_min)?;
    let eps = eps_ntu(n, c_star)?;
    let q = eps * c_min * (t_hot_in_k - t_cold_in_k);
    Ok(Rating {
        q_w: q,
        t_hot_out_k: t_hot_in_k - q / c_hot_w_k,
        t_cold_out_k: t_cold_in_k + q / c_cold_w_k,
        eps,
        ntu: n,
        c_star,
    })
}

/// Size of an exchanger for a duty and four terminal temperatures.
#[derive(Debug, Clone, PartialEq)]
pub struct Sizing {
    pub q_w: f64,
    pub lmtd_k: f64,
    pub ua_w_k: f64,
    pub ntu: f64,
    pub eps: f64,
    pub c_star: f64,
    /// `(U, A)` pairs, W/m2/K and m2, at the coefficients of `U_REPORT_W_M2_K`.
    pub area_m2: Vec<(f64, f64)>,
    /// UA over the measured HTTR unit; the number of HTTR-size modules.
    pub modules_httr: f64,
}

/// Area for a duty at an overall coefficient, `A = Q / (U dT_lm)`, m2.
///
/// eq: C.13
pub fn area(q_w: f64, u_w_m2_k: f64, lmtd_k: f64) -> Result<f64> {
    positive("u_w_m2_k", u_w_m2_k)?;
    positive("lmtd_k", lmtd_k)?;
    Ok(q_w / (u_w_m2_k * lmtd_k))
}

/// Number of HTTR-size modules that give a conductance, `N = UA / UA_HTTR`.
///
/// eq: C.14
pub fn modules(ua_w_k: f64) -> f64 {
    ua_w_k / HTTR_UA_W_K
}

/// Size a counterflow exchanger. The capacity rates follow from the duty and
/// the two temperature changes.
///
/// eq: C.13
pub fn size(
    q_w: f64,
    t_hot_in_k: f64,
    t_hot_out_k: f64,
    t_cold_in_k: f64,
    t_cold_out_k: f64,
) -> Result<Sizing> {
    positive("q_w", q_w)?;
    let dt_hot_end = t_hot_in_k - t_cold_out_k;
    let dt_cold_end = t_hot_out_k - t_cold_in_k;
    let lm = lmtd(dt_hot_end, dt_cold_end)?;
    let ua = q_w / lm;
    let c_hot = q_w / positive("t_hot_in - t_hot_out", t_hot_in_k - t_hot_out_k)?;
    let c_cold = q_w / positive("t_cold_out - t_cold_in", t_cold_out_k - t_cold_in_k)?;
    let c_min = c_hot.min(c_cold);
    let c_star = c_min / c_hot.max(c_cold);
    let mut areas = Vec::new();
    for u in U_REPORT_W_M2_K {
        areas.push((u, area(q_w, u, lm)?));
    }
    Ok(Sizing {
        q_w,
        lmtd_k: lm,
        ua_w_k: ua,
        ntu: ntu(ua, c_min)?,
        eps: effectiveness(q_w, c_min, t_hot_in_k, t_cold_in_k)?,
        c_star,
        area_m2: areas,
        modules_httr: modules(ua),
    })
}

/// Secondary supply temperature down the cascade,
/// `T_s,sup = T_ROT - dT_app - dT_duct`, K.
///
/// eq: C.17
pub fn cascade(rot_k: f64, approach_k: f64, duct_k: f64) -> f64 {
    rot_k - approach_k - duct_k
}

/// Limits of card C.18 that the exchanger and the reformer must meet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// Secondary supply less reformer process exit, K; at least 10 K.
    pub supply_over_reformer_k: f64,
    pub supply_ok: bool,
    /// Hottest tube wall, taken as the primary inlet, against 955 C.
    pub tube_ok: bool,
}

/// Check the limits of card C.18 (INL-21537 Table 2 PDF p16; Tochio Table 2).
///
/// eq: C.18
pub fn check_limits(t_supply_c: f64, t_reformer_out_c: f64, t_primary_in_c: f64) -> Limits {
    let gap = t_supply_c - t_reformer_out_c;
    Limits {
        supply_over_reformer_k: gap,
        supply_ok: gap >= 10.0,
        tube_ok: t_primary_in_c <= 955.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_c_7_lmtd_reaches_its_balanced_limit() {
        assert!((lmtd(25.0, 25.0).unwrap() - 25.0).abs() < 1e-12);
        let a = lmtd(25.0, 25.0 * (1.0 + 1e-6)).unwrap();
        assert!((a - 25.0).abs() < 1e-4);
        let b = lmtd(77.0, 135.0).unwrap();
        assert!((b - 103.3).abs() < 0.1, "{b}");
        assert!(lmtd(-1.0, 3.0).is_err());
    }

    #[test]
    fn eq_c_11_counterflow_limits() {
        assert_eq!(eps_ntu(0.0, 0.5).unwrap(), 0.0);
        let big = eps_ntu(200.0, 0.5).unwrap();
        assert!((big - 1.0).abs() < 1e-9);
        // C.12: balanced limit agrees with the general form near c* = 1.
        let bal = eps_ntu(17.36, 1.0).unwrap();
        let near = eps_ntu(17.36, 1.0 - 1e-6).unwrap();
        assert!((bal - near).abs() < 1e-5);
        assert!(eps_ntu(1.0, 1.5).is_err());
    }

    #[test]
    fn eq_c_13_sizing_and_rating_agree() {
        // Balanced 25 K approach at 121.7 MW, the Case B form of card C.13.
        let s = size(121.7e6, 1198.15, 764.15, 739.15, 1173.15).unwrap();
        assert!((s.lmtd_k - 25.0).abs() < 1e-9);
        assert!((s.ua_w_k - 121.7e6 / 25.0).abs() < 1e-3);
        assert!((s.c_star - 1.0).abs() < 1e-12);
        let c = 121.7e6 / 434.0;
        let r = rate(1198.15, 739.15, c, c, s.ua_w_k).unwrap();
        assert!((r.q_w / 121.7e6 - 1.0).abs() < 1e-9);
        assert!((r.t_cold_out_k - 1173.15).abs() < 1e-6);
        assert!((s.modules_httr - s.ua_w_k / 85.0e3).abs() < 1e-9);
        let (u, a) = s.area_m2[0];
        assert_eq!(u, 400.0);
        assert!((a - 121.7e6 / 400.0 / 25.0).abs() < 1e-6);
    }

    #[test]
    fn eq_c_3_closure_is_zero_when_the_loss_covers_the_gap() {
        let qh = hot_side(4.1, 5195.0, 828.0, 383.0);
        let qc = cold_side(3.6, 5195.0, 248.0, 751.0);
        assert!(closure(qh, qc, qh - qc).abs() < 1e-9);
        assert!((loss(0.008, 100.0) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn eq_c_17_cascade_and_c_18_limits() {
        let t = cascade(1198.15, 25.0, 0.0) - 273.15;
        assert!((t - 900.0).abs() < 1e-9);
        let l = check_limits(900.0, 871.0, 925.0);
        assert!(l.supply_ok && l.tube_ok);
        assert!(!check_limits(880.0, 871.0, 925.0).supply_ok);
    }
}
