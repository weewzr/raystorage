//! STRIP: equilibrium-stage stripper with a partial condenser and a kettle
//! reboiler, class ReducedOrder (plan section 4).
//!
//! Each stage holds a liquid at its bubble point, p*_CO2 + p*_H2O = p, with p* from
//! the Aronu model, and a vapour set by a Murphree efficiency on the CO2 fraction.
//! The reboiler is the last equilibrium stage. The rich feed and the condenser
//! reflux enter the top stage. The condenser cools the overhead to `t_cond_k` and
//! returns all condensed water; the CO2 vent leaves saturated with water (I.8).
//! Kinetics are left out: the plan's equilibrium stage (I.3) assumes the reaction
//! is fast at 120 C, and the Murphree efficiency is an assumption (no local source
//! measures it for MEA). MEA volatility and heat loss are not modelled.
//!
//! Five unknowns per stage, top to bottom: temperature, loading (or, at the
//! reboiler when the lean loading is given, the reboiler duty), liquid water flow,
//! vapour flow and vapour CO2 fraction. `newton::solve` runs Newton then
//! pseudo-transient continuation.

use crate::newton::{self, Shape, SolvePath};
use crate::solve::brent;
use crate::solvent::{self, idx, Notes, Solvent, CP_SOLVENT_J_KG_K, MW_CO2};
use crate::stream::{Closure, SolventStream, Sp, Stream, NSP};
use nush2_mea::aronu::{MW_MEA, MW_WATER};
use nush2_numerics::{EngineError, Result};
use nush2_props::species;
use nush2_props::steam::psat_pa;

const NB: usize = 5;
/// Residual target, plan section 5.
pub const TOL: f64 = 1.0e-7;

/// What the reboiler is set by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StripMode {
    /// Lean loading leaving the reboiler; the duty follows.
    LeanLoading(f64),
    /// Reboiler duty, W; the lean loading follows.
    ReboilerDuty(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StripperSpec {
    pub tag: String,
    /// Stages above the reboiler.
    pub stages: usize,
    /// Murphree efficiency on the vapour CO2 fraction, assumed.
    pub murphree: f64,
    pub p_top_pa: f64,
    pub p_bottom_pa: f64,
    /// Condenser outlet temperature, K.
    pub t_cond_k: f64,
    /// Temperature of the enthalpy path, K (see `solvent`).
    pub t_phi_k: f64,
    /// MEA mass fraction on a CO2-free basis for the thermodynamics.
    pub w_mea: f64,
    pub mode: StripMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageState {
    pub t_k: f64,
    pub alpha: f64,
    pub y_co2: f64,
    pub vapour_mol_s: f64,
    pub p_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StripperResult {
    pub lean: SolventStream,
    /// CO2 product with saturated water at the condenser outlet.
    pub vent: Stream,
    pub q_reb_w: f64,
    pub q_cond_w: f64,
    pub reflux_mol_s: f64,
    pub t_reb_k: f64,
    pub t_top_k: f64,
    /// H2O over CO2 in the top vapour, mol/mol.
    pub steam_ratio_top: f64,
    pub co2_stripped_mol_s: f64,
    /// Reboiler duty per kg of CO2 vented, J/kg.
    pub srd_j_kg: f64,
    pub stages: Vec<StageState>,
    pub closure: Closure,
    pub residual: f64,
    pub iterations: usize,
    pub path: SolvePath,
    pub notes: Notes,
}

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

/// CO2 removed from the solvent, mol/s, n_MEA (alpha_rich - alpha_lean).
///
/// eq: I.1
pub fn co2_balance(n_mea: f64, alpha_rich: f64, alpha_lean: f64) -> f64 {
    n_mea * (alpha_rich - alpha_lean)
}

/// Rich-end pinch: water over CO2 in vapour leaving at equilibrium with the
/// liquid, (p - p*_CO2)/p*_CO2.
///
/// eq: I.4
pub fn steam_ratio(p_pa: f64, pstar_co2_pa: f64) -> Result<f64> {
    positive("pstar_co2_pa", pstar_co2_pa)?;
    Ok((p_pa - pstar_co2_pa) / pstar_co2_pa)
}

/// Bubble temperature of the solvent at loading `alpha` and pressure `p_pa`, K:
/// p*_CO2 + p*_H2O = p. MEA vapour pressure is not modelled.
///
/// eq: I.5
pub fn t_reboiler(solvent: &Solvent, alpha: f64, p_pa: f64) -> Result<f64> {
    let f = |t: f64| -> Result<f64> {
        let (pc, pw) = solvent.bubble_pressures(t, alpha)?;
        Ok((pc + pw) / p_pa - 1.0)
    };
    brent(f, 300.0, 403.0, 1e-9, 200)
}

/// Reboiler duty from the stage energy balance, W: enthalpy out less enthalpy in.
///
/// eq: I.6
pub fn reboiler(h_out_w: f64, h_in_w: f64) -> f64 {
    h_out_w - h_in_w
}

/// Condenser duty, W, removed from the overhead: H2O condensed, steam and CO2
/// cooled, taken as the enthalpy of the overhead less vent and reflux.
///
/// eq: I.7
pub fn condenser(h_overhead_w: f64, h_vent_w: f64, h_reflux_w: f64) -> f64 {
    h_overhead_w - h_vent_w - h_reflux_w
}

/// Reflux, mol/s: water in the overhead less the water a saturated vent carries.
///
/// eq: I.8
pub fn reflux(co2_overhead: f64, h2o_overhead: f64, t_cond_k: f64, p_pa: f64) -> Result<f64> {
    let y = psat_pa(t_cond_k)? / p_pa;
    if !(0.0..1.0).contains(&y) {
        return Err(EngineError::Range {
            param: "t_cond_k".into(),
            msg: format!("{t_cond_k} K boils water at {p_pa} Pa"),
        });
    }
    Ok(h2o_overhead - co2_overhead * y / (1.0 - y))
}

struct Column<'a> {
    spec: &'a StripperSpec,
    solvent: Solvent,
    n: usize,
    m: f64,
    rich_c: f64,
    rich_w: f64,
    h_rich: f64,
    s_c: f64,
    s_e: f64,
    s_q: f64,
    y_vent_w: f64,
    h_reflux_mol: f64,
}

#[derive(Debug, Clone, Copy)]
struct St {
    t: f64,
    a: f64,
    w: f64,
    v: f64,
    y: f64,
}

impl<'a> Column<'a> {
    fn p(&self, k: usize) -> f64 {
        let f = k as f64 / self.n as f64;
        self.spec.p_top_pa + f * (self.spec.p_bottom_pa - self.spec.p_top_pa)
    }

    fn lean_fixed(&self) -> Option<f64> {
        match self.spec.mode {
            StripMode::LeanLoading(a) => Some(a),
            StripMode::ReboilerDuty(_) => None,
        }
    }

    /// Stage k from the unknowns; at the reboiler in lean mode the loading slot holds
    /// the scaled duty.
    fn stage(&self, x: &[f64], k: usize) -> (St, Option<f64>) {
        let b = k * NB;
        let mut s = St {
            t: x[b],
            a: x[b + 1],
            w: x[b + 2],
            v: x[b + 3],
            y: x[b + 4],
        };
        let mut q = None;
        if k == self.n {
            match self.spec.mode {
                StripMode::LeanLoading(a) => {
                    q = Some(x[b + 1] * self.s_q);
                    s.a = a;
                }
                StripMode::ReboilerDuty(qr) => q = Some(qr),
            }
        }
        (s, q)
    }

    fn h_liq(&self, s: &St) -> Result<f64> {
        self.solvent.h_liquid_w(self.m, s.w, self.m * s.a, s.t)
    }

    fn h_vap(&self, s: &St) -> Result<f64> {
        let mut n = [0.0; NSP];
        n[idx(Sp::Co2)] = s.v * s.y;
        n[idx(Sp::H2o)] = s.v * (1.0 - s.y);
        solvent::h_gas_w(&n, s.t)
    }

    fn reflux_of(&self, top: &St) -> f64 {
        let c = top.v * top.y;
        top.v * (1.0 - top.y) - c * self.y_vent_w / (1.0 - self.y_vent_w)
    }

    fn residuals(&self, x: &[f64]) -> Result<Vec<f64>> {
        let nst = self.n + 1;
        let mut r = vec![0.0; nst * NB];
        let top = self.stage(x, 0).0;
        let reflux = self.reflux_of(&top);
        for k in 0..nst {
            let (s, q) = self.stage(x, k);
            let p = self.p(k);
            let (l_c, l_w, h_lin) = if k == 0 {
                (
                    self.rich_c,
                    self.rich_w + reflux,
                    self.h_rich + reflux * self.h_reflux_mol,
                )
            } else {
                let a = self.stage(x, k - 1).0;
                (self.m * a.a, a.w, self.h_liq(&a)?)
            };
            let (v_c, v_w, h_vin, y_in) = if k == self.n {
                (0.0, 0.0, 0.0, None)
            } else {
                let b = self.stage(x, k + 1).0;
                (b.v * b.y, b.v * (1.0 - b.y), self.h_vap(&b)?, Some(b.y))
            };
            let (pc, pw) = self.solvent.bubble_pressures(s.t, s.a)?;
            let ystar = pc / p;
            let base = k * NB;
            r[base] = (l_c + v_c - self.m * s.a - s.v * s.y) / self.s_c;
            r[base + 1] = (l_w + v_w - s.w - s.v * (1.0 - s.y)) / self.s_c;
            r[base + 2] =
                (h_lin + h_vin + q.unwrap_or(0.0) - self.h_liq(&s)? - self.h_vap(&s)?) / self.s_e;
            r[base + 3] = (pc + pw) / p - 1.0;
            r[base + 4] = match y_in {
                Some(yb) => s.y - yb - self.spec.murphree * (ystar - yb),
                None => s.y - ystar,
            };
        }
        Ok(r)
    }

    fn guess(&self) -> Result<Vec<f64>> {
        let nst = self.n + 1;
        let a_rich = self.rich_c / self.m;
        let a_lean = match self.spec.mode {
            StripMode::LeanLoading(a) => a,
            StripMode::ReboilerDuty(q) => {
                (a_rich - q / 2.0e5 / self.m).clamp(0.05, (a_rich - 0.02).max(0.03))
            }
        };
        let s_total = self.m * (a_rich - a_lean);
        let mut x = vec![0.0; nst * NB];
        for k in 0..nst {
            let f = (k as f64 + 1.0) / nst as f64;
            let a = a_rich + f * (a_lean - a_rich);
            let p = self.p(k);
            let t = t_reboiler(&self.solvent, a, p)?;
            let (pc, _) = self.solvent.bubble_pressures(t, a)?;
            let y = (pc / p).clamp(1e-4, 0.999);
            let a_above = if k == 0 {
                a_rich
            } else {
                a_rich + (k as f64 / nst as f64) * (a_lean - a_rich)
            };
            let co2_up = if k == 0 {
                s_total
            } else {
                (self.m * a_above + s_total - self.m * a_rich).max(1e-3 * s_total)
            };
            let b = k * NB;
            x[b] = t;
            x[b + 1] = a;
            x[b + 2] = self.rich_w;
            x[b + 3] = co2_up / y;
            x[b + 4] = y;
        }
        if self.lean_fixed().is_some() {
            x[self.n * NB + 1] = 1.0;
        }
        Ok(x)
    }

    fn shape(&self) -> Shape {
        let nst = self.n + 1;
        let mut lo = vec![0.0; nst * NB];
        let mut hi = vec![0.0; nst * NB];
        let mut scale = vec![0.0; nst * NB];
        for k in 0..nst {
            let b = k * NB;
            let alpha_slot = if k == self.n && self.lean_fixed().is_some() {
                (0.0, 100.0, 1.0)
            } else {
                (0.0005, 1.2, 0.1)
            };
            for (j, (l, h, s)) in [
                (298.2, 403.0, 300.0),
                alpha_slot,
                (0.2 * self.rich_w, 5.0 * self.rich_w, self.rich_w),
                (0.0, 1.0e3 * self.s_c, self.s_c),
                (1e-6, 1.0, 1.0),
            ]
            .into_iter()
            .enumerate()
            {
                lo[b + j] = l;
                hi[b + j] = h;
                scale[b + j] = s;
            }
        }
        Shape {
            block: NB,
            reach: 1,
            lo,
            hi,
            scale,
        }
    }
}

/// Solve the stripper on a rich solvent at its inlet temperature.
///
/// eq: I.3
pub fn stages(spec: &StripperSpec, rich: &SolventStream) -> Result<StripperResult> {
    positive("rich n_mea", rich.n_mea)?;
    positive("rich n_co2", rich.n_co2)?;
    positive("p_top_pa", spec.p_top_pa)?;
    positive("p_bottom_pa", spec.p_bottom_pa)?;
    if spec.stages < 1 || !(0.0..=1.0).contains(&spec.murphree) || spec.murphree == 0.0 {
        return Err(EngineError::Range {
            param: "stages, murphree".into(),
            msg: format!("{} stages at efficiency {}", spec.stages, spec.murphree),
        });
    }
    let solvent = Solvent::new(spec.w_mea, spec.t_phi_k)?;
    let h_rich = solvent.h_liquid_w(rich.n_mea, rich.n_h2o, rich.n_co2, rich.t_k)?;
    let m_rich = rich.n_mea * MW_MEA + rich.n_h2o * MW_WATER + rich.n_co2 * MW_CO2;
    let y_vent_w = psat_pa(spec.t_cond_k)? / spec.p_top_pa;
    let col = Column {
        spec,
        solvent,
        n: spec.stages,
        m: rich.n_mea,
        rich_c: rich.n_co2,
        rich_w: rich.n_h2o,
        h_rich,
        s_c: rich.n_co2,
        s_e: m_rich * CP_SOLVENT_J_KG_K,
        s_q: rich.n_co2 * 2.0e5,
        y_vent_w,
        h_reflux_mol: species::h_liquid_water_j_mol(spec.t_cond_k, spec.p_top_pa)?,
    };
    if let StripMode::LeanLoading(a) = spec.mode {
        if !(a > 0.0 && a < rich.n_co2 / rich.n_mea) {
            return Err(EngineError::Range {
                param: "lean loading".into(),
                msg: format!("{a} must lie between 0 and the rich loading"),
            });
        }
    }
    let x0 = col.guess()?;
    let out = newton::solve(|x| col.residuals(x), &x0, &col.shape(), TOL)?;
    let x = &out.x;
    let nst = col.n + 1;
    let mut st = Vec::with_capacity(nst);
    for k in 0..nst {
        let s = col.stage(x, k).0;
        st.push(StageState {
            t_k: s.t,
            alpha: s.a,
            y_co2: s.y,
            vapour_mol_s: s.v,
            p_pa: col.p(k),
        });
    }
    let (bot, q) = col.stage(x, col.n);
    let q_reb = q.unwrap_or(0.0);
    let top = col.stage(x, 0).0;
    let refl = col.reflux_of(&top);
    let c_top = top.v * top.y;
    let w_vent = c_top * y_vent_w / (1.0 - y_vent_w);
    let mut nv = [0.0; NSP];
    nv[idx(Sp::Co2)] = c_top;
    nv[idx(Sp::H2o)] = w_vent;
    let vent = Stream::gas(
        format!("{}_VENT", spec.tag),
        spec.t_cond_k,
        spec.p_top_pa,
        nv,
    );
    let h_vent = solvent::h_gas_w(&nv, spec.t_cond_k)?;
    let q_cond = condenser(col.h_vap(&top)?, h_vent, refl * col.h_reflux_mol);
    let lean = SolventStream {
        tag: format!("{}_LEAN", spec.tag),
        t_k: bot.t,
        p_pa: spec.p_bottom_pa,
        n_mea: rich.n_mea,
        n_h2o: bot.w,
        n_co2: rich.n_mea * bot.a,
    };
    let h_lean = col.h_liq(&bot)?;
    let energy_in = h_rich + q_reb;
    let energy_out = h_lean + h_vent + q_cond;
    let scale = energy_in.abs().max(energy_out.abs()).max(1.0);
    let el = |c: f64, h2o: f64| -> [f64; 4] {
        [
            2.0 * rich.n_mea + c,
            7.0 * rich.n_mea + 2.0 * h2o,
            rich.n_mea + 2.0 * c + h2o,
            rich.n_mea,
        ]
    };
    let e_in = el(rich.n_co2, rich.n_h2o);
    let mut e_out = el(lean.n_co2, lean.n_h2o);
    e_out[0] += c_top - 0.0;
    e_out[1] += 2.0 * w_vent;
    e_out[2] += 2.0 * c_top + w_vent;
    // MEA atoms counted once: the vent carries none.
    let mut element_rel = [0.0; 4];
    for k in 0..4 {
        let s = e_in[k].abs().max(e_out[k].abs());
        element_rel[k] = if s > 0.0 {
            (e_in[k] - e_out[k]).abs() / s
        } else {
            0.0
        };
    }
    let stripped = c_top;
    Ok(StripperResult {
        srd_j_kg: q_reb / (stripped * MW_CO2),
        lean,
        vent,
        q_reb_w: q_reb,
        q_cond_w: q_cond,
        reflux_mol_s: refl,
        t_reb_k: bot.t,
        t_top_k: top.t,
        steam_ratio_top: (1.0 - top.y) / top.y,
        co2_stripped_mol_s: stripped,
        stages: st,
        closure: Closure {
            element_rel,
            energy_rel: (energy_in - energy_out).abs() / scale,
        },
        residual: out.residual,
        iterations: out.iterations,
        path: out.path,
        notes: col.solvent.take_notes(),
    })
}

/// Desorption driving force on a stage, Pa: the H.4 form with the sign reversed,
/// p* - p_CO2. The equilibrium stage takes it to zero, so this is reported only.
///
/// eq: I.2
pub fn desorb(pstar_co2_pa: f64, p_co2_pa: f64) -> f64 {
    pstar_co2_pa - p_co2_pa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_i_01_balance_and_eq_i_08_reflux() {
        assert!((co2_balance(10.0, 0.5, 0.25) - 2.5).abs() < 1e-12);
        // At 40 C and 2 bar the vent holds 3.7% water.
        let r = reflux(1.0, 1.0, 313.15, 2.0e5).unwrap();
        let y = psat_pa(313.15).unwrap() / 2.0e5;
        assert!((r - (1.0 - y / (1.0 - y))).abs() < 1e-12);
    }

    #[test]
    fn eq_i_05_reboiler_temperature_rises_as_loading_falls() {
        let s = Solvent::new(0.30, 383.15).unwrap();
        let hi = t_reboiler(&s, 0.15, 2.0e5).unwrap();
        let lo = t_reboiler(&s, 0.30, 2.0e5).unwrap();
        println!(
            "bubble T at 2 bar: {:.1} C at 0.15, {:.1} C at 0.30",
            hi - 273.15,
            lo - 273.15
        );
        assert!(hi > lo);
        assert!(hi > 380.0 && hi < 403.0);
    }
}
