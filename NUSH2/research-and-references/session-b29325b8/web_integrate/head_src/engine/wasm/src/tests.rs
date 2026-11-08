//! Native tests of the three replies. They parse each reply as the page would,
//! check every key the contract requires, and compare the default solve with
//! the published results files written by the same engine.

use crate::registry::{self, Kind, KPIS, REQUIRED_KPIS, SPECS};
use crate::{alloc, call, dealloc, dispatch, solve};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

const REQUIRED: [&str; 24] = [
    "rating_mwth",
    "reactor_required_mwth",
    "surplus_mwth",
    "surplus_ahn_mwth",
    "unmet_heat_mwth",
    "unmet_power_mwe",
    "q_process_mwth",
    "h2_rich_kmol_h",
    "h2_rich_kg_h",
    "h2_product_t_y",
    "ng_mmscfd",
    "co2_captured_t_y",
    "co2_atm_t_y",
    "capture_per_pass",
    "capture_plant",
    "q_reboiler_mw",
    "reboiler_gap_mw",
    "e_aux_mwe",
    "abatement_t_y",
    "abatement_cho_t_y",
    "abatement_ieaghg_t_y",
    "abatement_system_t_y",
    "abatement_lifecycle_t_y",
    "cost_s_per_t",
];

const STATUS_WORDS: [&str; 5] = [
    "rigorous",
    "reduced_order",
    "empirical",
    "fixed_assumption",
    "external",
];

fn reply(cmd: u32, body: &Value) -> Value {
    let text = dispatch(cmd, body.to_string().as_bytes());
    serde_json::from_str(&text).expect("reply parses as JSON")
}

fn solve_with(inputs: Value, kpi_only: bool) -> Value {
    reply(
        2,
        &json!({"schema": 1, "inputs": inputs, "kpi_only": kpi_only}),
    )
}

fn obj(v: &Value) -> &Map<String, Value> {
    v.as_object().expect("object")
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Minimal CSV split with quoted fields.
fn split(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn csv_rows(name: &str) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(root().join("results").join(name)).expect("results file");
    text.lines().map(split).collect()
}

fn csv_value(name: &str, key_cols: &[(usize, &str)], value_col: usize) -> f64 {
    let rows = csv_rows(name);
    let hit = rows
        .iter()
        .find(|r| {
            key_cols
                .iter()
                .all(|(i, k)| r.get(*i).map(String::as_str) == Some(*k))
        })
        .unwrap_or_else(|| panic!("{name}: no row for {key_cols:?}"));
    hit[value_col].parse().expect("number")
}

fn close(a: f64, b: f64, what: &str) {
    let tol = 2e-6 * a.abs().max(b.abs()).max(1.0);
    assert!((a - b).abs() <= tol, "{what}: wasm {a} against results {b}");
}

#[test]
fn registry_reply_carries_every_record_field() {
    let r = reply(1, &json!({}));
    assert_eq!(r["schema"], 1);
    assert_eq!(r["status"], "ok");
    assert!(r["engine"]["version"].is_string());
    assert!(r["engine"]["commit"].is_string());
    let params = r["params"].as_array().expect("params");
    assert_eq!(params.len(), SPECS.len());
    let fields = [
        "id",
        "label",
        "group",
        "unit",
        "kind",
        "default",
        "hard_min",
        "hard_max",
        "model_min",
        "model_max",
        "step",
        "options",
        "class",
        "source",
        "note",
    ];
    for p in params {
        let o = obj(p);
        for f in fields {
            assert!(o.contains_key(f), "{} lacks {f}", p["id"]);
        }
        assert!(p["class"].is_string(), "{} has no class", p["id"]);
        assert!(p["source"].is_string(), "{} has no source", p["id"]);
        match p["kind"].as_str() {
            Some("number") => {
                let d = p["default"].as_f64().expect("default");
                let (hmin, hmax) = (
                    p["hard_min"].as_f64().unwrap(),
                    p["hard_max"].as_f64().unwrap(),
                );
                let (mmin, mmax) = (
                    p["model_min"].as_f64().unwrap(),
                    p["model_max"].as_f64().unwrap(),
                );
                assert!(
                    hmin <= d && d <= hmax,
                    "{} default {d} outside hard range",
                    p["id"]
                );
                assert!(
                    hmin <= mmin && mmax <= hmax,
                    "{} model range outside hard range",
                    p["id"]
                );
                assert!(p["step"].as_f64().unwrap() > 0.0);
            }
            Some("enum") => {
                let opts = p["options"].as_array().expect("options");
                assert!(
                    opts.iter().any(|o| o["value"] == p["default"]),
                    "{} default not an option",
                    p["id"]
                );
            }
            other => panic!("bad kind {other:?}"),
        }
    }
    let units = obj(&r["units"]);
    assert!(!units.is_empty());
    for (id, u) in units {
        let s = u["status"].as_str().unwrap_or("");
        assert!(STATUS_WORDS.contains(&s), "unit {id} status {s}");
    }
    let kpis: Vec<&str> = r["kpis"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["id"].as_str().unwrap())
        .collect();
    for k in REQUIRED {
        assert!(kpis.contains(&k), "registry kpis lack {k}");
    }
    assert_eq!(
        &KPIS[..REQUIRED_KPIS]
            .iter()
            .map(|k| k.0)
            .collect::<Vec<_>>()[..],
        &REQUIRED[..]
    );
}

#[test]
fn default_solve_has_every_section_and_kpi() {
    let r = solve_with(json!({}), false);
    if let Ok(path) = std::env::var("NUSH2_WASM_DUMP") {
        std::fs::write(&path, r.to_string()).expect("dump written");
    }
    assert_eq!(r["status"], "ok", "messages {}", r["messages"]);
    for key in [
        "schema",
        "engine",
        "status",
        "violations",
        "flags",
        "messages",
        "kpi",
        "streams",
        "units",
        "profiles",
        "balances",
        "cost",
        "engine_fields_missing",
        "targets",
    ] {
        assert!(obj(&r).contains_key(key), "reply lacks {key}");
    }
    let kpi = obj(&r["kpi"]);
    let missing: Vec<&str> = r["engine_fields_missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["field"].as_str().unwrap())
        .collect();
    for k in REQUIRED {
        let v = kpi.get(k).unwrap_or_else(|| panic!("kpi lacks {k}"));
        if v.is_null() {
            let field = format!("kpi.{k}");
            assert!(
                missing.contains(&field.as_str()),
                "{k} is null without an engine_fields_missing entry"
            );
        } else {
            assert!(
                v.as_f64().map(f64::is_finite).unwrap_or(false),
                "{k} is not a finite number"
            );
        }
    }
    assert!(kpi["unmet_heat_mwth"].is_null());
    assert!(missing.contains(&"kpi.unmet_heat_mwth"));
    assert!(r["streams"].is_null());
    assert!(missing.contains(&"streams"));
    for (_, p) in obj(&r["profiles"]) {
        assert!(p.is_null());
    }
    for (id, u) in obj(&r["units"]) {
        for sec in ["inputs", "outputs", "conditions", "results"] {
            for e in u[sec].as_array().unwrap_or_else(|| panic!("{id}.{sec}")) {
                for f in ["id", "label", "value", "unit", "class", "src"] {
                    assert!(obj(e).contains_key(f), "{id}.{sec} entry lacks {f}");
                }
            }
        }
        assert!(STATUS_WORDS.contains(&u["status"].as_str().unwrap()));
    }
    for sec in ["energy", "power", "carbon"] {
        for e in r["balances"][sec].as_array().unwrap() {
            for f in ["id", "label", "value", "unit", "class", "src"] {
                assert!(obj(e).contains_key(f), "balances.{sec} entry lacks {f}");
            }
        }
    }
    let grid = r["cost"]["grid"].as_array().unwrap();
    assert_eq!(
        grid.len(),
        2 * 4 * 3 * 3 * 6,
        "grid covers charged x stage x life x n x level"
    );
    for g in grid {
        for f in ["charged_mwth", "n", "stage", "life_y", "level", "s_per_t"] {
            assert!(obj(g).contains_key(f), "grid row lacks {f}");
        }
    }
    assert_eq!(r["cost"]["ladder"].as_array().unwrap().len(), 6);
    assert_eq!(
        r["cost"]["headline"].as_array().unwrap().len(),
        2 * 4 * 3 * 3
    );
    assert_eq!(r["targets"]["cost"]["met"], false);
}

#[test]
fn default_solve_matches_the_published_results() {
    let r = solve_with(json!({}), false);
    let k = &r["kpi"];
    let fdb = "Final_Design_Basis.csv";
    let b = |id: &str| csv_value(fdb, &[(0, id)], 3);
    close(
        k["h2_product_t_y"].as_f64().unwrap(),
        b("RES_H2_T_Y"),
        "h2_product_t_y",
    );
    close(
        k["ng_mmscfd"].as_f64().unwrap(),
        b("RES_NG_MMSCFD"),
        "ng_mmscfd",
    );
    close(
        k["reactor_required_mwth"].as_f64().unwrap(),
        b("RES_REACTOR_NEEDED_MWTH"),
        "required",
    );
    close(
        k["surplus_mwth"].as_f64().unwrap(),
        b("RES_MARGIN_210_MWTH"),
        "surplus",
    );
    close(
        k["q_process_mwth"].as_f64().unwrap(),
        b("RES_PROCESS_HEAT_MW"),
        "q_process",
    );
    close(
        k["co2_captured_t_y"].as_f64().unwrap(),
        b("RES_CAPTURED_T_Y"),
        "captured",
    );
    close(k["co2_atm_t_y"].as_f64().unwrap(), b("RES_ATM_T_Y"), "atm");
    close(
        k["capture_plant"].as_f64().unwrap(),
        b("RES_PLANT_CAPTURE"),
        "plant capture",
    );
    close(
        k["q_reboiler_mw"].as_f64().unwrap(),
        b("RES_REBOILER_MW"),
        "reboiler",
    );
    close(
        k["reboiler_gap_mw"].as_f64().unwrap(),
        b("RES_REBOILER_GAP_MW"),
        "gap",
    );
    close(
        k["e_aux_mwe"].as_f64().unwrap(),
        b("RES_E_TOTAL_MWE"),
        "e_aux",
    );
    close(
        k["unmet_power_mwe"].as_f64().unwrap(),
        b("RES_GRID_IMPORT_AT_RATED_MWE"),
        "import",
    );
    close(
        k["abatement_t_y"].as_f64().unwrap(),
        b("RES_ABATEMENT_T_Y"),
        "abatement",
    );
    close(
        k["abatement_cho_t_y"].as_f64().unwrap(),
        b("RES_ABATEMENT_CHO_T_Y"),
        "cho",
    );
    // IEAGHG bound: the unrounded 8.996 kg/kg, as Emissions_Ledger.csv prints it
    // (build order section 5 item 3 moves Final_Design_Basis.csv onto the same value).
    let ieaghg = csv_value(
        "Emissions_Ledger.csv",
        &[
            (0, "B_REC_F"),
            (1, "IEAGHG_9p00"),
            (2, "direct_atm"),
            (3, "ABATEMENT"),
        ],
        5,
    );
    close(
        k["abatement_ieaghg_t_y"].as_f64().unwrap(),
        ieaghg,
        "ieaghg",
    );
    close(
        k["h2_rich_kg_h"].as_f64().unwrap(),
        b("RES_PSA_H2_FEED_KG_H"),
        "h2 rich",
    );
    close(
        k["solvent_t_h"].as_f64().unwrap(),
        b("RES_SOLVENT_T_H"),
        "solvent",
    );
    close(
        k["cost_s_per_t"].as_f64().unwrap(),
        b("AC_L5_BOAK_25Y_N1P00"),
        "L5",
    );
    close(
        k["cost_l3_s_per_t"].as_f64().unwrap(),
        b("AC_L3_BOAK_25Y_N1P00"),
        "L3",
    );
    close(
        k["cost_lac_dcf_s_per_t"].as_f64().unwrap(),
        b("LAC_DCF_BOAK_25Y_N1P00"),
        "LAC",
    );
    close(
        k["lcoh_nush2_s_per_kg"].as_f64().unwrap(),
        b("LCOH_NUSH2_BOAK_25Y_N1P00"),
        "LCOH NUS-H2",
    );
    close(
        k["lcoh_smr_s_per_kg"].as_f64().unwrap(),
        b("LCOH_SMR_BOAK_25Y_N1P00"),
        "LCOH SMR",
    );
    let em = "Emissions_Ledger.csv";
    let line = |view: &str| {
        csv_value(
            em,
            &[(0, "B_REC_F"), (1, "INL_9p21"), (2, view), (3, "ABATEMENT")],
            5,
        )
    };
    close(
        k["abatement_system_t_y"].as_f64().unwrap(),
        line("system"),
        "system",
    );
    close(
        k["abatement_lifecycle_t_y"].as_f64().unwrap(),
        line("lifecycle_upstream"),
        "lifecycle",
    );
    let sens = csv_value(
        "Sensitivity_Results.csv",
        &[
            (0, "HEAT_BASIS"),
            (1, "B_REC_F"),
            (5, "143.400000"),
            (6, "MARGIN_210"),
        ],
        7,
    );
    close(
        k["surplus_ahn_mwth"].as_f64().unwrap(),
        sens,
        "surplus on the Ahn basis",
    );
    // The headline table at n = 1.0, BOAK and FOAK, 25 y and 60 y, rating charged.
    for row in r["cost"]["headline"].as_array().unwrap() {
        if row["charged"] != "rating" || row["n"] != json!(1.0) {
            continue;
        }
        let stage = row["stage"].as_str().unwrap();
        let life = row["life_y"].as_f64().unwrap() as u32;
        if !matches!(stage, "BOAK" | "FOAK") || !matches!(life, 25 | 60) {
            continue;
        }
        let tag = format!("{stage}_{life}Y_N1P00");
        close(
            row["ac_l5"].as_f64().unwrap(),
            b(&format!("AC_L5_{tag}")),
            &tag,
        );
        close(
            row["lac_dcf"].as_f64().unwrap(),
            b(&format!("LAC_DCF_{tag}")),
            &tag,
        );
        close(
            row["lcoh_nush2"].as_f64().unwrap(),
            b(&format!("LCOH_NUSH2_{tag}")),
            &tag,
        );
        close(
            row["lcoh_smr"].as_f64().unwrap(),
            b(&format!("LCOH_SMR_{tag}")),
            &tag,
        );
    }
}

#[test]
fn sweep_reply_parses_and_is_capped() {
    let r = reply(
        3,
        &json!({"inputs": {}, "param": "rating_mwth", "values": [180, 210, 240]}),
    );
    assert_eq!(r["status"], "ok");
    let rows = r["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert_eq!(row["status"], "ok");
        for k in REQUIRED {
            assert!(obj(&row["kpi"]).contains_key(k), "sweep row lacks {k}");
        }
    }
    let s0 = rows[0]["kpi"]["surplus_mwth"].as_f64().unwrap();
    let s2 = rows[2]["kpi"]["surplus_mwth"].as_f64().unwrap();
    assert!(s2 > s0);
    let many: Vec<f64> = (0..26).map(|i| 150.0 + i as f64).collect();
    let r = reply(3, &json!({"param": "rating_mwth", "values": many}));
    assert_eq!(r["status"], "rejected");
    let r = reply(3, &json!({"param": "rating_mwth", "values": [500]}));
    assert_eq!(r["rows"][0]["status"], "rejected");
}

#[test]
fn rust_refuses_out_of_range_and_unknown_input() {
    let r = solve_with(json!({"rating_mwth": 400}), true);
    assert_eq!(r["status"], "rejected");
    assert_eq!(r["violations"][0]["param"], "rating_mwth");
    assert_eq!(r["violations"][0]["max"], 350.0);
    let r = solve_with(json!({"ihx_approach_k": 60}), true);
    assert_eq!(
        r["status"], "rejected",
        "helium supply plus approach above 950 C"
    );
    let r = solve_with(json!({"nonsense": 1}), true);
    assert_eq!(r["status"], "rejected");
    let r = solve_with(json!({"stage": "SOAK"}), true);
    assert_eq!(r["status"], "rejected");
    let r = solve_with(json!({"srd": "high"}), true);
    assert_eq!(r["status"], "rejected");
    let r = reply(2, &json!({"schema": 2}));
    assert_eq!(r["status"], "rejected");
    let r: Value = serde_json::from_str(&dispatch(2, b"{not json")).unwrap();
    assert_eq!(r["status"], "rejected");
    let r: Value = serde_json::from_str(&dispatch(9, b"{}")).unwrap();
    assert_eq!(r["status"], "rejected");
}

#[test]
fn model_range_raises_a_flag_without_blocking() {
    let r = solve_with(json!({"rating_mwth": 300}), true);
    assert_eq!(r["status"], "ok");
    let flags = r["flags"].as_array().unwrap();
    assert!(flags
        .iter()
        .any(|f| f["param"] == "rating_mwth" && f["state"] == "outside_model_range"));
    let r = solve_with(json!({}), true);
    let flags = r["flags"].as_array().unwrap();
    assert!(
        flags.iter().any(|f| f["param"] == "T_HE_SUPPLY"),
        "900 C supply is flagged at default"
    );
}

/// Every control moves at least one KPI, so no dead control ships. A number
/// record is probed at its model maximum, or at another range end when the
/// model maximum equals the default.
#[test]
fn every_control_moves_a_kpi() {
    let base = solve_with(json!({}), true);
    assert_eq!(base["status"], "ok");
    let reg = reply(1, &json!({}));
    let p = nush2_plant::params::ParamSet::embedded().unwrap();
    for rec in reg["params"].as_array().unwrap() {
        let id = rec["id"].as_str().unwrap();
        let probes: Vec<Value> = match rec["kind"].as_str().unwrap() {
            "number" => {
                let d = rec["default"].as_f64().unwrap();
                let cands = ["model_max", "model_min", "hard_max", "hard_min"];
                let x = cands
                    .iter()
                    .map(|c| rec[*c].as_f64().unwrap())
                    .find(|x| (x - d).abs() > 1e-12)
                    .expect("a probe value");
                vec![json!(x)]
            }
            _ => rec["options"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|o| o["value"] != rec["default"])
                .map(|o| o["value"].clone())
                .collect(),
        };
        let spec = registry::spec(id).unwrap();
        if let Kind::Number { .. } = spec.kind {
            assert!(registry::default_value(spec, &p).is_number());
        }
        for x in probes {
            let r = solve_with(json!({ id: x.clone() }), true);
            assert_eq!(r["status"], "ok", "{id} = {x}: {}", r);
            let moved = REQUIRED.iter().any(|k| {
                let (a, b) = (&base["kpi"][*k], &r["kpi"][*k]);
                match (a.as_f64(), b.as_f64()) {
                    (Some(a), Some(b)) => (a - b).abs() > 1e-9 * a.abs().max(1.0),
                    _ => false,
                }
            });
            assert!(moved, "{id} = {x} moves no KPI");
        }
    }
}

#[test]
fn ffi_round_trip_frames_the_reply() {
    let body = br#"{"schema":1,"inputs":{},"kpi_only":true}"#;
    let p = alloc(body.len());
    unsafe {
        std::ptr::copy_nonoverlapping(body.as_ptr(), p, body.len());
        let r = call(2, p, body.len());
        dealloc(p, body.len());
        let mut len = [0u8; 4];
        std::ptr::copy_nonoverlapping(r, len.as_mut_ptr(), 4);
        let n = u32::from_le_bytes(len) as usize;
        let text = std::str::from_utf8(std::slice::from_raw_parts(r.add(4), n)).unwrap();
        let v: Value = serde_json::from_str(text).unwrap();
        assert_eq!(v["status"], "ok");
        dealloc(r, n + 4);
        let r = call(1, std::ptr::null(), 0);
        std::ptr::copy_nonoverlapping(r, len.as_mut_ptr(), 4);
        let n = u32::from_le_bytes(len) as usize;
        dealloc(r, n + 4);
    }
    let z = alloc(0);
    unsafe { dealloc(z, 0) };
}

#[test]
fn native_solve_time_is_recorded() {
    let t = std::time::Instant::now();
    let r = solve::solve(&json!({"inputs": {}}));
    let ms = t.elapsed().as_secs_f64() * 1e3;
    assert_eq!(r["status"], "ok");
    eprintln!("native full solve {ms:.1} ms");
}

/// The PFD draws these 20 units (edited Figure 2). Each must reach the page
/// with an engine status and the four live sections, and every null value in
/// a unit must be listed in engine_fields_missing with a reason.
#[test]
fn every_pfd_unit_has_status_and_sections() {
    const PFD: [&str; 20] = [
        "htgr",
        "circ_primary",
        "ihx",
        "power",
        "smr",
        "wgs",
        "syngas_cooler",
        "separator",
        "absorber",
        "lean_pump",
        "lean_cooler",
        "rich_lean_hx",
        "stripper",
        "reboiler",
        "condenser",
        "co2_compressor",
        "co2_storage",
        "psa",
        "recycle_compressor",
        "cryogenic",
    ];
    let reg = reply(1, &json!({}));
    let ids: Vec<&String> = obj(&reg["units"]).keys().collect();
    assert_eq!(ids.len(), PFD.len(), "registry units {ids:?}");
    let r = solve_with(json!({}), false);
    assert_eq!(r["status"], "ok");
    let missing: Vec<&str> = r["engine_fields_missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["field"].as_str().unwrap())
        .collect();
    for id in PFD {
        assert!(
            STATUS_WORDS.contains(&reg["units"][id]["status"].as_str().unwrap_or("")),
            "registry status of {id}"
        );
        let u = &r["units"][id];
        assert!(u.is_object(), "solve lacks unit {id}");
        assert_eq!(
            u["status"], reg["units"][id]["status"],
            "{id} status differs"
        );
        let mut live = 0;
        for sec in ["inputs", "outputs", "conditions", "results"] {
            for e in u[sec].as_array().unwrap() {
                live += 1;
                if e["value"].is_null() {
                    let field = format!("units.{id}.{sec}.{}", e["id"].as_str().unwrap());
                    assert!(
                        missing.contains(&field.as_str()),
                        "{field} is null and unlisted"
                    );
                    assert!(e["src"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
                }
            }
        }
        assert!(live > 0, "{id} has no engine field at all");
    }
}
