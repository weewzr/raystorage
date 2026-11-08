//! CSV exporter. Returns strings, so the same tables reach the CLI and the
//! browser. Schemas follow freeze section 8, with a trailing `status` column.
//! Economic rows also carry `dcf_value` and `dcf_unit`, because S$/kg and S$/t
//! have no home in a S$M/y column. Rows without a model read `not_run` and stay
//! empty.
//!
//! Every result row is the Headline preset: the anchored balance with the terms
//! whose gates passed on their models (module `terms`). Its rows keep the status
//! `anchored_port`, since the balance is still the anchored port; the term rows
//! of the design basis say which terms are modelled. The energy balance also
//! carries Case B REC_F on the Anchored and Modelled presets, so the drift sits
//! in the file.
//!
//! The mass balance carries the Level 3 flowsheet runs of M7 with every species,
//! under run ids that start with `FS_` and the status `flowsheet_m7`. Rows that
//! M8 models compute (heat recovery on real flows, the Level 4 rebuild of the
//! reactor side, the reboiler grade check) read `level4_m8`.

use crate::anchored::{Balance, Case, CostLevel, Model, Scenario};
use crate::carbon::carbon_kmol_h;
use crate::dcf::DcfResult;
use crate::drift::{self, TermRecord};
use crate::economics::{CostQuery, Stage, LEVELS};
use crate::flowsheet::{self, FlowsheetRun, Suite};
use crate::gates::Gates;
use crate::heatrec::{self, SourceId};
use crate::params::{ParamEntry, ParamSet};
use crate::sens::{sweeps, SensRow};
use crate::terms::{Source, Term};
use nush2_numerics::Result;
use nush2_props::units::{kmol_h_to_t_h, mmscfd_to_kmol_h, MW_CO2, MW_H2, MW_H2O, MW_MEA};
use nush2_units::case5::T05_LIMIT_K;
use nush2_units::stream::{ModelClass, Port, Sp};

/// Status of every result row at this milestone.
pub const STATUS: &str = "anchored_port";
/// Status of rows computed by the Level 3 flowsheet.
pub const FLOWSHEET_STATUS: &str = "flowsheet_m7";
/// Status of rows computed by the M8 models on real flows.
pub const LEVEL4_STATUS: &str = "level4_m8";

/// Status of a design-basis result row. A modelled term slot carries the
/// milestone of its model; Level 4 rows read `level4_m8`.
fn row_status(id: &str) -> &'static str {
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
        };
    }
    if id.starts_with("L4_") {
        LEVEL4_STATUS
    } else {
        STATUS
    }
}

/// Source text of a result row from its status.
fn status_source(status: &str) -> &'static str {
    match status {
        "model_m4" => "engine M4 gas units",
        "model_m6" => "engine M6 column models",
        FLOWSHEET_STATUS => "engine Level 3 flowsheet",
        LEVEL4_STATUS => "engine Level 4 heat recovery and utility",
        _ => "engine anchored port",
    }
}

/// A fixed exporter run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunDef {
    pub id: &'static str,
    pub case: Case,
    pub scn: Scenario,
}

/// Run definitions of freeze section 7.3, sorted by id.
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
    Ok(Tables {
        final_design_basis: final_design_basis(p, &mh, &rec)?,
        integrated_mass_balance: mass_balance(&mh, &fs)?,
        integrated_energy_balance: energy_balance(&mh, &m0, &mm, &fs)?,
        emissions_ledger: emissions_ledger(&mh)?,
        economic_ledger: economic_ledger(&mh)?,
        sensitivity_results: sensitivity(&rec.p_head, &m0, &fs)?,
    })
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

fn final_design_basis(p: &ParamSet, m: &Model, rec: &TermRecord) -> Result<String> {
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
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    for r in rows {
        let status = row_status(&r.id);
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
            t(status_source(status)),
            t("see equation"),
            String::new(),
            t(r.equation),
            t(r.decision),
            t(status),
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
        plain_name(&e.id),
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
            "A_900",
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
        rows.push(ResultRow {
            id: id.to_string(),
            group,
            parameter: plain_name(id.trim_start_matches("RES_")),
            b: sb.get(key).copied(),
            a_inl: sa_inl.get(key).copied(),
            a_mea: sa_mea.get(key).copied(),
            unit,
            equation,
            decision,
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
                    "Reformer helium annulus drop at {n:.0} HA tubes, smooth wall, fins left out"
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

/// Mass balance header: freeze section 8, with the four C2+ species beside their
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

fn mass_balance(m: &Model, fs: &Result<Suite>) -> Result<String> {
    let mut c = Csv::new(&MASS_HEADER);
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
    for r in RUNS {
        let s = m.scale(r.case);
        let b = balance_for(m, &r)?;
        let k = b.c;
        let h2_kmol = m.h2_a_kgh * s * k.h_scale / MW_H2;
        let h2_feed = h2_kmol / m.psa_rec;
        let cap = k.cap * s;
        let ret = fs.as_ref().ok().map(|f| f.closure.h2_return);
        let h2_ret = ret.map(|r| r * (h2_feed - h2_kmol));
        let fs_run = matched_run(fs, &r);
        let streams = [
            S {
                id: "H2_PRODUCT",
                name: "Purified H2 after external PSA",
                from: "PSA",
                to: "PRODUCT",
                kg_h: Some(h2_kmol * MW_H2),
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
                name: "Burned nitrogen-rich cut",
                from: "CRYO",
                to: "STACK",
                kg_h: None,
                kmol_h: None,
                h2: None,
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
                name: "H2-rich gas into the PSA: capture outlet plus the cryogenic return",
                from: "CAPTURE",
                to: "PSA",
                kg_h: None,
                kmol_h: None,
                h2: Some(h2_feed),
                co2: None,
                ch4: None,
                carbon: Some((k.g + k.p) * s),
                rigour: "external_boundary",
            },
            S {
                id: "PSA_REJECT_RETURN",
                name: "PSA reject returning across the boundary",
                from: "PSA",
                to: "CRYO",
                kg_h: None,
                kmol_h: None,
                h2: Some(h2_feed - h2_kmol),
                co2: None,
                ch4: None,
                carbon: Some(k.g * s),
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
                h2: None,
                co2: None,
                ch4: None,
                carbon: Some(k.exported * s),
                rigour: "external_boundary",
            },
        ];
        for x in streams {
            c.row(&[
                t(r.id),
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
            cells[0] = t(r.id);
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
        cells[26] = format!("engine Level 3 flowsheet, from {from}");
        cells[27] = t(FLOWSHEET_STATUS);
        c.row(&cells);
    }
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
            t(r.status),
        ]);
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
    let (short, heat_first) = m.shortfall_at(m.r_rated, &b);
    let alloc = heatrec::allocate(b.q_reb, &b.stack);
    let from_tg = (b.gap - b.gap_after_tg).max(0.0);
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
            "Syngas heat above 130 C to the reboiler",
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
            "Heat stack for the reboiler",
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
            "UNMET_THERMAL",
            "Heat duty above the rated reactor",
            "sink",
            "heat",
            Some((heat_first - m.r_rated).max(0.0)),
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
        }
    }
    Ok(c.out)
}

fn emissions_ledger(m: &Model) -> Result<String> {
    let mut c = Csv::new(&[
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
    ]);
    for r in RUNS {
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
                        t(r.id),
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
    }
    Ok(c.out)
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
            "Assumed 2.7% of capital",
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
            "User decision Q10",
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
            "IAEA-TECDOC-1682 via old repo",
            "via_old_repo",
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
    // Method 2 rows, headline run only.
    let r = RUNS[4];
    let b = balance_for(m, &r)?;
    for (d, _) in dcf_grid(m, &b)? {
        for (id, line, v, unit, eq) in [
            (
                "LCOH_SMR",
                "LCOH of the unabated SMR",
                d.lcoh_smr,
                "S$/kg",
                "L.35",
            ),
            (
                "LCOH_NUSH2",
                "LCOH of NUS-H2",
                d.lcoh_nush2,
                "S$/kg",
                "L.35",
            ),
            ("LAC_DCF", "Levelised abatement cost", d.lac, "S$/t", "L.36"),
            (
                "LAC_L5_GAP",
                "LAC_DCF less the L5 ledger",
                d.gap_abs,
                "S$/t",
                "L.37",
            ),
        ] {
            c.row(&[
                t(r.id),
                t(d.stage.id()),
                d.life_y.to_string(),
                format!("{:.4}", d.n),
                format!("{:.2}", d.r_charged),
                t("DCF"),
                t(id),
                t(line),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                num(d.annual_abatement_t),
                t("calculated"),
                t("IEAGHG 2017-02 Tables 3 and 4 for the SMR island, ledger inputs otherwise"),
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
                note: "decision S11 variant did not converge",
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
            note,
        })
        .collect())
}

fn sensitivity(p: &ParamSet, m0: &Model, fs: &Result<Suite>) -> Result<String> {
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
    let mut rows = sweeps(p)?;
    rows.sort_by(|a, b| {
        (a.sweep_id, &a.run_id, a.metric)
            .cmp(&(b.sweep_id, &b.run_id, b.metric))
            .then(a.test_value.total_cmp(&b.test_value))
    });
    for r in rows {
        c.row(&[
            t(r.sweep_id),
            r.run_id,
            t(r.parameter),
            t(r.unit),
            num(r.base_value),
            num(r.test_value),
            t(r.metric),
            num(r.result),
            num(r.delta),
            t("calculated"),
            t(r.note),
            t(STATUS),
        ]);
    }
    for r in cryo_split_rows(m0, fs)? {
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
            t(r.note),
            t(if r.metric == "NOT_CONVERGED" {
                "not_converged"
            } else {
                FLOWSHEET_M8_STATUS
            }),
        ]);
    }
    Ok(c.out)
}
