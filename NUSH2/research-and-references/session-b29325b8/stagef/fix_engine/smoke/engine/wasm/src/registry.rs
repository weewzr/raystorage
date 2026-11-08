//! Parameter records of the browser simulator. Ranges live here and are
//! enforced here, so a request that bypasses the page is refused all the same.
//!
//! Hard range blocks a value. Model range only raises a flag. A number record
//! maps onto one id of the engine registry (`engine/data/parameters.toml`), so
//! its default, class and source are read from that registry at run time.

use nush2_plant::anchored::Model;
use nush2_plant::params::{ParamEntry, ParamSet};
use serde_json::{json, Map, Value};

/// How a record reaches the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Override of one engine registry id, after multiplying by `scale`.
    Registry(&'static str),
    Case,
    Scenario,
    HeatBasis,
    Stage,
    Life,
    Exponent,
    Charged,
    OccLevel,
}

/// One option of an enum record.
#[derive(Debug, Clone, Copy)]
pub struct Opt {
    pub value: &'static str,
    pub label: &'static str,
    pub tag: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Number {
        hard: (f64, f64),
        model: (f64, f64),
        step: f64,
        /// Display unit to engine unit factor.
        scale: f64,
    },
    Enum {
        options: &'static [Opt],
        default: &'static str,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Spec {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub unit: &'static str,
    pub kind: Kind,
    pub target: Target,
    /// Used for enum records, which have no engine registry entry.
    pub class: &'static str,
    pub source: &'static str,
    pub note: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn num(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    unit: &'static str,
    engine_id: &'static str,
    hard: (f64, f64),
    model: (f64, f64),
    step: f64,
    scale: f64,
    note: &'static str,
) -> Spec {
    Spec {
        id,
        label,
        group,
        unit,
        kind: Kind::Number {
            hard,
            model,
            step,
            scale,
        },
        target: Target::Registry(engine_id),
        class: "",
        source: "",
        note,
    }
}

const BASIS: &[Opt] = &[
    Opt {
        value: "case_b",
        label: "Case B, 100,000 Nm3/h H2",
        tag: "headline",
    },
    Opt {
        value: "case_a",
        label: "Case A, INL reference",
        tag: "reference",
    },
];

const TAIL_GAS: &[Opt] = &[
    Opt {
        value: "recycle",
        label: "Recycle to the reformer",
        tag: "approved",
    },
    Opt {
        value: "recycle_fixed",
        label: "Recycle, cryogenic cut carbon held",
        tag: "sensitivity",
    },
    Opt {
        value: "burn_on_site",
        label: "Burn on site, H2 held",
        tag: "sensitivity",
    },
    Opt {
        value: "export",
        label: "Export across the boundary",
        tag: "sensitivity",
    },
    Opt {
        value: "inl_selexol",
        label: "INL Case 6 with Selexol",
        tag: "sensitivity",
    },
];

const HEAT_BASIS: &[Opt] = &[
    Opt {
        value: "inl_scaled",
        label: "INL process heat, scaled",
        tag: "central",
    },
    Opt {
        value: "ahn_2026",
        label: "Ahn 2026 process heat, helium flow following duty",
        tag: "sensitivity",
    },
];

const STAGE: &[Opt] = &[
    Opt {
        value: "BOAK",
        label: "BOAK",
        tag: "central",
    },
    Opt {
        value: "FOAK",
        label: "FOAK",
        tag: "",
    },
    Opt {
        value: "10OAK_P46",
        label: "10-OAK, learning from BOAK (INL PDF p46)",
        tag: "",
    },
    Opt {
        value: "10OAK_P47",
        label: "10-OAK, learning from FOAK (INL PDF p47)",
        tag: "",
    },
];

const LIFE: &[Opt] = &[
    Opt {
        value: "25",
        label: "25 y",
        tag: "central",
    },
    Opt {
        value: "40",
        label: "40 y",
        tag: "",
    },
    Opt {
        value: "60",
        label: "60 y",
        tag: "",
    },
];

const EXPONENT: &[Opt] = &[
    Opt {
        value: "1.0",
        label: "1.0, INL recipe",
        tag: "central",
    },
    Opt {
        value: "0.6",
        label: "0.6",
        tag: "",
    },
    Opt {
        value: "0.31",
        label: "0.31, NGNP 350 and 600 MWth pair",
        tag: "",
    },
];

const CHARGED: &[Opt] = &[
    Opt {
        value: "rating",
        label: "Reactor rating",
        tag: "central",
    },
    Opt {
        value: "required",
        label: "Reactor required by the process",
        tag: "",
    },
];

const OCC: &[Opt] = &[
    Opt {
        value: "low",
        label: "Low",
        tag: "",
    },
    Opt {
        value: "central",
        label: "Central",
        tag: "central",
    },
    Opt {
        value: "high",
        label: "High",
        tag: "",
    },
];

#[allow(clippy::too_many_arguments)]
const fn pick(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    target: Target,
    options: &'static [Opt],
    default: &'static str,
    source: &'static str,
    note: &'static str,
) -> Spec {
    Spec {
        id,
        label,
        group,
        unit: "",
        kind: Kind::Enum { options, default },
        target,
        class: "user_decision",
        source,
        note,
    }
}

/// Every control the page may show. A row is here only if the Headline solve
/// responds to it; the native test checks that.
pub const SPECS: &[Spec] = &[
    pick(
        "basis",
        "Plant basis",
        "Plant basis",
        Target::Case,
        BASIS,
        "case_b",
        "Decision D03 in docs/FINAL_DESIGN_DECISIONS.md",
        "Case A is the INL reference plant",
    ),
    pick(
        "tail_gas",
        "PSA reject fate",
        "Plant basis",
        Target::Scenario,
        TAIL_GAS,
        "recycle",
        "Project decision on the PSA reject fate",
        "Recycle is the approved basis; the others are sensitivities",
    ),
    pick(
        "heat_basis",
        "Process heat basis",
        "Plant basis",
        Target::HeatBasis,
        HEAT_BASIS,
        "inl_scaled",
        "Engine sensitivity HEAT_BASIS",
        "Ahn basis sets process heat to the Ahn Case B value and scales helium flow with duty",
    ),
    num(
        "rating_mwth",
        "Reactor rating",
        "Reactor and helium loop",
        "MWth",
        "REACTOR_RATED_MWTH",
        (100.0, 350.0),
        (150.0, 215.0),
        1.0,
        1.0,
        "Required range 98 to 330 MWth from the project screening model; HTR-Modul 170, MHR-100 215 concept",
    ),
    num(
        "ihx_approach_k",
        "IHX approach",
        "Reactor and helium loop",
        "K",
        "IHX_APPROACH_K",
        (25.0, 100.0),
        (50.0, 82.0),
        1.0,
        1.0,
        "Evidence range: about 50 K typical (IAEA) up to the 77 to 82 K measured at the HTTR hot end (IAEA TECDOC-1382 Table 3-23, IAEA NP-T-4.3 printed p.14); 25 K is the INL assumption (TEV-961 printed p.10)",
    ),
    num(
        "eta_power",
        "Power conversion efficiency",
        "Reactor and helium loop",
        "-",
        "ETA_POWER",
        (0.35, 0.48),
        (0.397, 0.475),
        0.001,
        1.0,
        "TEV-961 note 3; INL/EXT-11-21537 Table 4",
    ),
    num(
        "p_abs_bar",
        "Absorber pressure",
        "Capture",
        "bar",
        "P_ABSORBER",
        (20.0, 30.0),
        (21.0, 26.8),
        0.1,
        1.0,
        "Sets the lean pump head in the Headline solve",
    ),
    num(
        "w_mea",
        "MEA strength",
        "Capture",
        "wt%",
        "W_MEA",
        (20.0, 40.0),
        (30.0, 30.0),
        1.0,
        0.01,
        "Column data exist at 30 wt% only; sets solvent rate and lean pump power",
    ),
    num(
        "lean_loading",
        "Lean loading",
        "Capture",
        "mol/mol",
        "LOADING_LEAN",
        (0.15, 0.40),
        (0.11, 0.30),
        0.01,
        1.0,
        "Rich loading stays at the fixed cap; sets solvent rate",
    ),
    num(
        "capture_target",
        "Capture per pass",
        "Capture",
        "-",
        "ETA_MEA",
        (0.70, 0.95),
        (0.88, 0.92),
        0.005,
        1.0,
        "Per-pass capture on absorber-inlet CO2",
    ),
    num(
        "srd",
        "Specific reboiler duty",
        "Capture",
        "GJ/t",
        "SRD_CENTRAL",
        (3.0, 5.0),
        (3.4, 4.2),
        0.05,
        1.0,
        "Decision D26 in docs/FINAL_DESIGN_DECISIONS.md",
    ),
    num(
        "p_strip_bar",
        "Stripper pressure",
        "Capture",
        "bar",
        "P_STRIPPER",
        (1.5, 2.0),
        (1.5, 2.0),
        0.05,
        1.0,
        "Sets the lean pump head in the Headline solve",
    ),
    num(
        "availability",
        "Availability",
        "Operation",
        "-",
        "AVAILABILITY",
        (0.60, 0.92),
        (0.65, 0.90),
        0.01,
        1.0,
        "Hydrogen counted only in availability hours",
    ),
    pick(
        "stage",
        "Cost stage",
        "Cost",
        Target::Stage,
        STAGE,
        "BOAK",
        "INL/RPT-23-72972",
        "The dashboard prints BOAK and FOAK side by side whatever is selected",
    ),
    pick(
        "life_y",
        "Reactor life",
        "Cost",
        Target::Life,
        LIFE,
        "25",
        "Engine registry LIVES_SHORT, LIVES_MID, LIVES_LONG",
        "",
    ),
    pick(
        "n",
        "Size exponent",
        "Cost",
        Target::Exponent,
        EXPONENT,
        "1.0",
        "Decision D09 in docs/FINAL_DESIGN_DECISIONS.md",
        "The cost table prints all three exponents at equal prominence",
    ),
    pick(
        "charged",
        "Reactor charged at",
        "Cost",
        Target::Charged,
        CHARGED,
        "rating",
        "Project screening model, reactor size charged",
        "",
    ),
    num(
        "discount",
        "Discount rate",
        "Cost",
        "-",
        "DISCOUNT",
        (0.03, 0.12),
        (0.05, 0.10),
        0.005,
        1.0,
        "Project basis",
    ),
    num(
        "gas_s_gj",
        "Natural gas price",
        "Cost",
        "S$/GJ",
        "GAS_CENTRAL",
        (8.0, 25.0),
        (10.0, 20.0),
        0.5,
        1.0,
        "",
    ),
    num(
        "ts_s_t",
        "CO2 transport and storage",
        "Cost",
        "S$/t",
        "TS_CENTRAL",
        (23.2, 112.0),
        (23.2, 112.0),
        0.1,
        1.0,
        "Decision D10 in docs/FINAL_DESIGN_DECISIONS.md",
    ),
    pick(
        "occ_level",
        "Reactor overnight cost",
        "Cost",
        Target::OccLevel,
        OCC,
        "central",
        "Engine registry OCC_LOW, OCC_CENTRAL, OCC_HIGH",
        "Selects which registry value the central cost query uses",
    ),
    num(
        "capture_fom",
        "Capture fixed O&M",
        "Cost",
        "1/y",
        "CAPTURE_FOM",
        (0.027, 0.06),
        (0.027, 0.06),
        0.001,
        1.0,
        "",
    ),
];

/// Section 11 rows with no control in this release, and why.
pub const DROPPED: &[(&str, &str)] = &[
    (
        "baseline",
        "Every solve returns abatement on the INL baseline with the Cho and IEAGHG bounds, so no selector is needed",
    ),
    (
        "backup_firing",
        "The engine has a backup firing estimate, yet its abatement and cost totals do not take a backup switch",
    ),
    (
        "t_he_supply_c",
        "The anchored heat balance does not read helium supply temperature; it is shown as a fixed value",
    ),
    (
        "circ_case",
        "The engine run uses the central circulator set; no run input selects another",
    ),
    (
        "credit_export",
        "The engine run always uses export steam in the reboiler heat order",
    ),
    (
        "credit_zld",
        "The engine run never counts ZLD steam as free",
    ),
    (
        "t_ref_out_c",
        "Reformer outlet acts only in the Level 3 flowsheet, which the Headline solve does not run",
    ),
    (
        "p_ref_bar",
        "Reformer pressure acts only in the Level 3 flowsheet",
    ),
    ("s_c", "Steam to carbon acts only in the Level 3 flowsheet"),
    ("t_hts_in_c", "HTS inlet acts only in the Level 3 flowsheet"),
    ("t_lts_in_c", "LTS inlet acts only in the Level 3 flowsheet"),
    (
        "wgs_approach_k",
        "Shift approach acts only in the Level 3 flowsheet",
    ),
    ("wgs_mode", "Shift arrangement acts only in the Level 3 flowsheet"),
    (
        "solvent_spec",
        "The Headline solve fixes capture and returns solvent rate; a rate-specified mode is absent",
    ),
    (
        "solvent_t_h",
        "Solvent rate is an output of the Headline solve",
    ),
];

/// Engine registry values shown without a control: (engine id, label, model
/// maximum for a flag, flag text).
pub const FIXED: &[(&str, &str, Option<f64>, &str)] = &[
    (
        "T_HE_SUPPLY",
        "Helium supply temperature",
        Some(859.0),
        "Highest measured secondary helium temperature is 859 C, HTTR 2004 high-temperature run (IAEA NP-T-4.3 printed p.14)",
    ),
    ("H2_B", "Hydrogen product at Case B", None, ""),
    ("T_HE_RETURN", "Helium return temperature", None, ""),
    ("ROT", "Reactor outlet temperature", None, ""),
    ("LOADING_RICH", "Rich loading cap", None, ""),
    (
        "PSA_H2_RECOVERY",
        "PSA H2 recovery, external block",
        None,
        "",
    ),
    ("ELECTRICITY_PRICE", "Electricity price", None, ""),
    ("GRID_EF", "Grid emission factor", None, ""),
    ("FX", "Exchange rate", None, ""),
];

/// Units of the live solve, one per selectable unit of the PFD (the 20 units
/// drawn in the edited Figure 2). Status is the rigour the engine registry
/// gives the input that sets the unit in the anchored port; `None` means the
/// unit is a balance the port computes, which the engine tags reduced order.
/// The last field names the screening term the unit carries, if any.
pub const UNITS: &[(&str, &str, Option<&str>, &str)] = &[
    ("htgr", "HTGR and helium loop", None, ""),
    ("circ_primary", "Helium circulators", None, "T11"),
    (
        "ihx",
        "Intermediate heat exchanger",
        Some("LOSS_CENTRAL_IHX"),
        "T12",
    ),
    ("power", "Power and steam block", Some("ETA_POWER"), ""),
    (
        "smr",
        "Helium-heated steam methane reformer",
        Some("Q_PROCESS"),
        "T03",
    ),
    ("wgs", "Water gas shift", Some("PHI_RECYCLE"), "T01"),
    (
        "syngas_cooler",
        "Syngas cooler and heat recovery",
        Some("Q_EX4_ABOVE_130C"),
        "T05",
    ),
    (
        "separator",
        "Condensate separator",
        Some("SYNGAS_CHAIN_C5_KO_DRM_1_MMBTU_H"),
        "",
    ),
    ("absorber", "MEA absorber", Some("ETA_MEA"), "T02"),
    ("lean_pump", "Lean amine pump", None, "T07"),
    ("lean_cooler", "Lean amine cooler", Some("K_COOLING"), "T08"),
    (
        "rich_lean_hx",
        "Rich and lean amine exchanger",
        Some("SRD_CENTRAL"),
        "",
    ),
    ("stripper", "MEA stripper", Some("SRD_CENTRAL"), "T06"),
    ("reboiler", "Reboiler", Some("SRD_CENTRAL"), "T06"),
    ("condenser", "Overhead condenser", Some("SRD_CENTRAL"), ""),
    ("co2_compressor", "CO2 compressor", Some("E_CO2COMP"), "T09"),
    (
        "co2_storage",
        "CO2 storage, ship offshore",
        Some("TS_CENTRAL"),
        "",
    ),
    ("psa", "PSA, external block", Some("PSA_H2_RECOVERY"), ""),
    (
        "recycle_compressor",
        "Recycle compressor",
        Some("E_PURGE"),
        "",
    ),
    ("cryogenic", "Cryogenic unit", Some("PHI_RECYCLE"), "T04"),
];

/// Map an engine rigour word onto the five status words of the contract.
pub fn status_word(rigour: &str) -> &'static str {
    match rigour {
        "rigorous" => "rigorous",
        "reduced_order" => "reduced_order",
        "empirical" => "empirical",
        "fixed_assumption" => "fixed_assumption",
        "external_boundary" => "external",
        _ => "fixed_assumption",
    }
}

pub fn spec(id: &str) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.id == id)
}

pub fn entry<'a>(p: &'a ParamSet, id: &str) -> Option<&'a ParamEntry> {
    p.entries().find(|e| e.id == id)
}

/// Default of a record in display units, or the enum default.
pub fn default_value(s: &Spec, p: &ParamSet) -> Value {
    match (s.kind, s.target) {
        (Kind::Number { scale, .. }, Target::Registry(id)) => match p.get(id) {
            Ok(v) => json!(v / scale),
            Err(_) => Value::Null,
        },
        (Kind::Enum { default, .. }, _) => json!(default),
        _ => Value::Null,
    }
}

fn option_label(s: &Spec, o: &Opt, m: Option<&Model>) -> String {
    match (s.target, m) {
        (Target::Exponent, Some(m)) => {
            let v = match o.value {
                "1.0" => m.n_linear,
                "0.6" => m.n_team,
                _ => m.n_ngnp,
            };
            format!("{}, engine value {:.4}", o.label, v)
        }
        _ => o.label.to_string(),
    }
}

/// The parameter record of the contract.
pub fn record(s: &Spec, p: &ParamSet, m: Option<&Model>) -> Value {
    let mut r = Map::new();
    r.insert("id".into(), json!(s.id));
    r.insert("label".into(), json!(s.label));
    r.insert("group".into(), json!(s.group));
    r.insert("unit".into(), json!(s.unit));
    r.insert("default".into(), default_value(s, p));
    match s.kind {
        Kind::Number {
            hard,
            model,
            step,
            scale,
        } => {
            r.insert("kind".into(), json!("number"));
            r.insert("hard_min".into(), json!(hard.0));
            r.insert("hard_max".into(), json!(hard.1));
            r.insert("model_min".into(), json!(model.0));
            r.insert("model_max".into(), json!(model.1));
            r.insert("step".into(), json!(step));
            r.insert("options".into(), Value::Null);
            r.insert("engine_scale".into(), json!(scale));
        }
        Kind::Enum { options, .. } => {
            r.insert("kind".into(), json!("enum"));
            for k in ["hard_min", "hard_max", "model_min", "model_max", "step"] {
                r.insert(k.into(), Value::Null);
            }
            let opts: Vec<Value> = options
                .iter()
                .map(|o| json!({"value": o.value, "label": option_label(s, o, m), "tag": o.tag}))
                .collect();
            r.insert("options".into(), Value::Array(opts));
        }
    }
    match s.target {
        Target::Registry(id) => {
            r.insert("engine_id".into(), json!(id));
            match entry(p, id) {
                Some(e) => {
                    r.insert("class".into(), json!(e.class));
                    r.insert(
                        "source".into(),
                        json!(format!("{}; {}", e.source, e.locator)),
                    );
                    r.insert("rigour".into(), json!(e.rigour));
                }
                None => {
                    r.insert("class".into(), Value::Null);
                    r.insert("source".into(), Value::Null);
                    r.insert("rigour".into(), Value::Null);
                }
            }
        }
        _ => {
            r.insert("engine_id".into(), Value::Null);
            r.insert("class".into(), json!(s.class));
            r.insert("source".into(), json!(s.source));
            r.insert("rigour".into(), Value::Null);
        }
    }
    r.insert("note".into(), json!(s.note));
    Value::Object(r)
}

/// Output keys with label and unit. The first block is the required set of
/// the contract, the second holds the cost and target extras.
pub const KPIS: &[(&str, &str, &str)] = &[
    ("rating_mwth", "HTGR rating", "MWth"),
    (
        "reactor_required_mwth",
        "Reactor heat the process needs",
        "MWth",
    ),
    ("surplus_mwth", "Thermal surplus at rating", "MWth"),
    (
        "surplus_ahn_mwth",
        "Thermal surplus on the Ahn heat basis",
        "MWth",
    ),
    ("unmet_heat_mwth", "Unmet thermal demand", "MWth"),
    ("unmet_power_mwe", "Grid import at rating", "MWe"),
    (
        "q_process_mwth",
        "SMR process heat, includes preheat",
        "MWth",
    ),
    ("h2_rich_kmol_h", "H2 in absorber overhead to PSA", "kmol/h"),
    ("h2_rich_kg_h", "H2 in absorber overhead to PSA", "kg/h"),
    (
        "h2_product_t_y",
        "Purified H2 after the external PSA",
        "t/y",
    ),
    ("ng_mmscfd", "Natural gas feed", "MMSCFD"),
    ("co2_captured_t_y", "CO2 captured", "t/y"),
    ("co2_atm_t_y", "CO2 to atmosphere", "t/y"),
    ("capture_per_pass", "Capture per pass", "-"),
    ("capture_plant", "Plant capture", "-"),
    ("q_reboiler_mw", "MEA regeneration duty", "MW"),
    ("reboiler_gap_mw", "Reboiler heat from reactor steam", "MW"),
    ("e_aux_mwe", "Auxiliary electricity", "MWe"),
    ("abatement_t_y", "Annual abatement, INL baseline", "t/y"),
    ("abatement_cho_t_y", "Annual abatement, Cho baseline", "t/y"),
    (
        "abatement_ieaghg_t_y",
        "Annual abatement, IEAGHG baseline",
        "t/y",
    ),
    (
        "abatement_system_t_y",
        "Annual abatement, system view",
        "t/y",
    ),
    (
        "abatement_lifecycle_t_y",
        "Annual abatement, lifecycle view with upstream gas",
        "t/y",
    ),
    (
        "cost_s_per_t",
        "Abatement cost, L5 ledger, selected stage, life and exponent",
        "S$/t",
    ),
    (
        "cost_l3_s_per_t",
        "Abatement cost at ladder step L3",
        "S$/t",
    ),
    (
        "cost_lac_dcf_s_per_t",
        "Levelised abatement cost, discounted cash flow",
        "S$/t",
    ),
    ("lcoh_nush2_s_per_kg", "LCOH of NUS-H2", "S$/kg"),
    ("lcoh_smr_s_per_kg", "LCOH of the unabated SMR", "S$/kg"),
    ("reactor_charged_mwth", "Reactor size charged", "MWth"),
    ("solvent_t_h", "Lean solvent rate", "t/h"),
    ("target_abatement_t_y", "Abatement target", "t/y"),
    ("target_cost_s_per_t", "Cost target", "S$/t"),
];

/// Number of required keys at the head of `KPIS`.
pub const REQUIRED_KPIS: usize = 24;
