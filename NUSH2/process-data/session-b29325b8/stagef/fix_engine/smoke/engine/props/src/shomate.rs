//! NIST Shomate polynomials for ideal-gas heat capacity, enthalpy and entropy.
//!
//! The form is the one printed in the NIST Chemistry WebBook, with t = T / 1000:
//!
//! - Cp = A + B t + C t^2 + D t^3 + E / t^2, J/mol/K
//! - H(T) - H(298.15 K) = A t + B t^2 / 2 + C t^3 / 3 + D t^4 / 4 - E / t + F - H, kJ/mol
//! - S(T) = A ln t + B t + C t^2 / 2 + D t^3 / 3 - E / (2 t^2) + G, J/mol/K
//!
//! The H coefficient is the standard enthalpy of formation at 298.15 K in kJ/mol.
//! The public API here is SI, so enthalpies leave this module in J/mol.

use nush2_numerics::{EngineError, Result};

/// One row of Shomate coefficients, as printed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shomate {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
    pub g: f64,
    pub h: f64,
}

impl Shomate {
    /// Heat capacity, J/mol/K.
    ///
    /// eq: A.9
    pub fn cp_j_mol_k(&self, t_k: f64) -> f64 {
        let t = t_k / 1000.0;
        self.a + self.b * t + self.c * t * t + self.d * t * t * t + self.e / (t * t)
    }

    /// Sensible enthalpy H(T) - H(298.15 K), J/mol.
    ///
    /// eq: A.9
    pub fn dh_j_mol(&self, t_k: f64) -> f64 {
        let t = t_k / 1000.0;
        let kj = self.a * t
            + self.b * t * t / 2.0
            + self.c * t * t * t / 3.0
            + self.d * t * t * t * t / 4.0
            - self.e / t
            + self.f
            - self.h;
        kj * 1000.0
    }

    /// Standard entropy at 1 bar, J/mol/K.
    pub fn s_j_mol_k(&self, t_k: f64) -> f64 {
        let t = t_k / 1000.0;
        self.a * t.ln() + self.b * t + self.c * t * t / 2.0 + self.d * t * t * t / 3.0
            - self.e / (2.0 * t * t)
            + self.g
    }

    /// Standard enthalpy of formation at 298.15 K, J/mol.
    pub fn hf298_j_mol(&self) -> f64 {
        self.h * 1000.0
    }
}

/// Shomate row with the temperature interval it was fitted on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShomateRange {
    pub t_min_k: f64,
    pub t_max_k: f64,
    pub coeff: Shomate,
}

impl ShomateRange {
    pub fn contains(&self, t_k: f64) -> bool {
        t_k >= self.t_min_k && t_k <= self.t_max_k
    }
}

/// Ordered Shomate rows of one species. A temperature on a shared limit takes
/// the lower row. Anything outside every row is a range error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShomateTable {
    pub name: &'static str,
    pub rows: &'static [ShomateRange],
}

impl ShomateTable {
    /// The row that covers `t_k`.
    pub fn row(&self, t_k: f64) -> Result<&'static Shomate> {
        self.rows
            .iter()
            .find(|r| r.contains(t_k))
            .map(|r| &r.coeff)
            .ok_or_else(|| {
                let (lo, hi) = self.limits();
                EngineError::Range {
                    param: format!("t_k for {}", self.name),
                    msg: format!("{t_k} K lies outside the Shomate range {lo} to {hi} K"),
                }
            })
    }

    /// Lowest and highest temperature covered, K. A table with no rows has
    /// inverted limits, so every lookup on it is a range error.
    pub fn limits(&self) -> (f64, f64) {
        let lo = self.rows.first().map_or(f64::INFINITY, |r| r.t_min_k);
        let hi = self.rows.last().map_or(f64::NEG_INFINITY, |r| r.t_max_k);
        (lo, hi)
    }

    pub fn cp_j_mol_k(&self, t_k: f64) -> Result<f64> {
        Ok(self.row(t_k)?.cp_j_mol_k(t_k))
    }

    pub fn dh_j_mol(&self, t_k: f64) -> Result<f64> {
        Ok(self.row(t_k)?.dh_j_mol(t_k))
    }

    pub fn s_j_mol_k(&self, t_k: f64) -> Result<f64> {
        Ok(self.row(t_k)?.s_j_mol_k(t_k))
    }
}

/// Tabulated ideal-gas heat capacity, for species the WebBook gives no Shomate
/// row for. Cp is linear between nodes, so the enthalpy integral is the
/// trapezoid sum and is exact for this interpolant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpTable {
    pub name: &'static str,
    /// Standard enthalpy of formation at 298.15 K, J/mol.
    pub hf298_j_mol: f64,
    /// (T in K, Cp in J/mol/K), rising in T. The first node is 298.15 K.
    pub points: &'static [(f64, f64)],
}

impl CpTable {
    fn out_of_range(&self, t_k: f64) -> EngineError {
        let lo = self.points.first().map_or(f64::NAN, |p| p.0);
        let hi = self.points.last().map_or(f64::NAN, |p| p.0);
        EngineError::Range {
            param: format!("t_k for {}", self.name),
            msg: format!("{t_k} K lies outside the Cp table {lo} to {hi} K"),
        }
    }

    /// Heat capacity by linear interpolation, J/mol/K.
    pub fn cp_j_mol_k(&self, t_k: f64) -> Result<f64> {
        for w in self.points.windows(2) {
            let ((t0, c0), (t1, c1)) = (w[0], w[1]);
            if t_k >= t0 && t_k <= t1 {
                return Ok(c0 + (c1 - c0) * (t_k - t0) / (t1 - t0));
            }
        }
        Err(self.out_of_range(t_k))
    }

    /// Sensible enthalpy H(T) - H(298.15 K), J/mol.
    pub fn dh_j_mol(&self, t_k: f64) -> Result<f64> {
        let mut area = 0.0;
        for w in self.points.windows(2) {
            let ((t0, c0), (t1, c1)) = (w[0], w[1]);
            if t_k >= t1 {
                area += 0.5 * (c0 + c1) * (t1 - t0);
            } else if t_k >= t0 {
                let c = c0 + (c1 - c0) * (t_k - t0) / (t1 - t0);
                return Ok(area + 0.5 * (c0 + c) * (t_k - t0));
            }
        }
        match self.points.last() {
            Some(&(t_last, _)) if t_k == t_last => Ok(area),
            _ => Err(self.out_of_range(t_k)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR: CpTable = CpTable {
        name: "linear",
        hf298_j_mol: 0.0,
        points: &[(298.15, 30.0), (400.0, 40.0), (500.0, 40.0)],
    };

    #[test]
    fn cp_table_integrates_exactly_for_its_interpolant() {
        let t = 350.0;
        let c = LINEAR.cp_j_mol_k(t).unwrap();
        let want = 0.5 * (30.0 + c) * (t - 298.15);
        assert!((LINEAR.dh_j_mol(t).unwrap() - want).abs() < 1e-9);
        assert_eq!(LINEAR.dh_j_mol(298.15).unwrap(), 0.0);
        let full = 0.5 * 70.0 * (400.0 - 298.15) + 40.0 * 100.0;
        assert!((LINEAR.dh_j_mol(500.0).unwrap() - full).abs() < 1e-9);
    }

    #[test]
    fn cp_table_refuses_points_outside_its_nodes() {
        assert!(LINEAR.cp_j_mol_k(298.0).is_err());
        assert!(LINEAR.dh_j_mol(500.1).is_err());
        assert!(LINEAR.dh_j_mol(f64::NAN).is_err());
    }

    #[test]
    fn empty_table_is_a_range_error() {
        let t = ShomateTable {
            name: "empty",
            rows: &[],
        };
        assert!(t.cp_j_mol_k(500.0).is_err());
    }
}
