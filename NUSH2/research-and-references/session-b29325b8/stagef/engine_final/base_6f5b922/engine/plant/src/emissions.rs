//! Emissions ledger. Direct emissions to atmosphere on the INL baseline, with
//! Cho and IEAGHG bounds. System and lifecycle views stay sensitivities.

use crate::anchored::{Balance, Carbon, Model};
use nush2_props::units::{MMBTU_GJ, ST};
use serde::Serialize;

/// Annual emissions, t/y. Site is what the boundary sees, atmosphere is what
/// reaches air.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Emissions {
    pub baseline: f64,
    pub site: f64,
    pub atm: f64,
    pub captured: f64,
    pub a_site: f64,
    pub a_atm: f64,
}

/// Lifecycle adjustment terms, t/y.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Lifecycle {
    pub up_credit: f64,
    pub nuclear: f64,
    pub ts_burden: f64,
}

/// Backup firing over the outage hours.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Backup {
    pub hours: f64,
    pub h2_extra_t_y: f64,
    pub emit_cc_t_y: f64,
    pub emit_nocc_t_y: f64,
    pub a_extra_cc_vs_new_smr: f64,
    pub gas_mmscfd_cc: f64,
    pub gas_mmscfd_nocc: f64,
}

impl Model {
    pub fn emissions(&self, c: &Carbon, s: f64) -> Emissions {
        let baseline = self.co2_conv * ST * self.days * s * c.h_scale;
        let site = self.tpy(c.burn, s);
        let atm = self.tpy(c.burn + c.exported, s);
        let captured = self.tpy(c.cap, s);
        Emissions {
            baseline,
            site,
            atm,
            captured,
            a_site: baseline - site,
            a_atm: baseline - atm,
        }
    }

    /// Change to abatement if lost steam and avoided baseline grid power count, t/y.
    pub fn system_adjust(&self, b: &Balance, s: f64) -> f64 {
        let h_s = s * b.c.h_scale;
        let lost_mm = self.export_conv * h_s
            - b.export_kept
            - nush2_props::units::mw_to_mmbtu_h(b.tg_surplus);
        let lost =
            lost_mm * MMBTU_GJ * self.hours * self.ef_ng_lhv / self.system_boiler_eff / 1000.0;
        let power = self.e_conv * h_s * self.hours * self.ef_grid;
        -lost + power
    }

    /// Lifecycle terms, t/y, charged at reactor rating `r`.
    pub fn lifecycle_terms(&self, b: &Balance, s: f64, r: f64) -> Lifecycle {
        let c = b.c;
        let h_s = s * c.h_scale;
        let gj = self.ng_hhv * MMBTU_GJ * self.days;
        Lifecycle {
            up_credit: (self.ng_conv * h_s - c.ng * s) * gj * self.ef_up / 1000.0,
            nuclear: r * self.eta_pwr * self.hours * self.ef_nuc / 1000.0,
            ts_burden: self.tpy(c.cap, s) * self.ts_burden,
        }
    }

    /// Outage hours run on a gas-fired conventional train, H2 sold as extra.
    /// The two rates are TEV-953 Table 2: the unabated SMR and the conventional
    /// plant with capture. Card K.20 gives the third route, `backup_heater`.
    pub fn backup(&self, s: f64) -> Backup {
        let out_h = nush2_props::units::HOURS_PER_YEAR - self.hours;
        let h2_kgh = self.h2_a_kgh * s;
        let emit_cc = self.co2_emit_convcc * ST / 24.0 * s;
        let emit_nocc = self.co2_conv * ST / 24.0 * s;
        Backup {
            hours: out_h,
            h2_extra_t_y: h2_kgh * out_h / 1000.0,
            emit_cc_t_y: emit_cc * out_h,
            emit_nocc_t_y: emit_nocc * out_h,
            a_extra_cc_vs_new_smr: (emit_nocc - emit_cc) * out_h,
            gas_mmscfd_cc: self.ng_cc_total * s,
            gas_mmscfd_nocc: self.ng_conv * s,
        }
    }

    /// Outage hours run on this plant with its process heat from a gas-fired
    /// heater: site intensity plus heater CO2 per kg H2, card K.20. The
    /// heater efficiency is the card's assumed value. Reboiler and power
    /// supply in those hours are outside the card, so the row says so.
    ///
    /// eq: K.20
    pub fn backup_heater(&self, b: &Balance, s: f64) -> BackupHeater {
        let out_h = nush2_props::units::HOURS_PER_YEAR - self.hours;
        let h2_kgh = self.h2_a_kgh * s * b.c.h_scale;
        let h2_t_y = h2_kgh * self.hours / 1000.0;
        let e_site = self.emissions(&b.c, s).site / h2_t_y;
        // MW x 3.6 GJ/MWh x kg/GJ = kg/h of CO2 from the heater.
        let heater_kg_h = b.q_proc * 3.6 * self.ef_ng_lhv / self.backup_heater_eff;
        let e_bk = e_site + heater_kg_h / h2_kgh;
        let h2_extra = h2_kgh * out_h / 1000.0;
        BackupHeater {
            hours: out_h,
            h2_extra_t_y: h2_extra,
            e_site_kg_kg: e_site,
            e_bk_kg_kg: e_bk,
            emit_t_y: e_bk * h2_extra,
            a_extra_vs_new_smr: (self.co2_conv * ST / 24.0 * s - e_bk * h2_kgh / 1000.0) * out_h,
        }
    }

    /// CO2 dissolved in the process condensate, vented instead of held in the
    /// carbon split (decision S14), as kmol C/h at Case A scale. It moves from
    /// the captured stream to the atmosphere so carbon still closes.
    ///
    /// eq: K.8
    pub fn vent_condensate(&self, c: &Carbon, s: f64, co2_kmol_h_at_s: f64) -> Carbon {
        let d = co2_kmol_h_at_s / s;
        Carbon {
            cap: c.cap - d,
            burn: c.burn + d,
            ..*c
        }
    }
}

/// Backup firing on this plant's own heater over the outage hours, card K.20.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BackupHeater {
    pub hours: f64,
    pub h2_extra_t_y: f64,
    pub e_site_kg_kg: f64,
    pub e_bk_kg_kg: f64,
    pub emit_t_y: f64,
    /// Against an unabated SMR making the same H2 in the same hours.
    pub a_extra_vs_new_smr: f64,
}

#[cfg(test)]
mod tests {
    use crate::anchored::{Model, Scenario};

    #[test]
    fn eq_k_20_heater_adds_heat_to_the_site_intensity() {
        let m = Model::embedded().unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let k = m.backup_heater(&b, m.s_b);
        let h2_kgh = m.h2_a_kgh * m.s_b * b.c.h_scale;
        let heater = b.q_proc * 3.6 * m.ef_ng_lhv / m.backup_heater_eff / h2_kgh;
        assert!((k.e_bk_kg_kg - k.e_site_kg_kg - heater).abs() < 1e-12);
        assert!(k.e_bk_kg_kg > k.e_site_kg_kg);
        // Same outage hours and extra H2 as the conventional-train routes.
        let conv = m.backup(m.s_b);
        assert_eq!(k.hours, conv.hours);
        assert!((k.h2_extra_t_y - conv.h2_extra_t_y).abs() < 1e-6 * conv.h2_extra_t_y);
        assert!((k.emit_t_y - k.e_bk_kg_kg * k.h2_extra_t_y).abs() < 1e-9 * k.emit_t_y);
    }

    #[test]
    fn eq_k_08_vented_condensate_keeps_carbon() {
        let m = Model::embedded().unwrap();
        let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
        let v = m.vent_condensate(&b.c, m.s_b, 5.0);
        assert!(((v.cap + v.burn) - (b.c.cap + b.c.burn)).abs() < 1e-12);
        let e0 = m.emissions(&b.c, m.s_b);
        let e1 = m.emissions(&v, m.s_b);
        let moved = m.tpy(5.0 / m.s_b, m.s_b);
        assert!((e1.atm - e0.atm - moved).abs() < 1e-6);
        assert!((e0.a_atm - e1.a_atm - moved).abs() < 1e-6);
    }
}
