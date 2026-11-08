//! SMR-EQ: reformer outlet at chemical equilibrium.
//!
//! Class Rigorous. Two extents at the set outlet temperature and pressure, ideal
//! gas, K(T) from `nush2_props::equilibrium`, C2+ converted fully to CO and H2
//! ahead of the two extents. The 871 C outlet is the INL assumption and is
//! disclosed as such: INL reached it by Gibbs minimisation with no tube model, and
//! the tested mock-up ran 102 to 125 K below its helium.
//!
//! Duty is the plain enthalpy rise of the stream on the formation basis, so it
//! holds sensible heat and heat of reaction in one number.

use crate::gas::closure;
use crate::stream::{Boundary, Closure, EnergyKind, EnergyStream, ModelClass, Stream, UnitRecord};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::{self, KSource, Method};

/// Outlet condition of the reformer.
#[derive(Debug, Clone, PartialEq)]
pub struct SmrEqSpec {
    pub tag: String,
    pub t_out_k: f64,
    pub p_out_pa: f64,
    pub k_source: KSource,
    pub outlet_tag: String,
}

/// Outcome of the equilibrium reformer.
#[derive(Debug, Clone, PartialEq)]
pub struct SmrEqResult {
    pub outlet: Stream,
    /// Heat taken up by the gas, W.
    pub duty_w: f64,
    /// Share of the hydrocarbon carbon fed that left as CO or CO2.
    pub conversion: f64,
    pub method: Method,
    pub closure: Closure,
    pub record: UnitRecord,
}

/// Solve the unit.
pub fn solve(feed: &Stream, spec: &SmrEqSpec) -> Result<SmrEqResult> {
    if feed.n_liq_h2o != 0.0 {
        return Err(EngineError::Range {
            param: "feed".into(),
            msg: format!("{} carries liquid water into the reformer", feed.tag),
        });
    }
    let eq = equilibrium::reform(&feed.n, spec.t_out_k, spec.p_out_pa, spec.k_source)?;
    let outlet = Stream::gas(spec.outlet_tag.clone(), spec.t_out_k, spec.p_out_pa, eq.n);
    let duty_w = outlet.h_w()? - feed.h_w()?;
    let closure = closure(&[feed], &[&outlet], duty_w)?;
    let record = UnitRecord {
        tag: spec.tag.clone(),
        class: ModelClass::Rigorous,
        boundary: Boundary::Inside,
        inlets: vec![feed.tag.clone()],
        outlets: vec![outlet.tag.clone()],
        duties: vec![EnergyStream {
            tag: format!("Q_{}", spec.tag),
            kind: EnergyKind::Heat,
            w: duty_w,
            t_hot_k: None,
            t_cold_k: Some(spec.t_out_k),
            from: "HELIUM".to_string(),
            to: spec.tag.clone(),
        }],
        closure: closure.clone(),
        flags: vec!["hot-end approach assumed from INL; no tested tube shows it".to_string()],
    };
    Ok(SmrEqResult {
        outlet,
        duty_w,
        conversion: eq.conversion,
        method: eq.method,
        closure,
        record,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closes;
    use crate::stream::{Sp, NSP};

    fn spec(t: f64, p: f64) -> SmrEqSpec {
        SmrEqSpec {
            tag: "SMR".into(),
            t_out_k: t,
            p_out_pa: p,
            k_source: KSource::Zhang,
            outlet_tag: "S05".into(),
        }
    }

    fn feed() -> Stream {
        let mut n = [0.0; NSP];
        n[Sp::Ch4 as usize] = 1.0;
        n[Sp::H2o as usize] = 3.98;
        Stream::gas("S04", 800.0, 41.9e5, n)
    }

    #[test]
    fn te1236_conversion_point_and_closure() {
        // TE1236 Table 2.1 prints 79.8% at 860 C and 41.9 bar, S/C 3.98. A pure
        // methane feed is the printed case.
        let r = solve(&feed(), &spec(1133.15, 41.9e5)).unwrap();
        closes!(r.closure);
        assert!((r.conversion - 0.798).abs() < 0.005, "{}", r.conversion);
        assert!(r.duty_w > 0.0);
    }

    #[test]
    fn liquid_in_the_feed_is_refused() {
        let mut f = feed();
        f.n_liq_h2o = 1.0;
        assert!(solve(&f, &spec(1133.15, 41.9e5)).is_err());
    }
}
