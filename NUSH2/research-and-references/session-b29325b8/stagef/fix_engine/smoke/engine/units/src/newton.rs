//! Damped Newton for the column models. The Jacobian is built by
//! forward differences with a block colouring, since a slice or stage residual
//! depends only on its neighbours, and solved by dense LU with partial pivoting.
//! Two paths run in order: Newton with a backtracking line search, then
//! pseudo-transient continuation from the same start. A failure of both returns
//! `EngineError::NotConverged` and no best-guess state.

use nush2_numerics::{EngineError, Result};

/// Shape of a block-structured system.
#[derive(Debug, Clone)]
pub(crate) struct Shape {
    /// Unknowns per block (slice or stage).
    pub block: usize,
    /// Residual block j depends on unknown blocks j - reach to j + reach.
    pub reach: usize,
    /// Lower and upper bounds of every unknown.
    pub lo: Vec<f64>,
    pub hi: Vec<f64>,
    /// Size of every unknown. The forward-difference step is 1e-7 times the larger
    /// of the unknown and its size.
    pub scale: Vec<f64>,
}

/// Which path produced the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolvePath {
    Newton,
    PseudoTransient,
}

#[derive(Debug, Clone)]
pub(crate) struct Outcome {
    pub x: Vec<f64>,
    pub residual: f64,
    pub iterations: usize,
    pub path: SolvePath,
}

fn max_abs(r: &[f64]) -> f64 {
    r.iter().fold(0.0_f64, |m, v| m.max(v.abs()))
}

fn norm2(r: &[f64]) -> f64 {
    r.iter().map(|v| v * v).sum::<f64>().sqrt()
}

/// Solve a x = b in place by LU with row scaling and partial pivoting. `a` is row
/// major, n by n. Returns false for a singular or non-finite matrix.
pub(crate) fn lu_dense(a: &mut [f64], b: &mut [f64], n: usize) -> bool {
    for i in 0..n {
        let row = &mut a[i * n..(i + 1) * n];
        let big = row.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        if big == 0.0 || !big.is_finite() {
            return false;
        }
        row.iter_mut().for_each(|v| *v /= big);
        b[i] /= big;
    }
    for k in 0..n {
        let mut piv = k;
        let mut big = a[k * n + k].abs();
        for i in (k + 1)..n {
            let v = a[i * n + k].abs();
            if v > big {
                big = v;
                piv = i;
            }
        }
        if big == 0.0 || !big.is_finite() {
            return false;
        }
        if piv != k {
            for j in 0..n {
                a.swap(k * n + j, piv * n + j);
            }
            b.swap(k, piv);
        }
        let pivot = a[k * n + k];
        for i in (k + 1)..n {
            let f = a[i * n + k] / pivot;
            if f != 0.0 {
                for j in k..n {
                    a[i * n + j] -= f * a[k * n + j];
                }
                b[i] -= f * b[k];
            }
        }
    }
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= a[i * n + j] * b[j];
        }
        b[i] = s / a[i * n + i];
    }
    b.iter().all(|v| v.is_finite())
}

fn jacobian<F>(f: &mut F, x: &[f64], r0: &[f64], s: &Shape) -> Result<Vec<f64>>
where
    F: FnMut(&[f64]) -> Result<Vec<f64>>,
{
    let n = x.len();
    let nb = n / s.block;
    let colours = 2 * s.reach + 1;
    let mut jac = vec![0.0; n * n];
    let mut xp = x.to_vec();
    for c in 0..colours {
        for k in 0..s.block {
            let mut cols = Vec::new();
            for j in (c..nb).step_by(colours) {
                let i = j * s.block + k;
                // Step towards the interior so a bound never stops the difference.
                let step = 1e-7 * x[i].abs().max(s.scale[i]);
                let h = if x[i] + step > s.hi[i] { -step } else { step };
                xp[i] = x[i] + h;
                cols.push((j, i, h));
            }
            if cols.is_empty() {
                continue;
            }
            let rp = f(&xp)?;
            for &(j, i, h) in &cols {
                xp[i] = x[i];
                let b0 = j.saturating_sub(s.reach);
                let b1 = (j + s.reach).min(nb - 1);
                for row in (b0 * s.block)..((b1 + 1) * s.block) {
                    jac[row * n + i] = (rp[row] - r0[row]) / h;
                }
            }
        }
    }
    Ok(jac)
}

/// Largest step fraction that keeps every unknown inside its bounds, leaving a
/// tenth of the gap to a bound unused.
fn bound_fraction(x: &[f64], dx: &[f64], s: &Shape) -> f64 {
    let mut lam = 1.0_f64;
    for i in 0..x.len() {
        if dx[i] < 0.0 && x[i] + dx[i] < s.lo[i] {
            lam = lam.min(0.9 * (x[i] - s.lo[i]) / -dx[i]);
        } else if dx[i] > 0.0 && x[i] + dx[i] > s.hi[i] {
            lam = lam.min(0.9 * (s.hi[i] - x[i]) / dx[i]);
        }
    }
    lam.max(0.0)
}

fn newton<F>(f: &mut F, x0: &[f64], s: &Shape, tol: f64, budget: usize) -> Result<Outcome>
where
    F: FnMut(&[f64]) -> Result<Vec<f64>>,
{
    let n = x0.len();
    let mut x = x0.to_vec();
    let mut r = f(&x)?;
    for it in 0..budget {
        let res = max_abs(&r);
        if res < tol {
            return Ok(Outcome {
                x,
                residual: res,
                iterations: it,
                path: SolvePath::Newton,
            });
        }
        let mut jac = jacobian(f, &x, &r, s)?;
        let mut dx: Vec<f64> = r.iter().map(|v| -v).collect();
        if !lu_dense(&mut jac, &mut dx, n) {
            break;
        }
        let mut lam = bound_fraction(&x, &dx, s);
        let base = norm2(&r);
        let mut accepted = false;
        for _ in 0..30 {
            let xt: Vec<f64> = x.iter().zip(&dx).map(|(a, d)| a + lam * d).collect();
            if let Ok(rt) = f(&xt) {
                if rt.iter().all(|v| v.is_finite()) && norm2(&rt) < (1.0 - 1e-4 * lam) * base {
                    x = xt;
                    r = rt;
                    accepted = true;
                    break;
                }
            }
            lam *= 0.5;
        }
        if !accepted {
            break;
        }
    }
    Err(EngineError::NotConverged(format!(
        "Newton stopped at residual {:.3e}",
        max_abs(&r)
    )))
}

fn pseudo_transient<F>(f: &mut F, x0: &[f64], s: &Shape, tol: f64, budget: usize) -> Result<Outcome>
where
    F: FnMut(&[f64]) -> Result<Vec<f64>>,
{
    let n = x0.len();
    let mut x = x0.to_vec();
    let mut r = f(&x)?;
    let mut tau = 1.0_f64;
    for it in 0..budget {
        let res = max_abs(&r);
        if res < tol {
            return Ok(Outcome {
                x,
                residual: res,
                iterations: it,
                path: SolvePath::PseudoTransient,
            });
        }
        let jac0 = jacobian(f, &x, &r, s)?;
        let mut step_taken = false;
        for _ in 0..12 {
            let mut jac = jac0.clone();
            for i in 0..n {
                jac[i * n + i] += tau * (jac0[i * n + i].abs() + 1e-12);
            }
            let mut dx: Vec<f64> = r.iter().map(|v| -v).collect();
            if !lu_dense(&mut jac, &mut dx, n) {
                tau *= 4.0;
                continue;
            }
            let lam = bound_fraction(&x, &dx, s);
            let xt: Vec<f64> = x.iter().zip(&dx).map(|(a, d)| a + lam * d).collect();
            match f(&xt) {
                Ok(rt) if rt.iter().all(|v| v.is_finite()) => {
                    let ratio = norm2(&rt) / norm2(&r).max(1e-300);
                    if ratio < 2.0 {
                        tau = (tau * ratio).clamp(1e-12, 1e6);
                        x = xt;
                        r = rt;
                        step_taken = true;
                        break;
                    }
                    tau *= 4.0;
                }
                _ => tau *= 4.0,
            }
        }
        if !step_taken {
            break;
        }
    }
    Err(EngineError::NotConverged(format!(
        "pseudo-transient continuation stopped at residual {:.3e}",
        max_abs(&r)
    )))
}

/// Newton, then pseudo-transient continuation from the same start.
///
/// eq: H.26
pub(crate) fn solve<F>(mut f: F, x0: &[f64], s: &Shape, tol: f64) -> Result<Outcome>
where
    F: FnMut(&[f64]) -> Result<Vec<f64>>,
{
    match newton(&mut f, x0, s, tol, 60) {
        Ok(o) => Ok(o),
        Err(first) => pseudo_transient(&mut f, x0, s, tol, 400)
            .map_err(|second| EngineError::NotConverged(format!("{first}; then {second}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_lu_recovers_a_known_solution() {
        let mut a = vec![4.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0];
        let mut b = vec![5.0, 5.0, 3.0];
        assert!(lu_dense(&mut a, &mut b, 3));
        for v in b {
            assert!((v - 1.0).abs() < 1e-12);
        }
        let mut s = vec![1.0, 2.0, 2.0, 4.0];
        let mut c = vec![1.0, 1.0];
        assert!(!lu_dense(&mut s, &mut c, 2));
    }

    #[test]
    fn coloured_newton_solves_a_tridiagonal_nonlinear_chain() {
        // x_j^3 + x_j - (x_{j-1} + x_{j+1})/4 = 1 on 30 blocks of 1 unknown.
        let nb = 30;
        let f = |x: &[f64]| -> Result<Vec<f64>> {
            Ok((0..nb)
                .map(|j| {
                    let l = if j > 0 { x[j - 1] } else { 0.0 };
                    let r = if j + 1 < nb { x[j + 1] } else { 0.0 };
                    x[j].powi(3) + x[j] - 0.25 * (l + r) - 1.0
                })
                .collect())
        };
        let shape = Shape {
            block: 1,
            reach: 1,
            lo: vec![-10.0; nb],
            hi: vec![10.0; nb],
            scale: vec![1.0; nb],
        };
        let out = solve(f, &vec![3.0; nb], &shape, 1e-10).unwrap();
        assert!(out.residual < 1e-10);
        let r = f(&out.x).unwrap();
        assert!(r.iter().all(|v| v.abs() < 1e-9));
    }
}
