//! CSV exporter. Returns strings, so the same tables reach the CLI and the
//! browser. Schemas follow the results file headers, with a trailing `status` column.
//! Economic rows also carry `dcf_value` and `dcf_unit`, because S$/kg and S$/t
//! have no home in a S$M/y column. Rows without a model read `not_run` and stay
//! empty.
//!
//! Every result row is the Headline preset: the anchored balance with the terms
//! whose gates passed on their models (module `terms`). The energy balance also
//! carries Case B REC_F on the Anchored and Modelled presets, so the drift sits
//! in the file.
//!
//! Status comes from the V&V record. Each table is built twice, on the
//! Headline preset and on the Anchored preset. A row whose cells
//! differ between the two rests on a term the Headline preset took from its
//! model because its gate passed, and its status names those terms, for
//! example `gated_T05`. A row that matches the Anchored run keeps
//! `anchored_port`, because that is what it is. Rows of the Modelled preset
//! that differ from the Anchored rows of the same term take the word of that
//! preset's gated terms in the same way.
//!
//! The mass balance carries the Level 3 flowsheet runs of M7 with every species,
//! under run ids that start with `FS_` and the status `flowsheet_m7`. Rows that
//! M8 models compute (heat recovery on real flows, the Level 4 rebuild of the
//! reactor side, the reboiler grade check) read `level4_m8`. Sweep rows that rest
//! on a Level 3 result read `flowsheet_m8`, and a sweep point that did not solve
//! reads `not_converged` with empty results.

use crate::anchored::{Balance, Case, CostLevel, Model, Route, Scenario};
use crate::carbon::carbon_kmol_h;
use crate::dcf::DcfResult;
use crate::drift::{self, TermRecord};
use crate::economics::{CostQuery, Stage, LEVELS};
use crate::flowsheet::{self, FlowsheetRun, Suite};
use crate::gates::Gates;
use crate::heatrec::{self, SourceId};
use crate::params::{ParamEntry, ParamSet};
use crate::sens::{sweeps, FsInputs, FsPoint, SensRow};
use crate::terms::{Source, Term};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::KSource;
use nush2_props::units::{kmol_h_to_t_h, mmscfd_to_kmol_h, MW_CO2, MW_H2, MW_H2O, MW_MEA};
use nush2_units::case5::T05_LIMIT_K;
use nush2_units::stream::{EnergyKind, ModelClass, Port, Sp};

/// Status of every result row at this milestone.
pub const STATUS: &str = "anchored_port";
/// Status of rows computed by the Level 3 flowsheet.
pub const FLOWSHEET_STATUS: &str = "flowsheet_m7";
/// Status of rows computed by the M8 models on real flows.
pub const LEVEL4_STATUS: &str = "level4_m8";

/// Terms each preset took from a model, read from the term record, which the
/// V&V gate outcomes decide. They name the status of rows that rest on them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marks {
    pub headline: Vec<&'static str>,
    pub modelled: Vec<&'static str>,
}

impl Marks {
    pub fn from_record(rec: &TermRecord) -> Self {
        let used = |list: &[Term]| -> Vec<&'static str> {
            list.iter()
                .filter(|t| t.used == Source::Modelled)
                .map(|t| t.id)
                .collect()
        };
        Self {
            headline: used(&rec.headline),
            modelled: used(&rec.modelled),
        }
    }

    fn word(ids: &[&str]) -> Option<String> {
        if ids.is_empty() {
            None
        } else {
            Some(format!("gated_{}", ids.join("_")))
        }
    }

    /// Status of a Headline row that rests on a modelled term, `None` when the
    /// Headline preset uses no model.
    pub fn headline_word(&self) -> Option<String> {
        Self::word(&self.headline)
    }

    /// The same for the Modelled preset.
    pub fn modelled_word(&self) -> Option<String> {
        Self::word(&self.modelled)
    }
}

/// Status of a design-basis result row. A modelled term slot carries the
/// milestone of its model; the value a preset uses carries the gated word when
/// that preset took the term from its model; Level 4 rows read `level4_m8`.
fn row_status(id: &str, marks: &Marks) -> String {
    let slot = id
        .strip_prefix("TERM_")
        .and_then(|r| r.strip_suffix("_MODELLED"))
        .filter(|t| t.len() == 3);
    if let Some(t) = slot {
        return match t {
            "T01" | "T05" => "model_m4",
            "T02" | "T06" | "T07" | "T08" => "model_m6",
            "T10" => LEVEL4_STATUS,
            _ => FLOWSHEET_STATUS,
        }
        .to_string();
    }
    for (suffix, list, word) in [
        ("_USED_HEADLINE", &marks.headline, marks.headline_word()),
        ("_USED_MODELLED", &marks.modelled, marks.modelled_word()),
    ] {
        let used = id
            .strip_prefix("TERM_")
            .and_then(|r| r.strip_suffix(suffix))
            .filter(|t| list.contains(t));
        if let (Some(_), Some(w)) = (used, word) {
            return w;
        }
    }
    if let Some(rest) = id.strip_prefix("DRIFT_") {
        let word = if rest.starts_with("HEADLINE_") {
            marks.headline_word()
        } else if rest.starts_with("MODELLED_") {
            marks.modelled_word()
        } else {
            None
        };
        return word.unwrap_or_else(|| STATUS.to_string());
    }
    if id.starts_with("TUBE_") {
        return TUBE_STATUS.to_string();
    }
    if id.starts_with("M6_ISLAND_") {
        return "model_m6".to_string();
    }
    if id.starts_with("L4_") {
        LEVEL4_STATUS.to_string()
    } else {
        STATUS.to_string()
    }
}

/// Source text of a result row from its status.
fn status_source(status: &str) -> &'static str {
    match status {
        "model_m4" => "engine M4 gas units",
        "model_m6" => "engine M6 column models",
        TUBE_STATUS => {
            "engine M5 Case B tube model: Hoseinzade and Adams 2017 Table 6 tube (PDF p11) with \
             the JAERI-Tech 2005-031 finned helium annulus"
        }
        FLOWSHEET_STATUS => "engine Level 3 flowsheet",
        LEVEL4_STATUS => "engine Level 4 heat recovery and utility",
        s if s.starts_with("gated_") => "engine anchored port with the gated model terms",
        _ => "engine anchored port",
    }
}

/// Split one CSV line into cells, undoing `esc`.
fn cells(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}

/// Two printed cells hold the same value: equal as text, or both numbers that
/// compare equal, so `-0.000000` and `0.000000` match. A residual near zero can
/// print with either sign on different platforms, and the status must not
/// follow that sign.
fn same_cell(a: &str, b: &str) -> bool {
    a == b || matches!((a.parse::<f64>(), b.parse::<f64>()), (Ok(x), Ok(y)) if x == y)
}

/// The cells of two rows, the status cell left out, hold the same values.
fn same_cells(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same_cell(x, y))
}

/// Replace the last cell, the status, of one CSV line.
fn with_status(line: &str, word: &str) -> String {
    match line.rsplit_once(',') {
        Some((head, _)) => format!("{head},{}", esc(word)),
        None => line.to_string(),
    }
}

/// Mark the rows of `head`, a table on the Headline preset, that differ from
/// the same row of `anch`, the table the same code builds on the Anchored
/// preset. Rows pair by line. Only rows that still read `anchored_port` change,
/// so a row with a status of its own keeps it.
fn mark(head: String, anch: &str, word: Option<&str>) -> Result<String> {
    let Some(word) = word else {
        return Ok(head);
    };
    let hl: Vec<&str> = head.lines().collect();
    let al: Vec<&str> = anch.lines().collect();
    if hl.len() != al.len() {
        return Err(EngineError::Parse(format!(
            "the Headline table has {} lines and the Anchored table {}, so rows cannot pair",
            hl.len(),
            al.len()
        )));
    }
    let mut out = String::with_capacity(head.len() + 1024);
    for (i, (h, a)) in hl.iter().zip(&al).enumerate() {
        let hc = cells(h);
        let ac = cells(a);
        let differs = hc.len() != ac.len() || !same_cells(&hc[..hc.len() - 1], &ac[..ac.len() - 1]);
        if i > 0 && differs && hc.last().map(String::as_str) == Some(STATUS) {
            out.push_str(&with_status(h, word));
        } else {
            out.push_str(h);
        }
        out.push('\n');
    }
    Ok(out)
}

/// Mark the energy rows of the Modelled preset whose values differ from the
/// Anchored rows of the same term.
fn mark_modelled(text: String, word: Option<&str>) -> String {
    let Some(word) = word else {
        return text;
    };
    let anchored: std::collections::BTreeMap<String, Vec<String>> = text
        .lines()
        .map(cells)
        .filter(|c| c.len() > 8 && c[0] == "B_REC_F_ANCHORED")
        .map(|c| (c[1].clone(), c[5..8].to_vec()))
        .collect();
    let mut out = String::with_capacity(text.len() + 1024);
    for line in text.lines() {
        let c = cells(line);
        let differs = c.len() > 8
            && c[0] == "B_REC_F_MODELLED"
            && c.last().map(String::as_str) == Some(STATUS)
            && anchored
                .get(&c[1])
                .is_some_and(|v| !same_cells(v, &c[5..8]));
        if differs {
            out.push_str(&with_status(line, word));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// A fixed exporter run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunDef {
    pub id: &'static str,
    pub case: Case,
    pub scn: Scenario,
}

/// Run definitions of the fixed runs, sorted by id.
pub const RUNS: [RunDef; 6] = [
    RunDef {
        id: "A_INL_SEL",
        case: Case::A,
        scn: Scenario::InlSel,
    },
    RunDef {
        id: "A_REC_F",
        case: Case::A,
        scn: Scenario::RecF,
    },
    RunDef {
        id: "B_COMB_H",
        case: Case::B,
        scn: Scenario::CombH,
    },
    RunDef {
        id: "B_EXPORT",
        case: Case::B,
        scn: Scenario::Export,
    },
    RunDef {
        id: "B_REC_F",
        case: Case::B,
        scn: Scenario::RecF,
    },
    RunDef {
        id: "B_REC_FIX",
        case: Case::B,
        scn: Scenario::RecFix,
    },
];

/// The six tables the plant crate can build. Validation lives in `nush2_vv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tables {
    pub final_design_basis: String,
    pub integrated_mass_balance: String,
    pub integrated_energy_balance: String,
    pub emissions_ledger: String,
    pub economic_ledger: String,
    pub sensitivity_results: String,
}

/// CSV text builder with `\n` line ends.
struct Csv {
    out: String,
    cols: usize,
}

impl Csv {
    fn new(header: &[&str]) -> Self {
        Self {
            out: format!("{}\n", header.join(",")),
            cols: header.len(),
        }
    }

    fn row(&mut self, cells: &[String]) {
        debug_assert_eq!(cells.len(), self.cols);
        let line: Vec<String> = cells.iter().map(|c| esc(c)).collect();
        self.out.push_str(&line.join(","));
        self.out.push('\n');
    }
}

/// CSV quoting for one cell.
pub fn esc(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Fixed six decimals, the default for results.
pub fn num(v: f64) -> String {
    format!("{v:.6}")
}

fn opt(v: Option<f64>) -> String {
    v.map(num).unwrap_or_default()
}

/// Shortest text that reads back to the same value, used for inputs.
fn exact(v: f64) -> String {
    format!("{v}")
}

fn t(s: &str) -> String {
    s.to_string()
}

fn plain_name(id: &str) -> String {
    let mut s = id.replace('_', " ").to_lowercase();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

/// Which cases an input applies to.
fn scope(id: &str) -> (bool, bool) {
    match id {
        "H2_B" | "Q_PROCESS_AHN_CASE_B" => (true, false),
        "H2_A" | "H2_A_MMSCFD" | "WATER_GPM_CASE_A" => (false, true),
        _ => (true, true),
    }
}

/// Stable tag for a size exponent.
pub fn n_tag(n: f64) -> String {
    format!("N{:.2}", n).replace('.', "P")
}

/// The fixed run of id `id`.
fn run_def(id: &str) -> Result<RunDef> {
    RUNS.iter()
        .find(|r| r.id == id)
        .copied()
        .ok_or_else(|| EngineError::Parse(format!("{id} missing from RUNS")))
}

fn balance_for(m: &Model, r: &RunDef) -> Result<Balance> {
    m.balance(r.scn, &m.opts(m.scale(r.case)))
}

/// All tables from one parameter set on the recorded gates.
pub fn tables(p: &ParamSet) -> Result<Tables> {
    tables_with(p, &Gates::recorded())
}

/// All tables from one parameter set on the given gate outcomes. `p` is the
/// registry without a preset; the Headline registry is built here.
pub fn tables_with(p: &ParamSet, gates: &Gates) -> Result<Tables> {
    let m0 = Model::new(p)?;
    // A flowsheet that fails to converge writes `not_converged` rows and keeps
    // the anchored terms; it never stops the anchored tables.
    let fs = flowsheet::suite(&m0);
    let rec = drift::record(p, &fs, gates)?;
    let mh = Model::new(&rec.p_head)?;
    let mm = Model::new(&rec.p_mod)?;
    let marks = Marks::from_record(&rec);
    let hw = marks.headline_word();
    let hw = hw.as_deref();
    // Flowsheet solves run once and feed both builds of the sensitivity table.
    let fsx = fs_inputs(&m0, &fs);
    let cryo = cryo_split_rows(&m0, &fs)?;
    let energy = mark(
        energy_balance(&mh, &m0, &mm, &fs)?,
        &energy_balance(&m0, &m0, &mm, &fs)?,
        hw,
    )?;
    // The Modelled preset's carbon ledger, written as B_REC_F_MODELLED rows.
    let mw = marks.modelled_word();
    let modelled = Some((&mm, &m0, mw.as_deref().unwrap_or(STATUS)));
    Ok(Tables {
        final_design_basis: mark(
            final_design_basis(p, &mh, &rec, &marks)?,
            &final_design_basis(p, &m0, &rec, &marks)?,
            hw,
        )?,
        integrated_mass_balance: mark(
            mass_balance(&mh, modelled, &fs)?,
            &mass_balance(&m0, modelled, &fs)?,
            hw,
        )?,
        integrated_energy_balance: mark_modelled(energy, marks.modelled_word().as_deref()),
        emissions_ledger: mark(
            emissions_ledger(&mh, modelled)?,
            &emissions_ledger(&m0, modelled)?,
            hw,
        )?,
        economic_ledger: mark(economic_ledger(&mh)?, &economic_ledger(&m0)?, hw)?,
        sensitivity_results: mark(
            sensitivity(&rec.p_head, &fsx, &cryo)?,
            &sensitivity(p, &fsx, &cryo)?,
            hw,
        )?,
    })
}

fn fs_point(r: &FlowsheetRun) -> FsPoint {
    FsPoint {
        ng_mmscfd: r.ng_mmscfd,
        process_heat_mw: r.process_heat_mw(),
        captured_kmol_h: r.carbon.captured,
        emitted_c_kmol_h: r.carbon.burned,
    }
}

/// Level 3 inputs of the flowsheet sweeps: the S8 reformer runs of the suite,
/// one Case B REC_F run on NIST Gibbs constants (S2) and the condensate CO2 of
/// FS_B_REC_F (S14). A run that fails is `None`.
pub fn fs_inputs(m0: &Model, fs: &Result<Suite>) -> FsInputs {
    let suite = fs.as_ref().ok();
    let base_run = suite.and_then(|s| s.run("FS_B_REC_F"));
    let reformer = flowsheet::T_REFORMER_SENSITIVITIES_C
        .iter()
        .map(|t| {
            let id = format!("FS_B_REC_F_T{t:.0}");
            (*t, suite.and_then(|s| s.run(&id)).map(fs_point))
        })
        .collect();
    let k_nist = base_run.and_then(|r| {
        let mut spec = r.spec.clone();
        spec.k_source = KSource::NistGibbs;
        flowsheet::solve(
            m0,
            &spec,
            "FS_B_REC_F_K_NIST",
            flowsheet::TearStart::Anchored,
        )
        .ok()
        .map(|run| fs_point(&run))
    });
    FsInputs {
        base: base_run.map(fs_point),
        reformer,
        k_nist,
        condensate_co2_kmol_h: base_run.map(|r| r.carbon.condensate),
        syngas_mw: base_run
            .and_then(|r| r.gas("S09A").ok())
            .and_then(|g| heatrec::syngas_above_w(g, T05_LIMIT_K).ok())
            .map(|w| w / 1.0e6),
    }
}

struct ResultRow {
    id: String,
    group: &'static str,
    parameter: String,
    b: Option<f64>,
    a_inl: Option<f64>,
    a_mea: Option<f64>,
    unit: &'static str,
    equation: &'static str,
    decision: &'static str,
}

fn final_design_basis(p: &ParamSet, m: &Model, rec: &TermRecord, marks: &Marks) -> Result<String> {
    let mut c = Csv::new(&[
        "id",
        "group",
        "parameter",
        "case_b",
        "case_a_inl",
        "case_a_mea",
        "unit",
        "low",
        "high",
        "class",
        "rigour",
        "source",
        "locator",
        "source_status",
        "equation",
        "decision",
        "status",
    ]);
    for e in p.entries() {
        input_row(&mut c, e);
    }
    let mut rows: Vec<ResultRow> = Vec::new();
    results_block(m, &mut rows)?;
    dcf_block(m, &mut rows)?;
    term_block(rec, &mut rows);
    level4_block(rec, &mut rows);
    tube_block(&mut rows)?;
    island_block(&mut rows)?;
    drift_block(p, rec, &mut rows)?;
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    for r in rows {
        let status = row_status(&r.id, marks);
        c.row(&[
            r.id,
            t(r.group),
            r.parameter,
            opt(r.b),
            opt(r.a_inl),
            opt(r.a_mea),
            t(r.unit),
            String::new(),
            String::new(),
            t("calculated"),
            t("reduced_order"),
            t(status_source(&status)),
            t("see equation"),
            String::new(),
            t(r.equation),
            t(r.decision),
            status,
        ]);
    }
    Ok(c.out)
}

fn input_row(c: &mut Csv, e: &ParamEntry) {
    let (b, a) = scope(&e.id);
    let v = exact(e.value);
    c.row(&[
        e.id.clone(),
        e.group.clone(),
        if e.name.is_empty() {
            plain_name(&e.id)
        } else {
            e.name.clone()
        },
        if b { v.clone() } else { String::new() },
        if a { v.clone() } else { String::new() },
        if a { v } else { String::new() },
        e.unit.clone(),
        e.low.map(exact).unwrap_or_default(),
        e.high.map(exact).unwrap_or_default(),
        e.class.clone(),
        e.rigour.clone(),
        e.source.clone(),
        e.locator.clone(),
        e.source_status.clone(),
        String::new(),
        e.decision.clone(),
        t("input"),
    ]);
}

/// eq: K.6
/// eq: K.22
fn results_block(m: &Model, rows: &mut Vec<ResultRow>) -> Result<()> {
    let sb = m.case_summary(Scenario::RecF, m.s_b)?;
    let sa_inl = m.case_summary(Scenario::InlSel, 1.0)?;
    let sa_mea = m.case_summary(Scenario::RecF, 1.0)?;
    // id, summary key, group, unit, equation, decision
    let keys: [(&str, &str, &str, &str, &str, &str); 44] = [
        ("RES_H2_KG_H", "h2_kg_h", "production", "kg/h", "J.2", "D03"),
        ("RES_H2_T_Y", "h2_t_y", "production", "t/y", "J.3", "D03"),
        ("RES_HOURS", "hours", "production", "h/y", "J.3", "D05"),
        ("RES_DAYS", "days", "production", "d/y", "J.3", "D05"),
        (
            "RES_PSA_H2_FEED_KG_H",
            "psa_h2_feed_kg_h",
            "production",
            "kg/h",
            "J.12",
            "D04",
        ),
        (
            "RES_PSA_H2_REJECT_KG_H",
            "psa_h2_reject_kg_h",
            "production",
            "kg/h",
            "J.12",
            "D04",
        ),
        (
            "RES_NG_MMSCFD",
            "ng_mmscfd",
            "production",
            "MMSCFD",
            "J.4",
            "",
        ),
        (
            "RES_NG_UNABATED_MMSCFD",
            "ng_unabated_mmscfd",
            "production",
            "MMSCFD",
            "J.4",
            "",
        ),
        (
            "RES_NG_NORECYCLE_MMSCFD",
            "ng_norecycle_mmscfd",
            "production",
            "MMSCFD",
            "J.4",
            "",
        ),
        ("RES_WATER_GPM", "water_gpm", "production", "gpm", "J.1", ""),
        (
            "RES_FEED_C_KMOL_H",
            "feed_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.5",
            "",
        ),
        (
            "RES_TUBE_C_KMOL_H",
            "tube_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.8",
            "",
        ),
        (
            "RES_CO2_ABS_KMOL_H",
            "co2_abs_kmol_h",
            "carbon",
            "kmol/h",
            "J.6",
            "",
        ),
        (
            "RES_CAPTURED_KMOL_H",
            "cap_kmol_h",
            "carbon",
            "kmol/h",
            "J.9",
            "",
        ),
        (
            "RES_SLIP_KMOL_H",
            "slip_kmol_h",
            "carbon",
            "kmol/h",
            "J.9",
            "",
        ),
        (
            "RES_TAILGAS_C_KMOL_H",
            "tailgas_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.10",
            "D04",
        ),
        (
            "RES_RECYCLED_C_KMOL_H",
            "recycled_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.10",
            "D04",
        ),
        (
            "RES_BURNED_C_KMOL_H",
            "burned_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.10",
            "D04",
        ),
        (
            "RES_PRODUCT_C_KMOL_H",
            "product_C_kmol_h",
            "carbon",
            "kmol/h",
            "J.11",
            "",
        ),
        (
            "RES_BASELINE_T_Y",
            "baseline_t_y",
            "carbon",
            "t/y",
            "K.3",
            "D07",
        ),
        (
            "RES_CAPTURED_T_Y",
            "captured_t_y",
            "carbon",
            "t/y",
            "K.7",
            "D07",
        ),
        ("RES_ATM_T_Y", "atm_t_y", "carbon", "t/y", "K.8", "D07"),
        ("RES_ABATEMENT_T_Y", "A_t_y", "carbon", "t/y", "K.11", "D07"),
        (
            "RES_ABATEMENT_CHO_T_Y",
            "A_cho",
            "carbon",
            "t/y",
            "K.12",
            "D07",
        ),
        (
            "RES_ABATEMENT_IEAGHG_T_Y",
            "A_ieaghg",
            "carbon",
            "t/y",
            "K.12",
            "D07",
        ),
        (
            "RES_PLANT_CAPTURE",
            "plant_capture",
            "carbon",
            "-",
            "J.33",
            "D07",
        ),
        ("RES_REDUCTION", "reduction", "carbon", "-", "K.13", "D07"),
        (
            "RES_INTENSITY_KG_KG",
            "ci_kg_kg",
            "carbon",
            "kg/kg",
            "K.13",
            "D07",
        ),
        ("RES_PROCESS_HEAT_MW", "q_proc", "heat", "MWth", "J.14", ""),
        ("RES_SLIP_HEAT_MW", "q_recyc", "heat", "MWth", "J.15", ""),
        ("RES_REBOILER_MW", "q_reb", "heat", "MW", "J.16", "D26"),
        ("RES_REBOILER_GAP_MW", "gap", "heat", "MW", "J.18", "D27"),
        ("RES_HE_SEC_KG_S", "he_sec_kg_s", "heat", "kg/s", "B.12", ""),
        (
            "RES_HE_PRIM_KG_S",
            "he_prim_kg_s_at_R",
            "heat",
            "kg/s",
            "B.12",
            "",
        ),
        (
            "RES_POWER_BRANCH_MWTH",
            "branch",
            "heat",
            "MWth",
            "B.20",
            "",
        ),
        (
            "RES_REACTOR_NEEDED_MWTH",
            "R",
            "heat",
            "MWth",
            "J.26",
            "D24",
        ),
        (
            "RES_MARGIN_210_MWTH",
            "margin_210",
            "heat",
            "MWth",
            "J.36",
            "D03",
        ),
        (
            "RES_REACTOR_BEST_MWTH",
            "R_best",
            "heat",
            "MWth",
            "J.27",
            "",
        ),
        (
            "RES_REACTOR_HIGH_MWTH",
            "R_high",
            "heat",
            "MWth",
            "J.27",
            "",
        ),
        ("RES_E_PROCESS_MWE", "e_proc", "power", "MWe", "J.19", ""),
        (
            "RES_E_CIRC_SEC_MWE",
            "circ_sec_MWe",
            "power",
            "MWe",
            "B.13",
            "",
        ),
        (
            "RES_E_CIRC_PRIM_MWE",
            "circ_prim_MWe",
            "power",
            "MWe",
            "B.13",
            "",
        ),
        ("RES_E_TOTAL_MWE", "e_total", "power", "MWe", "J.24", ""),
        (
            "RES_SOLVENT_T_H",
            "solvent_t_h",
            "conditions",
            "t/h",
            "J.20",
            "",
        ),
    ];
    for (id, key, group, unit, equation, decision) in keys {
        let parameter = if id == "RES_ABATEMENT_IEAGHG_T_Y" {
            format!(
                "Abatement on the IEAGHG bound of {:.6} kg/kg, unrounded: 0.8091 kg CO2/Nm3 over \
                 0.08994 kg H2/Nm3, IEAGHG 2017-02 Table 2",
                m.base_ieaghg
            )
        } else {
            plain_name(id.trim_start_matches("RES_"))
        };
        rows.push(ResultRow {
            id: id.to_string(),
            group,
            parameter,
            b: sb.get(key).copied(),
            a_inl: sa_inl.get(key).copied(),
            a_mea: sa_mea.get(key).copied(),
            unit,
            equation,
            decision,
        });
    }
    let one =
        |id: &str, parameter: String, v: f64, unit: &'static str, eq: &'static str| ResultRow {
            id: id.to_string(),
            group: "carbon",
            parameter,
            b: Some(v),
            a_inl: None,
            a_mea: None,
            unit,
            equation: eq,
            decision: "D07",
        };
    rows.push(one(
        "RES_BASELINE_IEAGHG_KG_KG",
        "IEAGHG baseline bound, unrounded, the one value every file uses".into(),
        m.base_ieaghg,
        "kg/kg",
        "K.2",
    ));
    // Carbon views the report prints: feed carbon as CO2, carbon never generated
    // against the baseline, and the gas saving on the INL pair and on Ahn.
    let hours = m.hours;
    let t_y = |kmol_h: f64| kmol_h_to_t_h(kmol_h) * hours;
    let get = |k: &str| sb.get(k).copied().unwrap_or(f64::NAN);
    let (base, capt, atm) = (get("baseline_t_y"), get("captured_t_y"), get("atm_t_y"));
    let feed = capt + atm + t_y(get("product_C_kmol_h"));
    rows.push(one(
        "RES_FEED_CARBON_T_Y",
        "Feed carbon as CO2: captured, to atmosphere and the methane in the product".into(),
        feed,
        "t/y",
        "K.6",
    ));
    rows.push(one(
        "RES_CARBON_NEVER_GENERATED_T_Y",
        "Baseline CO2 never generated: baseline less captured less atmosphere".into(),
        base - capt - atm,
        "t/y",
        "K.11",
    ));
    rows.push(one(
        "RES_GAS_SAVED_CARBON_T_Y",
        "Baseline CO2 less the feed carbon as CO2".into(),
        base - feed,
        "t/y",
        "K.11",
    ));
    let ng_b = get("ng_mmscfd");
    let ng_u = get("ng_unabated_mmscfd");
    rows.push(one(
        "RES_GAS_SAVING_INL",
        "Gas saved against the unabated SMR on the INL pair".into(),
        1.0 - ng_b / ng_u,
        "-",
        "L.16",
    ));
    rows.push(one(
        "RES_GAS_SAVING_AHN",
        "Gas saved against the unabated SMR on Ahn's equivalent pair".into(),
        1.0 - m.ng_ahn_equivalent / m.ng_conv,
        "-",
        "L.16",
    ));
    // The labelled SMR plus CCS comparator, derivation card L.32.
    let cmp = m.comparator();
    let label = format!(
        "Comparator, IEAGHG 2017-02 Case 3 (MEA on flue gas) at 100,000 Nm3/h, indicative: \
         abatement on the IEAGHG baseline of {:.6} kg/kg, the one value every file uses, at \
         this study's availability, cost as IEAGHG states it on its own gas price and \
         availability",
        m.base_ieaghg
    );
    for (id, what, v, unit, eq) in [
        (
            "RES_COMPARATOR_ABATEMENT_T_Y",
            "abatement",
            cmp.a_t_y,
            "t/y",
            "K.21",
        ),
        (
            "RES_COMPARATOR_CAPTURE",
            "overall capture",
            cmp.overall_capture,
            "-",
            "K.21",
        ),
        (
            "RES_COMPARATOR_CAC_TS23P2_S_T",
            "abatement cost as published, T&S EUR 10/t (S$23.2/t)",
            cmp.cac_t232,
            "S$/t",
            "L.32",
        ),
        (
            "RES_COMPARATOR_CAC_TS60_S_T",
            "abatement cost at T&S S$60/t",
            cmp.cac_t60,
            "S$/t",
            "L.32",
        ),
        (
            "RES_COMPARATOR_CAC_TS112_S_T",
            "abatement cost at T&S S$112/t",
            cmp.cac_t112,
            "S$/t",
            "L.32",
        ),
    ] {
        rows.push(ResultRow {
            group: "cost",
            ..one(id, format!("{label}: {what}"), v, unit, eq)
        });
    }
    let import = |scn: Scenario, s: f64| -> Result<f64> {
        let b = m.balance(scn, &m.opts(s))?;
        Ok(m.shortfall_at(m.r_rated, &b).0.max(0.0))
    };
    rows.push(ResultRow {
        id: "RES_GRID_IMPORT_AT_RATED_MWE".into(),
        group: "power",
        parameter: "Grid import at rated size".into(),
        b: Some(import(Scenario::RecF, m.s_b)?),
        a_inl: Some(import(Scenario::InlSel, 1.0)?),
        a_mea: Some(import(Scenario::RecF, 1.0)?),
        unit: "MWe",
        equation: "J.29",
        decision: "D03",
    });
    rows.push(ResultRow {
        id: "PHI_NORECYCLE".into(),
        group: "carbon",
        parameter: "Carbon split, no recycle".into(),
        b: Some(m.phi_nr),
        a_inl: Some(m.phi_nr),
        a_mea: Some(m.phi_nr),
        unit: "-",
        equation: "J.6",
        decision: "",
    });
    let bb = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let a = m.emissions(&bb.c, m.s_b).a_atm;
    rows.push(ResultRow {
        id: "RES_TARGET_ABATEMENT_MARGIN_T_Y".into(),
        group: "cost",
        parameter: "Abatement above the target".into(),
        b: Some(a - m.target_a),
        a_inl: None,
        a_mea: None,
        unit: "t/y",
        equation: "K.22",
        decision: "",
    });
    for (stage, life) in [
        (Stage::Boak, m.lives[0]),
        (Stage::Boak, m.lives[2]),
        (Stage::Foak, m.lives[0]),
        (Stage::Foak, m.lives[2]),
    ] {
        let q = CostQuery::central(stage, life, m.n_linear, m.r_rated);
        let l5 = m.ledger(&bb, &q)?.l5() * 1e6 / a;
        rows.push(ResultRow {
            id: format!("RES_TARGET_COST_EXCESS_L5_{}_{life:.0}Y", stage.id()),
            group: "cost",
            parameter: "L5 abatement cost above the target, positive means missed".into(),
            b: Some(l5 - m.target_c),
            a_inl: None,
            a_mea: None,
            unit: "S$/t",
            equation: "L.26",
            decision: "",
        });
        rows.push(ResultRow {
            id: format!("RES_TARGET_COST_EXCESS_DCF_{}_{life:.0}Y", stage.id()),
            group: "cost",
            parameter: "Levelised abatement cost above the target, positive means missed".into(),
            b: Some(m.dcf(&bb, &q)?.lac - m.target_c),
            a_inl: None,
            a_mea: None,
            unit: "S$/t",
            equation: "L.36",
            decision: "",
        });
    }
    let (cap, fom) = m.smr_island(m.s_b);
    rows.push(ResultRow {
        id: "RES_SMR_ISLAND_CAPITAL_SM".into(),
        group: "cost",
        parameter: "SMR island capital, S$2025".into(),
        b: Some(cap),
        a_inl: None,
        a_mea: None,
        unit: "S$M",
        equation: "L.34",
        decision: "D08",
    });
    rows.push(ResultRow {
        id: "RES_SMR_ISLAND_FOM_SM_Y".into(),
        group: "cost",
        parameter: "SMR island fixed O&M, S$2025".into(),
        b: Some(fom),
        a_inl: None,
        a_mea: None,
        unit: "S$M/y",
        equation: "L.34",
        decision: "D08",
    });
    rows.push(ResultRow {
        id: "RES_SMR_ISLAND_VOM_SM_Y".into(),
        group: "cost",
        parameter: "SMR island raw water, chemicals and catalysts, S$2025".into(),
        b: Some(m.smr_island_variable(m.s_b)),
        a_inl: None,
        a_mea: None,
        unit: "S$M/y",
        equation: "L.34",
        decision: "D08",
    });
    Ok(())
}

/// DCF queries printed in the results: BOAK and FOAK, 25 y and 60 y, three
/// exponents, rated reactor charged.
pub fn dcf_grid(m: &Model, b: &Balance) -> Result<Vec<(DcfResult, f64)>> {
    let mut out = Vec::new();
    for stage in [Stage::Boak, Stage::Foak] {
        for life in [m.lives[0], m.lives[2]] {
            for n in [m.n_linear, m.n_team, m.n_ngnp] {
                let q = CostQuery::central(stage, life, n, m.r_rated);
                let l3 = m.ledger(b, &q)?.l3() * 1e6 / m.emissions(&b.c, b.s).a_atm;
                out.push((m.dcf(b, &q)?, l3));
            }
        }
    }
    Ok(out)
}

fn dcf_block(m: &Model, rows: &mut Vec<ResultRow>) -> Result<()> {
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    for (d, l3) in dcf_grid(m, &b)? {
        let tag = format!("{}_{}Y_{}", d.stage.id(), d.life_y, n_tag(d.n));
        let mut add = |prefix: &str, name: &str, v: f64, unit: &'static str, eq: &'static str| {
            rows.push(ResultRow {
                id: format!("{prefix}_{tag}"),
                group: "cost",
                parameter: format!(
                    "{name}, {}, {} y, n = {:.2}, 210 MWth charged",
                    d.stage.id(),
                    d.life_y,
                    d.n
                ),
                b: Some(v),
                a_inl: None,
                a_mea: None,
                unit,
                equation: eq,
                decision: "D08",
            });
        };
        add(
            "LCOH_SMR",
            "LCOH of the unabated SMR",
            d.lcoh_smr,
            "S$/kg",
            "L.35",
        );
        add(
            "LCOH_NUSH2",
            "LCOH of NUS-H2",
            d.lcoh_nush2,
            "S$/kg",
            "L.35",
        );
        add("LAC_DCF", "Levelised abatement cost", d.lac, "S$/t", "L.36");
        add("AC_L5", "Ledger abatement cost at L5", d.l5, "S$/t", "L.26");
        add("AC_L3", "Ledger abatement cost at L3", l3, "S$/t", "L.26");
    }
    Ok(())
}

fn source_word(t: &Term) -> &'static str {
    match t.used {
        Source::Anchored => "anchored",
        Source::Modelled => "modelled",
    }
}

fn gate_text(t: &Term) -> String {
    match t.gate_state {
        Some(g) => format!("gate {} {}", t.gate, g.label()),
        None => "no gate".to_string(),
    }
}

fn term_block(rec: &TermRecord, rows: &mut Vec<ResultRow>) {
    for (h, m) in rec.headline.iter().zip(&rec.modelled) {
        let unit_of = |id: &str, unit: &'static str| if id == "T09" { "MW" } else { unit };
        rows.push(ResultRow {
            id: format!("TERM_{}", h.id),
            group: "conditions",
            parameter: format!(
                "{}, anchored value, {}, Headline uses {}, Modelled uses {}",
                h.name,
                gate_text(h),
                source_word(h),
                source_word(m)
            ),
            b: Some(h.anchored),
            a_inl: None,
            a_mea: None,
            unit: h.unit,
            equation: "",
            decision: "",
        });
        if let Some(v) = h.modelled {
            rows.push(ResultRow {
                id: format!("TERM_{}_MODELLED", h.id),
                group: "conditions",
                parameter: format!(
                    "{}, modelled: {}; {}",
                    h.name,
                    h.modelled_note,
                    gate_text(h)
                ),
                b: Some(v),
                a_inl: None,
                a_mea: None,
                unit: unit_of(h.id, h.unit),
                equation: "",
                decision: "",
            });
        }
        for (suffix, t, name) in [
            ("USED_HEADLINE", h, "Headline"),
            ("USED_MODELLED", m, "Modelled"),
        ] {
            rows.push(ResultRow {
                id: format!("TERM_{}_{suffix}", h.id),
                group: "conditions",
                parameter: format!(
                    "{}, value the {name} preset uses ({})",
                    t.name,
                    source_word(t)
                ),
                b: Some(t.used_value()),
                a_inl: None,
                a_mea: None,
                unit: if t.used == Source::Modelled {
                    unit_of(t.id, t.unit)
                } else {
                    t.unit
                },
                equation: "",
                decision: "",
            });
        }
    }
}

/// The values `nush2 drift` prints for the section 7 checks, as rows: Case B
/// REC_F on each preset and the high corner on Headline. Rows of the Modelled
/// and Headline presets carry that preset's status word.
fn drift_block(p: &ParamSet, rec: &TermRecord, rows: &mut Vec<ResultRow>) -> Result<()> {
    let mut add = |id: String, name: String, v: f64, unit: &'static str, eq: &'static str| {
        rows.push(ResultRow {
            id,
            group: "drift",
            parameter: name,
            b: Some(v),
            a_inl: None,
            a_mea: None,
            unit,
            equation: eq,
            decision: "",
        });
    };
    for (tag, name, reg) in [
        ("ANCHORED", "Anchored", p),
        ("HEADLINE", "Headline", &rec.p_head),
        ("MODELLED", "Modelled", &rec.p_mod),
    ] {
        let pt = drift::point(reg)?;
        for (what, label, v, unit, eq) in [
            ("R_MWTH", "reactor need", pt.r_mwth, "MWth", "J.26"),
            (
                "SPARE_MWTH",
                "spare against 210 MWth",
                pt.spare_mwth,
                "MWth",
                "J.36",
            ),
            (
                "ABATEMENT_T_Y",
                "abatement",
                pt.abatement_t_y,
                "t/y",
                "K.11",
            ),
            ("GAP_MW", "reboiler gap", pt.gap_mw, "MW", "J.18"),
            ("E_TOTAL_MWE", "electricity", pt.e_total_mwe, "MWe", "J.24"),
        ] {
            add(
                format!("DRIFT_{tag}_{what}"),
                format!("Drift check, {name} preset, Case B REC_F {label}"),
                v,
                unit,
                eq,
            );
        }
    }
    let hc = drift::high_corner(&rec.p_head)?;
    for (what, label, v, unit) in [
        (
            "R_HIGH_MWTH",
            "reactor need at the high corner",
            hc.r_mwth,
            "MWth",
        ),
        (
            "R_HIGH_OVER_RATING_MWTH",
            "high-corner need less the 210 MWth rating",
            hc.r_mwth - hc.rated_mwth,
            "MWth",
        ),
        (
            "R_HIGH_UNMET_HEAT_MWTH",
            "unmet heat at the high corner with the reactor held at 210 MWth",
            hc.unmet_heat_mwth,
            "MWth",
        ),
        (
            "R_HIGH_UNMET_E_MWE",
            "electricity import at the high corner with the reactor held at 210 MWth",
            hc.unmet_e_mwe,
            "MWe",
        ),
    ] {
        add(
            format!("DRIFT_HEADLINE_{what}"),
            format!("Drift check, Headline preset, Case B REC_F {label}"),
            v,
            unit,
            "J.27",
        );
    }
    Ok(())
}

/// Status of the rows of the M5 Case B tube model.
const TUBE_STATUS: &str = "model_m5";

/// The reformer exit the assumed tube reaches with helium at its 900 C supply,
/// at the HA tube count and at the upper limit of the tube-count search, and
/// the shortfall below INL's 871 C at each. The tube cannot reach 871 C within
/// the search, so these rows carry the finding (decision D29).
fn tube_block(rows: &mut Vec<ResultRow>) -> Result<()> {
    use nush2_units::tube_case_b as tube;
    let activity = nush2_units::jaeri::A2_FIT_ACTIVITY;
    let he_c = tube::HE_T_IN_K - 273.15;
    for (n, what) in [
        (
            tube::HA_TUBES,
            "the tube count of Hoseinzade and Adams 2017 Table 6",
        ),
        (
            tube::N_TUBES_SEARCH_MAX,
            "the upper limit of the tube-count search",
        ),
    ] {
        let (point, _) = tube::run(n, activity)?;
        let tag = format!("N{n:.0}");
        let mut add = |id: String, parameter: String, v: f64, unit: &'static str| {
            rows.push(ResultRow {
                id,
                group: "heat",
                parameter,
                b: Some(v),
                a_inl: None,
                a_mea: None,
                unit,
                equation: "D.25",
                decision: "D29",
            });
        };
        add(
            format!("TUBE_EXIT_C_{tag}"),
            format!(
                "Reformer process-gas exit reachable on the assumed tube at {n:.0} tubes, {what}, \
                 helium supply {he_c:.0} C, Case B feed; tube model run tube_case_b at JAERI A-2 \
                 activity {activity:.3}"
            ),
            point.exit_c,
            "C",
        );
        add(
            format!("TUBE_SHORTFALL_{:.0}C_K_{tag}", REFORMER_INL_C),
            format!(
                "Shortfall of that exit below INL's {REFORMER_INL_C:.0} C reformer exit (TEV-961 \
                 PDF p14), {n:.0} tubes"
            ),
            REFORMER_INL_C - point.exit_c,
            "K",
        );
    }
    Ok(())
}

/// Duties of the capture island of the M6 column model at its Case B design
/// point, the run the live reply shows beside the anchored reboiler. Its
/// reboiler runs at the M6 specific duty (TERM_T06_MODELLED), not the anchored
/// one, so these duties sit beside their own reboiler row and not beside
/// MEA_REGEN. The lean cooler has no card of its own.
fn island_block(rows: &mut Vec<ResultRow>) -> Result<()> {
    let d = crate::terms::design_duties_cached()?;
    for (id, what, v, eq) in [
        ("REBOILER_MW", "reboiler duty", d.reboiler_mw, "I.6"),
        (
            "CONDENSER_MW",
            "stripper condenser duty",
            d.condenser_mw,
            "I.7",
        ),
        ("LEAN_COOLER_MW", "lean cooler duty", d.lean_cooler_mw, ""),
        (
            "RICH_LEAN_MW",
            "rich/lean exchanger duty",
            d.rich_lean_mw,
            "I.10",
        ),
    ] {
        rows.push(ResultRow {
            id: format!("M6_ISLAND_{id}"),
            group: "heat",
            parameter: format!(
                "Capture island {what}, M6 column model at its Case B design point; the \
                 Headline preset keeps the columns anchored because the M6 gate failed"
            ),
            b: Some(v),
            a_inl: None,
            a_mea: None,
            unit: "MW",
            equation: eq,
            decision: "",
        });
    }
    Ok(())
}

/// INL's equilibrium reformer exit, C, the headline basis (decision D29).
const REFORMER_INL_C: f64 = flowsheet::T_REFORMER_OUT_C;

fn level4_block(rec: &TermRecord, rows: &mut Vec<ResultRow>) {
    let mut add = |id: &str, name: &str, v: f64, unit: &'static str, eq: &'static str| {
        if v.is_finite() {
            rows.push(ResultRow {
                id: format!("L4_{id}"),
                group: "nuclear",
                parameter: name.to_string(),
                b: Some(v),
                a_inl: None,
                a_mea: None,
                unit,
                equation: eq,
                decision: "",
            });
        }
    };
    if let Ok(l) = &rec.level4 {
        add(
            "IHX_DUTY_MW",
            "IHX duty, process heat with slip heat, Headline",
            l.q_duty_mw,
            "MW",
            "C.2",
        );
        add(
            "HE_SEC_FLOW_KG_S",
            "Secondary helium from the duty",
            l.he_sec_kg_s,
            "kg/s",
            "B.9",
        );
        add(
            "HE_SEC_FLOW_PORT_KG_S",
            "Secondary helium as the anchored port scales the INL flow",
            l.he_sec_port_kg_s,
            "kg/s",
            "B.8",
        );
        add(
            "HE_PRIM_FLOW_KG_S",
            "Primary helium for the reactor need",
            l.he_prim_kg_s,
            "kg/s",
            "B.9",
        );
        add(
            "W_CIRC_SEC_MW",
            "Secondary circulator shaft power on the B.9 flow",
            l.w_sec_mw,
            "MW",
            "B.10",
        );
        add(
            "W_CIRC_PRIM_MW",
            "Primary circulator shaft power on the B.9 flow",
            l.w_prim_mw,
            "MW",
            "B.10",
        );
        add(
            "E_CIRC_MWE",
            "Circulator electric draw on the B.9 flows",
            l.e_circ_mwe,
            "MWe",
            "B.12",
        );
        add(
            "RISE_SEC_K",
            "Secondary helium rise across its circulator",
            l.rise_sec_k,
            "K",
            "B.6",
        );
        add(
            "RISE_PRIM_K",
            "Primary helium rise across its circulator",
            l.rise_prim_k,
            "K",
            "B.6",
        );
        add(
            "DUCT_LOSS_B14_MW",
            "Duct loss on pipe diameter, card B.14",
            l.duct_b14_mw,
            "MW",
            "B.14",
        );
        add(
            "DUCT_LOSS_PORT_MW",
            "Duct loss as the anchored port scales it, linear in plant size",
            l.duct_port_mw,
            "MW",
            "B.14",
        );
        add(
            "IHX_LMTD_K",
            "IHX log mean temperature difference",
            l.ihx.lmtd_k,
            "K",
            "C.7",
        );
        add(
            "IHX_UA_MW_K",
            "IHX conductance for the duty",
            l.ihx.ua_mw_k,
            "MW/K",
            "C.13",
        );
        add(
            "IHX_NTU",
            "IHX number of transfer units",
            l.ihx.ntu,
            "-",
            "C.10",
        );
        add(
            "IHX_EFFECTIVENESS",
            "IHX effectiveness",
            l.ihx.eps,
            "-",
            "C.9",
        );
        for (u, a) in &l.ihx.area_m2 {
            let id = format!("IHX_AREA_U{u:.0}_M2");
            let name = format!("IHX area at U = {u:.0} W/m2/K");
            add(&id, &name, *a, "m2", "C.13");
        }
        add(
            "IHX_MODULES_HTTR",
            "HTTR-size IHX modules for the conductance (UA over 85 kW/K)",
            l.ihx.modules_httr,
            "-",
            "C.14",
        );
        add(
            "IHX_PRIMARY_OUT_C",
            "Primary helium leaving the IHX",
            l.ihx.primary_out_c,
            "C",
            "B.7",
        );
        add(
            "SUPPLY_OVER_REFORMER_K",
            "Secondary supply less reformer exit, at least 10 K",
            l.limits.supply_over_reformer_k,
            "K",
            "C.18",
        );
        for d in std::iter::once(&l.t10).chain(&l.t10_alt) {
            let n = d.smr_tubes;
            add(
                &format!("T10_SMR_KPA_N{n:.0}"),
                &format!(
                    "Reformer helium annulus drop at {n:.0} Hoseinzade and Adams 2017 tubes, \
                     smooth wall, fins left out"
                ),
                d.smr_kpa,
                "kPa",
                "C.16",
            );
            add(
                &format!("T10_SMR_RE_N{n:.0}"),
                &format!("Reformer annulus Reynolds number at {n:.0} tubes"),
                d.smr_re,
                "-",
                "C.16",
            );
            add(
                &format!("T10_SMR_HE_OUT_C_N{n:.0}"),
                &format!("Helium leaving the reformer tube at {n:.0} tubes"),
                d.smr_he_out_c,
                "C",
                "B.8",
            );
            add(
                &format!("T10_TOTAL_KPA_N{n:.0}"),
                &format!("Secondary loop drop, reformer plus IHX tubes, at {n:.0} reformer tubes"),
                d.total_kpa,
                "kPa",
                "C.16",
            );
        }
        add(
            "T10_IHX_KPA",
            "IHX tube-side drop, HTTR tubes, straight-tube Blasius",
            l.t10.ihx_kpa,
            "kPa",
            "C.16",
        );
        add(
            "T10_IHX_RE",
            "IHX tube Reynolds number",
            l.t10.ihx_re,
            "-",
            "C.16",
        );
        add(
            "T10_IHX_TUBES",
            "IHX tubes, HTTR-size modules times 96",
            l.t10.ihx_tubes,
            "-",
            "C.14",
        );
    }
    if let Some(p) = &rec.pinch {
        add(
            "REBOILER_T_C",
            "Reboiler temperature of the M6 stripper model",
            p.t_reb_c,
            "C",
            "I.17",
        );
        add(
            "PINCH_APPROACH_K",
            "Smallest approach of a usable reboiler source, at the syngas cut",
            p.pinch_approach_k,
            "K",
            "J.30",
        );
        for g in &p.grades {
            add(
                &format!("GRADE_{}_C", g.source),
                &format!(
                    "Coldest temperature of the {} source",
                    g.source.to_lowercase()
                ),
                g.t_c,
                "C",
                "I.17",
            );
            add(
                &format!("GRADE_{}_APPROACH_K", g.source),
                &format!(
                    "{} source temperature less the reboiler",
                    g.source.to_lowercase()
                ),
                g.approach_k,
                "K",
                "I.17",
            );
            add(
                &format!("GRADE_{}_USABLE", g.source),
                &format!(
                    "1 when the {} source is above the reboiler",
                    g.source.to_lowercase()
                ),
                if g.usable { 1.0 } else { 0.0 },
                "flag",
                "J.30",
            );
        }
    }
    if let Some(v) = rec.syngas_flowsheet_mw {
        add(
            "SYNGAS_ABOVE_CUT_FLOWSHEET_MW",
            "Syngas heat above 130 C on the Level 3 EX3 outlet at Case B REC_F",
            v,
            "MW",
            "F.11",
        );
    }
    if let Some(v) = rec.phi_flowsheet {
        add(
            "PHI_FLOWSHEET",
            "Carbon split after the shift in the Level 3 flowsheet at Case B REC_F",
            v,
            "-",
            "J.6",
        );
    }
}

/// Mass balance header, with the four C2+ species beside their
/// sum and the liquid part of the water, so the flowsheet rows carry every
/// species.
const MASS_HEADER: [&str; 28] = [
    "run_id",
    "stream_id",
    "stream_name",
    "from_unit",
    "to_unit",
    "crosses_boundary",
    "t_c",
    "p_bar",
    "total_kg_h",
    "total_kmol_h",
    "h2",
    "co",
    "co2",
    "ch4",
    "c2plus",
    "c2h6",
    "c3h8",
    "nc4h10",
    "nc5h12",
    "n2",
    "h2o",
    "h2o_liquid",
    "mea",
    "carbon_kmol_h",
    "class",
    "rigour",
    "source",
    "status",
];

fn mass_balance(
    m: &Model,
    modelled: Option<(&Model, &Model, &str)>,
    fs: &Result<Suite>,
) -> Result<String> {
    let mut c = Csv::new(&MASS_HEADER);
    for r in RUNS {
        anchored_mass_run(&mut c, m, &r, r.id, fs)?;
    }
    // Case B REC_F on the Modelled preset, so its carbon ledger can be checked
    // from the table. A row that differs from the Anchored preset carries the
    // Modelled status word.
    if let Some((mm, m0, word)) = modelled {
        let b_rec = run_def("B_REC_F")?;
        let mut on_mod = Csv::new(&MASS_HEADER);
        anchored_mass_run(&mut on_mod, mm, &b_rec, "B_REC_F_MODELLED", fs)?;
        let mut on_anch = Csv::new(&MASS_HEADER);
        anchored_mass_run(&mut on_anch, m0, &b_rec, "B_REC_F_MODELLED", fs)?;
        let marked = mark(on_mod.out, &on_anch.out, Some(word))?;
        c.out
            .push_str(marked.split_once('\n').map_or("", |(_, rest)| rest));
    }
    match fs {
        Ok(suite) => {
            for run in &suite.runs {
                flowsheet_rows(&mut c, run)?;
            }
        }
        Err(e) => {
            for id in flowsheet::SUITE_IDS {
                let mut cells = vec![String::new(); MASS_HEADER.len()];
                cells[0] = t(id);
                cells[1] = t("NOT_CONVERGED");
                cells[2] = t("Level 3 flowsheet");
                cells[5] = t("false");
                cells[24] = t("calculated");
                cells[25] = t("rigorous");
                cells[26] = e.to_string();
                cells[27] = t("not_converged");
                c.row(&cells);
            }
        }
    }
    Ok(c.out)
}

/// Anchored-port stream rows of one fixed run, written under `run_id`.
fn anchored_mass_run(
    c: &mut Csv,
    m: &Model,
    r: &RunDef,
    run_id: &str,
    fs: &Result<Suite>,
) -> Result<()> {
    struct S {
        id: &'static str,
        name: &'static str,
        from: &'static str,
        to: &'static str,
        kg_h: Option<f64>,
        kmol_h: Option<f64>,
        h2: Option<f64>,
        co2: Option<f64>,
        ch4: Option<f64>,
        carbon: Option<f64>,
        rigour: &'static str,
    }
    let r = *r;
    let s = m.scale(r.case);
    let b = balance_for(m, &r)?;
    let k = b.c;
    let h2_kmol = m.h2_a_kgh * s * k.h_scale / MW_H2;
    let h2_feed = h2_kmol / m.psa_rec;
    let cap = k.cap * s;
    // Only the recycle runs hold a cryogenic unit. Where the reject is burned or
    // exported it leaves the PSA once, by that route, and nothing returns.
    let cryo = matches!(r.scn, Scenario::InlSel | Scenario::RecF | Scenario::RecFix);
    let ret = fs.as_ref().ok().map(|f| f.closure.h2_return);
    let h2_ret = if cryo {
        ret.map(|r| r * (h2_feed - h2_kmol))
    } else {
        Some(0.0)
    };
    let h2_reject = h2_feed - h2_kmol;
    let mw_ch4 = nush2_props::species::Sp::Ch4.mw_kg_mol() * 1000.0;
    let fs_run = matched_run(fs, &r);
    let streams = [
        S {
            id: "H2_PRODUCT",
            name: "Purified H2 after external PSA",
            from: "PSA",
            to: "PRODUCT",
            kg_h: Some(h2_kmol * MW_H2 + k.p * s * mw_ch4),
            kmol_h: Some(h2_kmol + k.p * s),
            h2: Some(h2_kmol),
            co2: None,
            ch4: Some(k.p * s),
            carbon: Some(k.p * s),
            rigour: "external_boundary",
        },
        S {
            id: "CO2_TO_STORAGE",
            name: "Compressed CO2 to storage",
            from: "CO2_COMPRESSOR",
            to: "STORAGE",
            kg_h: Some(kmol_h_to_t_h(cap) * 1000.0),
            kmol_h: Some(cap),
            h2: None,
            co2: Some(cap),
            ch4: None,
            carbon: Some(cap),
            rigour: "reduced_order",
        },
        S {
            id: "N2_RICH_CUT",
            name: if cryo {
                "Burned nitrogen-rich cut"
            } else {
                "PSA reject burned on site"
            },
            from: if cryo { "CRYO" } else { "PSA" },
            to: "STACK",
            kg_h: None,
            kmol_h: None,
            // The cryogenic split of H2 is not part of the anchored ledger.
            h2: match r.scn {
                Scenario::CombH | Scenario::CombN => Some(h2_reject),
                Scenario::Export => Some(0.0),
                _ => None,
            },
            co2: None,
            ch4: None,
            carbon: Some(k.burn * s),
            rigour: "reduced_order",
        },
        S {
            id: "NG_FEED",
            name: "Fresh natural gas",
            from: "BATTERY_LIMIT",
            to: "REFORMER",
            kg_h: None,
            kmol_h: Some(mmscfd_to_kmol_h(k.ng) * s),
            h2: None,
            co2: None,
            ch4: None,
            carbon: Some(k.f * s),
            rigour: "fixed_assumption",
        },
        S {
            id: "CRYO_H2_RETURN",
            name: "Cryogenic H2 return ahead of the PSA, share calculated by closure in the \
                   Level 3 flowsheet",
            from: "CRYO",
            to: "PSA",
            kg_h: h2_ret.map(|h| h * MW_H2),
            kmol_h: h2_ret,
            h2: h2_ret,
            co2: None,
            ch4: None,
            carbon: Some(0.0),
            rigour: "fixed_assumption",
        },
        S {
            id: "PSA_FEED",
            name: if cryo {
                "H2-rich gas into the PSA from the capture outlet; the cryogenic return \
                 enters as CRYO_H2_RETURN"
            } else {
                "H2-rich gas into the PSA from the capture outlet"
            },
            from: "CAPTURE",
            to: "PSA",
            kg_h: None,
            kmol_h: None,
            h2: h2_ret.map(|r| h2_feed - r),
            co2: None,
            ch4: None,
            carbon: Some((k.g + k.p) * s),
            rigour: "external_boundary",
        },
        S {
            id: "PSA_REJECT_RETURN",
            name: if cryo {
                "PSA reject returning across the boundary"
            } else {
                "PSA reject returning across the boundary, none in this run"
            },
            from: "PSA",
            to: "CRYO",
            kg_h: None,
            kmol_h: None,
            h2: Some(if cryo { h2_reject } else { 0.0 }),
            co2: None,
            ch4: None,
            carbon: Some(if cryo { k.g * s } else { 0.0 }),
            rigour: "external_boundary",
        },
        S {
            id: "RECYCLE_TO_REFORMER",
            name: "Recycled methane-rich gas",
            from: "CRYO",
            to: "REFORMER",
            kg_h: None,
            kmol_h: None,
            h2: None,
            co2: None,
            ch4: None,
            carbon: Some(k.r * s),
            rigour: "reduced_order",
        },
        S {
            id: "TAIL_GAS_EXPORT",
            name: "Tail gas leaving the boundary",
            from: "PSA",
            to: "OFFSITE",
            kg_h: None,
            kmol_h: None,
            h2: Some(if r.scn == Scenario::Export {
                h2_reject
            } else {
                0.0
            }),
            co2: None,
            ch4: None,
            carbon: Some(k.exported * s),
            rigour: "external_boundary",
        },
    ];
    for x in streams {
        c.row(&[
            t(run_id),
            t(x.id),
            t(x.name),
            t(x.from),
            t(x.to),
            flowsheet::crosses(x.from, x.to).to_string(),
            String::new(),
            String::new(),
            opt(x.kg_h),
            opt(x.kmol_h),
            opt(x.h2),
            String::new(),
            opt(x.co2),
            opt(x.ch4),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            opt(x.carbon),
            t("calculated"),
            t(x.rigour),
            t("engine anchored port, carbon ledger J.8 to J.11"),
            t(STATUS),
        ]);
    }
    let env = fs_run
        .map(|run| -> Result<(f64, f64)> {
            let (ein, eout) = flowsheet::envelope_elements(run)?;
            let (min, mout) = flowsheet::envelope_mass(run)?;
            Ok(((ein[1] - eout[1]) * 3.6, (min - mout) * 3600.0))
        })
        .transpose()?;
    let env_source = fs_run.map_or(String::new(), |run| {
        format!(
            "Level 3 flowsheet {} envelope; the anchored ledger tracks carbon only",
            run.id
        )
    });
    let env_status = if env.is_some() {
        FLOWSHEET_STATUS
    } else {
        "not_run"
    };
    for (id, name, value, status, col) in [
        (
            "CLOSURE_CARBON",
            "Carbon residual",
            Some(k.closure * s),
            STATUS,
            23,
        ),
        (
            "CLOSURE_H",
            "Hydrogen residual, kmol H/h in total_kmol_h",
            env.map(|e| e.0),
            env_status,
            9,
        ),
        (
            "CLOSURE_MASS",
            "Mass residual, kg/h in total_kg_h",
            env.map(|e| e.1),
            env_status,
            8,
        ),
    ] {
        let mut cells = vec![String::new(); MASS_HEADER.len()];
        cells[0] = t(run_id);
        cells[1] = t(id);
        cells[2] = t(name);
        cells[5] = t("false");
        if let Some(v) = value {
            cells[col] = if col == 23 { num(v) } else { sci6(v) };
        }
        cells[24] = t("calculated");
        cells[25] = t("rigorous");
        cells[26] = if col == 23 {
            t("element closure, full species in the FS_ runs")
        } else if value.is_some() {
            env_source.clone()
        } else {
            t("no Level 3 flowsheet for this run")
        };
        cells[27] = t(status);
        c.row(&cells);
    }
    Ok(())
}

/// Level 3 run that matches a fixed run, when one exists.
fn matched_run<'a>(fs: &'a Result<Suite>, r: &RunDef) -> Option<&'a FlowsheetRun> {
    let id = match (r.case, r.scn) {
        (Case::A, Scenario::InlSel) => "FS_A_INL_SEL",
        (Case::A, Scenario::RecF) => "FS_A_REC_F",
        (Case::B, Scenario::RecF) => "FS_B_REC_F",
        _ => return None,
    };
    fs.as_ref().ok().and_then(|f| f.run(id))
}

fn rigour_of(class: ModelClass) -> &'static str {
    match class {
        ModelClass::Rigorous => "rigorous",
        ModelClass::ReducedOrder => "reduced_order",
        ModelClass::Empirical => "empirical",
        ModelClass::FixedAssumption => "fixed_assumption",
        ModelClass::ExternalBoundary => "external_boundary",
    }
}

fn sci6(v: f64) -> String {
    format!("{v:.6e}")
}

/// Stream rows and closure rows of one flowsheet run.
fn flowsheet_rows(c: &mut Csv, run: &FlowsheetRun) -> Result<()> {
    let from_of = |tag: &str| -> (String, Option<ModelClass>) {
        if let Some(u) = run
            .flowsheet
            .units
            .iter()
            .find(|u| u.outlets.iter().any(|o| o == tag))
        {
            return (u.tag.clone(), Some(u.class));
        }
        let b = run.boundary.iter().find(|b| b.stream == tag);
        (b.and_then(|b| b.from).unwrap_or("").to_string(), None)
    };
    let to_of = |tag: &str| -> String {
        if let Some(u) = run
            .flowsheet
            .units
            .iter()
            .find(|u| u.inlets.iter().any(|o| o == tag))
        {
            return u.tag.clone();
        }
        run.boundary
            .iter()
            .find(|b| b.stream == tag)
            .and_then(|b| b.to)
            .unwrap_or("")
            .to_string()
    };
    for (tag, port) in &run.flowsheet.streams {
        let (from, class) = from_of(tag);
        let rigour = if matches!(tag.as_str(), "S12" | "S13" | "S14" | "S21") {
            "external_boundary"
        } else {
            class.map(rigour_of).unwrap_or("fixed_assumption")
        };
        let mut cells = vec![String::new(); MASS_HEADER.len()];
        cells[0] = run.id.clone();
        cells[1] = tag.clone();
        cells[2] = t(flowsheet::stream_name(tag));
        cells[3] = from.clone();
        cells[4] = to_of(tag);
        cells[5] = flowsheet::crosses_boundary(run, tag).to_string();
        match port {
            Port::Gas(s) => {
                let k = |sp: Sp| s.n[sp as usize] * 3.6;
                cells[6] = num(s.t_k - 273.15);
                cells[7] = num(s.p_pa / 1.0e5);
                cells[8] = num(s.mass_kg_s()? * 3600.0);
                cells[9] = num(s.total() * 3.6);
                cells[10] = num(k(Sp::H2));
                cells[11] = num(k(Sp::Co));
                cells[12] = num(k(Sp::Co2));
                cells[13] = num(k(Sp::Ch4));
                cells[14] = num(k(Sp::C2h6) + k(Sp::C3h8) + k(Sp::Nc4h10) + k(Sp::Nc5h12));
                cells[15] = num(k(Sp::C2h6));
                cells[16] = num(k(Sp::C3h8));
                cells[17] = num(k(Sp::Nc4h10));
                cells[18] = num(k(Sp::Nc5h12));
                cells[19] = num(k(Sp::N2));
                cells[20] = num(k(Sp::H2o));
                cells[21] = num(s.n_liq_h2o * 3.6);
                cells[23] = num(carbon_kmol_h(s));
            }
            Port::Solvent(s) => {
                cells[6] = num(s.t_k - 273.15);
                cells[7] = num(s.p_pa / 1.0e5);
                cells[8] = num((s.n_mea * MW_MEA + s.n_h2o * MW_H2O + s.n_co2 * MW_CO2) * 3.6);
                cells[9] = num((s.n_mea + s.n_h2o + s.n_co2) * 3.6);
                cells[12] = num(s.n_co2 * 3.6);
                cells[20] = num(s.n_h2o * 3.6);
                cells[21] = num(s.n_h2o * 3.6);
                cells[22] = num(s.n_mea * 3.6);
                // CO2 carbon only; the MEA carbon never leaves the loop.
                cells[23] = num(s.n_co2 * 3.6);
            }
            Port::He(_) => {}
        }
        cells[24] = t("calculated");
        cells[25] = t(rigour);
        cells[26] = match port {
            // The purified H2 holds the product CH4, so its total is above the
            // H2 rate the anchored balance carries as the headline.
            Port::Gas(s) if tag == "S13" => format!(
                "engine Level 3 flowsheet, from {from}; the total holds the product CH4 at \
                 0.1 mol% (TEV-953 A-40, TEV-961 A-50), so it exceeds the headline H2 of the \
                 anchored balance, which counts H2 alone; H2 alone here is {:.1} kg/h, set equal \
                 to the anchored H2 rate when the run is on a hydrogen basis",
                s.n[Sp::H2 as usize] * 3.6 * MW_H2
            ),
            _ => format!("engine Level 3 flowsheet, from {from}"),
        };
        cells[27] = t(FLOWSHEET_STATUS);
        c.row(&cells);
    }
    // Combustion O2 is no stream species, so it is written here as its own row,
    // so that the envelope closes from the table alone.
    let mut o2 = vec![String::new(); MASS_HEADER.len()];
    o2[0] = run.id.clone();
    o2[1] = t("COMBUSTION_O2");
    o2[2] = t(
        "Stoichiometric O2 to the N2-rich cut burner, O2 only, air N2 and excess air not \
         tracked; kmol O2/h in total_kmol_h",
    );
    o2[3] = t("BATTERY_LIMIT");
    o2[4] = t("N2-BURN");
    o2[5] = flowsheet::crosses("BATTERY_LIMIT", "N2-BURN").to_string();
    o2[8] = num(run.air_o2_mol_s * flowsheet::MW_O2_KG_MOL * 3600.0);
    o2[9] = num(run.air_o2_mol_s * 3.6);
    o2[23] = num(0.0);
    o2[24] = t("calculated");
    o2[25] = t("rigorous");
    o2[26] = t("engine Level 3 flowsheet, N2-BURN oxygen demand");
    o2[27] = t(FLOWSHEET_STATUS);
    c.row(&o2);
    let (ein, eout) = flowsheet::envelope_elements(run)?;
    let (min, mout) = flowsheet::envelope_mass(run)?;
    for (id, name, col, v) in [
        (
            "CLOSURE_CARBON",
            "Carbon residual of the envelope, kmol C/h",
            23,
            (ein[0] - eout[0]) * 3.6,
        ),
        (
            "CLOSURE_H",
            "Hydrogen residual of the envelope, kmol H/h in total_kmol_h",
            9,
            (ein[1] - eout[1]) * 3.6,
        ),
        (
            "CLOSURE_MASS",
            "Mass residual of the envelope, kg/h in total_kg_h",
            8,
            (min - mout) * 3600.0,
        ),
        (
            "CLOSURE_N",
            "Nitrogen residual of the envelope, kmol N/h in total_kmol_h",
            9,
            (ein[3] - eout[3]) * 3.6,
        ),
        (
            "CLOSURE_O",
            "Oxygen residual of the envelope, kmol O/h in total_kmol_h",
            9,
            (ein[2] - eout[2]) * 3.6,
        ),
    ] {
        let mut cells = vec![String::new(); MASS_HEADER.len()];
        cells[0] = run.id.clone();
        cells[1] = t(id);
        cells[2] = t(name);
        cells[5] = t("false");
        cells[col] = sci6(v);
        cells[24] = t("calculated");
        cells[25] = t("rigorous");
        cells[26] = t("envelope of every unit inside the boundary, cards A.2 and A.5");
        cells[27] = t(FLOWSHEET_STATUS);
        c.row(&cells);
    }
    Ok(())
}

/// One row of the energy balance before it is written.
struct ERow {
    id: &'static str,
    name: &'static str,
    side: &'static str,
    kind: &'static str,
    value: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
    temperature: Option<f64>,
    rigour: &'static str,
    source: String,
    equation: &'static str,
    status: &'static str,
}

impl ERow {
    #[allow(clippy::too_many_arguments)]
    fn port(
        id: &'static str,
        name: &'static str,
        side: &'static str,
        kind: &'static str,
        value: Option<f64>,
        temperature: Option<f64>,
        equation: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            side,
            kind,
            value,
            low: None,
            high: None,
            temperature,
            rigour: "reduced_order",
            source: "engine anchored port".to_string(),
            equation,
            status: if value.is_some() { STATUS } else { "not_run" },
        }
    }
}

/// Closure group of an energy row of a fixed run. In `reactor`, `reboiler` and
/// `electricity` the sources equal the sinks; the rows of `part:POWER_BRANCH`
/// add up to that row, and BRANCH_EXTRACTION_WORK sits inside POWER_GENERATED
/// as well; `memo` rows repeat or annotate a value held in a group and enter
/// no sum. Every row of a Level 3 run is `memo`.
pub fn closure_group(run_id: &str, term_id: &str) -> &'static str {
    if run_id.starts_with("FS_") {
        return "memo";
    }
    match term_id {
        "HTGR_HEAT" | "CIRC_SHAFT_HEAT" | "IHX_DUTY" | "IHX_LOSS" | "DUCT_LOSS" | "VESSEL_LOSS"
        | "POWER_BRANCH" => "reactor",
        "BRANCH_TO_REBOILER" | "BRANCH_EXTRACTION_WORK" | "BRANCH_TO_TURBINE" => {
            "part:POWER_BRANCH"
        }
        "MEA_REGEN" | "REB_FROM_SELEXOL" | "REB_FROM_EXPORT" | "REB_FROM_SYNGAS"
        | "REB_FROM_ZLD" | "REB_FROM_TAIL_GAS" | "REB_FROM_REACTOR" => "reboiler",
        "POWER_GENERATED" | "TAIL_GAS_POWER" | "HE_CIRCULATION" | "PUMP_COMPRESS" | "COOLING"
        | "REFORMING_POWER" | "PURIFICATION_POWER" | "STEAM_SYSTEM_POWER" | "WATER_POWER" => {
            "electricity"
        }
        _ => "memo",
    }
}

fn write_erows(c: &mut Csv, run_id: &str, rows: Vec<ERow>) {
    for r in rows {
        c.row(&[
            t(run_id),
            t(r.id),
            t(r.name),
            t(r.side),
            t(r.kind),
            opt(r.value),
            opt(r.low),
            opt(r.high),
            opt(r.temperature),
            t("calculated"),
            t(r.rigour),
            r.source,
            t(r.equation),
            t(closure_group(run_id, r.id)),
            t(r.status),
        ]);
    }
}

/// The power branch of card B.19 in its three parts, MW: steam to the
/// reboiler, back-pressure work on that steam, and heat to the condensing
/// turbine. The parts add up to the branch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BranchSplit {
    pub to_reboiler: f64,
    pub extraction_work: f64,
    pub to_turbine: f64,
}

/// Split the branch of balance `b` on the options its run used.
///
/// eq: B.19
pub fn branch_split(m: &Model, b: &Balance) -> BranchSplit {
    let o = m.opts(b.s);
    let wx = match o.route {
        Route::Extraction => b.e_nuc.min(o.r_work * b.gap_after_tg),
        Route::Direct => 0.0,
    };
    BranchSplit {
        to_reboiler: b.gap_after_tg,
        extraction_work: wx,
        to_turbine: b.branch - b.gap_after_tg - wx,
    }
}

/// Shift heat release of the HTS and LTS beds of a Level 3 run, MW.
pub fn wgs_release_mw(run: &FlowsheetRun) -> Result<f64> {
    let hts = heatrec::wgs_release_w(run.gas("S06")?, run.gas("S07")?)?;
    let lts = heatrec::wgs_release_w(run.gas("S08")?, run.gas("S09")?)?;
    Ok((hts + lts) / 1.0e6)
}

/// Rows of one fixed run on one preset. The reactor heat budget closes on the
/// rows alone: HTGR_HEAT plus CIRC_SHAFT_HEAT equals IHX_DUTY, IHX_LOSS,
/// DUCT_LOSS, VESSEL_LOSS and POWER_BRANCH.
fn run_rows(m: &Model, r: &RunDef, fs_run: Option<&FlowsheetRun>) -> Result<Vec<ERow>> {
    let s = m.scale(r.case);
    let b = balance_for(m, r)?;
    let sum = m.case_summary(r.scn, s)?;
    let e = &b.e_rows;
    let (short, _) = m.shortfall_at(m.r_rated, &b);
    let alloc = heatrec::allocate(b.q_reb, &b.stack);
    let from_tg = (b.gap - b.gap_after_tg).max(0.0);
    let split = branch_split(m, &b);
    let syngas_t = T05_LIMIT_K - 273.15;
    let mut rows = vec![
        ERow {
            low: sum.get("R_best").copied(),
            high: sum.get("R_high").copied(),
            ..ERow::port(
                "HTGR_HEAT",
                "Reactor heat at the need",
                "source",
                "heat",
                Some(b.r),
                Some(m.rot),
                "J.26",
            )
        },
        ERow {
            low: sum.get("R_best").copied(),
            high: sum.get("R_high").copied(),
            ..ERow::port(
                "MIN_REACTOR_RATING",
                "Minimum reactor rating, central with best and high corners",
                "source",
                "heat",
                Some(b.r),
                None,
                "J.27",
            )
        },
        ERow::port(
            "CIRC_SHAFT_HEAT",
            "Circulator shaft work returned to the helium",
            "source",
            "heat",
            Some(b.w_s + b.w_p),
            None,
            "B.16",
        ),
        ERow::port(
            "IHX_DUTY",
            "IHX duty",
            "sink",
            "heat",
            Some(b.q_proc + b.q_recyc),
            Some(m.t_he_s),
            "C.1",
        ),
        ERow::port(
            "IHX_LOSS",
            "IHX loss",
            "sink",
            "heat",
            Some(b.q_ihx),
            None,
            "B.15",
        ),
        ERow::port(
            "DUCT_LOSS",
            "Hot duct loss",
            "sink",
            "heat",
            Some(b.q_duct),
            None,
            "B.14",
        ),
        ERow::port(
            "VESSEL_LOSS",
            "Reactor vessel loss",
            "sink",
            "heat",
            Some(b.qv),
            None,
            "B.13",
        ),
        ERow::port(
            "POWER_BRANCH",
            "Reactor heat to the power and steam branch",
            "sink",
            "heat",
            Some(b.branch),
            None,
            "B.19",
        ),
        ERow::port(
            "BRANCH_TO_REBOILER",
            "Power branch heat sent to the reboiler as steam",
            "sink",
            "heat",
            Some(split.to_reboiler),
            None,
            "B.19",
        ),
        ERow::port(
            "BRANCH_EXTRACTION_WORK",
            "Back-pressure work on the reboiler steam, a part of POWER_GENERATED",
            "source",
            "electricity",
            Some(split.extraction_work),
            None,
            "B.19",
        ),
        ERow::port(
            "BRANCH_TO_TURBINE",
            "Power branch heat to the condensing turbine",
            "sink",
            "heat",
            Some(split.to_turbine),
            None,
            "B.19",
        ),
        ERow::port(
            "SMR_HEAT",
            "Process heat with slip heat",
            "sink",
            "heat",
            Some(b.q_proc + b.q_recyc),
            None,
            "J.14",
        ),
    ];
    let wgs = match fs_run {
        Some(run) => ERow {
            rigour: "rigorous",
            source: format!(
                "Level 3 flowsheet {}, HTS and LTS beds at their inlet temperature",
                run.id
            ),
            status: LEVEL4_STATUS,
            ..ERow::port(
                "WGS_RELEASE",
                "Water-gas shift heat release",
                "source",
                "heat",
                Some(wgs_release_mw(run)?),
                None,
                "E.6",
            )
        },
        None => ERow {
            source: "no Level 3 flowsheet for this run".to_string(),
            ..ERow::port(
                "WGS_RELEASE",
                "Water-gas shift heat release",
                "source",
                "heat",
                None,
                None,
                "E.6",
            )
        },
    };
    rows.push(wgs);
    rows.extend([
        ERow::port(
            "SYNGAS_RECOVERY",
            "Syngas heat recovered above 130 C",
            "source",
            "heat",
            Some(b.stack.ex4),
            Some(m.ex4.t_out),
            "F.2",
        ),
        ERow::port(
            "MEA_REGEN",
            "Reboiler duty",
            "sink",
            "heat",
            Some(b.q_reb),
            None,
            "J.16",
        ),
        ERow::port(
            "STEAM_AVAILABLE",
            "Recovered heat stack before the reboiler draw",
            "source",
            "heat",
            Some(b.avail),
            None,
            "J.17",
        ),
    ]);
    for (src, id, name, temp) in [
        (
            SourceId::Selexol,
            "REB_FROM_SELEXOL",
            "Reboiler heat from freed Selexol steam, first",
            Some(m.t_lp_steam_c),
        ),
        (
            SourceId::Export,
            "REB_FROM_EXPORT",
            "Reboiler heat from export steam, second",
            Some(m.t_hp_steam_c),
        ),
        (
            SourceId::Syngas,
            "REB_FROM_SYNGAS",
            "Reboiler heat from syngas above 130 C, third",
            Some(syngas_t),
        ),
        (
            SourceId::Zld,
            "REB_FROM_ZLD",
            "Reboiler heat from ZLD steam, fourth, off by default",
            Some(m.t_lp_steam_c),
        ),
    ] {
        let mut row = ERow::port(
            id,
            name,
            "source",
            "heat",
            Some(alloc.from(src)),
            temp,
            "J.17",
        );
        row.source = "engine heat recovery, approved order".to_string();
        row.status = LEVEL4_STATUS;
        rows.push(row);
    }
    let mut tg = ERow::port(
        "REB_FROM_TAIL_GAS",
        "Reboiler heat from tail gas burned on site",
        "source",
        "heat",
        Some(from_tg),
        None,
        "J.18",
    );
    tg.source = "engine heat recovery, approved order".to_string();
    tg.status = LEVEL4_STATUS;
    rows.push(tg);
    let mut rx = ERow::port(
        "REB_FROM_REACTOR",
        "Reboiler heat from reactor steam, last",
        "source",
        "heat",
        Some(b.gap_after_tg),
        None,
        "J.18",
    );
    rx.source = "engine heat recovery, approved order".to_string();
    rx.status = LEVEL4_STATUS;
    rows.push(rx);
    rows.extend([
        ERow::port(
            "POWER_GENERATED",
            "Electricity from reactor heat",
            "source",
            "electricity",
            Some(b.e_nuc),
            None,
            "B.18",
        ),
        ERow::port(
            "HE_CIRCULATION",
            "Helium circulators",
            "sink",
            "electricity",
            Some(b.e_circ),
            None,
            "B.12",
        ),
        ERow::port(
            "PUMP_COMPRESS",
            "Lean pump, CO2 compression, purge recovery",
            "sink",
            "electricity",
            Some(e.mea_pump + e.co2comp + e.purge),
            None,
            "J.22",
        ),
        ERow::port(
            "COOLING",
            "Cooling towers and MEA fans",
            "sink",
            "electricity",
            Some(e.cooling + e.mea_cooling),
            None,
            "J.21",
        ),
        ERow::port(
            "REFORMING_POWER",
            "Reformer train electricity, TEV-961 Table 2 row on the gas rate",
            "sink",
            "electricity",
            Some(e.reforming),
            None,
            "J.19",
        ),
        ERow::port(
            "PURIFICATION_POWER",
            "Gas purification electricity, Selexol plus PSA or the PSA alone",
            "sink",
            "electricity",
            Some(e.purification),
            None,
            "J.19",
        ),
        ERow::port(
            "STEAM_SYSTEM_POWER",
            "Steam system electricity, TEV-961 Table 2 row",
            "sink",
            "electricity",
            Some(e.steam),
            None,
            "J.19",
        ),
        ERow::port(
            "WATER_POWER",
            "Water treatment electricity, TEV-961 Table 2 row",
            "sink",
            "electricity",
            Some(e.water),
            None,
            "J.19",
        ),
        ERow::port(
            "TAIL_GAS_POWER",
            "Electricity from tail gas burned on site",
            "source",
            "electricity",
            Some(b.e_tg),
            None,
            "K.17",
        ),
        ERow::port(
            "UNMET_THERMAL",
            "Heat duty above the rated reactor",
            "sink",
            "heat",
            Some(m.unmet_thermal(m.r_rated, &b)),
            None,
            "J.18",
        ),
        ERow::port(
            "UNMET_ELECTRICAL",
            "Electricity above the rated reactor",
            "sink",
            "electricity",
            Some(short.max(0.0)),
            None,
            "J.29",
        ),
        ERow::port(
            "SPARE_MARGIN",
            "Rated reactor less the need",
            "source",
            "heat",
            Some(m.r_rated - b.r),
            None,
            "J.36",
        ),
    ]);
    Ok(rows)
}

/// Rows of one Level 3 run: the process-side heat the flowsheet computes. The
/// reactor side is not rebuilt from these duties, because T03 stays anchored.
fn flowsheet_energy_rows(run: &FlowsheetRun) -> Result<Vec<ERow>> {
    let fs_row = |id, name, side, v: f64, eq, status, source: String| ERow {
        id,
        name,
        side,
        kind: "heat",
        value: Some(v),
        low: None,
        high: None,
        temperature: None,
        rigour: "rigorous",
        source,
        equation: eq,
        status,
    };
    let syngas = heatrec::syngas_above_w(run.gas("S09A")?, T05_LIMIT_K)? / 1.0e6;
    let q_reb = run.duty("Q_CAPTURE").unwrap_or(0.0) / 1.0e6;
    let mut rows = vec![
        fs_row(
            "SMR_HEAT",
            "Feed preheat plus reformer duty of the flowsheet",
            "sink",
            run.process_heat_mw(),
            "J.14",
            FLOWSHEET_STATUS,
            format!(
                "Level 3 flowsheet, reformer exit {:.0} C",
                run.spec.t_reformer_out_c
            ),
        ),
        fs_row(
            "WGS_RELEASE",
            "Water-gas shift heat release, HTS and LTS",
            "source",
            wgs_release_mw(run)?,
            "E.6",
            LEVEL4_STATUS,
            "engine heat recovery on the flowsheet beds".to_string(),
        ),
        fs_row(
            "SYNGAS_RECOVERY",
            "Syngas heat above 130 C on the flowsheet EX3 outlet",
            "source",
            syngas,
            "F.11",
            LEVEL4_STATUS,
            "engine heat recovery on the flowsheet stream S09A".to_string(),
        ),
        fs_row(
            "MEA_REGEN",
            "Reboiler duty of the capture island",
            "sink",
            q_reb,
            "J.16",
            FLOWSHEET_STATUS,
            "Level 3 flowsheet, anchored capture island".to_string(),
        ),
    ];
    let mut t = rows.remove(2);
    t.temperature = Some(T05_LIMIT_K - 273.15);
    rows.insert(2, t);
    Ok(rows)
}

fn energy_balance(mh: &Model, m0: &Model, mm: &Model, fs: &Result<Suite>) -> Result<String> {
    let mut c = Csv::new(&[
        "run_id",
        "term_id",
        "term",
        "side",
        "kind",
        "value_mw",
        "low_mw",
        "high_mw",
        "temperature_c",
        "class",
        "rigour",
        "source",
        "equation",
        "closure_group",
        "status",
    ]);
    for r in RUNS {
        let rows = run_rows(mh, &r, matched_run(fs, &r))?;
        write_erows(&mut c, r.id, rows);
    }
    let b_rec = RUNS
        .iter()
        .find(|r| r.id == "B_REC_F")
        .copied()
        .ok_or_else(|| nush2_numerics::EngineError::Parse("B_REC_F missing".into()))?;
    for (suffix, m) in [("ANCHORED", m0), ("MODELLED", mm)] {
        let mut rows = run_rows(m, &b_rec, matched_run(fs, &b_rec))?;
        for row in rows.iter_mut() {
            if row.status == STATUS {
                row.source = format!("engine anchored port, {} preset", suffix.to_lowercase());
            }
        }
        write_erows(&mut c, &format!("B_REC_F_{suffix}"), rows);
    }
    if let Ok(suite) = fs {
        for run in &suite.runs {
            write_erows(&mut c, &run.id, flowsheet_energy_rows(run)?);
            envelope_energy_rows(&mut c, run)?;
        }
    }
    Ok(c.out)
}

/// Per-unit duties and envelope enthalpy flows of one Level 3 run, signed, in
/// the closure group `envelope`: sources less sinks is zero. Duties marked by
/// closure come from blocks with no energy model, so for them the closure is
/// bookkeeping.
fn envelope_energy_rows(c: &mut Csv, run: &FlowsheetRun) -> Result<()> {
    let mut put = |id: String, name: String, side: &str, kind: &str, w: f64, rigour, source| {
        c.row(&[
            run.id.clone(),
            id,
            name,
            t(side),
            t(kind),
            num(w / 1.0e6),
            String::new(),
            String::new(),
            String::new(),
            t("calculated"),
            t(rigour),
            t(source),
            t("A.8"),
            t("envelope"),
            t(FLOWSHEET_STATUS),
        ]);
    };
    let env = flowsheet::envelope_enthalpy(run)?;
    for (tag, h) in &env.inlets {
        put(
            format!("H_{tag}"),
            format!("Enthalpy flow of {tag} into the envelope, formation basis, signed"),
            "source",
            "enthalpy",
            *h,
            "rigorous",
            "Level 3 flowsheet stream",
        );
    }
    for d in &run.duties {
        let kind = match d.kind {
            EnergyKind::Heat => "heat",
            EnergyKind::ShaftWork => "shaft",
            EnergyKind::Electric => "electricity",
        };
        let (rigour, source, how) = if d.by_closure {
            (
                "fixed_assumption",
                "Level 3 flowsheet duty set by closure, no energy model",
                ", set by closure",
            )
        } else {
            ("rigorous", "Level 3 flowsheet unit model", "")
        };
        put(
            d.tag.clone(),
            format!("{} duty into the process, signed{how}", d.unit),
            "source",
            kind,
            d.w_into_process,
            rigour,
            source,
        );
    }
    for (tag, h) in &env.outlets {
        put(
            format!("H_{tag}"),
            format!("Enthalpy flow of {tag} out of the envelope, formation basis, signed"),
            "sink",
            "enthalpy",
            *h,
            "rigorous",
            "Level 3 flowsheet stream",
        );
    }
    Ok(())
}

const EMISSIONS_HEADER: [&str; 13] = [
    "run_id",
    "baseline_id",
    "view",
    "line_id",
    "line",
    "t_per_y",
    "kg_per_kg_h2",
    "per_pass_capture",
    "plant_capture",
    "class",
    "source",
    "equation",
    "status",
];

fn emissions_ledger(m: &Model, modelled: Option<(&Model, &Model, &str)>) -> Result<String> {
    let mut c = Csv::new(&EMISSIONS_HEADER);
    for r in RUNS {
        emissions_run(&mut c, m, &r, r.id)?;
    }
    // Case B REC_F on the Modelled preset, rows that differ from the Anchored
    // preset carrying the Modelled status word.
    if let Some((mm, m0, word)) = modelled {
        let b_rec = run_def("B_REC_F")?;
        let mut on_mod = Csv::new(&EMISSIONS_HEADER);
        emissions_run(&mut on_mod, mm, &b_rec, "B_REC_F_MODELLED")?;
        let mut on_anch = Csv::new(&EMISSIONS_HEADER);
        emissions_run(&mut on_anch, m0, &b_rec, "B_REC_F_MODELLED")?;
        let marked = mark(on_mod.out, &on_anch.out, Some(word))?;
        c.out
            .push_str(marked.split_once('\n').map_or("", |(_, rest)| rest));
    }
    Ok(c.out)
}

/// Emission lines of one fixed run, written under `run_id`.
fn emissions_run(c: &mut Csv, m: &Model, r: &RunDef, run_id: &str) -> Result<()> {
    let r = *r;
    let s = m.scale(r.case);
    let b = balance_for(m, &r)?;
    let e = m.emissions(&b.c, s);
    let h2 = m.h2_a_kgh * s * b.c.h_scale * m.hours / 1000.0;
    let per_pass = if r.scn == Scenario::InlSel {
        m.eta_sel
    } else {
        m.eta_mea
    };
    let plant = e.captured / (e.captured + e.atm);
    let sys = m.system_adjust(&b, s);
    let lc = m.lifecycle_terms(&b, s, b.r);
    for (bid, base) in [
        ("CHO_9p35", m.base_cho * h2),
        ("IEAGHG_9p00", m.base_ieaghg * h2),
        ("INL_9p21", e.baseline),
    ] {
        let a = base - e.atm;
        type Line<'a> = (&'a str, &'a str, f64, &'a str);
        let views: [(&str, Vec<Line<'_>>); 4] = [
            (
                "direct_atm",
                vec![
                    ("BASELINE", "Unabated SMR baseline", base, "K.3"),
                    ("CAPTURED", "Captured and stored", e.captured, "K.7"),
                    ("N2_RICH_CUT", "Residual to atmosphere", e.atm, "K.8"),
                    ("ABATEMENT", "Net abatement", a, "K.11"),
                ],
            ),
            (
                "lifecycle_burdens",
                vec![
                    (
                        "NUCLEAR_LC",
                        "Nuclear lifecycle burden",
                        -lc.nuclear,
                        "K.19",
                    ),
                    ("TS_BURDEN", "T&S burden", -lc.ts_burden, "K.19"),
                    (
                        "ABATEMENT",
                        "Net abatement",
                        a - lc.nuclear - lc.ts_burden,
                        "K.19",
                    ),
                ],
            ),
            (
                "lifecycle_upstream",
                vec![
                    (
                        "UPSTREAM_CREDIT",
                        "Saved upstream gas",
                        lc.up_credit,
                        "K.19",
                    ),
                    (
                        "NUCLEAR_LC",
                        "Nuclear lifecycle burden",
                        -lc.nuclear,
                        "K.19",
                    ),
                    ("TS_BURDEN", "T&S burden", -lc.ts_burden, "K.19"),
                    (
                        "ABATEMENT",
                        "Net abatement",
                        a + lc.up_credit - lc.nuclear - lc.ts_burden,
                        "K.19",
                    ),
                ],
            ),
            (
                "system",
                vec![("ABATEMENT", "Net abatement", a + sys, "K.18")],
            ),
        ];
        for (view, lines) in views {
            for (lid, name, v, eq) in lines {
                c.row(&[
                    t(run_id),
                    t(bid),
                    t(view),
                    t(lid),
                    t(name),
                    num(v),
                    num(v / h2),
                    num(per_pass),
                    num(plant),
                    t("calculated"),
                    t("engine anchored port"),
                    t(eq),
                    t(STATUS),
                ]);
            }
        }
    }
    Ok(())
}

/// Class, source, source status and equation of each ledger line.
fn line_meta(id: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match id {
        "GAS" => (
            "Gas saved against the unabated SMR",
            "TEV-953, TEV-961 Table 2",
            "primary",
            "L.16",
        ),
        "REACTOR_CAP" => (
            "Reactor capital",
            "INL/RPT-23-72972 PDF p47",
            "primary",
            "L.6",
        ),
        "CAPTURE_CAP" => (
            "Capture capital",
            "IEAGHG 2017-02 Tables 2 and 3",
            "primary",
            "L.12",
        ),
        "CAPTURE_FOM" => (
            "Capture fixed O&M",
            "2.7% of capture capital per year, derived from IEAGHG 2017-02 cases, not verified (decision D10)",
            "via_reader",
            "L.13",
        ),
        "CAPTURE_ALLIN" => (
            "All-in capture charge, off in the bottom-up ledger",
            "ETC exhibit",
            "not_verified",
            "L.15",
        ),
        "TS" => (
            "CO2 transport and storage",
            "Author tariff, decision D10, with the low and high ends in TS_LOW and TS_HIGH",
            "primary",
            "L.15",
        ),
        "REACTOR_OM" => (
            "Reactor O&M and fuel",
            "INL/RPT-23-72972 PDF p37, p47",
            "primary",
            "L.11",
        ),
        "IHX" => (
            "IHX and secondary loop",
            "IAEA-TECDOC-1682 Table 3.17, not verified",
            "not_verified",
            "L.18",
        ),
        "LICENCE" => (
            "Extra licensing",
            "NRC blog, generic proxy",
            "via_reader",
            "L.19",
        ),
        "DECOM" => ("Decommissioning", "INL NZW09 PDF p7", "primary", "L.20"),
        "IDC" => (
            "Interest during construction on the reactor",
            "INL NZW09 PDF p7",
            "primary",
            "L.21",
        ),
        "POWER_ISLAND" => (
            "Power island add-back",
            "INL recipe, calculated",
            "primary",
            "L.22",
        ),
        "RECYCLE_TRAIN" => (
            "Recycle and cryogenic train",
            "IEAGHG 2017-02 proxy, no published cost",
            "via_reader",
            "L.23",
        ),
        "LOST_STEAM" => ("Lost steam export", "TEV-953 Table 2", "primary", "L.24"),
        "BASE_POWER" => (
            "Baseline purchased power avoided",
            "TEV-953 Table 2",
            "primary",
            "L.24",
        ),
        _ => ("Other revenue", "none", "not_verified", "L.24"),
    }
}

fn economic_ledger(m: &Model) -> Result<String> {
    let mut c = Csv::new(&[
        "run_id",
        "stage",
        "life_y",
        "n",
        "reactor_charged_mwth",
        "level",
        "line_id",
        "line",
        "value_sgdm_y",
        "low",
        "high",
        "cum_sgdm_y",
        "cum_s_per_t",
        "abatement_t_y",
        "class",
        "source",
        "source_status",
        "equation",
        "dcf_value",
        "dcf_unit",
        "status",
    ]);
    let lives = m.lives;
    let exponents = [m.n_linear, m.n_team, m.n_ngnp];
    for r in RUNS {
        let b = balance_for(m, &r)?;
        let a = m.emissions(&b.c, b.s).a_atm;
        let headline = r.id == "B_REC_F";
        let stages: Vec<Stage> = if headline {
            Stage::ALL.to_vec()
        } else {
            vec![Stage::Foak, Stage::Boak]
        };
        let life_set: Vec<f64> = if headline {
            lives.to_vec()
        } else {
            vec![lives[0], lives[2]]
        };
        let n_set: Vec<f64> = if headline {
            exponents.to_vec()
        } else {
            vec![m.n_linear]
        };
        let charged = crate::charged(m, r.case, &b);
        let r_set: Vec<f64> = if headline {
            vec![charged, b.r]
        } else {
            vec![charged]
        };
        for stage in &stages {
            for life in &life_set {
                for n in &n_set {
                    for rc in &r_set {
                        let q = CostQuery::central(*stage, *life, *n, *rc);
                        let led = m.ledger(&b, &q)?;
                        let lo = m.ledger(
                            &b,
                            &CostQuery {
                                level: CostLevel::Low,
                                ..q
                            },
                        )?;
                        let hi = m.ledger(
                            &b,
                            &CostQuery {
                                level: CostLevel::High,
                                ..q
                            },
                        )?;
                        let (lo_l, hi_l) = (lo.lines(), hi.lines());
                        let mut cum = 0.0;
                        for (i, (id, rung, v)) in led.lines().into_iter().enumerate() {
                            cum += v;
                            // Zero in the bottom-up ledger, and outside the schema.
                            if matches!(id, "CAPTURE_ALLIN" | "EXPORT_REVENUE") {
                                continue;
                            }
                            let (line, src, st, eq) = line_meta(id);
                            c.row(&[
                                t(r.id),
                                t(stage.id()),
                                format!("{life:.0}"),
                                format!("{n:.4}"),
                                format!("{rc:.2}"),
                                t(LEVELS[rung]),
                                t(id),
                                t(line),
                                num(v),
                                num(lo_l[i].2),
                                num(hi_l[i].2),
                                num(cum),
                                num(cum * 1e6 / a),
                                num(a),
                                t("calculated"),
                                t(src),
                                t(st),
                                t(eq),
                                String::new(),
                                String::new(),
                                t(STATUS),
                            ]);
                        }
                    }
                }
            }
        }
    }
    // Method 2 rows on the headline run, for every stage, life, exponent and
    // charged size the ledger carries, so the two families match.
    let r = RUNS[4];
    let b = balance_for(m, &r)?;
    for d in dcf_family(m, &b)? {
        let mut lines: Vec<(String, String, f64, &str, &str, &str)> = vec![
            (
                "LCOH_SMR".into(),
                "LCOH of the unabated SMR".into(),
                d.lcoh_smr,
                "S$/kg",
                "L.35",
                LCOH_SOURCE,
            ),
            (
                "LCOH_NUSH2".into(),
                "LCOH of NUS-H2".into(),
                d.lcoh_nush2,
                "S$/kg",
                "L.35",
                LCOH_SOURCE,
            ),
            (
                "LAC_DCF".into(),
                "Levelised abatement cost".into(),
                d.lac,
                "S$/t",
                "L.36",
                DCF_SOURCE,
            ),
            (
                "LAC_L5_GAP".into(),
                "LAC_DCF less the L5 ledger".into(),
                d.gap_abs,
                "S$/t",
                "L.37",
                DCF_SOURCE,
            ),
            (
                "LAC_L5_GAP_REL".into(),
                "LAC_DCF less the L5 ledger, share of L5; the limit is 0.10".into(),
                d.gap_rel,
                "-",
                "L.37",
                DCF_SOURCE,
            ),
        ];
        // The named terms of the gap, so a difference is always explained by a
        // line, on the size exponent of the headline.
        if d.n == m.n_linear {
            for g in &d.gap_terms {
                lines.push((
                    format!("GAP_{}", g.id),
                    format!("Part of LAC_DCF less L5 from {}", g.id),
                    g.delta,
                    "S$/t",
                    "L.37",
                    DCF_SOURCE,
                ));
            }
        }
        for (id, line, v, unit, eq, src) in lines {
            c.row(&[
                t(r.id),
                t(d.stage.id()),
                d.life_y.to_string(),
                format!("{:.4}", d.n),
                format!("{:.2}", d.r_charged),
                t("DCF"),
                id,
                line,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                num(d.annual_abatement_t),
                t("calculated"),
                t(src),
                t("primary"),
                t(eq),
                num(v),
                t(unit),
                t(STATUS),
            ]);
        }
    }
    Ok(c.out)
}

/// Source text of the DCF rows.
const DCF_SOURCE: &str =
    "IEAGHG 2017-02 Tables 3 and 4 for the SMR island, ledger inputs otherwise";

/// Source text of the LCOH rows, with the limit only LCOH carries.
const LCOH_SOURCE: &str = "IEAGHG 2017-02 Tables 3 and 4 for the SMR island, ledger inputs \
     otherwise. Limit: the island capital includes IEAGHG's own power island (TPC EUR 20.12M, \
     PDF p222), while purchased power and export steam follow TEV-953 Table 2. Both plants \
     carry the island unchanged, so the mismatch cancels in LAC and reaches LCOH only";

/// The DCF family of the headline run: every stage, life and exponent, with
/// the rated reactor and the need charged, in ledger order.
pub fn dcf_family(m: &Model, b: &Balance) -> Result<Vec<DcfResult>> {
    let charged = crate::charged(m, Case::B, b);
    let mut out = Vec::new();
    for stage in Stage::ALL {
        for life in m.lives {
            for n in [m.n_linear, m.n_team, m.n_ngnp] {
                for rc in [charged, b.r] {
                    out.push(m.dcf(b, &CostQuery::central(stage, life, n, rc))?);
                }
            }
        }
    }
    Ok(out)
}

/// Status of the sensitivity rows that run the Level 3 flowsheet at M8.
pub const FLOWSHEET_M8_STATUS: &str = "flowsheet_m8";

/// Decision S11: the reject H2 that REC-CRYO does not return to the PSA goes to
/// the recycle instead of the nitrogen-rich cut, as one recorded sensitivity on
/// Case B REC_F. The default split is unchanged.
pub fn cryo_split_rows(m: &Model, fs: &Result<Suite>) -> Result<Vec<SensRow>> {
    let Ok(suite) = fs else {
        return Ok(Vec::new());
    };
    let Some(base) = suite.run("FS_B_REC_F") else {
        return Ok(Vec::new());
    };
    let mut spec = base.spec.clone();
    spec.h2_rest_to_recycle = true;
    let note = "decision S11: reject H2 not returned to the PSA routed to the recycle; one \
                recorded sensitivity, the default split is unchanged";
    let var = match flowsheet::solve(
        m,
        &spec,
        "FS_B_REC_F_H2_TO_RECYCLE",
        flowsheet::TearStart::Anchored,
    ) {
        Ok(v) => v,
        Err(_) => {
            return Ok(vec![SensRow {
                sweep_id: "CRYO_H2_SPLIT",
                run_id: "FS_B_REC_F".into(),
                parameter: "Reject H2 not returned to the PSA goes to the recycle",
                unit: "flag",
                base_value: 0.0,
                test_value: 1.0,
                metric: "NOT_CONVERGED",
                result: f64::NAN,
                delta: f64::NAN,
                note: "decision S11 variant did not converge".into(),
                status: Some(crate::sens::NOT_CONVERGED),
            }])
        }
    };
    let phi = |r: &FlowsheetRun| {
        r.gas("S11")
            .map(nush2_units::case5::phi_after_shift)
            .unwrap_or(f64::NAN)
    };
    let metrics: [(&'static str, f64, f64); 8] = [
        ("PHI_AFTER_SHIFT", phi(base), phi(&var)),
        ("TUBE_C_KMOL_H", base.carbon.tube, var.carbon.tube),
        (
            "TAIL_GAS_C_KMOL_H",
            base.carbon.tail_gas,
            var.carbon.tail_gas,
        ),
        (
            "RECYCLED_C_KMOL_H",
            base.carbon.recycled,
            var.carbon.recycled,
        ),
        ("BURNED_C_KMOL_H", base.carbon.burned, var.carbon.burned),
        ("CAPTURED_KMOL_H", base.carbon.captured, var.carbon.captured),
        ("NG_MMSCFD", base.ng_mmscfd, var.ng_mmscfd),
        (
            "PROCESS_HEAT_MW",
            base.process_heat_mw(),
            var.process_heat_mw(),
        ),
    ];
    Ok(metrics
        .into_iter()
        .map(|(metric, b0, v)| SensRow {
            sweep_id: "CRYO_H2_SPLIT",
            run_id: "FS_B_REC_F".into(),
            parameter: "Reject H2 not returned to the PSA goes to the recycle",
            unit: "flag",
            base_value: 0.0,
            test_value: 1.0,
            metric,
            result: v,
            delta: v - b0,
            note: note.into(),
            status: Some(FLOWSHEET_M8_STATUS),
        })
        .collect())
}

/// The sensitivity table on registry `p`. `cryo` holds the S11 rows, solved
/// once by the caller.
fn sensitivity(p: &ParamSet, fsx: &FsInputs, cryo: &[SensRow]) -> Result<String> {
    let mut c = Csv::new(&[
        "sweep_id",
        "run_id",
        "parameter",
        "unit",
        "base_value",
        "test_value",
        "metric",
        "result",
        "delta",
        "class",
        "note",
        "status",
    ]);
    let mut rows = sweeps(p, fsx)?;
    rows.sort_by(|a, b| {
        (a.sweep_id, &a.run_id, a.metric)
            .cmp(&(b.sweep_id, &b.run_id, b.metric))
            .then(a.test_value.total_cmp(&b.test_value))
    });
    rows.extend(cryo.iter().cloned());
    for r in rows {
        c.row(&[
            t(r.sweep_id),
            r.run_id,
            t(r.parameter),
            t(r.unit),
            num(r.base_value),
            num(r.test_value),
            t(r.metric),
            opt(Some(r.result).filter(|v| v.is_finite())),
            opt(Some(r.delta).filter(|v| v.is_finite())),
            t("calculated"),
            r.note,
            t(r.status.unwrap_or(STATUS)),
        ]);
    }
    Ok(c.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marks() -> Marks {
        Marks {
            headline: vec!["T05"],
            modelled: vec!["T01", "T05"],
        }
    }

    #[test]
    fn a_row_that_moves_with_a_gated_term_says_so() {
        let head = "id,value,status\nA,1.0,anchored_port\nB,2.5,anchored_port\nC,3.0,level4_m8\n";
        let anch = "id,value,status\nA,1.0,anchored_port\nB,2.0,anchored_port\nC,9.0,level4_m8\n";
        let out = mark(head.to_string(), anch, Some("gated_T05")).unwrap();
        assert_eq!(
            out,
            "id,value,status\nA,1.0,anchored_port\nB,2.5,gated_T05\nC,3.0,level4_m8\n"
        );
        // No gated term, no change.
        assert_eq!(mark(head.to_string(), anch, None).unwrap(), head);
        // Tables of another shape cannot pair.
        assert!(mark(head.to_string(), "id,value,status\n", Some("x")).is_err());
    }

    #[test]
    fn a_signed_zero_is_not_a_change() {
        let head = "id,value,status\nA,-0.000000,anchored_port\nB,1.5,anchored_port\n";
        let anch = "id,value,status\nA,0.000000,anchored_port\nB,1.50,anchored_port\n";
        assert_eq!(
            mark(head.to_string(), anch, Some("gated_T05")).unwrap(),
            head
        );
        assert!(same_cell("NaN", "NaN"));
        assert!(!same_cell("0.000001", "0.000000"));
        assert!(!same_cell("text", "0"));
    }

    #[test]
    fn quoted_cells_split_back() {
        assert_eq!(
            cells("a,\"b, c\",\"say \"\"hi\"\"\",d"),
            vec!["a", "b, c", "say \"hi\"", "d"]
        );
    }

    #[test]
    fn term_rows_follow_the_preset_that_used_the_model() {
        let m = marks();
        assert_eq!(m.headline_word().unwrap(), "gated_T05");
        assert_eq!(m.modelled_word().unwrap(), "gated_T01_T05");
        assert_eq!(row_status("TERM_T05_USED_HEADLINE", &m), "gated_T05");
        assert_eq!(row_status("TERM_T01_USED_HEADLINE", &m), STATUS);
        assert_eq!(row_status("TERM_T01_USED_MODELLED", &m), "gated_T01_T05");
        assert_eq!(row_status("TERM_T05_MODELLED", &m), "model_m4");
        assert_eq!(row_status("RES_H2_T_Y", &m), STATUS);
        let none = Marks {
            headline: vec![],
            modelled: vec![],
        };
        assert_eq!(row_status("TERM_T05_USED_HEADLINE", &none), STATUS);
    }

    #[test]
    fn modelled_energy_rows_compare_with_the_anchored_rows() {
        let row = |run: &str, term: &str, v: &str| {
            format!("{run},{term},n,source,heat,{v},,,,calculated,r,s,J.1,anchored_port")
        };
        let text = [
            row("B_REC_F_ANCHORED", "X", "1.0"),
            row("B_REC_F_ANCHORED", "Y", "2.0"),
            row("B_REC_F_MODELLED", "X", "1.0"),
            row("B_REC_F_MODELLED", "Y", "2.1"),
        ]
        .join("\n")
            + "\n";
        let out = mark_modelled(text, Some("gated_T01_T05"));
        let last: Vec<&str> = out.lines().map(|l| l.rsplit_once(',').unwrap().1).collect();
        assert_eq!(
            last,
            [
                "anchored_port",
                "anchored_port",
                "anchored_port",
                "gated_T01_T05"
            ]
        );
    }
}
