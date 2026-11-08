//! Method 1, annual incremental ledger. Lines L0 to L5 are annual differences
//! against the same-size unabated SMR. Every call states its size exponent.

use crate::anchored::{crf, sff, Balance, CostLevel, Model, Scenario};
use nush2_numerics::Result;
use nush2_props::units::{mw_to_mmbtu_h, MMBTU_GJ};
use serde::Serialize;

/// Maturity stage of the reactor cost. Both learning routes are published.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Stage {
    Foak,
    Boak,
    /// INL PDF p46: learning from BOAK.
    Oak10FromBoak,
    /// INL PDF p47: learning from FOAK.
    Oak10FromFoak,
}

impl Stage {
    pub const ALL: [Stage; 4] = [
        Stage::Foak,
        Stage::Boak,
        Stage::Oak10FromBoak,
        Stage::Oak10FromFoak,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Stage::Foak => "FOAK",
            Stage::Boak => "BOAK",
            Stage::Oak10FromBoak => "10OAK_P46",
            Stage::Oak10FromFoak => "10OAK_P47",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    BottomUp,
    AllIn,
}

/// One ledger evaluation. `CostQuery::central` fills the central set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostQuery {
    pub level: CostLevel,
    pub stage: Stage,
    pub life: f64,
    pub n: f64,
    pub capture_mode: CaptureMode,
    pub ts: Option<f64>,
    pub gas: Option<f64>,
    pub r: Option<f64>,
    pub export_rev: f64,
    pub ng_override: Option<f64>,
    pub capt_k: Option<f64>,
    pub fom: Option<f64>,
    pub rx_mult: f64,
}

impl CostQuery {
    pub fn central(stage: Stage, life: f64, n: f64, r: f64) -> Self {
        Self {
            level: CostLevel::Central,
            stage,
            life,
            n,
            capture_mode: CaptureMode::BottomUp,
            ts: None,
            gas: None,
            r: Some(r),
            export_rev: 0.0,
            ng_override: None,
            capt_k: None,
            fom: None,
            rx_mult: 1.0,
        }
    }
}

/// Capital and annual quantities before annualising, S$M and S$M/y. Both cost
/// methods start from this one set.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CostItems {
    pub r_charged: f64,
    pub gas_price: f64,
    /// MMSCFD of the unabated SMR at the same H2.
    pub ng_smr: f64,
    /// MMSCFD of the plant.
    pub ng_plant: f64,
    /// S$M/y per MMSCFD at the stated price.
    pub gas_s_m_per_mmscfd: f64,
    pub reactor_capital: f64,
    pub capture_capital: f64,
    pub capture_fom_y: f64,
    pub capture_allin_y: f64,
    pub ts_y: f64,
    pub reactor_om_y: f64,
    pub ihx_capital: f64,
    pub ihx_life: f64,
    pub licence_capital: f64,
    pub decom_total: f64,
    pub decom_rate: f64,
    pub power_island_capital: f64,
    pub recycle_capital: f64,
    pub recycle_fom_y: f64,
    /// Export steam of the unabated SMR and of the plant, MMBtu/h.
    pub steam_smr_mm: f64,
    pub steam_plant_mm: f64,
    /// S$M/y per MMBtu/h at the stated gas price.
    pub steam_s_m_per_mm: f64,
    /// Purchased power of the unabated SMR, S$M/y.
    pub smr_power_y: f64,
    pub export_revenue_y: f64,
    pub annual_captured_t: f64,
}

/// Annual ledger lines, S$M/y.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Ledger {
    pub gas: f64,
    pub reactor_capital: f64,
    pub capture_capital: f64,
    pub capture_fom: f64,
    pub capture_allin: f64,
    pub ts: f64,
    pub reactor_om: f64,
    pub ihx: f64,
    pub licence: f64,
    pub decom: f64,
    pub idc: f64,
    pub power_island: f64,
    pub recycle_train: f64,
    pub lost_steam: f64,
    pub baseline_power: f64,
    pub export_revenue: f64,
}

/// Rungs of the ladder.
pub const LEVELS: [&str; 6] = ["L0", "L1", "L2", "L3", "L4", "L5"];

impl Ledger {
    /// Lines in ledger order with their freeze ids and rung index.
    pub fn lines(&self) -> [(&'static str, usize, f64); 16] {
        [
            ("GAS", 0, self.gas),
            ("REACTOR_CAP", 0, self.reactor_capital),
            ("CAPTURE_CAP", 0, self.capture_capital),
            ("CAPTURE_FOM", 0, self.capture_fom),
            ("CAPTURE_ALLIN", 0, self.capture_allin),
            ("TS", 0, self.ts),
            ("REACTOR_OM", 1, self.reactor_om),
            ("IHX", 2, self.ihx),
            ("LICENCE", 3, self.licence),
            ("DECOM", 3, self.decom),
            ("IDC", 4, self.idc),
            ("POWER_ISLAND", 4, self.power_island),
            ("RECYCLE_TRAIN", 4, self.recycle_train),
            ("LOST_STEAM", 5, self.lost_steam),
            ("BASE_POWER", 5, self.baseline_power),
            ("EXPORT_REVENUE", 5, self.export_revenue),
        ]
    }

    /// Cumulative totals at L0 to L5, S$M/y.
    pub fn ladder(&self) -> [f64; 6] {
        let mut out = [0.0; 6];
        for (_, rung, v) in self.lines() {
            out[rung] += v;
        }
        for i in 1..6 {
            out[i] += out[i - 1];
        }
        out
    }

    pub fn l3(&self) -> f64 {
        self.ladder()[3]
    }

    pub fn l5(&self) -> f64 {
        self.ladder()[5]
    }
}

impl Model {
    /// Stage multiplier on the reactor cost.
    pub fn stage_mult(&self, stage: Stage, level: CostLevel) -> f64 {
        let foak = self.foak.at(level);
        match stage {
            Stage::Foak => foak,
            Stage::Oak10FromFoak => foak * self.lf10,
            Stage::Boak => 1.0,
            Stage::Oak10FromBoak => self.lf10,
        }
    }

    /// Reactor capital, S$M.
    pub fn reactor_capex(&self, r: f64, level: CostLevel, stage: Stage, n: f64) -> f64 {
        let base = self.occ_kwt.at(level) * self.ref_mwt * 1e3 / 1e6
            * self.f2019
            * self.stage_mult(stage, level);
        base * (r / self.ref_mwt).powf(n)
    }

    /// Reactor O&M and fuel, S$M/y. Fuel scales linearly and staff with `n`.
    pub fn reactor_om(&self, r: f64, level: CostLevel, n: f64) -> f64 {
        let rate = self.opex.at(level) * self.f2019 * self.hours / 1e6;
        rate * (self.fuel_share * r
            + (1.0 - self.fuel_share) * self.ref_mwt * (r / self.ref_mwt).powf(n))
    }

    /// Interest during construction factor, uniform spend.
    pub fn idc_factor(&self) -> f64 {
        ((1.0 + self.rate).powf(self.idc_years) - 1.0) / (self.rate * self.idc_years)
    }

    /// Capital and annual quantities for one query.
    pub fn cost_items(&self, b: &Balance, q: &CostQuery) -> Result<CostItems> {
        let s = b.s;
        let c = b.c;
        let lv = q.level;
        let r_charged = q.r.unwrap_or(b.r);
        let gp = q.gas.unwrap_or(self.gas.central);
        let ts = q.ts.unwrap_or(self.ts.at(lv));
        let h_s = s * c.h_scale;
        let annual_cap = self.tpy(c.cap, s);
        let gj_per_mmscf = self.ng_hhv * MMBTU_GJ;
        let ng_smr = self.ng_conv * h_s;
        let ng_plant = q.ng_override.unwrap_or(c.ng) * s;

        let cap_rx = self.reactor_capex(r_charged, lv, q.stage, q.n) * q.rx_mult;
        let (capture_capital, capture_fom_y, capture_allin_y) = match q.capture_mode {
            CaptureMode::BottomUp => {
                let capx = q.capt_k.unwrap_or(self.capt_k.at(lv)) * b.cap_t;
                (capx, capx * q.fom.unwrap_or(self.fom.at(lv)), 0.0)
            }
            CaptureMode::AllIn => (0.0, 0.0, annual_cap * self.allin / 1e6),
        };

        let ihx_duty = (b.q_proc + b.q_recyc) / self.ihx_ref_mw;
        let (ihx_capital, ihx_life) = match lv {
            CostLevel::Low => (self.ihx_low_s * (b.q_proc / self.q_proc), self.item_life),
            CostLevel::Central => (self.ihx_s * ihx_duty, self.item_life),
            CostLevel::High => (
                self.ihx_s * self.ihx_high_mult * ihx_duty,
                self.ihx_life_high,
            ),
        };

        let stage_power = match lv {
            CostLevel::Central => self.stage_mult(q.stage, CostLevel::Central),
            _ => 1.0,
        };
        let recycle_capital = match b.scn {
            Scenario::RecF | Scenario::RecFix | Scenario::InlSel => self.recycle_capex_a.at(lv) * s,
            _ => 0.0,
        };

        Ok(CostItems {
            r_charged,
            gas_price: gp,
            ng_smr,
            ng_plant,
            gas_s_m_per_mmscfd: gj_per_mmscf * self.days * gp / 1e6,
            reactor_capital: cap_rx,
            capture_capital,
            capture_fom_y,
            capture_allin_y,
            ts_y: annual_cap * ts / 1e6,
            reactor_om_y: self.reactor_om(r_charged, lv, q.n),
            ihx_capital,
            ihx_life,
            licence_capital: self.licence.at(lv) * self.fx,
            decom_total: self.decom.at(lv) * (r_charged * self.decom_kwe_per_kwt) * 1e3 / 1e6
                * self.fx,
            decom_rate: self.sff_rate.at(lv),
            power_island_capital: self.ecs_kwe * b.e_total * 1e3 / 1e6 * self.f2019 * stage_power,
            recycle_capital,
            recycle_fom_y: recycle_capital * self.fom.central,
            steam_smr_mm: self.export_conv * h_s,
            steam_plant_mm: b.export_kept + mw_to_mmbtu_h(b.tg_surplus),
            steam_s_m_per_mm: MMBTU_GJ * self.hours * gp / 1e6,
            smr_power_y: self.e_conv * h_s * self.hours * self.price_e / 1e3,
            export_revenue_y: q.export_rev,
            annual_captured_t: annual_cap,
        })
    }

    /// Annual cost differences against the unabated SMR at the same H2, S$M/y.
    pub fn ledger(&self, b: &Balance, q: &CostQuery) -> Result<Ledger> {
        let k = self.cost_items(b, q)?;
        let cr25 = crf(self.rate, self.item_life)?;
        let crl = crf(self.rate, q.life)?;
        Ok(Ledger {
            gas: -(k.ng_smr - k.ng_plant) * k.gas_s_m_per_mmscfd,
            reactor_capital: k.reactor_capital * crl,
            capture_capital: k.capture_capital * cr25,
            capture_fom: k.capture_fom_y,
            capture_allin: k.capture_allin_y,
            ts: k.ts_y,
            reactor_om: k.reactor_om_y,
            ihx: k.ihx_capital * crf(self.rate, k.ihx_life)?,
            licence: k.licence_capital * crl,
            decom: k.decom_total * sff(k.decom_rate, q.life)?,
            idc: k.reactor_capital * (self.idc_factor() - 1.0) * crl,
            power_island: k.power_island_capital * crl,
            recycle_train: k.recycle_capital * cr25 + k.recycle_fom_y,
            lost_steam: (k.steam_smr_mm - k.steam_plant_mm) * k.steam_s_m_per_mm,
            baseline_power: -k.smr_power_y,
            export_revenue: -k.export_revenue_y,
        })
    }
}
