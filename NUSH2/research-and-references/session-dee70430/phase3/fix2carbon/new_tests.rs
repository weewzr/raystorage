
// ------------------ the reference of the headline: three findings of fix round 2

/// Rule 1 of the boundary table of N.7 at every central baseline: for each of
/// the three references the GAS line of the cost, the avoided-gas term of N.9.2
/// and the decomposition read one gas difference, and the decomposition adds up
/// to the direct abatement against that reference (equation N.9.2).
#[test]
fn rule_1_of_n_7_holds_for_every_central_baseline() {
    let mut gases = Vec::new();
    for v in [0.0, 1.0, 2.0] {
        let mut p = ParamSet::embedded().unwrap();
        p.set("CARBON_BASELINE", v).unwrap();
        let m = Model::new(&p).unwrap();
        let s = m.s_b;
        let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
        let ab = m.abatement(&b, s).unwrap();
        let d = ab.decomposition.unwrap();
        let q = boak_query(&m);
        let k = m.cost_items(&b, &q).unwrap();
        let led = m.ledger(&b, &q).unwrap();
        assert!(rel(k.ng_smr, d.n_ref_mmscfd) < 1e-12, "baseline {v}");
        let diff = -led.gas / k.gas_s_m_per_mmscfd;
        assert!(
            rel(diff, d.n_ref_mmscfd - k.ng_plant) < 1e-12,
            "baseline {v}"
        );
        let central = ab.rows.iter().find(|r| r.id == ab.central).unwrap();
        assert!(rel(d.total, central.a_direct) < 1e-9, "baseline {v}");
        // The abatement the cost divides by is the one against this reference.
        assert!(rel(m.emissions(&b.c, s).a_net, central.a_net) < 1e-12);
        gases.push(d.n_ref_mmscfd);
    }
    // A higher intensity implies more reference gas, so the three differ.
    assert!(gases[0] < gases[1] && gases[1] < gases[2], "{gases:?}");
}

/// The BASELINE sweep of the sensitivity table follows the baseline in the cost
/// as well as in the abatement: its three test points equal the points of the
/// CARBON_BASELINE sweep, metric by metric, and its base value is the central
/// intensity of the basis and not INL's.
#[test]
fn the_baseline_sensitivity_group_equals_the_carbon_baseline_group() {
    let m = model();
    let t = default_tables();
    let mut by_switch = std::collections::BTreeMap::new();
    let mut baseline = Vec::new();
    for line in t.sensitivity_results.lines().skip(1) {
        let c = split_cells(line);
        // sweep_id, run_id, parameter, unit, base_value, test_value, metric, result
        match c[0].as_str() {
            "CARBON_BASELINE" => {
                let v: f64 = c[5].parse().unwrap();
                by_switch.insert((v as u32, c[6].clone()), c[7].parse::<f64>().unwrap());
            }
            "BASELINE" => baseline.push(c),
            _ => {}
        }
    }
    assert_eq!(baseline.len(), 3 * 10, "three points, ten metrics");
    let central = m.baseline_kg_kg(m.carbon_params.baseline);
    for c in &baseline {
        let switch = match c[1].as_str() {
            "B_REC_F_BASE_IEAGHG" => 0,
            "B_REC_F_BASE_INL" => 1,
            "B_REC_F_BASE_CHO" => 2,
            other => panic!("unexpected run {other}"),
        };
        let got: f64 = c[7].parse().unwrap();
        let want = by_switch[&(switch, c[6].clone())];
        assert!(
            (got - want).abs() <= 1e-9 * want.abs().max(1.0),
            "{} {}: {got} against {want}",
            c[1],
            c[6]
        );
        // The base value is the central intensity of the basis (8.995997).
        let base: f64 = c[4].parse().unwrap();
        assert!((base - central).abs() < 1e-6, "{} {base} {central}", c[1]);
    }
    // The INL and Cho points moved the cost as well as the abatement: the cost
    // is not the central cost rescaled by the new abatement alone.
    let value = |run: &str, metric: &str| -> f64 {
        baseline
            .iter()
            .find(|c| c[1] == run && c[6] == metric)
            .map(|c| c[7].parse().unwrap())
            .unwrap()
    };
    let (l5_c, a_c) = (
        value("B_REC_F_BASE_IEAGHG", "AC_L5_25"),
        value("B_REC_F_BASE_IEAGHG", "A_T_Y"),
    );
    for run in ["B_REC_F_BASE_INL", "B_REC_F_BASE_CHO"] {
        let rescaled = l5_c * a_c / value(run, "A_T_Y");
        let got = value(run, "AC_L5_25");
        assert!(got < rescaled * 0.99, "{run}: {got} against {rescaled}");
    }
}

fn boak_nosys_query(m: &Model) -> nush2_plant::economics::CostQuery {
    nush2_plant::economics::CostQuery {
        system_lines: false,
        ..boak_query(m)
    }
}

/// The two lines that price the conventional plant of INL are switchable: with
/// them off both vanish from the ledger and from the cash flow, every other
/// line is bit for bit unchanged, and the ledger still tells the cash flow
/// within the gap it had.
#[test]
fn the_two_system_lines_vanish_together_when_they_are_switched_off() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let (q, qn) = (boak_query(m), boak_nosys_query(m));
    assert!(q.system_lines);
    let (led, led_n) = (m.ledger(&b, &q).unwrap(), m.ledger(&b, &qn).unwrap());
    assert!(led.lost_steam > 0.0 && led.baseline_power < 0.0);
    assert_eq!(led_n.lost_steam, 0.0);
    assert_eq!(led_n.baseline_power, 0.0);
    for ((id, _, v), (_, _, w)) in led.lines().into_iter().zip(led_n.lines()) {
        if id != "LOST_STEAM" && id != "BASE_POWER" {
            assert_eq!(v.to_bits(), w.to_bits(), "{id}");
        }
    }
    // L5 without the lines is L5 less the two lines, to rounding.
    let want = led.l5() - led.lost_steam - led.baseline_power;
    assert!(rel(led_n.l5(), want) < 1e-12, "{} {want}", led_n.l5());
    // The cash flow drops the same two lines.
    let (d, d_n) = (m.dcf(&b, &q).unwrap(), m.dcf(&b, &qn).unwrap());
    assert!(d_n.lac < d.lac);
    let a = m.emissions(&b.c, s).a_net;
    let l5_n = led_n.l5() * 1e6 / a;
    assert!(rel(d_n.lac, l5_n) < 0.01, "{} {l5_n}", d_n.lac);
    // Ledger and cash flow move by nearly the same amount (the cash flow
    // discounts annual flows that the ledger takes as they are).
    let (dl, dd) = ((led.l5() - led_n.l5()) * 1e6 / a, d.lac - d_n.lac);
    assert!(rel(dd, dl) < 0.01, "{dd} {dl}");
}

/// On the legacy basis the switch is on by default and the two lines are
/// bit for bit what they were at 8750b09.
#[test]
fn the_system_lines_are_on_by_default_and_unchanged_on_the_legacy_basis() {
    let m = legacy_model();
    let s = m.s_b;
    let b = m.balance(Scenario::RecF, &m.opts(s)).unwrap();
    let q = boak_query(&m);
    assert!(q.system_lines);
    let k = m.cost_items(&b, &q).unwrap();
    let h_s = s * b.c.h_scale;
    assert_eq!(k.steam_smr_mm.to_bits(), (m.export_conv * h_s).to_bits());
    assert_eq!(
        k.smr_power_y.to_bits(),
        (m.e_conv * h_s * m.hours * m.price_e / 1e3).to_bits()
    );
}

/// The plant-built reference prices one gas and divides by one abatement
/// (rule 1 for the floor): the gas of `floor_reference` is the gas that the
/// floor emission implies through the carbon identity of N.7.3, and the
/// abatement is the net abatement of the PLANT_FLOOR row.
#[test]
fn the_plant_built_floor_is_one_reference_for_gas_and_abatement() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let ab = m.abatement(&b, s).unwrap();
    let fl = m.floor_reference(&b, s).unwrap().expect("plant ledger");
    let row = ab.rows.iter().find(|r| r.id == "PLANT_FLOOR").unwrap();
    assert_eq!(fl.a_net.to_bits(), row.a_net.to_bits());
    assert_eq!(fl.kg_kg.to_bits(), row.kg_kg_ref.to_bits());
    // Carbon identity of N.7.3: E_ref / k = cbar n_ref - P - (1 - v) K'.
    let rf = m.reference_ledger().unwrap();
    let k = cf::tonnes_per_kmol_h(m.hours) * s;
    let v = m.carbon_params.cond_vent;
    let k_ref = rf.lines().condensate * b.c.h_scale;
    let carbon = row.e_ref_t_y / k + b.c.p + (1.0 - v) * k_ref;
    let gas = carbon / ng_carbon_per_mol() / mmscfd_to_kmol_h(1.0) * s;
    assert!(rel(fl.gas_mmscfd, gas) < 1e-9, "{} {gas}", fl.gas_mmscfd);
    // The floor lies below the generic intensity, so its abatement and its gas
    // are the lower ones and its cost per tonne the higher one.
    let central = ab.rows.iter().find(|r| r.id == ab.central).unwrap();
    assert!(fl.a_net < central.a_net);
    assert!(fl.gas_mmscfd < ab.decomposition.unwrap().n_ref_mmscfd);
    let q = boak_query(m);
    let qf = nush2_plant::economics::CostQuery {
        ng_smr: Some(fl.gas_mmscfd),
        ..q
    };
    let l5 = m.ledger(&b, &q).unwrap().l5() * 1e6 / central.a_net;
    let l5_floor = m.ledger(&b, &qf).unwrap().l5() * 1e6 / fl.a_net;
    assert!(l5_floor > l5, "{l5_floor} {l5}");
    // The legacy carbon has no plant ledger and so no floor.
    let lm = legacy_model();
    let lb = lm.balance(Scenario::RecF, &lm.opts(lm.s_b)).unwrap();
    assert!(lm.floor_reference(&lb, lm.s_b).unwrap().is_none());
}

fn design_basis_cell(id: &str, col: usize) -> String {
    let t = default_tables();
    for line in t.final_design_basis.lines().skip(1) {
        let c = split_cells(line);
        if c[0] == id {
            return c[col].clone();
        }
    }
    panic!("row {id} missing");
}

fn design_basis_value(id: &str) -> f64 {
    let cell = design_basis_cell(id, 3);
    cell.parse().unwrap_or_else(|_| panic!("{id}: {cell:?}"))
}

fn stage_of(id: &str) -> nush2_plant::economics::Stage {
    nush2_plant::economics::Stage::ALL
        .into_iter()
        .find(|s| s.id() == id)
        .unwrap()
}

/// The rows printed beside the headline: the cost under the plant-built floor
/// and without the two system lines are the engine values, and the cost the
/// two lines add is the difference of the two headline rows.
#[test]
fn the_exported_floor_and_system_line_rows_are_the_engine_values() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let fl = m.floor_reference(&b, s).unwrap().unwrap();
    let a = m.emissions(&b.c, s).a_net;
    let q = boak_query(m);
    let qf = nush2_plant::economics::CostQuery {
        ng_smr: Some(fl.gas_mmscfd),
        ..q
    };
    let qn = boak_nosys_query(m);
    let tag = "BOAK_25Y_N1P00";
    let close = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() <= 1e-5_f64.max(1e-9 * want.abs()),
            "{what}: {got} against {want}"
        );
    };
    close(
        design_basis_value(&format!("FLOOR_L5_{tag}")),
        m.ledger(&b, &qf).unwrap().l5() * 1e6 / fl.a_net,
        "floor L5",
    );
    close(
        design_basis_value(&format!("FLOOR_LAC_DCF_{tag}")),
        m.dcf_at(&b, &qf, fl.a_net).unwrap().lac,
        "floor DCF",
    );
    close(
        design_basis_value(&format!("NOSYS_L5_{tag}")),
        m.ledger(&b, &qn).unwrap().l5() * 1e6 / a,
        "no system lines L5",
    );
    close(
        design_basis_value(&format!("NOSYS_LAC_DCF_{tag}")),
        m.dcf(&b, &qn).unwrap().lac,
        "no system lines DCF",
    );
    // The effect row is the headline row less the row without the lines.
    let head_l5 = design_basis_value(&format!("AC_L5_{tag}"));
    let head_dcf = design_basis_value(&format!("LAC_DCF_{tag}"));
    close(
        design_basis_value(&format!("SYSLINES_L5_{tag}")),
        head_l5 - design_basis_value(&format!("NOSYS_L5_{tag}")),
        "effect of the lines on L5",
    );
    close(
        design_basis_value(&format!("SYSLINES_LAC_DCF_{tag}")),
        head_dcf - design_basis_value(&format!("NOSYS_LAC_DCF_{tag}")),
        "effect of the lines on the DCF",
    );
    // The floor costs more than the headline, whatever the stage and life.
    for stage in ["FOAK", "BOAK", "10OAK_P46", "10OAK_P47"] {
        for life in [25, 60] {
            let t = format!("{stage}_{life}Y_N1P00");
            let floor = design_basis_value(&format!("FLOOR_L5_{t}"));
            let qs = nush2_plant::economics::CostQuery::central(
                stage_of(stage),
                f64::from(life),
                m.n_linear,
                m.r_rated,
            );
            let head = m.ledger(&b, &qs).unwrap().l5() * 1e6 / a;
            assert!(floor > head, "{t}: {floor} {head}");
        }
    }
}

/// Break-even shares printed under the floor are the engine solution, and a
/// share above the printed one passes the strict test (decision D3). A target
/// that no share reaches prints an empty cell.
#[test]
fn the_exported_floor_break_even_shares_meet_the_target_strictly() {
    use nush2_plant::subsidy::{Definition, GrantQuery};
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let fl = m.floor_reference(&b, s).unwrap().unwrap();
    let q = nush2_plant::economics::CostQuery {
        ng_smr: Some(fl.gas_mmscfd),
        ..boak_query(m)
    };
    let timing = m.subsidy.default_timing;
    let mut printed = 0;
    for definition in Definition::ALL {
        let be = m.breakeven(&b, &q, definition, timing, fl.a_net).unwrap();
        let id = format!("FLOOR_BREAKEVEN_{}_BOTH_BOAK_25Y", definition.id());
        let cell = design_basis_cell(&id, 3);
        match be.both.share() {
            Some(share) => {
                let got: f64 = cell.parse().unwrap();
                assert!((got - share).abs() < 1e-6, "{id}: {got} {share}");
                // Above the infimum both methods are strictly below the target.
                let above = (share + 1e-4).min(m.subsidy.share_max);
                let g = GrantQuery {
                    share: above,
                    definition,
                    timing,
                };
                let e = m.subsidy_eval(&b, &q, g, fl.a_net).unwrap();
                assert!(e.target_met, "{id} at {above}");
                printed += 1;
            }
            None => assert!(cell.is_empty(), "{id}: {cell:?} for an unreachable target"),
        }
    }
    // Under the floor, definition B reaches the target.
    assert!(printed >= 1);
}

/// Decision D2 and the two comparison inputs are stated wherever the headline
/// is shown: the cost rows, the central rows of the emissions ledger, every row
/// of the co-funding table and the two inputs.
#[test]
fn the_headline_rows_say_generic_reference_intensity_and_the_inputs_say_comparison() {
    let t = default_tables();
    let words = "generic reference intensity";
    let mut cost_rows = 0;
    for line in t.final_design_basis.lines().skip(1) {
        let c = split_cells(line);
        let id = c[0].as_str();
        if id.starts_with("AC_L5_") || id.starts_with("AC_L3_") || id.starts_with("LAC_DCF_") {
            cost_rows += 1;
            assert!(c[2].contains(words), "{id}: {}", c[2]);
        }
        if matches!(
            id,
            "RES_ABATEMENT_T_Y" | "RES_BASELINE_T_Y" | "RES_ABATEMENT_DIRECT_T_Y"
        ) {
            assert!(c[2].contains(words), "{id}: {}", c[2]);
        }
        if id == "EXPORT_STEAM_CONV" || id == "E_CONV_NOCC" {
            assert!(c[11].contains("comparison input"), "{id}: {}", c[11]);
            assert!(c[11].starts_with("TEV-953 Table 2 doc p29;"), "{id}");
        }
    }
    assert_eq!(cost_rows, 3 * 12, "AC_L5, AC_L3 and LAC_DCF over twelve points");
    let mut central_rows = 0;
    for line in t.emissions_ledger.lines().skip(1) {
        let c = split_cells(line);
        if c[1] == "IEAGHG_9p00" && c[2] == "independent_carbon" && c[4].contains("(central") {
            central_rows += 1;
            assert!(c[4].contains(words), "{line}");
        }
    }
    assert!(central_rows >= 4, "{central_rows}");
    for line in t.subsidy_scenarios.lines().skip(1) {
        let c = split_cells(line);
        assert!(c[31].contains(words), "{line}");
    }
    for line in t.economic_ledger.lines().skip(1) {
        let c = split_cells(line);
        match c[6].as_str() {
            "GAS" => assert!(c[7].contains(words), "{line}"),
            "LOST_STEAM" | "BASE_POWER" => assert!(c[7].contains("comparison input"), "{line}"),
            _ => {}
        }
    }
}
