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
use nush2_plant::params::ParamSet;
use nush2_plant::terms::Source;
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

pub fn registry() -> Value {
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
    pub missing: Vec<Value>,
    pub messages: Vec<Value>,
    pub targets: Value,
    pub sections: Option<Map<String, Value>>,
}

const SRC: &str = "nush2_plant anchored port";

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
    let cp = cost_point(&m, &b, r.stage, life, n, r_ch)?;
    let e = sol.emissions;
    let sys = m.system_adjust(&b, s);
    let lc = m.lifecycle_terms(&b, s, b.r);
    let per_pass = if r.scn == Scenario::InlSel {
        m.eta_sel
    } else {
        m.eta_mea
    };
    let ng = sol
        .terms
        .iter()
        .find(|t| t.id == "T13")
        .map(|t| t.anchored)
        .unwrap_or(f64::NAN);

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
    kpi.insert("unmet_heat_mwth".into(), Value::Null);
    missing.push(json!({"field": "kpi.unmet_heat_mwth",
        "reason": "The engine serves heat duties first and reports the shortfall as grid import; the live solve returns no unmet thermal demand, and the native export row UNMET_THERMAL is in the precomputed table on the Results page"}));

    let cost_met = cp.ac_l5 <= m.target_c && cp.dcf.lac <= m.target_c;
    let targets = json!({
        "abatement": {"value": finite(e.a_atm), "target": m.target_a, "met": e.a_atm >= m.target_a, "unit": "t/y"},
        "cost": {
            "value_l5": finite(cp.ac_l5),
            "value_lac_dcf": finite(cp.dcf.lac),
            "target": m.target_c,
            "met": cost_met,
            "unit": "S$/t",
            "basis": "L5 ledger and DCF levelised cost at the selected stage, life, exponent and size charged; met only when both sit at or below the target",
        },
    });

    let sections = if kpi_only {
        None
    } else {
        let mut sec = Map::new();
        sec.insert("streams".into(), Value::Null);
        messages.push(json!({"level": "info", "unit": "plant", "code": "no_streams",
            "text": "The Headline solve runs the anchored port, which has no stream table. The Level 3 flowsheet streams are precomputed from results/Integrated_Mass_Balance.csv"}));
        missing.push(json!({"field": "streams", "reason": "the fast tier does not run the Level 3 flowsheet"}));
        let mut profiles = Map::new();
        for key in ["smr", "absorber", "stripper", "train_T", "train_P"] {
            profiles.insert(key.into(), Value::Null);
            missing.push(json!({"field": format!("profiles.{key}"),
                "reason": "the anchored port carries no axial or train profile; column models stay anchored because their gates failed"}));
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
            units_section(&p, base, &b, &sol, &k, ts_sm_y, &mut missing),
        );
        sec.insert(
            "balances".into(),
            balances_section(&b, &sol, &k, &mut missing),
        );
        sec.insert("cost".into(), cost_section(&m, &b, &cp, r, &mut messages));
        Some(sec)
    };
    Ok(Solved {
        kpi,
        missing,
        messages,
        targets,
        sections,
    })
}

fn units_section(
    p: &ParamSet,
    base: &ParamSet,
    b: &Balance,
    sol: &PlantSolution,
    k: &BTreeMap<&'static str, f64>,
    ts_sm_y: f64,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let calc = "calculated";
    let folded = "the Headline solve folds this unit into the fixed specific reboiler duty and computes no duty for it";
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
                    sol.terms
                        .iter()
                        .find(|t| t.id == "T13")
                        .map(|t| t.anchored)
                        .unwrap_or(f64::NAN),
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
                    missing_item(
                        "wgs_heat_release",
                        "Shift heat release",
                        "MW",
                        "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page",
                    ),
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
                vec![missing_item("duty", "Condenser duty", "MW", folded)],
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
                vec![missing_item("duty", "Exchanger duty", "MW", folded)],
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
                    missing_item(
                        "duty",
                        "Lean cooler duty",
                        "MW",
                        "the anchored port scales the MEA cooling load on the reboiler duty and computes no lean cooler duty",
                    ),
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
                vec![missing_item(
                    "condensate",
                    "Condensate flow",
                    "t/h",
                    "the Headline solve runs no flash; condensate comes only from the Level 3 flowsheet",
                )],
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
                        "Grid import at rating",
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

fn balances_section(
    b: &Balance,
    sol: &PlantSolution,
    k: &BTreeMap<&'static str, f64>,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let c = "calculated";
    let h = &sol.heat;
    let w = &sol.power;
    let energy = vec![
        item(
            "htgr_heat_output",
            "HTGR heat output at the operating point",
            h.r_needed,
            "MWth",
            c,
            SRC,
        ),
        missing_item(
            "ihx_duty",
            "IHX duty",
            "MW",
            "the live solve returns no IHX duty; the native export row IHX_DUTY of Integrated_Energy_Balance.csv is in the precomputed table on the Results page",
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
        missing_item(
            "wgs_heat_release",
            "WGS heat release",
            "MW",
            "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page",
        ),
        item(
            "syngas_recovery_ex4",
            "Syngas heat above 130 C to the reboiler",
            h.stack.ex4,
            "MW",
            c,
            SRC,
        ),
        item(
            "syngas_recovery_selexol",
            "Freed Selexol steam",
            h.stack.selexol,
            "MW",
            c,
            SRC,
        ),
        item(
            "syngas_recovery_export",
            "Export steam to the reboiler",
            h.stack.export,
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
            "Recovered steam available to the reboiler",
            h.avail,
            "MW",
            c,
            SRC,
        ),
        item(
            "reactor_steam_to_reboiler",
            "Reactor steam to the reboiler",
            h.gap,
            "MW",
            c,
            SRC,
        ),
        item(
            "electricity_generation",
            "Nuclear electricity generation",
            w.e_nuc,
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
        missing_item(
            "unmet_thermal",
            "Unmet thermal demand",
            "MWth",
            "the live solve returns no unmet thermal demand; the native export row UNMET_THERMAL of Integrated_Energy_Balance.csv is in the precomputed table on the Results page",
        ),
        item(
            "unmet_electrical",
            "Grid import at rating",
            w.import_at_rated,
            "MWe",
            c,
            SRC,
        ),
    ];
    missing.push(json!({"field": "balances.energy.ihx_duty", "reason": "the live solve returns no IHX duty; the native export row IHX_DUTY of Integrated_Energy_Balance.csv is in the precomputed table on the Results page"}));
    missing.push(json!({"field": "balances.energy.wgs_heat_release", "reason": "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page"}));
    missing.push(json!({"field": "balances.energy.unmet_thermal", "reason": "the live solve returns no unmet thermal demand; the native export row UNMET_THERMAL of Integrated_Energy_Balance.csv is in the precomputed table on the Results page"}));
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
            "Nuclear electricity generation",
            w.e_nuc,
            "MWe",
            c,
            SRC,
        ),
        item(
            "import_at_rated",
            "Grid import at rating",
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
    missing.push(json!({"field": "balances.mass", "reason": "species mass balance comes from the Level 3 flowsheet, which the fast tier does not run"}));
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
            let declared: Vec<&str> = vec!["unmet_heat_mwth"];
            let ok = guard(&mut out.kpi, &declared, &mut out.messages);
            reply.insert(
                "status".into(),
                json!(if ok { "ok" } else { "not_converged" }),
            );
            reply.insert("violations".into(), json!([]));
            reply.insert("messages".into(), Value::Array(out.messages));
            reply.insert("kpi".into(), Value::Object(out.kpi));
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
        rows.push(json!({
            "x": x,
            "status": one.get("status").cloned().unwrap_or(Value::Null),
            "kpi": one.get("kpi").cloned().unwrap_or(json!({})),
            "violations": one.get("violations").cloned().unwrap_or(json!([])),
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
