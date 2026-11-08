//! TEV-961 Case 5 (ROT 875 C, He supply 850 C): the closest full flowsheet to the
//! project basis. Stream sheets are at PDF p74 to p80, read from page images. Case
//! 6 sheets are absent from the local PDF, so Case 5 stands in, as plan section 4
//! records. Every number below is a printed Case 5 value unless a comment says it
//! is calculated.
//!
//! The chain here is the gas side of plan section 4: reformer outlet, HPS-GEN1,
//! HTS, WGS-EX1, WGS-EX2, LTS, WGS-EX3, WGS-EX4, KO-DRM-1.

use crate::cooler::{self, CoolerResult, CoolerSpec};
use crate::flash::{self, FlashResult};
use crate::smr_eq::{self, SmrEqResult, SmrEqSpec};
use crate::stream::{Sp, Stream, NSP};
use crate::wgs::{self, HtsSize, HtsSpec, LtsSpec, Pellet, ShiftResult};
use nush2_numerics::Result;
use nush2_props::equilibrium::KSource;
use nush2_props::mixture;
use nush2_props::units::LB;

/// Pa per psi.
const PSI_PA: f64 = 6894.757;

fn f_to_k(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8 + 273.15
}

fn lb_h_to_kg_s(lb_h: f64) -> f64 {
    lb_h * LB / 3600.0
}

/// MMBtu/h to W, by the conversion in `nush2_props::units`.
pub fn mmbtu_h_to_w(v: f64) -> f64 {
    nush2_props::units::mmbtu_h_to_mw(v) * 1.0e6
}

fn flows(pairs: &[(Sp, f64)]) -> [f64; NSP] {
    let mut n = [0.0; NSP];
    for (sp, v) in pairs {
        n[*sp as usize] += *v;
    }
    n
}

fn scaled_to_mass(mut n: [f64; NSP], kg_s: f64) -> Result<[f64; NSP]> {
    let m = mixture::mass_kg_s(&n)?;
    for v in n.iter_mut() {
        *v *= kg_s / m;
    }
    Ok(n)
}

/// Raw syngas mol%, A-50 PDF p74. Printed to 0.1 mol%, so it is a rounded state.
pub const RAW_MOL_PCT: [(Sp, f64); 6] = [
    (Sp::H2, 48.8),
    (Sp::Co, 8.7),
    (Sp::Co2, 4.8),
    (Sp::N2, 0.7),
    (Sp::H2o, 33.4),
    (Sp::Ch4, 3.6),
];
/// SG-2 mass, lb/h, A-56 PDF p80.
pub const SYNGAS_LB_H: f64 = 349_724.0;
/// Reformer outlet, A-55 PDF p79.
pub const REFORMER_OUT_T_F: f64 = 1600.0;
pub const REFORMER_OUT_P_PSI: f64 = 439.0;
/// NG-10, reformer inlet, A-55 PDF p79.
pub const REFORMER_IN_T_F: f64 = 1000.0;
pub const REFORMER_IN_P_PSI: f64 = 445.0;

/// Duties, MMBtu/h, A-55 PDF p79 and A-56 PDF p80.
pub const DUTY_REFORM_1: f64 = 404.2;
pub const DUTY_REFORM_2: f64 = 74.0;
pub const DUTY_HPS_GEN1: f64 = 236.3;
pub const DUTY_WGS_EX1: f64 = 66.7;
pub const DUTY_WGS_EX2: f64 = 24.2;
pub const DUTY_WGS_EX3: f64 = 65.7;
pub const DUTY_WGS_EX4: f64 = 117.1;
pub const DUTY_KO_DRM: f64 = 19.7;

/// HTS, A-53 PDF p77 and A-56 PDF p80.
pub const HTS_OUT_T_F: f64 = 764.0;
pub const HTS_OUT_CO_DRY_PCT: f64 = 3.01;
/// LTS, A-53 PDF p77 and A-56 PDF p80.
pub const LTS_OUT_T_F: f64 = 415.0;
pub const LTS_OUT_CO_DRY_PCT: f64 = 0.33;
/// Selexol inlet CO2 on the dry basis, A-53 PDF p77.
pub const SELEXOL_IN_CO2_PCT: f64 = 17.7;
/// KO-LIQ-1, lb/h, A-56 PDF p80.
pub const KO_LIQ_LB_H: f64 = 126_553.0;
/// SG-8, lb/h, A-56 PDF p80.
pub const KO_GAS_LB_H: f64 = 223_172.0;

/// Natural gas mol%, A-50 PDF p74, with the lumping of plan section 3: C6 enters
/// as 1.2 mol C5 per mol, and O2 joins N2.
const NG_MOL_PCT: [(Sp, f64); 7] = [
    (Sp::Ch4, 93.571),
    (Sp::C2h6, 3.749),
    (Sp::C3h8, 0.920),
    (Sp::Nc4h10, 0.260),
    (Sp::Nc5h12, 0.040 + 1.2 * 0.010),
    (Sp::N2, 1.190 + 0.010),
    (Sp::Co2, 0.250),
];
/// NAT-GAS mass, lb/h, A-55 PDF p79.
const NG_LB_H: f64 = 65_046.0;
/// RECY-2 mass, lb/h, A-55 PDF p79.
const RECYCLE_LB_H: f64 = 22_790.0;
/// PSA reject mol%, calculated because the sheets print none: `inl.md` s3.3
/// rebuilds it from the raw syngas, a shift to 0.33% dry CO, 99% CO2 removal and
/// 88% H2 recovery. The rebuilt reject mass is within 2.4% of the printed FUEL-2.
const RECYCLE_MOL_PCT: [(Sp, f64); 5] = [
    (Sp::H2, 59.9),
    (Sp::Ch4, 30.7),
    (Sp::N2, 6.1),
    (Sp::Co, 2.2),
    (Sp::Co2, 1.2),
];
/// WATER-3 plus STEAM-2, lb/h, A-55 PDF p79.
const WATER_LB_H: f64 = 13_513.0 + 248_376.0;

/// Printed raw syngas at the reformer outlet.
pub fn raw_syngas() -> Result<Stream> {
    let n = scaled_to_mass(flows(&RAW_MOL_PCT), lb_h_to_kg_s(SYNGAS_LB_H))?;
    Ok(Stream::gas(
        "SG-2",
        f_to_k(REFORMER_OUT_T_F),
        REFORMER_OUT_P_PSI * PSI_PA,
        n,
    ))
}

/// Reformer feed NG-10 at 1000 F: natural gas, recycle and water, each on its
/// printed mass. The recycle composition is calculated, see `RECYCLE_MOL_PCT`.
pub fn reformer_feed() -> Result<Stream> {
    let ng = scaled_to_mass(flows(&NG_MOL_PCT), lb_h_to_kg_s(NG_LB_H))?;
    let rec = scaled_to_mass(flows(&RECYCLE_MOL_PCT), lb_h_to_kg_s(RECYCLE_LB_H))?;
    let mut n = [0.0; NSP];
    for i in 0..NSP {
        n[i] = ng[i] + rec[i];
    }
    n[Sp::H2o as usize] = lb_h_to_kg_s(WATER_LB_H) / nush2_props::species::Sp::H2o.mw_kg_mol();
    Ok(Stream::gas(
        "NG-10",
        f_to_k(REFORMER_IN_T_F),
        REFORMER_IN_P_PSI * PSI_PA,
        n,
    ))
}

/// SMR-EQ on NG-10 at the printed outlet state.
pub fn reformer(source: KSource) -> Result<SmrEqResult> {
    smr_eq::solve(
        &reformer_feed()?,
        &SmrEqSpec {
            tag: "SMR-EQ".into(),
            t_out_k: f_to_k(REFORMER_OUT_T_F),
            p_out_pa: REFORMER_OUT_P_PSI * PSI_PA,
            k_source: source,
            outlet_tag: "S05".into(),
        },
    )
}

/// Options of the shift and cooling chain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrainOpts {
    pub k_source: KSource,
    pub pellet: Pellet,
    pub hts_size: HtsSize,
    pub lts_approach_k: f64,
}

impl TrainOpts {
    /// The HTS bed stops at the Q/K of the printed IEAGHG outlet on `k_source`.
    pub fn sized_on_ieaghg(k_source: KSource) -> Result<Self> {
        Ok(TrainOpts {
            k_source,
            pellet: Pellet::L,
            hts_size: HtsSize::OutletApproach(wgs::ieaghg_hts_q_over_k(k_source)?),
            lts_approach_k: wgs::LTS_APPROACH_K,
        })
    }

    /// The HTS bed runs at the space velocity of the Chinchen tests.
    pub fn chinchen_space_velocity(k_source: KSource) -> Self {
        TrainOpts {
            k_source,
            pellet: Pellet::L,
            hts_size: HtsSize::Ghsv(wgs::CHINCHEN_GHSV_DRY_1_H),
            lts_approach_k: wgs::LTS_APPROACH_K,
        }
    }
}

/// Every unit of the chain, in flow order.
#[derive(Debug, Clone, PartialEq)]
pub struct Train {
    pub hps_gen1: CoolerResult,
    pub hts: ShiftResult,
    pub ex1: CoolerResult,
    pub ex2: CoolerResult,
    pub lts: ShiftResult,
    pub ex3: CoolerResult,
    pub ex4: CoolerResult,
    pub ko: CoolerResult,
    pub drum: FlashResult,
}

fn cooler_spec(tag: &str, t_f: f64, p_psi: f64, out: &str) -> CoolerSpec {
    CoolerSpec {
        tag: tag.to_string(),
        t_out_k: f_to_k(t_f),
        p_out_pa: p_psi * PSI_PA,
        outlet_tag: out.to_string(),
    }
}

/// Run the chain from the reformer outlet stream `start`. Set temperatures and
/// pressures are the printed A-56 values.
pub fn train(start: &Stream, opts: &TrainOpts) -> Result<Train> {
    let hps_gen1 = cooler::cool(start, &cooler_spec("HPS-GEN1", 635.0, 436.0, "S06"))?;
    let hts = wgs::hts(
        &hps_gen1.outlet,
        &HtsSpec {
            tag: "WGS-HTS".into(),
            size: opts.hts_size,
            pellet: opts.pellet,
            p_out_pa: 419.0 * PSI_PA,
            k_source: opts.k_source,
            outlet_tag: "S07".into(),
        },
    )?;
    let ex1 = cooler::cool(&hts.outlet, &cooler_spec("WGS-EX1", 480.0, 416.0, "SG-3"))?;
    let ex2 = cooler::cool(&ex1.outlet, &cooler_spec("WGS-EX2", 375.0, 413.0, "S08"))?;
    let lts = wgs::lts(
        &ex2.outlet,
        &LtsSpec {
            tag: "WGS-LTS".into(),
            approach_k: opts.lts_approach_k,
            p_out_pa: 396.0 * PSI_PA,
            k_source: opts.k_source,
            outlet_tag: "S09".into(),
        },
    )?;
    let ex3 = cooler::cool(&lts.outlet, &cooler_spec("WGS-EX3", 300.0, 393.0, "SG-6"))?;
    let ex4 = cooler::cool(&ex3.outlet, &cooler_spec("WGS-EX4", 158.0, 390.0, "SG-7"))?;
    let ko = cooler::cool(&ex4.outlet, &cooler_spec("KO-DRM-1", 104.0, 389.0, "SG-7B"))?;
    let drum = flash::flash(
        &ko.outlet,
        ko.outlet.t_k,
        ko.outlet.p_pa,
        "FLASH",
        "SG-8",
        "S10",
    )?;
    Ok(Train {
        hps_gen1,
        hts,
        ex1,
        ex2,
        lts,
        ex3,
        ex4,
        ko,
        drum,
    })
}

/// Term T01: CO2 / (CO2 + CH4 + CO) after the shift, the carbon split the
/// screening model calls phi.
pub fn phi_after_shift(s: &Stream) -> f64 {
    let (co2, ch4, co) = (s.flow(Sp::Co2), s.flow(Sp::Ch4), s.flow(Sp::Co));
    co2 / (co2 + ch4 + co)
}

/// Lower limit of the heat the screening model counts in term T05, K.
pub const T05_LIMIT_K: f64 = 130.0 + 273.15;

/// Term T05 on Case 5: heat released cooling EX3 outlet gas from 149 C to 130 C,
/// W. The plant scales it by captured CO2 as the screening model does.
pub fn syngas_above_130c_w(t: &Train) -> Result<f64> {
    cooler::heat_above(&t.ex3.outlet, T05_LIMIT_K)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_masses_add_up_to_the_feed() {
        let f = reformer_feed().unwrap();
        let m = f.mass_kg_s().unwrap();
        assert!((m / lb_h_to_kg_s(SYNGAS_LB_H) - 1.0).abs() < 1e-5, "{m}");
    }

    #[test]
    fn temperature_conversion() {
        assert!((f_to_k(32.0) - 273.15).abs() < 1e-12);
        assert!((f_to_k(1600.0) - 1144.261).abs() < 1e-2);
    }
}
