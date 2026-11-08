//! Xu and Froment 1989 intrinsic rate equations for Ni/MgAl2O4, Part I, read from
//! page images. Rate forms are Eqs. 3 to 5 (PDF p5), constants are Tables 5 and 6
//! (PDF p7), units are from the notation list (PDF p8) and Part II (PDF p6).
//!
//! Source traps that shape this module:
//!
//! - T2, T3: Hoseinzade and Adams Eqs. 48 to 55 carry KH2O 100 times too small and
//!   k values that silently hold the fresh-catalyst factor 2.246. `XfConstants::check`
//!   refuses any set other than Table 6.
//! - T9: E is listed in kJ/mol while rates are in kmol, so E goes through R in
//!   kJ/mol/K here.
//! - T10: Table 5 has two reference temperatures, 648 K and 823 K.
//! - T14: the text layer corrupts numbers, so nothing here came from it.
//!
//! Neither paper prints K1(T), K2(T) or K3(T). They come from Zhang Table 1
//! through `nush2_props::equilibrium`, and K3 = K1 K2.
//!
//! Tables 5 and 6 sit at the reference activity, an aged catalyst. Rate
//! coefficients need 2.246 for fresh catalyst (PDF p7). `Kinetics::activity` is 1.0
//! until a fit sets it, and it multiplies k1, k2 and k3 only. Adsorption constants
//! are left alone.
//!
//! Validity (PDF p2, Table 1): steam reforming 3 to 15 bar and 773 to 848 K, reverse
//! shift 3 to 10 bar and 573 to 673 K. Use in a reformer at 1144 K and 30 bar is an
//! extrapolation of about 300 K and a factor of 2 in pressure.

use nush2_numerics::{EngineError, Result};

/// R on the kJ/mol/K scale of the printed energies, kept beside
/// `nush2_props::units::R_GAS`. Xu and Froment list R in their
/// notation (Part I, PDF p8) and print no value, so no page fixes the value their
/// constants were fitted with. 8.314e-3 is kept; it lies 5.6e-5 below the exact SI
/// value, which moves k1 at 800 K by about 2e-3 (test
/// `eq_d_13_gas_constants_are_exact_or_explained`).
pub const R_KJ_MOL_K: f64 = 8.314e-3;
/// mol/(kg s) per kmol/(kg h).
const KMOL_H_TO_MOL_S: f64 = 1000.0 / 3600.0;
/// Rates diverge as pH2 goes to zero (PDF p8), so a feed with no H2 needs a seed.
/// Below this partial pressure, bar, the rate is evaluated at this value.
pub const P_H2_FLOOR_BAR: f64 = 1.0e-3;

/// Index of an adsorption constant.
pub const CO: usize = 0;
pub const H2: usize = 1;
pub const CH4: usize = 2;
pub const H2O: usize = 3;

/// One constant set: A exp(-E / R T) for rate coefficients, and
/// A exp(-dH / R T) for adsorption constants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XfConstants {
    /// k1, k2, k3: kmol bar^0.5 / (kg h), kmol / (kg h bar), kmol bar^0.5 / (kg h).
    pub a_k: [f64; 3],
    /// Activation energies, kJ/mol.
    pub e_k: [f64; 3],
    /// KCO, KH2, KCH4 in 1/bar and KH2O dimensionless.
    pub a_ads: [f64; 4],
    /// Adsorption enthalpies, kJ/mol.
    pub dh_ads: [f64; 4],
}

/// XF1 Table 6 (PDF p7) with the energies of Table 5.
///
/// eq: D.13
/// eq: D.14
pub const TABLE6: XfConstants = XfConstants {
    a_k: [4.225e15, 1.955e6, 1.020e15],
    e_k: [240.1, 67.13, 243.9],
    a_ads: [8.23e-5, 6.12e-9, 6.65e-4, 1.77e5],
    dh_ads: [-70.65, -82.90, -38.28, 88.68],
};

/// Values at the reference temperature of XF1 Table 5, PDF p7.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Table5Row {
    pub name: &'static str,
    pub value: f64,
    pub t_ref_k: f64,
}

/// XF1 Table 5 in the order k1, k2, k3, KCO, KH2, KCH4, KH2O.
pub const TABLE5: [Table5Row; 7] = [
    Table5Row {
        name: "k1",
        value: 1.842e-4,
        t_ref_k: 648.0,
    },
    Table5Row {
        name: "k2",
        value: 7.558,
        t_ref_k: 648.0,
    },
    Table5Row {
        name: "k3",
        value: 2.193e-5,
        t_ref_k: 648.0,
    },
    Table5Row {
        name: "KCO",
        value: 40.91,
        t_ref_k: 648.0,
    },
    Table5Row {
        name: "KH2",
        value: 0.02960,
        t_ref_k: 648.0,
    },
    Table5Row {
        name: "KCH4",
        value: 0.1791,
        t_ref_k: 823.0,
    },
    Table5Row {
        name: "KH2O",
        value: 0.4152,
        t_ref_k: 823.0,
    },
];

/// Table 5 value carried to the Table 6 form: value exp(E / R Tr), divided by the
/// printed pre-exponential. The ratio tests the two tables against each other.
pub fn table5_over_table6() -> [f64; 7] {
    let mut out = [0.0; 7];
    for (i, row) in TABLE5.iter().enumerate() {
        let (e, a) = if i < 3 {
            (TABLE6.e_k[i], TABLE6.a_k[i])
        } else {
            (TABLE6.dh_ads[i - 3], TABLE6.a_ads[i - 3])
        };
        out[i] = row.value * (e / (R_KJ_MOL_K * row.t_ref_k)).exp() / a;
    }
    out
}

impl XfConstants {
    /// Refuses any set that differs from Table 6. The two messages name the known
    /// errors of the Hoseinzade and Adams set.
    pub fn check(&self) -> Result<()> {
        let same = |a: f64, b: f64| (a / b - 1.0).abs() < 1.0e-9;
        if !same(self.a_ads[H2O], TABLE6.a_ads[H2O]) {
            return Err(EngineError::Range {
                param: "KH2O".into(),
                msg: format!(
                    "{} refused: KH2O is dimensionless and Table 6 prints 1.77e5 (card D.14, Xu and Froment 1989 Table 6)",
                    self.a_ads[H2O]
                ),
            });
        }
        for i in 0..3 {
            if !same(self.a_k[i], TABLE6.a_k[i]) {
                return Err(EngineError::Range {
                    param: format!("k{}", i + 1),
                    msg: format!(
                        "{} refused: a fresh or fitted factor goes in Kinetics::activity (card D.13)",
                        self.a_k[i]
                    ),
                });
            }
        }
        for i in 0..4 {
            if !same(self.a_ads[i], TABLE6.a_ads[i]) || self.dh_ads[i] != TABLE6.dh_ads[i] {
                return Err(EngineError::Range {
                    param: "adsorption constants".into(),
                    msg: "only the Table 6 set is accepted".into(),
                });
            }
        }
        if self.e_k != TABLE6.e_k {
            return Err(EngineError::Range {
                param: "activation energies".into(),
                msg: "only the Table 6 set is accepted".into(),
            });
        }
        Ok(())
    }
}

/// Partial pressures, bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Partial {
    pub ch4: f64,
    pub h2o: f64,
    pub h2: f64,
    pub co: f64,
    pub co2: f64,
}

/// Rates of reactions I, II and III, mol / (kg s) of catalyst at the set activity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rates {
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
}

impl Rates {
    /// Rate of CH4 consumption, Eq. 4.
    ///
    /// eq: A.4
    /// eq: D.19
    pub fn ch4(&self) -> f64 {
        self.r1 + self.r3
    }

    /// Rate of CO2 formation, Eq. 4.
    ///
    /// eq: A.4
    /// eq: D.19
    pub fn co2(&self) -> f64 {
        self.r2 + self.r3
    }
}

/// Rate model at one activity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kinetics {
    constants: XfConstants,
    activity: f64,
}

impl Default for Kinetics {
    fn default() -> Self {
        Kinetics {
            constants: TABLE6,
            activity: 1.0,
        }
    }
}

impl Kinetics {
    /// Table 6 at a chosen activity: 1.0 until the JAERI A-2 fit sets it.
    ///
    /// eq: D.21
    pub fn with_activity(activity: f64) -> Result<Self> {
        if !(activity > 0.0 && activity.is_finite()) {
            return Err(EngineError::Range {
                param: "activity".into(),
                msg: format!("{activity} must be positive"),
            });
        }
        Ok(Kinetics {
            constants: TABLE6,
            activity,
        })
    }

    /// A rate model from a caller's constants. Anything but Table 6 is refused.
    pub fn from_constants(constants: XfConstants, activity: f64) -> Result<Self> {
        constants.check()?;
        let mut k = Self::with_activity(activity)?;
        k.constants = constants;
        Ok(k)
    }

    pub fn activity(&self) -> f64 {
        self.activity
    }

    /// Eqs. 3: `k1_eq` in bar^2 and `k2_eq` dimensionless.
    ///
    /// eq: D.3
    /// eq: D.9
    /// eq: D.13
    /// eq: D.14
    /// eq: D.15
    /// eq: D.16
    /// eq: D.17
    /// eq: D.18
    pub fn rates(&self, t_k: f64, p: &Partial, k1_eq: f64, k2_eq: f64) -> Result<Rates> {
        if !(t_k > 0.0 && t_k.is_finite()) {
            return Err(EngineError::Range {
                param: "t_k".into(),
                msg: format!("{t_k} K must be positive"),
            });
        }
        let c = &self.constants;
        let rt = R_KJ_MOL_K * t_k;
        let k = |i: usize| c.a_k[i] * (-c.e_k[i] / rt).exp();
        let ads = |i: usize| c.a_ads[i] * (-c.dh_ads[i] / rt).exp();
        let p_h2 = p.h2.max(P_H2_FLOOR_BAR);
        let den =
            1.0 + ads(CO) * p.co + ads(H2) * p_h2 + ads(CH4) * p.ch4 + ads(H2O) * p.h2o / p_h2;
        let den2 = den * den;
        let k3_eq = k1_eq * k2_eq;
        let p25 = p_h2 * p_h2 * p_h2.sqrt();
        let r1 = k(0) / p25 * (p.ch4 * p.h2o - p_h2.powi(3) * p.co / k1_eq) / den2;
        let r2 = k(1) / p_h2 * (p.co * p.h2o - p_h2 * p.co2 / k2_eq) / den2;
        let r3 =
            k(2) / (p25 * p_h2) * (p.ch4 * p.h2o * p.h2o - p_h2.powi(4) * p.co2 / k3_eq) / den2;
        let f = self.activity * KMOL_H_TO_MOL_S;
        Ok(Rates {
            r1: f * r1,
            r2: f * r2,
            r3: f * r3,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Partial {
        // The sample point of derivation card D.15 at 30 bar.
        Partial {
            ch4: 0.15 * 30.0,
            h2o: 0.45 * 30.0,
            h2: 0.25 * 30.0,
            co: 0.03 * 30.0,
            co2: 0.12 * 30.0,
        }
    }

    #[test]
    fn eq_d_15_den_squared_matches_the_hand_value() {
        // Derivation card D.15 computes DEN^2 = 82.7 at 1000 K for this composition.
        let c = TABLE6;
        let rt = R_KJ_MOL_K * 1000.0;
        let ads = |i: usize| c.a_ads[i] * (-c.dh_ads[i] / rt).exp();
        let p = sample();
        let den =
            1.0 + ads(CO) * p.co + ads(H2) * p.h2 + ads(CH4) * p.ch4 + ads(H2O) * p.h2o / p.h2;
        assert!((den * den - 82.7).abs() < 0.5, "{}", den * den);
    }

    #[test]
    fn activity_scales_every_rate() {
        let (k1, k2) = (1.0e2, 1.5);
        let base = Kinetics::default()
            .rates(1000.0, &sample(), k1, k2)
            .unwrap();
        let half = Kinetics::with_activity(0.5)
            .unwrap()
            .rates(1000.0, &sample(), k1, k2)
            .unwrap();
        assert!((half.r1 / base.r1 - 0.5).abs() < 1e-12);
        assert!((half.r2 / base.r2 - 0.5).abs() < 1e-12);
        assert!((half.r3 / base.r3 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn eq_d_17_and_d_18_rates_vanish_at_equilibrium() {
        let p = sample();
        let k1 = p.h2.powi(3) * p.co / (p.ch4 * p.h2o);
        let k2 = p.h2 * p.co2 / (p.co * p.h2o);
        let r = Kinetics::default().rates(1000.0, &p, k1, k2).unwrap();
        assert!(r.r1.abs() < 1e-9 && r.r2.abs() < 1e-9 && r.r3.abs() < 1e-9);
    }

    #[test]
    fn a_feed_with_no_hydrogen_stays_finite() {
        let mut p = sample();
        p.h2 = 0.0;
        let r = Kinetics::default().rates(800.0, &p, 1.0, 5.0).unwrap();
        assert!(r.r1.is_finite() && r.r3.is_finite());
    }
}
