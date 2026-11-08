import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "engine/wasm/src/solve.rs"

# units_section signature and helpers
rep(F, """    k: &BTreeMap<&'static str, f64>,
    ts_sm_y: f64,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let calc = "calculated";
    let folded = "the Headline solve folds this unit into the fixed specific reboiler duty and computes no duty for it";
    let mut out = Map::new();""", """    k: &BTreeMap<&'static str, f64>,
    ts_sm_y: f64,
    live: &Live,
    missing: &mut Vec<Value>,
) -> Value {
    let ks = |key: &str| k.get(key).copied().unwrap_or(f64::NAN);
    let calc = "calculated";
    let wgs = wgs_item(live);
    let island = |id: &str, label: &str, pick: fn(&IslandDuties) -> f64| match &live.island {
        Ok(d) => item(id, label, pick(d), "MW", calc, &island_src(base)),
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
                &format!("live Level 3 flowsheet {}, stream S10 from the flash", run.id),
            ),
            Err(e) => missing_item(
                "condensate",
                "Condensate flow",
                "t/h",
                &format!("stream S10 of the live Level 3 flowsheet {} is unreadable: {e}", run.id),
            ),
        },
        Err(why) => missing_item(
            "condensate",
            "Condensate flow",
            "t/h",
            &format!("the condensate comes from the live Level 3 flowsheet, and {why}"),
        ),
    };
    let mut out = Map::new();""")

rep(F, """                vec![item(
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
                )],""", """                vec![item(
                    "ng",
                    "Natural gas feed",
                    sol.ng_mmscfd,
                    "MMSCFD",
                    calc,
                    SRC,
                )],""")

rep(F, """                    missing_item(
                        "wgs_heat_release",
                        "Shift heat release",
                        "MW",
                        "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page",
                    ),
                ],
            ),""", """                    wgs.clone(),
                ],
            ),""")

rep(F, """                vec![missing_item("duty", "Condenser duty", "MW", folded)],""",
    """                vec![island("duty", "Condenser duty", |d| d.condenser_mw)],""")
rep(F, """                vec![missing_item("duty", "Exchanger duty", "MW", folded)],""",
    """                vec![island("duty", "Exchanger duty", |d| d.rich_lean_mw)],""")
rep(F, """                    missing_item(
                        "duty",
                        "Lean cooler duty",
                        "MW",
                        "the anchored port scales the MEA cooling load on the reboiler duty and computes no lean cooler duty",
                    ),""", """                    island("duty", "Lean cooler duty", |d| d.lean_cooler_mw),""")
rep(F, """                vec![missing_item(
                    "condensate",
                    "Condensate flow",
                    "t/h",
                    "the Headline solve runs no flash; condensate comes only from the Level 3 flowsheet",
                )],""", """                vec![condensate.clone()],""")

# balances
rep(F, """fn balances_section(
    b: &Balance,
    sol: &PlantSolution,
    k: &BTreeMap<&'static str, f64>,
    missing: &mut Vec<Value>,
) -> Value {""", """/// Shift heat release of the HTS and LTS beds of the live Level 3 flowsheet,
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
                &format!("the shift beds of the live Level 3 flowsheet {} are unreadable: {e}", run.id),
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
) -> Value {""")

rep(F, """        missing_item(
            "ihx_duty",
            "IHX duty",
            "MW",
            "the live solve returns no IHX duty; the native export row IHX_DUTY of Integrated_Energy_Balance.csv is in the precomputed table on the Results page",
        ),""", """        item(
            "ihx_duty",
            "IHX duty, process heat with slip heat",
            b.q_proc + b.q_recyc,
            "MW",
            c,
            SRC,
        ),""")
rep(F, """        missing_item(
            "wgs_heat_release",
            "WGS heat release",
            "MW",
            "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page",
        ),""", """        wgs_item(live),""")
rep(F, """        missing_item(
            "unmet_thermal",
            "Unmet thermal demand",
            "MWth",
            "the live solve returns no unmet thermal demand; the native export row UNMET_THERMAL of Integrated_Energy_Balance.csv is in the precomputed table on the Results page",
        ),""", """        item(
            "unmet_thermal",
            "Heat duty above the rated reactor",
            h.unmet,
            "MWth",
            c,
            SRC,
        ),""")
rep(F, """    missing.push(json!({"field": "balances.energy.ihx_duty", "reason": "the live solve returns no IHX duty; the native export row IHX_DUTY of Integrated_Energy_Balance.csv is in the precomputed table on the Results page"}));
    missing.push(json!({"field": "balances.energy.wgs_heat_release", "reason": "the live solve runs no Level 3 flowsheet; the native export fills WGS_RELEASE from run FS_B_REC_F, in the precomputed table on the Results page"}));
    missing.push(json!({"field": "balances.energy.unmet_thermal", "reason": "the live solve returns no unmet thermal demand; the native export row UNMET_THERMAL of Integrated_Energy_Balance.csv is in the precomputed table on the Results page"}));
""", """    for e in energy.iter().filter(|e| e["value"].is_null()) {
        missing.push(json!({
            "field": format!("balances.energy.{}", e["id"].as_str().unwrap_or("")),
            "reason": e["src"].clone(),
        }));
    }
""")
rep(F, """    missing.push(json!({"field": "balances.mass", "reason": "species mass balance comes from the Level 3 flowsheet, which the fast tier does not run"}));""",
    """    missing.push(json!({"field": "balances.mass", "reason": "the live reply takes the shift heat, the condensate and the train profiles from the Level 3 flowsheet; its species balance is precomputed in results/Integrated_Mass_Balance.csv"}));""")

# reply carries kpi_class and feasibility
rep(F, """            reply.insert("kpi".into(), Value::Object(out.kpi));
            reply.insert("engine_fields_missing".into(), Value::Array(out.missing));""", """            reply.insert("kpi".into(), Value::Object(out.kpi));
            reply.insert("kpi_class".into(), Value::Object(out.kpi_class));
            reply.insert("feasibility".into(), out.feasibility);
            reply.insert("engine_fields_missing".into(), Value::Array(out.missing));""")
rep(F, """            let declared: Vec<&str> = vec!["unmet_heat_mwth"];
            let ok = guard(&mut out.kpi, &declared, &mut out.messages);""", """            let ok = guard(&mut out.kpi, &[], &mut out.messages);""")
print("ok")
