//! M8 acceptance: term flips only on a passed gate, the energy rows of the brief
//! s5 list, the reactor heat budget closing on its own rows, the boundary flag
//! from where each stream's ends sit, and the drift checks of plan section 7.

use nush2_plant::export::{tables, Tables};
use nush2_plant::flowsheet::{self, inside_boundary};
use nush2_plant::gates::{self, GateState, Gates};
use nush2_plant::params::ParamSet;
use nush2_plant::terms::{self, Source};
use nush2_plant::{anchored::Model, drift, Preset};
use std::collections::BTreeMap;
use std::sync::OnceLock;

fn tabs() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| tables(&ParamSet::embedded().unwrap()).unwrap())
}

fn split(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '"' if q && it.peek() == Some(&'"') => {
                cur.push('"');
                it.next();
            }
            '"' => q = !q,
            ',' if !q => out.push(std::mem::take(&mut cur)),
            o => cur.push(o),
        }
    }
    out.push(cur);
    out
}

/// Rows of a CSV as maps from header to cell.
fn rows(text: &str) -> Vec<BTreeMap<String, String>> {
    let mut lines = text.lines();
    let head = split(lines.next().unwrap());
    lines
        .filter(|l| !l.is_empty())
        .map(|l| head.iter().cloned().zip(split(l)).collect())
        .collect()
}

fn validation_csv() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../results/Validation_Results.csv"
    );
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn flip_requires_gate() {
    let csv = validation_csv();
    let all = rows(&csv);
    let pairs: Vec<(&str, &str)> = all
        .iter()
        .map(|r| (r["test_id"].as_str(), r["status"].as_str()))
        .collect();
    let plain: Vec<(&str, &str)> = pairs
        .iter()
        .copied()
        .filter(|(id, _)| !id.starts_with("GATE_"))
        .collect();
    // The gate rows, the rows behind them and the recorded table agree.
    let from_rows = Gates::from_rows(&plain);
    let from_gate_rows = Gates::from_gate_rows(&pairs);
    assert_eq!(
        from_rows, from_gate_rows,
        "GATE_ rows disagree with their rows"
    );
    assert_eq!(
        from_gate_rows,
        Gates::recorded(),
        "gates::RECORDED disagrees with results/Validation_Results.csv"
    );
    let p = ParamSet::embedded().unwrap();
    for preset in [Preset::Headline, Preset::Modelled] {
        for t in terms::for_preset(&p, preset, &Gates::recorded()).unwrap() {
            let gate_row = pairs
                .iter()
                .find(|(id, _)| *id == format!("GATE_{}", t.id))
                .map(|(_, s)| *s);
            if t.used == Source::Modelled {
                // Each flipped term has a passing gate row and every row behind it
                // that carries a limit passed.
                assert_eq!(gate_row, Some("pass"), "{} flipped without a gate", t.id);
                let def = gates::def(t.id).unwrap();
                let behind: Vec<&(&str, &str)> =
                    plain.iter().filter(|(id, _)| def.selects(id)).collect();
                assert!(!behind.is_empty(), "{} has no rows behind its gate", t.id);
                for (id, s) in behind {
                    assert!(
                        *s == "pass" || *s == "reported",
                        "{} flipped while {id} reads {s}",
                        t.id
                    );
                }
            } else if terms::wants_model(preset, t.id, Some(GateState::Passed))
                && t.modelled.is_some()
            {
                // A term this preset would flip stays only when its gate did not pass.
                assert_ne!(gate_row, Some("pass"), "{} has a passed gate", t.id);
            }
        }
    }
    // Headline flips T05 alone on the recorded gates; T04, T07 and T08 stay.
    let head = terms::for_preset(&p, Preset::Headline, &Gates::recorded()).unwrap();
    for id in ["T04", "T07", "T08"] {
        let t = head.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t.used, Source::Anchored, "{id}");
    }
    let t05 = head.iter().find(|t| t.id == "T05").unwrap();
    assert_eq!(t05.used, Source::Modelled);
}

#[test]
fn energy_rows_cover_the_brief_list() {
    let e = rows(&tabs().integrated_energy_balance);
    let required = [
        "HTGR_HEAT",
        "IHX_DUTY",
        "IHX_LOSS",
        "SMR_HEAT",
        "WGS_RELEASE",
        "SYNGAS_RECOVERY",
        "MEA_REGEN",
        "STEAM_AVAILABLE",
        "POWER_GENERATED",
        "HE_CIRCULATION",
        "PUMP_COMPRESS",
        "COOLING",
        "UNMET_THERMAL",
        "UNMET_ELECTRICAL",
        "MIN_REACTOR_RATING",
        "DUCT_LOSS",
        "VESSEL_LOSS",
        "CIRC_SHAFT_HEAT",
        "POWER_BRANCH",
    ];
    for run in [
        "A_INL_SEL",
        "A_REC_F",
        "B_REC_F",
        "B_REC_F_ANCHORED",
        "B_REC_F_MODELLED",
    ] {
        for id in required {
            let r = e
                .iter()
                .find(|r| r["run_id"] == run && r["term_id"] == id)
                .unwrap_or_else(|| panic!("{run} lacks {id}"));
            assert!(!r["value_mw"].is_empty(), "{run} {id} is empty");
            assert_ne!(r["status"], "not_run", "{run} {id}");
        }
    }
    // A run with no Level 3 flowsheet writes the shift release as not run, empty.
    let r = e
        .iter()
        .find(|r| r["run_id"] == "B_COMB_H" && r["term_id"] == "WGS_RELEASE")
        .unwrap();
    assert_eq!(r["status"], "not_run");
    assert!(r["value_mw"].is_empty());
}

#[test]
fn reactor_budget_closes_on_its_rows() {
    let e = rows(&tabs().integrated_energy_balance);
    let runs: Vec<String> = e
        .iter()
        .filter(|r| r["term_id"] == "HTGR_HEAT")
        .map(|r| r["run_id"].clone())
        .collect();
    assert!(runs.len() >= 8);
    for run in runs {
        let v = |id: &str| -> f64 {
            e.iter()
                .find(|r| r["run_id"] == run && r["term_id"] == id)
                .unwrap()["value_mw"]
                .parse()
                .unwrap()
        };
        let sources = v("HTGR_HEAT") + v("CIRC_SHAFT_HEAT");
        let sinks =
            v("IHX_DUTY") + v("IHX_LOSS") + v("DUCT_LOSS") + v("VESSEL_LOSS") + v("POWER_BRANCH");
        assert!(
            (sources - sinks).abs() < 1e-5,
            "{run}: {sources} against {sinks}"
        );
        // The reboiler is met in the approved order and the parts add up.
        let parts = v("REB_FROM_SELEXOL")
            + v("REB_FROM_EXPORT")
            + v("REB_FROM_SYNGAS")
            + v("REB_FROM_ZLD")
            + v("REB_FROM_TAIL_GAS")
            + v("REB_FROM_REACTOR");
        assert!(
            (parts - v("MEA_REGEN")).abs() < 1e-5,
            "{run}: reboiler {parts}"
        );
    }
}

#[test]
fn crosses_boundary_follows_figure_2() {
    let mb = rows(&tabs().integrated_mass_balance);
    let mut seen = 0;
    for r in &mb {
        if r["stream_id"].starts_with("CLOSURE") || r["stream_id"] == "NOT_CONVERGED" {
            continue;
        }
        let want = inside_boundary(&r["from_unit"]) != inside_boundary(&r["to_unit"]);
        assert_eq!(
            r["crosses_boundary"],
            want.to_string(),
            "{} {}",
            r["run_id"],
            r["stream_id"]
        );
        seen += 1;
    }
    assert!(seen > 100);
    let flag = |run: &str, id: &str| -> String {
        mb.iter()
            .find(|r| r["run_id"] == run && r["stream_id"] == id)
            .unwrap_or_else(|| panic!("{run} {id}"))["crosses_boundary"]
            .clone()
    };
    for (id, want) in [
        ("RECYCLE_TO_REFORMER", "false"),
        ("H2_PRODUCT", "false"),
        ("TAIL_GAS_EXPORT", "false"),
        ("CO2_TO_STORAGE", "false"),
        ("PSA_FEED", "true"),
        ("PSA_REJECT_RETURN", "true"),
        ("CRYO_H2_RETURN", "true"),
        ("NG_FEED", "true"),
    ] {
        assert_eq!(flag("B_REC_F", id), want, "{id}");
    }
    for (id, want) in [
        ("S03", "false"),
        ("S13", "false"),
        ("S20", "false"),
        ("S12", "true"),
        ("S14", "true"),
        ("S21", "true"),
        ("S16", "true"),
    ] {
        assert_eq!(flag("FS_B_REC_F", id), want, "{id}");
    }
}

#[test]
fn anchored_closures_come_from_the_flowsheet_where_one_exists() {
    let mb = rows(&tabs().integrated_mass_balance);
    let get = |run: &str, id: &str| {
        mb.iter()
            .find(|r| r["run_id"] == run && r["stream_id"] == id)
            .unwrap()
            .clone()
    };
    let h = get("B_REC_F", "CLOSURE_H");
    assert_eq!(h["status"], "flowsheet_m7");
    let v: f64 = h["total_kmol_h"].parse().unwrap();
    assert!(v.abs() < 1e-6, "{v}");
    let mass = get("B_REC_F", "CLOSURE_MASS");
    assert!(!mass["total_kg_h"].is_empty());
    assert_eq!(get("B_COMB_H", "CLOSURE_H")["status"], "not_run");
    let ret = get("B_REC_F", "CRYO_H2_RETURN");
    let h2: f64 = ret["h2"].parse().unwrap();
    assert!(h2 > 0.0);
}

#[test]
fn drift_reads_the_section_7_limits() {
    let p = ParamSet::embedded().unwrap();
    let m = Model::new(&p).unwrap();
    let fs = flowsheet::suite(&m);
    let d = drift::drift(&p, &fs, &Gates::recorded()).unwrap();
    assert_eq!(d.findings.len(), 8);
    // The high corner is checked against the rating as well as the central R.
    let hc = drift::high_corner(&d.record.p_head).unwrap();
    let f = d
        .findings
        .iter()
        .find(|f| f.check.starts_with("Headline high-corner"))
        .unwrap();
    assert_eq!(f.raised, hc.r_mwth > drift::R_LIMIT_MWTH);
    // Headline equals Anchored in carbon, since no flip touches carbon.
    assert_eq!(d.headline.abatement_t_y, d.anchored.abatement_t_y);
    for f in &d.findings {
        println!(
            "{}: {} -> {} raised {}",
            f.check, f.limit, f.value, f.raised
        );
    }
    let md = drift::markdown(&d);
    assert_eq!(md.lines().count(), 15);
}

/// Every closure group of a fixed run closes on its own rows: in `reactor`,
/// `reboiler` and `electricity` the sources equal the sinks, and the parts of
/// `part:POWER_BRANCH` add up to that row. In the Level 3 runs the `envelope`
/// group closes. The rows print six decimals, so ten rounded cells can carry
/// 5e-6 MW; the limit is 1e-5 MW.
#[test]
fn eq_j_24_energy_groups_close_on_their_rows() {
    let e = rows(&tabs().integrated_energy_balance);
    let mut sums: BTreeMap<(String, String), f64> = BTreeMap::new();
    let mut parts: BTreeMap<(String, String), f64> = BTreeMap::new();
    let mut value: BTreeMap<(String, String), f64> = BTreeMap::new();
    for r in &e {
        let Ok(v) = r["value_mw"].parse::<f64>() else {
            continue;
        };
        value.insert((r["run_id"].clone(), r["term_id"].clone()), v);
        let g = r["closure_group"].clone();
        if let Some(parent) = g.strip_prefix("part:") {
            *parts
                .entry((r["run_id"].clone(), parent.to_string()))
                .or_default() += v;
        } else if g != "memo" {
            let sign = if r["side"] == "source" { 1.0 } else { -1.0 };
            *sums.entry((r["run_id"].clone(), g)).or_default() += sign * v;
        }
    }
    let groups: std::collections::BTreeSet<&str> = sums.keys().map(|k| k.1.as_str()).collect();
    for g in ["reactor", "reboiler", "electricity", "envelope"] {
        assert!(groups.contains(g), "no {g} group");
    }
    for ((run, g), s) in &sums {
        // The Level 3 envelope holds about twenty rounded rows, so its limit
        // is 1e-4 MW.
        let lim = if g == "envelope" { 1e-4 } else { 1e-5 };
        assert!(s.abs() < lim, "{run} {g}: sources less sinks {s}");
    }
    assert!(!parts.is_empty());
    for ((run, parent), s) in &parts {
        let v = value[&(run.clone(), parent.clone())];
        assert!(
            (s - v).abs() < 1e-5,
            "{run} {parent}: parts {s} against {v}"
        );
    }
    // The electricity side covers every consumer: POWER_GENERATED and tail-gas
    // power together equal the process and circulator electricity.
    let b = |id: &str| value[&("B_REC_F".to_string(), id.to_string())];
    let sinks = b("HE_CIRCULATION")
        + b("PUMP_COMPRESS")
        + b("COOLING")
        + b("REFORMING_POWER")
        + b("PURIFICATION_POWER")
        + b("STEAM_SYSTEM_POWER")
        + b("WATER_POWER");
    assert!((b("POWER_GENERATED") + b("TAIL_GAS_POWER") - sinks).abs() < 1e-5);
}

/// Each node of an anchored run that has streams in and out closes on carbon,
/// and on H2 where every stream at the node carries its H2. H2_PRODUCT mass
/// counts every species it holds.
#[test]
fn eq_j_11_anchored_nodes_close() {
    let mb = rows(&tabs().integrated_mass_balance);
    let runs: std::collections::BTreeSet<String> = mb
        .iter()
        .filter(|r| r["stream_id"] == "PSA_FEED")
        .map(|r| r["run_id"].clone())
        .collect();
    assert!(runs.len() >= 7, "{runs:?}");
    for run in &runs {
        let streams: Vec<_> = mb
            .iter()
            .filter(|r| &r["run_id"] == run && !r["stream_id"].starts_with("CLOSURE_"))
            .collect();
        let nodes: std::collections::BTreeSet<String> = streams
            .iter()
            .flat_map(|r| [r["from_unit"].clone(), r["to_unit"].clone()])
            .collect();
        for node in nodes {
            let ins: Vec<_> = streams.iter().filter(|r| r["to_unit"] == node).collect();
            let outs: Vec<_> = streams.iter().filter(|r| r["from_unit"] == node).collect();
            if ins.is_empty() || outs.is_empty() {
                continue;
            }
            for col in ["carbon_kmol_h", "h2"] {
                let all = ins.iter().chain(&outs).all(|r| !r[col].is_empty());
                if !all {
                    assert_ne!(col, "carbon_kmol_h", "{run} {node} lacks carbon");
                    continue;
                }
                let sum = |v: &[&&BTreeMap<String, String>]| -> f64 {
                    v.iter().map(|r| r[col].parse::<f64>().unwrap()).sum()
                };
                let gap = sum(&ins) - sum(&outs);
                assert!(gap.abs() < 1e-5, "{run} {node} {col}: in less out {gap}");
            }
        }
        let p = streams
            .iter()
            .find(|r| r["stream_id"] == "H2_PRODUCT")
            .unwrap();
        let num = |c: &str| p[c].parse::<f64>().unwrap();
        let species = num("h2") * nush2_props::units::MW_H2
            + num("ch4") * nush2_props::species::Sp::Ch4.mw_kg_mol() * 1000.0;
        assert!(
            (num("total_kg_h") - species).abs() < 1e-3,
            "{run} product mass"
        );
    }
}

/// The burner's combustion O2 is a row of every Level 3 run, so mass and oxygen
/// close from the table alone.
#[test]
fn combustion_o2_is_a_row() {
    let mb = rows(&tabs().integrated_mass_balance);
    let o2: Vec<_> = mb
        .iter()
        .filter(|r| r["stream_id"] == "COMBUSTION_O2")
        .collect();
    assert!(!o2.is_empty());
    for r in o2 {
        assert!(r["run_id"].starts_with("FS_"));
        assert_eq!(r["crosses_boundary"], "true");
        assert!(r["total_kmol_h"].parse::<f64>().unwrap() > 0.0);
    }
}

#[test]
fn cryo_split_sensitivity_is_recorded() {
    let s = rows(&tabs().sensitivity_results);
    let v: Vec<_> = s
        .iter()
        .filter(|r| r["sweep_id"] == "CRYO_H2_SPLIT")
        .collect();
    assert!(!v.is_empty());
    for r in v {
        assert!(r["status"] == "flowsheet_m8" || r["status"] == "not_converged");
    }
}
