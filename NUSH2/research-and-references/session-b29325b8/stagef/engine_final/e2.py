import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "engine/wasm/src/solve.rs"
rep(F, """    let ng = sol
        .terms
        .iter()
        .find(|t| t.id == "T13")
        .map(|t| t.anchored)
        .unwrap_or(f64::NAN);
""", """    let ng = sol.ng_mmscfd;
""")
rep(F, """    put("target_cost_s_per_t", m.target_c);
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
""", """    put("target_cost_s_per_t", m.target_c);
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
        "Met at these inputs: the plant is feasible and both the L5 ledger and the DCF sit at \\
         or below the target at the selected stage, life, exponent and size charged"
            .to_string()
    } else {
        format!(
            "Not met at these inputs: L5 {:.1} S$/t and DCF {:.1} S$/t against the {:.0} S$/t \\
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
""")
rep(F, """    let sections = if kpi_only {
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
""", """    let sections = if kpi_only {
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
            messages.push(json!({"level": "info", "unit": "plant", "code": "live_flowsheet",
                "text": format!("No live Level 3 flowsheet in this solve: {why}")}));
        }
        let mut profiles = Map::new();
        for key in ["smr", "absorber", "stripper"] {
            profiles.insert(key.into(), Value::Null);
            missing.push(json!({"field": format!("profiles.{key}"),
                "reason": "no axial profile runs inside the live solve: the reformer tube model and the column models take far longer than the solve, and the column models stay anchored because their gates failed"}));
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
""")
rep(F, """        sec.insert(
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
""", """        sec.insert(
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
        points.push(json!({"stream": tag, "name": flowsheet::stream_name(tag), "value": finite(v)}));
    }
    Ok(json!({
        "unit": if key == "train_T" { "C" } else { "bar" },
        "class": "calculated",
        "run": run.id,
        "src": format!("live Level 3 flowsheet {}, streams in process order", run.id),
        "points": points,
    }))
}
""")
rep(F, """pub struct Solved {
    pub kpi: Map<String, Value>,
    pub missing: Vec<Value>,""", """pub struct Solved {
    pub kpi: Map<String, Value>,
    pub kpi_class: Map<String, Value>,
    pub feasibility: Value,
    pub missing: Vec<Value>,""")
print("ok")
