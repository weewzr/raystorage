//! Extended UNIQUAC for H2O, MEA and CO2 after Aronu et al., Chem. Eng. Sci. 66
//! (2011) 6393-6406, with corrigendum 78 (2012) 246-247.
//!
//! Printed in Aronu and read from the page images: Table 2 (PDF p4), Table 7 and
//! Tables 8 and 9 (PDF p7), Eqs 14 to 19 (PDF p4).
//!
//! Label carried by every output of this module: Thomsen activity forms, A(T), b, z
//! and v_inf come from general knowledge of the cited framework and from no local
//! source. The reproduction of Aronu's 104 published model values supports them
//! indirectly.

use nush2_numerics::{EngineError, Result};

/// Species count.
pub const NS: usize = 9;
pub const I_H2O: usize = 0;
pub const I_MEA: usize = 1;
pub const I_CO2: usize = 2;
pub const I_H3O: usize = 3;
pub const I_MEAH: usize = 4;
pub const I_OH: usize = 5;
pub const I_HCO3: usize = 6;
pub const I_CO3: usize = 7;
pub const I_MEACOO: usize = 8;

/// Species order of every array in this crate.
///
/// eq: G.1
pub const SPECIES: [&str; NS] = [
    "H2O", "MEA", "CO2", "H3O+", "MEAH+", "OH-", "HCO3-", "CO3--", "MEACOO-",
];
/// Ionic charge.
pub const CHARGE: [f64; NS] = [0.0, 0.0, 0.0, 1.0, 1.0, -1.0, -1.0, -2.0, -1.0];
/// UNIQUAC volume parameter r, Aronu Table 7, PDF p7.
pub const R_PAR: [f64; NS] = [
    0.9200, 4.2800, 5.7410, 0.13779, 1.0241, 9.3973, 9.1571, 9.7452, 1.0741,
];
/// UNIQUAC surface parameter q, Aronu Table 7, PDF p7.
pub const Q_PAR: [f64; NS] = [
    1.4000, 4.2800, 6.0806, 1e-15, 2.5150, 8.8171, 6.3461, 6.4614, 0.1106,
];
/// kg/mol of water, the prototype's value, kept beside `nush2_props::units::MW_H2O`
/// (18.015 kg/kmol) so the Aronu port reproduces the prototype; the gap is 1.6e-5
/// relative.
pub const MW_WATER: f64 = 0.018_015_28;
/// kg/mol of MEA, the prototype's value, kept beside `nush2_props::units::MW_MEA`:
/// the same number on the kg/mol scale this port uses.
pub const MW_MEA: f64 = 0.061_08;
/// Coordination number. Thomsen framework, no local source.
pub const Z_COORD: f64 = 10.0;
/// Debye-Hueckel b, kg^0.5 mol^-0.5. Thomsen framework, no local source.
pub const B_DH: f64 = 1.5;
/// u reference temperature, K, Aronu Eq 15.
const T_REF: f64 = 298.15;

/// u0, Aronu Table 8, PDF p7, lower triangle. The upper triangle stays zero.
const U0: [[f64; NS]; NS] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [173.9645, 414.6924, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-151.4573, 87.5583, 40.5176, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [1.0e4, 1.0e9, 1.0e9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-20.7732, 310.1293, 30.8035, 1.0e9, 0.0, 0.0, 0.0, 0.0, 0.0],
    [
        600.4952, 1.0e9, 2500.0, 1.0e9, 1.0e9, 1562.8810, 0.0, 0.0, 0.0,
    ],
    [
        517.0278, 655.0881, 597.9726, 1.0e9, 732.7007, 2500.0, 743.6159, 0.0, 0.0,
    ],
    [
        361.3877, 1.0e9, 2500.0, 1.0e9, 1.0e9, 1588.0250, 719.159, 1458.3440, 0.0,
    ],
    [
        2758.3852, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1500.0,
    ],
];

/// uT, Aronu Table 9, PDF p7, lower triangle. The upper triangle stays zero.
const UT: [[f64; NS]; NS] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.8027, 0.6647, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [6.0908, 4.6660, 13.6290, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [-1.9174, 0.1213, 7.3541, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [8.5455, 0.0, 0.0, 0.0, 0.0, 5.6169, 0.0, 0.0, 0.0],
    [6.9504, 15.2488, 5.8077, 0.0, 2.8863, 0.0, 17.1148, 0.0, 0.0],
    [3.3516, 0.0, 0.0, 0.0, 0.0, 2.7496, 2.6115, -1.3448, 0.0],
    [16.9192, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
];

/// Coefficients C1 to C4 of `ln K or ln H = C1 + C2/T + C3 ln T + C4 T`,
/// Aronu Eq 16 and Table 2, PDF p4.
const K_WATER: [f64; 4] = [132.899, -13445.90, -22.4773, 0.0];
const K_CO2: [f64; 4] = [231.465, -12092.10, -36.7816, 0.0];
const K_HCO3: [f64; 4] = [216.049, -12431.70, -35.4819, 0.0];
const K_MEA: [f64; 4] = [-4.9074, -6166.12, 0.0, -0.00098482];
const K_MEACOO: [f64; 4] = [2.8898, -3635.09, 0.0, 0.0];
const H_CO2: [f64; 4] = [170.7126, -8477.711, -21.95743, 0.005781];

/// eq: G.2
fn table2(c: &[f64; 4], t_k: f64) -> f64 {
    c[0] + c[1] / t_k + c[2] * t_k.ln() + c[3] * t_k
}

/// Interaction energy u_ij at `t_k`, Aronu Eq 15, symmetric in i and j.
///
/// eq: G.16
pub fn u_ij(i: usize, j: usize, t_k: f64) -> f64 {
    let (a, b) = if i >= j { (i, j) } else { (j, i) };
    U0[a][b] + UT[a][b] * (t_k - T_REF)
}

/// Debye-Hueckel A(T), kg^0.5 mol^-0.5, with t in deg C. Thomsen framework, no
/// local source.
///
/// eq: G.17
pub fn a_dh(t_k: f64) -> f64 {
    let t = t_k - 273.15;
    1.131 + 1.335e-3 * t + 1.164e-5 * t * t
}

/// Henry constant of CO2 in water in Pa m3/mol, Aronu Eq 17, PDF p4.
///
/// eq: G.22
pub fn henry_water_co2_pa_m3_mol(t_k: f64) -> f64 {
    2.82e6 * (-2044.0 / t_k).exp()
}

/// Henry constant of N2O in water in Pa m3/mol, Aronu Eq 18, PDF p4.
pub fn henry_water_n2o_pa_m3_mol(t_k: f64) -> f64 {
    8.55e6 * (-2284.0 / t_k).exp()
}

/// Everything in the model that depends on temperature alone.
#[derive(Debug, Clone)]
pub struct Thermo {
    pub t_k: f64,
    /// psi[k][i] = exp(-(u_ki - u_ii)/T), exponent clipped to 700 in magnitude.
    psi: [[f64; NS]; NS],
    /// ln gamma of each species at infinite dilution in water, combinatorial plus
    /// residual, used for the unsymmetric reference of the solutes.
    ln_gamma_inf: [f64; NS],
    a_dh: f64,
    pub ln_k_water: f64,
    pub ln_k_co2: f64,
    pub ln_k_hco3: f64,
    /// Aronu Table 2 value, on the aqueous pKa basis.
    pub ln_k_mea_table: f64,
    pub ln_k_meacoo: f64,
    /// Henry constant of CO2, ln(H / Pa), Aronu Table 2.
    pub ln_h_co2: f64,
}

impl Thermo {
    /// Build for a positive finite temperature. Range policy lives in `speciate`.
    ///
    /// eq: G.2
    pub fn new(t_k: f64) -> Result<Self> {
        if !(t_k.is_finite() && t_k > 0.0) {
            return Err(EngineError::Range {
                param: "t_k".into(),
                msg: format!("{t_k} K is not a positive temperature"),
            });
        }
        let mut psi = [[0.0; NS]; NS];
        for (k, row) in psi.iter_mut().enumerate() {
            for (i, p) in row.iter_mut().enumerate() {
                let e = -(u_ij(k, i, t_k) - u_ij(i, i, t_k)) / t_k;
                *p = e.clamp(-700.0, 700.0).exp();
            }
        }
        let z_half = Z_COORD / 2.0;
        let mut ln_gamma_inf = [0.0; NS];
        for i in 0..NS {
            let rr = R_PAR[i] / R_PAR[I_H2O];
            let qq = Q_PAR[i] / Q_PAR[I_H2O];
            let rq = rr / qq;
            let ln_c = rr.ln() + 1.0 - rr - z_half * Q_PAR[i] * (rq.ln() + 1.0 - rq);
            let ln_r = Q_PAR[i] * (1.0 - psi[I_H2O][i].ln() - psi[i][I_H2O]);
            ln_gamma_inf[i] = ln_c + ln_r;
        }
        Ok(Self {
            t_k,
            psi,
            ln_gamma_inf,
            a_dh: a_dh(t_k),
            ln_k_water: table2(&K_WATER, t_k),
            ln_k_co2: table2(&K_CO2, t_k),
            ln_k_hco3: table2(&K_HCO3, t_k),
            ln_k_mea_table: table2(&K_MEA, t_k),
            ln_k_meacoo: table2(&K_MEACOO, t_k),
            ln_h_co2: table2(&H_CO2, t_k),
        })
    }

    /// ln gamma of MEA at infinite dilution in water, referred to pure MEA.
    pub fn ln_gamma_inf_mea(&self) -> f64 {
        self.ln_gamma_inf[I_MEA]
    }

    /// The K_MEA convention. Table 2 K_MEA is the aqueous pKa, with MEA referred to
    /// infinite dilution in water, while Aronu Eq 11 refers MEA to the pure liquid.
    /// The effective constant on the stated basis is the Table 2 value plus
    /// ln gamma_inf,MEA. K_MEACOO stays as printed.
    ///
    /// eq: G.7
    pub fn ln_k_mea_effective(&self) -> f64 {
        self.ln_k_mea_table + self.ln_gamma_inf_mea()
    }

    /// ln gamma: symmetric for H2O and MEA, unsymmetric at infinite dilution in
    /// water for the seven solutes. Debye-Hueckel molality is on kg of water.
    /// `x` must hold positive mole fractions.
    pub(crate) fn ln_gamma_raw(&self, x: &[f64; NS]) -> [f64; NS] {
        let z_half = Z_COORD / 2.0;
        let sum_r: f64 = x.iter().zip(&R_PAR).map(|(a, b)| a * b).sum();
        let sum_q: f64 = x.iter().zip(&Q_PAR).map(|(a, b)| a * b).sum();
        let mut theta = [0.0; NS];
        for i in 0..NS {
            theta[i] = x[i] * Q_PAR[i] / sum_q;
        }
        let mut s = [0.0; NS];
        for i in 0..NS {
            let mut acc = 0.0;
            for k in 0..NS {
                acc += theta[k] * self.psi[k][i];
            }
            s[i] = acc;
        }
        let mut lng = [0.0; NS];
        for i in 0..NS {
            let phi_x = R_PAR[i] / sum_r;
            let phi_theta = phi_x / (Q_PAR[i] / sum_q);
            let ln_c =
                phi_x.ln() + 1.0 - phi_x - z_half * Q_PAR[i] * (phi_theta.ln() + 1.0 - phi_theta);
            let mut term = 0.0;
            for k in 0..NS {
                term += theta[k] * self.psi[i][k] / s[k];
            }
            lng[i] = ln_c + Q_PAR[i] * (1.0 - s[i].ln() - term);
        }
        for i in I_CO2..NS {
            lng[i] -= self.ln_gamma_inf[i];
        }

        let kg_water = x[I_H2O] * MW_WATER;
        let mut ionic = 0.0;
        for i in 0..NS {
            ionic += 0.5 * x[i] / kg_water * CHARGE[i] * CHARGE[i];
        }
        let si = ionic.max(0.0).sqrt();
        for i in I_CO2..NS {
            lng[i] -= CHARGE[i] * CHARGE[i] * self.a_dh * si / (1.0 + B_DH * si);
        }
        let xb = B_DH * si;
        let sigma = if xb > 1e-6 {
            3.0 / (xb * xb * xb) * (1.0 + xb - 1.0 / (1.0 + xb) - 2.0 * (1.0 + xb).ln())
        } else {
            1.0
        };
        lng[I_H2O] += (2.0 / 3.0) * MW_WATER * self.a_dh * ionic.powf(1.5) * sigma;
        lng
    }

    /// Checked form of the activity model for any caller.
    ///
    /// eq: G.6
    /// eq: G.13
    /// eq: G.14
    /// eq: G.15
    /// eq: G.17
    /// eq: G.18
    pub fn ln_gamma(&self, x: &[f64; NS]) -> Result<[f64; NS]> {
        if x.iter().any(|v| !(v.is_finite() && *v > 0.0)) {
            return Err(EngineError::Range {
                param: "x".into(),
                msg: "mole fractions must be positive and finite".into(),
            });
        }
        Ok(self.ln_gamma_raw(x))
    }
}

/// ln gamma of MEA at infinite dilution in water at `t_k`.
pub fn ln_gamma_inf_mea(t_k: f64) -> Result<f64> {
    Ok(Thermo::new(t_k)?.ln_gamma_inf_mea())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_g_16_u_matrix_is_symmetric() {
        for i in 0..NS {
            for j in 0..NS {
                assert_eq!(u_ij(i, j, 350.0), u_ij(j, i, 350.0));
            }
        }
    }

    #[test]
    fn eq_g_02_table2_values_at_298k() {
        // pKw 14.0 on a molal basis over 55.51 mol/kg, squared for the mole fraction basis.
        let th = Thermo::new(298.15).unwrap();
        let want = (1.0e-14_f64 / (55.51 * 55.51)).ln();
        assert!((th.ln_k_water - want).abs() < 0.05, "{}", th.ln_k_water);
    }

    #[test]
    fn bad_mole_fractions_are_an_error() {
        let th = Thermo::new(320.0).unwrap();
        assert!(th.ln_gamma(&[0.0; NS]).is_err());
        assert!(Thermo::new(f64::NAN).is_err());
    }
}
