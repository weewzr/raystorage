//! Helpers on `Stream` shared by the gas-side units: enthalpy, element counts, dry
//! fractions and the closure check every unit reports.

use crate::stream::{Closure, Sp, Stream, NSP};
use nush2_numerics::{EngineError, Result};
use nush2_props::mixture;

/// Element closure limit of the shared unit test macro.
pub const ELEMENT_TOL: f64 = 1.0e-9;
/// Energy closure limit of the shared unit test macro.
pub const ENERGY_TOL: f64 = 1.0e-6;

/// Asserts element closure 1e-9 and energy closure 1e-6 on a `Closure`.
#[macro_export]
macro_rules! closes {
    ($c:expr) => {{
        let c: &$crate::stream::Closure = &$c;
        for (i, v) in c.element_rel.iter().enumerate() {
            assert!(
                *v < $crate::gas::ELEMENT_TOL,
                "element {i} closure {v} misses the limit"
            );
        }
        assert!(
            c.energy_rel < $crate::gas::ENERGY_TOL,
            "energy closure {} misses the limit",
            c.energy_rel
        );
    }};
}

impl Stream {
    /// All-vapour stream.
    pub fn gas(tag: impl Into<String>, t_k: f64, p_pa: f64, n: [f64; NSP]) -> Self {
        Stream {
            tag: tag.into(),
            t_k,
            p_pa,
            n,
            n_liq_h2o: 0.0,
            n_co2_diss: 0.0,
        }
    }

    /// Flow of one species, mol/s.
    pub fn flow(&self, sp: Sp) -> f64 {
        self.n[sp as usize]
    }

    /// Total flow, mol/s.
    pub fn total(&self) -> f64 {
        self.n.iter().sum()
    }

    /// Enthalpy flow on the formation basis, W.
    pub fn h_w(&self) -> Result<f64> {
        mixture::h_flow_w(&self.n, self.n_liq_h2o, self.t_k, self.p_pa)
    }

    /// Mass flow, kg/s.
    pub fn mass_kg_s(&self) -> Result<f64> {
        mixture::mass_kg_s(&self.n)
    }

    /// Element flows of C, H, O, N, mol/s.
    pub fn elements(&self) -> Result<[f64; 4]> {
        mixture::elements(&self.n)
    }

    /// Mole fraction of `sp` on the water-free basis. All water is left out,
    /// liquid and vapour alike, which is how a dry-gas analysis reads.
    pub fn dry_fraction(&self, sp: Sp) -> Result<f64> {
        if matches!(sp, Sp::H2o) {
            return Err(EngineError::Range {
                param: "sp".into(),
                msg: "H2O has no fraction on the dry basis".into(),
            });
        }
        let dry = self.total() - self.flow(Sp::H2o);
        if dry > 0.0 {
            Ok(self.flow(sp) / dry)
        } else {
            Err(EngineError::Range {
                param: "stream".into(),
                msg: format!("{} carries no dry gas", self.tag),
            })
        }
    }

    /// Mole fractions, all water counted.
    pub fn wet_fractions(&self) -> Result<[f64; NSP]> {
        mixture::mole_fractions(&self.n)
    }
}

/// Closure of a unit: `heat_in_w` is the net heat added to the process side, so a
/// cooler passes a negative value.
pub fn closure(inlets: &[&Stream], outlets: &[&Stream], heat_in_w: f64) -> Result<Closure> {
    let mut e_in = [0.0; 4];
    let mut e_out = [0.0; 4];
    let (mut h_in, mut h_out) = (0.0, 0.0);
    for s in inlets {
        for (a, b) in e_in.iter_mut().zip(s.elements()?) {
            *a += b;
        }
        h_in += s.h_w()?;
    }
    for s in outlets {
        for (a, b) in e_out.iter_mut().zip(s.elements()?) {
            *a += b;
        }
        h_out += s.h_w()?;
    }
    let mut element_rel = [0.0; 4];
    for k in 0..4 {
        let scale = e_in[k].abs().max(e_out[k].abs());
        element_rel[k] = if scale > 0.0 {
            (e_in[k] - e_out[k]).abs() / scale
        } else {
            0.0
        };
    }
    let scale = h_in.abs().max(h_out.abs()).max(heat_in_w.abs()).max(1.0);
    Ok(Closure {
        element_rel,
        energy_rel: (h_in + heat_in_w - h_out).abs() / scale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wet() -> Stream {
        let mut n = [0.0; NSP];
        n[Sp::H2 as usize] = 3.0;
        n[Sp::Co2 as usize] = 1.0;
        n[Sp::H2o as usize] = 4.0;
        Stream::gas("T", 700.0, 30.0e5, n)
    }

    #[test]
    fn dry_fraction_leaves_water_out() {
        let s = wet();
        assert!((s.dry_fraction(Sp::H2).unwrap() - 0.75).abs() < 1e-15);
        assert!(s.dry_fraction(Sp::H2o).is_err());
        assert!((s.total() - 8.0).abs() < 1e-15);
    }

    #[test]
    fn closure_of_an_identity_is_zero() {
        let s = wet();
        let c = closure(&[&s], &[&s], 0.0).unwrap();
        closes!(c);
    }

    #[test]
    fn closure_sees_a_missing_heat_term() {
        let a = wet();
        let mut b = wet();
        b.t_k = 600.0;
        let q = a.h_w().unwrap() - b.h_w().unwrap();
        let good = closure(&[&a], &[&b], -q).unwrap();
        closes!(good);
        let bad = closure(&[&a], &[&b], 0.0).unwrap();
        assert!(bad.energy_rel > 1e-3);
    }
}
