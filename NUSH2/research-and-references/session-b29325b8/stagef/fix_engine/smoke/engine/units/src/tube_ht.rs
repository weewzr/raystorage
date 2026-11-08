//! Heat transfer of the reformer tube, from JAERI-Tech 2005-031 Appendix A-1
//! (PDF p93 to p98, read from page images).
//!
//! - He side: Knudsen finned annulus, A.1.1, PDF p93.
//! - Bed side: Yagi-Kunii wall coefficient with radiation, A.1.2, PDF p94 to p96,
//!   with the annular equivalent diameter JAERI uses. JAERI fitted the curves of
//!   its source to the polynomials printed there, and these are those polynomials.
//! - Overall coefficient on the finned outer area, A.1.3, PDF p97.
//!
//! Worked numbers for 2002-11-28 are printed in A-2-b, PDF p102 to p105, so the
//! test module reruns them.

use nush2_numerics::{EngineError, Result};
use std::f64::consts::PI;

/// Packing shape, which sets the lateral mixing term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Cylinder,
    Sphere,
}

/// Packed bed for the wall coefficient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BedHt {
    pub void: f64,
    /// Particle diameter, m.
    pub dp_m: f64,
    /// Bed equivalent diameter, m.
    pub dt_m: f64,
    /// Bed length, m.
    pub l_m: f64,
    /// Particle surface emissivity.
    pub emissivity: f64,
    /// Particle conductivity, W/m/K.
    pub lambda_s: f64,
    pub shape: Shape,
}

/// Gas state in the bed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BedGas {
    pub t_c: f64,
    pub lambda_g: f64,
    pub nu_m2_s: f64,
    pub prandtl: f64,
    /// Superficial velocity, m/s.
    pub u0_m_s: f64,
}

/// Wall coefficient of the packed bed, W/m2/K, JAERI A.1.2 Eqs. 1 to 10 on the SI
/// constants printed at PDF p95.
pub fn alpha_i(b: &BedHt, g: &BedGas) -> Result<f64> {
    if !(b.void > 0.26 && b.void < 0.6 && g.lambda_g > 0.0 && g.nu_m2_s > 0.0 && g.u0_m_s > 0.0) {
        return Err(EngineError::Range {
            param: "bed heat transfer".into(),
            msg: format!("void {} or the gas state lies outside the form", b.void),
        });
    }
    let e = b.void;
    let p = b.emissivity;
    let r3 = ((g.t_c + 273.0) / 100.0).powi(3);
    let a_rs = 0.2270 * (p / (2.0 - p)) * r3;
    let a_rv = 0.2270 / (1.0 + e / (2.0 * (1.0 - e)) * (1.0 - p) / p) * r3;
    let ratio = b.lambda_s / g.lambda_g;
    let lg = ratio.log10();
    let phi1 = 10f64.powf(-0.3184 * lg - 0.4869);
    let phi2 = 10f64.powf(0.2035 * lg * lg - 0.9233 * lg - 0.4758);
    let phi = phi2 + (phi1 - phi2) * (e - 0.260) / 0.216;
    let dp_lg = b.dp_m / g.lambda_g;
    let le0 = e * (1.0 + a_rv * dp_lg)
        + (1.0 - e) / (1.0 / (1.0 / phi + a_rs * dp_lg) + (2.0 / 3.0) / ratio);
    let re = b.dp_m * g.u0_m_s / g.nu_m2_s;
    let pr = g.prandtl;
    let dpdt = b.dp_m / b.dt_m;
    let psi_beta = match b.shape {
        Shape::Cylinder => -0.6538 * dpdt + 0.176,
        Shape::Sphere => -0.3618 * dpdt + 0.1301,
    };
    let le = le0 + psi_beta * pr * re;
    let ew = 0.7;
    let phi_w = 10f64.powf(-0.2255 * lg - 0.7379);
    let lw0 = ew * (2.0 + a_rv * dp_lg)
        + (1.0 - ew) / (1.0 / (1.0 / phi_w + a_rs * dp_lg) + 1.0 / (3.0 * ratio));
    let aws = 1.0 / (1.0 / lw0 - 0.5 / le0);
    let awt = 4.0 * pr.cbrt() * re.sqrt();
    let psi_w = 0.054;
    let aw = aws + 1.0 / (1.0 / awt + 1.0 / (psi_w * pr * re));
    let bb = 0.5 / dpdt * aw / le;
    let y = 4.0 * dpdt * (b.l_m / b.dt_m) * le / (pr * re);
    let a21 = -0.0406 * bb * bb + 0.7187 * bb + 1.3375;
    let phib = -0.0016 * bb * bb + 0.0399 * bb - 0.0238;
    Ok(dpdt * le * (a21 + phib / y) * g.lambda_g / b.dp_m)
}

/// Finned tube, with the areas per metre of A-2-b, PDF p104.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FinnedTube {
    /// Tube inner diameter, m.
    pub di_m: f64,
    /// Fin root diameter, m.
    pub dr_m: f64,
    /// Fin tip diameter, m.
    pub df_m: f64,
    /// Fin pitch, m.
    pub pitch_m: f64,
    /// Fin thickness, m.
    pub fin_t_m: f64,
    /// Conductivity of tube and fin, W/m/K.
    pub lambda_w: f64,
    /// Inner fouling, m2 K/W.
    pub r_i: f64,
}

impl FinnedTube {
    fn fin_height(&self) -> f64 {
        0.5 * (self.df_m - self.dr_m)
    }

    /// Bare root and fin area per metre, m2/m.
    fn outer_parts(&self) -> (f64, f64) {
        let gap = self.pitch_m - self.fin_t_m;
        let per_m = 1.0 / self.pitch_m;
        let bare = PI * self.dr_m * gap * per_m;
        let tip = PI * self.df_m * self.fin_t_m * per_m;
        let flank = PI / 4.0 * (self.df_m.powi(2) - self.dr_m.powi(2)) * per_m * 2.0;
        (bare, tip + flank)
    }

    /// Outer area per metre, m2/m.
    pub fn a_outer_per_m(&self) -> f64 {
        let (bare, fin) = self.outer_parts();
        bare + fin
    }

    /// Overall coefficient on the outer area, W/m2/K, JAERI A.1.3.
    ///
    /// eq: D.27
    pub fn u_outer(&self, alpha_o: f64, alpha_inner: f64) -> Result<f64> {
        if !(alpha_o > 0.0 && alpha_inner > 0.0) {
            return Err(EngineError::Range {
                param: "film coefficients".into(),
                msg: format!("{alpha_o} and {alpha_inner} must be positive"),
            });
        }
        let (bare, fin) = self.outer_parts();
        let ao = bare + fin;
        let ai = PI * self.di_m;
        let dml = (self.dr_m - self.di_m) / (self.dr_m / self.di_m).ln();
        let am = PI * dml;
        let ts = 0.5 * (self.dr_m - self.di_m);
        let ub = self.fin_height() * (alpha_o / (self.lambda_w * 0.5 * self.fin_t_m)).sqrt();
        let ef = ub.tanh() / ub;
        let rf = (1.0 / alpha_o) * (1.0 - ef) / (ef + bare / fin);
        let inv = 1.0 / alpha_o
            + rf
            + ao / am * ts / self.lambda_w
            + self.r_i * ao / ai
            + ao / ai / alpha_inner;
        Ok(1.0 / inv)
    }
}

/// Helium film coefficient in the finned annulus, W/m2/K, JAERI A.1.1.
///
/// eq: D.29
pub fn alpha_o_knudsen(
    t: &FinnedTube,
    d2_m: f64,
    m_he_kg_s: f64,
    mu: f64,
    lambda: f64,
    prandtl: f64,
) -> Result<f64> {
    let de = d2_m - t.df_m;
    if !(de > 0.0 && m_he_kg_s > 0.0 && mu > 0.0 && lambda > 0.0) {
        return Err(EngineError::Range {
            param: "helium annulus".into(),
            msg: format!("gap {de} m or the helium state lies outside the form"),
        });
    }
    let area = PI / 4.0 * (d2_m * d2_m - t.df_m * t.df_m);
    let re = de * m_he_kg_s / area / mu;
    let s = t.pitch_m - t.fin_t_m;
    let nu = 0.039
        * re.powf(0.87)
        * prandtl.powf(0.4)
        * (s / de).powf(0.4)
        * (t.fin_height() / de).powf(-0.19);
    Ok(nu * lambda / de)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A-2-b whole-exchanger column, PDF p102 to p105.
    const KCAL_W: f64 = 1.163;

    fn jaeri_tube() -> FinnedTube {
        FinnedTube {
            di_m: 0.1278,
            dr_m: 0.1478,
            df_m: 0.1518,
            pitch_m: 0.003,
            fin_t_m: 0.001,
            lambda_w: 21.0 * KCAL_W,
            r_i: 0.0002 / KCAL_W,
        }
    }

    #[test]
    fn bed_coefficient_reruns_the_printed_sheet() {
        let b = BedHt {
            void: 0.45,
            dp_m: 0.005,
            dt_m: 0.0673,
            l_m: 6.0,
            emissivity: 0.14,
            lambda_s: 0.903 * KCAL_W,
            shape: Shape::Cylinder,
        };
        let g = BedGas {
            t_c: 599.0,
            lambda_g: 0.14139 * KCAL_W,
            nu_m2_s: 6.11e-6,
            prandtl: 0.972,
            u0_m_s: 0.6332659,
        };
        let got = alpha_i(&b, &g).unwrap();
        assert!((got / 540.6 - 1.0).abs() < 0.01, "got {got}, printed 540.6");
    }

    #[test]
    fn eq_d_27_overall_coefficient_reruns_the_printed_sheet() {
        let t = jaeri_tube();
        // Whole-exchanger film coefficients and U, A-2-a PDF p99, Sec 1 to 4 column.
        let u = t.u_outer(2324.4, 542.4).unwrap();
        assert!((u / 142.0 - 1.0).abs() < 0.01, "got {u}, printed 142.0");
        assert!((t.a_outer_per_m() / 1.096 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn eq_d_29_helium_film_coefficient_reruns_the_printed_sheet() {
        // A-2-b, whole column, PDF p101, p102: Re 8192, Pr 0.6443, Nu 60.01,
        // W/m/K 0.3643, Pa s 4.519e-5, printed coefficient 2326.
        let t = jaeri_tube();
        let got = alpha_o_knudsen(&t, 0.1612, 0.091, 4.519e-5, 0.3643, 0.6443).unwrap();
        assert!((got / 2326.0 - 1.0).abs() < 0.01, "got {got}");
    }

    #[test]
    fn spheres_mix_less_than_cylinders() {
        let mut b = BedHt {
            void: 0.45,
            dp_m: 0.005,
            dt_m: 0.0673,
            l_m: 6.0,
            emissivity: 0.14,
            lambda_s: 1.05,
            shape: Shape::Cylinder,
        };
        let g = BedGas {
            t_c: 599.0,
            lambda_g: 0.1644,
            nu_m2_s: 6.11e-6,
            prandtl: 0.972,
            u0_m_s: 0.63,
        };
        let cyl = alpha_i(&b, &g).unwrap();
        b.shape = Shape::Sphere;
        let sph = alpha_i(&b, &g).unwrap();
        assert!(sph < cyl);
    }
}
