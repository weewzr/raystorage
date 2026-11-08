//! FLASH: isothermal vapour-liquid split of cooled syngas.
//!
//! Class ReducedOrder. Rachford-Rice with an ideal vapour. Water follows Raoult
//! with IF97 Psat, and CO2 follows Henry with Aronu 2011 Eq 17 (PDF p4), read as
//! Pa m3/mol. Every other gas is held insoluble, because the library has no
//! solubility for H2, CH4 or CO. The heat of solution of CO2 is neglected, since
//! dissolved CO2 stays on its ideal-gas enthalpy.
//!
//! Solving the split with K of the insoluble gases at infinity gives
//!
//!   g(b) = z_nc / b + z_w (K_w - 1) / (1 + b (K_w - 1))
//!                   + z_c (K_c - 1) / (1 + b (K_c - 1)) = 0
//!
//! which falls monotonically in the vapour fraction b.

use crate::gas::closure;
use crate::solve::brent;
use crate::stream::{Boundary, Closure, ModelClass, Sp, Stream, StreamTag, UnitRecord, NSP};
use nush2_mea::aronu::henry_water_co2_pa_m3_mol;
use nush2_numerics::{EngineError, Result};
use nush2_props::{species, steam};

/// Solver limits.
const VF_TOL: f64 = 1.0e-10;
const VF_ITERS: usize = 200;
/// Liquid water is not sought above this temperature, K.
const T_NO_LIQUID_K: f64 = 640.0;
const H2O: usize = Sp::H2o as usize;
const CO2: usize = Sp::Co2 as usize;

/// Phase split of a mixture at (T, p).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Split {
    /// mol/s of the H2O flow that sits in the liquid.
    pub n_liq_h2o: f64,
    /// mol/s of the CO2 flow dissolved in that liquid.
    pub n_co2_diss: f64,
    /// Vapour fraction on total moles.
    pub vapour_fraction: f64,
    /// True when no liquid forms.
    pub single_phase: bool,
}

/// CO2 Henry constant on the mole-fraction scale, Pa.
///
/// eq: F.6
fn henry_x_pa(t_k: f64, p_pa: f64) -> Result<f64> {
    let v = steam::state(t_k, p_pa)?.v_m3_kg;
    let c_w = 1.0 / (v * species::Sp::H2o.mw_kg_mol());
    Ok(henry_water_co2_pa_m3_mol(t_k) * c_w)
}

/// Vapour-liquid split of `n` at (T, p).
///
/// eq: F.3
/// eq: F.5
/// eq: F.6
/// eq: F.7
/// eq: F.12
pub fn split(n: &[f64; NSP], t_k: f64, p_pa: f64) -> Result<Split> {
    let vapour_only = Split {
        n_liq_h2o: 0.0,
        n_co2_diss: 0.0,
        vapour_fraction: 1.0,
        single_phase: true,
    };
    if !(t_k > 0.0 && p_pa > 0.0 && t_k.is_finite() && p_pa.is_finite()) {
        return Err(EngineError::Range {
            param: "t_k and p_pa".into(),
            msg: format!("{t_k} K and {p_pa} Pa must be positive"),
        });
    }
    let total: f64 = n.iter().sum();
    if total.is_nan() || total <= 0.0 {
        return Err(EngineError::Range {
            param: "n".into(),
            msg: "the flash needs a positive total flow".into(),
        });
    }
    let n_w = n[H2O];
    if n_w <= 0.0 || t_k >= T_NO_LIQUID_K {
        return Ok(vapour_only);
    }
    let ps = steam::psat_pa(t_k)?;
    if p_pa <= ps {
        return Ok(vapour_only);
    }
    let k_w = ps / p_pa;
    let k_c = henry_x_pa(t_k, p_pa)? / p_pa;
    let z_w = n_w / total;
    let z_c = n[CO2] / total;
    let z_nc = 1.0 - z_w - z_c;
    // At b = 1 the gas holds every mole, so this is the dew test.
    if z_w / k_w + z_c / k_c <= 1.0 {
        return Ok(vapour_only);
    }
    let g = |b: f64| -> Result<f64> {
        Ok(z_nc / b
            + z_w * (k_w - 1.0) / (1.0 + b * (k_w - 1.0))
            + z_c * (k_c - 1.0) / (1.0 + b * (k_c - 1.0)))
    };
    // Liquid must keep a positive amount of CO2 in the last term.
    let lo = if z_nc > 0.0 { 1.0e-12 } else { 1.0e-9 };
    let b = brent(g, lo, 1.0, VF_TOL, VF_ITERS)?;
    let l = total * (1.0 - b);
    let v = total * b;
    let x_w = n_w / (l + v * k_w);
    let x_c = n[CO2] / (l + v * k_c);
    Ok(Split {
        n_liq_h2o: l * x_w,
        n_co2_diss: l * x_c,
        vapour_fraction: b,
        single_phase: false,
    })
}

/// Outcome of a flash drum.
#[derive(Debug, Clone, PartialEq)]
pub struct FlashResult {
    pub vapour: Stream,
    pub liquid: Option<Stream>,
    pub split: Split,
    pub closure: Closure,
    pub record: UnitRecord,
}

/// Flash at the given (T, p), with no heat term. A hotter or colder feed would
/// need a cooler ahead of the drum, so the feed must already sit at `t_k`.
///
/// eq: F.8
pub fn flash(
    feed: &Stream,
    t_k: f64,
    p_pa: f64,
    tag: &str,
    vapour_tag: &str,
    liquid_tag: &str,
) -> Result<FlashResult> {
    let sp = split(&feed.n, t_k, p_pa)?;
    let mut n_v = feed.n;
    n_v[H2O] -= sp.n_liq_h2o;
    n_v[CO2] -= sp.n_co2_diss;
    let vapour = Stream::gas(vapour_tag, t_k, p_pa, n_v);
    let liquid = if sp.single_phase {
        None
    } else {
        let mut n_l = [0.0; NSP];
        n_l[H2O] = sp.n_liq_h2o;
        n_l[CO2] = sp.n_co2_diss;
        Some(Stream {
            tag: liquid_tag.to_string(),
            t_k,
            p_pa,
            n: n_l,
            n_liq_h2o: sp.n_liq_h2o,
            n_co2_diss: sp.n_co2_diss,
        })
    };
    let mut outs: Vec<&Stream> = vec![&vapour];
    if let Some(l) = &liquid {
        outs.push(l);
    }
    // The drum carries no heat term, so the enthalpy gap of a feed that arrives
    // at another temperature shows up in the closure.
    let closure = closure(&[feed], &outs, 0.0)?;
    let mut outlets: Vec<StreamTag> = vec![vapour_tag.to_string()];
    if liquid.is_some() {
        outlets.push(liquid_tag.to_string());
    }
    let flags = if sp.single_phase {
        vec!["no liquid forms at these conditions".to_string()]
    } else {
        Vec::new()
    };
    let record = UnitRecord {
        tag: tag.to_string(),
        class: ModelClass::ReducedOrder,
        boundary: Boundary::Inside,
        inlets: vec![feed.tag.clone()],
        outlets,
        duties: Vec::new(),
        closure: closure.clone(),
        flags,
    };
    Ok(FlashResult {
        vapour,
        liquid,
        split: sp,
        closure,
        record,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closes;

    fn syngas(h2o: f64) -> [f64; NSP] {
        let mut n = [0.0; NSP];
        n[Sp::H2 as usize] = 70.0;
        n[Sp::Co2 as usize] = 18.0;
        n[Sp::Ch4 as usize] = 5.0;
        n[H2O] = h2o;
        n
    }

    #[test]
    fn eq_f_05_saturated_vapour_follows_psat() {
        let (t, p) = (313.15, 26.8e5);
        let n = syngas(50.0);
        let s = split(&n, t, p).unwrap();
        assert!(!s.single_phase);
        let n_v_w = n[H2O] - s.n_liq_h2o;
        let n_v: f64 = n.iter().sum::<f64>() - s.n_liq_h2o - s.n_co2_diss;
        let y_w = n_v_w / n_v;
        let x_w = s.n_liq_h2o / (s.n_liq_h2o + s.n_co2_diss);
        let want = x_w * steam::psat_pa(t).unwrap() / p;
        assert!((y_w / want - 1.0).abs() < 1e-8, "{y_w} against {want}");
    }

    #[test]
    fn eq_f_03_dry_or_hot_gas_stays_vapour() {
        let s = split(&syngas(0.01), 313.15, 26.8e5).unwrap();
        assert!(s.single_phase);
        let s = split(&syngas(50.0), 500.0, 5.0e5).unwrap();
        assert!(s.single_phase);
        let s = split(&syngas(50.0), 900.0, 30.0e5).unwrap();
        assert!(s.single_phase);
    }

    #[test]
    fn eq_f_06_dissolved_co2_follows_henry() {
        let (t, p) = (313.15, 26.8e5);
        let n = syngas(50.0);
        let s = split(&n, t, p).unwrap();
        let n_v: f64 = n.iter().sum::<f64>() - s.n_liq_h2o - s.n_co2_diss;
        let y_c = (n[CO2] - s.n_co2_diss) / n_v;
        let x_c = s.n_co2_diss / (s.n_liq_h2o + s.n_co2_diss);
        let want = y_c * p / henry_x_pa(t, p).unwrap();
        assert!((x_c / want - 1.0).abs() < 1e-8);
    }

    #[test]
    fn eq_f_08_drum_closes_and_keeps_every_atom() {
        let (t, p) = (313.15, 26.8e5);
        let mut feed = Stream::gas("F", t, p, syngas(50.0));
        let sp = split(&feed.n, t, p).unwrap();
        feed.n_liq_h2o = sp.n_liq_h2o;
        feed.n_co2_diss = sp.n_co2_diss;
        let r = flash(&feed, t, p, "KO", "V", "L").unwrap();
        closes!(r.closure);
        assert!(r.liquid.is_some());
    }

    #[test]
    fn bad_inputs_are_refused() {
        let n = syngas(50.0);
        assert!(split(&n, -1.0, 1.0e5).is_err());
        assert!(split(&n, 300.0, f64::NAN).is_err());
        assert!(split(&[0.0; NSP], 300.0, 1.0e5).is_err());
    }
}
