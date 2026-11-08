
// ------------------------------------- one reference gas for cost and carbon

fn boak_query(m: &Model) -> nush2_plant::economics::CostQuery {
    nush2_plant::economics::CostQuery::central(
        nush2_plant::economics::Stage::Boak,
        25.0,
        m.n_linear,
        m.r_rated,
    )
}

/// Rule 1 of the boundary table of N.7: the GAS line of the cost ledger, the
/// saved upstream gas of the lifecycle view and the avoided-gas term of N.9.2
/// read one pair of gas flows, the reference gas implied by the central
/// intensity and the plant's own gas (derivation test 19).
#[test]
fn rule_1_of_n_7_the_gas_line_the_decomposition_and_the_lifecycle_read_one_gas_difference() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let ab = m.abatement(&b, s).unwrap();
    let d = ab.decomposition.unwrap();
    let q = boak_query(m);
    let k = m.cost_items(&b, &q).unwrap();
    let led = m.ledger(&b, &q).unwrap();

    // The reference gas of the cost items is the gas of the decomposition.
    assert!(
        rel(k.ng_smr, d.n_ref_mmscfd) < 1e-12,
        "{} {}",
        k.ng_smr,
        d.n_ref_mmscfd
    );
    assert!(rel(k.ng_plant, d.n_plant_mmscfd) < 1e-9);
    // GAS line over the price is the gas difference, to 1e-12 of the line.
    let diff = -led.gas / k.gas_s_m_per_mmscfd;
    assert!(rel(diff, d.n_ref_mmscfd - k.ng_plant) < 1e-12);
    // The avoided-gas carbon of N.9.2 is the carbon of the same difference.
    let t_y_per_mmscfd =
        mmscfd_to_kmol_h(1.0) * ng_carbon_per_mol() * cf::tonnes_per_kmol_h(m.hours);
    let from_gas = (d.n_ref_mmscfd - d.n_plant_mmscfd) * t_y_per_mmscfd;
    assert!(rel(d.avoided_gas, from_gas) < 1e-9, "{} {}", d.avoided_gas, from_gas);
    // The saved upstream gas of the lifecycle view is priced on it too.
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let gj = m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    assert!(rel(lc.up_credit_co2e, diff * gj * m.ef_up / 1000.0) < 1e-12);
    // And the case summary prints the same unabated gas.
    let k_sum = m.plant_summary(Scenario::RecF, s).unwrap();
    assert!(rel(k_sum["ng_unabated_mmscfd"], d.n_ref_mmscfd) < 1e-12);
    assert!(rel(k_sum["ng_saved_mmscfd"], diff) < 1e-12);
    assert!(rel(k_sum["gas_saving_SM_y"], -led.gas) < 1e-9);
}

/// INL's 52.5 MMSCFD is a comparison and no longer a model input: pricing it
/// changes the gas line by the gas difference times the price and nothing else,
/// and the plant's own reference is the lower one.
#[test]
fn inls_reference_gas_is_a_comparison_with_a_reported_cost_effect() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let q = boak_query(m);
    let inl = m.ng_conv * s * b.c.h_scale;
    let q_inl = nush2_plant::economics::CostQuery {
        ng_smr: Some(inl),
        ..q
    };
    let k = m.cost_items(&b, &q).unwrap();
    let k_inl = m.cost_items(&b, &q_inl).unwrap();
    let (led, led_inl) = (m.ledger(&b, &q).unwrap(), m.ledger(&b, &q_inl).unwrap());
    // Only the gas of the unabated plant differs.
    assert_eq!(k_inl.ng_smr.to_bits(), inl.to_bits());
    assert!(k.ng_smr < k_inl.ng_smr, "{} {}", k.ng_smr, k_inl.ng_smr);
    let shift = -(inl - k.ng_smr) * k.gas_s_m_per_mmscfd;
    assert!(rel(led_inl.gas - led.gas, shift) < 1e-12);
    assert_eq!(
        led_inl.reactor_capital.to_bits(),
        led.reactor_capital.to_bits()
    );
    assert_eq!(led_inl.lost_steam.to_bits(), led.lost_steam.to_bits());
    // The default cost is higher than the cost under INL's gas, and so is the DCF.
    let a = m.emissions(&b.c, s).a_net;
    let (l5, l5_inl) = (led.l5() * 1e6 / a, led_inl.l5() * 1e6 / a);
    assert!(l5 > l5_inl, "{l5} {l5_inl}");
    assert!(m.dcf(&b, &q).unwrap().lac > m.dcf(&b, &q_inl).unwrap().lac);
    // The shift of L5 is the gas shift over the net abatement, exactly.
    assert!(rel(l5 - l5_inl, -shift * 1e6 / a) < 1e-9);
}

/// Fed with INL's own intensity the implied gas reproduces INL's gas: the
/// carbon identity of N.7.3 closes on the published pair within 0.5 percent
/// (derivation test 16), so the cost does not depend on the copied gas by a
/// different route.
#[test]
fn the_implied_gas_reproduces_inls_gas_when_given_inls_intensity() {
    let mut p = ParamSet::embedded().unwrap();
    p.set("CARBON_BASELINE", 1.0).unwrap();
    let m = Model::new(&p).unwrap();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let n = m.reference_gas_mmscfd(&b.c, s).unwrap();
    let inl = m.ng_conv * s * b.c.h_scale;
    assert!(rel(n, inl) < 0.005, "{n} against {inl}");
    // The cost follows that baseline: its GAS line prices the same gas.
    let k = m.cost_items(&b, &boak_query(&m)).unwrap();
    assert!(rel(k.ng_smr, n) < 1e-15);
}

/// The anchored basis keeps INL's gas bit for bit in the cost items, the
/// summary and the lifecycle terms.
#[test]
fn the_legacy_basis_keeps_inls_reference_gas_bit_for_bit() {
    let m = legacy_model();
    let s = m.s_b;
    let b = m.balance(Scenario::RecF, &m.opts(s)).unwrap();
    let k = m.cost_items(&b, &boak_query(&m)).unwrap();
    assert_eq!(
        k.ng_smr.to_bits(),
        (m.ng_conv * (s * b.c.h_scale)).to_bits()
    );
    assert_eq!(
        m.reference_gas_mmscfd(&b.c, s).unwrap().to_bits(),
        k.ng_smr.to_bits()
    );
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let gj = m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    assert_eq!(
        lc.up_credit_co2e.to_bits(),
        ((m.ng_conv * (s * b.c.h_scale) - b.c.ng * s) * gj * m.ef_up / 1000.0).to_bits()
    );
    let sum = m.case_summary(Scenario::RecF, s).unwrap();
    assert_eq!(
        sum["ng_unabated_mmscfd"].to_bits(),
        (m.ng_conv * s).to_bits()
    );
    assert_eq!(
        sum["ng_saved_mmscfd"].to_bits(),
        ((m.ng_conv - m.ng_c6) * s).to_bits()
    );
}

// --------------------------------------- the CO2e terms are always labelled

/// Decision D4: no CO2e quantity sits under a CO2 label. In the emissions table
/// the lifecycle views are the only place where the CO2e upstream factor and the
/// CO2e nuclear factor enter, and every row of those views that carries one says
/// CO2e, names the warming-potential horizon and the source. No other view
/// mentions CO2e. The check runs over every lifecycle view of every run.
#[test]
fn no_co2e_quantity_appears_without_a_label_in_any_view() {
    let p = ParamSet::embedded().unwrap();
    let m = Model::new(&p).unwrap();
    let t = nush2_plant::export::tables(&p).unwrap();
    let horizon = format!("GWP100 = {}", m.gwp_up);
    let (mut lifecycle_rows, mut other_rows) = (0, 0);
    for line in t.emissions_ledger.lines().skip(1) {
        let c = split_cells(line);
        let (run, view, id, name, source, status) = (&c[0], &c[2], &c[3], &c[4], &c[10], &c[12]);
        if status != nush2_plant::export::CARBON_STATUS {
            continue;
        }
        if view.starts_with("lifecycle_") {
            lifecycle_rows += 1;
            if id == "TS_BURDEN" {
                assert!(
                    !name.contains("CO2e") && name.contains("CO2"),
                    "{run} {line}"
                );
                continue;
            }
            assert!(name.contains("CO2e"), "unlabelled CO2e row: {line}");
            assert!(
                source.contains("CO2e") && source.contains(&horizon),
                "{line}"
            );
            match id.as_str() {
                "UPSTREAM_CREDIT" | "ABATEMENT" if view == "lifecycle_upstream" => {
                    assert!(
                        name.contains(&horizon),
                        "no horizon on the upstream term: {line}"
                    );
                }
                "NUCLEAR_LC" => assert!(name.contains("horizon not stated"), "{line}"),
                _ => {}
            }
        } else {
            other_rows += 1;
            assert!(!name.contains("CO2e"), "CO2e in a CO2 view: {line}");
            assert!(!source.contains("CO2e"), "CO2e in a CO2 view: {line}");
        }
    }
    assert!(lifecycle_rows >= 4 * 3 * 3, "{lifecycle_rows}");
    assert!(other_rows > 100, "{other_rows}");
    // The legacy table keeps the wording of the committed tables.
    let mut pl = ParamSet::embedded().unwrap();
    pl.set("CARBON_BASIS", 0.0).unwrap();
    let tl = nush2_plant::export::tables(&pl).unwrap();
    assert!(!tl.emissions_ledger.contains("CO2e"));
}

/// The lifecycle view of the exporter is equation N.9.5 on the aggregate
/// factor: the sources read state the upstream intensity only as a CO2e total
/// at one horizon, so the aggregate enters in the CO2 slot with the methane term
/// at zero. No CO2/CH4 split is invented.
#[test]
fn eq_n_9_5_the_exported_lifecycle_is_the_closed_form_on_the_aggregate_factor() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let e = m.emissions(&b.c, s);
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let n_ref = m.reference_gas_mmscfd(&b.c, s).unwrap();
    let dg = (n_ref - b.c.ng * s) * m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    let from_form = cf::lifecycle(e.a_net, dg, m.ef_up / 1000.0, 0.0, 0.0, lc.nuclear_co2e);
    let from_terms = e.a_direct + lc.up_credit_co2e - lc.nuclear_co2e - lc.ts_burden;
    assert!(
        rel(from_form, from_terms) < 1e-12,
        "{from_form} {from_terms}"
    );
    // The warming potential is a label of the factor: 100 years, 30 (IEA 2026).
    assert_eq!(m.gwp_up, 30.0);
}
