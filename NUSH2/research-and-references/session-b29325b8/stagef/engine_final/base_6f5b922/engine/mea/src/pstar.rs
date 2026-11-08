//! Equilibrium CO2 pressure over the solvent, gas phase corrections, loading at a
//! given pressure, water activity and the tabulated p* used by column code.
//!
//! `pstar_kpa` is gamma_CO2 x_CO2 H(T) on the prototype's kPa convention. The SRK
//! correction applies only to the 104-point comparison, where the vapour is CO2 and
//! water (card G.21). The absorber keeps the Poynting factor and takes an ideal
//! vapour (card G.19), because its vapour is a carrier gas this binary cannot hold.
//!
//! Assumed inputs: v_inf is 3.5e-5 m3/mol, since Aronu cites Brelvi-O'Connell and
//! prints no value. The SRK critical constants are general knowledge and come from no
//! local source. Water Psat is IF97, in place of the prototype's Antoine form.

use crate::aronu::{henry_water_co2_pa_m3_mol, I_CO2, I_H2O};
use crate::solve::brent;
use crate::speciate::{speciate, Speciation};
use nush2_numerics::{EngineError, Result};
use nush2_props::steam::psat_pa;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// J/mol/K, the prototype's value, kept beside `nush2_props::units::R_GAS` (the
/// exact SI product) so the 104-point port reproduces the prototype. The two agree
/// to 7.4e-8 relative (test `eq_d_13_gas_constants_are_exact_or_explained`).
pub const R_GAS: f64 = 8.314462;
/// CO2 partial molar volume at infinite dilution, m3/mol, assumed.
pub const V_INF_CO2: f64 = 3.5e-5;
/// Upper cap on the vapour CO2 fraction in `pco2_gas_kpa`, from the prototype.
const Y_CAP: f64 = 0.98;

/// p* in kPa from a converged state.
pub fn pstar_of(sp: &Speciation) -> f64 {
    sp.ln_gamma[I_CO2].exp() * sp.x[I_CO2] * sp.ln_h_co2.exp() / 1000.0
}

/// Equilibrium CO2 pressure in kPa, ideal gas.
pub fn pstar_kpa(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    Ok(pstar_of(&speciate(t_k, w_mea, alpha)?))
}

/// Water activity on the pure liquid basis.
pub fn a_water(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    let sp = speciate(t_k, w_mea, alpha)?;
    Ok(sp.ln_gamma[I_H2O].exp() * sp.x[I_H2O])
}

/// Water partial pressure over the solvent at low pressure, Pa: a_w times the IF97
/// saturation pressure, with no vapour phase correction.
pub fn water_partial_pressure_pa(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    Ok(a_water(t_k, w_mea, alpha)? * psat_pa(t_k)?)
}

/// Apparent Henry constant of CO2 in the solvent, Pa m3/mol: gamma*_CO2 times the
/// Henry constant in water, Aronu Eqs 17 and 19, PDF p4. Basis warning from
/// `mea_thermo.md` s3: Eq 19 equates a mole fraction activity coefficient to a
/// concentration ratio, so rate work keeps Khan's closure.
pub fn henry_apparent_pa_m3_mol(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    let sp = speciate(t_k, w_mea, alpha)?;
    Ok(sp.ln_gamma[I_CO2].exp() * henry_water_co2_pa_m3_mol(t_k))
}

/// Poynting factor for CO2 between the water saturation pressure and `p_pa`.
pub fn poynting(t_k: f64, p_pa: f64) -> Result<f64> {
    Ok((V_INF_CO2 * (p_pa - psat_pa(t_k)?) / (R_GAS * t_k)).exp())
}

/// Largest real root of the SRK cubic Z^3 - Z^2 + c1 Z + c0 = 0.
fn largest_real_root(c1: f64, c0: f64) -> f64 {
    let p = c1 - 1.0 / 3.0;
    let q = -2.0 / 27.0 + c1 / 3.0 + c0;
    let disc = q * q / 4.0 + p * p * p / 27.0;
    let t = if disc >= 0.0 {
        let s = disc.sqrt();
        (-q / 2.0 + s).cbrt() + (-q / 2.0 - s).cbrt()
    } else {
        let r = 2.0 * (-p / 3.0).sqrt();
        let phi = (3.0 * q / (p * r)).clamp(-1.0, 1.0).acos();
        r * (phi / 3.0).cos()
    };
    t + 1.0 / 3.0
}

/// Compressibility factor and CO2 fugacity coefficient of one SRK state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SrkState {
    /// Largest real root of the cubic.
    pub z: f64,
    pub phi_co2: f64,
}

/// SRK fugacity coefficient of CO2 in a CO2 and H2O vapour with k_ij = 0.
///
/// eq: G.21
///
/// Domain: a vapour of CO2 and water only, the Aronu vapour of the 104-point model
/// column comparison. The binary has no carrier gas, so at absorber pressure it can
/// describe a liquid: at 313.15 K and 26.8 bar the cubic has a single real root
/// near Z = 0.025 and phi_CO2 runs from about 80 to 560 (test
/// `eq_g_21_phi_at_absorber_pressure` in `nush2_units`). The absorber therefore
/// does not call it (`nush2_units::solvent::PHI_CO2_ABSORBER`).
pub fn srk_phi_co2(t_k: f64, p_pa: f64, y_co2: f64) -> Result<f64> {
    Ok(srk_co2_h2o(t_k, p_pa, y_co2)?.phi_co2)
}

/// The SRK state behind `srk_phi_co2`, with the root it took.
///
/// eq: G.21
pub fn srk_co2_h2o(t_k: f64, p_pa: f64, y_co2: f64) -> Result<SrkState> {
    if !(t_k.is_finite() && t_k > 0.0 && p_pa.is_finite() && p_pa > 0.0) {
        return Err(EngineError::Range {
            param: "t_k, p_pa".into(),
            msg: format!("{t_k} K and {p_pa} Pa must be positive"),
        });
    }
    if !(0.0..=1.0).contains(&y_co2) {
        return Err(EngineError::Range {
            param: "y_co2".into(),
            msg: format!("{y_co2} lies outside 0 to 1"),
        });
    }
    const TC: [f64; 2] = [304.12, 647.1];
    const PC: [f64; 2] = [7.374e6, 22.064e6];
    const OMEGA: [f64; 2] = [0.225, 0.344];
    let y = [y_co2, 1.0 - y_co2];
    let mut a = [0.0; 2];
    let mut b = [0.0; 2];
    for i in 0..2 {
        let m = 0.480 + 1.574 * OMEGA[i] - 0.176 * OMEGA[i] * OMEGA[i];
        let s = 1.0 + m * (1.0 - (t_k / TC[i]).sqrt());
        a[i] = 0.42748 * R_GAS * R_GAS * TC[i] * TC[i] / PC[i] * s * s;
        b[i] = 0.08664 * R_GAS * TC[i] / PC[i];
    }
    let mut amix = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            amix += y[i] * y[j] * (a[i] * a[j]).sqrt();
        }
    }
    let bmix = y[0] * b[0] + y[1] * b[1];
    let rt = R_GAS * t_k;
    let big_a = amix * p_pa / (rt * rt);
    let big_b = bmix * p_pa / rt;
    let z = largest_real_root(big_a - big_b - big_b * big_b, -big_a * big_b);
    let a0 = y[0] * (a[0] * a[0]).sqrt() + y[1] * (a[0] * a[1]).sqrt();
    let phi_co2 = (b[0] / bmix * (z - 1.0)
        - (z - big_b).ln()
        - big_a / big_b * (2.0 * a0 / amix - b[0] / bmix) * (1.0 + big_b / z).ln())
    .exp();
    Ok(SrkState { z, phi_co2 })
}

/// p* with the Poynting and SRK corrections at total pressure `ptot_kpa`, kPa.
pub fn pco2_gas_kpa(t_k: f64, w_mea: f64, alpha: f64, ptot_kpa: f64) -> Result<f64> {
    if !(ptot_kpa.is_finite() && ptot_kpa > 0.0) {
        return Err(EngineError::Range {
            param: "ptot_kpa".into(),
            msg: format!("{ptot_kpa} must be positive"),
        });
    }
    let pm = pstar_kpa(t_k, w_mea, alpha)?;
    let y = (pm / ptot_kpa).min(Y_CAP);
    let phi = srk_phi_co2(t_k, ptot_kpa * 1e3, y)?;
    Ok(pm * poynting(t_k, ptot_kpa * 1e3)? / phi)
}

/// Loading at which p* equals `p_kpa`.
pub fn alpha_at(t_k: f64, w_mea: f64, p_kpa: f64) -> Result<f64> {
    if !(p_kpa.is_finite() && p_kpa > 0.0) {
        return Err(EngineError::Range {
            param: "p_kpa".into(),
            msg: format!("{p_kpa} must be positive"),
        });
    }
    let f = |a: f64| -> Result<f64> { Ok((pstar_kpa(t_k, w_mea, a)? / p_kpa).ln()) };
    let lo = (0.02, f(0.02)?);
    let mut hi = (0.9, f(0.9)?);
    if lo.1.signum() == hi.1.signum() {
        hi = (1.4, f(1.4)?);
    }
    brent(f, lo, hi, 1e-6, 100)
}

/// Grid of `PStarTable`: 16 temperatures from 30 to 130 C.
pub const TABLE_NT: usize = 16;
/// Grid of `PStarTable`: 40 loadings from 0.02 to 0.75.
pub const TABLE_NA: usize = 40;
const TABLE_T0: f64 = 303.15;
const TABLE_T1: f64 = 403.15;
const TABLE_A0: f64 = 0.02;
const TABLE_A1: f64 = 0.75;

/// ln p* on a fixed grid for one MEA mass fraction, read by cubic Lagrange
/// interpolation in each direction.
///
/// p* varies as loading squared at low loading, so ln p* has a logarithmic singularity
/// that a polynomial in loading follows badly near 0.02. The table therefore stores
/// the smooth remainder ln p* - 2 ln alpha and restores the term after interpolation.
/// This changes the interpolation only: every node still equals a direct solve.
///
/// The 0.5% gate of `pstar_table_interp` covers 30 mass%, the design strength.
/// `pstar_table_interp_other_strengths` shows 15 mass% inside it, while 45 and 60 mass%
/// exceed it near loading 0.52 and 38 C on this grid. Above 30 mass%,
/// `pstar_lookup_kpa` therefore skips the table and solves directly.
#[derive(Debug, Clone, PartialEq)]
pub struct PStarTable {
    pub w_mea: f64,
    smooth: [[f64; TABLE_NA]; TABLE_NT],
}

/// Four-point stencil on a uniform grid: start index and weights. The one copy in
/// the engine; `nush2_units::solvent` reads its water activity table through it.
///
/// eq: G.33
pub fn stencil(v: f64, v0: f64, h: f64, n: usize) -> (usize, [f64; 4]) {
    let s = (v - v0) / h;
    let i = (s.floor() as usize).clamp(1, n - 3);
    let u = s - i as f64;
    let w = [
        -u * (u - 1.0) * (u - 2.0) / 6.0,
        (u + 1.0) * (u - 1.0) * (u - 2.0) / 2.0,
        -(u + 1.0) * u * (u - 2.0) / 2.0,
        (u + 1.0) * u * (u - 1.0) / 6.0,
    ];
    (i - 1, w)
}

impl PStarTable {
    /// Grid temperature, K.
    pub fn t_node(i: usize) -> f64 {
        TABLE_T0 + (TABLE_T1 - TABLE_T0) * i as f64 / (TABLE_NT - 1) as f64
    }

    /// Grid loading.
    pub fn alpha_node(j: usize) -> f64 {
        TABLE_A0 + (TABLE_A1 - TABLE_A0) * j as f64 / (TABLE_NA - 1) as f64
    }

    /// Solve every node directly.
    pub fn build(w_mea: f64) -> Result<Self> {
        let mut smooth = [[0.0; TABLE_NA]; TABLE_NT];
        for (i, row) in smooth.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let a = Self::alpha_node(j);
                *cell = pstar_kpa(Self::t_node(i), w_mea, a)?.ln() - 2.0 * a.ln();
            }
        }
        Ok(Self { w_mea, smooth })
    }

    /// Interpolated p* in kPa. Range errors match the grid.
    pub fn pstar_kpa(&self, t_k: f64, alpha: f64) -> Result<f64> {
        if !(TABLE_T0..=TABLE_T1).contains(&t_k) {
            return Err(EngineError::Range {
                param: "t_k".into(),
                msg: format!("{t_k} K lies outside {TABLE_T0} to {TABLE_T1} K"),
            });
        }
        if !(TABLE_A0..=TABLE_A1).contains(&alpha) {
            return Err(EngineError::Range {
                param: "alpha".into(),
                msg: format!("{alpha} lies outside {TABLE_A0} to {TABLE_A1}"),
            });
        }
        let ht = (TABLE_T1 - TABLE_T0) / (TABLE_NT - 1) as f64;
        let ha = (TABLE_A1 - TABLE_A0) / (TABLE_NA - 1) as f64;
        let (it, wt) = stencil(t_k, TABLE_T0, ht, TABLE_NT);
        let (ia, wa) = stencil(alpha, TABLE_A0, ha, TABLE_NA);
        let mut v = 0.0;
        for (p, wp) in wt.iter().enumerate() {
            for (q, wq) in wa.iter().enumerate() {
                v += wp * wq * self.smooth[it + p][ia + q];
            }
        }
        Ok((v + 2.0 * alpha.ln()).exp())
    }
}

thread_local! {
    static TABLES: RefCell<BTreeMap<u64, Rc<PStarTable>>> = const { RefCell::new(BTreeMap::new()) };
}

/// The table for `w_mea`, built on first use and held per thread.
pub fn pstar_table(w_mea: f64) -> Result<Rc<PStarTable>> {
    let key = w_mea.to_bits();
    if let Some(t) = TABLES.with(|m| m.borrow().get(&key).cloned()) {
        return Ok(t);
    }
    let built = Rc::new(PStarTable::build(w_mea)?);
    TABLES.with(|m| m.borrow_mut().insert(key, built.clone()));
    Ok(built)
}

/// Highest MEA mass fraction at which column code reads `PStarTable`. The 0.5%
/// interpolation gate covers 30 mass%, the design strength, and nothing above it.
pub const TABLE_W_MAX: f64 = 0.30;

/// Whether `pstar_lookup_kpa` reads the table at `w_mea`.
pub fn table_covers(w_mea: f64) -> bool {
    w_mea <= TABLE_W_MAX
}

/// p* in kPa, the path column code should take. At or below 30 mass% MEA it reads
/// `PStarTable`. Above that it solves the speciation directly, because the table's
/// interpolation gate was never shown to hold there.
pub fn pstar_lookup_kpa(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    if table_covers(w_mea) {
        pstar_table(w_mea)?.pstar_kpa(t_k, alpha)
    } else {
        pstar_kpa(t_k, w_mea, alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_root_matches_a_known_factorisation() {
        // (Z - 0.9)(Z - 0.08)(Z - 0.02) has unit Z^2 sum, so it fits the SRK form.
        let (a, b, c) = (0.9, 0.08, 0.02);
        let z = largest_real_root(a * b + a * c + b * c, -a * b * c);
        assert!((z - 0.9).abs() < 1e-12, "{z}");
    }

    #[test]
    fn single_root_branch_is_taken_when_disc_is_positive() {
        // One real root near 1 with a tiny coefficient set.
        let z = largest_real_root(0.001, -1e-5);
        assert!((z * z * z - z * z + 0.001 * z - 1e-5).abs() < 1e-12, "{z}");
    }

    #[test]
    fn poynting_at_21_bar_and_40c() {
        let f = poynting(313.15, 21e5).unwrap();
        assert!((f - 1.029).abs() < 0.001, "{f}");
    }

    #[test]
    fn phi_tends_to_one_at_low_pressure() {
        let phi = srk_phi_co2(350.0, 1000.0, 0.5).unwrap();
        assert!((phi - 1.0).abs() < 1e-3, "{phi}");
    }

    #[test]
    fn lookup_reads_the_table_only_up_to_thirty_percent() {
        assert!(table_covers(0.30));
        assert!(!table_covers(0.31));
        // Above 30 mass% the lookup equals a direct solve exactly, between nodes too.
        let (t, w, a) = (331.7, 0.33, 0.437);
        let direct = pstar_kpa(t, w, a).unwrap();
        assert_eq!(pstar_lookup_kpa(t, w, a).unwrap(), direct);
        // At 30 mass% it reads the table, which equals a direct solve on a node.
        let (tn, an) = (PStarTable::t_node(4), PStarTable::alpha_node(12));
        let node = pstar_kpa(tn, 0.30, an).unwrap();
        let read = pstar_lookup_kpa(tn, 0.30, an).unwrap();
        assert!((read / node - 1.0).abs() < 1e-9, "{read} against {node}");
    }

    #[test]
    fn stencil_is_exact_on_nodes_and_sums_to_one() {
        let (_, w) = stencil(0.7, 0.0, 0.1, 40);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        let (i, w) = stencil(0.5, 0.0, 0.1, 40);
        assert_eq!(i, 4);
        assert!((w[1] - 1.0).abs() < 1e-12);
    }
}
