//! Small dense solvers for the speciation system. `nush2_numerics` holds no LU
//! or Brent routine yet, so these stay private to this crate.

use nush2_numerics::{EngineError, Result};

/// Solve `a x = b` by LU with row equilibration and partial pivoting.
/// Returns `None` for a singular or non-finite matrix.
pub(crate) fn lu_solve<const N: usize>(mut a: [[f64; N]; N], mut b: [f64; N]) -> Option<[f64; N]> {
    for i in 0..N {
        let big = a[i].iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        if big == 0.0 || !big.is_finite() {
            return None;
        }
        for v in a[i].iter_mut() {
            *v /= big;
        }
        b[i] /= big;
    }
    for k in 0..N {
        let mut piv = k;
        let mut big = a[k][k].abs();
        for i in (k + 1)..N {
            if a[i][k].abs() > big {
                big = a[i][k].abs();
                piv = i;
            }
        }
        if big == 0.0 || !big.is_finite() {
            return None;
        }
        a.swap(k, piv);
        b.swap(k, piv);
        let row_k = a[k];
        let b_k = b[k];
        for i in (k + 1)..N {
            let f = a[i][k] / row_k[k];
            if f != 0.0 {
                for j in k..N {
                    a[i][j] -= f * row_k[j];
                }
                b[i] -= f * b_k;
            }
        }
    }
    let mut x = [0.0; N];
    for i in (0..N).rev() {
        let mut s = b[i];
        for j in (i + 1)..N {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    if x.iter().all(|v| v.is_finite()) {
        Some(x)
    } else {
        None
    }
}

/// Brent root finder on a bracket with known end values, beside
/// `nush2_units::solve::brent`, which evaluates its own ends; `nush2_numerics`
/// holds neither, so each crate keeps its own.
pub(crate) fn brent<F>(
    mut f: F,
    (mut a, mut fa): (f64, f64),
    (mut b, mut fb): (f64, f64),
    xtol: f64,
    max_iter: usize,
) -> Result<f64>
where
    F: FnMut(f64) -> Result<f64>,
{
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }
    if fa.signum() == fb.signum() {
        return Err(EngineError::NotConverged(format!(
            "brent bracket [{a}, {b}] has equal signs"
        )));
    }
    let (mut c, mut fc) = (a, fa);
    let mut d = b - a;
    let mut e = d;
    for _ in 0..max_iter {
        if fb.signum() == fc.signum() {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        let tol = 2.0 * f64::EPSILON * b.abs() + 0.5 * xtol;
        let m = 0.5 * (c - b);
        if m.abs() <= tol || fb == 0.0 {
            return Ok(b);
        }
        if e.abs() < tol || fa.abs() <= fb.abs() {
            d = m;
            e = m;
        } else {
            let s = fb / fa;
            let (mut p, mut q);
            if a == c {
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                let r = fa / fc;
                let t = fb / fc;
                p = s * (2.0 * m * r * (r - t) - (b - a) * (t - 1.0));
                q = (r - 1.0) * (t - 1.0) * (s - 1.0);
            }
            if p > 0.0 {
                q = -q;
            } else {
                p = -p;
            }
            if 2.0 * p < (3.0 * m * q - (tol * q).abs()).min((e * q).abs()) {
                e = d;
                d = p / q;
            } else {
                d = m;
                e = m;
            }
        }
        a = b;
        fa = fb;
        b += if d.abs() > tol { d } else { tol.copysign(m) };
        fb = f(b)?;
    }
    Err(EngineError::NotConverged(format!(
        "brent after {max_iter} steps, last iterate {b}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lu_recovers_a_known_solution() {
        let a = [[4.0, -2.0, 1.0], [-2.0, 4.0, -2.0], [1.0, -2.0, 4.0]];
        // b = a * [1, -2, 2].
        let x = lu_solve(a, [10.0, -14.0, 13.0]).unwrap();
        assert!((x[0] - 1.0).abs() < 1e-12);
        assert!((x[1] + 2.0).abs() < 1e-12);
        assert!((x[2] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn lu_reports_a_singular_matrix() {
        assert!(lu_solve([[1.0, 2.0], [2.0, 4.0]], [1.0, 2.0]).is_none());
    }

    #[test]
    fn brent_finds_a_root_and_rejects_a_bad_bracket() {
        let f = |x: f64| -> Result<f64> { Ok(x.cos() - x) };
        let r = brent(f, (0.0, 1.0), (1.0, 1.0_f64.cos() - 1.0), 1e-12, 100).unwrap();
        assert!((r - 0.739_085_133_215_160_6).abs() < 1e-10);
        assert!(brent(f, (0.0, 1.0), (0.1, 0.9), 1e-12, 100).is_err());
    }
}
