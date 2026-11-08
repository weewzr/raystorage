//! Rate closure of Khan, Krishnamoorthi and Mahmud, Chem. Eng. Res. Des. 89 (2011)
//! 1600-1608, Eqs 2 to 8, PDF p3 and p4. The set is used whole. Its rate constant,
//! Henry constant and diffusivity were fitted to work together, so none of them may
//! be swapped for a value tied to another activity or diffusivity model
//! (`mea_thermo.md` s7).
//!
//! Three readings of the page need stating.
//!
//! 1. Khan prints no units for Eqs 2 and 8. Eq 2 is taken in m3/kmol/s, because it
//!    then gives 5,918 at 298.15 K against 5,993 from 4.4e11 exp(-5400/T) m3/kmol/s
//!    (Part5b Eq 41, PDF p11). Eq 8 is taken in atm L/mol, the assumption recorded in
//!    the derivations (H.9), because at zero MEA and 298.15 K it gives 29.95 against
//!    29.32 from Aronu Eq 17 in that unit.
//! 2. Eq 4 prints E1 = sqrt(M)/tanh(sqrt(M)) beside M^2 = D k2 C/kL^2 and calls M the
//!    Hatta modulus. Those two statements conflict: taken together they give E1 that
//!    tends to the square root of the Hatta number, which contradicts film theory for
//!    a pseudo-first-order reaction. This module uses E1 = Ha/tanh(Ha) with
//!    Ha^2 = D k2 C/kL^2, which satisfies both the name and the limit. Derivation H.6
//!    writes M with no square, which is this same equation.
//! 3. Khan gives no correlation for D_B, the MEA diffusivity in Eq 4, so callers
//!    supply it.
//!
//! Eqs 5a to 5c are Onda's packing correlations. `G` and `L` must be mass velocities
//! in kg/m2/s for those groups to be dimensionless. Khan's C1 depends on packing size
//! and carries no printed value, so the caller supplies it too.
//!
//! Rate fits in the local literature end near loading 0.50 (`mea_thermo.md` s7), so
//! use above that loading is extrapolation.

use nush2_numerics::{EngineError, Result};

/// R1 stoichiometry, Khan PDF p2: CO2 + 2 RNH2 -> RNHCOO- + RNH3+.
pub const B_STOICH: f64 = 2.0;
/// 1 atm L/mol in Pa m3/mol.
pub const PA_M3_MOL_PER_ATM_L_MOL: f64 = 101.325;
/// Standard gravity, m/s2.
const G: f64 = 9.81;

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

/// Second-order rate constant, m3/kmol/s, Khan Eq 2 (Hikita 1977).
pub fn k2_hikita(t_k: f64) -> Result<f64> {
    positive("t_k", t_k)?;
    Ok(10f64.powf(10.99 - 2152.0 / t_k))
}

/// Henry constant of CO2 as Khan Eq 8 prints it, with `c_mea` in kmol/m3.
/// The value is in atm L/mol on the reading stated in the module notes.
pub fn henry_khan_atm_l_mol(t_k: f64, c_mea_kmol_m3: f64) -> Result<f64> {
    positive("t_k", t_k)?;
    if !(c_mea_kmol_m3.is_finite() && c_mea_kmol_m3 >= 0.0) {
        return Err(EngineError::Range {
            param: "c_mea_kmol_m3".into(),
            msg: format!("{c_mea_kmol_m3} must not be negative"),
        });
    }
    Ok(10f64.powf(5.3 - 0.035 * c_mea_kmol_m3 - 1140.0 / t_k))
}

/// Khan Eq 8 in Pa m3/mol.
pub fn henry_khan_pa_m3_mol(t_k: f64, c_mea_kmol_m3: f64) -> Result<f64> {
    Ok(henry_khan_atm_l_mol(t_k, c_mea_kmol_m3)? * PA_M3_MOL_PER_ATM_L_MOL)
}

/// CO2 diffusivity in water, m2/s, Khan Eq 6b. Dugas Eq 5.15 (PDF p132) prints the
/// same form with the unit.
pub fn d_co2_water_m2_s(t_k: f64) -> Result<f64> {
    positive("t_k", t_k)?;
    Ok(2.35e-6 * (-2119.0 / t_k).exp())
}

/// CO2 diffusivity in solution, m2/s, Khan Eq 6a. `mu_ratio` is solution viscosity
/// over water viscosity, Khan Eq 7, `viscosity::ratio_to_water`.
pub fn d_co2_solution_m2_s(t_k: f64, mu_ratio: f64) -> Result<f64> {
    positive("mu_ratio", mu_ratio)?;
    Ok(d_co2_water_m2_s(t_k)? * mu_ratio.powf(-0.8))
}

/// Hatta number from Khan's Eq 4 group. Inputs in m2/s, m3/kmol/s, kmol/m3, m/s.
pub fn hatta(d_al: f64, k2: f64, c_bl: f64, k_l0: f64) -> Result<f64> {
    positive("d_al", d_al)?;
    positive("k2", k2)?;
    positive("c_bl", c_bl)?;
    positive("k_l0", k_l0)?;
    Ok((d_al * k2 * c_bl).sqrt() / k_l0)
}

/// Pseudo-first-order enhancement E1 = Ha/tanh(Ha). See module note 2.
pub fn e1_of_hatta(ha: f64) -> Result<f64> {
    positive("ha", ha)?;
    Ok(ha / ha.tanh())
}

/// Instantaneous enhancement, Khan Eq 4. `c_bl` and `c_ai` share a unit.
pub fn e_infinity(c_bl: f64, d_b: f64, b: f64, d_al: f64, c_ai: f64) -> Result<f64> {
    positive("c_bl", c_bl)?;
    positive("d_b", d_b)?;
    positive("b", b)?;
    positive("d_al", d_al)?;
    positive("c_ai", c_ai)?;
    Ok(1.0 + c_bl * d_b / (b * d_al * c_ai))
}

/// Wellek explicit enhancement factor, Khan Eq 4.
pub fn enhancement_wellek(e_inf: f64, e1: f64) -> Result<f64> {
    if !(e_inf.is_finite() && e_inf > 1.0 && e1.is_finite() && e1 > 1.0) {
        return Err(EngineError::Range {
            param: "e_inf, e1".into(),
            msg: format!("{e_inf} and {e1} must exceed 1"),
        });
    }
    let inv_inf = (1.0 / (e_inf - 1.0)).powf(1.35);
    let inv_1 = (1.0 / (e1 - 1.0)).powf(1.35);
    Ok(1.0 + 1.0 / (inv_inf + inv_1).powf(1.0 / 1.35))
}

/// Overall absorption rate per unit volume, Khan Eq 3. With `p_pa` in Pa, `k_g` in
/// mol/m2/s/Pa, `henry` in Pa m3/mol, `k_l0` in m/s and `a` in 1/m, the result is
/// mol/m3/s.
pub fn overall_rate(p_pa: f64, k_g: f64, a: f64, henry: f64, k_l0: f64, e: f64) -> Result<f64> {
    positive("k_g", k_g)?;
    positive("a", a)?;
    positive("henry", henry)?;
    positive("k_l0", k_l0)?;
    positive("e", e)?;
    if !(p_pa.is_finite() && p_pa >= 0.0) {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} must not be negative"),
        });
    }
    Ok(p_pa / (1.0 / (k_g * a) + henry / (k_l0 * a * e)))
}

/// Inputs shared by Onda's Eqs 5a to 5c, SI units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OndaGroups {
    /// Gas mass velocity, kg/m2/s.
    pub g: f64,
    /// Liquid mass velocity, kg/m2/s.
    pub l: f64,
    /// Total packing area, 1/m.
    pub a_v: f64,
    /// Nominal packing size, m.
    pub d_p: f64,
    pub mu_g: f64,
    pub mu_l: f64,
    pub rho_g: f64,
    pub rho_l: f64,
    /// CO2 gas diffusivity, m2/s.
    pub d_ag: f64,
    /// CO2 liquid diffusivity, m2/s.
    pub d_al: f64,
    /// Critical surface tension of the packing, N/m.
    pub sigma_c: f64,
    /// Liquid surface tension, N/m.
    pub sigma_l: f64,
}

impl OndaGroups {
    fn check(&self) -> Result<()> {
        for (n, v) in [
            ("g", self.g),
            ("l", self.l),
            ("a_v", self.a_v),
            ("d_p", self.d_p),
            ("mu_g", self.mu_g),
            ("mu_l", self.mu_l),
            ("rho_g", self.rho_g),
            ("rho_l", self.rho_l),
            ("d_ag", self.d_ag),
            ("d_al", self.d_al),
            ("sigma_c", self.sigma_c),
            ("sigma_l", self.sigma_l),
        ] {
            positive(n, v)?;
        }
        Ok(())
    }

    /// Gas film coefficient k_g, mol/m2/s/Pa, Khan Eq 5a.
    pub fn k_g(&self, t_k: f64, c1: f64, r: f64) -> Result<f64> {
        self.check()?;
        positive("t_k", t_k)?;
        positive("c1", c1)?;
        positive("r", r)?;
        let re = self.g / (self.a_v * self.mu_g);
        let sc = self.mu_g / (self.rho_g * self.d_ag);
        Ok(self.a_v * self.d_ag / (r * t_k)
            * c1
            * re.powf(0.7)
            * sc.powf(1.0 / 3.0)
            * (self.a_v * self.d_p).powf(-2.0))
    }

    /// Wetted area over total area, Khan Eq 5c, read from PDF p3.
    pub fn wetted_fraction(&self) -> Result<f64> {
        self.check()?;
        let surface = (self.sigma_c / self.sigma_l).powf(0.75);
        let re = (self.l / (self.a_v * self.mu_l)).powf(0.1);
        let fr = (self.l * self.l * self.a_v / (self.rho_l * self.rho_l * G)).powf(-0.05);
        let we = (self.l * self.l / (self.rho_l * self.sigma_l * self.a_v)).powf(0.2);
        Ok(1.0 - (-1.45 * surface * re * fr * we).exp())
    }

    /// Physical liquid film coefficient k_l^o, m/s, Khan Eq 5b.
    pub fn k_l0(&self) -> Result<f64> {
        let a_w = self.a_v * self.wetted_fraction()?;
        let re = self.l / (a_w * self.mu_l);
        let sc = self.mu_l / (self.rho_l * self.d_al);
        let group = (self.mu_l * G / self.rho_l).powf(1.0 / 3.0);
        Ok(group * 0.0051 * re.powf(2.0 / 3.0) * sc.powf(-0.5) * (self.a_v * self.d_p).powf(0.4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k2_units_agree_with_the_versteeg_form() {
        let k = k2_hikita(298.15).unwrap();
        let v = 4.4e11 * (-5400.0_f64 / 298.15).exp();
        assert!((k / v - 1.0).abs() < 0.03, "{k} against {v}");
    }

    #[test]
    fn henry_units_agree_with_water_at_25c() {
        let h = henry_khan_pa_m3_mol(298.15, 0.0).unwrap();
        let w = crate::aronu::henry_water_co2_pa_m3_mol(298.15);
        assert!((h / w - 1.0).abs() < 0.03, "{h} against {w}");
    }

    #[test]
    fn enhancement_respects_both_limits() {
        // Slow regime: bounded above by the Hatta branch.
        let e = enhancement_wellek(1.0e6, e1_of_hatta(30.0).unwrap()).unwrap();
        assert!((e - 30.0).abs() < 0.1, "{e}");
        // Instantaneous regime: bounded above by E infinity.
        let e = enhancement_wellek(40.0, e1_of_hatta(1.0e4).unwrap()).unwrap();
        assert!(e > 39.0 && e <= 40.0, "{e}");
    }

    #[test]
    fn e1_tends_to_the_hatta_number() {
        assert!((e1_of_hatta(100.0).unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn overall_rate_is_gas_limited_at_large_enhancement() {
        // p k_g a = 1e4 * 1e-5 * 200 = 20 mol/m3/s when the liquid side offers no resistance.
        let r = overall_rate(1.0e4, 1.0e-5, 200.0, 4000.0, 1.0e-4, 1.0e12).unwrap();
        assert!((r - 20.0).abs() < 1e-6, "{r}");
    }

    #[test]
    fn onda_groups_return_plausible_values() {
        let o = OndaGroups {
            g: 1.5,
            l: 8.0,
            a_v: 250.0,
            d_p: 0.025,
            mu_g: 1.8e-5,
            mu_l: 2.0e-3,
            rho_g: 1.1,
            rho_l: 1050.0,
            d_ag: 1.6e-5,
            d_al: 1.2e-9,
            sigma_c: 0.075,
            sigma_l: 0.065,
        };
        let f = o.wetted_fraction().unwrap();
        assert!(f > 0.0 && f < 1.0, "{f}");
        let k = o.k_l0().unwrap();
        assert!(k > 1e-5 && k < 1e-3, "{k}");
        let kg = o.k_g(320.0, 5.23, 8.314).unwrap();
        assert!(kg > 1e-7 && kg < 1e-3, "{kg}");
    }

    #[test]
    fn bad_inputs_are_errors() {
        assert!(k2_hikita(-1.0).is_err());
        assert!(henry_khan_atm_l_mol(300.0, -1.0).is_err());
        assert!(enhancement_wellek(1.0, 2.0).is_err());
        assert!(hatta(0.0, 1.0, 1.0, 1.0).is_err());
    }
}
