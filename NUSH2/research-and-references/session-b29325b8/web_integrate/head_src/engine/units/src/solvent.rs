//! Solvent state functions shared by the absorber and the stripper: equilibrium
//! CO2 and water pressures, density, viscosity and enthalpy of 30 mass% MEA and
//! its benchmark strengths.
//!
//! p* comes from the Aronu model in `nush2_mea`. At or below 30 mass% MEA it is read
//! from `PStarTable` inside its grid, and above 30 mass% it is solved directly
//! (`pstar_lookup_kpa`). Water activity follows the same rule through `AwTable`,
//! built here from direct solves on the `PStarTable` grid.
//!
//! Outside the model's stated range the code does one of three things, each
//! counted in `Notes` so a caller can report it:
//! - below 298.15 K, ln p* is extrapolated linearly in 1/T from 298.15 and 303.15 K,
//!   and density and viscosity are read at 293.15 K when the state is colder;
//! - below loading 0.001 the solvent holds no CO2 to speak of, so p* is zero;
//! - above 10 bar of CO2 the Aronu fit is past its stated reliability (G.32).
//!
//! Enthalpy. The liquid is built along one path: heat the unloaded solvent from
//! 298.15 K to `t_phi`, absorb CO2 gas there with the integral heat Phi(alpha), then
//! heat the loaded liquid to T. Phi uses the Gibbs-Helmholtz heat of absorption of
//! `nush2_mea::enthalpy` on p*, which carries plausibility class only. The solvent
//! heat capacity of 3.6 kJ/kg/K is the plan's assumed value (G.25, derivation note
//! M21), since no local source tabulates it.

use crate::stream::{Sp, NSP};
use nush2_mea::aronu::{MW_MEA, MW_WATER};
use nush2_mea::{density, pstar, viscosity};
use nush2_numerics::{EngineError, Result};
use nush2_props::species;
use nush2_props::steam::psat_pa;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

/// Solvent heat capacity, J/kg/K, assumed (G.25).
pub const CP_SOLVENT_J_KG_K: f64 = 3600.0;
/// CO2 molar mass, kg/mol.
pub const MW_CO2: f64 = 0.044_01;
/// Reference temperature of every enthalpy here, K.
pub const T_REF_K: f64 = 298.15;
/// Lowest temperature of the speciation, K.
const T_SPEC_MIN: f64 = 298.15;
const T_SPEC_STEP: f64 = 303.15;
/// Lowest temperature of the density and viscosity fits, K.
const T_PROP_MIN: f64 = 293.15;
const ALPHA_ZERO: f64 = 0.001;
/// Grid of `AwTable`, matching `PStarTable`.
const NT: usize = pstar::TABLE_NT;
const NA: usize = pstar::TABLE_NA;
const GRID_T0: f64 = 303.15;
const GRID_T1: f64 = 403.15;
const GRID_A0: f64 = 0.02;
const GRID_A1: f64 = 0.75;
/// p_CO2 above which Aronu is past its stated reliability, Pa (G.32).
pub const PCO2_RELIABLE_PA: f64 = 10.0e5;
/// Loading grid of the integral heat of absorption.
const PHI_STEP: f64 = 0.01;
const PHI_MAX: f64 = 1.2;

/// Counts of states that left the model's stated range during a solve.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Notes {
    pub cold_extrapolated: u64,
    pub property_floor: u64,
    pub pco2_above_reliable: u64,
}

fn stencil(v: f64, v0: f64, h: f64, n: usize) -> (usize, [f64; 4]) {
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

/// ln a_w on the `PStarTable` grid for one MEA strength, read by cubic Lagrange
/// interpolation in both directions. Every node is a direct solve.
#[derive(Debug, Clone, PartialEq)]
pub struct AwTable {
    pub w_mea: f64,
    ln_aw: Vec<f64>,
}

impl AwTable {
    pub fn t_node(i: usize) -> f64 {
        GRID_T0 + (GRID_T1 - GRID_T0) * i as f64 / (NT - 1) as f64
    }

    pub fn alpha_node(j: usize) -> f64 {
        GRID_A0 + (GRID_A1 - GRID_A0) * j as f64 / (NA - 1) as f64
    }

    pub fn build(w_mea: f64) -> Result<Self> {
        let mut ln_aw = Vec::with_capacity(NT * NA);
        for i in 0..NT {
            for j in 0..NA {
                ln_aw.push(pstar::a_water(Self::t_node(i), w_mea, Self::alpha_node(j))?.ln());
            }
        }
        Ok(Self { w_mea, ln_aw })
    }

    fn inside(t_k: f64, alpha: f64) -> bool {
        (GRID_T0..=GRID_T1).contains(&t_k) && (GRID_A0..=GRID_A1).contains(&alpha)
    }

    /// Interpolated a_w. Range error off the grid.
    pub fn a_water(&self, t_k: f64, alpha: f64) -> Result<f64> {
        if !Self::inside(t_k, alpha) {
            return Err(EngineError::Range {
                param: "t_k, alpha".into(),
                msg: format!("{t_k} K and {alpha} lie off the water activity grid"),
            });
        }
        let ht = (GRID_T1 - GRID_T0) / (NT - 1) as f64;
        let ha = (GRID_A1 - GRID_A0) / (NA - 1) as f64;
        let (it, wt) = stencil(t_k, GRID_T0, ht, NT);
        let (ia, wa) = stencil(alpha, GRID_A0, ha, NA);
        let mut v = 0.0;
        for (p, wp) in wt.iter().enumerate() {
            for (q, wq) in wa.iter().enumerate() {
                v += wp * wq * self.ln_aw[(it + p) * NA + ia + q];
            }
        }
        Ok(v.exp())
    }
}

thread_local! {
    static AW: RefCell<BTreeMap<u64, Rc<AwTable>>> = const { RefCell::new(BTreeMap::new()) };
}

fn aw_table(w_mea: f64) -> Result<Rc<AwTable>> {
    let key = w_mea.to_bits();
    if let Some(t) = AW.with(|m| m.borrow().get(&key).cloned()) {
        return Ok(t);
    }
    let built = Rc::new(AwTable::build(w_mea)?);
    AW.with(|m| m.borrow_mut().insert(key, built.clone()));
    Ok(built)
}

/// Solvent of one MEA strength, CO2-free mass fraction `w_mea`.
#[derive(Debug, Clone)]
pub struct Solvent {
    pub w_mea: f64,
    /// Temperature at which CO2 is absorbed along the enthalpy path, K.
    pub t_phi_k: f64,
    /// Integral heat of absorption Phi(alpha), J per mol MEA, on `PHI_STEP`.
    phi: Vec<f64>,
    h_co2_phi: f64,
    h_water_ref: f64,
    notes: Cell<Notes>,
}

impl Solvent {
    pub fn new(w_mea: f64, t_phi_k: f64) -> Result<Self> {
        if !(nush2_mea::speciate::W_MIN..=nush2_mea::speciate::W_MAX).contains(&w_mea) {
            return Err(EngineError::Range {
                param: "w_mea".into(),
                msg: format!("{w_mea} lies outside the Aronu range"),
            });
        }
        let mut s = Solvent {
            w_mea,
            t_phi_k,
            phi: Vec::new(),
            h_co2_phi: species::h_j_mol(species::Sp::Co2, t_phi_k)?,
            h_water_ref: species::h_liquid_water_j_mol(T_REF_K, 101_325.0)?,
            notes: Cell::new(Notes::default()),
        };
        let n = (PHI_MAX / PHI_STEP).round() as usize;
        let dh: Vec<f64> = (0..=n)
            .map(|k| s.dh_abs_j_mol(t_phi_k, (k as f64 * PHI_STEP).max(GRID_A0)))
            .collect::<Result<_>>()?;
        let mut phi = vec![0.0; n + 1];
        for k in 1..=n {
            phi[k] = phi[k - 1] + 0.5 * PHI_STEP * (dh[k - 1] + dh[k]);
        }
        s.phi = phi;
        s.notes.set(Notes::default());
        Ok(s)
    }

    /// States outside the stated range met since the last `take_notes`.
    pub fn take_notes(&self) -> Notes {
        self.notes.replace(Notes::default())
    }

    fn note(&self, f: impl FnOnce(&mut Notes)) {
        let mut n = self.notes.get();
        f(&mut n);
        self.notes.set(n);
    }

    fn pstar_inside_kpa(&self, t_k: f64, alpha: f64) -> Result<f64> {
        if pstar::table_covers(self.w_mea)
            && (GRID_T0..=GRID_T1).contains(&t_k)
            && (GRID_A0..=GRID_A1).contains(&alpha)
        {
            pstar::pstar_lookup_kpa(t_k, self.w_mea, alpha)
        } else {
            pstar::pstar_kpa(t_k, self.w_mea, alpha)
        }
    }

    /// Equilibrium CO2 pressure over the solvent, Pa, ideal gas.
    pub fn pstar_pa(&self, t_k: f64, alpha: f64) -> Result<f64> {
        if !(alpha.is_finite() && t_k.is_finite()) {
            return Err(EngineError::Range {
                param: "t_k, alpha".into(),
                msg: format!("{t_k} K and {alpha} must be finite"),
            });
        }
        if alpha < ALPHA_ZERO {
            return Ok(0.0);
        }
        if t_k < T_SPEC_MIN {
            self.note(|n| n.cold_extrapolated += 1);
            let a = self.pstar_inside_kpa(T_SPEC_MIN, alpha)?.ln();
            let b = self.pstar_inside_kpa(T_SPEC_STEP, alpha)?.ln();
            let slope = (b - a) / (1.0 / T_SPEC_STEP - 1.0 / T_SPEC_MIN);
            return Ok(1000.0 * (a + slope * (1.0 / t_k - 1.0 / T_SPEC_MIN)).exp());
        }
        Ok(1000.0 * self.pstar_inside_kpa(t_k, alpha)?)
    }

    /// p* with the Poynting factor and the SRK fugacity coefficient at total
    /// pressure `p_pa`, the absorber form of `pstar::pco2_gas_kpa`, Pa.
    pub fn pstar_gas_pa(&self, t_k: f64, alpha: f64, p_pa: f64) -> Result<f64> {
        let ideal = self.pstar_pa(t_k, alpha)?;
        if ideal == 0.0 {
            return Ok(0.0);
        }
        if ideal > PCO2_RELIABLE_PA {
            self.note(|n| n.pco2_above_reliable += 1);
        }
        let t = t_k.max(T_SPEC_MIN);
        let y = (ideal / p_pa).min(0.98);
        let phi = pstar::srk_phi_co2(t, p_pa, y)?;
        Ok(ideal * pstar::poynting(t, p_pa)? / phi)
    }

    /// Water activity on the pure liquid basis.
    pub fn a_water(&self, t_k: f64, alpha: f64) -> Result<f64> {
        let a = alpha.max(ALPHA_ZERO);
        let t = if t_k < T_SPEC_MIN {
            self.note(|n| n.cold_extrapolated += 1);
            T_SPEC_MIN
        } else {
            t_k
        };
        if pstar::table_covers(self.w_mea) && AwTable::inside(t, a) {
            aw_table(self.w_mea)?.a_water(t, a)
        } else {
            pstar::a_water(t, self.w_mea, a)
        }
    }

    /// Water partial pressure over the solvent, Pa.
    pub fn p_water_pa(&self, t_k: f64, alpha: f64) -> Result<f64> {
        Ok(self.a_water(t_k, alpha)? * psat_pa(t_k)?)
    }

    /// p* of CO2 and of water, Pa, ideal gas, for a state inside the speciation
    /// range (298.15 K and up). Off the tables both come from one direct solve.
    pub fn bubble_pressures(&self, t_k: f64, alpha: f64) -> Result<(f64, f64)> {
        let a = alpha.max(ALPHA_ZERO);
        let tables = pstar::table_covers(self.w_mea)
            && AwTable::inside(t_k, a)
            && (GRID_T0..=GRID_T1).contains(&t_k);
        if t_k < T_SPEC_MIN || tables {
            return Ok((self.pstar_pa(t_k, alpha)?, self.p_water_pa(t_k, alpha)?));
        }
        let sp = nush2_mea::speciate(t_k, self.w_mea, a)?;
        let aw = sp.ln_gamma[nush2_mea::aronu::I_H2O].exp() * sp.x[nush2_mea::aronu::I_H2O];
        let pc = if alpha < ALPHA_ZERO {
            0.0
        } else {
            1000.0 * pstar::pstar_of(&sp)
        };
        Ok((pc, aw * psat_pa(t_k)?))
    }

    /// Differential heat of absorption, J/mol CO2, by Gibbs-Helmholtz on p* at
    /// T +/- 2 K, the method of `nush2_mea::enthalpy`.
    pub fn dh_abs_j_mol(&self, t_k: f64, alpha: f64) -> Result<f64> {
        let lo = self.pstar_pa(t_k - 2.0, alpha)?.ln();
        let hi = self.pstar_pa(t_k + 2.0, alpha)?.ln();
        Ok(-pstar::R_GAS * (hi - lo) / (1.0 / (t_k + 2.0) - 1.0 / (t_k - 2.0)))
    }

    /// Integral heat of absorption at `t_phi_k`, J per mol MEA. Below loading 0.02
    /// the heat is held at its 0.02 value.
    pub fn phi_j_mol_mea(&self, alpha: f64) -> Result<f64> {
        if !(0.0..=PHI_MAX).contains(&alpha) {
            return Err(EngineError::Range {
                param: "alpha".into(),
                msg: format!("{alpha} lies outside 0 to {PHI_MAX}"),
            });
        }
        let s = alpha / PHI_STEP;
        let k = (s.floor() as usize).min(self.phi.len() - 2);
        let u = s - k as f64;
        Ok(self.phi[k] + u * (self.phi[k + 1] - self.phi[k]))
    }

    fn prop_t(&self, t_k: f64) -> f64 {
        if t_k < T_PROP_MIN {
            self.note(|n| n.property_floor += 1);
            T_PROP_MIN
        } else {
            t_k
        }
    }

    /// Density, kg/m3, Weiland.
    pub fn density(&self, t_k: f64, alpha: f64) -> Result<f64> {
        density::density_kg_m3(self.prop_t(t_k), self.w_mea, alpha.clamp(0.0, 1.0))
    }

    /// Total MEA concentration, kmol/m3.
    pub fn c_mea(&self, t_k: f64, alpha: f64) -> Result<f64> {
        density::c_mea_kmol_m3(self.prop_t(t_k), self.w_mea, alpha.clamp(0.0, 1.0))
    }

    /// Viscosity, Pa s, Weiland.
    pub fn viscosity(&self, t_k: f64, alpha: f64) -> Result<f64> {
        viscosity::viscosity_pa_s(self.prop_t(t_k), self.w_mea, alpha.clamp(0.0, 1.0))
    }

    /// Viscosity over water viscosity, Khan Eq 7.
    pub fn mu_ratio(&self, t_k: f64, alpha: f64) -> Result<f64> {
        viscosity::ratio_to_water(self.prop_t(t_k), self.w_mea, alpha.clamp(0.0, 1.0))
    }

    /// Enthalpy flow of a liquid of apparent flows MEA, H2O and CO2 (mol/s) at
    /// `t_k`, W, on the formation basis with MEA as its own reference.
    pub fn h_liquid_w(&self, n_mea: f64, n_h2o: f64, n_co2: f64, t_k: f64) -> Result<f64> {
        let alpha = if n_mea > 0.0 { n_co2 / n_mea } else { 0.0 };
        let m_free = n_mea * MW_MEA + n_h2o * MW_WATER;
        let m_all = m_free + n_co2 * MW_CO2;
        Ok(
            m_free * CP_SOLVENT_J_KG_K * (self.t_phi_k - T_REF_K) + n_co2 * self.h_co2_phi
                - n_mea * self.phi_j_mol_mea(alpha)?
                + m_all * CP_SOLVENT_J_KG_K * (t_k - self.t_phi_k)
                + n_h2o * self.h_water_ref,
        )
    }
}

/// Ideal-gas enthalpy flow, W, formation basis. Below 298.15 K, the lower end of
/// the gas tables, each species is carried down at its 298.15 K heat capacity.
pub fn h_gas_w(n: &[f64; NSP], t_k: f64) -> Result<f64> {
    let mut h = 0.0;
    for (i, &v) in n.iter().enumerate() {
        if v == 0.0 {
            continue;
        }
        let sp = props_sp(i)?;
        let hi = if t_k < T_REF_K {
            species::h_j_mol(sp, T_REF_K)? + species::cp_j_mol_k(sp, T_REF_K)? * (t_k - T_REF_K)
        } else {
            species::h_j_mol(sp, t_k)?
        };
        h += v * hi;
    }
    Ok(h)
}

/// Molar heat capacity of an ideal gas mixture, J/mol/K, at max(T, 298.15 K).
pub fn cp_gas_j_mol_k(n: &[f64; NSP], t_k: f64) -> Result<f64> {
    let total: f64 = n.iter().sum();
    let mut c = 0.0;
    for (i, &v) in n.iter().enumerate() {
        if v > 0.0 {
            c += v * species::cp_j_mol_k(props_sp(i)?, t_k.max(T_REF_K))?;
        }
    }
    Ok(c / total)
}

/// Mean molar mass, kg/mol.
pub fn mw_gas(n: &[f64; NSP]) -> Result<f64> {
    let total: f64 = n.iter().sum();
    let mut m = 0.0;
    for (i, &v) in n.iter().enumerate() {
        m += v * props_sp(i)?.mw_kg_mol();
    }
    Ok(m / total)
}

fn props_sp(i: usize) -> Result<species::Sp> {
    const MAP: [species::Sp; NSP] = [
        species::Sp::H2,
        species::Sp::Co,
        species::Sp::Co2,
        species::Sp::Ch4,
        species::Sp::H2o,
        species::Sp::N2,
        species::Sp::C2h6,
        species::Sp::C3h8,
        species::Sp::Nc4h10,
        species::Sp::Nc5h12,
    ];
    MAP.get(i).copied().ok_or_else(|| EngineError::Range {
        param: "species index".into(),
        msg: format!("{i} is not a species"),
    })
}

/// Index of a species in `Stream::n`.
pub fn idx(sp: Sp) -> usize {
    sp as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aw_table_equals_direct_solves_on_nodes_and_between_them() {
        let t = AwTable::build(0.30).unwrap();
        for (i, j) in [(0, 0), (5, 17), (15, 39)] {
            let (tk, a) = (AwTable::t_node(i), AwTable::alpha_node(j));
            let direct = pstar::a_water(tk, 0.30, a).unwrap();
            assert!((t.a_water(tk, a).unwrap() / direct - 1.0).abs() < 1e-12);
        }
        let mut worst = 0.0_f64;
        for (tk, a) in [(317.0, 0.27), (351.4, 0.433), (389.9, 0.12), (333.3, 0.51)] {
            let direct = pstar::a_water(tk, 0.30, a).unwrap();
            worst = worst.max((t.a_water(tk, a).unwrap() / direct - 1.0).abs());
        }
        println!("a_w table worst relative gap between nodes {worst:.2e}");
        assert!(worst < 1e-4, "{worst}");
    }

    #[test]
    fn enthalpy_path_rises_with_temperature_and_falls_with_absorption() {
        let s = Solvent::new(0.30, 323.15).unwrap();
        let (m, w) = (1.0, 7.0);
        let h1 = s.h_liquid_w(m, w, 0.25, 320.0).unwrap();
        let h2 = s.h_liquid_w(m, w, 0.25, 330.0).unwrap();
        assert!(h2 > h1);
        let dh = s.dh_abs_j_mol(323.15, 0.3).unwrap();
        assert!(dh > 5.0e4 && dh < 1.2e5, "{dh}");
        // Absorbing gas CO2 at t_phi releases the tabulated heat.
        let gas = species::h_j_mol(species::Sp::Co2, 323.15).unwrap();
        let before = s.h_liquid_w(m, w, 0.25, 323.15).unwrap() + 0.01 * gas;
        let after = s.h_liquid_w(m, w, 0.26, 323.15).unwrap();
        let released = before - after;
        let want = s.phi_j_mol_mea(0.26).unwrap() - s.phi_j_mol_mea(0.25).unwrap();
        assert!((released - want).abs() < 1e-6 * want, "{released} {want}");
    }

    #[test]
    fn fresh_solvent_has_no_back_pressure_and_cold_states_are_counted() {
        let s = Solvent::new(0.20, 313.15).unwrap();
        assert_eq!(s.pstar_pa(300.0, 0.0).unwrap(), 0.0);
        s.take_notes();
        let cold = s.pstar_pa(294.0, 0.1).unwrap();
        let warm = s.pstar_pa(299.0, 0.1).unwrap();
        assert!(cold > 0.0 && cold < warm);
        assert_eq!(s.take_notes().cold_extrapolated, 1);
    }
}
