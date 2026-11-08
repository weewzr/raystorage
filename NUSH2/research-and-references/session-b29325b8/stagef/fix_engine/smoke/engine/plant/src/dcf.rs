//! Method 2, discounted cash flow. Year-by-year real cash flows for two plants
//! of equal H2 output on a common horizon: the unabated SMR and NUS-H2.
//!
//! Conventions, all from the build plan:
//! - Year 0 is the end of construction, and flows fall at year ends, so
//!   operation runs from year 1 to the horizon.
//! - The reactor spends uniformly over the construction years. A part year comes
//!   first, as the earliest bucket.
//! - The SMR island enters at year 0 as IEAGHG Total Capital Requirement (TCR),
//!   and Total Plant Cost is left out. TCR already carries the island's own
//!   interest during construction, so `uniform_spend` would count it twice.
//!   Capture, IHX, recycle train, licence and power island also enter at year 0,
//!   as in the ledger.
//! - On any horizon, the items with a 25 y life and the SMR island are replaced
//!   in full every 25 y of operation, so at years 25 and 50 on a 60 y horizon.
//!   The unused life of the last unit bought is credited at the horizon, straight
//!   line, in that year's cash flow. Decommissioning is paid at the horizon with
//!   no sinking fund.
//! - Both plants carry the SMR island unchanged, so reformer differences are
//!   assumed to cancel. Its variable lines are raw water and chemicals with
//!   catalysts from IEAGHG Table 4. Table 4 prices them at 95% availability, so
//!   they scale with operating hours. Its feedstock, fuel and electricity lines
//!   are left out because the ledger's gas, steam and power lines cover them.

use crate::anchored::{Balance, Model};
use crate::economics::{CostItems, CostQuery, Stage};
use nush2_numerics::{EngineError, Result};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Side {
    Smr,
    NusH2,
}

/// One cost line on the timeline, S$M at each year. Credits are negative.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CashLine {
    pub id: &'static str,
    pub side: Side,
    pub flows: Vec<(i32, f64)>,
    pub pv: f64,
}

/// A named contribution to LAC_DCF minus L5, S$/t.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GapTerm {
    pub id: &'static str,
    pub l5_s_per_t: f64,
    pub dcf_s_per_t: f64,
    pub delta: f64,
}

/// Result of one DCF run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DcfResult {
    pub stage: Stage,
    pub life_y: u32,
    pub n: f64,
    pub r_charged: f64,
    pub smr_capital_sm: f64,
    pub smr_fom_sm_y: f64,
    /// Raw water plus chemicals and catalysts of the SMR island, S$M/y.
    pub smr_vom_sm_y: f64,
    pub annual_h2_t: f64,
    pub annual_abatement_t: f64,
    pub npv_cost_smr_sm: f64,
    pub npv_cost_nush2_sm: f64,
    pub npv_h2_t: f64,
    pub npv_abated_t: f64,
    /// S$/kg.
    pub lcoh_smr: f64,
    /// S$/kg.
    pub lcoh_nush2: f64,
    /// S$/t.
    pub lac: f64,
    /// Method 1 at the same query, S$/t.
    pub l5: f64,
    pub gap_abs: f64,
    pub gap_rel: f64,
    pub gap_terms: Vec<GapTerm>,
    #[serde(skip)]
    pub lines: Vec<CashLine>,
}

fn discount(rate: f64, year: i32) -> f64 {
    (1.0 + rate).powi(-year)
}

fn line(rate: f64, id: &'static str, side: Side, flows: Vec<(i32, f64)>) -> CashLine {
    let pv = flows.iter().map(|(y, v)| v * discount(rate, *y)).sum();
    CashLine {
        id,
        side,
        flows,
        pv,
    }
}

fn annual(life: u32, v: f64) -> Vec<(i32, f64)> {
    (1..=life as i32).map(|y| (y, v)).collect()
}

/// Uniform spend of `total` over `years`, ending at year 0. A part year comes first.
///
/// eq: L.34
pub fn uniform_spend(total: f64, years: f64) -> Result<Vec<(i32, f64)>> {
    if years <= 0.0 {
        return Err(EngineError::Range {
            param: "CONSTRUCTION_YEARS".into(),
            msg: format!("{years} must be positive"),
        });
    }
    let whole = years.floor() as i32;
    let part = years - years.floor();
    let mut out = Vec::new();
    if part > 0.0 {
        out.push((-whole, total * part / years));
    }
    for k in 0..whole {
        out.push((k - whole + 1, total / years));
    }
    Ok(out)
}

/// Capital bought at year 0 and again in full every `item_life` years while the
/// horizon runs past that year. A unit bought at the horizon would earn nothing,
/// so none is. The unused life of the last unit is credited at the horizon,
/// straight line, as a negative flow. Lives are whole years, like the flows.
///
/// eq: L.34
fn with_replacement(capital: f64, item_life: u32, horizon: u32) -> Result<Vec<(i32, f64)>> {
    if item_life == 0 {
        return Err(EngineError::Range {
            param: "item_life".into(),
            msg: "an item life must be at least one whole year".into(),
        });
    }
    let mut out = Vec::new();
    let mut bought = 0;
    while bought < horizon {
        out.push((bought as i32, capital));
        bought += item_life;
    }
    // `bought` is now the year the last unit wears out, at or past the horizon.
    let unused = bought - horizon;
    if unused > 0 {
        out.push((
            horizon as i32,
            -capital * f64::from(unused) / f64::from(item_life),
        ));
    }
    Ok(out)
}

impl Model {
    /// SMR island capital and fixed O&M in S$M and S$M/y at scale `s`, re-based
    /// with the EUR2014 to S$2025 factor of the cost inputs.
    pub fn smr_island(&self, s: f64) -> (f64, f64) {
        let size = s / self.s_b;
        (
            self.smr_tcr_eurm * self.f2014 * size,
            self.smr_fom_eur_y / 1e6 * self.f2014 * size,
        )
    }

    /// SMR island raw water plus chemicals and catalysts in S$M/y at scale `s`.
    /// IEAGHG Table 4 states them at its own availability, so they follow the
    /// operating hours of this plant.
    pub fn smr_island_variable(&self, s: f64) -> f64 {
        let hours = self.avail / self.smr_availability;
        (self.smr_water_eur_y + self.smr_chem_eur_y) / 1e6 * self.f2014 * (s / self.s_b) * hours
    }

    /// Run the DCF for one query, compared with L5 at the same query.
    ///
    /// eq: L.28
    /// eq: L.34
    /// eq: L.35
    /// eq: L.36
    /// eq: L.37
    pub fn dcf(&self, b: &Balance, q: &CostQuery) -> Result<DcfResult> {
        let k: CostItems = self.cost_items(b, q)?;
        let life = q.life as u32;
        if life == 0 || f64::from(life) != q.life {
            return Err(EngineError::Range {
                param: "life".into(),
                msg: format!("{} must be a whole number of years", q.life),
            });
        }
        let item_life = self.item_life as u32;
        let r = self.rate;
        let s = b.s;
        let h_s = s * b.c.h_scale;
        let (island, island_fom) = self.smr_island(h_s);
        let island_vom = self.smr_island_variable(h_s);
        let annual_h2_t = self.h2_a_kgh * h_s * self.hours / 1000.0;
        let annual_abatement_t = self.emissions(&b.c, s).a_atm;

        let gas_smr = k.ng_smr * k.gas_s_m_per_mmscfd;
        let gas_plant = k.ng_plant * k.gas_s_m_per_mmscfd;
        let steam_smr = k.steam_smr_mm * k.steam_s_m_per_mm;
        let steam_plant = k.steam_plant_mm * k.steam_s_m_per_mm;

        use Side::{NusH2, Smr};
        let lines = vec![
            line(
                r,
                "SMR_ISLAND_CAP",
                Smr,
                with_replacement(island, item_life, life)?,
            ),
            line(r, "SMR_ISLAND_FOM", Smr, annual(life, island_fom)),
            line(r, "SMR_ISLAND_VOM", Smr, annual(life, island_vom)),
            line(r, "GAS", Smr, annual(life, gas_smr)),
            line(r, "STEAM_CREDIT", Smr, annual(life, -steam_smr)),
            line(r, "PURCHASED_POWER", Smr, annual(life, k.smr_power_y)),
            line(
                r,
                "SMR_ISLAND_CAP",
                NusH2,
                with_replacement(island, item_life, life)?,
            ),
            line(r, "SMR_ISLAND_FOM", NusH2, annual(life, island_fom)),
            line(r, "SMR_ISLAND_VOM", NusH2, annual(life, island_vom)),
            line(r, "GAS", NusH2, annual(life, gas_plant)),
            line(r, "STEAM_CREDIT", NusH2, annual(life, -steam_plant)),
            line(
                r,
                "REACTOR_CAP",
                NusH2,
                uniform_spend(k.reactor_capital, self.idc_years)?,
            ),
            line(
                r,
                "CAPTURE_CAP",
                NusH2,
                with_replacement(k.capture_capital, item_life, life)?,
            ),
            line(r, "CAPTURE_FOM", NusH2, annual(life, k.capture_fom_y)),
            line(r, "CAPTURE_ALLIN", NusH2, annual(life, k.capture_allin_y)),
            line(r, "TS", NusH2, annual(life, k.ts_y)),
            line(r, "REACTOR_OM", NusH2, annual(life, k.reactor_om_y)),
            line(
                r,
                "IHX",
                NusH2,
                with_replacement(k.ihx_capital, k.ihx_life as u32, life)?,
            ),
            line(r, "LICENCE", NusH2, vec![(0, k.licence_capital)]),
            line(r, "DECOM", NusH2, vec![(life as i32, k.decom_total)]),
            line(r, "POWER_ISLAND", NusH2, vec![(0, k.power_island_capital)]),
            line(
                r,
                "RECYCLE_TRAIN",
                NusH2,
                with_replacement(k.recycle_capital, item_life, life)?,
            ),
            line(r, "RECYCLE_FOM", NusH2, annual(life, k.recycle_fom_y)),
            line(
                r,
                "EXPORT_REVENUE",
                NusH2,
                annual(life, -k.export_revenue_y),
            ),
        ];

        let npv =
            |side: Side| -> f64 { lines.iter().filter(|l| l.side == side).map(|l| l.pv).sum() };
        let npv_cost_smr_sm = npv(Smr);
        let npv_cost_nush2_sm = npv(NusH2);
        let annuity: f64 = (1..=life as i32).map(|y| discount(r, y)).sum();
        let npv_h2_t = annual_h2_t * annuity;
        let npv_abated_t = annual_abatement_t * annuity;
        let lac = (npv_cost_nush2_sm - npv_cost_smr_sm) * 1e6 / npv_abated_t;

        let led = self.ledger(b, q)?;
        let l5 = led.l5() * 1e6 / annual_abatement_t;

        // Each line's share of the gap, in S$/t. The ledger keeps IDC as its own
        // line and folds recycle fixed O&M into the train, so those pair up here.
        let pv_of = |ids: &[&str]| -> f64 {
            lines
                .iter()
                .filter(|l| ids.contains(&l.id))
                .map(|l| if l.side == NusH2 { l.pv } else { -l.pv })
                .sum()
        };
        let per_t = 1e6 / annual_abatement_t;
        let dcf_per_t = 1e6 / npv_abated_t;
        let term = |id: &'static str, ledger_s_m_y: f64, ids: &[&str]| -> GapTerm {
            let l5_s_per_t = ledger_s_m_y * per_t;
            let dcf_s_per_t = pv_of(ids) * dcf_per_t;
            GapTerm {
                id,
                l5_s_per_t,
                dcf_s_per_t,
                delta: dcf_s_per_t - l5_s_per_t,
            }
        };
        let gap_terms = vec![
            term("GAS", led.gas, &["GAS"]),
            term(
                "REACTOR_CAP_WITH_IDC",
                led.reactor_capital + led.idc,
                &["REACTOR_CAP"],
            ),
            term("CAPTURE_CAP", led.capture_capital, &["CAPTURE_CAP"]),
            term(
                "CAPTURE_FOM",
                led.capture_fom + led.capture_allin,
                &["CAPTURE_FOM", "CAPTURE_ALLIN"],
            ),
            term("TS", led.ts, &["TS"]),
            term("REACTOR_OM", led.reactor_om, &["REACTOR_OM"]),
            term("IHX", led.ihx, &["IHX"]),
            term("LICENCE", led.licence, &["LICENCE"]),
            term("DECOM", led.decom, &["DECOM"]),
            term("POWER_ISLAND", led.power_island, &["POWER_ISLAND"]),
            term(
                "RECYCLE_TRAIN",
                led.recycle_train,
                &["RECYCLE_TRAIN", "RECYCLE_FOM"],
            ),
            term(
                "STEAM_AND_POWER",
                led.lost_steam + led.baseline_power + led.export_revenue,
                &["STEAM_CREDIT", "PURCHASED_POWER", "EXPORT_REVENUE"],
            ),
        ];

        Ok(DcfResult {
            stage: q.stage,
            life_y: life,
            n: q.n,
            r_charged: k.r_charged,
            smr_capital_sm: island,
            smr_fom_sm_y: island_fom,
            smr_vom_sm_y: island_vom,
            annual_h2_t,
            annual_abatement_t,
            npv_cost_smr_sm,
            npv_cost_nush2_sm,
            npv_h2_t,
            npv_abated_t,
            lcoh_smr: npv_cost_smr_sm * 1e6 / (npv_h2_t * 1000.0),
            lcoh_nush2: npv_cost_nush2_sm * 1e6 / (npv_h2_t * 1000.0),
            lac,
            l5,
            gap_abs: lac - l5,
            gap_rel: (lac - l5) / l5,
            gap_terms,
            lines,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchored::Scenario;
    use crate::economics::Stage;

    fn headline(stage: Stage, life: f64, n: f64) -> DcfResult {
        let m = Model::embedded().unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let q = CostQuery::central(stage, life, n, m.r_rated);
        m.dcf(&b, &q).unwrap()
    }

    #[test]
    fn eq_l_34_uniform_spend_sums_to_total() {
        let f = uniform_spend(110.0, 5.5).unwrap();
        assert_eq!(f.len(), 6);
        assert_eq!(f[0].0, -5);
        assert_eq!(f[5].0, 0);
        let sum: f64 = f.iter().map(|x| x.1).sum();
        assert!((sum - 110.0).abs() < 1e-9);
        assert!((f[0].1 - 10.0).abs() < 1e-9);
    }

    #[test]
    fn eq_l_34_replacement_every_25_y_with_straight_line_credit() {
        // Bought at 0, 25 and 50, with 15 of the last 25 y unused at year 60.
        assert_eq!(
            with_replacement(5.0, 25, 60).unwrap(),
            vec![(0, 5.0), (25, 5.0), (50, 5.0), (60, -3.0)]
        );
        // The first unit wears out exactly at the horizon.
        assert_eq!(with_replacement(5.0, 25, 25).unwrap(), vec![(0, 5.0)]);
        // Nothing is bought at the horizon itself.
        assert_eq!(
            with_replacement(5.0, 25, 50).unwrap(),
            vec![(0, 5.0), (25, 5.0)]
        );
        // Any other horizon follows the same rule, 10 of 25 y unused here.
        assert_eq!(
            with_replacement(5.0, 25, 40).unwrap(),
            vec![(0, 5.0), (25, 5.0), (40, -2.0)]
        );
        // A horizon inside the first life credits the first unit.
        assert_eq!(
            with_replacement(5.0, 25, 20).unwrap(),
            vec![(0, 5.0), (20, -1.0)]
        );
        assert!(with_replacement(5.0, 0, 60).is_err());
    }

    #[test]
    fn dcf_lines_follow_the_replacement_rule_on_both_plants() {
        let r60 = headline(Stage::Boak, 60.0, 1.0);
        let r25 = headline(Stage::Boak, 25.0, 1.0);
        let replaced = [
            (Side::Smr, "SMR_ISLAND_CAP"),
            (Side::NusH2, "SMR_ISLAND_CAP"),
            (Side::NusH2, "CAPTURE_CAP"),
            (Side::NusH2, "IHX"),
            (Side::NusH2, "RECYCLE_TRAIN"),
        ];
        for (side, id) in replaced {
            let find = |r: &DcfResult| -> CashLine {
                r.lines
                    .iter()
                    .find(|l| l.side == side && l.id == id)
                    .unwrap()
                    .clone()
            };
            let long = find(&r60);
            assert_eq!(find(&r25).flows.len(), 1, "{id}");
            let years: Vec<i32> = long.flows.iter().map(|f| f.0).collect();
            assert_eq!(years, vec![0, 25, 50, 60], "{id}");
            let c = long.flows[0].1;
            assert!(c > 0.0, "{id}");
            assert_eq!(long.flows[1].1, c, "{id}");
            assert_eq!(long.flows[2].1, c, "{id}");
            assert!((long.flows[3].1 + 0.6 * c).abs() < 1e-9, "{id}");
            let pv = c * (1.0 + 1.08f64.powi(-25) + 1.08f64.powi(-50) - 0.6 * 1.08f64.powi(-60));
            assert!(
                (long.pv - pv).abs() < 1e-9,
                "{id}: {} against {pv}",
                long.pv
            );
        }
    }

    #[test]
    fn smr_variable_lines_scale_from_95_to_85_percent_availability() {
        let m = Model::embedded().unwrap();
        let want = (99_365.0 + 420_000.0) / 1e6 * 2.322 * 0.85 / 0.95;
        assert!((m.smr_island_variable(m.s_b) - want).abs() < 1e-9);
        let r = headline(Stage::Boak, 25.0, 1.0);
        let pv = |side: Side| {
            r.lines
                .iter()
                .find(|l| l.side == side && l.id == "SMR_ISLAND_VOM")
                .unwrap()
                .pv
        };
        assert_eq!(pv(Side::Smr), pv(Side::NusH2));
        assert!(pv(Side::Smr) > 0.0);
    }

    #[test]
    fn annual_lines_match_the_ledger_exactly() {
        let r = headline(Stage::Boak, 25.0, 1.0);
        for t in &r.gap_terms {
            if matches!(
                t.id,
                "GAS" | "TS" | "REACTOR_OM" | "CAPTURE_FOM" | "STEAM_AND_POWER"
            ) {
                assert!(t.delta.abs() < 1e-9, "{} moved by {}", t.id, t.delta);
            }
        }
    }

    #[test]
    fn eq_l_03_decommissioning_at_end_of_life_equals_the_sinking_fund() {
        // PV at the horizon spread over the annuity is the sinking fund factor.
        let r = headline(Stage::Boak, 60.0, 1.0);
        let d = r.gap_terms.iter().find(|t| t.id == "DECOM").unwrap();
        assert!(d.delta.abs() < 1e-9, "moved by {}", d.delta);
    }

    #[test]
    fn gap_terms_sum_to_the_gap() {
        for life in [25.0, 60.0] {
            let r = headline(Stage::Foak, life, 0.6);
            let sum: f64 = r.gap_terms.iter().map(|t| t.delta).sum();
            assert!(
                (sum - r.gap_abs).abs() < 1e-9,
                "{sum} against {}",
                r.gap_abs
            );
        }
    }

    #[test]
    fn smr_island_is_common_to_both_plants() {
        let m = Model::embedded().unwrap();
        let r = headline(Stage::Boak, 25.0, 1.0);
        let (cap, fom) = m.smr_island(m.s_b);
        assert!((cap - 222.89 * 2.322).abs() < 1e-9);
        assert!((fom - 7.545514 * 2.322).abs() < 1e-9);
        let pv = |side: Side, id: &str| {
            r.lines
                .iter()
                .find(|l| l.side == side && l.id == id)
                .unwrap()
                .pv
        };
        assert_eq!(
            pv(Side::Smr, "SMR_ISLAND_CAP"),
            pv(Side::NusH2, "SMR_ISLAND_CAP")
        );
        assert_eq!(
            pv(Side::Smr, "SMR_ISLAND_FOM"),
            pv(Side::NusH2, "SMR_ISLAND_FOM")
        );
    }

    #[test]
    fn whole_years_only() {
        let m = Model::embedded().unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let q = CostQuery::central(Stage::Boak, 25.5, 1.0, m.r_rated);
        assert!(m.dcf(&b, &q).is_err());
    }
}
