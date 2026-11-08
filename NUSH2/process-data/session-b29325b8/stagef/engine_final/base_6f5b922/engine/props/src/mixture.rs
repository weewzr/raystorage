//! Ideal-gas mixtures of the tracked species. Flow arrays are mol/s by `Sp` index.

use crate::species::{self, Sp, ALL, NSP};
use crate::units::{BAR_PA, R_GAS};
use nush2_numerics::{EngineError, Result};

/// Species flows, mol/s, by `Sp` index.
pub type Flows = [f64; NSP];

/// Total flow, mol/s. Refuses a negative or non-finite flow, or an empty stream.
pub fn total(n: &Flows) -> Result<f64> {
    for (sp, v) in ALL.iter().zip(n) {
        if !(v.is_finite() && *v >= 0.0) {
            return Err(EngineError::Range {
                param: format!("flow of {}", sp.name()),
                msg: format!("{v} mol/s must be finite and not negative"),
            });
        }
    }
    let sum: f64 = n.iter().sum();
    if sum > 0.0 {
        Ok(sum)
    } else {
        Err(EngineError::Range {
            param: "flows".into(),
            msg: "the total flow must be positive".into(),
        })
    }
}

/// Mole fractions.
pub fn mole_fractions(n: &Flows) -> Result<Flows> {
    let sum = total(n)?;
    Ok(n.map(|v| v / sum))
}

/// Mass flow, kg/s.
pub fn mass_kg_s(n: &Flows) -> Result<f64> {
    total(n)?;
    Ok(ALL.iter().zip(n).map(|(s, v)| s.mw_kg_mol() * v).sum())
}

/// Mean molar mass, kg/mol.
pub fn mean_mw_kg_mol(n: &Flows) -> Result<f64> {
    Ok(mass_kg_s(n)? / total(n)?)
}

/// Element flows of C, H, O, N, mol/s.
pub fn elements(n: &Flows) -> Result<[f64; 4]> {
    total(n)?;
    let mut out = [0.0; 4];
    for (sp, v) in ALL.iter().zip(n) {
        for (element, atoms) in out.iter_mut().zip(sp.atoms()) {
            *element += f64::from(atoms) * v;
        }
    }
    Ok(out)
}

/// Sum of `f(species)` weighted by flow, skipping absent species so that a range
/// limit of a species with no flow cannot stop the calculation.
fn sum_present<F>(n: &Flows, mut f: F) -> Result<f64>
where
    F: FnMut(Sp) -> Result<f64>,
{
    let mut out = 0.0;
    for (sp, v) in ALL.iter().zip(n) {
        if *v > 0.0 {
            out += v * f(*sp)?;
        }
    }
    Ok(out)
}

/// Molar heat capacity of the gas, J/mol/K.
pub fn cp_j_mol_k(n: &Flows, t_k: f64) -> Result<f64> {
    let sum = total(n)?;
    Ok(sum_present(n, |sp| species::cp_j_mol_k(sp, t_k))? / sum)
}

/// Molar enthalpy of the gas on the formation basis, J/mol.
pub fn h_j_mol(n: &Flows, t_k: f64) -> Result<f64> {
    let sum = total(n)?;
    Ok(sum_present(n, |sp| species::h_j_mol(sp, t_k))? / sum)
}

/// Enthalpy flow of a stream on the formation basis, W. `n_liq_h2o` is the part
/// of the H2O flow held as liquid, and the rest of the flows are ideal gas.
pub fn h_flow_w(n: &Flows, n_liq_h2o: f64, t_k: f64, p_pa: f64) -> Result<f64> {
    total(n)?;
    let n_h2o = n[Sp::H2o.index()];
    if !(n_liq_h2o >= 0.0 && n_liq_h2o <= n_h2o) {
        return Err(EngineError::Range {
            param: "n_liq_h2o".into(),
            msg: format!("{n_liq_h2o} mol/s must lie between 0 and the H2O flow {n_h2o}"),
        });
    }
    let mut gas = *n;
    gas[Sp::H2o.index()] = n_h2o - n_liq_h2o;
    let vapour = sum_present(&gas, |sp| species::h_j_mol(sp, t_k))?;
    if n_liq_h2o > 0.0 {
        Ok(vapour + n_liq_h2o * species::h_liquid_water_j_mol(t_k, p_pa)?)
    } else {
        Ok(vapour)
    }
}

/// Molar entropy of the ideal mixture at (T, p), J/mol/K. Range error if a C2+
/// species is present, since none of them carries an entropy here.
pub fn s_j_mol_k(n: &Flows, t_k: f64, p_pa: f64) -> Result<f64> {
    let sum = total(n)?;
    if !(p_pa > 0.0 && p_pa.is_finite()) {
        return Err(EngineError::Range {
            param: "p_pa".into(),
            msg: format!("{p_pa} Pa must be positive"),
        });
    }
    let ln_p = (p_pa / BAR_PA).ln();
    let total_s = sum_present(n, |sp| {
        let y = n[sp.index()] / sum;
        Ok(species::s_j_mol_k(sp, t_k)? - R_GAS * (y.ln() + ln_p))
    })?;
    Ok(total_s / sum)
}

/// Density of the gas as an ideal gas, kg/m3.
pub fn density_ideal_kg_m3(n: &Flows, t_k: f64, p_pa: f64) -> Result<f64> {
    if !(t_k > 0.0 && p_pa > 0.0 && t_k.is_finite() && p_pa.is_finite()) {
        return Err(EngineError::Range {
            param: "t_k and p_pa".into(),
            msg: format!("{t_k} K and {p_pa} Pa must be positive"),
        });
    }
    Ok(p_pa * mean_mw_kg_mol(n)? / (R_GAS * t_k))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flows(pairs: &[(Sp, f64)]) -> Flows {
        let mut n = [0.0; NSP];
        for (sp, v) in pairs {
            n[sp.index()] = *v;
        }
        n
    }

    #[test]
    fn fractions_sum_to_one_and_mass_follows_composition() {
        let n = flows(&[(Sp::H2, 3.0), (Sp::Co, 1.0)]);
        let y = mole_fractions(&n).unwrap();
        assert!((y.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        let m = 3.0 * Sp::H2.mw_kg_mol() + Sp::Co.mw_kg_mol();
        assert!((mass_kg_s(&n).unwrap() - m).abs() < 1e-15);
        assert!((mean_mw_kg_mol(&n).unwrap() - m / 4.0).abs() < 1e-15);
    }

    #[test]
    fn elements_count_every_atom() {
        let n = flows(&[
            (Sp::Ch4, 1.0),
            (Sp::H2o, 2.0),
            (Sp::N2, 0.5),
            (Sp::C2h6, 1.0),
        ]);
        assert_eq!(elements(&n).unwrap(), [3.0, 14.0, 2.0, 1.0]);
    }

    #[test]
    fn bad_flows_are_refused() {
        assert!(total(&[0.0; NSP]).is_err());
        assert!(total(&flows(&[(Sp::H2, -1.0)])).is_err());
        assert!(total(&flows(&[(Sp::H2, f64::NAN)])).is_err());
    }

    #[test]
    fn enthalpy_flow_is_the_sum_of_the_species() {
        let n = flows(&[(Sp::H2, 2.0), (Sp::H2o, 1.0)]);
        let t = 700.0;
        let want =
            2.0 * species::h_j_mol(Sp::H2, t).unwrap() + species::h_j_mol(Sp::H2o, t).unwrap();
        assert!((h_flow_w(&n, 0.0, t, 1.0e5).unwrap() - want).abs() < 1e-9);
    }

    #[test]
    fn condensing_one_mol_releases_the_latent_heat() {
        let n = flows(&[(Sp::H2, 2.0), (Sp::H2o, 1.0)]);
        let (t, p) = (373.15, 2.0e5);
        let dry = h_flow_w(&n, 0.0, t, p).unwrap();
        let wet = h_flow_w(&n, 1.0, t, p).unwrap();
        // About 40.6 kJ/mol near the normal boiling point.
        let released = dry - wet;
        assert!(released > 40.0e3 && released < 41.2e3, "got {released}");
    }

    #[test]
    fn liquid_flow_cannot_exceed_the_water_present() {
        let n = flows(&[(Sp::H2o, 1.0)]);
        assert!(h_flow_w(&n, 1.5, 350.0, 1.0e5).is_err());
        assert!(h_flow_w(&n, -0.1, 350.0, 1.0e5).is_err());
    }

    #[test]
    fn absent_species_cannot_trip_a_range_limit() {
        // H2O has no data below 273.15 K and N2 does.
        let n = flows(&[(Sp::N2, 1.0)]);
        assert!(h_j_mol(&n, 200.0).is_ok());
    }

    #[test]
    fn entropy_of_mixing_is_positive() {
        let t = 800.0;
        let pure = flows(&[(Sp::H2, 1.0)]);
        let mix = flows(&[(Sp::H2, 1.0), (Sp::Co, 1.0)]);
        let s_pure = s_j_mol_k(&pure, t, 1.0e5).unwrap();
        let s_co = species::s_j_mol_k(Sp::Co, t).unwrap();
        let want = 0.5 * (s_pure + s_co) + R_GAS * 2.0f64.ln();
        assert!((s_j_mol_k(&mix, t, 1.0e5).unwrap() - want).abs() < 1e-9);
    }

    #[test]
    fn ideal_density_of_nitrogen() {
        let n = flows(&[(Sp::N2, 1.0)]);
        // 1 bar and 300 K.
        let rho = density_ideal_kg_m3(&n, 300.0, 1.0e5).unwrap();
        assert!((rho - 1.1231).abs() < 1e-3, "got {rho}");
    }
}
