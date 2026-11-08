//! The three commands. Each maps onto the public `nush2_plant` API on the
//! Headline preset and the fast tier: `run`, `Model::case_summary`,
//! `Model::balance`, `Model::emissions`, `Model::system_adjust`,
//! `Model::lifecycle_terms` and `cost_point`. Nothing here adds a model term;
//! the only arithmetic is the sum the engine exporter itself prints for the
//! system and lifecycle views (K.18, K.19) and the per-rung S$/t of the ladder
//! (L.26), each taken as the exporter writes it.

use crate::registry::{self, Kind, Spec, Target, DROPPED, FIXED, KPIS, SPECS, UNITS};
use nush2_numerics::EngineError;
use nush2_plant::anchored::{Balance, Case, Model, Scenario};
use nush2_plant::economics::{Stage, LEVELS};
use nush2_plant::flowsheet::{self, FlowsheetRun, TearStart};
use nush2_plant::heatrec::{self, SourceId};
use nush2_plant::params::ParamSet;
use nush2_plant::terms::{self, DesignIsland, IslandDuties, Source};
use nush2_plant::{cost_point, run, Inputs, PlantSolution, Preset, Tier};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const SCHEMA: u64 = 1;
pub const SWEEP_MAX: usize = 25;
/// Cross rule on the derived reactor outlet: helium supply plus IHX approach.
pub const ROT_MAX_C: f64 = 950.0;

pub fn engine() -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commit": option_env!("GIT_SHA").unwrap_or("local"),
        "preset": "headline",
        "tier": "fast",
    })
}

/// Reply for a request that never reached the engine.
pub fn refused(code: &str, text: &str) -> Value {
    json!({
        "schema": SCHEMA,
        "engine": engine(),
        "status": "rejected",
        "violations": [],
        "flags": [],
        "messages": [{"level": "error", "unit": "request", "code": code, "text": text}],
        "kpi": {},
    })
}

fn base_params() -> Result<ParamSet, EngineError> {
    ParamSet::embedded()
}

/// Hard and model range of a number record after the cross rules. The IHX
/// approach is capped so that helium supply plus approach stays at or below
/// 950 C, with the supply temperature read from the engine registry.
pub fn limits(s: &Spec, p: &ParamSet) -> Option<((f64, f64), (f64, f64))> {
    match (s.kind, s.target) {
        (Kind::Number { hard, model, .. }, Target::Registry(id)) => {
            let mut hard = hard;
            if id == "IHX_APPROACH_K" {
                if let Ok(t) = p.get("T_HE_SUPPLY") {
                    hard.1 = hard.1.min(ROT_MAX_C - t);
                }
            }
            let model = (model.0.clamp(hard.0, hard.1), model.1.clamp(hard.0, hard.1));
            Some((hard, model))
        }
        _ => None,
    }
}

// ------------------------------------------------------------------ registry

/// Fill the per-process caches that read no live input: the H2 return closure
/// behind the live Level 3 flowsheet and the M6 capture island at its design
/// point. The page asks for the registry once, at load, so the first solve
/// does not pay for them; every number stays the one the solve would compute.
/// A failure stays in its cache and reaches the solve that needs it.
pub fn warm_caches() {
    let _ = flowsheet::base_closure_cached();
    let _ = terms::design_island_cached();
}

/// Registry reply. It also warms the caches of `warm_caches`.
pub fn registry() -> Value {
    warm_caches();
    let p = match base_params() {
        Ok(p) => p,
        Err(e) => return refused("engine_registry", &e.to_string()),
    };
    let m = Model::new(&p).ok();
    let params: Vec<Value> = SPECS
        .iter()
        .map(|s| {
            let mut r = registry::record(s, &p, m.as_ref());
            if let (Some((hard, model)), Some(o)) = (limits(s, &p), r.as_object_mut()) {
                o.insert("hard_min".into(), json!(hard.0));
                o.insert("hard_max".into(), json!(hard.1));
                o.insert("model_min".into(), json!(model.0));
                o.insert("model_max".into(), json!(model.1));
            }
            r
        })
        .collect();
    let mut units = Map::new();
    for (id, label, driver, term) in UNITS {
        units.insert((*id).into(), unit_status(&p, label, *driver, term));
    }
    let kpis: Vec<Value> = KPIS
        .iter()
        .enumerate()
        .map(|(i, (id, label, unit))| {
            json!({"id": id, "label": label, "unit": unit, "required": i < registry::REQUIRED_KPIS})
        })
        .collect();
    let fixed: Vec<Value> = FIXED
        .iter()
        .map(|(id, label, model_max, text)| {
            let e = registry::entry(&p, id);
            json!({
                "id": id,
                "label": label,
                "value": p.get(id).ok(),
                "unit": e.map(|e| e.unit.clone()),
                "class": e.map(|e| e.class.clone()),
                "source": e.map(|e| format!("{}; {}", e.source, e.locator)),
                "model_max": model_max,
                "note": text,
            })
        })
        .collect();
    let dropped: Vec<Value> = DROPPED
        .iter()
        .map(|(id, why)| json!({"id": id, "reason": why}))
        .collect();
    json!({
        "schema": SCHEMA,
        "engine": engine(),
        "status": "ok",
        "params": params,
        "units": units,
        "kpis": kpis,
        "fixed": fixed,
        "dropped": dropped,
        "levels": LEVELS,
        "limits": {"sweep_max_values": SWEEP_MAX, "rot_max_c": ROT_MAX_C},
    })
}

fn unit_status(p: &ParamSet, label: &str, driver: Option<&str>, term: &str) -> Value {
    let (status, basis) = match driver {
        Some(id) => match registry::entry(p, id) {
            Some(e) => (
                registry::status_word(&e.rigour),
                format!("rigour of engine registry entry {id}"),
            ),
            None => (
                "fixed_assumption",
                format!("engine registry entry {id} absent"),
            ),
        },
        None => (
            "reduced_order",
            "balance computed by the anchored port, tagged reduced order by the engine exporter"
                .to_string(),
        ),
    };
    json!({
        "label": label,
        "status": status,
        "driver": driver,
        "term": if term.is_empty() { Value::Null } else { json!(term) },
        "basis": basis,
    })
}

// ------------------------------------------------------------------ inputs

/// Inputs after validation, ready for the engine.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub case: Case,
    pub scn: Scenario,
    pub ahn: bool,
    pub stage: Stage,
    pub life_ix: usize,
    pub n_ix: usize,
    pub required: bool,
    pub occ: &'static str,
    pub overrides: BTreeMap<String, f64>,
    pub echo: Map<String, Value>,
}

fn violation(param: &str, value: &Value, min: Option<f64>, max: Option<f64>, rule: &str) -> Value {
    json!({"param": param, "value": value, "min": min, "max": max, "rule": rule})
}

/// Check every input against its record. Returns the resolved inputs, or the
/// violations, plus model-range flags in both cases.
pub fn resolve(inputs: Option<&Value>, p: &ParamSet) -> (Option<Resolved>, Vec<Value>, Vec<Value>) {
    let mut violations = Vec::new();
    let mut flags = Vec::new();
    let empty = Map::new();
    let given = match inputs {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(o)) => o,
        Some(other) => {
            violations.push(violation(
                "inputs",
                other,
                None,
                None,
                "inputs must be an object of id to value",
            ));
            return (None, violations, flags);
        }
    };
    for (k, v) in given {
        if registry::spec(k).is_none() {
            violations.push(violation(k, v, None, None, "unknown parameter"));
        }
    }
    let mut r = Resolved {
        case: Case::B,
        scn: Scenario::RecF,
        ahn: false,
        stage: Stage::Boak,
        life_ix: 0,
        n_ix: 0,
        required: false,
        occ: "central",
        overrides: BTreeMap::new(),
        echo: Map::new(),
    };
    for s in SPECS {
        let v = match given.get(s.id) {
            Some(v) => v.clone(),
            None => registry::default_value(s, p),
        };
        match s.kind {
            Kind::Number {
                scale,
                model: evidence,
                ..
            } => {
                let x = match v.as_f64() {
                    Some(x) if x.is_finite() => x,
                    _ => {
                        violations.push(violation(s.id, &v, None, None, "must be a finite number"));
                        continue;
                    }
                };
                let (hard, model) = match limits(s, p) {
                    Some(l) => l,
                    None => continue,
                };
                if x < hard.0 || x > hard.1 {
                    let rule = if s.id == "ihx_approach_k" && x > hard.1 {
                        "hard_range; helium supply plus IHX approach must stay at or below 950 C"
                    } else {
                        "hard_range"
                    };
                    violations.push(violation(s.id, &v, Some(hard.0), Some(hard.1), rule));
                    continue;
                }
                if x < model.0 || x > model.1 {
                    // The text quotes the evidence range as published; the
                    // check uses it after the cross rules have capped it.
                    let capped = if model != evidence {
                        format!(
                            ", and the cross rules cap the control at {} {}",
                            hard.1, s.unit
                        )
                    } else {
                        String::new()
                    };
                    flags.push(json!({
                        "param": s.id,
                        "state": "outside_model_range",
                        "value": x,
                        "min": model.0,
                        "max": model.1,
                        "text": format!(
                            "{} at {} {} lies outside the evidence range {} to {} {}{}",
                            s.label, x, s.unit, evidence.0, evidence.1, s.unit, capped
                        ),
                    }));
                }
                if let Target::Registry(id) = s.target {
                    r.overrides.insert(id.to_string(), x * scale);
                }
                r.echo.insert(s.id.into(), json!(x));
            }
            Kind::Enum { options, .. } => {
                let opt = v
                    .as_str()
                    .and_then(|t| options.iter().find(|o| o.value == t));
                let Some(opt) = opt else {
                    violations.push(violation(s.id, &v, None, None, "not one of the options"));
                    continue;
                };
                let text: &'static str = opt.value;
                match s.target {
                    Target::Case => r.case = if text == "case_a" { Case::A } else { Case::B },
                    Target::Scenario => {
                        r.scn = match text {
                            "recycle_fixed" => Scenario::RecFix,
                            "burn_on_site" => Scenario::CombH,
                            "export" => Scenario::Export,
                            "inl_selexol" => Scenario::InlSel,
                            _ => Scenario::RecF,
                        }
                    }
                    Target::HeatBasis => r.ahn = text == "ahn_2026",
                    Target::Stage => {
                        r.stage = match text {
                            "FOAK" => Stage::Foak,
                            "10OAK_P46" => Stage::Oak10FromBoak,
                            "10OAK_P47" => Stage::Oak10FromFoak,
                            _ => Stage::Boak,
                        }
                    }
                    Target::Life => {
                        r.life_ix = match text {
                            "40" => 1,
                            "60" => 2,
                            _ => 0,
                        }
                    }
                    Target::Exponent => {
                        r.n_ix = match text {
                            "0.6" => 1,
                            "0.31" => 2,
                            _ => 0,
                        }
                    }
                    Target::Charged => r.required = text == "required",
                    Target::OccLevel => r.occ = text,
                    Target::Registry(_) => {}
                }
                r.echo.insert(s.id.into(), json!(text));
            }
        }
    }
    // Cross rule: lean loading stays below the rich cap and at or below 0.45.
    if let (Some(lean), Ok(rich)) = (r.overrides.get("LOADING_LEAN"), p.get("LOADING_RICH")) {
        if *lean > 0.45 || *lean >= rich {
            violations.push(violation(
                "lean_loading",
                &json!(lean),
                None,
                Some(0.45_f64.min(rich)),
                "lean loading must stay at or below 0.45 and below the rich cap",
            ));
        }
    }
    for (id, label, model_max, text) in FIXED {
        if let (Some(mx), Ok(v)) = (model_max, p.get(id)) {
            if v > *mx {
                let unit = registry::entry(p, id)
                    .map(|e| e.unit.clone())
                    .unwrap_or_default();
                flags.push(json!({
                    "param": id,
                    "state": "outside_model_range",
                    "value": v,
                    "min": Value::Null,
                    "max": mx,
                    "text": format!("{label} is fixed at {v} {unit}. {text}"),
                }));
            }
        }
    }
    if violations.is_empty() {
        (Some(r), violations, flags)
    } else {
        (None, violations, flags)
    }
}

// ------------------------------------------------------------------ engine

/// Overrides of the engine sensitivity HEAT_BASIS (sens.rs): process heat at
/// the Ahn Case B value, helium flow following duty.
fn ahn_overrides(m: &Model) -> [(&'static str, f64); 2] {
    let q_ahn_a = m.q_proc_ahn_b / m.s_b;
    [
        ("Q_PROCESS", q_ahn_a),
        ("HE_FLOW", m.he_flow * q_ahn_a / m.q_proc),
    ]
}

fn item(id: &str, label: &str, value: f64, unit: &str, class: &str, src: &str) -> Value {
    json!({"id": id, "label": label, "value": finite(value), "unit": unit, "class": class, "src": src})
}

fn missing_item(id: &str, label: &str, unit: &str, reason: &str) -> Value {
    json!({"id": id, "label": label, "value": Value::Null, "unit": unit, "class": Value::Null, "src": reason})
}

fn finite(v: f64) -> Value {
    if v.is_finite() {
        json!(v)
    } else {
        Value::Null
    }
}

fn reg_item(p: &ParamSet, id: &str, label: &str) -> Value {
    match registry::entry(p, id) {
        Some(e) => item(
            id,
            label,
            e.value,
            &e.unit,
            &e.class,
            &format!("{}; {}", e.source, e.locator),
        ),
        None => missing_item(id, label, "", "absent from the engine registry"),
    }
}

/// Everything one solve produces before it is wrapped in the reply.
pub struct Solved {
    pub kpi: Map<String, Value>,
    pub kpi_class: Map<String, Value>,
    pub feasibility: Value,
    pub missing: Vec<Value>,
    pub messages: Vec<Value>,
    pub targets: Value,
    pub sections: Option<Map<String, Value>>,
}

const SRC: &str = "nush2_plant anchored port";

/// Label of the power memo `import_at_rated`. Zero at or above the need; below
/// it the card J.29 value, which reads high when back-pressure work feeds a
/// reboiler gap (message `import_memo_high` gives the bound).
const IMPORT_MEMO_LABEL: &str = "Grid import if the reactor is held at its rating, a card J.29 \
     memo item that reads high below the need when back-pressure work feeds a reboiler gap";

/// Process order of the train profile, the order the Results page draws.
pub const TRAIN: [&str; 13] = [
    "S04", "S05", "S06", "S07", "S07A", "S08", "S09", "S09A", "S09B", "S09C", "S11", "S12", "S13",
];

/// What the full reply takes from models beyond the anchored port: the Level 3
/// flowsheet of the selected case and scenario, run live on the inputs, and the
/// capture island duties and column profiles of the M6 column model at its
/// Case B design point.
pub struct Live {
    pub fs: std::result::Result<FlowsheetRun, String>,
    pub island: std::result::Result<DesignIsland, String>,
}

/// Run the Level 3 flowsheet live for the selected case and scenario on the
/// registry without a preset, as the exporter runs its suite, with the H2
/// return closure of the embedded registry, which no live control moves.
pub fn live_flowsheet(inputs: &Inputs) -> std::result::Result<FlowsheetRun, String> {
    let p = inputs.base_params().map_err(|e| e.to_string())?;
    let m = Model::new(&p).map_err(|e| e.to_string())?;
    let closure = flowsheet::base_closure_cached().map_err(|e| e.to_string())?;
    let (id, spec) = flowsheet::base_spec(&m, inputs.case, inputs.scenario, closure.h2_return)
        .ok_or_else(|| {
            format!(
                "the Level 3 flowsheet covers the recycle and INL Selexol scenarios only, \
                 and the {} scenario is neither",
                inputs.scenario.id()
            )
        })?;
    flowsheet::solve(&m, &spec, &id, TearStart::Anchored)
        .map_err(|e| format!("the live Level 3 flowsheet {id} did not solve: {e}"))
}

/// Inputs the M6 column model fixes at its design point.
const ISLAND_INPUTS: [&str; 6] = [
    "W_MEA",
    "LOADING_LEAN",
    "LOADING_RICH",
    "ETA_MEA",
    "P_ABSORBER",
    "P_STRIPPER",
];

/// The design point the M6 column model runs at, in words, from the registry.
fn island_point(base: &ParamSet) -> String {
    let g = |id: &str| base.get(id).unwrap_or(f64::NAN);
    format!(
        "Case B recycle at {} kg/kg MEA, lean loading {}, rich cap {}, {} capture per pass, \
         absorber {} bar and stripper {} bar",
        g("W_MEA"),
        g("LOADING_LEAN"),
        g("LOADING_RICH"),
        g("ETA_MEA"),
        g("P_ABSORBER"),
        g("P_STRIPPER")
    )
}

/// Island duties and column profiles of the M6 column model when the solve sits
/// at its design point, or the reason they do not apply.
pub fn live_island(
    r: &Resolved,
    p: &ParamSet,
    base: &ParamSet,
) -> std::result::Result<DesignIsland, String> {
    let moved: Vec<&str> = ISLAND_INPUTS
        .iter()
        .copied()
        .filter(|id| match (p.get(id), base.get(id)) {
            (Ok(a), Ok(b)) => a != b,
            _ => true,
        })
        .collect();
    if r.case != Case::B || r.scn != Scenario::RecF || !moved.is_empty() {
        return Err(format!(
            "the M6 column model runs only at its design point, {}; this solve moves away from \
             it{}",
            island_point(base),
            if moved.is_empty() {
                String::new()
            } else {
                format!(" through {}", moved.join(", "))
            }
        ));
    }
    terms::design_island_cached().map_err(|e| format!("the M6 column model failed: {e}"))
}

/// Column profiles of the M6 design-point run: absorber slices by height above
/// the bottom of the packing, stripper stages from the top.
fn column_profile(
    island: &std::result::Result<DesignIsland, String>,
    key: &str,
    base: &ParamSet,
) -> std::result::Result<Value, String> {
    let d = island.as_ref().map_err(|e| e.clone())?;
    let points: Vec<Value> = if key == "absorber" {
        d.absorber
            .iter()
            .map(|s| {
                json!({
                    "z_m": finite(s.z_m),
                    "t_gas_c": finite(s.t_gas_k - 273.15),
                    "t_liq_c": finite(s.t_liq_k - 273.15),
                    "y_co2": finite(s.y_co2),
                    "loading": finite(s.alpha),
                })
            })
            .collect()
    } else {
        d.stripper
            .iter()
            .enumerate()
            .map(|(i, s)| {
                json!({
                    "stage": i + 1,
                    "t_c": finite(s.t_k - 273.15),
                    "loading": finite(s.alpha),
                    "y_co2": finite(s.y_co2),
                    "p_bar": finite(s.p_pa / 1.0e5),
                })
            })
            .collect()
    };
    if points.is_empty() {
        return Err(format!("the M6 {key} run returned no profile"));
    }
    Ok(json!({
        "class": "calculated",
        "src": island_src(base),
        "axis": if key == "absorber" {
            "z_m, height above the bottom of the packing"
        } else {
            "stage, counted from the top"
        },
        "units": if key == "absorber" {
            json!({"z_m": "m", "t_gas_c": "C", "t_liq_c": "C", "y_co2": "mol/mol", "loading": "mol CO2/mol MEA"})
        } else {
            json!({"stage": "-", "t_c": "C", "loading": "mol CO2/mol MEA", "y_co2": "mol/mol", "p_bar": "bar"})
        },
        "points": points,
    }))
}

/// Source text of an M6 island duty.
fn island_src(base: &ParamSet) -> String {
    format!(
        "M6 column model at its design point, {}, the M6_ISLAND rows of \
         Final_Design_Basis.csv; that run's reboiler works at the M6 specific duty, so \
         these duties sit beside its own reboiler row, M6_ISLAND_REBOILER_MW, and not \
         beside the anchored reboiler shown here; the M6 gate failed, so the Headline \
         preset keeps the columns anchored and the cooling load on the anchored K_COOLING",
        island_point(base)
    )
}

/// Whether the rated reactor runs the plant: thermal surplus at the rating at
/// or above zero and no heat duty above the rating.
pub fn feasibility(sol: &PlantSolution) -> (bool, Value) {
    let h = &sol.heat;
    let deficit = (-h.margin).max(0.0);
    let feasible = h.margin >= 0.0 && h.unmet <= 0.0;
    let reason = if feasible {
        format!(
            "Feasible: the {:.1} MWth rating covers the {:.1} MWth the process needs, with \
             {:.1} MWth to spare",
            h.r_rated, h.r_needed, h.margin
        )
    } else if h.unmet > 0.0 {
        format!(
            "Infeasible: the process needs {:.1} MWth and the rating is {:.1} MWth, a deficit \
             of {:.1} MWth. {:.1} MWth of heat duty sits above the rating, and grid import \
             supplies electricity only",
            h.r_needed, h.r_rated, deficit, h.unmet
        )
    } else {
        format!(
            "Infeasible: the process needs {:.1} MWth and the rating is {:.1} MWth, a deficit \
             of {:.1} MWth. The heat duties fit within the rating and the power branch falls \
             short, so {:.2} MWe is imported",
            h.r_needed, h.r_rated, deficit, sol.power.import_at_rated
        )
    };
    (
        feasible,
        json!({
            "feasible": feasible,
            "deficit_mwth": finite(deficit),
            "unmet_heat_mwth": finite(h.unmet),
            "surplus_mwth": finite(h.margin),
            "rating_mwth": finite(h.r_rated),
            "required_mwth": finite(h.r_needed),
            "reason": reason,
        }),
    )
}

/// One evaluated list of cost target cases: BOAK and FOAK, plus the selected
/// stage, at 25 y and 60 y plus the selected life, at every exponent, with the
/// rated reactor charged, and the selected case itself when it charges the
/// need. `table` marks the cells of the headline cost table and `selected`
/// the case of the dashboard strip, so both read the same verdict.
pub fn target_cases(
    m: &Model,
    b: &Balance,
    r: &Resolved,
    feasible: bool,
    messages: &mut Vec<Value>,
) -> Vec<Value> {
    let ns = [m.n_linear, m.n_team, m.n_ngnp];
    let life_sel = m.lives[r.life_ix];
    let mut stages = vec![Stage::Boak, Stage::Foak];
    if !stages.contains(&r.stage) {
        stages.push(r.stage);
    }
    let mut lives = vec![m.lives[0], m.lives[2]];
    if !lives.contains(&life_sel) {
        lives.push(life_sel);
    }
    let mut keys: Vec<(Stage, f64, usize, bool)> = Vec::new();
    for stage in &stages {
        for life in &lives {
            for n_ix in 0..ns.len() {
                keys.push((*stage, *life, n_ix, false));
            }
        }
    }
    let sel = (r.stage, life_sel, r.n_ix, r.required);
    if !keys.contains(&sel) {
        keys.push(sel);
    }
    let target = m.target_c;
    let mut out = Vec::new();
    for key in keys {
        let (stage, life, n_ix, required) = key;
        let r_ch = if required { b.r } else { m.r_rated };
        match cost_point(m, b, stage, life, ns[n_ix], r_ch) {
            Ok(c) => {
                let below = c.ac_l5 <= target && c.dcf.lac <= target;
                out.push(json!({
                    "stage": stage.id(),
                    "life_y": life,
                    "n": ns[n_ix],
                    "charged": if required { "required" } else { "rating" },
                    "charged_mwth": finite(r_ch),
                    "ac_l5": finite(c.ac_l5),
                    "lac_dcf": finite(c.dcf.lac),
                    "at_or_below_target": below,
                    "feasible": feasible,
                    "met": feasible && below,
                    "selected": key == sel,
                    "table": matches!(stage, Stage::Boak | Stage::Foak)
                        && (life == m.lives[0] || life == m.lives[2])
                        && !required,
                }));
            }
            Err(e) => messages.push(
                json!({"level": "warn", "unit": "cost", "code": "target_case",
                "text": format!("{} {} y n {}: {}", stage.id(), life, ns[n_ix], e)}),
            ),
        }
    }
    out
}

/// Class of a KPI: the registry class of the input it repeats, or calculated.
pub fn kpi_class(id: &str, r: &Resolved, base: &ParamSet) -> String {
    let from = match id {
        "rating_mwth" => Some("REACTOR_RATED_MWTH"),
        "capture_per_pass" if r.scn == Scenario::InlSel => Some("ETA_SELEXOL"),
        "capture_per_pass" => Some("ETA_MEA"),
        "target_abatement_t_y" => Some("TARGET_ABATEMENT"),
        "target_cost_s_per_t" => Some("TARGET_COST"),
        _ => None,
    };
    from.and_then(|e| registry::entry(base, e).map(|e| e.class.clone()))
        .unwrap_or_else(|| "calculated".to_string())
}

pub fn evaluate(r: &Resolved, base: &ParamSet, kpi_only: bool) -> Result<Solved, EngineError> {
    let mut overrides = r.overrides.clone();
    match r.occ {
        "low" => {
            overrides.insert("OCC_CENTRAL".into(), base.get("OCC_LOW")?);
        }
        "high" => {
            overrides.insert("OCC_CENTRAL".into(), base.get("OCC_HIGH")?);
        }
        _ => {}
    }
    let mut inputs = Inputs {
        case: r.case,
        scenario: r.scn,
        preset: Preset::Headline,
        tier: Tier::Fast,
        overrides,
    };
    let m0 = Model::new(&inputs.params()?)?;
    let ahn = ahn_overrides(&m0);
    if r.ahn {
        for (k, v) in ahn {
            inputs.overrides.insert(k.into(), v);
        }
    }
    let sol: PlantSolution = run(&inputs)?;
    let p = inputs.params()?;
    let m = Model::new(&p)?;
    let s = m.scale(r.case);
    let b: Balance = m.balance(r.scn, &m.opts(s))?;
    let k = m.case_summary(r.scn, s)?;
    let surplus_ahn = if r.ahn {
        sol.heat.margin
    } else {
        let mut i2 = inputs.clone();
        for (key, v) in ahn {
            i2.overrides.insert(key.into(), v);
        }
        run(&i2)?.heat.margin
    };

    let mut messages: Vec<Value> = Vec::new();
    let mut missing: Vec<Value> = Vec::new();
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);

    if (sol.heat.r_needed - b.r).abs() > 1e-9 * b.r.abs().max(1.0) {
        messages.push(json!({"level": "warn", "unit": "htgr", "code": "run_balance_mismatch",
            "text": format!("run gives {} MWth and balance gives {} MWth", sol.heat.r_needed, b.r)}));
    }
    for f in &sol.flags {
        messages.push(json!({"level": "info", "unit": "plant", "code": "engine_flag", "text": f}));
    }
    let anchored: Vec<&str> = sol
        .terms
        .iter()
        .filter(|t| t.used == Source::Anchored)
        .map(|t| t.id)
        .collect();
    if !anchored.is_empty() {
        messages.push(json!({"level": "info", "unit": "plant", "code": "anchored_terms",
            "text": format!("Terms on their anchored values in this preset: {}", anchored.join(", "))}));
    }

    let lives = m.lives;
    let ns = [m.n_linear, m.n_team, m.n_ngnp];
    let life = lives[r.life_ix];
    let n = ns[r.n_ix];
    let r_ch = if r.required { b.r } else { m.r_rated };
    if !r.required && m.r_rated < b.r {
        messages.push(json!({"level": "warn", "unit": "htgr", "code": "rating_below_need",
            "text": "The rating charged is below the heat the process needs, so reactor capital is understated"}));
    }
    // Card J.29 turns the back-pressure work of the power branch at the cycle
    // efficiency, where card B.19 counts it one to one, so below the need the
    // import reads high by up to w (1 - eta) when that work is nonzero.
    let wx = nush2_plant::export::branch_split(&m, &b).extraction_work;
    if sol.power.import_at_rated > 0.0 && wx > 0.0 {
        messages.push(
            json!({"level": "info", "unit": "power", "code": "import_memo_high",
            "text": format!(
                "The grid import below the need is the card J.29 memo. J.29 converts the {:.2} MWe \
                 of back-pressure work on the reboiler steam at the cycle efficiency, where card \
                 B.19 counts it one to one, so the import reads high by up to {:.2} MWe",
                wx,
                wx * (1.0 - m.eta_pwr)
            )}),
        );
    }
    let cp = cost_point(&m, &b, r.stage, life, n, r_ch)?;
    let e = sol.emissions;
    let sys = m.system_adjust(&b, s);
    let lc = m.lifecycle_terms(&b, s, b.r);
    let per_pass = if r.scn == Scenario::InlSel {
        m.eta_sel
    } else {
        m.eta_mea
    };
    let ng = sol.ng_mmscfd;

    let mut kpi = Map::new();
    let mut put = |key: &str, v: f64| {
        kpi.insert(key.into(), json!(v));
    };
    put("rating_mwth", sol.heat.r_rated);
    put("reactor_required_mwth", sol.heat.r_needed);
    put("surplus_mwth", sol.heat.margin);
    put("surplus_ahn_mwth", surplus_ahn);
    put("unmet_power_mwe", sol.power.import_at_rated);
    put("q_process_mwth", sol.heat.q_proc);
    put("h2_rich_kmol_h", ks("psa_h2_feed_kmol_h"));
    put("h2_rich_kg_h", ks("psa_h2_feed_kg_h"));
    put("h2_product_t_y", ks("h2_t_y"));
    put("ng_mmscfd", ng);
    put("co2_captured_t_y", e.captured);
    put("co2_atm_t_y", e.atm);
    put("capture_per_pass", per_pass);
    put("capture_plant", ks("plant_capture"));
    put("q_reboiler_mw", sol.heat.q_reb);
    put("reboiler_gap_mw", sol.heat.gap);
    put("e_aux_mwe", sol.power.e_total);
    put("abatement_t_y", e.a_atm);
    put("abatement_cho_t_y", ks("A_cho"));
    put("abatement_ieaghg_t_y", ks("A_ieaghg"));
    put("abatement_system_t_y", e.a_atm + sys);
    put(
        "abatement_lifecycle_t_y",
        e.a_atm + lc.up_credit - lc.nuclear - lc.ts_burden,
    );
    put("cost_s_per_t", cp.ac_l5);
    put("cost_l3_s_per_t", cp.ac_l3);
    put("cost_lac_dcf_s_per_t", cp.dcf.lac);
    put("lcoh_nush2_s_per_kg", cp.dcf.lcoh_nush2);
    put("lcoh_smr_s_per_kg", cp.dcf.lcoh_smr);
    put("reactor_charged_mwth", r_ch);
    put("solvent_t_h", ks("solvent_t_h"));
    put("target_abatement_t_y", m.target_a);
    put("target_cost_s_per_t", m.target_c);
    put("unmet_heat_mwth", sol.heat.unmet);
    let kpi_class: Map<String, Value> = kpi
        .keys()
        .map(|id| (id.clone(), json!(kpi_class(id, r, base))))
        .collect();

    let (feasible, feasibility) = feasibility(&sol);
    let below = cp.ac_l5 <= m.target_c && cp.dcf.lac <= m.target_c;
    let cases = target_cases(&m, &b, r, feasible, &mut messages);
    let selected_met = cases
        .iter()
        .find(|c| c["selected"] == json!(true))
        .and_then(|c| c["met"].as_bool())
        .unwrap_or(false);
    let cost_reason = if !feasible {
        format!(
            "Not met at these inputs. {}",
            feasibility["reason"].as_str().unwrap_or("")
        )
    } else if below {
        "Met at these inputs: the plant is feasible and both the L5 ledger and the DCF sit at \
         or below the target at the selected stage, life, exponent and size charged"
            .to_string()
    } else {
        format!(
            "Not met at these inputs: L5 {:.1} S$/t and DCF {:.1} S$/t against the {:.0} S$/t \
             target at the selected stage, life, exponent and size charged",
            cp.ac_l5, cp.dcf.lac, m.target_c
        )
    };
    let targets = json!({
        "abatement": {
            "value": finite(e.a_atm),
            "target": m.target_a,
            "met": feasible && e.a_atm >= m.target_a,
            "at_or_above_target": e.a_atm >= m.target_a,
            "feasible": feasible,
            "unit": "t/y",
        },
        "cost": {
            "value_l5": finite(cp.ac_l5),
            "value_lac_dcf": finite(cp.dcf.lac),
            "target": m.target_c,
            "met": feasible && below && selected_met,
            "at_or_below_target": below,
            "feasible": feasible,
            "reason": cost_reason,
            "unit": "S$/t",
            "basis": "L5 ledger and DCF levelised cost at the selected stage, life, exponent and size charged; met only when the plant is feasible at the rating and both sit at or below the target",
            "cases": cases,
        },
    });

    let sections = if kpi_only {
        None
    } else {
        let live = Live {
            fs: live_flowsheet(&inputs),
            island: live_island(r, &p, base),
        };
        let mut sec = Map::new();
        sec.insert("streams".into(), Value::Null);
        messages.push(json!({"level": "info", "unit": "plant", "code": "no_streams",
            "text": "The live reply returns no stream table; the Level 3 flowsheet streams are precomputed from results/Integrated_Mass_Balance.csv"}));
        missing.push(json!({"field": "streams", "reason": "the live reply carries the anchored balance; the stream table of the Level 3 flowsheet is precomputed in results/Integrated_Mass_Balance.csv"}));
        if let Err(why) = &live.fs {
            messages.push(
                json!({"level": "info", "unit": "plant", "code": "live_flowsheet",
                "text": format!("No live Level 3 flowsheet in this solve: {why}")}),
            );
        }
        let mut profiles = Map::new();
        profiles.insert("smr".into(), Value::Null);
        missing.push(json!({"field": "profiles.smr",
            "reason": format!("the reformer tube model runs only at a fixed Case B design point that reads no live input, and on the assumed tube it does not reach the {:.0} C exit of this solve's balance (rows TUBE_EXIT_C_N199 and TUBE_SHORTFALL_871C_K_N199 of results/Final_Design_Basis.csv), so its axial profile would describe a different reformer", flowsheet::T_REFORMER_OUT_C)}));
        for key in ["absorber", "stripper"] {
            match column_profile(&live.island, key, base) {
                Ok(v) => {
                    profiles.insert(key.into(), v);
                }
                Err(why) => {
                    profiles.insert(key.into(), Value::Null);
                    missing.push(json!({"field": format!("profiles.{key}"),
                        "reason": format!("the {key} profile comes from the M6 column model, and {why}")}));
                }
            }
        }
        for (key, unit) in [("train_T", "C"), ("train_P", "bar")] {
            match train_profile(&live.fs, key) {
                Ok(v) => {
                    profiles.insert(key.into(), v);
                }
                Err(why) => {
                    profiles.insert(key.into(), Value::Null);
                    missing.push(json!({"field": format!("profiles.{key}"),
                        "reason": format!("the train profile, in {unit}, comes from the live Level 3 flowsheet, and {why}")}));
                }
            }
        }
        sec.insert("profiles".into(), Value::Object(profiles));
        let ts_sm_y = cp
            .ledger
            .lines()
            .iter()
            .find(|(id, _, _)| *id == "TS")
            .map(|(_, _, v)| *v)
            .unwrap_or(f64::NAN);
        sec.insert(
            "units".into(),
            units_section(&p, base, &b, &sol, &k, ts_sm_y, &live, &mut missing),
        );
        sec.insert(
            "balances".into(),
            balances_section(&b, &sol, &k, &live, &mut missing),
        );
        sec.insert("cost".into(), cost_section(&m, &b, &cp, r, &mut messages));
        Some(sec)
    };
    Ok(Solved {
        kpi,
        kpi_class,
        feasibility,
        missing,
        messages,
        targets,
        sections,
    })
}

/// Train profile of the live Level 3 flowsheet in process order: temperature in
/// C for `train_T`, pressure in bar for `train_P`.
fn train_profile(
    fs: &std::result::Result<FlowsheetRun, String>,
    key: &str,
) -> std::result::Result<Value, String> {
    let run = fs.as_ref().map_err(|e| e.clone())?;
    let mut points = Vec::new();
    for tag in TRAIN {
        let s = run.gas(tag).map_err(|e| e.to_string())?;
        let v = if key == "train_T" {
            s.t_k - 273.15
        } else {
            s.p_pa / 1.0e5
        };
        points
            .push(json!({"stream": tag, "name": flowsheet::stream_name(tag), "value": finite(v)}));
    }
    Ok(json!({
        "unit": if key == "train_T" { "C" } else { "bar" },
        "class": "calculated",
        "run": run.id,
        "src": format!("live Level 3 flowsheet {}, streams in process order", run.id),
        "points": points,
    }))
}

#[allow(clippy::too_many_arguments)]
fn units_section(
    p: &ParamSet,
    base: &ParamSet,
    b: &Balance,
    sol: &PlantSolution,
    k: &BTreeMap<&'static str, f64>,
    ts_sm_y: f64,
    live: &Live,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let calc = "calculated";
    let wgs = wgs_item(live);
    let island = |id: &str, label: &str, pick: fn(&IslandDuties) -> f64| match &live.island {
        Ok(d) => item(id, label, pick(&d.duties), "MW", calc, &island_src(base)),
        Err(why) => missing_item(id, label, "MW", why),
    };
    let condensate = match &live.fs {
        Ok(run) => match run.gas("S10").and_then(|s| s.mass_kg_s()) {
            Ok(kg_s) => item(
                "condensate",
                "Condensate flow",
                kg_s * 3.6,
                "t/h",
                calc,
                &format!(
                    "live Level 3 flowsheet {}, stream S10 from the flash",
                    run.id
                ),
            ),
            Err(e) => missing_item(
                "condensate",
                "Condensate flow",
                "t/h",
                &format!(
                    "stream S10 of the live Level 3 flowsheet {} is unreadable: {e}",
                    run.id
                ),
            ),
        },
        Err(why) => missing_item(
            "condensate",
            "Condensate flow",
            "t/h",
            &format!("the condensate comes from the live Level 3 flowsheet, and {why}"),
        ),
    };
    let mut out = Map::new();
    for (id, label, driver, term) in UNITS {
        let mut u = unit_status(base, label, *driver, term);
        let t = sol.terms.iter().find(|t| t.id == *term);
        if let (Some(t), Some(o)) = (t, u.as_object_mut()) {
            o.insert(
                "used".into(),
                json!(if t.used == Source::Anchored {
                    "anchored"
                } else {
                    "modelled"
                }),
            );
            o.insert("gate".into(), json!(t.gate));
        }
        let (inputs, conditions, results): (Vec<Value>, Vec<Value>, Vec<Value>) = match *id {
            "htgr" => (
                vec![],
                vec![
                    reg_item(p, "T_HE_SUPPLY", "Helium supply"),
                    reg_item(p, "T_HE_RETURN", "Helium return"),
                    reg_item(p, "ROT", "Reactor outlet"),
                ],
                vec![
                    item(
                        "rating",
                        "Rating",
                        sol.heat.r_rated,
                        "MWth",
                        "user_decision",
                        "REACTOR_RATED_MWTH",
                    ),
                    item(
                        "required",
                        "Heat the process needs",
                        sol.heat.r_needed,
                        "MWth",
                        calc,
                        SRC,
                    ),
                    item(
                        "margin",
                        "Surplus at rating",
                        sol.heat.margin,
                        "MWth",
                        calc,
                        SRC,
                    ),
                    item(
                        "e_circ",
                        "Circulator electricity",
                        sol.power.e_circ,
                        "MWe",
                        calc,
                        SRC,
                    ),
                    item(
                        "he_prim",
                        "Primary helium flow at need",
                        ks("he_prim_kg_s_at_R"),
                        "kg/s",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "ihx" => (
                vec![],
                vec![reg_item(p, "IHX_APPROACH_K", "Approach")],
                vec![
                    item("q_ihx", "IHX loss", b.q_ihx, "MW", calc, SRC),
                    item("q_duct", "Duct loss", b.q_duct, "MW", calc, SRC),
                    item("q_vessel", "Vessel loss", b.qv, "MW", calc, SRC),
                ],
            ),
            "smr" => (
                vec![item(
                    "ng",
                    "Natural gas feed",
                    sol.ng_mmscfd,
                    "MMSCFD",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C",
                        "Reformer outlet, INL equilibrium assumption",
                    ),
                    reg_item(p, "T_HE_SUPPLY", "Helium supply"),
                    reg_item(p, "T_HE_RETURN", "Helium return"),
                ],
                vec![
                    item(
                        "q_proc",
                        "Process heat, includes preheat",
                        sol.heat.q_proc,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "q_recyc",
                        "Recycle slip heat",
                        sol.heat.q_recyc,
                        "MW",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "wgs" => (
                vec![],
                vec![
                    reg_item(p, "PHI_RECYCLE", "Carbon split after shift"),
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_HPS_GEN1_T_OUT_C",
                        "High-temperature shift inlet, INL Case 5",
                    ),
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_WGS_EX2_T_OUT_C",
                        "Low-temperature shift inlet, INL Case 5",
                    ),
                ],
                vec![
                    item(
                        "tube_c",
                        "Tube carbon",
                        ks("tube_C_kmol_h"),
                        "kmol/h",
                        calc,
                        SRC,
                    ),
                    wgs.clone(),
                ],
            ),
            "syngas_cooler" => (
                vec![],
                vec![
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C",
                        "Syngas inlet, INL Case 5",
                    ),
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_WGS_EX4_T_OUT_C",
                        "Last exchanger outlet, INL Case 5",
                    ),
                ],
                vec![
                    item(
                        "ex4",
                        "Syngas above 130 C to the reboiler",
                        sol.heat.stack.ex4,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "selexol",
                        "Freed Selexol steam",
                        sol.heat.stack.selexol,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "export",
                        "Export steam to the reboiler",
                        sol.heat.stack.export,
                        "MW",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "absorber" => (
                vec![item(
                    "co2_abs",
                    "CO2 entering",
                    ks("co2_abs_kmol_h"),
                    "kmol/h",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "P_ABSORBER", "Pressure"),
                    reg_item(p, "ETA_MEA", "Capture per pass"),
                    reg_item(p, "W_MEA", "MEA strength"),
                    reg_item(p, "LOADING_LEAN", "Lean loading"),
                    reg_item(p, "LOADING_RICH", "Rich loading cap"),
                ],
                vec![
                    item("cap", "CO2 captured", ks("cap_kmol_h"), "kmol/h", calc, SRC),
                    item("slip", "CO2 slip", ks("slip_kmol_h"), "kmol/h", calc, SRC),
                    item(
                        "solvent",
                        "Lean solvent rate",
                        ks("solvent_t_h"),
                        "t/h",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "stripper" => (
                vec![item(
                    "solvent",
                    "Rich solvent, at the lean rate",
                    ks("solvent_t_h"),
                    "t/h",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "P_STRIPPER", "Pressure"),
                    reg_item(p, "LOADING_LEAN", "Lean loading"),
                    item("srd", "Specific reboiler duty", b.srd, "GJ/t", calc, SRC),
                ],
                vec![
                    item("cap", "CO2 released", ks("cap_kmol_h"), "kmol/h", calc, SRC),
                    item("q_reb", "Reboiler duty", sol.heat.q_reb, "MW", calc, SRC),
                ],
            ),
            "reboiler" => (
                vec![item(
                    "cap_t_h",
                    "CO2 captured",
                    ks("cap_t_h"),
                    "t/h",
                    calc,
                    SRC,
                )],
                vec![item(
                    "srd",
                    "Specific reboiler duty",
                    b.srd,
                    "GJ/t",
                    calc,
                    SRC,
                )],
                vec![
                    item("q_reb", "Reboiler duty", sol.heat.q_reb, "MW", calc, SRC),
                    item(
                        "selexol",
                        "Freed Selexol steam",
                        sol.heat.stack.selexol,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "export",
                        "Export steam to the reboiler",
                        sol.heat.stack.export,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "ex4",
                        "Syngas above 130 C to the reboiler",
                        sol.heat.stack.ex4,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "avail",
                        "Recovered heat available",
                        sol.heat.avail,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "gap",
                        "Reactor steam to the reboiler",
                        sol.heat.gap,
                        "MW",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "condenser" => (
                vec![],
                vec![reg_item(p, "P_STRIPPER", "Pressure, stripper top")],
                vec![island("duty", "Condenser duty", |d| d.condenser_mw)],
            ),
            "rich_lean_hx" => (
                vec![item(
                    "solvent",
                    "Lean solvent rate",
                    ks("solvent_t_h"),
                    "t/h",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "LOADING_RICH", "Rich loading cap"),
                    reg_item(p, "LOADING_LEAN", "Lean loading"),
                ],
                vec![island("duty", "Exchanger duty", |d| d.rich_lean_mw)],
            ),
            "lean_pump" => (
                vec![item(
                    "solvent",
                    "Lean solvent rate",
                    ks("solvent_t_h"),
                    "t/h",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "P_STRIPPER", "Suction, stripper pressure"),
                    reg_item(p, "P_ABSORBER", "Discharge, absorber pressure"),
                    reg_item(p, "RHO_SOLVENT", "Solvent density"),
                    reg_item(p, "ETA_PUMP", "Pump efficiency"),
                    reg_item(p, "ETA_MOTOR", "Motor efficiency"),
                ],
                vec![item(
                    "e_pump",
                    "Lean pump electricity",
                    sol.power.rows.mea_pump,
                    "MWe",
                    calc,
                    SRC,
                )],
            ),
            "lean_cooler" => (
                vec![item(
                    "q_reb",
                    "Reboiler duty the cooling load scales on",
                    sol.heat.q_reb,
                    "MW",
                    calc,
                    SRC,
                )],
                vec![reg_item(
                    p,
                    "K_COOLING",
                    "Cooling electricity per MW rejected",
                )],
                vec![
                    item(
                        "e_cool",
                        "MEA cooling electricity",
                        sol.power.rows.mea_cooling,
                        "MWe",
                        calc,
                        SRC,
                    ),
                    island("duty", "Lean cooler duty", |d| d.lean_cooler_mw),
                ],
            ),
            "separator" => (
                vec![],
                vec![
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_KO_DRM_1_T_OUT_C",
                        "Separator temperature, INL Case 5",
                    ),
                    reg_item(p, "P_ABSORBER", "Pressure, taken as the absorber pressure"),
                    reg_item(
                        p,
                        "SYNGAS_CHAIN_C5_KO_DRM_1_MMBTU_H",
                        "Knock-out drum duty, INL Case 5, unscaled",
                    ),
                ],
                vec![condensate.clone()],
            ),
            "circ_primary" => (
                vec![
                    item(
                        "he_prim",
                        "Primary helium flow at need",
                        ks("he_prim_kg_s_at_R"),
                        "kg/s",
                        calc,
                        SRC,
                    ),
                    item(
                        "he_sec",
                        "Secondary helium flow",
                        ks("he_sec_kg_s"),
                        "kg/s",
                        calc,
                        SRC,
                    ),
                ],
                vec![
                    reg_item(p, "CIRC_CENTRAL_P_MPA", "Helium pressure"),
                    reg_item(p, "CIRC_CENTRAL_DP_PRIM_KPA", "Primary loop pressure drop"),
                    reg_item(p, "CIRC_CENTRAL_DP_SEC_KPA", "Secondary loop pressure drop"),
                    reg_item(p, "CIRC_CENTRAL_ETA", "Isentropic efficiency"),
                    reg_item(p, "ETA_MOTOR", "Motor efficiency"),
                    reg_item(p, "T_CIRC_PRIM_K", "Primary circulator inlet"),
                    reg_item(p, "T_CIRC_SEC_K", "Secondary circulator inlet"),
                ],
                vec![
                    item("w_p", "Primary shaft work", b.w_p, "MW", calc, SRC),
                    item("w_s", "Secondary shaft work", b.w_s, "MW", calc, SRC),
                    item(
                        "e_prim",
                        "Primary circulator electricity",
                        ks("circ_prim_MWe"),
                        "MWe",
                        calc,
                        SRC,
                    ),
                    item(
                        "e_sec",
                        "Secondary circulator electricity",
                        ks("circ_sec_MWe"),
                        "MWe",
                        calc,
                        SRC,
                    ),
                    item(
                        "e_circ",
                        "Circulator electricity",
                        sol.power.e_circ,
                        "MWe",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "power" => (
                vec![item(
                    "branch",
                    "Reactor heat to the power and steam branch",
                    b.branch,
                    "MWth",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "ETA_POWER", "Power conversion efficiency"),
                    reg_item(p, "R_WORK_CENTRAL", "Back-pressure work ratio"),
                ],
                vec![
                    item(
                        "gap",
                        "Reactor steam to the reboiler",
                        sol.heat.gap,
                        "MW",
                        calc,
                        SRC,
                    ),
                    item(
                        "e_total",
                        "Site electricity demand",
                        sol.power.e_total,
                        "MWe",
                        calc,
                        SRC,
                    ),
                    item(
                        "import_at_rated",
                        IMPORT_MEMO_LABEL,
                        sol.power.import_at_rated,
                        "MWe",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "co2_storage" => (
                vec![item(
                    "captured",
                    "CO2 shipped to storage",
                    sol.emissions.captured,
                    "t/y",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "TS_CENTRAL", "Transport and storage tariff"),
                    reg_item(p, "TS_BURDEN", "Ship fuel burden per tonne stored"),
                ],
                vec![
                    item(
                        "stored",
                        "CO2 delivered to storage",
                        sol.emissions.captured,
                        "t/y",
                        calc,
                        SRC,
                    ),
                    item(
                        "ts_cost",
                        "Transport and storage charge, ledger line TS",
                        ts_sm_y,
                        "S$M/y",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "recycle_compressor" => (
                vec![item(
                    "tailgas_c",
                    "PSA reject carbon",
                    ks("tailgas_C_kmol_h"),
                    "kmol/h",
                    calc,
                    SRC,
                )],
                vec![
                    reg_item(p, "E_PURGE", "INL purge recovery electricity, Case 6"),
                    reg_item(p, "PSA_REJECT_KMOL_H", "INL reject flow, Case 5"),
                ],
                vec![item(
                    "e_purge",
                    "Recycle and nitrogen removal electricity",
                    sol.power.rows.purge,
                    "MWe",
                    calc,
                    SRC,
                )],
            ),
            "psa" => (
                vec![item(
                    "h2_feed",
                    "H2 in feed",
                    ks("psa_h2_feed_kg_h"),
                    "kg/h",
                    calc,
                    SRC,
                )],
                vec![reg_item(p, "PSA_H2_RECOVERY", "H2 recovery, assumed")],
                vec![
                    item("h2_product", "H2 product", ks("h2_kg_h"), "kg/h", calc, SRC),
                    item(
                        "h2_reject",
                        "H2 in reject",
                        ks("psa_h2_reject_kg_h"),
                        "kg/h",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "cryogenic" => (
                vec![item(
                    "tailgas_c",
                    "Tail gas carbon",
                    ks("tailgas_C_kmol_h"),
                    "kmol/h",
                    calc,
                    SRC,
                )],
                vec![reg_item(p, "PHI_RECYCLE", "Carbon split after shift")],
                vec![
                    item(
                        "recycled_c",
                        "Carbon recycled to the reformer",
                        ks("recycled_C_kmol_h"),
                        "kmol/h",
                        calc,
                        SRC,
                    ),
                    item(
                        "burned_c",
                        "Carbon in the burned cut",
                        ks("burned_C_kmol_h"),
                        "kmol/h",
                        calc,
                        SRC,
                    ),
                    item(
                        "recycled_t_y",
                        "Recycled carbon as CO2",
                        ks("recycled_t_y"),
                        "t/y",
                        calc,
                        SRC,
                    ),
                ],
            ),
            "co2_compressor" => (
                vec![],
                vec![
                    reg_item(p, "E_CO2COMP", "INL compression electricity, Case 6"),
                    reg_item(p, "CO2_CAPTURED_C6", "INL captured CO2, Case 6"),
                ],
                vec![
                    item(
                        "e_co2",
                        "Compression electricity",
                        sol.power.rows.co2comp,
                        "MWe",
                        calc,
                        SRC,
                    ),
                    item(
                        "captured",
                        "CO2 compressed",
                        sol.emissions.captured,
                        "t/y",
                        calc,
                        SRC,
                    ),
                ],
            ),
            _ => (vec![], vec![], vec![]),
        };
        let outputs: Vec<Value> = match *id {
            "power" => vec![item(
                "e_nuc",
                "Electricity generated",
                sol.power.e_nuc,
                "MWe",
                calc,
                SRC,
            )],
            _ => vec![],
        };
        // A null value is an engine gap: list it with the reason the item carries.
        for (sec, list) in [
            ("inputs", &inputs),
            ("outputs", &outputs),
            ("conditions", &conditions),
            ("results", &results),
        ] {
            for e in list.iter().filter(|e| e["value"].is_null()) {
                missing.push(json!({
                    "field": format!("units.{}.{}.{}", id, sec, e["id"].as_str().unwrap_or("")),
                    "reason": e["src"].clone(),
                }));
            }
        }
        if let Some(o) = u.as_object_mut() {
            o.insert("inputs".into(), Value::Array(inputs));
            o.insert("outputs".into(), Value::Array(outputs));
            o.insert("conditions".into(), Value::Array(conditions));
            o.insert("results".into(), Value::Array(results));
        }
        out.insert((*id).into(), u);
    }
    Value::Object(out)
}

/// Shift heat release of the HTS and LTS beds of the live Level 3 flowsheet,
/// the quantity the export writes as WGS_RELEASE.
fn wgs_item(live: &Live) -> Value {
    let label = "Shift heat release";
    match &live.fs {
        Ok(run) => match nush2_plant::export::wgs_release_mw(run) {
            Ok(v) => item(
                "wgs_heat_release",
                label,
                v,
                "MW",
                "calculated",
                &format!(
                    "live Level 3 flowsheet {}, HTS and LTS beds at their inlet temperature",
                    run.id
                ),
            ),
            Err(e) => missing_item(
                "wgs_heat_release",
                label,
                "MW",
                &format!(
                    "the shift beds of the live Level 3 flowsheet {} are unreadable: {e}",
                    run.id
                ),
            ),
        },
        Err(why) => missing_item(
            "wgs_heat_release",
            label,
            "MW",
            &format!("the shift heat release comes from the live Level 3 flowsheet, and {why}"),
        ),
    }
}

fn balances_section(
    b: &Balance,
    sol: &PlantSolution,
    k: &BTreeMap<&'static str, f64>,
    live: &Live,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let c = "calculated";
    let h = &sol.heat;
    let w = &sol.power;
    // Reboiler draws in the approved heat order, as the export writes the
    // REB_FROM_* rows: zero from every source when no MEA reboiler runs, and
    // tail gas burned on site ahead of reactor steam.
    let draw = heatrec::allocate(h.q_reb, &h.stack);
    let from_tail_gas = (b.gap - b.gap_after_tg).max(0.0);
    // The operating point. The reactor runs at the need, or at its rating when
    // the rating is lower; it never runs above its rating. Below the need the
    // heat duties come first (UNMET_THERMAL), the rest of the rating feeds the
    // power branch, and the grid supplies what the branch cannot make
    // (UNMET_ELECTRICAL, J.29). At or above the need nothing is imported, and
    // `Model::shortfall_at` already gives zero there, so the KPI, the power
    // memo and this list read the same value.
    let output = h.r_needed.min(h.r_rated);
    let import = w.import_at_rated;
    let energy = vec![
        item(
            "htgr_heat_need",
            "Reactor heat at the need, the heat the process asks of the reactor",
            h.r_needed,
            "MWth",
            c,
            SRC,
        ),
        item(
            "htgr_heat_output",
            "Reactor heat output, at the need or at the rating when the rating is lower",
            output,
            "MWth",
            c,
            SRC,
        ),
        item(
            "unmet_reactor_heat",
            "Reactor heat the rating cannot supply, the need less the rating",
            (h.r_needed - h.r_rated).max(0.0),
            "MWth",
            c,
            SRC,
        ),
        item(
            "ihx_duty",
            "IHX duty, process heat with slip heat",
            b.q_proc + b.q_recyc,
            "MW",
            c,
            SRC,
        ),
        item("ihx_loss", "IHX loss", h.q_ihx, "MW", c, SRC),
        item("duct_loss", "Duct loss", h.q_duct, "MW", c, SRC),
        item("vessel_loss", "Vessel loss", h.qv, "MW", c, SRC),
        item(
            "smr_process_heat",
            "SMR process heat, includes preheat",
            h.q_proc,
            "MW",
            c,
            SRC,
        ),
        item(
            "recycle_slip_heat",
            "Recycle slip heat",
            h.q_recyc,
            "MW",
            c,
            SRC,
        ),
        wgs_item(live),
        item(
            "syngas_recovery_ex4",
            "Reboiler heat from syngas above 130 C",
            draw.from(SourceId::Syngas),
            "MW",
            c,
            SRC,
        ),
        item(
            "syngas_recovery_selexol",
            "Reboiler heat from freed Selexol steam",
            draw.from(SourceId::Selexol),
            "MW",
            c,
            SRC,
        ),
        item(
            "syngas_recovery_export",
            "Reboiler heat from export steam",
            draw.from(SourceId::Export),
            "MW",
            c,
            SRC,
        ),
        item(
            "mea_regeneration",
            "MEA regeneration duty",
            h.q_reb,
            "MW",
            c,
            SRC,
        ),
        item(
            "available_steam",
            "Recovered heat stack before the reboiler draw",
            h.avail,
            "MW",
            c,
            SRC,
        ),
        item(
            "tail_gas_to_reboiler",
            "Reboiler heat from tail gas burned on site",
            from_tail_gas,
            "MW",
            c,
            SRC,
        ),
        item(
            "reactor_steam_to_reboiler",
            "Reboiler heat from reactor steam, last",
            b.gap_after_tg,
            "MW",
            c,
            SRC,
        ),
        item(
            "electricity_generation",
            "Nuclear electricity generation at the reactor output",
            w.e_nuc - import,
            "MWe",
            c,
            SRC,
        ),
        item(
            "helium_circulation",
            "Helium circulation demand",
            w.e_circ,
            "MWe",
            c,
            SRC,
        ),
        item(
            "process_electricity",
            "Process electricity, pumps and compressors included",
            w.e_proc,
            "MWe",
            c,
            SRC,
        ),
        item(
            "cooling_utilities",
            "Cooling water electricity",
            w.rows.cooling,
            "MWe",
            c,
            SRC,
        ),
        item(
            "mea_cooling",
            "MEA cooling electricity",
            w.rows.mea_cooling,
            "MWe",
            c,
            SRC,
        ),
        item(
            "unmet_thermal",
            "Heat duty above the rated reactor",
            h.unmet,
            "MWth",
            c,
            SRC,
        ),
        item(
            "unmet_electrical",
            "Grid import at the reactor output, the electricity the power branch cannot make",
            import,
            "MWe",
            c,
            SRC,
        ),
    ];
    for e in energy.iter().filter(|e| e["value"].is_null()) {
        missing.push(json!({
            "field": format!("balances.energy.{}", e["id"].as_str().unwrap_or("")),
            "reason": e["src"].clone(),
        }));
    }
    let e = &w.rows;
    let power = vec![
        item("reforming", "Reforming", e.reforming, "MWe", c, SRC),
        item(
            "purification",
            "Purification",
            e.purification,
            "MWe",
            c,
            SRC,
        ),
        item("steam", "Steam system", e.steam, "MWe", c, SRC),
        item("co2comp", "CO2 compression", e.co2comp, "MWe", c, SRC),
        item("purge", "Purge recovery", e.purge, "MWe", c, SRC),
        item("cooling", "Cooling water", e.cooling, "MWe", c, SRC),
        item("water", "Water treatment", e.water, "MWe", c, SRC),
        item("mea_pump", "Lean pump", e.mea_pump, "MWe", c, SRC),
        item("mea_cooling", "MEA cooling", e.mea_cooling, "MWe", c, SRC),
        item("e_proc", "Process electricity", w.e_proc, "MWe", c, SRC),
        item("e_circ", "Circulators", w.e_circ, "MWe", c, SRC),
        item("e_total", "Auxiliary electricity", w.e_total, "MWe", c, SRC),
        item(
            "e_nuc",
            "Nuclear electricity generation at the need",
            w.e_nuc,
            "MWe",
            c,
            SRC,
        ),
        item(
            "import_at_rated",
            IMPORT_MEMO_LABEL,
            w.import_at_rated,
            "MWe",
            c,
            SRC,
        ),
    ];
    let carbon = vec![
        item(
            "feed_c",
            "Feed carbon",
            ks("feed_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "tube_c",
            "Tube carbon",
            ks("tube_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "co2_abs",
            "CO2 to the absorber",
            ks("co2_abs_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item("captured", "Captured", ks("cap_kmol_h"), "kmol/h", c, SRC),
        item("slip", "Absorber slip", ks("slip_kmol_h"), "kmol/h", c, SRC),
        item(
            "tailgas_c",
            "Tail gas carbon",
            ks("tailgas_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "recycled_c",
            "Recycled carbon",
            ks("recycled_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "burned_c",
            "Burned carbon",
            ks("burned_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "product_c",
            "Carbon in product",
            ks("product_C_kmol_h"),
            "kmol/h",
            c,
            SRC,
        ),
        item(
            "baseline_t_y",
            "Unabated SMR baseline",
            sol.emissions.baseline,
            "t/y",
            c,
            SRC,
        ),
        item(
            "captured_t_y",
            "Captured and stored",
            sol.emissions.captured,
            "t/y",
            c,
            SRC,
        ),
        item(
            "atm_t_y",
            "Residual to atmosphere",
            sol.emissions.atm,
            "t/y",
            c,
            SRC,
        ),
        item(
            "abatement_t_y",
            "Net abatement",
            sol.emissions.a_atm,
            "t/y",
            c,
            SRC,
        ),
    ];
    missing.push(json!({"field": "balances.mass", "reason": "the live reply takes the shift heat, the condensate and the train profiles from the Level 3 flowsheet; its species balance is precomputed in results/Integrated_Mass_Balance.csv"}));
    json!({
        "energy": energy,
        "power": power,
        "carbon": carbon,
        "mass": Value::Null,
        "closure": {
            "carbon_kmol_h": finite(b.c.closure),
            "basis": "Residual of the anchored carbon balance at INL scale",
        },
    })
}

fn level_rows(a: f64, ladder: &[f64; 6]) -> Vec<(usize, f64, f64)> {
    (0..6)
        .map(|i| (i, ladder[i], ladder[i] * 1e6 / a))
        .collect()
}

fn cost_section(
    m: &Model,
    b: &Balance,
    cp: &nush2_plant::CostPoint,
    r: &Resolved,
    messages: &mut Vec<Value>,
) -> Value {
    const NAMES: [(&str, &str); 16] = [
        ("GAS", "Gas saved against the unabated SMR"),
        ("REACTOR_CAP", "Reactor capital"),
        ("CAPTURE_CAP", "Capture capital"),
        ("CAPTURE_FOM", "Capture fixed O&M"),
        ("CAPTURE_ALLIN", "Capture all-in charge"),
        ("TS", "CO2 transport and storage"),
        ("REACTOR_OM", "Reactor O&M and fuel"),
        ("IHX", "IHX and secondary loop"),
        ("LICENCE", "Extra licensing"),
        ("DECOM", "Decommissioning"),
        ("IDC", "Interest during construction on the reactor"),
        ("POWER_ISLAND", "Power island add-back"),
        ("RECYCLE_TRAIN", "Recycle and cryogenic train"),
        ("LOST_STEAM", "Lost steam export"),
        ("BASE_POWER", "Baseline purchased power avoided"),
        ("EXPORT_REVENUE", "Export revenue"),
    ];
    fn name(id: &str) -> &str {
        NAMES
            .iter()
            .find(|(k, _)| *k == id)
            .map(|(_, v)| *v)
            .unwrap_or(id)
    }
    let a = m.emissions(&b.c, b.s).a_atm;
    let ledger: Vec<Value> = cp
        .ledger
        .lines()
        .iter()
        .map(|(id, rung, v)| json!({"id": id, "label": name(id), "level": LEVELS[*rung], "value": finite(*v), "unit": "S$M/y"}))
        .collect();
    let ladder: Vec<Value> = level_rows(a, &cp.ladder_sm_y)
        .into_iter()
        .map(|(i, sm, st)| json!({"level": LEVELS[i], "s_m_y": finite(sm), "s_per_t": finite(st)}))
        .collect();
    let ns = [m.n_linear, m.n_team, m.n_ngnp];
    let mut grid = Vec::new();
    let mut headline = Vec::new();
    for required in [false, true] {
        let r_ch = if required { b.r } else { m.r_rated };
        for stage in Stage::ALL {
            for life in m.lives {
                for n in ns {
                    match cost_point(m, b, stage, life, n, r_ch) {
                        Ok(c) => {
                            for (i, _, st) in level_rows(a, &c.ladder_sm_y) {
                                grid.push(json!({
                                    "charged": if required { "required" } else { "rating" },
                                    "charged_mwth": finite(r_ch),
                                    "n": n,
                                    "stage": stage.id(),
                                    "life_y": life,
                                    "level": LEVELS[i],
                                    "s_per_t": finite(st),
                                }));
                            }
                            headline.push(json!({
                                "charged": if required { "required" } else { "rating" },
                                "charged_mwth": finite(r_ch),
                                "n": n,
                                "stage": stage.id(),
                                "life_y": life,
                                "ac_l3": finite(c.ac_l3),
                                "ac_l5": finite(c.ac_l5),
                                "lac_dcf": finite(c.dcf.lac),
                                "lcoh_nush2": finite(c.dcf.lcoh_nush2),
                                "lcoh_smr": finite(c.dcf.lcoh_smr),
                                "dcf_gap_rel": finite(c.dcf.gap_rel),
                            }));
                        }
                        Err(e) => messages.push(
                            json!({"level": "warn", "unit": "cost", "code": "cost_point",
                            "text": format!("{} {} y n {}: {}", stage.id(), life, n, e)}),
                        ),
                    }
                }
            }
        }
    }
    json!({
        "selected": {
            "stage": r.stage.id(),
            "life_y": cp.life_y,
            "n": cp.n,
            "charged_mwth": finite(cp.r_charged),
            "ac_l3": finite(cp.ac_l3),
            "ac_l5": finite(cp.ac_l5),
            "dcf": {
                "lcoh_smr": finite(cp.dcf.lcoh_smr),
                "lcoh_nush2": finite(cp.dcf.lcoh_nush2),
                "lac": finite(cp.dcf.lac),
                "l5": finite(cp.dcf.l5),
                "gap_abs": finite(cp.dcf.gap_abs),
                "gap_rel": finite(cp.dcf.gap_rel),
            },
        },
        "abatement_t_y": finite(a),
        "ledger": ledger,
        "ladder": ladder,
        "grid": grid,
        "headline": headline,
    })
}

// ------------------------------------------------------------------ replies

fn engine_error(e: &EngineError) -> (&'static str, Vec<Value>, Vec<Value>) {
    match e {
        EngineError::Range { param, msg } => (
            "rejected",
            vec![violation(param, &Value::Null, None, None, msg)],
            vec![],
        ),
        EngineError::NotConverged(msg) => (
            "not_converged",
            vec![],
            vec![json!({"level": "error", "unit": "plant", "code": "not_converged", "text": msg})],
        ),
        EngineError::UnknownParam(id) => (
            "rejected",
            vec![violation(
                id,
                &Value::Null,
                None,
                None,
                "unknown engine parameter",
            )],
            vec![],
        ),
        EngineError::Parse(msg) => (
            "rejected",
            vec![],
            vec![json!({"level": "error", "unit": "plant", "code": "engine_parse", "text": msg})],
        ),
    }
}

/// Walk the KPI map and turn every non-finite value into null with a message.
fn guard(kpi: &mut Map<String, Value>, declared_null: &[&str], messages: &mut Vec<Value>) -> bool {
    let mut ok = true;
    for (key, v) in kpi.iter_mut() {
        if v.is_null() && !declared_null.contains(&key.as_str()) {
            ok = false;
            messages.push(
                json!({"level": "error", "unit": "plant", "code": "non_finite",
                "text": format!("{key} is not a finite number")}),
            );
        }
    }
    ok
}

fn solve_inner(body: &Value, base: &ParamSet, kpi_only_default: bool) -> Value {
    if let Some(sv) = body.get("schema") {
        if sv.as_u64() != Some(SCHEMA) {
            return refused("schema", "schema must be 1");
        }
    }
    let kpi_only = body
        .get("kpi_only")
        .and_then(Value::as_bool)
        .unwrap_or(kpi_only_default);
    let (resolved, violations, flags) = resolve(body.get("inputs"), base);
    let mut reply = Map::new();
    reply.insert("schema".into(), json!(SCHEMA));
    reply.insert("engine".into(), engine());
    reply.insert("flags".into(), Value::Array(flags));
    let Some(r) = resolved else {
        reply.insert("status".into(), json!("rejected"));
        reply.insert("violations".into(), Value::Array(violations));
        reply.insert("messages".into(), json!([]));
        reply.insert("kpi".into(), json!({}));
        reply.insert("engine_fields_missing".into(), json!([]));
        return Value::Object(reply);
    };
    reply.insert("inputs".into(), Value::Object(r.echo.clone()));
    match evaluate(&r, base, kpi_only) {
        Ok(mut out) => {
            let ok = guard(&mut out.kpi, &[], &mut out.messages);
            reply.insert(
                "status".into(),
                json!(if ok { "ok" } else { "not_converged" }),
            );
            reply.insert("violations".into(), json!([]));
            reply.insert("messages".into(), Value::Array(out.messages));
            reply.insert("kpi".into(), Value::Object(out.kpi));
            reply.insert("kpi_class".into(), Value::Object(out.kpi_class));
            reply.insert("feasibility".into(), out.feasibility);
            reply.insert("engine_fields_missing".into(), Value::Array(out.missing));
            reply.insert("targets".into(), out.targets);
            if let Some(sec) = out.sections {
                for (k, v) in sec {
                    reply.insert(k, v);
                }
            }
        }
        Err(e) => {
            let (status, v, msgs) = engine_error(&e);
            reply.insert("status".into(), json!(status));
            reply.insert("violations".into(), Value::Array(v));
            reply.insert("messages".into(), Value::Array(msgs));
            reply.insert("kpi".into(), json!({}));
            reply.insert("engine_fields_missing".into(), json!([]));
        }
    }
    Value::Object(reply)
}

pub fn solve(body: &Value) -> Value {
    match base_params() {
        Ok(p) => solve_inner(body, &p, false),
        Err(e) => refused("engine_registry", &e.to_string()),
    }
}

pub fn sweep(body: &Value) -> Value {
    let p = match base_params() {
        Ok(p) => p,
        Err(e) => return refused("engine_registry", &e.to_string()),
    };
    let Some(param) = body.get("param").and_then(Value::as_str) else {
        return refused("sweep_param", "param must name a registry parameter");
    };
    let Some(spec) = registry::spec(param) else {
        return refused("sweep_param", &format!("unknown parameter {param}"));
    };
    let Some(values) = body.get("values").and_then(Value::as_array) else {
        return refused("sweep_values", "values must be an array");
    };
    if values.is_empty() || values.len() > SWEEP_MAX {
        return refused(
            "sweep_values",
            &format!(
                "a sweep takes 1 to {SWEEP_MAX} values, got {}",
                values.len()
            ),
        );
    }
    let base_inputs = match body.get("inputs") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(o)) => o.clone(),
        Some(_) => return refused("sweep_inputs", "inputs must be an object"),
    };
    let mut rows = Vec::new();
    for x in values {
        let mut inputs = base_inputs.clone();
        inputs.insert(param.to_string(), x.clone());
        let one = solve_inner(&json!({"inputs": inputs, "kpi_only": true}), &p, true);
        // The engine verdicts travel with each row, so a consumer of the sweep
        // never reads a cost at an infeasible rating without its mark.
        let pick = |a: &str, b: &str, c: Option<&str>| {
            let v = &one[a][b];
            match c {
                Some(c) => v[c].clone(),
                None => v.clone(),
            }
        };
        rows.push(json!({
            "x": x,
            "status": one.get("status").cloned().unwrap_or(Value::Null),
            "kpi": one.get("kpi").cloned().unwrap_or(json!({})),
            "violations": one.get("violations").cloned().unwrap_or(json!([])),
            "feasible": pick("feasibility", "feasible", None),
            "cost_met": pick("targets", "cost", Some("met")),
            "abatement_met": pick("targets", "abatement", Some("met")),
        }));
    }
    json!({
        "schema": SCHEMA,
        "engine": engine(),
        "status": "ok",
        "param": param,
        "unit": spec.unit,
        "rows": rows,
    })
}
