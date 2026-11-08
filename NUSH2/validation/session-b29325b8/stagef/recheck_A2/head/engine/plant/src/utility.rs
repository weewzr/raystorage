//! Utility balance of the nuclear island and the Level 4 check on real flows.
//!
//! The first four functions are the closure of the anchored balance, which calls
//! them, so every preset runs through the same arithmetic. `level4` then takes a
//! solved balance and rebuilds the reactor side from unit models on the flows of
//! that balance: helium flows from the duties (card B.9), circulator power on
//! those flows (B.10, B.12), the IHX size behind the duty (C.13, C.14), the
//! duct loss on pipe diameter (B.14) and the secondary loop drop behind term
//! T10 (C.16). These are printed beside the balance; the balance itself keeps
//! the anchored circulator and loss terms, since T10, T11 and T12 stay anchored.

use crate::anchored::{Balance, Model};
use nush2_numerics::Result;
use nush2_props::helium;
use nush2_units::circulator::{self, Friction};
use nush2_units::ihx::{self, Limits, Sizing};
use nush2_units::jaeri::A2_FIT_ACTIVITY;
use nush2_units::smr_tube::HeatModel;
use nush2_units::tube_case_b;
use serde::Serialize;
use std::f64::consts::PI;

const C_TO_K: f64 = 273.15;

/// Terms of the first law on the nuclear island, MW.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactorTerms {
    pub q_proc: f64,
    pub q_recyc: f64,
    pub q_ihx: f64,
    pub q_duct: f64,
    pub qv: f64,
    pub branch: f64,
    pub w_s: f64,
    pub w_p: f64,
}

/// Reactor heat the plant needs,
/// `Q_R = Q_proc + Q_rec + Q_ihx + Q_duct + Q_ves + Q_br - W_s - W_p`, MW.
///
/// eq: B.20
/// eq: B.16
/// eq: J.25
pub fn reactor_required(t: &ReactorTerms) -> f64 {
    t.q_proc + t.q_recyc + t.q_ihx + t.q_duct + t.qv + t.branch - (t.w_s + t.w_p)
}

/// Spare rating, `Q_spare = Q_rated - Q_R`, MW. No revenue is credited.
///
/// eq: B.21
/// eq: J.36
pub fn margin(r_rated: f64, r_needed: f64) -> f64 {
    r_rated - r_needed
}

/// Electricity import when the reactor is held at `r_fixed`, heat duties first,
/// `E_imp = E_nuc - max(0, Q_rated - Q_heat) eta`, MWe. Negative means spare.
/// All the leftover heat runs at `eta`, so on the extraction route with a
/// reboiler gap the value reads high by up to `w (1 - eta)`, `w` the
/// back-pressure work of card B.19. `Model::shortfall_at` calls this only below
/// the need.
///
/// eq: J.29
pub fn import(e_nuc: f64, r_fixed: f64, heat_first: f64, eta_p: f64) -> f64 {
    e_nuc - (r_fixed - heat_first).max(0.0) * eta_p
}

/// Total electricity, `E_tot = E_proc + E_circ`, MWe.
///
/// eq: J.24
pub fn e_total(e_proc: f64, e_circ: f64) -> f64 {
    e_proc + e_circ
}

/// Secondary loop drop behind term T10 on one tube count.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoopDrop {
    /// Reformer tubes on the assumed HA tube.
    pub smr_tubes: f64,
    /// Helium leaving the reformer tube, C, from `tube_case_b`.
    pub smr_he_out_c: f64,
    pub smr_kpa: f64,
    pub smr_re: f64,
    /// IHX tubes: HTTR-size modules times 96.
    pub ihx_tubes: f64,
    pub ihx_kpa: f64,
    pub ihx_re: f64,
    pub total_kpa: f64,
    /// False when either Reynolds number lies outside the Blasius range.
    pub in_range: bool,
}

/// Level 4 rebuild of the reactor side on the flows of one balance.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Level4 {
    /// IHX duty, process heat with slip heat, MW.
    pub q_duty_mw: f64,
    /// Secondary helium from the duty (B.9) and as the port scales it.
    pub he_sec_kg_s: f64,
    pub he_sec_port_kg_s: f64,
    /// Primary helium for the reactor need (B.9).
    pub he_prim_kg_s: f64,
    /// Circulator shaft power on those flows at the central budget, MW.
    pub w_sec_mw: f64,
    pub w_prim_mw: f64,
    pub e_circ_mwe: f64,
    /// Temperature rise the shaft work leaves in each loop, K.
    pub rise_sec_k: f64,
    pub rise_prim_k: f64,
    /// Duct loss on pipe diameter (B.14) against the port's linear scaling, MW.
    pub duct_b14_mw: f64,
    pub duct_port_mw: f64,
    /// IHX sized on the duty with the primary leaving at the secondary return
    /// plus the approach.
    pub ihx: IhxView,
    pub limits: LimitsView,
    /// Drop on the HA tube count; the T10 modelled slot.
    pub t10: LoopDrop,
    /// The same at the tube counts that bring the exit to 800 and 825 C (decision D29).
    pub t10_alt: Vec<LoopDrop>,
}

/// Serializable copy of an IHX sizing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IhxView {
    pub q_mw: f64,
    pub lmtd_k: f64,
    pub ua_mw_k: f64,
    pub ntu: f64,
    pub eps: f64,
    pub area_m2: Vec<(f64, f64)>,
    pub modules_httr: f64,
    pub primary_out_c: f64,
}

/// Serializable copy of the card C.18 limits.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LimitsView {
    pub supply_over_reformer_k: f64,
    pub supply_ok: bool,
    pub tube_ok: bool,
}

impl From<&Sizing> for IhxView {
    fn from(s: &Sizing) -> Self {
        Self {
            q_mw: s.q_w / 1.0e6,
            lmtd_k: s.lmtd_k,
            ua_mw_k: s.ua_w_k / 1.0e6,
            ntu: s.ntu,
            eps: s.eps,
            area_m2: s.area_m2.clone(),
            modules_httr: s.modules_httr,
            primary_out_c: f64::NAN,
        }
    }
}

impl From<Limits> for LimitsView {
    fn from(l: Limits) -> Self {
        Self {
            supply_over_reformer_k: l.supply_over_reformer_k,
            supply_ok: l.supply_ok,
            tube_ok: l.tube_ok,
        }
    }
}

/// Tube counts behind decision D29: 327 reaches 800 C, 1079 reaches 825 C.
pub const S8_TUBE_COUNTS: [f64; 2] = [327.0, 1079.0];

fn friction(m: f64, area: f64, d_h: f64, length: f64, t_k: f64, p_pa: f64) -> Result<Friction> {
    let rho = helium::density_kg_m3(t_k, p_pa)?;
    let mu = helium::viscosity_pa_s(t_k)?;
    circulator::dp_blasius(m, area, d_h, length, rho, mu)
}

/// Secondary loop drop: the reformer helium annulus of the HA tube plus the IHX
/// tube side, each on the smooth-wall Blasius form. Fins, coil curvature,
/// headers, ducts and valves are not counted, so the sum is a lower bound on
/// the loop drop that the 200 kPa budget of card B.11 covers.
///
/// eq: C.16
pub fn loop_drop(
    he_sec_kg_s: f64,
    p_pa: f64,
    t_sup_k: f64,
    t_ret_k: f64,
    ihx_modules: f64,
    smr_tubes: f64,
) -> Result<LoopDrop> {
    let spec = tube_case_b::spec(smr_tubes, A2_FIT_ACTIVITY)?;
    let (df, guide) = match spec.heat {
        HeatModel::Correlated(c) => (c.tube.df_m, c.guide_id_m),
        HeatModel::Sections => {
            return Err(nush2_numerics::EngineError::Range {
                param: "tube_case_b".into(),
                msg: "the Case B tube has no annulus geometry".into(),
            })
        }
    };
    let (point, _) = tube_case_b::run(smr_tubes, A2_FIT_ACTIVITY)?;
    let t_smr = 0.5 * (t_sup_k + point.he_out_c + C_TO_K);
    let smr = friction(
        he_sec_kg_s / smr_tubes,
        PI / 4.0 * (guide * guide - df * df),
        guide - df,
        spec.length_m,
        t_smr,
        p_pa,
    )?;
    let ihx_tubes = (ihx_modules * ihx::HTTR_TUBES).max(1.0);
    let di = ihx::HTTR_TUBE_OD_M - 2.0 * ihx::HTTR_TUBE_WALL_M;
    let ihx_f = friction(
        he_sec_kg_s / ihx_tubes,
        PI / 4.0 * di * di,
        di,
        ihx::HTTR_TUBE_LENGTH_M,
        0.5 * (t_sup_k + t_ret_k),
        p_pa,
    )?;
    Ok(LoopDrop {
        smr_tubes,
        smr_he_out_c: point.he_out_c,
        smr_kpa: smr.dp_pa / 1.0e3,
        smr_re: smr.re,
        ihx_tubes,
        ihx_kpa: ihx_f.dp_pa / 1.0e3,
        ihx_re: ihx_f.re,
        total_kpa: (smr.dp_pa + ihx_f.dp_pa) / 1.0e3,
        in_range: smr.in_range && ihx_f.in_range,
    })
}

/// Rebuild the reactor side of balance `b` on its own flows. `t_reformer_out_c`
/// is the process exit the secondary supply must clear (card C.18).
pub fn level4(m: &Model, b: &Balance, t_reformer_out_c: f64) -> Result<Level4> {
    let cp = m.cp_he;
    let q_duty = b.q_proc + b.q_recyc;
    let t_sup = m.t_he_s + C_TO_K;
    let t_ret = m.t_he_r + C_TO_K;
    let rot = m.rot + C_TO_K;
    let he_sec = circulator::secondary_flow_kg_s(q_duty * 1.0e6, cp, t_sup, t_ret)?;
    let he_prim = circulator::primary_flow_kg_s(b.r * 1.0e6, cp, rot, t_ret, m.ihx_approach)?;
    let (dp_sec, dp_prim) = m.circ_dp_central();
    let p_mpa = m.circ_p_central_mpa();
    let eta = m.circ_eta_central();
    let w_sec = circulator::shaft(he_sec, cp, m.t_circ_s, p_mpa, dp_sec, eta);
    let w_prim = circulator::shaft(he_prim, cp, m.t_circ_p, p_mpa, dp_prim, eta);
    let t_prim_out = t_ret + m.ihx_approach;
    let sizing = ihx::size(q_duty * 1.0e6, rot, t_prim_out, t_ret, t_sup)?;
    let mut view = IhxView::from(&sizing);
    view.primary_out_c = t_prim_out - C_TO_K;
    let limits = ihx::check_limits(
        ihx::cascade(rot, m.ihx_approach, 0.0) - C_TO_K,
        t_reformer_out_c,
        m.rot,
    );
    let p_pa = p_mpa * 1.0e6;
    let t10 = loop_drop(
        he_sec,
        p_pa,
        t_sup,
        t_ret,
        sizing.modules_httr,
        tube_case_b::HA_TUBES,
    )?;
    let mut t10_alt = Vec::new();
    for n in S8_TUBE_COUNTS {
        t10_alt.push(loop_drop(
            he_sec,
            p_pa,
            t_sup,
            t_ret,
            sizing.modules_httr,
            n,
        )?);
    }
    Ok(Level4 {
        q_duty_mw: q_duty,
        he_sec_kg_s: he_sec,
        he_sec_port_kg_s: m.he_flow * q_duty / m.q_proc,
        he_prim_kg_s: he_prim,
        w_sec_mw: w_sec,
        w_prim_mw: w_prim,
        e_circ_mwe: circulator::electric(w_sec, w_prim, m.eta_motor),
        rise_sec_k: circulator::rise(w_sec, he_sec, cp)?,
        rise_prim_k: circulator::rise(w_prim, he_prim, cp)?,
        duct_b14_mw: circulator::duct_loss(he_sec, m.he_flow, m.loss_central_duct())?,
        duct_port_mw: b.q_duct,
        ihx: view,
        limits: limits.into(),
        t10,
        t10_alt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchored::Scenario;

    #[test]
    fn eq_b_20_reactor_need_adds_up() {
        let t = ReactorTerms {
            q_proc: 120.0,
            q_recyc: 1.0,
            q_ihx: 1.0,
            q_duct: 0.3,
            qv: 2.6,
            branch: 60.0,
            w_s: 3.0,
            w_p: 3.5,
        };
        assert!((reactor_required(&t) - 178.4).abs() < 1e-12);
        assert!((margin(210.0, 178.4) - 31.6).abs() < 1e-12);
        assert_eq!(import(20.0, 210.0, 150.0, 0.397), 20.0 - 60.0 * 0.397);
        assert_eq!(e_total(12.0, 7.5), 19.5);
    }

    #[test]
    fn level4_rebuilds_the_case_b_reactor_side() {
        let m = Model::embedded().unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let l = level4(&m, &b, 871.0).unwrap();
        // B.9 flow and the port's scaled flow differ only by c_p 5195 against
        // the INL-implied 5190.
        assert!((l.he_sec_kg_s / l.he_sec_port_kg_s - 1.0).abs() < 2e-3);
        // Balanced 25 K approach: LMTD 25 K and about 57 HTTR modules.
        assert!((l.ihx.lmtd_k - 25.0).abs() < 1e-6);
        assert!(l.ihx.modules_httr > 50.0 && l.ihx.modules_httr < 65.0);
        assert!((l.ihx.primary_out_c - 491.0).abs() < 1e-9);
        // Shaft power on B.9 flows lands on the port's within the flow ratio.
        assert!((l.w_sec_mw / b.w_s - l.he_sec_kg_s / l.he_sec_port_kg_s).abs() < 1e-9);
        assert!(l.t10.total_kpa > 0.0 && l.t10.smr_kpa > l.t10.ihx_kpa);
        assert!(l.limits.supply_ok);
    }
}
