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
}
