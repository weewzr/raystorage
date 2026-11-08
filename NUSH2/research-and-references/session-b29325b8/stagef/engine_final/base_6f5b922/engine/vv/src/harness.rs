//! Runs every check and benchmark and returns the rows of
//! `results/Validation_Results.csv`. Nothing is written unless the comparison ran
//! in this call, and a benchmark that cannot run is written as `not_run`.
//!
//! Code tests and literature benchmarks are kept apart. `cargo test` checks the
//! code and must be green; a literature benchmark that misses its frozen limit is
//! a row with status `fail`, and the report states it.

use crate::bench::{reboiler_order_probe, run_checks};
use crate::golden::{availability, balance_all_cases, ledger_grid, GoldStats, GOLD_TOL};
use crate::report::{Row, Status};
use nush2_numerics::Result;
use nush2_plant::anchored::{Model, Scenario};
use nush2_plant::dcf::DcfResult;
use nush2_plant::economics::{CostQuery, Stage};
use nush2_plant::export::{dcf_grid, n_tag, tables, RUNS};
use nush2_plant::flowsheet::{self, Check};
use nush2_plant::gates::{GateState, Rows, GATES};
use nush2_plant::params::ParamSet;
use nush2_units::stream::Evidence;

const SAME: &str = "same_dataset_disclosed";
const PORT_NOTE: &str =
    "Verification of the port against the screening script; no physics is validated by this row";

/// Relative gap above which `LAC_DCF` against L5 needs a named term in the docs.
pub const LAC_GAP_LIMIT: f64 = 0.10;

fn stat_row(id: &str, level: u8, subject: &str, st: &GoldStats) -> Row {
    Row {
        test_id: id.to_string(),
        level,
        subject: subject.to_string(),
        source: "screening.py via design calc goldens".to_string(),
        locator: "engine_goldens.json, freeze_values.json".to_string(),
        conditions: format!("{} values compared", st.compared),
        literature: None,
        simulated: st.max_err,
        unit: "relative error".to_string(),
        abs_error: None,
        rel_error: Some(st.max_err),
        tolerance: Some(GOLD_TOL),
        status: Status::of(st.passed()),
        data_use: SAME,
        explanation: if st.passed() {
            format!("{PORT_NOTE}. Largest error shown in simulated")
        } else {
            format!("{} values failed. Worst: {}", st.failed, st.worst)
        },
    }
}

/// Identity of a single-value comparison.
struct Pt<'a> {
    id: &'a str,
    level: u8,
    subject: &'a str,
    unit: &'a str,
    conditions: &'a str,
}

fn pt<'a>(id: &'a str, level: u8, subject: &'a str, unit: &'a str, conditions: &'a str) -> Pt<'a> {
    Pt {
        id,
        level,
        subject,
        unit,
        conditions,
    }
}

fn point(p: Pt<'_>, target: f64, got: f64, tol: f64) -> Row {
    let abs = (got - target).abs();
    Row {
        test_id: p.id.to_string(),
        level: p.level,
        subject: p.subject.to_string(),
        source: "screening.py, printed in the freeze".to_string(),
        locator: "freeze sections 1, 5, 6".to_string(),
        conditions: p.conditions.to_string(),
        literature: Some(target),
        simulated: got,
        unit: p.unit.to_string(),
        abs_error: Some(abs),
        rel_error: (target != 0.0).then(|| abs / target.abs()),
        tolerance: Some(tol),
        status: Status::of(abs <= tol),
        data_use: SAME,
        explanation: PORT_NOTE.to_string(),
    }
}

/// Names of the two largest terms behind a gap.
fn gap_text(d: &DcfResult) -> String {
    let mut terms = d.gap_terms.clone();
    terms.sort_by(|a, b| b.delta.abs().total_cmp(&a.delta.abs()));
    let named: Vec<String> = terms
        .iter()
        .take(2)
        .map(|t| format!("{} {:+.3} S$/t", t.id, t.delta))
        .collect();
    format!(
        "Same inputs, different timing, so this tests the cash-flow bookkeeping. \
         Largest terms: {}",
        named.join("; ")
    )
}

fn dcf_rows(m: &Model, rows: &mut Vec<Row>) -> Result<()> {
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    for (d, _) in dcf_grid(m, &b)? {
        let tag = format!("{}_{}Y_{}", d.stage.id(), d.life_y, n_tag(d.n));
        rows.push(Row {
            test_id: format!("LAC_DCF_VS_L5_{tag}"),
            level: 5,
            subject: "Levelised abatement cost against the L5 ledger".to_string(),
            source: "Method 1 annual ledger".to_string(),
            locator: "build order section 2".to_string(),
            conditions: format!(
                "{}, {} y, n = {:.2}, {:.0} MWth charged",
                d.stage.id(),
                d.life_y,
                d.n,
                d.r_charged
            ),
            literature: Some(d.l5),
            simulated: d.lac,
            unit: "S$/t".to_string(),
            abs_error: Some(d.gap_abs.abs()),
            rel_error: Some(d.gap_rel.abs()),
            tolerance: Some(LAC_GAP_LIMIT),
            status: Status::of(d.gap_rel.abs() <= LAC_GAP_LIMIT),
            data_use: SAME,
            explanation: gap_text(&d),
        });
    }
    Ok(())
}

/// Ids in the registry that the model never reads.
pub fn unread_params(p: &ParamSet) -> Result<Vec<String>> {
    let (_, read) = Model::new_logged(p)?;
    Ok(p.entries()
        .filter(|e| !read.contains(&e.id))
        .map(|e| e.id.clone())
        .collect())
}

/// Largest carbon residual of the fixed runs, kmol/h.
pub fn max_carbon_residual(m: &Model) -> Result<f64> {
    let mut worst: f64 = 0.0;
    for r in RUNS {
        let s = m.scale(r.case);
        let b = m.balance(r.scn, &m.opts(s))?;
        worst = worst.max((b.c.closure * s).abs());
    }
    Ok(worst)
}

/// The plant reproduction rows: the screening checks, goldens, headline values,
/// sizing walk, cash-flow gaps and the export and registry checks. Each one
/// verifies the port and must pass.
pub fn plant_rows() -> Result<Vec<Row>> {
    let p = ParamSet::embedded()?;
    let m = Model::new(&p)?;
    let mut rows = Vec::new();

    for c in run_checks(&m)? {
        let abs = (c.got - c.target).abs();
        rows.push(Row {
            test_id: format!("SCREENING_{}", c.id),
            level: 4,
            subject: "Screening reproduction check".to_string(),
            source: c.source.clone(),
            locator: c.locator.clone(),
            conditions: "Central inputs".to_string(),
            literature: Some(c.target),
            simulated: c.got,
            unit: c.unit.clone(),
            abs_error: Some(abs),
            rel_error: (c.target != 0.0).then(|| abs / c.target.abs()),
            tolerance: Some(c.tolerance),
            status: Status::of(c.passed()),
            data_use: SAME,
            explanation: PORT_NOTE.to_string(),
        });
    }

    rows.push(stat_row(
        "GOLD_BALANCE_ALL_CASES",
        4,
        "Carbon, heat and power balance, constants and swap table",
        &balance_all_cases()?,
    ));
    rows.push(stat_row(
        "GOLD_LEDGER_GRID",
        5,
        "Every L0 to L5 cell, ten cases",
        &ledger_grid()?,
    ));
    rows.push(stat_row(
        "GOLD_AVAILABILITY",
        5,
        "Freeze values at availability 0.65, 0.70, 0.75, 0.85, 0.90",
        &availability()?,
    ));

    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let e = m.emissions(&b.c, m.s_b);
    let cond = "Case B, REC_F, Anchored";
    rows.push(point(
        pt("HEADLINE_H2_T_Y", 4, "H2 per year", "t/y", cond),
        66925.0,
        m.h2_b_kgh * m.hours / 1000.0,
        1.0,
    ));
    rows.push(point(
        pt("HEADLINE_PROCESS_HEAT", 4, "Process heat", "MWth", cond),
        120.8,
        b.q_proc,
        0.05,
    ));
    rows.push(point(
        pt("HEADLINE_NG_MMSCFD", 4, "Natural gas", "MMSCFD", cond),
        23.2,
        b.c.ng * m.s_b,
        0.05,
    ));
    rows.push(point(
        pt("HEADLINE_REACTOR_NEED", 4, "Reactor needed", "MWth", cond),
        179.0,
        b.r,
        0.05,
    ));
    rows.push(point(
        pt("HEADLINE_SPARE", 4, "Spare against 210 MWth", "MWth", cond),
        31.0,
        m.r_rated - b.r,
        0.05,
    ));
    rows.push(point(
        pt("HEADLINE_ABATEMENT", 4, "Net abatement", "t/y", cond),
        578052.0,
        e.a_atm,
        1.0,
    ));

    for (id, stage, life, r_at, target) in [
        ("L3_BOAK_25Y_210", Stage::Boak, 25.0, m.r_rated, 137.7),
        ("L3_BOAK_60Y_210", Stage::Boak, 60.0, m.r_rated, 117.2),
        ("L3_FOAK_25Y_210", Stage::Foak, 25.0, m.r_rated, 209.9),
        ("L3_FOAK_60Y_210", Stage::Foak, 60.0, m.r_rated, 179.5),
        ("L3_BOAK_25Y_NEED", Stage::Boak, 25.0, b.r, 113.1),
        ("L3_BOAK_60Y_NEED", Stage::Boak, 60.0, b.r, 95.5),
    ] {
        let q = CostQuery::central(stage, life, m.n_linear, r_at);
        let got = m.ledger(&b, &q)?.l3() * 1e6 / e.a_atm;
        let name = format!("HEADLINE_{id}");
        rows.push(point(
            pt(&name, 5, "L3 ledger step", "S$/t", "Case B, REC_F, n = 1.0"),
            target,
            got,
            0.05,
        ));
    }

    let walk = m.sizing_walk(m.s_b)?;
    for (i, target) in [206.0, 140.8, 150.6, 154.3, 165.9, 166.2, 179.0]
        .iter()
        .enumerate()
    {
        let name = format!("SIZING_WALK_STEP_{i}");
        rows.push(point(
            pt(&name, 4, "Sizing walk step", "MWth", cond),
            *target,
            walk[i],
            0.05,
        ));
    }

    rows.push(point(
        pt(
            "REBOILER_ORDER_PROBE",
            4,
            "Export steam spent second at partial demand",
            "MW",
            "Case A, demand set to Selexol steam plus 5 MW",
        ),
        5.0,
        reboiler_order_probe(&m)?,
        1e-9,
    ));

    rows.push(point(
        pt(
            "CARBON_CLOSURE_ALL_RUNS",
            3,
            "Largest carbon residual of the six fixed runs",
            "kmol/h",
            "All fixed runs",
        ),
        0.0,
        max_carbon_residual(&m)?,
        0.01,
    ));

    dcf_rows(&m, &mut rows)?;

    let first = tables(&p)?;
    let second = tables(&p)?;
    rows.push(point(
        pt(
            "EXPORT_BYTE_IDENTICAL",
            6,
            "Two exports in one process match",
            "tables differing",
            "Six plant tables",
        ),
        0.0,
        if first == second { 0.0 } else { 1.0 },
        0.0,
    ));

    let unread = unread_params(&p)?;
    let conditions = format!("{} entries in parameters.toml", p.len());
    let mut row = point(
        pt(
            "PARAMS_COMPLETE",
            6,
            "Registry entries the model never reads",
            "entries",
            &conditions,
        ),
        0.0,
        unread.len() as f64,
        0.0,
    );
    if !unread.is_empty() {
        row.explanation = format!("Unread: {}", unread.join(" "));
    }
    rows.push(row);

    Ok(rows)
}

/// A flowsheet comparison as a validation row.
fn check_row(c: Check) -> Row {
    Row {
        test_id: c.test_id,
        level: c.level,
        subject: c.subject,
        source: c.source,
        locator: c.locator,
        conditions: c.conditions,
        literature: c.literature,
        simulated: c.simulated,
        unit: c.unit,
        abs_error: c.abs_error,
        rel_error: c.rel_error,
        tolerance: Some(c.tolerance),
        status: Status::of(c.pass),
        data_use: c.data_use,
        explanation: c.explanation,
    }
}

/// Level 3 flowsheet rows (M7), from a suite solved in this call.
pub fn flowsheet_rows() -> Result<Vec<Row>> {
    let m = Model::embedded()?;
    let s = flowsheet::suite(&m)?;
    Ok(flowsheet::checks(&m, &s)?
        .into_iter()
        .map(check_row)
        .collect())
}

/// Literature benchmarks that run in seconds: the M2 property rows, the matrix
/// rows run by this crate, the Aronu points and the not-run rows.
pub fn literature_rows() -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    crate::props_bench::rows_m2(&mut rows)?;
    crate::gas_bench::tev953(&mut rows)?;
    crate::gas_bench::tev961(&mut rows)?;
    crate::gas_bench::xu_froment(&mut rows)?;
    crate::gas_bench::chinchen(&mut rows)?;
    crate::gas_bench::tecdoc1236(&mut rows)?;
    crate::mea_bench::aronu(&mut rows)?;
    crate::mea_bench::hartono(&mut rows)?;
    crate::mea_bench::versteeg(&mut rows)?;
    crate::ihx_bench::inl_11109(&mut rows)?;
    crate::ihx_bench::tecdoc_1382(&mut rows)?;
    crate::not_run::matrix_rows(&mut rows)?;
    crate::not_run::section18_rows(&mut rows);
    Ok(rows)
}

/// Unit benchmarks of `nush2_units::bench`: M4 and M5, the LTS sensitivity on the
/// NIST constant and, when `columns` is set, the M6 column rows.
pub fn unit_rows(columns: bool) -> Result<Vec<Row>> {
    Ok(unit_rows_with_gate(columns)?.0)
}

/// As `unit_rows`, with the M6 column gate when the columns ran.
pub fn unit_rows_with_gate(columns: bool) -> Result<(Vec<Row>, Option<Evidence>)> {
    let mut rows = crate::units_bench::gas_rows()?;
    rows.push(crate::units_bench::lts_nist_sensitivity()?);
    let mut gate = None;
    if columns {
        let (c, g) = crate::units_bench::column_rows_with_gate()?;
        rows.extend(c);
        gate = Some(g);
    }
    Ok((rows, gate))
}

/// One `GATE_<term>` row per gated term, judged on the rows already produced
/// (module `nush2_plant::gates`). When the M6 columns ran, the four M6 gates
/// must agree with `nush2_units::bench::column_gate`, or the harness stops.
pub fn gate_rows(rows: &[Row], column_gate: Option<Evidence>) -> Result<Vec<Row>> {
    let pairs: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.test_id.as_str(), r.status.label()))
        .collect();
    let mut out = Vec::new();
    for g in GATES.iter() {
        let (state, blocking, judged) = g.evaluate(pairs.iter().copied());
        if g.gate == "M6" {
            if let Some(e) = column_gate {
                let want = match e {
                    Evidence::Validated => GateState::Passed,
                    Evidence::GateFailed => GateState::Failed,
                    _ => GateState::NotRun,
                };
                if want != state {
                    return Err(nush2_numerics::EngineError::Parse(format!(
                        "gate of {} reads {} on the rows and {:?} in column_gate",
                        g.term,
                        state.label(),
                        e
                    )));
                }
            }
        }
        let (conditions, explanation) = match g.rows {
            Rows::None(reason) => ("no rows".to_string(), format!("not_run: {reason}")),
            Rows::Select { .. } => (
                format!("{judged} rows judged against a limit; {}", g.what),
                match state {
                    GateState::Passed => "Every gate row passed".to_string(),
                    GateState::Failed => format!("Failed rows: {}", blocking.join(" ")),
                    GateState::NotRun => {
                        if blocking.is_empty() {
                            "not_run: no gate row was judged".to_string()
                        } else {
                            format!("not_run: rows not run: {}", blocking.join(" "))
                        }
                    }
                },
            ),
        };
        let status = match state {
            GateState::Passed => Status::Pass,
            GateState::Failed => Status::Fail,
            GateState::NotRun => Status::NotRun,
        };
        out.push(Row {
            test_id: format!("GATE_{}", g.term),
            level: 4,
            subject: format!("Gate {} of term {}", g.gate, g.term),
            source: "Validation_Results.csv rows of this export".to_string(),
            locator: "plan sections 7 and 14".to_string(),
            conditions,
            literature: Some(0.0),
            simulated: if state == GateState::NotRun {
                f64::NAN
            } else {
                blocking.len() as f64
            },
            unit: "rows failed".to_string(),
            abs_error: None,
            rel_error: None,
            tolerance: if state == GateState::NotRun {
                None
            } else {
                Some(0.0)
            },
            status,
            data_use: SAME,
            explanation,
        });
    }
    Ok(out)
}

/// Everything: the whole of `Validation_Results.csv`. A flowsheet that does not
/// solve leaves its rows out and says so on stderr, as the exporter did before.
pub fn run_all() -> Result<Vec<Row>> {
    let mut rows = plant_rows()?;
    match flowsheet_rows() {
        Ok(fs) => rows.extend(fs),
        Err(e) => eprintln!("flowsheet rows not written, the flowsheet did not solve: {e}"),
    }
    rows.extend(literature_rows()?);
    let (units, column_gate) = unit_rows_with_gate(true)?;
    rows.extend(units);
    let gates = gate_rows(&rows, column_gate)?;
    rows.extend(gates);
    let mut seen = std::collections::BTreeSet::new();
    let twice: Vec<&str> = rows
        .iter()
        .filter(|r| !seen.insert(r.test_id.as_str()))
        .map(|r| r.test_id.as_str())
        .collect();
    if !twice.is_empty() {
        return Err(nush2_numerics::EngineError::Parse(format!(
            "test ids repeat: {}",
            twice.join(" ")
        )));
    }
    Ok(rows)
}
