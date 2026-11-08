//! Small numerical routines for the gas units. `nush2_numerics` holds no Brent,
//! LU or Runge-Kutta routine yet, so these stay private to this crate.

use nush2_numerics::{EngineError, Result};

/// Brent root finder on a bracket. A bracket with no sign change is an error.
pub(crate) fn brent<F>(mut f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> Result<f64>
where
    F: FnMut(f64) -> Result<f64>,
{
    let mut fa = f(a)?;
    let mut fb = f(b)?;
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }
    if !(fa.is_finite() && fb.is_finite()) || fa * fb > 0.0 {
        return Err(EngineError::Range {
            param: "bracket".into(),
            msg: format!("f({a}) = {fa} and f({b}) = {fb} do not bracket a root"),
        });
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = b - a;
    let mut e = d;
    for _ in 0..max_iter {
        if fb * fc > 0.0 {
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
        let tol1 = 2.0 * f64::EPSILON * b.abs() + 0.5 * tol;
        let m = 0.5 * (c - b);
        if m.abs() <= tol1 || fb == 0.0 {
            return Ok(b);
        }
        if e.abs() >= tol1 && fa.abs() > fb.abs() {
            let s = fb / fa;
            let (mut p, mut q);
            if a == c {
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                let q0 = fa / fc;
                let r = fb / fc;
                p = s * (2.0 * m * q0 * (q0 - r) - (b - a) * (r - 1.0));
                q = (q0 - 1.0) * (r - 1.0) * (s - 1.0);
            }
            if p > 0.0 {
                q = -q;
            }
            p = p.abs();
            let lim = (3.0 * m * q - (tol1 * q).abs()).min((e * q).abs());
            if 2.0 * p < lim {
                e = d;
                d = p / q;
            } else {
                d = m;
                e = m;
            }
        } else {
            d = m;
            e = m;
        }
        a = b;
        fa = fb;
        b += if d.abs() > tol1 {
            d
        } else if m > 0.0 {
            tol1
        } else {
            -tol1
        };
        fb = f(b)?;
    }
    Err(EngineError::NotConverged(format!(
        "Brent after {max_iter} steps, last point {b}"
    )))
}

/// Solve `a x = b` by LU with row scaling and partial pivoting. `None` for a
/// singular or non-finite matrix.
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
        for (i, row) in a.iter().enumerate().skip(k + 1) {
            if row[k].abs() > big {
                big = row[k].abs();
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
    Some(x)
}

/// Dormand-Prince 5(4) on one state from `x0` to `x1`. Step control keeps the
/// local error below `tol`.
pub(crate) fn rk45<F>(
    mut f: F,
    x0: f64,
    y0: f64,
    x1: f64,
    tol: f64,
    max_steps: usize,
) -> Result<f64>
where
    F: FnMut(f64, f64) -> Result<f64>,
{
    const C: [f64; 6] = [0.2, 0.3, 0.8, 8.0 / 9.0, 1.0, 1.0];
    const A: [[f64; 6]; 6] = [
        [0.2, 0.0, 0.0, 0.0, 0.0, 0.0],
        [3.0 / 40.0, 9.0 / 40.0, 0.0, 0.0, 0.0, 0.0],
        [44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0, 0.0, 0.0, 0.0],
        [
            19372.0 / 6561.0,
            -25360.0 / 2187.0,
            64448.0 / 6561.0,
            -212.0 / 729.0,
            0.0,
            0.0,
        ],
        [
            9017.0 / 3168.0,
            -355.0 / 33.0,
            46732.0 / 5247.0,
            49.0 / 176.0,
            -5103.0 / 18656.0,
            0.0,
        ],
        [
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
        ],
    ];
    const E: [f64; 7] = [
        71.0 / 57600.0,
        0.0,
        -71.0 / 16695.0,
        71.0 / 1920.0,
        -17253.0 / 339200.0,
        22.0 / 525.0,
        -1.0 / 40.0,
    ];
    let span = x1 - x0;
    if span == 0.0 {
        return Ok(y0);
    }
    let mut x = x0;
    let mut y = y0;
    let mut h = span / 100.0;
    for _ in 0..max_steps {
        if (x1 - x).abs() <= 1.0e-13 * span.abs() {
            return Ok(y);
        }
        if (x + h - x1) * span.signum() > 0.0 {
            h = x1 - x;
        }
        let mut k = [0.0; 7];
        k[0] = f(x, y)?;
        for s in 0..6 {
            let mut yt = y;
            for (j, kj) in k.iter().enumerate().take(s + 1) {
                yt += h * A[s][j] * kj;
            }
            k[s + 1] = f(x + C[s] * h, yt)?;
        }
        let y5 = y + h
            * (35.0 / 384.0 * k[0] + 500.0 / 1113.0 * k[2] + 125.0 / 192.0 * k[3]
                - 2187.0 / 6784.0 * k[4]
                + 11.0 / 84.0 * k[5]);
        let err = (h * E.iter().zip(&k).map(|(e, v)| e * v).sum::<f64>()).abs();
        if !err.is_finite() {
            h *= 0.25;
            continue;
        }
        if err <= tol {
            x += h;
            y = y5;
        }
        let fac = if err > 0.0 {
            (0.9 * (tol / err).powf(0.2)).clamp(0.2, 5.0)
        } else {
            5.0
        };
        h *= fac;
    }
    Err(EngineError::NotConverged(format!(
        "RK45 after {max_steps} steps at x = {x}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brent_finds_a_cubic_root() {
        let r = brent(|x| Ok(x * x * x - 2.0), 0.0, 2.0, 1e-12, 100).unwrap();
        assert!((r - 2.0f64.powf(1.0 / 3.0)).abs() < 1e-10);
    }

    #[test]
    fn brent_needs_a_bracket() {
        assert!(brent(|x| Ok(x * x + 1.0), -1.0, 1.0, 1e-12, 100).is_err());
    }

    #[test]
    fn lu_solves_and_refuses_a_singular_matrix() {
        let x = lu_solve([[2.0, 1.0], [1.0, 3.0]], [3.0, 5.0]).unwrap();
        assert!((x[0] - 0.8).abs() < 1e-12 && (x[1] - 1.4).abs() < 1e-12);
        assert!(lu_solve([[1.0, 2.0], [2.0, 4.0]], [1.0, 1.0]).is_none());
    }

    #[test]
    fn rk45_reproduces_an_exponential() {
        let y = rk45(|_, y| Ok(-3.0 * y), 0.0, 1.0, 2.0, 1e-10, 10_000).unwrap();
        assert!((y - (-6.0f64).exp()).abs() < 1e-8);
    }
}
