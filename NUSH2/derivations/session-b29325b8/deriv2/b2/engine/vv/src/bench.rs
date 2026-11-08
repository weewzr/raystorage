//! Reproduction checks of the screening script. Targets live in
//! `data/screening_checks.csv`. One check of the parent script tested the
//! withdrawn deck ledger, so it has no row here.

use nush2_numerics::{EngineError, Result};
use nush2_plant::anchored::{crf, BalanceOpts, Model, Scenario};
use nush2_props::units::{kmol_h_to_t_h, GJ_PER_MWH, MMBTU_GJ, ST};

const CHECKS: &str = include_str!("../data/screening_checks.csv");

// Reader-note fixtures used by two checks. They restate earlier reader sums,
// so they stay beside the checks and are never model inputs.
const CP_HE_CHECK_KJ: f64 = 5.193;
const PRIMARY_FLOW_READER: f64 = 92.2;
const VESSEL_READER_MW: f64 = 2.96;
const DUCT_READER_MW: f64 = 0.45;
const CO2_READER_T_H: f64 = 70.4;
const SELEXOL_READER_MW: f64 = 19.05;
const EXPORT_READER_MW: f64 = 24.53;
const OPEX_CHECK_USD: f64 = 10.0;
const N_CHECK: f64 = 0.6;
const SIZE_CHECK_MWTH: f64 = 210.0;
const FOAK_HIGH_CHECK: f64 = 2.1;
const FOAK_LOW_CHECK: f64 = 1.3;
const SAVING_CHECK_MMSCFD: f64 = 18.5;
const ENERGY_CHECK_MMSCFD: f64 = 9.0;
const PRICE_CHECK: f64 = 15.0;

/// One check after it ran.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub id: String,
    pub target: f64,
    pub tolerance: f64,
    pub unit: String,
    pub source: String,
    pub locator: String,
    pub got: f64,
}

impl CheckResult {
    pub fn passed(&self) -> bool {
        (self.got - self.target).abs() <= self.tolerance
    }
}

fn got(m: &Model, id: &str) -> Result<f64> {
    let inl = m.carbon(Scenario::InlSel, m.phi_rec, m.eta_mea);
    let e_all = m.e_rows.sum();
    let e_no_purge = e_all - m.e_rows.purge;
    Ok(match id {
        "H2_A_T_Y" => m.h2_a_kgh * m.hours / 1000.0,
        "H2_B_T_Y" => m.h2_b_kgh * m.hours / 1000.0,
        "SCALE_S" => m.s_b,
        "HE_DUTY_MW" => m.he_flow * CP_HE_CHECK_KJ * (m.t_he_s - m.t_he_r) / 1000.0,
        "E_ROWS_SUM_MWE" => e_all,
        "SLIDE6_REACTOR_MWTH" => m.q_proc + e_no_purge / m.eta_pwr,
        "COMPLETE_REACTOR_MWTH" => m.q_proc + e_all / m.eta_pwr,
        "FEED_CARBON_ST_D" => kmol_h_to_t_h(m.f_c6) * 24.0 / ST,
        "C_PER_MOL_NG" => m.c_per_mol,
        "SELEXOL_INLET_ST_D" => kmol_h_to_t_h(inl.co2_abs) * 24.0 / ST,
        "CARBON_CLOSURE_PCT" => 100.0 * (inl.cap + inl.burn + inl.p) / inl.f,
        "NINETY_PCT_SYNGAS_ST_D" => m.eta_mea * kmol_h_to_t_h(inl.co2_abs) * 24.0 / ST,
        "GAS_SAVING_18P5_SM_Y" => {
            SAVING_CHECK_MMSCFD * m.ng_hhv * MMBTU_GJ * m.days * PRICE_CHECK / 1e6
        }
        "GJ_PER_D_9_MMSCFD" => ENERGY_CHECK_MMSCFD * m.ng_hhv * MMBTU_GJ,
        "CRF_25" => crf(m.rate, m.lives[0])?,
        "REACTOR_ANNUAL_N0P6_SM_Y" => {
            let cap = m.occ_kwt.central * m.ref_mwt * 1e3 / 1e6 * m.f2019 * crf(m.rate, 25.0)?;
            let om = OPEX_CHECK_USD * m.ref_mwt * m.hours * m.f2019 / 1e6;
            (cap + om) * (SIZE_CHECK_MWTH / m.ref_mwt).powf(N_CHECK)
        }
        "OAK10_INL_HIGH_USD_KWT" => m.occ_kwt.high * FOAK_HIGH_CHECK * m.lf10,
        "OAK10_INL_LOW_USD_KWT" => m.occ_kwt.low * FOAK_LOW_CHECK * m.lf10,
        "HTGR_IHX_CENTRAL_MWTH" => {
            let (dp_s, dp_p) = m.circ_dp_central();
            let wt = m.circ_shaft(m.he_flow, m.t_circ_s, 7.0, dp_s, 0.75)
                + m.circ_shaft(PRIMARY_FLOW_READER, m.t_circ_p, 7.0, dp_p, 0.75);
            m.q_proc
                + (e_no_purge + wt / m.eta_motor) / m.eta_pwr
                + VESSEL_READER_MW
                + DUCT_READER_MW
                - wt
        }
        "MEA_REACTOR_MWTH" => {
            let q = CO2_READER_T_H * m.srd.central / GJ_PER_MWH;
            let need = q - SELEXOL_READER_MW - EXPORT_READER_MW;
            let w = m.r_work_central * need;
            m.q_proc + need + w + (e_no_purge - w) / m.eta_pwr
        }
        "EX4_ABOVE_130_MW" => m.ex4_heat_above(m.ex4.t_out),
        other => return Err(EngineError::Parse(format!("no formula for check {other}"))),
    })
}

/// Run every check in the fixture.
pub fn run_checks(m: &Model) -> Result<Vec<CheckResult>> {
    let mut out = Vec::new();
    for line in CHECKS.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 6 {
            return Err(EngineError::Parse(format!("bad check row {line}")));
        }
        let parse = |s: &str| -> Result<f64> {
            s.parse()
                .map_err(|_| EngineError::Parse(format!("bad number {s}")))
        };
        out.push(CheckResult {
            id: f[0].to_string(),
            target: parse(f[1])?,
            tolerance: parse(f[2])?,
            unit: f[3].to_string(),
            source: f[4].to_string(),
            locator: f[5].to_string(),
            got: got(m, f[0])?,
        });
    }
    Ok(out)
}

/// Export steam spent at a demand of Selexol steam plus 5 MW, in MW. The approved
/// order spends export steam second, so the answer is 5 MW.
pub fn reboiler_order_probe(m: &Model) -> Result<f64> {
    let base: BalanceOpts = m.opts(1.0);
    let c = m.carbon(Scenario::RecF, base.phi, base.eta_c);
    let selexol = m.q_selexol_c5 * m.co2_cap_c6 / m.case5_cap;
    let o = BalanceOpts {
        srd: (selexol + 5.0) * GJ_PER_MWH / kmol_h_to_t_h(c.cap),
        ..base
    };
    let b = m.balance(Scenario::RecF, &o)?;
    Ok((m.export_c6 - b.export_kept) * MMBTU_GJ / GJ_PER_MWH)
}
