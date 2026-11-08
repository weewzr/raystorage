//! Syngas coolers: HPS-GEN1, WGS-EX1 to EX4 and the KO-DRM cooling step.
//!
//! Class Rigorous on the stream side: duty is the plain enthalpy difference at a
//! set outlet temperature, with the water that condenses held in the outlet
//! stream at the `flash::split` equilibrium. There is no exchanger sizing. Case 5
//! of TEV-961 stands in for Case 6, whose stream sheets are absent from the local
//! PDF.

use crate::flash;
use crate::gas::closure;
use crate::stream::{Boundary, Closure, EnergyKind, EnergyStream, ModelClass, Stream, UnitRecord};
use nush2_numerics::{EngineError, Result};

/// One cooler.
#[derive(Debug, Clone, PartialEq)]
pub struct CoolerSpec {
    pub tag: String,
    pub t_out_k: f64,
    pub p_out_pa: f64,
    pub outlet_tag: String,
}

/// Outcome of a cooler.
#[derive(Debug, Clone, PartialEq)]
pub struct CoolerResult {
    pub outlet: Stream,
    /// Heat removed from the gas, W.
    pub duty_w: f64,
    /// mol/s of water that condensed in this cooler.
    pub condensed_mol_s: f64,
    pub closure: Closure,
    pub record: UnitRecord,
}

/// Cool `inlet` to the set outlet state.
pub fn cool(inlet: &Stream, spec: &CoolerSpec) -> Result<CoolerResult> {
    if spec.t_out_k > inlet.t_k {
        return Err(EngineError::Range {
            param: "t_out_k".into(),
            msg: format!(
                "{} K lies above the inlet {} K on {}",
                spec.t_out_k, inlet.t_k, spec.tag
            ),
        });
    }
    let sp = flash::split(&inlet.n, spec.t_out_k, spec.p_out_pa)?;
    let outlet = Stream {
        tag: spec.outlet_tag.clone(),
        t_k: spec.t_out_k,
        p_pa: spec.p_out_pa,
        n: inlet.n,
        n_liq_h2o: sp.n_liq_h2o,
        n_co2_diss: sp.n_co2_diss,
    };
    let duty_w = inlet.h_w()? - outlet.h_w()?;
    let closure = closure(&[inlet], &[&outlet], -duty_w)?;
    let record = UnitRecord {
        tag: spec.tag.clone(),
        class: ModelClass::Rigorous,
        boundary: Boundary::Inside,
        inlets: vec![inlet.tag.clone()],
        outlets: vec![outlet.tag.clone()],
        duties: vec![EnergyStream {
            tag: format!("Q_{}", spec.tag),
            kind: EnergyKind::Heat,
            w: duty_w,
            t_hot_k: Some(inlet.t_k),
            t_cold_k: Some(spec.t_out_k),
            from: spec.tag.clone(),
            to: "UTILITY".to_string(),
        }],
        closure: closure.clone(),
        flags: Vec::new(),
    };
    Ok(CoolerResult {
        condensed_mol_s: sp.n_liq_h2o - inlet.n_liq_h2o,
        outlet,
        duty_w,
        closure,
        record,
    })
}

/// Heat released cooling `inlet` to `t_limit_k` at the inlet pressure, W.
pub fn heat_above(inlet: &Stream, t_limit_k: f64) -> Result<f64> {
    let spec = CoolerSpec {
        tag: "CURVE".to_string(),
        t_out_k: t_limit_k,
        p_out_pa: inlet.p_pa,
        outlet_tag: "CURVE_OUT".to_string(),
    };
    Ok(cool(inlet, &spec)?.duty_w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closes;
    use crate::stream::{Sp, NSP};

    fn gas(t: f64) -> Stream {
        let mut n = [0.0; NSP];
        n[Sp::H2 as usize] = 60.0;
        n[Sp::Co2 as usize] = 15.0;
        n[Sp::H2o as usize] = 25.0;
        Stream::gas("IN", t, 27.0e5, n)
    }

    fn spec(t: f64) -> CoolerSpec {
        CoolerSpec {
            tag: "EX".into(),
            t_out_k: t,
            p_out_pa: 27.0e5,
            outlet_tag: "OUT".into(),
        }
    }

    #[test]
    fn sensible_cooling_matches_the_mean_heat_capacity() {
        let s = gas(700.0);
        let r = cool(&s, &spec(600.0)).unwrap();
        closes!(r.closure);
        let cp = nush2_props::mixture::cp_j_mol_k(&s.n, 650.0).unwrap();
        let rough = s.total() * cp * 100.0;
        assert!((r.duty_w / rough - 1.0).abs() < 0.01);
        assert_eq!(r.condensed_mol_s, 0.0);
    }

    #[test]
    fn condensation_adds_latent_heat() {
        let s = gas(450.0);
        let r = cool(&s, &spec(320.0)).unwrap();
        closes!(r.closure);
        assert!(r.condensed_mol_s > 20.0);
        let sensible = s.total() * 35.0 * 130.0;
        assert!(r.duty_w > 2.0 * sensible);
    }

    #[test]
    fn heating_through_a_cooler_is_refused() {
        assert!(cool(&gas(400.0), &spec(500.0)).is_err());
    }
}
