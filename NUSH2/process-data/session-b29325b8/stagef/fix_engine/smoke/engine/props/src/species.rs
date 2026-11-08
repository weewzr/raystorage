//! Pure-species ideal-gas data on a formation-enthalpy basis.
//!
//! Sources, all from the NIST Chemistry WebBook (SRD 69), gas phase thermochemistry:
//!
//! - CH4, CO, CO2, H2, H2O, N2: Shomate rows, Chase 1998 (NIST-JANAF).
//! - C2H6: Cp, Gurvich, Veyts et al. 1989. Formation enthalpy, Pittam and Pilcher 1972.
//! - C3H8: Cp, Chao, Wilhoit and Zwolinski 1973. Formation enthalpy, Pittam and Pilcher 1972.
//! - n-C4H10: Cp, Chen, Wilhoit and Zwolinski 1975. Formation enthalpy, Pittam and Pilcher 1972.
//! - n-C5H12: Cp, Scott 1974. Formation enthalpy, Good 1970.
//!
//! The formation enthalpies of the four alkanes are the combustion-calorimetry
//! entries of the WebBook with the smallest stated uncertainty.
//!
//! The WebBook prints no Shomate row for these alkanes and no entropy for three
//! of them, so `s_j_mol_k` and `g_j_mol` return a range error for all four. The
//! reforming model converts them fully, so no equilibrium needs their Gibbs energy.
//!
//! NIST recheck of 2026-11-02: a script compared all 14 Shomate rows and all 56 Cp
//! nodes here with the raw WebBook pages and found no mismatch. The 9 Shomate rows
//! the earlier repository carried are identical to these, so no WebBook value had
//! to replace one. The Cp tables now run from 298.15 K to 1500 K, where that
//! repository stopped at 400 to 700 K.
//!
//! H2O vapour has no Shomate row below 500 K, so below 500 K its temperature
//! dependence comes from IF97 at 100 Pa, offset to meet the Shomate row at 500 K.
//! Liquid water sits on the same IF97 datum, so the latent heat in any energy
//! balance is the IF97 one.

use crate::shomate::{CpTable, Shomate, ShomateRange, ShomateTable};
use crate::steam;
use nush2_numerics::{EngineError, Result};

/// Number of tracked gas species.
pub const NSP: usize = 10;

/// Gas species, in the order of `nush2_units::stream::Sp`.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sp {
    H2,
    Co,
    Co2,
    Ch4,
    H2o,
    N2,
    C2h6,
    C3h8,
    Nc4h10,
    Nc5h12,
}

/// All species, in index order.
pub const ALL: [Sp; NSP] = [
    Sp::H2,
    Sp::Co,
    Sp::Co2,
    Sp::Ch4,
    Sp::H2o,
    Sp::N2,
    Sp::C2h6,
    Sp::C3h8,
    Sp::Nc4h10,
    Sp::Nc5h12,
];

/// kg/mol of C, H, O, N. These are the atomic weights behind the WebBook
/// molecular weights: 72.1488 for C5H12, 28.0134 for N2, 18.0153 for H2O and
/// 28.0101 for CO all follow from them to the printed digits.
const ATOMIC_WEIGHT: [f64; 4] = [12.0107e-3, 1.00794e-3, 15.9994e-3, 14.0067e-3];

impl Sp {
    /// Position in a flow array.
    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Sp::H2 => "H2",
            Sp::Co => "CO",
            Sp::Co2 => "CO2",
            Sp::Ch4 => "CH4",
            Sp::H2o => "H2O",
            Sp::N2 => "N2",
            Sp::C2h6 => "C2H6",
            Sp::C3h8 => "C3H8",
            Sp::Nc4h10 => "n-C4H10",
            Sp::Nc5h12 => "n-C5H12",
        }
    }

    /// Atoms of C, H, O, N per molecule.
    pub fn atoms(self) -> [u32; 4] {
        match self {
            Sp::H2 => [0, 2, 0, 0],
            Sp::Co => [1, 0, 1, 0],
            Sp::Co2 => [1, 0, 2, 0],
            Sp::Ch4 => [1, 4, 0, 0],
            Sp::H2o => [0, 2, 1, 0],
            Sp::N2 => [0, 0, 0, 2],
            Sp::C2h6 => [2, 6, 0, 0],
            Sp::C3h8 => [3, 8, 0, 0],
            Sp::Nc4h10 => [4, 10, 0, 0],
            Sp::Nc5h12 => [5, 12, 0, 0],
        }
    }

    /// Molar mass, kg/mol.
    pub fn mw_kg_mol(self) -> f64 {
        self.atoms()
            .iter()
            .zip(ATOMIC_WEIGHT)
            .map(|(&n, w)| f64::from(n) * w)
            .sum()
    }
}

const fn row(t_min_k: f64, t_max_k: f64, c: [f64; 8]) -> ShomateRange {
    ShomateRange {
        t_min_k,
        t_max_k,
        coeff: Shomate {
            a: c[0],
            b: c[1],
            c: c[2],
            d: c[3],
            e: c[4],
            f: c[5],
            g: c[6],
            h: c[7],
        },
    }
}

/// CH4, Chase 1998, webbook.nist.gov ID C74828.
pub const CH4: ShomateTable = ShomateTable {
    name: "CH4",
    rows: &[
        row(
            298.0,
            1300.0,
            [
                -0.703029, 108.4773, -42.52157, 5.862788, 0.678565, -76.84376, 158.7163, -74.87310,
            ],
        ),
        row(
            1300.0,
            6000.0,
            [
                85.81217, 11.26467, -2.114146, 0.138190, -26.42221, -153.5327, 224.4143, -74.87310,
            ],
        ),
    ],
};

/// CO, Chase 1998, webbook.nist.gov ID C630080.
pub const CO: ShomateTable = ShomateTable {
    name: "CO",
    rows: &[
        row(
            298.0,
            1300.0,
            [
                25.56759, 6.096130, 4.054656, -2.671301, 0.131021, -118.0089, 227.3665, -110.5271,
            ],
        ),
        row(
            1300.0,
            6000.0,
            [
                35.15070, 1.300095, -0.205921, 0.013550, -3.282780, -127.8375, 231.7120, -110.5271,
            ],
        ),
    ],
};

/// CO2, Chase 1998, webbook.nist.gov ID C124389.
pub const CO2: ShomateTable = ShomateTable {
    name: "CO2",
    rows: &[
        row(
            298.0,
            1200.0,
            [
                24.99735, 55.18696, -33.69137, 7.948387, -0.136638, -403.6075, 228.2431, -393.5224,
            ],
        ),
        row(
            1200.0,
            6000.0,
            [
                58.16639, 2.720074, -0.492289, 0.038844, -6.447293, -425.9186, 263.6125, -393.5224,
            ],
        ),
    ],
};

/// H2, Chase 1998, webbook.nist.gov ID C1333740.
pub const H2: ShomateTable = ShomateTable {
    name: "H2",
    rows: &[
        row(
            298.0,
            1000.0,
            [
                33.066178, -11.363417, 11.432816, -2.772874, -0.158558, -9.980797, 172.707974, 0.0,
            ],
        ),
        row(
            1000.0,
            2500.0,
            [
                18.563083, 12.257357, -2.859786, 0.268238, 1.977990, -1.147438, 156.288133, 0.0,
            ],
        ),
        row(
            2500.0,
            6000.0,
            [
                43.413560, -4.293079, 1.272428, -0.096876, -20.533862, -38.515158, 162.081354, 0.0,
            ],
        ),
    ],
};

/// H2O vapour, Chase 1998, webbook.nist.gov ID C7732185. Starts at 500 K.
pub const H2O: ShomateTable = ShomateTable {
    name: "H2O",
    rows: &[
        row(
            500.0,
            1700.0,
            [
                30.09200, 6.832514, 6.793435, -2.534480, 0.082139, -250.8810, 223.3967, -241.8264,
            ],
        ),
        row(
            1700.0,
            6000.0,
            [
                41.96426, 8.622053, -1.499780, 0.098119, -11.15764, -272.1797, 219.7809, -241.8264,
            ],
        ),
    ],
};

/// N2, Chase 1998, webbook.nist.gov ID C7727379.
pub const N2: ShomateTable = ShomateTable {
    name: "N2",
    rows: &[
        row(
            100.0,
            500.0,
            [
                28.98641, 1.853978, -9.647459, 16.63537, 0.000117, -8.671914, 226.4168, 0.0,
            ],
        ),
        row(
            500.0,
            2000.0,
            [
                19.50583, 19.88705, -8.598535, 1.369784, 0.527601, -4.935202, 212.3900, 0.0,
            ],
        ),
        row(
            2000.0,
            6000.0,
            [
                35.51872, 1.128728, -0.196103, 0.014662, -4.553760, -18.97091, 224.9810, 0.0,
            ],
        ),
    ],
};

/// C2H6, webbook.nist.gov ID C74840.
pub const C2H6: CpTable = CpTable {
    name: "C2H6",
    hf298_j_mol: -83.8e3,
    points: &[
        (298.15, 52.49),
        (300.0, 52.71),
        (400.0, 65.46),
        (500.0, 77.94),
        (600.0, 89.19),
        (700.0, 99.14),
        (800.0, 107.94),
        (900.0, 115.71),
        (1000.0, 122.55),
        (1100.0, 128.55),
        (1200.0, 133.80),
        (1300.0, 138.39),
        (1400.0, 142.40),
        (1500.0, 145.90),
    ],
};

/// C3H8, webbook.nist.gov ID C74986.
pub const C3H8: CpTable = CpTable {
    name: "C3H8",
    hf298_j_mol: -104.7e3,
    points: &[
        (298.15, 73.60),
        (300.0, 73.93),
        (400.0, 94.01),
        (500.0, 112.59),
        (600.0, 128.70),
        (700.0, 142.67),
        (800.0, 154.77),
        (900.0, 165.35),
        (1000.0, 174.60),
        (1100.0, 182.67),
        (1200.0, 189.74),
        (1300.0, 195.85),
        (1400.0, 201.21),
        (1500.0, 205.89),
    ],
};

/// n-C4H10, webbook.nist.gov ID C106978.
pub const NC4H10: CpTable = CpTable {
    name: "n-C4H10",
    hf298_j_mol: -125.6e3,
    points: &[
        (298.15, 98.49),
        (300.0, 98.95),
        (400.0, 124.77),
        (500.0, 148.66),
        (600.0, 169.28),
        (700.0, 187.02),
        (800.0, 202.38),
        (900.0, 215.73),
        (1000.0, 227.36),
        (1100.0, 237.48),
        (1200.0, 246.27),
        (1300.0, 253.93),
        (1400.0, 260.58),
        (1500.0, 266.40),
    ],
};

/// n-C5H12, webbook.nist.gov ID C109660. C6 enters the plant lumped into this
/// species, 1.2 mol per mol, so the carbon flow stays right.
pub const NC5H12: CpTable = CpTable {
    name: "n-C5H12",
    hf298_j_mol: -146.8e3,
    points: &[
        (298.15, 120.0),
        (300.0, 120.62),
        (400.0, 152.55),
        (500.0, 182.59),
        (600.0, 208.78),
        (700.0, 231.38),
        (800.0, 250.62),
        (900.0, 266.94),
        (1000.0, 281.58),
        (1100.0, 293.72),
        (1200.0, 304.60),
        (1300.0, 313.80),
        (1400.0, 322.17),
        (1500.0, 330.54),
    ],
};

/// Temperature where the H2O vapour Shomate row starts, K.
pub const H2O_BRANCH_K: f64 = 500.0;
/// Standard enthalpy of liquid water at 298.15 K less vapour, CODATA, J/mol.
/// WebBook ID C7732185, Cox, Wagman et al. 1984: -285.830 and -241.826 kJ/mol.
pub const CODATA_LATENT_298_J_MOL: f64 = 44.004e3;

enum Data {
    Poly(&'static ShomateTable),
    Tab(&'static CpTable),
}

fn data(sp: Sp) -> Data {
    match sp {
        Sp::H2 => Data::Poly(&H2),
        Sp::Co => Data::Poly(&CO),
        Sp::Co2 => Data::Poly(&CO2),
        Sp::Ch4 => Data::Poly(&CH4),
        Sp::H2o => Data::Poly(&H2O),
        Sp::N2 => Data::Poly(&N2),
        Sp::C2h6 => Data::Tab(&C2H6),
        Sp::C3h8 => Data::Tab(&C3H8),
        Sp::Nc4h10 => Data::Tab(&NC4H10),
        Sp::Nc5h12 => Data::Tab(&NC5H12),
    }
}

/// Molar enthalpy of H2O vapour at the point where the Shomate row starts, J/mol.
fn h2o_branch_h() -> Result<f64> {
    let row = H2O.row(H2O_BRANCH_K)?;
    Ok(row.hf298_j_mol() + row.dh_j_mol(H2O_BRANCH_K))
}

/// Ideal-gas heat capacity, J/mol/K.
pub fn cp_j_mol_k(sp: Sp, t_k: f64) -> Result<f64> {
    if sp == Sp::H2o && t_k < H2O_BRANCH_K {
        return Ok(steam::cp_dilute_j_kg_k(t_k)? * sp.mw_kg_mol());
    }
    match data(sp) {
        Data::Poly(t) => t.cp_j_mol_k(t_k),
        Data::Tab(t) => t.cp_j_mol_k(t_k),
    }
}

/// Ideal-gas molar enthalpy on the formation basis, J/mol.
///
/// eq: A.9
/// eq: A.10
pub fn h_j_mol(sp: Sp, t_k: f64) -> Result<f64> {
    if sp == Sp::H2o && t_k < H2O_BRANCH_K {
        let rise = steam::h_dilute_j_kg(H2O_BRANCH_K)? - steam::h_dilute_j_kg(t_k)?;
        return Ok(h2o_branch_h()? - rise * sp.mw_kg_mol());
    }
    match data(sp) {
        Data::Poly(t) => {
            let row = t.row(t_k)?;
            Ok(row.hf298_j_mol() + row.dh_j_mol(t_k))
        }
        Data::Tab(t) => Ok(t.hf298_j_mol + t.dh_j_mol(t_k)?),
    }
}

/// Standard entropy at 1 bar, J/mol/K. Range error for C2+, which carry none.
pub fn s_j_mol_k(sp: Sp, t_k: f64) -> Result<f64> {
    if sp == Sp::H2o && t_k < H2O_BRANCH_K {
        let rise = steam::s_dilute_j_kg_k(H2O_BRANCH_K)? - steam::s_dilute_j_kg_k(t_k)?;
        return Ok(H2O.s_j_mol_k(H2O_BRANCH_K)? - rise * sp.mw_kg_mol());
    }
    match data(sp) {
        Data::Poly(t) => t.s_j_mol_k(t_k),
        Data::Tab(t) => Err(EngineError::Range {
            param: format!("entropy of {}", t.name),
            msg: "no entropy is held for this species".into(),
        }),
    }
}

/// Standard Gibbs energy at 1 bar on the formation basis, J/mol.
pub fn g_j_mol(sp: Sp, t_k: f64) -> Result<f64> {
    Ok(h_j_mol(sp, t_k)? - t_k * s_j_mol_k(sp, t_k)?)
}

/// Molar enthalpy of liquid water at (T, p) on the formation basis, J/mol.
pub fn h_liquid_water_j_mol(t_k: f64, p_pa: f64) -> Result<f64> {
    let gap = steam::h_dilute_j_kg(t_k)? - steam::h_liquid_j_kg(t_k, p_pa)?;
    Ok(h_j_mol(Sp::H2o, t_k)? - gap * Sp::H2o.mw_kg_mol())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn molar_masses_match_the_webbook_digits() {
        let close = |sp: Sp, want: f64| {
            assert!(
                (sp.mw_kg_mol() * 1000.0 - want).abs() < 6e-5,
                "{}",
                sp.name()
            );
        };
        close(Sp::Nc5h12, 72.1488);
        close(Sp::N2, 28.0134);
        close(Sp::H2o, 18.0153);
        close(Sp::Co, 28.0101);
    }

    #[test]
    fn species_order_matches_the_index() {
        for (i, sp) in ALL.iter().enumerate() {
            assert_eq!(sp.index(), i);
        }
    }

    #[test]
    fn enthalpy_at_298_is_the_formation_enthalpy_for_the_alkanes() {
        assert_eq!(h_j_mol(Sp::C2h6, 298.15).unwrap(), -83.8e3);
        assert_eq!(h_j_mol(Sp::Nc5h12, 298.15).unwrap(), -146.8e3);
    }

    #[test]
    fn alkane_cp_nodes_are_returned_at_the_nodes() {
        assert_eq!(cp_j_mol_k(Sp::C3h8, 1000.0).unwrap(), 174.60);
        assert_eq!(cp_j_mol_k(Sp::Nc4h10, 600.0).unwrap(), 169.28);
    }

    #[test]
    fn alkanes_carry_no_entropy() {
        assert!(s_j_mol_k(Sp::C2h6, 500.0).is_err());
        assert!(g_j_mol(Sp::Nc5h12, 500.0).is_err());
    }

    #[test]
    fn enthalpy_rises_with_temperature_for_every_species() {
        for sp in ALL {
            let mut last = f64::NEG_INFINITY;
            for t in (300..=1300).step_by(50) {
                let h = h_j_mol(sp, f64::from(t)).unwrap();
                assert!(h > last, "{} at {t}", sp.name());
                last = h;
            }
        }
    }
}
