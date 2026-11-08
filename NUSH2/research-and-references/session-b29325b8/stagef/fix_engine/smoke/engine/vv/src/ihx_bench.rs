//! IHX benchmarks, on the rating model
//! of `nush2_units::ihx`.
//!
//! INL-11109 Table 9 prints the conductance with the inlets, flows and outlets of
//! four configurations, so the effectiveness-NTU rating at the printed UA must
//! return INL's outlets within 2 K. The HTTR set is the measured IHX state before
//! the loss of off-site power tests (TECDOC-1382 Table 3-23); its conductance is
//! taken from the same measurement (secondary duty over the log mean difference),
//! so those rows test the rating and the heat balance of one data set and say so.

use crate::fixture::Fixture;
use crate::report::Row;
use crate::rowkit::{Meta, SAME};
use nush2_numerics::Result;
use nush2_props::helium::CP_J_KG_K;
use nush2_units::ihx;

/// Limits: INL outlets 2 K, HTTR outlets 5 K.
pub const INL_TOL_K: f64 = 2.0;
pub const HTTR_TOL_K: f64 = 5.0;

const K: f64 = 273.15;

/// INL-11109 Table 9 rows.
pub fn inl_11109(rows: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("inl_11109.csv")?;
    for series in [
        "CFG1_900C_2MPA",
        "CFG1_900C_7MPA",
        "CFG1_950C_2MPA",
        "CFG1_950C_7MPA",
    ] {
        let fix = f.get(series, "ua")?;
        let ua = fix.value * 1.0e6;
        let th = f.v(series, "hot_in")?;
        let tc = f.v(series, "cold_in")?;
        let ch = f.v(series, "hot_flow")? * CP_J_KG_K;
        let cc = f.v(series, "cold_flow")? * CP_J_KG_K;
        let r = ihx::rate(th + K, tc + K, ch, cc, ua)?;
        let cond = format!(
            "Configuration 1, {series}: inlets {th:.0} and {tc:.0} C, printed UA {:.2} MW/C, \
             c_p {CP_J_KG_K:.0}",
            fix.value
        );
        let meta = |id: &str, subject: &str, unit: &str| {
            Meta::at(fix, format!("IHX-INL-{id}-{series}"), 2, subject, unit)
                .cond(cond.clone())
                .use_(SAME)
        };
        rows.push(meta("TH", "IHX hot side outlet", "C").judged(
            f.v(series, "hot_out")?,
            r.t_hot_out_k - K,
            INL_TOL_K,
            "Effectiveness-NTU counterflow (card C.11) at the printed UA",
        ));
        rows.push(meta("TC", "IHX cold side outlet", "C").judged(
            f.v(series, "cold_out")?,
            r.t_cold_out_k - K,
            INL_TOL_K,
            "Effectiveness-NTU counterflow (card C.11) at the printed UA",
        ));
        rows.push(meta("Q", "IHX duty", "MW").reported(
            Some(f.v(series, "duty")?),
            r.q_w / 1.0e6,
            "Printed to three figures; no limit set in the plan",
        ));
    }
    Ok(())
}

/// TECDOC-1382 Table 3-23 rows: the HTTR IHX before the LP tests.
pub fn tecdoc_1382(rows: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("iaea_tecdoc_1382.csv")?;
    for (series, tag) in [("HTTR_LP_15MW", "15"), ("HTTR_LP_30MW", "30")] {
        let fix = f.get(series, "prim_in")?;
        let tpi = fix.value;
        let tpo = f.v(series, "prim_out")?;
        let mp = f.v(series, "prim_flow")?;
        let tsi = f.v(series, "sec_in")?;
        let tso = f.v(series, "sec_out")?;
        let ms = f.v(series, "sec_flow")?;
        let qp = ihx::hot_side(mp, CP_J_KG_K, tpi, tpo);
        let qs = ihx::cold_side(ms, CP_J_KG_K, tsi, tso);
        let lm = ihx::lmtd(tpi - tso, tpo - tsi)?;
        let ua = qs / lm;
        let r = ihx::rate(tpi + K, tsi + K, mp * CP_J_KG_K, ms * CP_J_KG_K, ua)?;
        let cond = format!(
            "HTTR LP test from {tag} MW: primary {tpi:.0} to {tpo:.0} C at {mp} kg/s, \
             secondary {tsi:.0} to {tso:.0} C at {ms} kg/s; UA {:.1} kW/K from the secondary \
             duty over the LMTD",
            ua / 1.0e3
        );
        let meta = |id: &str, subject: &str, unit: &str| {
            Meta::at(fix, format!("IHX-HTTR-{tag}-{id}"), 2, subject, unit)
                .cond(cond.clone())
                .use_(SAME)
        };
        let note = "UA comes from the same measured set, so this tests the rating against the \
                    measured heat balance";
        rows.push(meta("TP", "IHX primary outlet", "C").judged(
            tpo,
            r.t_hot_out_k - K,
            HTTR_TOL_K,
            note,
        ));
        rows.push(meta("TS", "IHX secondary outlet", "C").judged(
            tso,
            r.t_cold_out_k - K,
            HTTR_TOL_K,
            note,
        ));
        rows.push(
            meta(
                "CLOSURE",
                "Primary less secondary duty over primary duty",
                "-",
            )
            .reported(
                Some(0.0),
                ihx::closure(qp, qs, 0.0) / qp,
                "Measured imbalance, the basis of the IHX loss fraction of card B.15",
            ),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Status;

    #[test]
    fn eq_c_11_inl_and_httr_rows_run() {
        let mut rows = Vec::new();
        inl_11109(&mut rows).unwrap();
        tecdoc_1382(&mut rows).unwrap();
        assert_eq!(rows.len(), 18);
        for r in &rows {
            assert!(r.simulated.is_finite(), "{}", r.test_id);
            assert_ne!(r.status, Status::NotRun, "{}", r.test_id);
        }
    }
}
