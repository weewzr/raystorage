//! Water-gas shift: CO + H2O = CO2 + H2.
//!
//! WGS-HTS, class Empirical. Adiabatic plug flow on the pseudo-first-order form of
//! Chinchen et al. 1984 (PDF p6): dpCO/dt = k (pCO - pCO,eq), integrated form
//! k = Sv ln(1 / (1 - C/Ce)), with Table 1 k(T) at 31 bar. Sv and t follow the
//! gas at reaction conditions through the empty reactor, because k stays near
//! constant from 16 to 31 bar only on that reading. Chinchen ran a steam ratio of
//! 1.0 against about 0.5 here, and no correction for that is applied.
//!
//! The bed volume of the plant is not published anywhere in the library, so
//! `HtsSize` offers two bases. `Ghsv` takes a space velocity, with Chinchen's own
//! test value of 1.75 m3/h of dry gas at 20 C and 1 bar over 350 ml, which is 5000
//! per hour, as the only printed figure. `OutletApproach` sizes the bed to stop at
//! a set Q/K, and `ieaghg_hts_q_over_k` gives that target from the HTS outlet
//! printed by IEAGHG 2017-02, a flowsheet independent of the INL cases. The rate
//! law then returns the bed volume as an output.
//!
//! WGS-LTS, class ReducedOrder. Adiabatic with a 10 K approach to equilibrium,
//! taken from the printed outlet states of TEV-961 Case 5 and IEAGHG. The library
//! holds no Cu/Zn kinetics.

use crate::gas::closure;
use crate::solve::{brent, rk45};
use crate::stream::{Boundary, Closure, ModelClass, Sp, Stream, UnitRecord, NSP};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;
use nush2_props::mixture;
use nush2_props::units::R_GAS;

/// LTS approach to equilibrium, K. Plan section 9.
pub const LTS_APPROACH_K: f64 = 10.0;
/// Chinchen semi-technical space velocity, dry gas at 20 C and 1 bar, 1/h.
/// 1.75 m3/h over 350 ml, PDF p6.
pub const CHINCHEN_GHSV_DRY_1_H: f64 = 1.75 / 350.0e-6;
/// IEAGHG 2017-02 stream 6, HTS outlet, PDF p44, mole fractions of CO2, CO, H2, H2O.
const IEAGHG_HTS_OUT: (f64, f64, f64, f64) = (0.1283, 0.0366, 0.5961, 0.2137);
/// IEAGHG 2017-02 stream 6 temperature, K. 412 C, PDF p44.
const IEAGHG_HTS_OUT_T_K: f64 = 412.0 + 273.15;
/// Reference state of that space velocity.
const T_REF_K: f64 = 293.15;
const P_REF_PA: f64 = 1.0e5;

const H2: usize = Sp::H2 as usize;
const CO: usize = Sp::Co as usize;
const CO2: usize = Sp::Co2 as usize;
const H2O: usize = Sp::H2o as usize;

/// Chinchen Table 1, deg C.
const TABLE1_T_C: [f64; 7] = [310.0, 330.0, 350.0, 370.0, 400.0, 425.0, 450.0];
/// Chinchen Table 1, 31 bar, pellet S (15-5, 5.4 x 3.6 mm), 1/s.
const TABLE1_K_S_31BAR: [f64; 7] = [0.017, 0.043, 0.092, 0.190, 0.404, 0.586, 0.918];
/// Chinchen Table 1, 31 bar, pellet L (15-4, 8.5 x 11.0 mm), 1/s.
const TABLE1_K_L_31BAR: [f64; 7] = [0.017, 0.043, 0.097, 0.154, 0.256, 0.358, 0.565];

/// Pellet size of Chinchen Table 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pellet {
    /// 15-5, 5.4 x 3.6 mm.
    S,
    /// 15-4, 8.5 x 11.0 mm.
    L,
}

/// Rate constant, 1/s, by ln k on 1/T between the 31 bar points of Table 1.
pub fn k_chinchen(t_k: f64, pellet: Pellet) -> Result<f64> {
    let t_c = t_k - 273.15;
    let (lo, hi) = (TABLE1_T_C[0], TABLE1_T_C[6]);
    if !(lo..=hi).contains(&t_c) {
        return Err(EngineError::Range {
            param: "t_k".into(),
            msg: format!("{t_c} C lies outside the Chinchen table, {lo} to {hi} C"),
        });
    }
    let table = match pellet {
        Pellet::S => &TABLE1_K_S_31BAR,
        Pellet::L => &TABLE1_K_L_31BAR,
    };
    let mut i = 0;
    while i < 5 && t_c > TABLE1_T_C[i + 1] {
        i += 1;
    }
    let inv = |c: f64| 1.0 / (c + 273.15);
    let w = (1.0 / t_k - inv(TABLE1_T_C[i])) / (inv(TABLE1_T_C[i + 1]) - inv(TABLE1_T_C[i]));
    Ok((table[i].ln() * (1.0 - w) + table[i + 1].ln() * w).exp())
}

/// Extent of the shift that reaches `k2`, mol/s. Positive runs forward.
pub fn extent_at(n: &[f64; NSP], k2: f64) -> Result<f64> {
    let (co, h2o, co2, h2) = (n[CO], n[H2O], n[CO2], n[H2]);
    let a = 1.0 - k2;
    let b = co2 + h2 + k2 * (co + h2o);
    let c = co2 * h2 - k2 * co * h2o;
    let disc = b * b - 4.0 * a * c;
    let x = -2.0 * c / (b + disc.max(0.0).sqrt());
    if !(k2 > 0.0 && x.is_finite() && x < co.min(h2o) && x > -co2.min(h2)) {
        return Err(EngineError::Range {
            param: "shift feed".into(),
            msg: format!("no shift extent for K = {k2} on this feed"),
        });
    }
    Ok(x)
}

fn shifted(n: &[f64; NSP], x: f64) -> [f64; NSP] {
    let mut out = *n;
    out[CO] -= x;
    out[H2O] -= x;
    out[CO2] += x;
    out[H2] += x;
    out
}

/// Gas temperature that holds `h_w` on composition `n`, by Newton on cp.
pub fn t_from_h(n: &[f64; NSP], h_w: f64, t_guess: f64) -> Result<f64> {
    let total = mixture::total(n)?;
    let mut t = t_guess;
    for _ in 0..60 {
        let f = mixture::h_flow_w(n, 0.0, t, 1.0e5)? - h_w;
        let cp = mixture::cp_j_mol_k(n, t)?;
        let dt = f / (total * cp);
        t -= dt;
        if dt.abs() < 1.0e-9 {
            return Ok(t);
        }
    }
    Err(EngineError::NotConverged(format!(
        "temperature from enthalpy, last value {t} K"
    )))
}

/// Outcome of a shift bed.
#[derive(Debug, Clone, PartialEq)]
pub struct ShiftResult {
    pub outlet: Stream,
    /// Extent, mol/s.
    pub xi: f64,
    /// Q / K2 at the outlet temperature, on the K source of the run.
    pub q_over_k: f64,
    /// Temperature where K2 equals the outlet Q, K.
    pub t_eq_k: f64,
    /// Bed volume, m3. Zero for the LTS, which has no rate model.
    pub bed_volume_m3: f64,
    pub closure: Closure,
    pub record: UnitRecord,
}

fn reaction_quotient(n: &[f64; NSP]) -> f64 {
    n[CO2] * n[H2] / (n[CO] * n[H2O])
}

/// Temperature where K2 equals `q`, K.
fn t_of_k2(q: f64, source: KSource) -> Result<f64> {
    brent(|t| Ok(source.pair(t)?.1 - q), 320.0, 1500.0, 1.0e-6, 200)
}

fn check_inlet(inlet: &Stream) -> Result<()> {
    if inlet.n_liq_h2o != 0.0 {
        return Err(EngineError::Range {
            param: "inlet".into(),
            msg: format!("{} is two-phase at the shift inlet", inlet.tag),
        });
    }
    if !(inlet.n[CO] > 0.0 && inlet.n[H2O] > 0.0) {
        return Err(EngineError::Range {
            param: "inlet".into(),
            msg: format!("{} needs CO and H2O for the shift", inlet.tag),
        });
    }
    Ok(())
}

/// Identity of a bed for the shared result builder.
struct Bed<'a> {
    tag: &'a str,
    class: ModelClass,
    source: KSource,
    volume_m3: f64,
    flags: Vec<String>,
}

fn finish(bed: Bed<'_>, inlet: &Stream, outlet: Stream, xi: f64) -> Result<ShiftResult> {
    let Bed {
        tag,
        class,
        source,
        volume_m3: bed_volume_m3,
        flags,
    } = bed;
    let closure = closure(&[inlet], &[&outlet], 0.0)?;
    let q = reaction_quotient(&outlet.n);
    let q_over_k = q / source.pair(outlet.t_k)?.1;
    let t_eq_k = t_of_k2(q, source)?;
    let record = UnitRecord {
        tag: tag.to_string(),
        class,
        boundary: Boundary::Inside,
        inlets: vec![inlet.tag.clone()],
        outlets: vec![outlet.tag.clone()],
        duties: Vec::new(),
        closure: closure.clone(),
        flags,
    };
    Ok(ShiftResult {
        outlet,
        xi,
        q_over_k,
        t_eq_k,
        bed_volume_m3,
        closure,
        record,
    })
}

/// Q/K2 of the printed IEAGHG HTS outlet on the given K source.
pub fn ieaghg_hts_q_over_k(source: KSource) -> Result<f64> {
    let (co2, co, h2, h2o) = IEAGHG_HTS_OUT;
    Ok(co2 * h2 / (co * h2o) / source.pair(IEAGHG_HTS_OUT_T_K)?.1)
}

/// How the HTS bed is sized.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HtsSize {
    /// Space velocity on dry gas at 20 C and 1 bar, 1/h.
    Ghsv(f64),
    /// Bed long enough to reach this Q/K2 at the outlet, below 1.
    OutletApproach(f64),
}

/// High-temperature shift bed.
#[derive(Debug, Clone, PartialEq)]
pub struct HtsSpec {
    pub tag: String,
    pub size: HtsSize,
    pub pellet: Pellet,
    pub p_out_pa: f64,
    pub k_source: KSource,
    pub outlet_tag: String,
}

/// Extent at the end of a bed of `v_bed` m3.
fn hts_extent(inlet: &Stream, spec: &HtsSpec, h_in: f64, v_bed: f64) -> Result<f64> {
    let n0 = inlet.n;
    let total = inlet.total();
    let (p0, p1) = (inlet.p_pa, spec.p_out_pa);
    let mut t_last = inlet.t_k;
    let rhs = |v: f64, xi: f64| -> Result<f64> {
        let n = shifted(&n0, xi);
        let t = t_from_h(&n, h_in, t_last)?;
        t_last = t;
        let p = p0 + (p1 - p0) * v / v_bed;
        let q_m3_s = total * R_GAS * t / p;
        let k2 = spec.k_source.pair(t)?.1;
        let xi_eq = extent_at(&n0, k2)?;
        Ok(k_chinchen(t, spec.pellet)? * (xi_eq - xi) / q_m3_s)
    };
    rk45(rhs, 0.0, 0.0, v_bed, 1.0e-8 * total, 20_000)
}

/// Adiabatic plug-flow HTS.
pub fn hts(inlet: &Stream, spec: &HtsSpec) -> Result<ShiftResult> {
    check_inlet(inlet)?;
    let h_in = inlet.h_w()?;
    let n0 = inlet.n;
    let dry_m3_h = (inlet.total() - n0[H2O]) * R_GAS * T_REF_K / P_REF_PA * 3600.0;
    let q_over_k_at = |v: f64| -> Result<f64> {
        let xi = hts_extent(inlet, spec, h_in, v)?;
        let n = shifted(&n0, xi);
        let t = t_from_h(&n, h_in, inlet.t_k)?;
        Ok(reaction_quotient(&n) / spec.k_source.pair(t)?.1)
    };
    let v_bed = match spec.size {
        HtsSize::Ghsv(g) => {
            if !(g > 0.0 && g.is_finite()) {
                return Err(EngineError::Range {
                    param: "ghsv".into(),
                    msg: format!("{g} per hour must be positive"),
                });
            }
            dry_m3_h / g
        }
        HtsSize::OutletApproach(target) => {
            if !(target > 0.0 && target < 1.0) {
                return Err(EngineError::Range {
                    param: "q_over_k".into(),
                    msg: format!("{target} must lie between 0 and 1"),
                });
            }
            let mut hi = dry_m3_h / CHINCHEN_GHSV_DRY_1_H;
            let mut tries = 0;
            while q_over_k_at(hi)? < target {
                hi *= 2.0;
                tries += 1;
                if tries > 12 {
                    return Err(EngineError::NotConverged(format!(
                        "HTS bed for Q/K {target} above {hi} m3"
                    )));
                }
            }
            brent(
                |v| Ok(q_over_k_at(v)? - target),
                1.0e-6 * hi,
                hi,
                1.0e-6 * hi,
                100,
            )?
        }
    };
    let xi = hts_extent(inlet, spec, h_in, v_bed)?;
    let n = shifted(&n0, xi);
    let t = t_from_h(&n, h_in, inlet.t_k)?;
    let outlet = Stream::gas(spec.outlet_tag.clone(), t, spec.p_out_pa, n);
    let flags = vec![
        match spec.size {
            HtsSize::Ghsv(g) => {
                format!("bed volume assumed from a space velocity of {g} per hour dry")
            }
            HtsSize::OutletApproach(q) => {
                format!("bed sized to an outlet Q/K of {q:.3}, no plant bed volume is published")
            }
        },
        "Chinchen steam ratio 1.0, no correction for the lower plant ratio".to_string(),
    ];
    let bed = Bed {
        tag: &spec.tag,
        class: ModelClass::Empirical,
        source: spec.k_source,
        volume_m3: v_bed,
        flags,
    };
    finish(bed, inlet, outlet, xi)
}

/// Low-temperature shift bed.
#[derive(Debug, Clone, PartialEq)]
pub struct LtsSpec {
    pub tag: String,
    /// Equilibrium sits this far above the outlet temperature, K.
    pub approach_k: f64,
    pub p_out_pa: f64,
    pub k_source: KSource,
    pub outlet_tag: String,
}

/// Adiabatic LTS at a set approach to equilibrium.
pub fn lts(inlet: &Stream, spec: &LtsSpec) -> Result<ShiftResult> {
    check_inlet(inlet)?;
    if !(spec.approach_k >= 0.0 && spec.approach_k.is_finite()) {
        return Err(EngineError::Range {
            param: "approach_k".into(),
            msg: format!("{} K must not be negative", spec.approach_k),
        });
    }
    let h_in = inlet.h_w()?;
    let n0 = inlet.n;
    let extent =
        |t: f64| -> Result<f64> { extent_at(&n0, spec.k_source.pair(t + spec.approach_k)?.1) };
    let gap = |t: f64| -> Result<f64> {
        let n = shifted(&n0, extent(t)?);
        mixture::h_flow_w(&n, 0.0, t, spec.p_out_pa).map(|h| h - h_in)
    };
    let t = brent(gap, inlet.t_k - 1.0, inlet.t_k + 250.0, 1.0e-7, 200)?;
    let xi = extent(t)?;
    let n = shifted(&n0, xi);
    let outlet = Stream::gas(spec.outlet_tag.clone(), t, spec.p_out_pa, n);
    let flags = vec![format!(
        "{} K approach assumed, no Cu/Zn kinetics in the library",
        spec.approach_k
    )];
    let bed = Bed {
        tag: &spec.tag,
        class: ModelClass::ReducedOrder,
        source: spec.k_source,
        volume_m3: 0.0,
        flags,
    };
    finish(bed, inlet, outlet, xi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closes;

    fn raw() -> Stream {
        let mut n = [0.0; NSP];
        n[H2] = 48.8;
        n[CO] = 8.7;
        n[CO2] = 4.8;
        n[Sp::N2 as usize] = 0.7;
        n[H2O] = 33.4;
        n[Sp::Ch4 as usize] = 3.6;
        Stream::gas("IN", 608.15, 30.0e5, n)
    }

    #[test]
    fn rate_constant_returns_the_table_at_its_nodes() {
        for (i, c) in TABLE1_T_C.iter().enumerate() {
            let k = k_chinchen(c + 273.15, Pellet::S).unwrap();
            assert!((k / TABLE1_K_S_31BAR[i] - 1.0).abs() < 1e-10);
            let k = k_chinchen(c + 273.15, Pellet::L).unwrap();
            assert!((k / TABLE1_K_L_31BAR[i] - 1.0).abs() < 1e-10);
        }
        assert!(k_chinchen(500.0, Pellet::S).is_err());
        assert!(k_chinchen(900.0, Pellet::L).is_err());
    }

    #[test]
    fn extent_reaches_the_equilibrium_constant() {
        let s = raw();
        for k2 in [0.5, 1.0, 5.0, 40.0] {
            let n = shifted(&s.n, extent_at(&s.n, k2).unwrap());
            assert!((reaction_quotient(&n) / k2 - 1.0).abs() < 1e-10, "{k2}");
        }
    }

    #[test]
    fn hts_closes_and_a_huge_bed_reaches_equilibrium() {
        let s = raw();
        let spec = HtsSpec {
            tag: "HTS".into(),
            size: HtsSize::Ghsv(100.0),
            pellet: Pellet::S,
            p_out_pa: 29.0e5,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        };
        let r = hts(&s, &spec).unwrap();
        closes!(r.closure);
        assert!((r.q_over_k - 1.0).abs() < 1e-3, "{}", r.q_over_k);
        assert!(r.outlet.t_k > s.t_k);
    }

    #[test]
    fn a_sized_bed_stops_at_its_target() {
        let s = raw();
        let spec = HtsSpec {
            tag: "HTS".into(),
            size: HtsSize::OutletApproach(0.9),
            pellet: Pellet::L,
            p_out_pa: 29.0e5,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        };
        let r = hts(&s, &spec).unwrap();
        closes!(r.closure);
        assert!((r.q_over_k - 0.9).abs() < 1e-4, "{}", r.q_over_k);
    }

    #[test]
    fn ieaghg_outlet_sits_near_nine_tenths_of_equilibrium() {
        let q = ieaghg_hts_q_over_k(KSource::Zhang).unwrap();
        assert!((q - 0.90).abs() < 0.01, "{q}");
    }

    #[test]
    fn lts_sits_at_the_set_approach() {
        let mut s = raw();
        s.t_k = 464.15;
        let spec = LtsSpec {
            tag: "LTS".into(),
            approach_k: LTS_APPROACH_K,
            p_out_pa: 27.0e5,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        };
        let r = lts(&s, &spec).unwrap();
        closes!(r.closure);
        assert!((r.t_eq_k - r.outlet.t_k - LTS_APPROACH_K).abs() < 1e-4);
    }

    #[test]
    fn bad_inputs_are_refused() {
        let mut s = raw();
        s.n_liq_h2o = 1.0;
        let spec = LtsSpec {
            tag: "LTS".into(),
            approach_k: 10.0,
            p_out_pa: 27.0e5,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        };
        assert!(lts(&s, &spec).is_err());
        let mut dry = raw();
        dry.n[CO] = 0.0;
        assert!(lts(&dry, &spec).is_err());
    }
}
