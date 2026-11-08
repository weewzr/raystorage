//! JAERI mock-up benchmarks for SMR-TUBE.
//!
//! Source: JAERI-Tech 2005-031, a scanned report, read from page images. Sheets
//! A-2-a and A-2-b (PDF p99 to p109) carry the 2002-11-28 run with every section
//! value, A-3-a (PDF p113) and A-4-a (PDF p114) carry the two later runs, and A-5
//! (PDF p115) carries one composition. Yin et al. 2007 Table 2 (PDF p5) reports the
//! inlet set of that A-5 point.
//!
//! The test section is stations 1 to 5: section lengths 1.87, 1.5, 1.5 and 1.5 m
//! from the A-2-b sheet (PDF p100), the same 6.37 m as the Table 3.1 tube, with no
//! catalyst above station 1 (PDF p108). So the sheet fixes the station elevations.
//! z = 0 is station 1 and z = 6.37 m is station 5 at the tip.
//!
//! Boundary data for every run are measured: helium at station 5, feed at station
//! 1, flows, and heater power. No heat transfer value is fitted:
//!
//! - U is JAERI's own calculated value per section (A-2-b PDF p105) scaled by the
//!   ratio of each run's printed whole-section U calc. The measured U sat 6 to 12%
//!   higher, and the model does not use it.
//! - The return tube uses JAERI's calculated K12 per section (PDF p107). JAERI's own
//!   sheet shows this theory falling well short of the measured return duty, and
//!   the benchmark reports that gap.
//! - Heat loss per section comes from the A-2-b balance and stands for the rig in
//!   every run, so the held-back runs take nothing from their own heat balance.
//!   Heater power is measured per run and split by the square of the printed
//!   section currents.
//!
//! The single fitted number is the activity, fit on A-2 only. It lumps catalyst
//! state, effectiveness factor and bed density, because the catalyst density is not
//! printed. Bulk density is set from the XF2 Table 2 solid density and the JAERI
//! packing ratio only to give the activity a defined meaning.

use crate::smr_tube::{self, HeatModel, Section, TubeResult, TubeSpec};
use crate::stream::{Sp, Stream, NSP};
use crate::xf::Kinetics;
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;
use std::f64::consts::PI;

/// Heated tube length, m. A-2-b PDF p100, Table 3.1 PDF p26.
pub const LENGTH_M: f64 = 6.37;
/// Section lengths, m. A-2-b PDF p100.
pub const SECTION_M: [f64; 4] = [1.87, 1.5, 1.5, 1.5];
/// Tube inner diameter, m. A-2-b PDF p102.
const DI_M: f64 = 0.1278;
/// Return tube outer and inner diameters, m. A-2-b PDF p102.
const DITO_M: f64 = 0.0605;
const DITI_M: f64 = 0.0527;
/// Catalyst bed length, m. Marked tentative on A-2-b PDF p102.
const BED_M: f64 = 6.0;
/// Particle diameter, m. A-2-b PDF p103.
const DP_M: f64 = 0.005;
/// Void fraction, 1 - 0.55. A-2-b PDF p103, an input to JAERI's sheet.
const VOID: f64 = 0.45;
/// Solid density, kg/m3. XF2 Table 2 PDF p4, for the ring catalyst of that paper.
const SOLID_DENSITY_KG_M3: f64 = 2355.2;
/// Finned outer area per metre, m2/m. A-2-b PDF p104.
const A_OUTER_M2_M: f64 = 1.0960;
/// W/m2/K per kcal/m2/h/K, as JAERI converts.
const KCAL_W: f64 = 1.163;
/// Section U calc, W/m2/K, 2002-11-28. A-2-b PDF p105.
const U_CALC_A2: [f64; 4] = [128.47, 138.29, 147.29, 155.54];
/// Whole-section U calc, W/m2/K, 2002-11-28. A-2-b PDF p105.
const U_CALC_A2_SEC14: f64 = 142.02;
/// Section K12 on the return tube inner area, kcal/m2/h/K. A-2-b PDF p107.
const K12_KCAL: [f64; 4] = [155.64, 160.76, 165.09, 168.80];
/// Section heat loss, W. A-2-b PDF p107.
const LOSS_A2_W: [f64; 4] = [5.8e3, 8.2e3, 7.3e3, 3.3e3];
/// Pressures of the A-2 run, gauge MPa at the feed and product points, PDF p100.
const P_IN_A2_PA: f64 = 4.16e6 + 101_325.0;
const P_OUT_A2_PA: f64 = 4.04e6 + 101_325.0;
const P_HE_A2_PA: f64 = 3.98e6 + 101_325.0;
/// Process pressure drop of the A-2 run from the feed and product gauges PT19G and
/// PT2C, Pa, PDF p100. The sheet computes the same 120 kPa as its internal loss.
pub const A2_DP_GAUGES_PA: f64 = P_IN_A2_PA - P_OUT_A2_PA;
/// The differential gauge DT20G between feed and product on the same sheet, Pa.
pub const A2_DP_DIFFERENTIAL_PA: f64 = 0.17e6;
/// Molar masses JAERI's own sheet does not need here.
const MW_CH4_G: f64 = 16.0425;
const MW_H2O_G: f64 = 18.0153;
/// Lower limit of the activity search.
const FIT_LOW: f64 = 1.0e-3;
/// Upper limit of the activity search: the XF1 fresh-catalyst factor, PDF p7, which
/// means no ageing and no diffusion limit at the set bulk density.
pub const FIT_HIGH: f64 = 2.246;
/// Result of `fit_activity_on_a2`, tuned on A-2. The objective keeps falling all the
/// way to the upper limit, so the fit stops there: 92.1 at 0.3, 87.8 at 1.0 and 86.4
/// at 2.246. Above about 0.3 the tube runs at its equilibrium limit, so A-2 cannot
/// fix the activity from below that limit, and this value says only that the
/// kinetics are not what limits the run.
pub const A2_FIT_ACTIVITY: f64 = FIT_HIGH;
/// Scales of the fit objective, K and conversion.
const FIT_T_SCALE_K: f64 = 5.0;
const FIT_X_SCALE: f64 = 0.01;
const GOLDEN: f64 = 0.618_033_988_749_895;

/// Catalyst pellet of the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pellet {
    Cylinder,
    Sphere,
}

/// Printed data of one run. Temperatures are deg C at stations 1 to 5.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub id: &'static str,
    pub source: &'static str,
    pub pellet: Pellet,
    /// True when part of the inlet set is assumed, as for A-5.
    pub inlet_assumed: bool,
    pub he_g_s: f64,
    pub ch4_g_s: f64,
    pub steam_g_s: f64,
    /// N2 in the feed gas, percent.
    pub feed_n2_pct: f64,
    pub p_in_pa: f64,
    pub he_c: [f64; 5],
    pub process_c: [f64; 5],
    pub return_c: [f64; 5],
    /// Dry product H2, N2, CO, CO2, CH4, percent.
    pub dry_pct: [f64; 5],
    /// Sec 1 to 4 duties, kW: helium, return tube, process sensible, reaction.
    pub q_he_kw: f64,
    pub q_return_kw: f64,
    pub q_sensible_kw: f64,
    pub q_reaction_kw: f64,
    pub heater_kw: f64,
    /// Section currents X1R2 to X1R5, A.
    pub heater_a: [f64; 4],
    /// Printed whole-section U calc, W/m2/K. A-5 prints none.
    pub u_calc: f64,
    pub u_meas: f64,
}

/// A-2-a PDF p99, A-2-b PDF p100 to p107.
pub const A2: Run = Run {
    id: "A-2",
    source: "JAERI-Tech 2005-031 A-2 PDF p99 to p107",
    pellet: Pellet::Cylinder,
    inlet_assumed: false,
    he_g_s: 91.0,
    ch4_g_s: 12.08,
    steam_g_s: 48.0,
    feed_n2_pct: 0.6,
    p_in_pa: P_IN_A2_PA,
    he_c: [615.0, 662.0, 738.0, 817.0, 857.0],
    process_c: [478.0, 591.0, 656.0, 728.0, 755.0],
    return_c: [606.0, 700.0, 743.0, 760.0, 756.0],
    dry_pct: [66.6, 0.58, 6.74, 11.48, 14.6],
    q_he_kw: 114.4,
    q_return_kw: 25.9,
    q_sensible_kw: 45.7,
    q_reaction_kw: 75.1,
    heater_kw: 5.1,
    heater_a: [7.4, 3.0, 5.6, 0.0],
    u_calc: 142.0,
    u_meas: 150.9,
};

/// A-3-a PDF p113. Pressures are not printed, so A-2 stands in.
pub const A3: Run = Run {
    id: "A-3",
    source: "JAERI-Tech 2005-031 A-3 PDF p113",
    pellet: Pellet::Cylinder,
    inlet_assumed: false,
    he_g_s: 91.3,
    ch4_g_s: 11.91,
    steam_g_s: 45.9,
    feed_n2_pct: 0.0,
    p_in_pa: P_IN_A2_PA,
    he_c: [625.0, 676.0, 745.0, 817.0, 860.0],
    process_c: [492.0, 598.0, 669.0, 737.0, 760.0],
    return_c: [621.0, 709.0, 745.0, 765.0, 761.0],
    dry_pct: [66.6, 0.51, 7.31, 11.34, 14.24],
    q_he_kw: 111.4,
    q_return_kw: 23.4,
    q_sensible_kw: 42.9,
    q_reaction_kw: 76.8,
    heater_kw: 6.9,
    heater_a: [8.0, 5.8, 5.5, 0.0],
    u_calc: 140.3,
    u_meas: 156.9,
};

/// A-4-a PDF p114, spheres. Pressures are not printed, so A-2 stands in.
pub const A4: Run = Run {
    id: "A-4",
    source: "JAERI-Tech 2005-031 A-4 PDF p114",
    pellet: Pellet::Sphere,
    inlet_assumed: false,
    he_g_s: 91.1,
    ch4_g_s: 11.86,
    steam_g_s: 46.9,
    feed_n2_pct: 0.0,
    p_in_pa: P_IN_A2_PA,
    he_c: [639.0, 676.0, 722.0, 812.0, 858.0],
    process_c: [504.0, 613.0, 636.0, 715.0, 740.0],
    return_c: [633.0, 688.0, 716.0, 749.0, 741.0],
    dry_pct: [65.8, 0.3, 6.1, 11.7, 16.1],
    q_he_kw: 103.6,
    q_return_kw: 18.2,
    q_sensible_kw: 38.2,
    q_reaction_kw: 69.0,
    heater_kw: 8.1,
    heater_a: [8.2, 5.9, 6.4, 0.0],
    u_calc: 129.9,
    u_meas: 138.7,
};

/// A-5 PDF p115 with the inlet set of Yin 2007 Table 2, PDF p5: 2,700 mol/h of CH4,
/// H2O/CH4 3.45, 520 C feed, 851.2 C helium and 4.26 MPa feed pressure. The helium
/// flow, station profiles, duties and heater power are printed nowhere, so the
/// A-2 values stand in and `inlet_assumed` marks it. The A-5 sheet gives 11.96 g/s
/// of CH4.
pub const A5: Run = Run {
    id: "A-5",
    source: "JAERI-Tech 2005-031 A-5 PDF p115, Yin 2007 Table 2 PDF p5",
    pellet: Pellet::Cylinder,
    inlet_assumed: true,
    he_g_s: 91.0,
    ch4_g_s: 11.96,
    steam_g_s: 3.45 * 11.96 / MW_CH4_G * MW_H2O_G,
    feed_n2_pct: 0.1,
    p_in_pa: 4.26e6,
    he_c: [f64::NAN, f64::NAN, f64::NAN, f64::NAN, 851.2],
    process_c: [520.0, f64::NAN, f64::NAN, f64::NAN, f64::NAN],
    return_c: [f64::NAN; 5],
    dry_pct: [65.5, 0.99, 6.89, 11.46, 15.16],
    q_he_kw: f64::NAN,
    q_return_kw: f64::NAN,
    q_sensible_kw: f64::NAN,
    q_reaction_kw: f64::NAN,
    heater_kw: 5.1,
    heater_a: [7.4, 3.0, 5.6, 0.0],
    u_calc: U_CALC_A2_SEC14,
    u_meas: f64::NAN,
};

impl Run {
    /// Conversion from the dry analysis, as JAERI defines it.
    pub fn conversion(&self) -> f64 {
        let [_, _, co, co2, ch4] = self.dry_pct;
        (co + co2) / (co + co2 + ch4)
    }

    /// Feed at station 1.
    pub fn feed(&self) -> Stream {
        let mut n = [0.0; NSP];
        let ch4 = self.ch4_g_s / MW_CH4_G;
        n[Sp::Ch4 as usize] = ch4;
        n[Sp::N2 as usize] = ch4 * self.feed_n2_pct / (100.0 - self.feed_n2_pct);
        n[Sp::H2o as usize] = self.steam_g_s / MW_H2O_G;
        Stream::gas("JAERI_FEED", self.process_c[0] + 273.15, self.p_in_pa, n)
    }
}

/// Elevation of station `i`, 0 to 4, m.
pub fn station_z_m(i: usize) -> f64 {
    SECTION_M.iter().take(i).sum()
}

/// Which printed U sets the level of the section values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UBasis {
    /// JAERI's calculated U, the benchmark basis.
    Calculated,
    /// JAERI's measured U, a sensitivity only.
    Measured,
}

/// Build the tube for one run at one activity.
pub fn spec(r: &Run, activity: f64, basis: UBasis) -> Result<TubeSpec> {
    let level = match basis {
        UBasis::Calculated => r.u_calc,
        UBasis::Measured => {
            if r.u_meas.is_nan() {
                return Err(EngineError::Range {
                    param: "u_meas".into(),
                    msg: format!("{} prints no measured U", r.id),
                });
            }
            r.u_meas
        }
    };
    let u_scale = level / U_CALC_A2_SEC14;
    let i2: f64 = r.heater_a.iter().map(|a| a * a).sum();
    let mut sections = Vec::with_capacity(4);
    let mut end = 0.0;
    for i in 0..4 {
        end += SECTION_M[i];
        let heater_w = r.heater_kw * 1.0e3 * r.heater_a[i] * r.heater_a[i] / i2;
        sections.push(Section {
            end_m: end,
            up_he_w_m_k: U_CALC_A2[i] * u_scale * A_OUTER_M2_M,
            up_ret_w_m_k: K12_KCAL[i] * KCAL_W * PI * DITI_M,
            q_he_w_m: (LOSS_A2_W[i] - heater_w) / SECTION_M[i],
        });
    }
    if let Some(last) = sections.last_mut() {
        last.end_m = LENGTH_M;
    }
    Ok(TubeSpec {
        tag: format!("JAERI {}", r.id),
        length_m: LENGTH_M,
        cat_start_m: LENGTH_M - BED_M,
        bed_area_m2: PI / 4.0 * (DI_M * DI_M - DITO_M * DITO_M),
        void: VOID,
        dp_m: DP_M,
        bulk_density_kg_m3: SOLID_DENSITY_KG_M3 * (1.0 - VOID),
        sections,
        heat: HeatModel::Sections,
        kinetics: Kinetics::with_activity(activity)?,
        k_source: KSource::Zhang,
        bayonet: true,
        ergun: true,
        he_m_kg_s: r.he_g_s * 1.0e-3,
        he_t_in_k: r.he_c[4] + 273.15,
        he_p_pa: P_HE_A2_PA,
        he_out_guess_k: Some(r.process_c[0] + 273.15 + 130.0),
        ret_out_guess_k: Some(r.process_c[0] + 273.15 + 120.0),
    })
}

/// Solve one run.
pub fn run(r: &Run, activity: f64) -> Result<TubeResult> {
    smr_tube::solve(&spec(r, activity, UBasis::Calculated)?, &r.feed())
}

/// Fit objective on one solved run: process temperature at stations 2 to 5 and
/// conversion, each over a stated scale. The exit composition alone barely tests
/// the kinetics, so the axial profile carries most of the weight.
pub fn objective(r: &Run, t: &TubeResult) -> f64 {
    let mut j = 0.0;
    for i in 1..5 {
        let got = t.at(station_z_m(i)).t_k - 273.15;
        let d = (got - r.process_c[i]) / FIT_T_SCALE_K;
        j += d * d;
    }
    let d = (t.conversion - r.conversion()) / FIT_X_SCALE;
    j + d * d
}

/// Golden-section search on ln activity over A-2 alone, ending with a check of the
/// upper limit, where the objective keeps falling.
pub fn fit_activity_on_a2(tol_ln: f64) -> Result<f64> {
    let f = |ln_a: f64| -> Result<f64> {
        let t = run(&A2, ln_a.exp())?;
        Ok(objective(&A2, &t))
    };
    let (ln_top, mut lo, mut hi) = (FIT_HIGH.ln(), FIT_LOW.ln(), FIT_HIGH.ln());
    let mut x1 = hi - GOLDEN * (hi - lo);
    let mut x2 = lo + GOLDEN * (hi - lo);
    let mut f1 = f(x1)?;
    let mut f2 = f(x2)?;
    for _ in 0..60 {
        if hi - lo < tol_ln {
            let mid = 0.5 * (lo + hi);
            return Ok(if f(ln_top)? <= f(mid)? {
                FIT_HIGH
            } else {
                mid.exp()
            });
        }
        if f1 < f2 {
            hi = x2;
            x2 = x1;
            f2 = f1;
            x1 = hi - GOLDEN * (hi - lo);
            f1 = f(x1)?;
        } else {
            lo = x1;
            x1 = x2;
            f1 = f2;
            x2 = lo + GOLDEN * (hi - lo);
            f2 = f(x2)?;
        }
    }
    Err(EngineError::NotConverged("activity fit".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printed_conversions_follow_from_the_dry_analysis() {
        assert!((A2.conversion() - 0.555).abs() < 5e-4);
        assert!((A3.conversion() - 0.567).abs() < 5e-4);
        assert!((A4.conversion() - 0.525).abs() < 5e-4);
        assert!((A5.conversion() - 0.548).abs() < 5e-4);
    }

    #[test]
    fn stations_span_the_heated_length() {
        assert_eq!(station_z_m(0), 0.0);
        assert!((station_z_m(4) - LENGTH_M).abs() < 1e-12);
        assert!((PI * DI_M * 1.0 / 0.4015 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn the_a2_area_and_u_give_the_printed_ua() {
        // 142.02 W/m2/K on 6.98 m2.
        let a = A_OUTER_M2_M * LENGTH_M;
        assert!((a - 6.98).abs() < 0.01, "{a}");
    }
}
