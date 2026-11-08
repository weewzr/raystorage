//! Liquid speciation by Newton's method. Five equilibria from Aronu Table 2 and
//! four balances (MEA, carbon, hydrogen, charge), with unknowns ln n_i per mol of
//! apparent MEA. The residual scaling and the starting guess are those of the
//! Python prototype.

use crate::aronu::{Thermo, I_CO2, I_CO3, I_H2O, I_H3O, I_HCO3, I_MEA, I_MEACOO, I_MEAH, I_OH, NS};
use crate::solve::lu_solve;
use nush2_numerics::{EngineError, Result};

/// Lowest temperature accepted, K. Aronu's title gives 40 to 120 C (PDF p1), and Table 2
/// rates K_MEACOO from 25 C (PDF p4), so 25 C to 40 C is extrapolation.
pub const T_MIN_K: f64 = 298.15;
/// Highest temperature accepted, K. `PStarTable` runs to 130 C, so 120 C to 130 C is
/// extrapolation.
pub const T_MAX_K: f64 = 403.15;
/// Lowest MEA mass fraction accepted, CO2-free basis.
pub const W_MIN: f64 = 0.10;
/// Highest MEA mass fraction accepted, CO2-free basis, the top of Aronu's range.
pub const W_MAX: f64 = 0.60;
/// Lowest loading accepted.
pub const ALPHA_MIN: f64 = 0.001;
/// Highest loading accepted. Aronu's 30 mass% data stop at 0.541, and 15 mass% at 0.864.
pub const ALPHA_MAX: f64 = 1.4;

const TOL: f64 = 1e-11;
const ACCEPT: f64 = 1e-9;
const MAX_ITER: usize = 200;
const STEP_LIMIT: f64 = 5.0;
const FD_STEP: f64 = 1e-7;

/// Reference basis of K_MEA in the residual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KMeaBasis {
    /// Table 2 value plus ln gamma_inf,MEA. The model's convention.
    PureMea,
    /// Table 2 value alone. Kept to show that the shift is required.
    AsPrinted,
}

/// One converged liquid state.
#[derive(Debug, Clone, PartialEq)]
pub struct Speciation {
    pub t_k: f64,
    /// MEA mass fraction on a CO2-free basis.
    pub w_mea: f64,
    /// mol CO2 per mol MEA.
    pub alpha: f64,
    /// mol of each species per mol of apparent MEA.
    pub n: [f64; NS],
    /// Mole fractions.
    pub x: [f64; NS],
    pub ln_gamma: [f64; NS],
    /// ln(H/Pa) of CO2 at `t_k`.
    pub ln_h_co2: f64,
    /// Largest absolute residual of the scaled system.
    pub residual: f64,
    /// Apparent water, mol per mol of apparent MEA.
    pub n_water_apparent: f64,
}

/// Range check shared by the direct solvers.
pub fn check_range(t_k: f64, w_mea: f64, alpha: f64) -> Result<()> {
    if !(T_MIN_K..=T_MAX_K).contains(&t_k) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_MIN_K} to {T_MAX_K} K"),
        });
    }
    if !(W_MIN..=W_MAX).contains(&w_mea) {
        return Err(EngineError::Range {
            param: "w_mea".into(),
            msg: format!("{w_mea} lies outside {W_MIN} to {W_MAX}"),
        });
    }
    if !(ALPHA_MIN..=ALPHA_MAX).contains(&alpha) {
        return Err(EngineError::Range {
            param: "alpha".into(),
            msg: format!("{alpha} lies outside {ALPHA_MIN} to {ALPHA_MAX}"),
        });
    }
    Ok(())
}

fn ideal_guess(alpha: f64, n_w: f64) -> [f64; NS] {
    let a = alpha.min(0.95);
    let (carb, hco3) = if a <= 0.5 {
        (a * 0.97, a * 0.03)
    } else {
        let hco3 = (a - 0.5) * 1.5;
        ((a - hco3).max(1e-3), hco3)
    };
    let mh = carb + hco3;
    let mea = (1.0 - mh - carb).max(1e-4);
    let co3 = (hco3 * 0.02).max(1e-6);
    let co2 = 1e-6 + 1e-4 * a;
    let mut n = [0.0; NS];
    n[I_H2O] = n_w;
    n[I_MEA] = mea;
    n[I_CO2] = co2;
    n[I_H3O] = 1e-11;
    n[I_MEAH] = mh;
    n[I_OH] = 1e-7;
    n[I_HCO3] = hco3;
    n[I_CO3] = co3;
    n[I_MEACOO] = carb;
    n
}

struct Problem<'a> {
    th: &'a Thermo,
    alpha: f64,
    n_w0: f64,
    ln_k_mea: f64,
}

fn inf_norm(r: &[f64; NS]) -> f64 {
    r.iter()
        .fold(0.0, |m, v| if v.abs() > m { v.abs() } else { m })
}

fn l2_norm(r: &[f64; NS]) -> f64 {
    r.iter().map(|v| v * v).sum::<f64>().sqrt()
}

impl Problem<'_> {
    fn residual(&self, lnn: &[f64; NS]) -> [f64; NS] {
        let th = self.th;
        let mut n = [0.0; NS];
        for i in 0..NS {
            n[i] = lnn[i].exp();
        }
        let total: f64 = n.iter().sum();
        let mut x = [0.0; NS];
        for i in 0..NS {
            x[i] = n[i] / total;
        }
        let lng = th.ln_gamma_raw(&x);
        let mut lna = [0.0; NS];
        for i in 0..NS {
            lna[i] = lng[i] + x[i].ln();
        }
        let hydrogen = 2.0 * n[I_H2O]
            + 7.0 * n[I_MEA]
            + 3.0 * n[I_H3O]
            + 8.0 * n[I_MEAH]
            + n[I_OH]
            + n[I_HCO3]
            + 6.0 * n[I_MEACOO];
        [
            lna[I_H3O] + lna[I_OH] - 2.0 * lna[I_H2O] - th.ln_k_water,
            lna[I_H3O] + lna[I_HCO3] - lna[I_CO2] - 2.0 * lna[I_H2O] - th.ln_k_co2,
            lna[I_H3O] + lna[I_CO3] - lna[I_HCO3] - lna[I_H2O] - th.ln_k_hco3,
            lna[I_H3O] + lna[I_MEA] - lna[I_MEAH] - lna[I_H2O] - self.ln_k_mea,
            lna[I_MEA] + lna[I_HCO3] - lna[I_MEACOO] - lna[I_H2O] - th.ln_k_meacoo,
            (n[I_MEA] + n[I_MEAH] + n[I_MEACOO] - 1.0) * 100.0,
            (n[I_CO2] + n[I_HCO3] + n[I_CO3] + n[I_MEACOO] - self.alpha) * 100.0,
            hydrogen - (2.0 * self.n_w0 + 7.0),
            (n[I_H3O] + n[I_MEAH] - n[I_OH] - n[I_HCO3] - 2.0 * n[I_CO3] - n[I_MEACOO]) * 100.0,
        ]
    }

    fn jacobian(&self, lnn: &[f64; NS], r0: &[f64; NS]) -> [[f64; NS]; NS] {
        let mut jac = [[0.0; NS]; NS];
        let mut probe = *lnn;
        for j in 0..NS {
            probe[j] = lnn[j] + FD_STEP;
            let rp = self.residual(&probe);
            probe[j] = lnn[j];
            for i in 0..NS {
                jac[i][j] = (rp[i] - r0[i]) / FD_STEP;
            }
        }
        jac
    }

    /// Damped Newton. Returns the iterate and its infinity norm, or `None` when the
    /// iteration stalls above the acceptance level.
    fn newton(&self, start: [f64; NS]) -> Option<([f64; NS], f64)> {
        let mut lnn = start;
        let mut r = self.residual(&lnn);
        for _ in 0..MAX_ITER {
            let norm = l2_norm(&r);
            if !norm.is_finite() {
                return None;
            }
            if inf_norm(&r) < TOL {
                return Some((lnn, inf_norm(&r)));
            }
            let jac = self.jacobian(&lnn, &r);
            let mut rhs = r;
            for v in rhs.iter_mut() {
                *v = -*v;
            }
            let Some(step) = lu_solve(jac, rhs) else {
                break;
            };
            let biggest = step.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            let mut lam = if biggest > STEP_LIMIT {
                STEP_LIMIT / biggest
            } else {
                1.0
            };
            let mut accepted = false;
            for _ in 0..40 {
                let mut trial = lnn;
                for i in 0..NS {
                    trial[i] += lam * step[i];
                }
                let rt = self.residual(&trial);
                if l2_norm(&rt) < norm * (1.0 - 1e-4 * lam) {
                    lnn = trial;
                    r = rt;
                    accepted = true;
                    break;
                }
                lam *= 0.5;
            }
            if !accepted {
                break;
            }
        }
        let end = inf_norm(&r);
        (end <= ACCEPT).then_some((lnn, end))
    }
}

fn log_of(n: &[f64; NS]) -> [f64; NS] {
    let mut out = [0.0; NS];
    for i in 0..NS {
        out[i] = n[i].ln();
    }
    out
}

fn solve_lnn(pb: &Problem<'_>) -> Option<([f64; NS], f64)> {
    let guess = ideal_guess(pb.alpha, pb.n_w0);
    if let Some(sol) = pb.newton(log_of(&guess)) {
        return Some(sol);
    }
    // Continuation in loading from a mid-range state.
    const BASE: f64 = 0.30;
    const STEPS: usize = 24;
    let mut step = Problem {
        th: pb.th,
        alpha: BASE,
        n_w0: pb.n_w0,
        ln_k_mea: pb.ln_k_mea,
    };
    let mut sol = step.newton(log_of(&ideal_guess(BASE, pb.n_w0)))?;
    for k in 1..=STEPS {
        step.alpha = BASE + (pb.alpha - BASE) * k as f64 / STEPS as f64;
        sol = step.newton(sol.0)?;
    }
    Some(sol)
}

/// Speciation on the model's K_MEA convention.
pub fn speciate(t_k: f64, w_mea: f64, alpha: f64) -> Result<Speciation> {
    speciate_on(t_k, w_mea, alpha, KMeaBasis::PureMea)
}

/// Speciation with a chosen K_MEA basis.
pub fn speciate_on(t_k: f64, w_mea: f64, alpha: f64, basis: KMeaBasis) -> Result<Speciation> {
    check_range(t_k, w_mea, alpha)?;
    let th = Thermo::new(t_k)?;
    let n_w0 = ((1.0 - w_mea) / 18.01528) / (w_mea / 61.08);
    let ln_k_mea = match basis {
        KMeaBasis::PureMea => th.ln_k_mea_effective(),
        KMeaBasis::AsPrinted => th.ln_k_mea_table,
    };
    let pb = Problem {
        th: &th,
        alpha,
        n_w0,
        ln_k_mea,
    };
    let (lnn, residual) = solve_lnn(&pb).ok_or_else(|| {
        EngineError::NotConverged(format!("speciation at {t_k} K, w {w_mea}, alpha {alpha}"))
    })?;
    let mut n = [0.0; NS];
    for i in 0..NS {
        n[i] = lnn[i].exp();
    }
    let total: f64 = n.iter().sum();
    let mut x = [0.0; NS];
    for i in 0..NS {
        x[i] = n[i] / total;
    }
    Ok(Speciation {
        t_k,
        w_mea,
        alpha,
        n,
        x,
        ln_gamma: th.ln_gamma_raw(&x),
        ln_h_co2: th.ln_h_co2,
        residual,
        n_water_apparent: n_w0,
    })
}

impl Speciation {
    /// Closure of the MEA, carbon, hydrogen, oxygen and charge balances, in that
    /// order, as unscaled differences per mol of apparent MEA. The oxygen balance is
    /// no part of the Newton system, so it is an independent check.
    pub fn balance_errors(&self) -> [f64; 5] {
        let n = &self.n;
        let meab = n[I_MEA] + n[I_MEAH] + n[I_MEACOO] - 1.0;
        let carbon = n[I_CO2] + n[I_HCO3] + n[I_CO3] + n[I_MEACOO] - self.alpha;
        let hydrogen = 2.0 * n[I_H2O]
            + 7.0 * n[I_MEA]
            + 3.0 * n[I_H3O]
            + 8.0 * n[I_MEAH]
            + n[I_OH]
            + n[I_HCO3]
            + 6.0 * n[I_MEACOO]
            - (2.0 * self.n_water_apparent + 7.0);
        let oxygen = n[I_H2O]
            + n[I_MEA]
            + 2.0 * n[I_CO2]
            + n[I_H3O]
            + n[I_MEAH]
            + n[I_OH]
            + 3.0 * n[I_HCO3]
            + 3.0 * n[I_CO3]
            + 3.0 * n[I_MEACOO]
            - (self.n_water_apparent + 1.0 + 2.0 * self.alpha);
        let charge = n[I_H3O] + n[I_MEAH] - n[I_OH] - n[I_HCO3] - 2.0 * n[I_CO3] - n[I_MEACOO];
        [meab, carbon, hydrogen, oxygen, charge]
    }
}
