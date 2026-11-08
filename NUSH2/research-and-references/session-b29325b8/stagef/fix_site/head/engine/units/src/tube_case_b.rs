//! Tube count and exit temperature for Case B on the Hoseinzade and Adams tube.
//!
//! Every geometry value here is an assumption for this plant. The tube itself is
//! HA Table 6 (PDF p11): 12 cm bore, 1 cm wall, 14 m, 1.2 cm catalyst, and HA marks
//! the catalyst size as estimated. HA's own shell-side treatment is a cross-flow
//! form scaled for fins, so it is not used. The helium side is the tested JAERI
//! finned annulus, with its fin size and 4.7 mm gap carried across, and the bed side
//! is the JAERI wall coefficient with the JAERI void, emissivity and solid
//! conductivity. Gas leaves at z = L, the INL arrangement, because the Case B heat
//! balance raises steam from the 871 C syngas.
//!
//! The feed is Case 5 NG-10 composition, scaled on tube carbon. Case 6 stream sheets
//! are absent from the local PDF. Helium flow is the Case 6 flow scaled on process
//! heat, so it gives the tube the same helium a whole INL heat train would see.

use crate::case5;
use crate::smr_tube::{self, Correlated, HeatModel, Section, TubeResult, TubeSpec};
use crate::solve::brent;
use crate::stream::{Stream, NSP};
use crate::tube_ht::{BedHt, FinnedTube, Shape};
use crate::xf::Kinetics;
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;
use std::f64::consts::PI;

/// HA Table 6, PDF p11.
pub const HA_TUBE_ID_M: f64 = 0.12;
pub const HA_WALL_M: f64 = 0.01;
pub const HA_LENGTH_M: f64 = 14.0;
pub const HA_DP_M: f64 = 0.012;
pub const HA_TUBES: f64 = 199.0;
/// JAERI A-2-b PDF p102: fin height, pitch and thickness, gap, tube conductivity.
const FIN_H_M: f64 = 0.002;
const FIN_PITCH_M: f64 = 0.003;
const FIN_T_M: f64 = 0.001;
const GAP_M: f64 = 0.0047;
const LAMBDA_WALL: f64 = 21.0 * 1.163;
/// JAERI A-2-b PDF p105, m2 K/W after conversion.
const R_FOUL: f64 = 0.0002 / 1.163;
/// JAERI A-2-b PDF p103.
const VOID: f64 = 0.45;
const EMISSIVITY: f64 = 0.14;
const LAMBDA_S: f64 = 1.05;
/// XF2 Table 2 PDF p4, as in `jaeri`.
const SOLID_DENSITY_KG_M3: f64 = 2355.2;
/// Upper limit of the tube-count search for a target exit temperature.
pub const N_TUBES_SEARCH_MAX: f64 = 5000.0;
/// Case 6 helium supply and flow, TEV-961 PDF p18.
pub const HE_T_IN_K: f64 = 900.0 + 273.15;
const HE_FLOW_CASE6_KG_S: f64 = 78.49;
/// Process heat of Case 6 and Case B, MW.
const Q_CASE6_MW: f64 = 176.8;
const Q_CASE_B_MW: f64 = 120.8;
/// Tube carbon of Case 6 with recycle, kmol/h, from the design calculation
/// the design calculation, scaled here on process heat.
const TUBE_C_CASE6_KMOL_H: f64 = 2303.7;
/// HE-1 pressure, psia, A-55 PDF p79, Pa.
const HE_P_PA: f64 = 1015.0 * 6894.757;

/// One point of the sweep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CasePoint {
    pub n_tubes: f64,
    pub exit_c: f64,
    pub conversion: f64,
    pub he_out_c: f64,
    /// Heat into the process gas across all tubes, MW.
    pub duty_mw: f64,
    /// Bed pressure drop, bar.
    pub dp_bar: f64,
    /// Mean flux on the bare root area, kW/m2.
    pub flux_kw_m2: f64,
}

/// Scale of Case B against Case 6.
pub fn he_total_kg_s() -> f64 {
    HE_FLOW_CASE6_KG_S * Q_CASE_B_MW / Q_CASE6_MW
}

/// Whole-plant reformer feed for Case B.
pub fn feed_total() -> Result<Stream> {
    let mut f = case5::reformer_feed()?;
    let carbon = f.elements()?[0];
    let target = TUBE_C_CASE6_KMOL_H * Q_CASE_B_MW / Q_CASE6_MW / 3.6;
    let k = target / carbon;
    f.n = f.n.map(|v| v * k);
    f.tag = "S04".to_string();
    Ok(f)
}

fn per_tube(total: &Stream, n_tubes: f64) -> Stream {
    let mut s = total.clone();
    let n: [f64; NSP] = total.n.map(|v| v / n_tubes);
    s.n = n;
    s
}

/// The tube at a given count.
pub fn spec(n_tubes: f64, activity: f64) -> Result<TubeSpec> {
    if !(n_tubes >= 1.0 && n_tubes.is_finite()) {
        return Err(EngineError::Range {
            param: "n_tubes".into(),
            msg: format!("{n_tubes} must be at least 1"),
        });
    }
    let dr = HA_TUBE_ID_M + 2.0 * HA_WALL_M;
    let df = dr + 2.0 * FIN_H_M;
    Ok(TubeSpec {
        tag: "SMR-TUBE".into(),
        length_m: HA_LENGTH_M,
        cat_start_m: 0.0,
        bed_area_m2: PI / 4.0 * HA_TUBE_ID_M * HA_TUBE_ID_M,
        void: VOID,
        dp_m: HA_DP_M,
        bulk_density_kg_m3: SOLID_DENSITY_KG_M3 * (1.0 - VOID),
        sections: vec![Section {
            end_m: HA_LENGTH_M,
            up_he_w_m_k: 0.0,
            up_ret_w_m_k: 0.0,
            q_he_w_m: 0.0,
        }],
        heat: HeatModel::Correlated(Correlated {
            tube: FinnedTube {
                di_m: HA_TUBE_ID_M,
                dr_m: dr,
                df_m: df,
                pitch_m: FIN_PITCH_M,
                fin_t_m: FIN_T_M,
                lambda_w: LAMBDA_WALL,
                r_i: R_FOUL,
            },
            guide_id_m: df + 2.0 * GAP_M,
            bed: BedHt {
                void: VOID,
                dp_m: HA_DP_M,
                dt_m: HA_TUBE_ID_M,
                l_m: HA_LENGTH_M,
                emissivity: EMISSIVITY,
                lambda_s: LAMBDA_S,
                shape: Shape::Cylinder,
            },
        }),
        kinetics: Kinetics::with_activity(activity)?,
        k_source: KSource::Zhang,
        bayonet: false,
        ergun: true,
        he_m_kg_s: he_total_kg_s() / n_tubes,
        he_t_in_k: HE_T_IN_K,
        he_p_pa: HE_P_PA,
        he_out_guess_k: None,
        ret_out_guess_k: None,
    })
}

/// Solve one tube at a count, returning the raw tube result too.
pub fn run(n_tubes: f64, activity: f64) -> Result<(CasePoint, TubeResult)> {
    let total = feed_total()?;
    let s = spec(n_tubes, activity)?;
    let t = smr_tube::solve(&s, &per_tube(&total, n_tubes))?;
    let dr = HA_TUBE_ID_M + 2.0 * HA_WALL_M;
    let end = t
        .nodes
        .last()
        .copied()
        .ok_or_else(|| EngineError::NotConverged("empty tube profile".to_string()))?;
    let point = CasePoint {
        n_tubes,
        exit_c: t.outlet.t_k - 273.15,
        conversion: t.conversion,
        he_out_c: t.he_out_k - 273.15,
        duty_mw: t.process_heat_w * n_tubes / 1.0e6,
        dp_bar: (total.p_pa - end.p_pa) / 1.0e5,
        flux_kw_m2: t.process_heat_w / (PI * dr * HA_LENGTH_M) / 1.0e3,
    };
    Ok((point, t))
}

/// Tube count that brings the exit to `t_exit_c`, searched on ln N between the
/// limits, or an error when the target lies outside what the limits reach.
pub fn tubes_for_exit(t_exit_c: f64, lo: f64, hi: f64, activity: f64) -> Result<CasePoint> {
    let n = brent(
        |ln_n| Ok(run(ln_n.exp(), activity)?.0.exit_c - t_exit_c),
        lo.ln(),
        hi.ln(),
        0.003,
        40,
    )?
    .exp();
    Ok(run(n, activity)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_carries_the_scaled_carbon() {
        let f = feed_total().unwrap();
        let c = f.elements().unwrap()[0] * 3.6;
        assert!((c - 2303.7 * 120.8 / 176.8).abs() < 1e-6, "{c}");
    }

    #[test]
    fn helium_flow_scales_on_heat() {
        assert!((he_total_kg_s() - 53.63).abs() < 0.01);
    }
}
