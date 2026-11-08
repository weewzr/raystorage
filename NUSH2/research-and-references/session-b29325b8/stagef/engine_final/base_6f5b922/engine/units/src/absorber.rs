//! ABS: rate-based packed absorber, class ReducedOrder (plan section 4).
//!
//! Rate closure. The Khan, Krishnamoorthi and Mahmud (2011) set is used whole from
//! `nush2_mea::kinetics`: Hikita k2 (Eq 2), the Danckwerts Henry constant (Eq 8),
//! the Versteeg CO2 diffusivity with the Weiland viscosity ratio (Eqs 6a, 6b, 7)
//! and the Wellek enhancement factor (Eq 4). Eq 4 is read as E1 = Ha/tanh(Ha), which
//! corrects the printed M^2 form (derivation H.6). Free MEA follows Khan's own
//! stoichiometry, C_B = C_MEA (1 - 2 alpha) (Eq 10, b = 2), so the reaction term
//! stops at loading 0.50 and only physical absorption remains above it.
//!
//! Khan prints no MEA diffusivity for E_infinity. D_B here is the 7 m MEA membrane
//! cell coefficient of Dugas 2009 Table 4.2 (PDF p92: 2.2e-10 m2/s at 30 C, 2.8 cP,
//! loading 0.25 to 0.35), carried to other states by the Snijder viscosity
//! dependence that Dugas prints as Eq 4.2 (PDF p89), D mu^0.6 constant. The second
//! 7 m row of that table (4.7e-10 m2/s at 3.3 cP, loading 0.45 to 0.55) would raise
//! D_B by a factor near 2.3. Both are integral coefficients of all species, so this
//! is a stand-in, stated as such.
//!
//! Hydraulics. Liquid film by Rocha (Isa Table 4 Eq 10, PDF p6, C_E = 0.9 from the
//! nomenclature, PDF p2), effective area by Billet and Schultes (Isa Table 5 Eqs 17
//! and 18, C_h = 0.554, PDF p2 and p6). The gas film by Rocha (Isa Eq 11) needs a
//! gas diffusivity, and the only local value is Chinchen et al. (1984) PDF p12: CO
//! in shift reaction gas at 31 bar, 0.035 cm2/s at 300 C and 0.052 cm2/s at 450 C.
//! That pair stands in for CO2 in H2-rich syngas, with viscosity and conductivity
//! from the JAERI syngas fit in `nush2_props::transport`. No local source gives a
//! gas diffusivity for N2 or CH4 carriers, so those benchmarks run with the gas film
//! omitted: the liquid side carries the whole resistance and the gas leaves each
//! slice at the liquid temperature. Water leaves every slice at equilibrium with
//! the liquid in both modes, the limit of H.22 with no gas resistance.
//!
//! Numerics. Finite volumes on `slices` cells, conservative in each phase, six unknowns
//! per cell: gas CO2, gas H2O and gas temperature leaving the cell, liquid CO2,
//! liquid H2O and liquid temperature leaving it. The two CO2 flows are carried as
//! logarithms, since near full removal the gas CO2 spans several decades. Rates
//! are taken at the mean of each slice's inlet and outlet states (box scheme),
//! which is second order in the slice height. `newton::solve` runs Newton then
//! pseudo-transient continuation; continuation in pressure from 1 bar and then a
//! doubled grid follow, as plan section 5 sets out. Solvent strength for p* stays
//! at the lean value, while the water balance itself is tracked.

use crate::newton::{self, Shape, SolvePath};
use crate::packing::Structured;
use crate::solve::brent;
use crate::solvent::{self, idx, Notes, Solvent, CP_SOLVENT_J_KG_K, MW_CO2};
use crate::stream::{Closure, SolventStream, Sp, Stream, NSP};
use nush2_mea::aronu::{MW_MEA, MW_WATER};
use nush2_mea::kinetics;
use nush2_numerics::{EngineError, Result};
use nush2_props::mixture;
use nush2_props::transport;
use std::f64::consts::PI;

/// Gravitational acceleration, m/s2.
const G: f64 = 9.81;
/// Gas constant, J/mol/K, as in `nush2_mea::pstar`.
const R: f64 = nush2_mea::pstar::R_GAS;
/// Rocha correction factor C_E, Isa nomenclature PDF p2.
pub const C_E: f64 = 0.9;
/// Billet and Schultes packing constant C_h, Isa nomenclature PDF p2.
pub const C_H: f64 = 0.554;
/// Khan R1 stoichiometric factor.
const B: f64 = kinetics::B_STOICH;
/// Snijder exponent as printed in Dugas Eq 4.2, PDF p89.
pub const SNIJDER_EXP: f64 = 0.6;

/// MEA diffusivity for E_infinity: a reference value carried to other states as
/// D mu^0.6 constant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeaDiffusivity {
    pub d_ref_m2_s: f64,
    pub mu_ref_pa_s: f64,
    pub source: &'static str,
}

impl MeaDiffusivity {
    pub fn at(&self, mu_pa_s: f64) -> f64 {
        self.d_ref_m2_s * (self.mu_ref_pa_s / mu_pa_s).powf(SNIJDER_EXP)
    }
}

/// Central: Dugas Table 4.2, 7 m MEA at 30 C, loading 0.25 to 0.35, 2.8 cP.
pub const DUGAS_7M_LEAN: MeaDiffusivity = MeaDiffusivity {
    d_ref_m2_s: 2.2e-10,
    mu_ref_pa_s: 2.8e-3,
    source: "Dugas 2009 Table 4.2 PDF p92, 7 m MEA, loading 0.25 to 0.35",
};

/// Sensitivity: the same table, loading 0.45 to 0.55, 3.3 cP.
pub const DUGAS_7M_RICH: MeaDiffusivity = MeaDiffusivity {
    d_ref_m2_s: 4.7e-10,
    mu_ref_pa_s: 3.3e-3,
    source: "Dugas 2009 Table 4.2 PDF p92, 7 m MEA, loading 0.45 to 0.55",
};
/// Smallest free MEA used in the rate, kmol/m3, so the enhancement stays defined.
const C_B_FLOOR: f64 = 1.0e-6;
/// Plan section 5: residual target of the column solve.
pub const TOL: f64 = 1.0e-7;
/// Unknowns per slice.
const NB: usize = 6;

/// Gas diffusivity of CO2 as a reference value scaled as T^n / p.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasDiffusivity {
    pub d_ref_m2_s: f64,
    pub t_ref_k: f64,
    pub p_ref_pa: f64,
    pub t_exp: f64,
    pub source: &'static str,
}

impl GasDiffusivity {
    pub fn at(&self, t_k: f64, p_pa: f64) -> f64 {
        self.d_ref_m2_s * (t_k / self.t_ref_k).powf(self.t_exp) * self.p_ref_pa / p_pa
    }
}

/// Chinchen et al. 1984 PDF p12: CO in reaction gas at 31 bar, 0.035 cm2/s at
/// 300 C and 0.052 cm2/s at 450 C. The exponent follows from the two values.
pub fn chinchen_syngas() -> GasDiffusivity {
    let (d1, t1, d2, t2) = (0.035e-4, 573.15, 0.052e-4, 723.15);
    GasDiffusivity {
        d_ref_m2_s: d1,
        t_ref_k: t1,
        p_ref_pa: 31.0e5,
        t_exp: (d2 / d1).ln() / (t2 / t1).ln(),
        source: "Chinchen et al. 1984 PDF p12, CO in shift gas, stand-in for CO2",
    }
}

/// Gas-side treatment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GasFilm {
    /// Rocha k_G with the given diffusivity, JAERI syngas viscosity and
    /// conductivity, Chilton-Colburn heat transfer (H.21).
    RochaSyngas(GasDiffusivity),
    /// No gas-film resistance; the gas leaves each slice at the liquid temperature.
    Omitted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbsorberSpec {
    pub tag: String,
    pub packing: Structured,
    pub diameter_m: f64,
    pub height_m: f64,
    pub slices: usize,
    pub p_pa: f64,
    pub gas_film: GasFilm,
    /// Temperature of the enthalpy path, K (see `solvent`).
    pub t_phi_k: f64,
    /// MEA mass fraction on a CO2-free basis for the thermodynamics. The lean
    /// stream's own water may drift from it through the column.
    pub w_mea: f64,
    /// MEA diffusivity used in E_infinity.
    pub d_mea: MeaDiffusivity,
}

/// One slice of a converged column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliceState {
    /// Slice centre above the bottom of the packing, m.
    pub z_m: f64,
    pub y_co2: f64,
    pub t_gas_k: f64,
    pub t_liq_k: f64,
    pub alpha: f64,
    /// CO2 flux on the interfacial area, mol/m2/s.
    pub flux: f64,
    pub hatta: f64,
    pub e_inf: f64,
    pub e: f64,
    /// Effective area, 1/m.
    pub a_e: f64,
    pub p_co2_pa: f64,
    pub pstar_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbsorberResult {
    /// Packed height solved, m.
    pub height_m: f64,
    pub gas_out: Stream,
    pub rich: SolventStream,
    /// Per-pass capture, H.27.
    pub capture: f64,
    pub slices: Vec<SliceState>,
    pub closure: Closure,
    pub residual: f64,
    pub iterations: usize,
    pub path: SolvePath,
    /// Fallback rungs used: pressure continuation, doubled grid.
    pub continued_in_pressure: bool,
    pub doubled_grid: bool,
    pub notes: Notes,
    /// Converged unknowns, for a warm start.
    pub x: Vec<f64>,
}

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

/// Second-order rate constant of CO2 with MEA, m3/kmol/s, Khan Eq 2 (Hikita).
///
/// eq: H.5
pub fn k2(t_k: f64) -> Result<f64> {
    kinetics::k2_hikita(t_k)
}

/// Henry constant of CO2, Pa m3/mol, Khan Eq 8 read in atm L/mol, with the total
/// MEA concentration in kmol/m3.
///
/// eq: H.9
pub fn henry_khan(t_k: f64, c_mea_kmol_m3: f64) -> Result<f64> {
    kinetics::henry_khan_pa_m3_mol(t_k, c_mea_kmol_m3)
}

/// Enhancement of CO2 absorption by reaction with MEA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enhancement {
    pub hatta: f64,
    pub e1: f64,
    pub e_inf: f64,
    pub e: f64,
}

/// Khan Eq 4: Hatta number Ha^2 = D_A k2 C_B / k_L^2, pseudo-first-order factor
/// E1 = Ha/tanh(Ha), instantaneous factor E_inf = 1 + C_B D_B/(b D_A C_Ai) with b = 2,
/// and the Wellek interpolation between them. `c_bl` and `c_ai` in kmol/m3. When
/// either limit sits within 1e-12 of 1 the reaction adds nothing and E = 1.
///
/// eq: H.6
/// eq: H.7
/// eq: H.8
pub fn enhancement(
    d_al: f64,
    d_b: f64,
    k2: f64,
    c_bl: f64,
    c_ai: f64,
    k_l0: f64,
) -> Result<Enhancement> {
    let ha = kinetics::hatta(d_al, k2, c_bl, k_l0)?;
    let e1 = kinetics::e1_of_hatta(ha)?;
    let e_inf = kinetics::e_infinity(c_bl, d_b, B, d_al, c_ai)?;
    let e = if e1 - 1.0 < 1e-12 || e_inf - 1.0 < 1e-12 {
        1.0
    } else {
        kinetics::enhancement_wellek(e_inf, e1)?
    };
    Ok(Enhancement {
        hatta: ha,
        e1,
        e_inf,
        e,
    })
}

/// Physical liquid-film coefficient, m/s, Rocha as printed in Isa Table 4 Eq 10
/// (PDF p6): k_L = 2 (D_L C_E U_L / (pi S))^0.5 with U_L the superficial velocity.
///
/// eq: H.10
pub fn kl_rocha(d_l: f64, u_l: f64, s: f64) -> Result<f64> {
    positive("d_l", d_l)?;
    positive("u_l", u_l)?;
    positive("s", s)?;
    Ok(2.0 * (d_l * C_E * u_l / (PI * s)).sqrt())
}

/// Gas-film coefficient, m/s, Rocha as printed in Isa Table 4 Eq 11 (PDF p6).
///
/// eq: H.11
pub fn kg_rocha(d_g: f64, u_l: f64, u_g: f64, rho_g: f64, mu_g: f64, s: f64) -> Result<f64> {
    for (n, v) in [
        ("d_g", d_g),
        ("u_g", u_g),
        ("rho_g", rho_g),
        ("mu_g", mu_g),
        ("s", s),
    ] {
        positive(n, v)?;
    }
    let re = (u_l + u_g) * rho_g * s / mu_g;
    let sc = mu_g / (d_g * rho_g);
    Ok(0.054 * (d_g / s) * re.powf(0.8) * sc.powf(0.33))
}

/// Effective area, 1/m, Billet and Schultes as printed in Isa Table 5 Eqs 17 and 18
/// (PDF p6), with C_h = 0.554. Isa states the correlation to 30 bar.
///
/// eq: H.12
pub fn area_bs(a: f64, u_l: f64, rho_l: f64, mu_l: f64) -> Result<f64> {
    positive("a", a)?;
    positive("u_l", u_l)?;
    let re = u_l * rho_l / (a * mu_l);
    let fr = u_l * u_l * a / G;
    let ratio = if re < 5.0 {
        C_H * re.powf(0.15) * fr.powf(0.1)
    } else {
        0.85 * C_H * re.powf(0.25) * fr.powf(0.1)
    };
    Ok(a * ratio)
}

/// Overall gas-side coefficient, mol/m2/s/Pa: 1/K_G = 1/k_G + H/(E k_L).
///
/// eq: H.4
pub fn k_overall(k_g: f64, henry: f64, e: f64, k_l0: f64) -> Result<f64> {
    positive("k_g", k_g)?;
    positive("henry", henry)?;
    positive("e", e)?;
    positive("k_l0", k_l0)?;
    Ok(1.0 / (1.0 / k_g + henry / (e * k_l0)))
}

/// Inputs of one flux evaluation, SI units with concentrations in kmol/m3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluxInputs {
    pub p_co2_pa: f64,
    pub pstar_pa: f64,
    /// Gas film, mol/m2/s/Pa. `None` leaves the gas film out.
    pub k_g: Option<f64>,
    pub k_l0: f64,
    pub henry: f64,
    pub d_al: f64,
    pub d_b: f64,
    pub k2: f64,
    pub c_bl: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flux {
    /// mol/m2/s, positive into the liquid.
    pub n: f64,
    pub p_i_pa: f64,
    pub enh: Enhancement,
}

fn enh_at(fi: &FluxInputs, p_i: f64) -> Result<Enhancement> {
    let c_ai = (p_i / fi.henry / 1000.0).max(1e-12);
    enhancement(fi.d_al, fi.d_b, fi.k2, fi.c_bl, c_ai, fi.k_l0)
}

/// CO2 flux through the two films: gas film N = k_G (p - p_i), liquid film
/// N = E k_L (p_i - p*)/H with the bulk liquid at equilibrium, and Henry's law at
/// the interface. p_i is solved by Brent because E_inf depends on it.
///
/// eq: H.1
/// eq: H.2
/// eq: H.3
pub fn flux(fi: &FluxInputs) -> Result<Flux> {
    let (p, ps) = (fi.p_co2_pa, fi.pstar_pa);
    let liquid = |p_i: f64| -> Result<(f64, Enhancement)> {
        let enh = enh_at(fi, p_i)?;
        Ok((enh.e * fi.k_l0 * (p_i - ps) / fi.henry, enh))
    };
    let p_i = match fi.k_g {
        None => p,
        Some(kg) => {
            if (p - ps).abs() <= 1e-12 * p.max(ps).max(1.0) {
                p
            } else {
                let f = |pi: f64| -> Result<f64> { Ok(kg * (p - pi) - liquid(pi)?.0) };
                brent(f, ps.min(p), ps.max(p), 1e-12 * p.max(ps), 200)?
            }
        }
    };
    let (n, enh) = liquid(p_i)?;
    Ok(Flux {
        n,
        p_i_pa: p_i,
        enh,
    })
}

/// Water flux, mol/m2/s, through the gas film: N = k_G,H2O (p_H2O - p*_H2O). The
/// column takes this at its no-resistance limit, so each slice's gas leaves at
/// equilibrium with its liquid (Sonderby assumption 3, PDF p3, puts the volatile
/// solvent's resistance on the gas side).
///
/// eq: H.22
pub fn water_flux(k_g_h2o: f64, p_h2o_pa: f64, pstar_h2o_pa: f64) -> f64 {
    k_g_h2o * (p_h2o_pa - pstar_h2o_pa)
}

/// Gas-liquid heat transfer coefficient, W/m2/K, by the Chilton-Colburn analogy:
/// h = k_G c_G c_p,G (Sc/Pr)^(2/3), with k_G in m/s, c_G in mol/m3 and c_p,G in
/// J/mol/K.
///
/// eq: H.21
pub fn heat_flux(k_g_m_s: f64, c_g: f64, cp_g_j_mol_k: f64, sc: f64, pr: f64) -> Result<f64> {
    positive("sc", sc)?;
    positive("pr", pr)?;
    Ok(k_g_m_s * c_g * cp_g_j_mol_k * (sc / pr).powf(2.0 / 3.0))
}

/// Per-pass capture on absorber-inlet CO2 (Vinjarapu 2024 Eq 20; Kwak Eq 2).
///
/// eq: H.27
pub fn capture(co2_in: f64, co2_out: f64) -> Result<f64> {
    positive("co2_in", co2_in)?;
    Ok((co2_in - co2_out) / co2_in)
}

/// CO2-free solvent mass flow, kg/s, that carries `n_co2` mol/s of CO2 between two
/// loadings at MEA mass fraction `w`.
///
/// eq: H.28
/// eq: I.9
pub fn solvent_rate(n_co2: f64, w: f64, alpha_rich: f64, alpha_lean: f64) -> Result<f64> {
    positive("n_co2", n_co2)?;
    positive("w", w)?;
    positive("alpha_rich - alpha_lean", alpha_rich - alpha_lean)?;
    Ok(n_co2 * MW_MEA / (w * (alpha_rich - alpha_lean)))
}

/// Column diameter, m, from the volumetric liquid flow and a liquid load in
/// m3/m2/h (40 to 100 in the design note).
///
/// eq: H.29
pub fn diameter(q_liquid_m3_s: f64, load_m3_m2_h: f64) -> Result<f64> {
    positive("q_liquid_m3_s", q_liquid_m3_s)?;
    positive("load_m3_m2_h", load_m3_m2_h)?;
    Ok((4.0 * q_liquid_m3_s * 3600.0 / (PI * load_m3_m2_h)).sqrt())
}

/// Co-absorbed fraction of a gas species. Zero central: no local source measures
/// H2, CH4 or CO solubility in loaded MEA, and the aMDEA reference stays a
/// disclosed term of the report.
///
/// eq: H.30
pub fn coabsorb(_sp: Sp) -> f64 {
    0.0
}

/// Slice boundaries above the bottom of the packing, m: `n` equal finite volumes.
///
/// eq: H.24
pub fn grid(height_m: f64, n: usize) -> Result<Vec<f64>> {
    positive("height_m", height_m)?;
    if n < 2 {
        return Err(EngineError::Range {
            param: "slices".into(),
            msg: format!("{n} slices cannot form a column"),
        });
    }
    Ok((0..=n).map(|j| height_m * j as f64 / n as f64).collect())
}

/// Fixed data of one solve.
struct Column<'a> {
    spec: &'a AbsorberSpec,
    solvent: Solvent,
    n: usize,
    area: f64,
    dz: f64,
    p: f64,
    inert: [f64; NSP],
    feed_c: f64,
    feed_w: f64,
    feed_t: f64,
    n_mea: f64,
    lean_c: f64,
    lean_w: f64,
    lean_t: f64,
    h_gas_in: f64,
    h_lean: f64,
    s_c: f64,
    s_w: f64,
    s_e: f64,
}

#[derive(Debug, Clone, Copy)]
struct Gas {
    c: f64,
    w: f64,
    t: f64,
}

#[derive(Debug, Clone, Copy)]
struct Liq {
    c: f64,
    w: f64,
    t: f64,
}

/// Rates and state of one slice at its outlet conditions.
struct SliceRates {
    n_c: f64,
    a_int: f64,
    p_w: f64,
    q_w: f64,
    state: SliceState,
}

impl<'a> Column<'a> {
    fn gas_flows(&self, g: Gas) -> [f64; NSP] {
        let mut n = self.inert;
        n[idx(Sp::Co2)] = g.c;
        n[idx(Sp::H2o)] = g.w;
        n
    }

    fn rates(&self, j: usize, g: Gas, l: Liq) -> Result<SliceRates> {
        let s = &self.solvent;
        let alpha = (l.c / self.n_mea).max(0.0);
        let ng = self.gas_flows(g);
        let total: f64 = ng.iter().sum();
        let p_co2 = (g.c / total).max(0.0) * self.p;
        let pstar = s.pstar_gas_pa(l.t, alpha, self.p)?;
        let rho_l = s.density(l.t, alpha)?;
        let mu_l = s.viscosity(l.t, alpha)?;
        let c_tot = s.c_mea(l.t, alpha)?;
        let m_l = self.n_mea * MW_MEA + l.w * MW_WATER + l.c * MW_CO2;
        let u_l = m_l / rho_l / self.area;
        let a_e = area_bs(self.spec.packing.a_m2_m3, u_l, rho_l, mu_l)?;
        let d_al = kinetics::d_co2_solution_m2_s(l.t, s.mu_ratio(l.t, alpha)?)?;
        let k_l0 = kl_rocha(d_al, u_l, self.spec.packing.channel_side_m)?;
        let d_b = self.spec.d_mea.at(mu_l);
        let c_bl = (c_tot * (1.0 - B * alpha)).max(C_B_FLOOR);
        let henry = henry_khan(l.t, c_tot)?;
        let mut q_w = 0.0;
        let k_g = match self.spec.gas_film {
            GasFilm::Omitted => None,
            GasFilm::RochaSyngas(dg) => {
                let tg = transport::syngas(g.t)?;
                let mw = solvent::mw_gas(&ng)?;
                let c_g = self.p / (R * g.t);
                let rho_g = c_g * mw;
                let u_g = total / c_g / self.area;
                let d_g = dg.at(g.t, self.p);
                let kg = kg_rocha(
                    d_g,
                    u_l,
                    u_g,
                    rho_g,
                    tg.mu_pa_s,
                    self.spec.packing.channel_side_m,
                )?;
                let cp = solvent::cp_gas_j_mol_k(&ng, g.t)?;
                let sc = tg.mu_pa_s / (rho_g * d_g);
                let pr = cp / mw * tg.mu_pa_s / tg.k_w_m_k;
                let h = heat_flux(kg, c_g, cp, sc, pr)?;
                q_w = h * (g.t - l.t);
                Some(kg / (R * g.t))
            }
        };
        let f = flux(&FluxInputs {
            p_co2_pa: p_co2,
            pstar_pa: pstar,
            k_g,
            k_l0,
            henry,
            d_al,
            d_b,
            k2: k2(l.t)?,
            c_bl,
        })?;
        let p_w = s.p_water_pa(l.t, alpha)?;
        let z = (j as f64 + 0.5) * self.dz;
        Ok(SliceRates {
            n_c: f.n,
            a_int: a_e * self.area * self.dz,
            p_w,
            q_w,
            state: SliceState {
                z_m: z,
                y_co2: g.c / total,
                t_gas_k: g.t,
                t_liq_k: l.t,
                alpha,
                flux: f.n,
                hatta: f.enh.hatta,
                e_inf: f.enh.e_inf,
                e: f.enh.e,
                a_e,
                p_co2_pa: p_co2,
                pstar_pa: pstar,
            },
        })
    }

    fn unpack(&self, x: &[f64], j: usize) -> (Gas, Liq) {
        let b = j * NB;
        (
            Gas {
                c: x[b].exp(),
                w: x[b + 1],
                t: x[b + 2],
            },
            Liq {
                c: x[b + 3].exp(),
                w: x[b + 4],
                t: x[b + 5],
            },
        )
    }

    fn h_gas(&self, g: Gas) -> Result<f64> {
        solvent::h_gas_w(&self.gas_flows(g), g.t)
    }

    fn h_liq(&self, l: Liq) -> Result<f64> {
        self.solvent.h_liquid_w(self.n_mea, l.w, l.c, l.t)
    }

    /// Gas entering slice j from below and liquid entering it from above.
    fn inlets(&self, x: &[f64], j: usize) -> (Gas, Liq) {
        let gin = if j == 0 {
            Gas {
                c: self.feed_c,
                w: self.feed_w,
                t: self.feed_t,
            }
        } else {
            self.unpack(x, j - 1).0
        };
        let lin = if j + 1 == self.n {
            Liq {
                c: self.lean_c,
                w: self.lean_w,
                t: self.lean_t,
            }
        } else {
            self.unpack(x, j + 1).1
        };
        (gin, lin)
    }

    /// Box scheme: rates of slice j at the mean of its inlet and outlet states.
    fn mid(g: Gas, gin: Gas, l: Liq, lin: Liq) -> (Gas, Liq) {
        (
            Gas {
                c: 0.5 * (g.c + gin.c),
                w: 0.5 * (g.w + gin.w),
                t: 0.5 * (g.t + gin.t),
            },
            Liq {
                c: 0.5 * (l.c + lin.c),
                w: 0.5 * (l.w + lin.w),
                t: 0.5 * (l.t + lin.t),
            },
        )
    }

    fn residuals(&self, x: &[f64]) -> Result<Vec<f64>> {
        let mut r = vec![0.0; self.n * NB];
        for j in 0..self.n {
            let (g, l) = self.unpack(x, j);
            let (gin, lin) = self.inlets(x, j);
            let h_gin = if j == 0 {
                self.h_gas_in
            } else {
                self.h_gas(gin)?
            };
            let h_lin = if j + 1 == self.n {
                self.h_lean
            } else {
                self.h_liq(lin)?
            };
            let (gm, lm) = Self::mid(g, gin, l, lin);
            let rt = self.rates(j, gm, lm)?;
            let h_g = self.h_gas(g)?;
            let h_l = self.h_liq(l)?;
            let dry_out: f64 = self.gas_flows(g).iter().sum::<f64>() - g.w;
            let w_eq = dry_out * rt.p_w / (self.p - rt.p_w);
            let b = j * NB;
            r[b] = (g.c - gin.c + rt.n_c * rt.a_int) / self.s_c;
            r[b + 1] = (g.w - w_eq) / self.s_w;
            r[b + 2] = match self.spec.gas_film {
                GasFilm::Omitted => g.t - lm.t,
                GasFilm::RochaSyngas(_) => {
                    let mut ng = [0.0; NSP];
                    ng[idx(Sp::Co2)] = gin.c - g.c;
                    ng[idx(Sp::H2o)] = gin.w - g.w;
                    let carried = transferred_h(&ng, gm.t)?;
                    (h_g - h_gin + carried + rt.q_w * rt.a_int) / self.s_e
                }
            };
            r[b + 3] = (l.c + g.c - lin.c - gin.c) / self.s_c;
            r[b + 4] = (l.w + g.w - lin.w - gin.w) / self.s_w;
            r[b + 5] = (h_g + h_l - h_gin - h_lin) / self.s_e;
        }
        Ok(r)
    }

    fn guess(&self) -> Vec<f64> {
        let n = self.n;
        let cap = ((0.48 - self.lean_c / self.n_mea).max(0.05) * self.n_mea / self.feed_c).min(0.9);
        let mut x = vec![0.0; n * NB];
        let top_c = self.feed_c * (1.0 - cap);
        for j in 0..n {
            // Gas CO2 falls geometrically from feed to top.
            let f = (j + 1) as f64 / n as f64;
            let gc = self.feed_c * (top_c / self.feed_c).powf(f);
            let below = if j == 0 {
                self.feed_c
            } else {
                self.feed_c * (top_c / self.feed_c).powf(j as f64 / n as f64)
            };
            let lc = self.lean_c + below - top_c;
            let absorbed_above = below - top_c;
            let m_l = self.n_mea * MW_MEA + self.lean_w * MW_WATER;
            let rise = (absorbed_above * 8.0e4 / (m_l * CP_SOLVENT_J_KG_K)).min(35.0);
            let t = (self.lean_t + rise).min(395.0);
            let b = j * NB;
            x[b] = gc.ln();
            x[b + 1] = self.feed_w.max(1e-9);
            x[b + 2] = t;
            x[b + 3] = lc.max(1e-9 * self.s_c).ln();
            x[b + 4] = self.lean_w;
            x[b + 5] = t;
        }
        x
    }

    fn shape(&self) -> Shape {
        let n = self.n * NB;
        let mut lo = vec![0.0; n];
        let mut hi = vec![0.0; n];
        let mut scale = vec![0.0; n];
        for j in 0..self.n {
            let b = j * NB;
            for (k, (l, h, s)) in [
                ((1e-14 * self.s_c).ln(), (10.0 * self.s_c).ln(), 1.0),
                (0.0, 100.0 * self.s_w, self.s_w * 1e-3),
                (250.0, 420.0, 300.0),
                ((1e-14 * self.s_c).ln(), (10.0 * self.s_c).ln(), 1.0),
                (0.0, 10.0 * self.lean_w.max(self.s_w), self.lean_w),
                (250.0, 420.0, 300.0),
            ]
            .into_iter()
            .enumerate()
            {
                lo[b + k] = l;
                hi[b + k] = h;
                scale[b + k] = s;
            }
        }
        Shape {
            block: NB,
            reach: 1,
            lo,
            hi,
            scale,
        }
    }
}

fn transferred_h(n: &[f64; NSP], t_k: f64) -> Result<f64> {
    // Signed flows are allowed here: a species may cross in either direction.
    let mut pos = [0.0; NSP];
    let mut neg = [0.0; NSP];
    for i in 0..NSP {
        if n[i] > 0.0 {
            pos[i] = n[i];
        } else {
            neg[i] = -n[i];
        }
    }
    let hp = if pos.iter().any(|v| *v > 0.0) {
        solvent::h_gas_w(&pos, t_k)?
    } else {
        0.0
    };
    let hn = if neg.iter().any(|v| *v > 0.0) {
        solvent::h_gas_w(&neg, t_k)?
    } else {
        0.0
    };
    Ok(hp - hn)
}

fn build<'a>(spec: &'a AbsorberSpec, gas_in: &Stream, lean: &SolventStream) -> Result<Column<'a>> {
    positive("diameter_m", spec.diameter_m)?;
    positive("height_m", spec.height_m)?;
    positive("p_pa", spec.p_pa)?;
    positive("lean n_mea", lean.n_mea)?;
    if spec.slices < 2 {
        return Err(EngineError::Range {
            param: "slices".into(),
            msg: format!("{} slices cannot form a column", spec.slices),
        });
    }
    let solvent = Solvent::new(spec.w_mea, spec.t_phi_k)?;
    let mut inert = gas_in.n;
    inert[idx(Sp::Co2)] = 0.0;
    inert[idx(Sp::H2o)] = 0.0;
    let feed_c = gas_in.n[idx(Sp::Co2)];
    positive("feed CO2", feed_c)?;
    let feed_w = gas_in.n[idx(Sp::H2o)];
    let area = PI * spec.diameter_m * spec.diameter_m / 4.0;
    let h_gas_in = solvent::h_gas_w(&gas_in.n, gas_in.t_k)?;
    let h_lean = solvent.h_liquid_w(lean.n_mea, lean.n_h2o, lean.n_co2, lean.t_k)?;
    let m_lean = lean.n_mea * MW_MEA + lean.n_h2o * MW_WATER;
    let g_total: f64 = gas_in.n.iter().sum();
    let s_c = feed_c + lean.n_co2;
    let s_w = 1e-2 * g_total;
    let s_e = m_lean * CP_SOLVENT_J_KG_K;
    let _ = mixture::total(&gas_in.n)?;
    Ok(Column {
        spec,
        solvent,
        n: spec.slices,
        area,
        dz: spec.height_m / spec.slices as f64,
        p: spec.p_pa,
        inert,
        feed_c,
        feed_w,
        feed_t: gas_in.t_k,
        n_mea: lean.n_mea,
        lean_c: lean.n_co2,
        lean_w: lean.n_h2o,
        lean_t: lean.t_k,
        h_gas_in,
        h_lean,
        s_c,
        s_w,
        s_e,
    })
}

fn finish(
    col: &Column<'_>,
    gas_in: &Stream,
    lean: &SolventStream,
    out: newton::Outcome,
) -> Result<AbsorberResult> {
    let x = out.x;
    let n = col.n;
    let mut slices = Vec::with_capacity(n);
    for j in 0..n {
        let (g, l) = col.unpack(&x, j);
        let (gin, lin) = col.inlets(&x, j);
        let (gm, lm) = Column::mid(g, gin, l, lin);
        slices.push(col.rates(j, gm, lm)?.state);
    }
    let (g_top, _) = col.unpack(&x, n - 1);
    let (_, l_bot) = col.unpack(&x, 0);
    let mut gas_out = gas_in.clone();
    gas_out.tag = format!("{}_GAS_OUT", col.spec.tag);
    gas_out.n = col.gas_flows(g_top);
    gas_out.t_k = g_top.t;
    gas_out.p_pa = col.p;
    let rich = SolventStream {
        tag: format!("{}_RICH", col.spec.tag),
        t_k: l_bot.t,
        p_pa: col.p,
        n_mea: col.n_mea,
        n_h2o: l_bot.w,
        n_co2: l_bot.c,
    };
    // Element closure over gas and liquid. MEA is C2H7NO.
    let liquid_el = |s: &SolventStream| -> [f64; 4] {
        [
            2.0 * s.n_mea + s.n_co2,
            7.0 * s.n_mea + 2.0 * s.n_h2o,
            s.n_mea + 2.0 * s.n_co2 + s.n_h2o,
            s.n_mea,
        ]
    };
    let e_in = add(gas_in.elements()?, liquid_el(lean));
    let e_out = add(gas_out.elements()?, liquid_el(&rich));
    let mut element_rel = [0.0; 4];
    for k in 0..4 {
        let s = e_in[k].abs().max(e_out[k].abs());
        element_rel[k] = if s > 0.0 {
            (e_in[k] - e_out[k]).abs() / s
        } else {
            0.0
        };
    }
    let h_in = col.h_gas_in + col.h_lean;
    let h_out = col.h_gas(g_top)?
        + col.h_liq(Liq {
            c: rich.n_co2,
            w: rich.n_h2o,
            t: rich.t_k,
        })?;
    let energy_scale = h_in.abs().max(h_out.abs()).max(1.0);
    Ok(AbsorberResult {
        height_m: col.dz * col.n as f64,
        capture: capture(col.feed_c, g_top.c)?,
        gas_out,
        rich,
        slices,
        closure: Closure {
            element_rel,
            energy_rel: (h_in - h_out).abs() / energy_scale,
        },
        residual: out.residual,
        iterations: out.iterations,
        path: out.path,
        continued_in_pressure: false,
        doubled_grid: false,
        notes: col.solvent.take_notes(),
        x,
    })
}

fn add(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn solve_once(
    spec: &AbsorberSpec,
    gas_in: &Stream,
    lean: &SolventStream,
    warm: Option<&[f64]>,
) -> Result<AbsorberResult> {
    let col = build(spec, gas_in, lean)?;
    let x0 = match warm {
        Some(w) if w.len() == col.n * NB => w.to_vec(),
        _ => col.guess(),
    };
    let shape = col.shape();
    let out = newton::solve(|x| col.residuals(x), &x0, &shape, TOL)?;
    finish(&col, gas_in, lean, out)
}

/// Finite-volume residuals of the column (H.14 to H.20) at unknowns `x`, scaled.
/// The CO2 flows in `x` are natural logarithms of mol/s.
/// Per slice: CO2 rate, water equilibrium, gas energy, CO2 and water conservation,
/// and total energy. Gas enters slice 0 at the bottom and liquid enters the top
/// slice (H.25).
///
/// eq: H.14
/// eq: H.15
/// eq: H.16
/// eq: H.17
/// eq: H.18
/// eq: H.19
/// eq: H.20
/// eq: H.25
pub fn rhs(
    spec: &AbsorberSpec,
    gas_in: &Stream,
    lean: &SolventStream,
    x: &[f64],
) -> Result<Vec<f64>> {
    let col = build(spec, gas_in, lean)?;
    if x.len() != col.n * NB {
        return Err(EngineError::Range {
            param: "x".into(),
            msg: format!("{} unknowns for {} slices", x.len(), col.n),
        });
    }
    col.residuals(x)
}

/// Solve the column with the fallback ladder of plan section 5: Newton, then
/// pseudo-transient continuation, then continuation in pressure from 1 bar, then
/// a doubled grid. Failure of every rung returns `NotConverged`.
pub fn solve(
    spec: &AbsorberSpec,
    gas_in: &Stream,
    lean: &SolventStream,
    warm: Option<&[f64]>,
) -> Result<AbsorberResult> {
    let first = match solve_once(spec, gas_in, lean, warm) {
        Ok(r) => return Ok(r),
        Err(e @ EngineError::Range { .. }) => return Err(e),
        Err(e) => e,
    };
    if spec.p_pa > 2.0e5 {
        let mut x: Option<Vec<f64>> = None;
        let mut ok = true;
        let steps = 6;
        for k in 0..=steps {
            let p = 1.0e5 * (spec.p_pa / 1.0e5).powf(k as f64 / steps as f64);
            let s = AbsorberSpec {
                p_pa: p,
                ..spec.clone()
            };
            match solve_once(&s, gas_in, lean, x.as_deref()) {
                Ok(r) => x = Some(r.x),
                Err(_) => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            if let Some(xw) = x {
                let mut r = solve_once(spec, gas_in, lean, Some(&xw))?;
                r.continued_in_pressure = true;
                return Ok(r);
            }
        }
    }
    let doubled = AbsorberSpec {
        slices: 2 * spec.slices,
        ..spec.clone()
    };
    match solve_once(&doubled, gas_in, lean, None) {
        Ok(mut r) => {
            r.doubled_grid = true;
            Ok(r)
        }
        Err(second) => Err(EngineError::NotConverged(format!(
            "absorber {}: {first}; doubled grid: {second}",
            spec.tag
        ))),
    }
}

/// Packed height, m, at which the column reaches `target` capture, between `z_lo`
/// and `z_hi`. The search starts at `spec.height_m` (clamped to the range) and
/// steps by a factor of 1.25 in height towards the target, each solve
/// warm-started from the last, then closes on the height by the Illinois form of
/// regula falsi to 1e-4 in capture. The height is `None` when even `z_hi` falls
/// short, and the result is then the column at `z_hi`.
pub fn height_for_capture(
    spec: &AbsorberSpec,
    gas_in: &Stream,
    lean: &SolventStream,
    target: f64,
    z_lo: f64,
    z_hi: f64,
) -> Result<(Option<f64>, AbsorberResult)> {
    let at = |z: f64, warm: Option<&[f64]>| -> Result<AbsorberResult> {
        solve(
            &AbsorberSpec {
                height_m: z,
                ..spec.clone()
            },
            gas_in,
            lean,
            warm,
        )
    };
    let mut z = spec.height_m.clamp(z_lo, z_hi);
    let mut r = at(z, None)?;
    let up = r.capture < target;
    let mut prev = (z, r.capture - target, r.x.clone());
    loop {
        let reached = r.capture >= target;
        if reached == up {
            break;
        }
        let edge = if up { z_hi } else { z_lo };
        if z == edge {
            return Ok(if up { (None, r) } else { (Some(z), r) });
        }
        prev = (z, r.capture - target, r.x.clone());
        z = if up {
            (z * 1.25).min(z_hi)
        } else {
            (z / 1.25).max(z_lo)
        };
        r = at(z, Some(&prev.2))?;
    }
    // Bracket [a, b] with f(a) < 0 <= f(b) in capture less target.
    let (mut a, mut b) = if up {
        ((prev.0, prev.1), (z, r.capture - target))
    } else {
        ((z, r.capture - target), (prev.0, prev.1))
    };
    let mut warm = r.x.clone();
    let mut best = r;
    let mut side = 0i8;
    for _ in 0..60 {
        if best.capture - target >= 0.0 && best.capture - target < 1e-4 {
            break;
        }
        let zn = b.0 - b.1 * (b.0 - a.0) / (b.1 - a.1);
        let rn = at(zn, Some(&warm))?;
        let fz = rn.capture - target;
        warm = rn.x.clone();
        if fz >= 0.0 {
            b = (zn, fz);
            if side == 1 {
                a.1 *= 0.5;
            }
            side = 1;
        } else {
            a = (zn, fz);
            if side == -1 {
                b.1 *= 0.5;
            }
            side = -1;
        }
        best = rn;
        if (b.0 - a.0).abs() < 1e-6 {
            break;
        }
    }
    if best.capture < target {
        best = at(b.0, Some(&warm))?;
    }
    Ok((Some(best.height_m), best))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_h_10_rocha_liquid_film_grows_with_root_velocity() {
        let a = kl_rocha(1.5e-9, 0.01, 0.014).unwrap();
        let b = kl_rocha(1.5e-9, 0.04, 0.014).unwrap();
        assert!((b / a - 2.0).abs() < 1e-12);
        assert!(a > 1e-5 && a < 1e-3, "{a}");
    }

    #[test]
    fn eq_h_12_billet_schultes_branches_meet_their_printed_forms() {
        // Re_L below 5 takes the 0.15 power, at or above 5 the 0.85 and 0.25 forms.
        let (a, rho, mu) = (250.0, 1000.0, 1.0e-3);
        let u_lo = 4.0 * a * mu / rho;
        let lo = area_bs(a, u_lo, rho, mu).unwrap() / a;
        let fr = u_lo * u_lo * a / G;
        assert!((lo - C_H * 4.0f64.powf(0.15) * fr.powf(0.1)).abs() < 1e-12);
        let u_hi = 10.0 * a * mu / rho;
        let hi = area_bs(a, u_hi, rho, mu).unwrap() / a;
        let fr = u_hi * u_hi * a / G;
        assert!((hi - 0.85 * C_H * 10.0f64.powf(0.25) * fr.powf(0.1)).abs() < 1e-12);
    }

    #[test]
    fn eq_h_06_enhancement_follows_hatta_when_reagent_is_plentiful() {
        let e = enhancement(1.5e-9, 1.0e-9, 6000.0, 4.0, 1e-6, 1.0e-4).unwrap();
        assert!((e.e / e.hatta - 1.0).abs() < 0.01, "{e:?}");
        // Little free amine: E falls towards 1.
        let f = enhancement(1.5e-9, 1.0e-9, 6000.0, 1e-6, 0.5, 1.0e-4).unwrap();
        assert!(f.e < 1.01, "{f:?}");
    }

    #[test]
    fn eq_h_01_flux_balances_both_films() {
        let fi = FluxInputs {
            p_co2_pa: 1.0e4,
            pstar_pa: 100.0,
            k_g: Some(1.0e-6),
            k_l0: 1.0e-4,
            henry: 3000.0,
            d_al: 1.5e-9,
            d_b: 1.0e-9,
            k2: 6000.0,
            c_bl: 3.0,
        };
        let f = flux(&fi).unwrap();
        let gas = 1.0e-6 * (fi.p_co2_pa - f.p_i_pa);
        assert!((gas / f.n - 1.0).abs() < 1e-8, "{gas} {}", f.n);
        assert!(f.p_i_pa > fi.pstar_pa && f.p_i_pa < fi.p_co2_pa);
    }

    #[test]
    fn eq_h_28_solvent_rate_returns_the_anchored_circulation_form() {
        // 1 mol/s of CO2 between 0.25 and 0.50 at 30 mass% needs 4 mol/s of MEA.
        let m = solvent_rate(1.0, 0.30, 0.50, 0.25).unwrap();
        assert!((m - 4.0 * MW_MEA / 0.30).abs() < 1e-12);
        assert!(solvent_rate(1.0, 0.30, 0.25, 0.25).is_err());
    }

    #[test]
    fn eq_h_29_diameter_from_liquid_load() {
        let d = diameter(1.0 / 3600.0 * 100.0, 100.0).unwrap();
        assert!((PI * d * d / 4.0 - 1.0).abs() < 1e-12);
    }
}
