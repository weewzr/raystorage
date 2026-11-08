import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "engine/wasm/src/tests.rs"

rep(F, """    assert!(kpi["unmet_heat_mwth"].is_null());
    assert!(missing.contains(&"kpi.unmet_heat_mwth"));
    assert!(r["streams"].is_null());""", """    assert_eq!(kpi["unmet_heat_mwth"], json!(0.0));
    assert!(!missing.contains(&"kpi.unmet_heat_mwth"));
    let classes = obj(&r["kpi_class"]);
    for (k, _) in kpi {
        let c = classes.get(k).and_then(Value::as_str);
        assert!(
            c.is_some_and(|c| !c.is_empty() && c != "kpi"),
            "{k} has no class"
        );
    }
    assert_eq!(classes["rating_mwth"], "user_decision");
    assert_eq!(classes["cost_s_per_t"], "calculated");
    assert_eq!(r["feasibility"]["feasible"], true);
    assert_eq!(r["feasibility"]["deficit_mwth"], json!(0.0));
    for f in [
        "balances.energy.ihx_duty",
        "balances.energy.wgs_heat_release",
        "balances.energy.unmet_thermal",
        "units.wgs.results.wgs_heat_release",
        "units.separator.results.condensate",
        "units.lean_cooler.results.duty",
        "units.rich_lean_hx.results.duty",
        "units.condenser.results.duty",
        "profiles.train_T",
        "profiles.train_P",
    ] {
        assert!(!missing.contains(&f), "{f} is still missing");
    }
    assert!(r["streams"].is_null());""")

rep(F, """    for (_, p) in obj(&r["profiles"]) {
        assert!(p.is_null());
    }""", """    for key in ["smr", "absorber", "stripper"] {
        assert!(r["profiles"][key].is_null());
        assert!(missing.contains(&format!("profiles.{key}").as_str()));
    }
    for key in ["train_T", "train_P"] {
        let pts = r["profiles"][key]["points"].as_array().expect("points");
        assert_eq!(pts.len(), crate::solve::TRAIN.len());
    }""")

rep(F, """    assert_eq!(r["targets"]["cost"]["met"], false);
}""", """    assert_eq!(r["targets"]["cost"]["met"], false);
    assert_eq!(r["targets"]["cost"]["feasible"], true);
    let cases = r["targets"]["cost"]["cases"].as_array().unwrap();
    assert_eq!(
        cases.iter().filter(|c| c["table"] == true).count(),
        2 * 2 * 3,
        "BOAK and FOAK, 25 and 60 y, three exponents"
    );
    assert_eq!(cases.iter().filter(|c| c["selected"] == true).count(), 1);
}""")

# Case B parity test: the live fields from the flowsheet and the island match the export.
rep(F, """    // The headline table at n = 1.0, BOAK and FOAK, 25 y and 60 y, rating charged.""", """    let energy = |id: &str| {
        r["balances"]["energy"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .and_then(|e| e["value"].as_f64())
            .unwrap_or_else(|| panic!("balances.energy.{id}"))
    };
    let eb = |term: &str| {
        csv_value(
            "Integrated_Energy_Balance.csv",
            &[(0, "B_REC_F"), (1, term)],
            5,
        )
    };
    // The CSV prints six decimals.
    let near = |a: f64, b: f64, what: &str| assert!((a - b).abs() <= 1e-6, "{what}: {a} {b}");
    near(energy("ihx_duty"), eb("IHX_DUTY"), "IHX duty");
    near(energy("wgs_heat_release"), eb("WGS_RELEASE"), "shift heat");
    near(energy("unmet_thermal"), eb("UNMET_THERMAL"), "unmet heat");
    near(
        k["unmet_heat_mwth"].as_f64().unwrap(),
        eb("UNMET_THERMAL"),
        "unmet heat KPI",
    );
    let unit_value = |u: &str, id: &str| {
        r["units"][u]["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .and_then(|e| e["value"].as_f64())
            .unwrap_or_else(|| panic!("units.{u}.results.{id}"))
    };
    let mb = |stream: &str, col: usize| {
        csv_value(
            "Integrated_Mass_Balance.csv",
            &[(0, "FS_B_REC_F"), (1, stream)],
            col,
        )
    };
    let s10 = mb("S10", 8);
    assert!(
        (unit_value("separator", "condensate") * 1000.0 - s10).abs() <= 1e-6 * s10,
        "condensate"
    );
    for (key, col) in [("train_T", 6), ("train_P", 7)] {
        for p in r["profiles"][key]["points"].as_array().unwrap() {
            let s = p["stream"].as_str().unwrap();
            near(p["value"].as_f64().unwrap(), mb(s, col), &format!("{key} {s}"));
        }
    }
    let d = nush2_plant::terms::design_duties_cached().unwrap();
    assert_eq!(unit_value("lean_cooler", "duty"), d.lean_cooler_mw);
    assert_eq!(unit_value("rich_lean_hx", "duty"), d.rich_lean_mw);
    assert_eq!(unit_value("condenser", "duty"), d.condenser_mw);
    // The headline table at n = 1.0, BOAK and FOAK, 25 y and 60 y, rating charged.""")

# New tests appended at the end of the file.
p = __import__("pathlib").Path(F)
s = p.read_bytes().decode("utf-8")
s = s.rstrip("\n") + "\n" + '''
/// Every KPI of the live Case A solve that has a case_a_mea cell in
/// Final_Design_Basis.csv equals that cell. The KPIs without one are listed.
#[test]
fn case_a_kpis_match_the_case_a_mea_cells() {
    const MAP: [(&str, &str); 21] = [
        ("rating_mwth", "REACTOR_RATED_MWTH"),
        ("reactor_required_mwth", "RES_REACTOR_NEEDED_MWTH"),
        ("surplus_mwth", "RES_MARGIN_210_MWTH"),
        ("unmet_power_mwe", "RES_GRID_IMPORT_AT_RATED_MWE"),
        ("q_process_mwth", "RES_PROCESS_HEAT_MW"),
        ("h2_rich_kg_h", "RES_PSA_H2_FEED_KG_H"),
        ("h2_product_t_y", "RES_H2_T_Y"),
        ("ng_mmscfd", "RES_NG_MMSCFD"),
        ("co2_captured_t_y", "RES_CAPTURED_T_Y"),
        ("co2_atm_t_y", "RES_ATM_T_Y"),
        ("capture_per_pass", "ETA_MEA"),
        ("capture_plant", "RES_PLANT_CAPTURE"),
        ("q_reboiler_mw", "RES_REBOILER_MW"),
        ("reboiler_gap_mw", "RES_REBOILER_GAP_MW"),
        ("e_aux_mwe", "RES_E_TOTAL_MWE"),
        ("abatement_t_y", "RES_ABATEMENT_T_Y"),
        ("abatement_cho_t_y", "RES_ABATEMENT_CHO_T_Y"),
        ("abatement_ieaghg_t_y", "RES_ABATEMENT_IEAGHG_T_Y"),
        ("solvent_t_h", "RES_SOLVENT_T_H"),
        ("target_abatement_t_y", "TARGET_ABATEMENT"),
        ("target_cost_s_per_t", "TARGET_COST"),
    ];
    // KPIs with no case_a_mea cell: their rows in the results are Case B only,
    // or they live in another file.
    const NO_CELL: [&str; 11] = [
        "surplus_ahn_mwth",
        "unmet_heat_mwth",
        "h2_rich_kmol_h",
        "abatement_system_t_y",
        "abatement_lifecycle_t_y",
        "cost_s_per_t",
        "cost_l3_s_per_t",
        "cost_lac_dcf_s_per_t",
        "lcoh_nush2_s_per_kg",
        "lcoh_smr_s_per_kg",
        "reactor_charged_mwth",
    ];
    let r = solve_with(json!({"basis": "case_a"}), true);
    assert_eq!(r["status"], "ok", "{}", r["messages"]);
    let rows = csv_rows("Final_Design_Basis.csv");
    let head = &rows[0];
    let col = head.iter().position(|h| h == "case_a_mea").unwrap();
    let mut compared = 0;
    for (kpi, id) in MAP {
        let row = rows
            .iter()
            .find(|row| row[0] == id)
            .unwrap_or_else(|| panic!("no row {id}"));
        let want: f64 = row[col]
            .parse()
            .unwrap_or_else(|_| panic!("{id} has no case_a_mea cell"));
        close(r["kpi"][kpi].as_f64().unwrap(), want, kpi);
        compared += 1;
    }
    assert_eq!(compared, MAP.len());
    for (k, _) in obj(&r["kpi"]) {
        assert!(
            MAP.iter().any(|(kpi, _)| kpi == k) || NO_CELL.contains(&k.as_str()),
            "{k} is neither compared nor listed"
        );
    }
    // Every result row with a case_a_mea cell that a KPI repeats is compared.
    for (kpi, id) in MAP {
        assert!(rows.iter().any(|row| row[0] == id && !row[col].is_empty()), "{kpi}");
    }
    // Case A at the 210 MWth rating needs more heat than the rating.
    assert_eq!(r["feasibility"]["feasible"], false);
    assert_eq!(r["targets"]["cost"]["met"], false);
}

/// At 100 MWth the plant cannot run: the reply says so and why, carries the
/// unmet heat of the export function, and no target case reads met.
#[test]
fn rating_100_is_infeasible_and_no_case_is_met() {
    for kpi_only in [true, false] {
        let r = solve_with(json!({"rating_mwth": 100}), kpi_only);
        assert_eq!(r["status"], "ok", "{}", r["messages"]);
        let f = &r["feasibility"];
        assert_eq!(f["feasible"], false);
        let need = r["kpi"]["reactor_required_mwth"].as_f64().unwrap();
        close(f["deficit_mwth"].as_f64().unwrap(), need - 100.0, "deficit");
        assert!(f["reason"].as_str().unwrap().starts_with("Infeasible"));
        // The unmet heat is the export function at this rating.
        let mut i = nush2_plant::Inputs::headline();
        i.overrides.insert("REACTOR_RATED_MWTH".into(), 100.0);
        let m = nush2_plant::anchored::Model::new(&i.params().unwrap()).unwrap();
        let b = m
            .balance(
                nush2_plant::anchored::Scenario::RecF,
                &m.opts(m.s_b),
            )
            .unwrap();
        let unmet = m.unmet_thermal(100.0, &b);
        assert!(unmet > 0.0);
        assert_eq!(r["kpi"]["unmet_heat_mwth"].as_f64().unwrap(), unmet);
        assert_eq!(f["unmet_heat_mwth"].as_f64().unwrap(), unmet);
        let cost = &r["targets"]["cost"];
        assert_eq!(cost["met"], false);
        assert_eq!(cost["feasible"], false);
        assert!(cost["reason"].as_str().unwrap().contains("Infeasible"));
        for c in cost["cases"].as_array().unwrap() {
            assert_eq!(c["met"], false, "{c}");
            assert_eq!(c["feasible"], false);
        }
        assert_eq!(r["targets"]["abatement"]["met"], false);
    }
}

/// With stage 10OAK_P46 and gas at S$25/GJ the selected case sits below the
/// target; the case list carries it with the same verdict as the strip, and
/// every verdict follows feasibility and both methods.
#[test]
fn ten_oak_at_25_gas_reads_one_verdict() {
    let r = solve_with(json!({"stage": "10OAK_P46", "gas_s_gj": 25}), true);
    assert_eq!(r["status"], "ok", "{}", r["messages"]);
    let cost = &r["targets"]["cost"];
    let t = cost["target"].as_f64().unwrap();
    let cases = cost["cases"].as_array().unwrap();
    let sel: Vec<&Value> = cases.iter().filter(|c| c["selected"] == true).collect();
    assert_eq!(sel.len(), 1);
    let s = sel[0];
    assert_eq!(s["stage"], "10OAK_P46");
    assert_eq!(s["life_y"], json!(25.0));
    assert_eq!(s["charged"], "rating");
    close(s["ac_l5"].as_f64().unwrap(), cost["value_l5"].as_f64().unwrap(), "L5");
    close(
        s["lac_dcf"].as_f64().unwrap(),
        cost["value_lac_dcf"].as_f64().unwrap(),
        "DCF",
    );
    assert_eq!(cost["met"], s["met"]);
    assert_eq!(cost["met"], true, "L5 {} DCF {}", s["ac_l5"], s["lac_dcf"]);
    assert_eq!(r["feasibility"]["feasible"], true);
    for c in cases {
        let below = c["ac_l5"].as_f64().unwrap() <= t && c["lac_dcf"].as_f64().unwrap() <= t;
        assert_eq!(c["at_or_below_target"], below, "{c}");
        assert_eq!(c["met"], below, "{c}");
    }
    // The selected stage is in the list at both lives and every exponent.
    for life in [25.0, 60.0] {
        assert_eq!(
            cases
                .iter()
                .filter(|c| c["stage"] == "10OAK_P46" && c["life_y"] == json!(life))
                .count(),
            3
        );
    }
    // The table cells hold BOAK and FOAK only.
    for c in cases.iter().filter(|c| c["table"] == true) {
        assert!(c["stage"] == "BOAK" || c["stage"] == "FOAK", "{c}");
    }
}

/// The live reply reuses the H2 return closure of the embedded registry, which
/// is exact only while no control moves it. Every number control at both ends
/// of its model range, the Ahn heat basis and both OCC levels leave it as it is.
#[test]
fn no_control_moves_the_h2_return_closure() {
    use nush2_plant::anchored::Model;
    use nush2_plant::flowsheet::{base_closure_cached, close_h2_return};
    let base = nush2_plant::params::ParamSet::embedded().unwrap();
    let r0 = base_closure_cached().unwrap().h2_return;
    let reg = reply(1, &json!({}));
    let mut sets: Vec<Vec<(String, f64)>> = Vec::new();
    for rec in reg["params"].as_array().unwrap() {
        let (Some(id), Some(scale)) = (rec["engine_id"].as_str(), rec["engine_scale"].as_f64())
        else {
            continue;
        };
        for end in ["model_min", "model_max"] {
            sets.push(vec![(id.to_string(), rec[end].as_f64().unwrap() * scale)]);
        }
    }
    let m0 = Model::new(&base).unwrap();
    let q_ahn = m0.q_proc_ahn_b / m0.s_b;
    sets.push(vec![
        ("Q_PROCESS".into(), q_ahn),
        ("HE_FLOW".into(), m0.he_flow * q_ahn / m0.q_proc),
    ]);
    for occ in ["OCC_LOW", "OCC_HIGH"] {
        sets.push(vec![("OCC_CENTRAL".into(), base.get(occ).unwrap())]);
    }
    assert!(sets.len() > 20);
    for set in sets {
        let mut p = base.clone();
        for (id, v) in &set {
            p.set(id, *v).unwrap();
        }
        let r = close_h2_return(&Model::new(&p).unwrap()).unwrap().h2_return;
        assert_eq!(r, r0, "{set:?}");
    }
}

/// The M6 island duties appear only at the design point of the column model.
#[test]
fn island_duties_follow_the_design_point() {
    let at = solve_with(json!({}), false);
    let moved = solve_with(json!({"lean_loading": 0.3}), false);
    let case_a = solve_with(json!({"basis": "case_a"}), false);
    let duty = |r: &Value| r["units"]["lean_cooler"]["results"][1]["value"].clone();
    assert!(duty(&at).is_number());
    for r in [&moved, &case_a] {
        assert!(duty(r).is_null());
        let why = r["engine_fields_missing"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["field"] == "units.lean_cooler.results.duty")
            .and_then(|m| m["reason"].as_str())
            .unwrap_or("");
        assert!(why.contains("design point"), "{why}");
    }
}
'''
p.write_bytes(s.encode("utf-8"))
print("ok")
