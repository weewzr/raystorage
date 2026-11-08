//! Equilibrium constants and the two-extent reforming equilibrium.
//!
//! Reactions:
//!
//! - R1, reforming: CH4 + H2O = CO + 3 H2, K1 in bar^2
//! - R2, shift: CO + H2O = CO2 + H2, K2 dimensionless
//!
//! Zhang et al. 2021, Table 1 (PDF p5, printed p1362) gives
//! K1 = 1.198e11 exp(-26830/T) MPa^2 and K2 = 1.767e-2 exp(4400/T). K1 is 1.198e13
//! bar^2 once the unit is converted. Their Eqs. 11 and 12 carry errors, so only
//! Table 1 is used here.
//!
//! `KSource::NistGibbs` builds the same constants from the Gibbs energies in
//! `species`, on a 1 bar standard state, so it can be printed beside Zhang.
//! Zhang stays the default. The departure of Zhang from the NIST route depends
//! strongly on temperature: within 6% from 680 K to 1144 K, and 17 to 20% low on
//! K2 at the low-temperature shift (464 K to 486 K). See the props test suite.
//!
//! The equilibrium model is ideal gas at the outlet temperature and pressure,
//! with C2+ converted fully to CO and H2 ahead of the two extents.

use crate::mixture::{self, Flows};
use crate::species::{self, Sp};
use crate::units::{BAR_PA, R_GAS};
use nush2_numerics::{EngineError, Result};

const T_MIN_K: f64 = 298.15;
const T_MAX_K: f64 = 1500.0;
const NEWTON_ITERS: usize = 100;
const BISECTIONS: usize = 200;
/// Residual tolerance on the ln K pair.
const RESIDUAL_TOL: f64 = 1.0e-10;

fn check_t(t_k: f64) -> Result<()> {
    if (T_MIN_K..=T_MAX_K).contains(&t_k) {
        Ok(())
    } else {
        Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_k} K lies outside {T_MIN_K} to {T_MAX_K} K"),
        })
    }
}

/// Zhang Table 1 K1, bar^2.
///
/// eq: D.7
pub fn k1_zhang_bar2(t_k: f64) -> Result<f64> {
    check_t(t_k)?;
    Ok(1.198e13 * (-26830.0 / t_k).exp())
}

/// Zhang Table 1 K2.
///
/// eq: D.8
pub fn k2_zhang(t_k: f64) -> Result<f64> {
    check_t(t_k)?;
    Ok(1.767e-2 * (4400.0 / t_k).exp())
}

/// K1 from NIST Gibbs energies on a 1 bar standard state, bar^2.
///
/// eq: D.12
pub fn k1_nist_bar2(t_k: f64) -> Result<f64> {
    check_t(t_k)?;
    let g = species::g_j_mol;
    let dg = g(Sp::Co, t_k)? + 3.0 * g(Sp::H2, t_k)? - g(Sp::Ch4, t_k)? - g(Sp::H2o, t_k)?;
    Ok((-dg / (R_GAS * t_k)).exp())
}

/// K2 from NIST Gibbs energies.
///
/// eq: D.12
pub fn k2_nist(t_k: f64) -> Result<f64> {
    check_t(t_k)?;
    let g = species::g_j_mol;
    let dg = g(Sp::Co2, t_k)? + g(Sp::H2, t_k)? - g(Sp::Co, t_k)? - g(Sp::H2o, t_k)?;
    Ok((-dg / (R_GAS * t_k)).exp())
}

/// Which K(T) pair a solve uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KSource {
    #[default]
    Zhang,
    NistGibbs,
}

impl KSource {
    /// (K1 in bar^2, K2).
    pub fn pair(self, t_k: f64) -> Result<(f64, f64)> {
        match self {
            KSource::Zhang => Ok((k1_zhang_bar2(t_k)?, k2_zhang(t_k)?)),
            KSource::NistGibbs => Ok((k1_nist_bar2(t_k)?, k2_nist(t_k)?)),
        }
    }
}

/// Which solver reached the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Newton,
    NestedBisection,
}

/// Equilibrium outlet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReformEq {
    /// Outlet flows, mol/s.
    pub n: Flows,
    /// Extent of R1 after C2+ conversion, mol/s.
    pub xi1: f64,
    /// Extent of R2 after C2+ conversion, mol/s.
    pub xi2: f64,
    /// Share of the hydrocarbon carbon fed that left as CO or CO2.
    pub conversion: f64,
    pub method: Method,
}

/// Flows of the five reacting species after C2+ conversion, mol/s.
#[derive(Debug, Clone, Copy)]
struct Basis {
    ch4: f64,
    h2o: f64,
    h2: f64,
    co: f64,
    co2: f64,
    inert: f64,
}

impl Basis {
    /// CH4, H2O, H2, CO, CO2 at the extents.
    ///
    /// eq: D.1
    /// eq: D.2
    /// eq: D.6
    fn amounts(&self, x1: f64, x2: f64) -> [f64; 5] {
        [
            self.ch4 - x1,
            self.h2o - x1 - x2,
            self.h2 + 3.0 * x1 + x2,
            self.co + x1 - x2,
            self.co2 + x2,
        ]
    }

    /// Open interval of x1 that leaves room for every species.
    fn x1_bounds(&self) -> (f64, f64) {
        let lo = (-self.co - self.co2)
            .max(-(self.h2 + self.co) / 4.0)
            .max(-(self.h2o + self.h2) / 2.0);
        let hi = self.ch4.min(self.h2o + self.co2);
        (lo, hi)
    }

    /// Open interval of x2 at a given x1.
    fn x2_bounds(&self, x1: f64) -> (f64, f64) {
        let lo = (-self.co2).max(-(self.h2 + 3.0 * x1));
        let hi = (self.co + x1).min(self.h2o - x1);
        (lo, hi)
    }

    /// eq: D.6
    fn total(&self, a: &[f64; 5]) -> f64 {
        a.iter().sum::<f64>() + self.inert
    }

    /// ln Q1 - ln K1 and ln Q2 - ln K2.
    ///
    /// eq: D.10
    fn residual(&self, x1: f64, x2: f64, ln_k: (f64, f64), ln_p_bar: f64) -> [f64; 2] {
        let a = self.amounts(x1, x2);
        let [ch4, h2o, h2, co, co2] = a.map(f64::ln);
        let f1 = co + 3.0 * h2 - ch4 - h2o + 2.0 * (ln_p_bar - self.total(&a).ln()) - ln_k.0;
        let f2 = co2 + h2 - co - h2o - ln_k.1;
        [f1, f2]
    }

    /// Newton on the two ln K residuals. `None` hands over to the bisection nest.
    fn newton(&self, ln_k: (f64, f64), ln_p_bar: f64) -> Option<(f64, f64)> {
        let (lo, hi) = self.x1_bounds();
        let mut x1 = 0.5 * (lo + hi);
        let (a2, b2) = self.x2_bounds(x1);
        let mut x2 = 0.5 * (a2 + b2);
        let norm = |f: &[f64; 2]| f[0].abs().max(f[1].abs());
        let mut f = self.residual(x1, x2, ln_k, ln_p_bar);
        for _ in 0..NEWTON_ITERS {
            if norm(&f) < RESIDUAL_TOL {
                return Some((x1, x2));
            }
            let [ch4, h2o, h2, co, co2] = self.amounts(x1, x2);
            let j11 = 1.0 / co + 9.0 / h2 + 1.0 / ch4 + 1.0 / h2o
                - 4.0 / (self.total(&[ch4, h2o, h2, co, co2]));
            let j12 = -1.0 / co + 3.0 / h2 + 1.0 / h2o;
            let j22 = 1.0 / co2 + 1.0 / h2 + 1.0 / co + 1.0 / h2o;
            let det = j11 * j22 - j12 * j12;
            if det.abs() < 1.0e-300 || !det.is_finite() {
                return None;
            }
            let d1 = -(j22 * f[0] - j12 * f[1]) / det;
            let d2 = -(j11 * f[1] - j12 * f[0]) / det;
            let mut step = 1.0;
            let mut accepted = false;
            for _ in 0..40 {
                let (t1, t2) = (x1 + step * d1, x2 + step * d2);
                if self.amounts(t1, t2).iter().all(|v| *v > 0.0) {
                    let trial = self.residual(t1, t2, ln_k, ln_p_bar);
                    if norm(&trial) < norm(&f) {
                        x1 = t1;
                        x2 = t2;
                        f = trial;
                        accepted = true;
                        break;
                    }
                }
                step *= 0.5;
            }
            if !accepted {
                return None;
            }
        }
        None
    }

    /// For a given x1, the x2 that satisfies R2. The residual rises with x2.
    fn inner_x2(&self, x1: f64, ln_k: (f64, f64), ln_p_bar: f64) -> f64 {
        let (mut lo, mut hi) = self.x2_bounds(x1);
        for _ in 0..BISECTIONS {
            let mid = 0.5 * (lo + hi);
            if self.residual(x1, mid, ln_k, ln_p_bar)[1] > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// Bisection on x1 around the inner x2 solve.
    fn nested(&self, ln_k: (f64, f64), ln_p_bar: f64) -> (f64, f64) {
        let (mut lo, mut hi) = self.x1_bounds();
        for _ in 0..BISECTIONS {
            let mid = 0.5 * (lo + hi);
            let x2 = self.inner_x2(mid, ln_k, ln_p_bar);
            if self.residual(mid, x2, ln_k, ln_p_bar)[0] > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let x1 = 0.5 * (lo + hi);
        (x1, self.inner_x2(x1, ln_k, ln_p_bar))
    }
}

/// Convert every C2+ species to CO and H2 with feed steam.
/// C_n H_(2n+2) + n H2O = n CO + (2n+1) H2.
///
/// eq: D.4
fn basis(feed: &Flows) -> Result<Basis> {
    mixture::total(feed)?;
    let mut b = Basis {
        ch4: feed[Sp::Ch4.index()],
        h2o: feed[Sp::H2o.index()],
        h2: feed[Sp::H2.index()],
        co: feed[Sp::Co.index()],
        co2: feed[Sp::Co2.index()],
        inert: feed[Sp::N2.index()],
    };
    for sp in [Sp::C2h6, Sp::C3h8, Sp::Nc4h10, Sp::Nc5h12] {
        let v = feed[sp.index()];
        let c = f64::from(sp.atoms()[0]);
        b.h2o -= c * v;
        b.co += c * v;
        b.h2 += (2.0 * c + 1.0) * v;
    }
    if b.h2o <= 0.0 {
        return Err(EngineError::Range {
            param: "feed H2O".into(),
            msg: "the steam fed cannot convert the C2+ species".into(),
        });
    }
    let (lo, hi) = b.x1_bounds();
    if lo >= hi {
        return Err(EngineError::Range {
            param: "feed".into(),
            msg: "the feed cannot hold CH4, H2O, H2, CO and CO2 together".into(),
        });
    }
    Ok(b)
}

fn outlet(feed: &Flows, b: &Basis, x1: f64, x2: f64, method: Method) -> Result<ReformEq> {
    let [ch4, h2o, h2, co, co2] = b.amounts(x1, x2);
    let mut n = [0.0; crate::species::NSP];
    n[Sp::Ch4.index()] = ch4;
    n[Sp::H2o.index()] = h2o;
    n[Sp::H2.index()] = h2;
    n[Sp::Co.index()] = co;
    n[Sp::Co2.index()] = co2;
    n[Sp::N2.index()] = b.inert;
    let hydrocarbon_c = feed[Sp::Ch4.index()]
        + [Sp::C2h6, Sp::C3h8, Sp::Nc4h10, Sp::Nc5h12]
            .iter()
            .map(|s| f64::from(s.atoms()[0]) * feed[s.index()])
            .sum::<f64>();
    let conversion = if hydrocarbon_c > 0.0 {
        1.0 - ch4 / hydrocarbon_c
    } else {
        0.0
    };
    Ok(ReformEq {
        n,
        xi1: x1,
        xi2: x2,
        conversion,
        method,
    })
}

/// Equilibrium outlet of `feed` at (T, p). Newton runs first, with the bisection
/// nest as its fallback, and a result that misses the tolerance is an error.
///
/// eq: D.11
/// eq: D.33
pub fn reform(feed: &Flows, t_k: f64, p_pa: f64, source: KSource) -> Result<ReformEq> {
    if !(p_pa > 0.0 && p_pa.is_finite()) {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} Pa must be positive"),
        });
    }
    let (k1, k2) = source.pair(t_k)?;
    let b = basis(feed)?;
    let ln_k = (k1.ln(), k2.ln());
    let ln_p_bar = (p_pa / BAR_PA).ln();
    if let Some((x1, x2)) = b.newton(ln_k, ln_p_bar) {
        return outlet(feed, &b, x1, x2, Method::Newton);
    }
    let (x1, x2) = b.nested(ln_k, ln_p_bar);
    let f = b.residual(x1, x2, ln_k, ln_p_bar);
    if f[0].abs().max(f[1].abs()) < 1.0e3 * RESIDUAL_TOL {
        outlet(feed, &b, x1, x2, Method::NestedBisection)
    } else {
        Err(EngineError::NotConverged(format!(
            "reforming equilibrium at {t_k} K, residuals {f:?}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::NSP;

    fn feed(pairs: &[(Sp, f64)]) -> Flows {
        let mut n = [0.0; NSP];
        for (sp, v) in pairs {
            n[sp.index()] = *v;
        }
        n
    }

    fn ln_k_at(t: f64) -> (f64, f64) {
        let (k1, k2) = KSource::Zhang.pair(t).unwrap();
        (k1.ln(), k2.ln())
    }

    #[test]
    fn unit_conversion_of_k1_follows_zhang_table_1() {
        let t = 1000.0;
        let mpa2 = 1.198e11 * (-26830.0f64 / t).exp();
        assert!((k1_zhang_bar2(t).unwrap() / (mpa2 * 100.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn eq_d_10_solution_satisfies_both_mass_action_laws() {
        let f = feed(&[(Sp::Ch4, 1.0), (Sp::H2o, 3.0)]);
        let (t, p) = (1144.15, 30.3e5);
        let r = reform(&f, t, p, KSource::Zhang).unwrap();
        assert_eq!(r.method, Method::Newton);
        let y = mixture::mole_fractions(&r.n).unwrap();
        let pp = |s: Sp| y[s.index()] * p / BAR_PA;
        let q1 = pp(Sp::Co) * pp(Sp::H2).powi(3) / (pp(Sp::Ch4) * pp(Sp::H2o));
        let q2 = pp(Sp::Co2) * pp(Sp::H2) / (pp(Sp::Co) * pp(Sp::H2o));
        assert!((q1 / k1_zhang_bar2(t).unwrap() - 1.0).abs() < 1e-9);
        assert!((q2 / k2_zhang(t).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn elements_close_with_c2_plus_and_inerts_in_the_feed() {
        let f = feed(&[
            (Sp::Ch4, 0.93571),
            (Sp::C2h6, 0.03749),
            (Sp::C3h8, 0.00920),
            (Sp::Nc4h10, 0.00260),
            (Sp::Nc5h12, 0.00052),
            (Sp::N2, 0.01200),
            (Sp::Co2, 0.00250),
            (Sp::H2o, 3.16137),
            (Sp::H2, 0.2),
        ]);
        let r = reform(&f, 1144.15, 30.3e5, KSource::Zhang).unwrap();
        let (a, b) = (
            mixture::elements(&f).unwrap(),
            mixture::elements(&r.n).unwrap(),
        );
        for k in 0..4 {
            assert!((a[k] - b[k]).abs() < 1e-9 * a[k].max(1.0), "element {k}");
        }
        assert_eq!(r.n[Sp::C2h6.index()], 0.0);
        assert_eq!(r.n[Sp::Nc5h12.index()], 0.0);
    }

    #[test]
    fn bisection_nest_agrees_with_newton() {
        let f = feed(&[
            (Sp::Ch4, 1.0),
            (Sp::H2o, 3.0),
            (Sp::Co2, 0.1),
            (Sp::H2, 0.2),
        ]);
        let b = basis(&f).unwrap();
        for (t, p_bar) in [(1144.15, 30.3), (1073.15, 41.9), (900.0, 5.0)] {
            let ln_k = ln_k_at(t);
            let (n1, n2) = b.newton(ln_k, f64::ln(p_bar)).unwrap();
            let (m1, m2) = b.nested(ln_k, f64::ln(p_bar));
            assert!((n1 - m1).abs() < 1e-9, "x1 at {t}");
            assert!((n2 - m2).abs() < 1e-9, "x2 at {t}");
        }
    }

    #[test]
    fn a_recycle_with_no_methane_can_run_backwards() {
        // Syngas with no CH4 forms some at low temperature, so x1 goes negative.
        let f = feed(&[(Sp::H2, 3.0), (Sp::Co, 1.0), (Sp::H2o, 0.5), (Sp::Co2, 0.1)]);
        let r = reform(&f, 800.0, 30.0e5, KSource::Zhang).unwrap();
        assert!(r.xi1 < 0.0);
        assert!(r.n[Sp::Ch4.index()] > 0.0);
    }

    #[test]
    fn nist_route_solves_as_well() {
        let f = feed(&[(Sp::Ch4, 1.0), (Sp::H2o, 3.0)]);
        let z = reform(&f, 1144.15, 30.3e5, KSource::Zhang).unwrap();
        let n = reform(&f, 1144.15, 30.3e5, KSource::NistGibbs).unwrap();
        assert!((z.conversion - n.conversion).abs() < 0.01);
    }

    #[test]
    fn bad_inputs_are_refused() {
        let good = feed(&[(Sp::Ch4, 1.0), (Sp::H2o, 3.0)]);
        assert!(reform(&good, 200.0, 1.0e5, KSource::Zhang).is_err());
        assert!(reform(&good, f64::NAN, 1.0e5, KSource::Zhang).is_err());
        assert!(reform(&good, 1000.0, 0.0, KSource::Zhang).is_err());
        // Enough C2+ to use up all the steam.
        let dry = feed(&[(Sp::C3h8, 1.0), (Sp::H2o, 2.0)]);
        assert!(reform(&dry, 1000.0, 1.0e5, KSource::Zhang).is_err());
        // Nothing to react.
        let inert = feed(&[(Sp::N2, 1.0)]);
        assert!(reform(&inert, 1000.0, 1.0e5, KSource::Zhang).is_err());
    }
}
