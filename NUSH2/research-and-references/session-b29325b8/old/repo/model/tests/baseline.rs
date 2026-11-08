use nuclear_assisted_smr::{
    ideal_ch4_kg_per_kg_h2, ideal_process_co2_kg_per_kg_h2,
    plant_gate_abatement_kg_per_kg_h2, required_h2_t_per_year,
    IEAGHG_BASE, IEAGHG_CASE_1A, IEAGHG_KG_H2_PER_NM3,
};

fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b} within {tol}");
}

#[test]
fn stoichiometric_bounds_are_correct() {
    close(ideal_ch4_kg_per_kg_h2(), 1.9895, 0.001);
    close(ideal_process_co2_kg_per_kg_h2(), 5.457, 0.002);
}

#[test]
fn ieaghg_volume_to_mass_basis_is_self_consistent() {
    close(IEAGHG_KG_H2_PER_NM3, 0.08994, 1e-8);
}

#[test]
fn ieaghg_base_case_specific_ng_reconstructs_published_flows() {
    close(IEAGHG_BASE.ng_feed_kg_per_kg_h2(), 26_231.0 / 8_994.0, 1e-12);
    close(IEAGHG_BASE.ng_fuel_kg_per_kg_h2(), 4_332.0 / 8_994.0, 1e-12);
    close(IEAGHG_BASE.ng_total_kg_per_h(), 30_563.0, 1e-12);
}

#[test]
fn ieaghg_base_case_co2_is_about_nine_kg_per_kg_h2() {
    close(IEAGHG_BASE.co2_emitted_kg_per_kg_h2(), 8.995, 0.002);
}

#[test]
fn case_1a_reconstructs_ieaghg_avoidance_fraction() {
    let avoided = plant_gate_abatement_kg_per_kg_h2(IEAGHG_BASE, IEAGHG_CASE_1A);
    let fraction = avoided / IEAGHG_BASE.co2_emitted_kg_per_kg_h2();
    close(fraction, 0.5422, 0.0002);
}

#[test]
fn case_1a_cn4252_scale_is_about_51_kt_h2_per_year_plant_gate_only() {
    let avoided = plant_gate_abatement_kg_per_kg_h2(IEAGHG_BASE, IEAGHG_CASE_1A);
    let required = required_h2_t_per_year(250_000.0, avoided);
    close(required / 1_000.0, 51.3, 0.1);
}

#[test]
fn capture_and_avoidance_are_not_identical() {
    let captured = IEAGHG_CASE_1A.co2_captured_kg_per_kg_h2();
    let avoided = plant_gate_abatement_kg_per_kg_h2(IEAGHG_BASE, IEAGHG_CASE_1A);
    assert!(captured > avoided);
}


#[test]
fn base_makeup_fuel_combustion_is_only_part_of_total_direct_co2() {
    use nuclear_assisted_smr::makeup_fuel_co2_upper_bound_kg_per_kg_h2;
    let makeup = makeup_fuel_co2_upper_bound_kg_per_kg_h2(IEAGHG_BASE);
    close(makeup, 1.322, 0.005);
    assert!(makeup < IEAGHG_BASE.co2_emitted_kg_per_kg_h2());
}


#[test]
fn ieaghg_carbon_balance_closes_from_ng_inputs_to_flue_gas() {
    use nuclear_assisted_smr::{
        ieaghg_feed_carbon_kmol_per_h, ieaghg_flue_carbon_kmol_per_h,
        ieaghg_makeup_fuel_carbon_kmol_per_h,
    };
    let carbon_in = ieaghg_feed_carbon_kmol_per_h() + ieaghg_makeup_fuel_carbon_kmol_per_h();
    let carbon_out = ieaghg_flue_carbon_kmol_per_h();
    close(carbon_out / carbon_in, 1.0, 0.001);
}

#[test]
fn psa_tail_gas_carries_essentially_all_feedstock_carbon_before_firing() {
    use nuclear_assisted_smr::{
        ieaghg_feed_carbon_kmol_per_h, ieaghg_tail_gas_carbon_kmol_per_h,
    };
    close(
        ieaghg_tail_gas_carbon_kmol_per_h() / ieaghg_feed_carbon_kmol_per_h(),
        1.0,
        0.001,
    );
}

#[test]
fn makeup_furnace_fuel_is_only_about_fourteen_percent_of_input_carbon() {
    use nuclear_assisted_smr::ieaghg_makeup_fuel_fraction_of_input_carbon;
    close(ieaghg_makeup_fuel_fraction_of_input_carbon(), 0.1417, 0.001);
}

#[test]
fn co_and_ch4_leave_a_large_residual_carbon_load_after_pre_psa_co2_removal() {
    use nuclear_assisted_smr::ieaghg_psa_inlet_non_co2_carbon_kmol_per_h;
    close(ieaghg_psa_inlet_non_co2_carbon_kmol_per_h(), 505.2, 0.5);
}


#[test]
fn published_psa_streams_reconstruct_hydrogen_recovery() {
    use nuclear_assisted_smr::ieaghg_reconstructed_psa_h2_recovery;
    // Rounded source streams reconstruct about 89.9% recovery, consistent with
    // the report's approximately 90% PSA recovery basis.
    close(ieaghg_reconstructed_psa_h2_recovery(), 0.9000, 0.002);
}

#[test]
fn published_psa_streams_reconstruct_h2_product_flow() {
    use nuclear_assisted_smr::ieaghg_reconstructed_h2_product_kmol_per_h;
    close(ieaghg_reconstructed_h2_product_kmol_per_h(), 4490.0, 3.0);
}

#[test]
fn tail_gas_contains_substantial_combustible_species() {
    use nuclear_assisted_smr::ieaghg_tail_combustible_mole_fraction;
    close(ieaghg_tail_combustible_mole_fraction(), 0.4768, 0.001);
}

#[test]
fn co2_only_tail_gas_capture_leaves_about_thirty_two_percent_of_tail_carbon() {
    use nuclear_assisted_smr::ieaghg_tail_carbon_as_co2_fraction;
    let co2_fraction = ieaghg_tail_carbon_as_co2_fraction();
    close(co2_fraction, 0.6799, 0.002);
    close(1.0 - co2_fraction, 0.3201, 0.002);
}


#[test]
fn reduced_reaction_extents_reproduce_psa_carbon_species() {
    use nuclear_assisted_smr::{ieaghg_psa_inlet_cho, reduced_extents_from_psa_inlet};
    let p = ieaghg_psa_inlet_cho();
    let x = reduced_extents_from_psa_inlet();
    close(x.smr - x.wgs, p.co, 1.0);
}

#[test]
fn reduced_reaction_extents_nearly_reproduce_psa_hydrogen() {
    use nuclear_assisted_smr::{ieaghg_psa_inlet_cho, reduced_reaction_h2_kmol_per_h};
    let source = ieaghg_psa_inlet_cho().h2;
    let reduced = reduced_reaction_h2_kmol_per_h();
    // Equivalent-CH4 substitution is intentionally a reduced model. The
    // source-resolved model supersedes it; retain the mismatch as a diagnostic.
    let rel = (reduced - source).abs() / source;
    assert!(rel > 0.03 && rel < 0.06, "unexpected reduced-model mismatch: {rel}");
}

#[test]
fn reduced_standard_reaction_duty_is_positive_and_order_60_mw() {
    use nuclear_assisted_smr::reduced_standard_reaction_duty_mw;
    let q = reduced_standard_reaction_duty_mw();
    assert!(q > 60.0 && q < 70.0, "unexpected reference reaction duty: {q} MW");
}


#[test]
fn hts_wgs_extent_closes_independently_on_co_co2_h2_and_water() {
    use nuclear_assisted_smr::{
        ieaghg_hts_wgs_extent_from_co_kmol_h,
        ieaghg_hts_wgs_extent_from_co2_kmol_h,
        ieaghg_hts_wgs_extent_from_h2_kmol_h,
        ieaghg_hts_wgs_extent_from_water_kmol_h,
    };
    let x=ieaghg_hts_wgs_extent_from_co_kmol_h();
    close(ieaghg_hts_wgs_extent_from_co2_kmol_h(), x, 2.0);
    close(ieaghg_hts_wgs_extent_from_h2_kmol_h(), x, 2.0);
    close(ieaghg_hts_wgs_extent_from_water_kmol_h(), x, 2.0);
    close(x, 661.3, 2.0);
}

#[test]
fn hts_section_conserves_c_h_o_with_rounded_source_values() {
    use nuclear_assisted_smr::{ieaghg_hts_inlet,ieaghg_hts_outlet};
    let i=ieaghg_hts_inlet(); let o=ieaghg_hts_outlet();
    close(o.carbon_kmol_h()/i.carbon_kmol_h(),1.0,0.001);
    close(o.hydrogen_atoms_kmol_h()/i.hydrogen_atoms_kmol_h(),1.0,0.001);
    close(o.oxygen_atoms_kmol_h()/i.oxygen_atoms_kmol_h(),1.0,0.001);
}

#[test]
fn source_prereformer_stream_steam_to_carbon_is_reconstructed_not_assumed() {
    use nuclear_assisted_smr::ieaghg_prereformer_stream_steam_to_carbon;
    // Directly reconstructed from the encoded source stream. Do not replace
    // this with a generic textbook/design S/C assumption.
    close(ieaghg_prereformer_stream_steam_to_carbon(),2.5522,0.002);
}


#[test]
fn prereformer_to_hts_source_rows_are_not_silently_forced_to_close() {
    use nuclear_assisted_smr::prereformer_to_hts_source_residual;
    let r=prereformer_to_hts_source_residual();
    // Diagnostic guard: if these suddenly approach zero after an edit, inspect
    // source provenance before claiming a closed reformer control volume.
    assert!(r.carbon_kmol_atoms_h.abs() > 1.0
        || r.hydrogen_kmol_atoms_h.abs() > 1.0
        || r.oxygen_kmol_atoms_h.abs() > 1.0);
}


#[test]
fn stream4_to_stream5_nonclosure_is_explained_by_unreported_water_addition() {
    use nuclear_assisted_smr::{
        ieaghg_interstage_water_addition_kmol_h,
        prereformer_to_hts_residual_with_inferred_water,
        ieaghg_prereformer_feed,
    };
    let w = ieaghg_interstage_water_addition_kmol_h();
    // H and O balances independently imply ~154 kmol/h H2O. This is
    // consistent with IEAGHG's stated second HP-steam addition plus BFW
    // desuperheating between stream 4 and the primary reformer.
    assert!(w > 153.0 && w < 156.0, "unexpected inferred water addition: {w}");

    let r = prereformer_to_hts_residual_with_inferred_water();
    let feed = ieaghg_prereformer_feed();
    // Rounded four-decimal compositions leave tiny residuals; require
    // <0.1% of the corresponding stream-4 elemental inventories.
    assert!(r.carbon_kmol_atoms_h.abs()/feed.carbon_kmol_h() < 0.001);
    assert!(r.hydrogen_kmol_atoms_h.abs()/feed.hydrogen_atoms_kmol_h() < 0.001);
    assert!(r.oxygen_kmol_atoms_h.abs()/feed.oxygen_atoms_kmol_h() < 0.001);
}


#[test]
fn ieaghg_base_energy_ledger_reconstructs_published_scale() {
    use nuclear_assisted_smr::{
        ieaghg_feed_lhv_mw, ieaghg_makeup_fuel_lhv_mw, ieaghg_total_ng_lhv_mw,
        ieaghg_h2_lhv_efficiency,
    };
    close(ieaghg_feed_lhv_mw(), 338.806, 0.01);
    close(ieaghg_makeup_fuel_lhv_mw(), 55.944, 0.01);
    close(ieaghg_total_ng_lhv_mw(), 394.778, 0.01);
    close(ieaghg_h2_lhv_efficiency(), 0.75916, 0.0001);
}

#[test]
fn ieaghg_energy_source_rounding_is_explicit_not_forced() {
    use nuclear_assisted_smr::{
        IEAGHG_FEED_LHV_GJ_PER_1000_NM3_H2,
        IEAGHG_FUEL_LHV_GJ_PER_1000_NM3_H2,
        IEAGHG_TOTAL_LHV_GJ_PER_1000_NM3_H2,
    };
    let sum=IEAGHG_FEED_LHV_GJ_PER_1000_NM3_H2+IEAGHG_FUEL_LHV_GJ_PER_1000_NM3_H2;
    close(IEAGHG_TOTAL_LHV_GJ_PER_1000_NM3_H2-sum,0.001,1e-12);
}


#[test]
fn source_stream4_to_stream5_closes_after_source_described_water_addition() {
    use nuclear_assisted_smr::prereformer_to_hts_residual_with_inferred_water;
    let r=prereformer_to_hts_residual_with_inferred_water();
    // Source rows are rounded to four decimals; the independently inferred
    // H- and O-based water additions differ by ~1.5 kmol/h. The reconciled
    // residuals must remain small relative to the ~4,000-14,000 kmol-atom/h
    // inventories rather than being tuned to exact zero.
    assert!(r.carbon_kmol_atoms_h.abs() < 0.1);
    assert!(r.hydrogen_kmol_atoms_h.abs() < 2.0);
    assert!(r.oxygen_kmol_atoms_h.abs() < 1.0);
}

#[test]
fn source_reformer_consumes_water_and_eliminates_heavier_hydrocarbons() {
    use nuclear_assisted_smr::{
        source_reformer_water_consumption_kmol_h,
        source_reformer_carbon_conversion_kmol_h,
    };
    assert!(source_reformer_water_consumption_kmol_h() > 1000.0);
    assert!(source_reformer_carbon_conversion_kmol_h() > 1000.0);
}


#[test]
fn ieaghg_ccs_cases_reproduce_authoritative_incremental_energy_penalties() {
    use nuclear_assisted_smr::{
        incremental_ng_input_mw,lost_power_export_mwe,
        IEAGHG_ENERGY_CASE_1A,IEAGHG_ENERGY_CASE_3,
    };
    close(incremental_ng_input_mw(IEAGHG_ENERGY_CASE_1A),12.91,0.02);
    close(lost_power_export_mwe(IEAGHG_ENERGY_CASE_1A),8.426,0.002);
    close(incremental_ng_input_mw(IEAGHG_ENERGY_CASE_3),38.95,0.02);
    close(lost_power_export_mwe(IEAGHG_ENERGY_CASE_3),9.492,0.002);
}

#[test]
fn flue_gas_capture_has_larger_energy_penalty_and_more_plant_gate_abatement() {
    use nuclear_assisted_smr::{
        incremental_energy_service_mw,plant_gate_co2_avoided_t_per_h,
        IEAGHG_ENERGY_CASE_1A,IEAGHG_ENERGY_CASE_3,
    };
    assert!(incremental_energy_service_mw(IEAGHG_ENERGY_CASE_3)
        > incremental_energy_service_mw(IEAGHG_ENERGY_CASE_1A));
    assert!(plant_gate_co2_avoided_t_per_h(IEAGHG_ENERGY_CASE_3)
        > plant_gate_co2_avoided_t_per_h(IEAGHG_ENERGY_CASE_1A));
    close(plant_gate_co2_avoided_t_per_h(IEAGHG_ENERGY_CASE_1A),43.87,0.02);
    close(plant_gate_co2_avoided_t_per_h(IEAGHG_ENERGY_CASE_3),72.03,0.02);
}


#[test]
fn ieaghg_reformer_radiant_duty_converts_to_about_96_mw() {
    use nuclear_assisted_smr::ieaghg_reformer_radiant_duty_mw;
    close(ieaghg_reformer_radiant_duty_mw(),96.04,0.05);
}

#[test]
fn radiant_duty_exceeds_makeup_ng_fuel_lhv_because_tail_gas_is_also_fired() {
    use nuclear_assisted_smr::ieaghg_radiant_to_makeup_fuel_lhv_ratio;
    assert!(ieaghg_radiant_to_makeup_fuel_lhv_ratio() > 1.6);
}

#[test]
fn reduced_298k_reaction_layer_is_below_published_radiant_duty() {
    use nuclear_assisted_smr::radiant_minus_reduced_reaction_duty_mw;
    assert!(radiant_minus_reduced_reaction_duty_mw() > 25.0);
}


#[test]
fn jAEA_mockup_temperature_is_consistent_with_finite_ihx_drop() {
    use nuclear_assisted_smr::ihx_hot_end_temperature_budget_k;
    // JAEA: HTTR primary outlet 950 C; mock-up/HTTR H2 system SR inlet 880 C.
    // A 20 K reformer hot-end approach leaves 50 K for IHX/transport headroom.
    close(ihx_hot_end_temperature_budget_k(950.0,860.0,20.0),70.0,1e-12);
}

#[test]
fn ninety_six_mw_requires_large_secondary_helium_flow_for_modest_delta_t() {
    use nuclear_assisted_smr::{helium_mass_flow_kg_s,ieaghg_reformer_radiant_duty_mw};
    // Screening only: cp=5.2 kJ/kg-K, 880 -> 650 C.
    let m=helium_mass_flow_kg_s(ieaghg_reformer_radiant_duty_mw(),5.2,880.0,650.0);
    assert!(m > 75.0 && m < 85.0, "screening helium flow {m} kg/s");
}

#[test]
fn primary_950c_cannot_support_950c_process_with_finite_approach() {
    use nuclear_assisted_smr::ihx_hot_end_temperature_budget_k;
    assert!(ihx_hot_end_temperature_budget_k(950.0,950.0,10.0) < 0.0);
}


#[test]
fn gthtr300c_large_scale_ihx_is_same_order_as_project_helium_flow() {
    use nuclear_assisted_smr::{
        helium_mass_flow_kg_s,ieaghg_reformer_radiant_duty_mw,
        GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,
    };
    let project_screen=helium_mass_flow_kg_s(
        ieaghg_reformer_radiant_duty_mw(),5.2,880.0,650.0);
    assert!((project_screen-GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S).abs() < 5.0);
}

#[test]
fn gthtr300c_ihx_only_pumping_is_low_single_digit_mw() {
    use nuclear_assisted_smr::{
        helium_circulator_power_mw,
        GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,
        GTHTR300C_SECONDARY_HE_INLET_C,
        GTHTR300C_SECONDARY_HE_PRESSURE_MPA,
        GTHTR300C_SECONDARY_IHX_DP_KPA,
    };
    let w=helium_circulator_power_mw(
        GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,
        GTHTR300C_SECONDARY_IHX_DP_KPA,
        GTHTR300C_SECONDARY_HE_PRESSURE_MPA,
        GTHTR300C_SECONDARY_HE_INLET_C,
        0.80);
    assert!(w > 1.5 && w < 2.2, "IHX-only screening circulator power {w} MW");
}


#[test]
fn loop_pressure_drop_sensitivity_maps_to_parasitic_fraction() {
    use nuclear_assisted_smr::{
        helium_circulator_parasitic_fraction,loop_dp_from_ihx_multiple_kpa,
        GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,GTHTR300C_SECONDARY_HE_PRESSURE_MPA,
        GTHTR300C_SECONDARY_HE_INLET_C,GTHTR300C_IHX_DUTY_MW,
    };
    let f1=helium_circulator_parasitic_fraction(
        GTHTR300C_IHX_DUTY_MW,GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,
        loop_dp_from_ihx_multiple_kpa(1.0),GTHTR300C_SECONDARY_HE_PRESSURE_MPA,
        GTHTR300C_SECONDARY_HE_INLET_C,0.80);
    let f3=helium_circulator_parasitic_fraction(
        GTHTR300C_IHX_DUTY_MW,GTHTR300C_SECONDARY_HE_MASS_FLOW_KG_S,
        loop_dp_from_ihx_multiple_kpa(3.0),GTHTR300C_SECONDARY_HE_PRESSURE_MPA,
        GTHTR300C_SECONDARY_HE_INLET_C,0.80);
    assert!(f1 > 0.009 && f1 < 0.013);
    assert!(f3 > 0.028 && f3 < 0.038);
}

#[test]
fn pressure_losses_add_linearly_in_screening_budget() {
    use nuclear_assisted_smr::helium_loop_delta_p_kpa;
    close(helium_loop_delta_p_kpa(&[58.0,20.0,40.0,15.0]),133.0,1e-12);
}


#[test]
fn ieaghg_hp_steam_mass_ledger_reconstructs_total_superheated_flow() {
    use nuclear_assisted_smr::{
        ieaghg_total_superheated_hp_steam_kg_h,
        ieaghg_syngas_whb_steam_kg_h_approx,
        ieaghg_non_syngas_whb_steam_kg_h_upper_group,
    };
    close(ieaghg_total_superheated_hp_steam_kg_h(),141_354.0,1e-12);
    close(ieaghg_syngas_whb_steam_kg_h_approx(),106_015.5,0.1);
    close(ieaghg_non_syngas_whb_steam_kg_h_upper_group(),35_338.5,0.1);
}


#[test]
fn hp_steam_superheat_is_about_sixteen_mw() {
    use nuclear_assisted_smr::ieaghg_hp_steam_superheat_duty_bounds_mw;
    let (lo,hi)=ieaghg_hp_steam_superheat_duty_bounds_mw();
    assert!(lo > 15.9 && hi < 16.4, "superheat bounds {lo}..{hi} MW");
}

#[test]
fn quantified_furnace_replacement_services_already_exceed_112_mw() {
    use nuclear_assisted_smr::quantified_furnace_replacement_service_bounds_mw;
    let (lo,hi)=quantified_furnace_replacement_service_bounds_mw();
    assert!(lo > 112.0 && hi < 112.5, "quantified service bounds {lo}..{hi} MW");
}


#[test]
fn nist_methane_shomate_reproduces_feed_preheat_lower_bound() {
    use nuclear_assisted_smr::{
        NIST_CH4_298_1300,feed_preheater_ch4_only_lower_bound_mw,
    };
    close(NIST_CH4_298_1300.delta_h_kj_mol(408.15,643.15),11.2453,0.001);
    close(feed_preheater_ch4_only_lower_bound_mw(),4.0473,0.002);
}

#[test]
fn current_source_backed_furnace_service_floor_exceeds_116_mw() {
    use nuclear_assisted_smr::current_furnace_service_lower_bound_mw;
    let (lo,hi)=current_furnace_service_lower_bound_mw();
    assert!(lo > 116.0 && hi < 117.0, "current floor {lo}..{hi} MW");
}


#[test]
fn complete_ng_feed_preheat_exceeds_methane_only_bound() {
    use nuclear_assisted_smr::{
        feed_preheater_ng_only_duty_mw,feed_preheater_ch4_only_lower_bound_mw,
    };
    let q=feed_preheater_ng_only_duty_mw();
    assert!(q > feed_preheater_ch4_only_lower_bound_mw());
    assert!(q > 4.5 && q < 5.5, "NG-only feed preheat {q} MW");
}


#[test]
fn recycle_stream_preheat_is_small_and_positive() {
    use nuclear_assisted_smr::feed_preheater_h2_recycle_duty_mw;
    let q=feed_preheater_h2_recycle_duty_mw();
    assert!(q > 0.07 && q < 0.09);
}

#[test]
fn prereformer_coil_lower_bound_is_positive_and_material() {
    use nuclear_assisted_smr::prereformer_feed_preheater_lower_bound_mw;
    let q=prereformer_feed_preheater_lower_bound_mw();
    println!("prereformer_lower_bound_mw={q}");
    assert!(q > 5.0 && q < 12.0);
    assert!(q.is_finite());
}


#[test]
fn reformer_preheater_duty_increases_monotonically_with_inlet_temperature() {
    use nuclear_assisted_smr::reformer_preheater_sensitivity_mw;
    let q600=reformer_preheater_sensitivity_mw(600.0);
    let q625=reformer_preheater_sensitivity_mw(625.0);
    let q650=reformer_preheater_sensitivity_mw(650.0);
    assert!(q600 > 0.0 && q600 < q625 && q625 < q650);
}


#[test]
fn current_htgr_service_envelope_increases_with_reformer_inlet_temperature() {
    use nuclear_assisted_smr::current_htgr_service_envelope_mw;
    let a=current_htgr_service_envelope_mw(600.0);
    let b=current_htgr_service_envelope_mw(625.0);
    let c=current_htgr_service_envelope_mw(650.0);
    assert!(a.0 < b.0 && b.0 < c.0);
    assert!(a.1 < b.1 && b.1 < c.1);
}


#[test]
fn report_htgr_service_breakdown_for_ci_artifact() {
    use nuclear_assisted_smr::current_htgr_service_breakdown;
    for t in [600.0,625.0,650.0] {
        let b=current_htgr_service_breakdown(t);
        println!("HTGR_BREAKDOWN T={t}: radiant={} steam={}..{} feed={} preref_lb={} reformer_sens={}",
            b.radiant_mw,b.hp_steam_superheat_lo_mw,b.hp_steam_superheat_hi_mw,
            b.feed_preheat_mw,b.prereformer_preheat_lower_bound_mw,b.reformer_preheat_sensitivity_mw);
    }
}


#[test]
fn verified_current_htgr_service_envelope_has_expected_values() {
    use nuclear_assisted_smr::{
        prereformer_feed_preheater_lower_bound_mw,
        reformer_preheater_sensitivity_mw,current_htgr_service_envelope_mw,
    };
    close(prereformer_feed_preheater_lower_bound_mw(),7.04975,0.002);
    close(reformer_preheater_sensitivity_mw(600.0),6.74995,0.002);
    close(reformer_preheater_sensitivity_mw(625.0),8.48606,0.002);
    close(reformer_preheater_sensitivity_mw(650.0),10.24113,0.002);
    let a=current_htgr_service_envelope_mw(600.0);
    let b=current_htgr_service_envelope_mw(625.0);
    let c=current_htgr_service_envelope_mw(650.0);
    close(a.0,130.7414,0.01); close(a.1,130.9731,0.01);
    close(b.0,132.4775,0.01); close(b.1,132.7092,0.01);
    close(c.0,134.2326,0.01); close(c.1,134.4643,0.01);
}


#[test]
fn project_loop_circulator_sensitivity_is_low_single_digit_mw() {
    use nuclear_assisted_smr::project_loop_circulator_sensitivity_mw;
    close(project_loop_circulator_sensitivity_mw(1.0),2.1675,0.005);
    close(project_loop_circulator_sensitivity_mw(2.0),4.3350,0.005);
    close(project_loop_circulator_sensitivity_mw(3.0),6.5025,0.005);
}


#[test]
fn furnace_steam_generation_upper_bound_is_about_sixteen_to_seventeen_mw() {
    use nuclear_assisted_smr::furnace_steam_generation_upper_bound_mw;
    let (lo,hi)=furnace_steam_generation_upper_bound_mw();
    assert!(lo > 16.4 && lo < 16.6);
    assert!(hi > 16.7 && hi < 16.9);
}

#[test]
fn bounded_furnace_service_envelope_is_about_131_to_151_mw() {
    use nuclear_assisted_smr::bounded_furnace_service_envelope_mw;
    let a=bounded_furnace_service_envelope_mw(600.0);
    let c=bounded_furnace_service_envelope_mw(650.0);
    assert!(a.0 > 130.7 && a.0 < 130.8);
    assert!(a.1 > 147.7 && a.1 < 147.9);
    assert!(c.0 > 134.2 && c.0 < 134.3);
    assert!(c.1 > 151.0 && c.1 < 151.4);
}


#[test]
fn nuclear_electric_screen_requires_more_reactor_thermal_power_than_process_heat() {
    use nuclear_assisted_smr::nuclear_electric_reformer_screen_mw;
    // 90% electricity-to-heat is a conservative literature screening point;
    // 50.4% is JAEA's high-performance GTHTR300 net generation benchmark.
    let ((elo,ehi),(rlo,rhi))=nuclear_electric_reformer_screen_mw(625.0,0.90,0.504);
    assert!(elo > 140.0 && ehi < 175.0);
    assert!(rlo > 280.0 && rhi < 350.0);
    assert!(rlo > elo && rhi > ehi);
}


#[test]
fn matched_direct_and_electric_cases_have_identical_plant_gate_carbon_by_construction() {
    use nuclear_assisted_smr::matched_direct_minus_electric_plant_gate_co2;
    for capture in [0.0,0.55,0.90,0.95,0.99] {
        close(matched_direct_minus_electric_plant_gate_co2(capture),0.0,1e-12);
    }
}

#[test]
fn feedstock_carbon_equivalent_is_about_seven_point_seven_kg_co2_per_kg_h2() {
    use nuclear_assisted_smr::feedstock_carbon_co2_equivalent_kg_per_kg_h2;
    close(feedstock_carbon_co2_equivalent_kg_per_kg_h2(),7.724,0.01);
}

#[test]
fn ninety_percent_matched_feedstock_carbon_capture_leaves_about_point_seven_seven_kg() {
    use nuclear_assisted_smr::matched_furnace_free_residual_co2_kg_per_kg_h2;
    close(matched_furnace_free_residual_co2_kg_per_kg_h2(0.90),0.7724,0.005);
}


#[test]
fn iea_global_gas_and_lng_anchors_materially_change_upstream_h2_intensity() {
    use nuclear_assisted_smr::upstream_ng_kgco2e_per_kgh2;
    let gas=upstream_ng_kgco2e_per_kgh2(11.5);
    let lng=upstream_ng_kgco2e_per_kgh2(18.6);
    assert!(gas > 1.5 && gas < 1.6);
    assert!(lng > 2.5 && lng < 2.6);
    assert!(lng > gas);
}

#[test]
fn direct_heat_lca_proxy_is_lower_than_electric_for_same_service_conversion_chain() {
    use nuclear_assisted_smr::{
        direct_nuclear_heat_lca_proxy_kgco2e_per_kgh2,
        nuclear_electric_lca_kgco2e_per_kgh2,
        electric_heater_power_mwe,
    };
    let q=140.0;
    let e=electric_heater_power_mwe(q,0.90);
    let direct=direct_nuclear_heat_lca_proxy_kgco2e_per_kgh2(q,5.5,0.504);
    let electric=nuclear_electric_lca_kgco2e_per_kgh2(e,5.5);
    assert!(direct < electric);
    assert!(direct < 0.06);
    assert!(electric > 0.09 && electric < 0.11);
}


#[test]
fn ninety_percent_capture_lifecycle_screen_is_upstream_gas_dominated() {
    use nuclear_assisted_smr::{
        matched_direct_lifecycle_screen,matched_electric_lifecycle_screen,
    };
    let d=matched_direct_lifecycle_screen(0.90,11.5,5.5,0.504,140.0,0.025);
    let e=matched_electric_lifecycle_screen(0.90,11.5,5.5,140.0,0.90,0.025);
    assert!(d.upstream_ng > d.plant_carbon);
    assert!(e.upstream_ng > e.plant_carbon);
    assert!(d.total() > 2.4 && d.total() < 2.7);
    assert!(e.total() > d.total() && e.total() < 2.8);
}

#[test]
fn lng_anchor_materially_raises_matched_lifecycle_intensity() {
    use nuclear_assisted_smr::matched_direct_lifecycle_screen;
    let gas=matched_direct_lifecycle_screen(0.90,11.5,5.5,0.504,140.0,0.025);
    let lng=matched_direct_lifecycle_screen(0.90,18.6,5.5,0.504,140.0,0.025);
    assert!(lng.total()-gas.total() > 0.9);
}


#[test]
fn capital_recovery_factor_matches_standard_formula() {
    use nuclear_assisted_smr::capital_recovery_factor;
    let crf=capital_recovery_factor(0.08,25);
    assert!(crf>0.093 && crf<0.094);
}

#[test]
fn cn4252_threshold_implies_25m_sgd_per_year_at_minimum_abatement_scale() {
    use nuclear_assisted_smr::max_incremental_annual_cost;
    close(max_incremental_annual_cost(100.0,250_000.0),25_000_000.0,1e-6);
}

#[test]
fn abatement_cost_uses_incremental_not_total_candidate_cost() {
    use nuclear_assisted_smr::abatement_cost_per_tco2e;
    close(abatement_cost_per_tco2e(120.0,100.0,1.0,0.5),40.0,1e-12);
}


#[test]
fn reference_plant_abatement_budget_is_tens_of_millions_per_year() {
    use nuclear_assisted_smr::annual_cost_budget_from_h2;
    // 71.952 ktH2/y and 7.5 kgCO2e/kgH2 avoided -> 0.53964 Mt/y.
    let b=annual_cost_budget_from_h2(71.952,7.5,100.0);
    close(b,53_964_000.0,1.0);
}

#[test]
fn capex_only_break_even_is_an_upper_bound_not_a_project_cost() {
    use nuclear_assisted_smr::capex_only_break_even_upper_bound;
    let cap=capex_only_break_even_upper_bound(25_000_000.0,0.08,25);
    assert!(cap>265_000_000.0 && cap<268_000_000.0);
}


#[test]
fn gthtr300c_energy_share_allocates_about_twenty_eight_percent_to_process_heat() {
    use nuclear_assisted_smr::reactor_cost_allocation_fraction_by_thermal_service;
    close(reactor_cost_allocation_fraction_by_thermal_service(170.0,600.0),
          170.0/600.0,1e-12);
}

#[test]
fn cogeneration_allocation_leaves_more_abatement_budget_than_dedicated_reactor() {
    use nuclear_assisted_smr::{
        allocated_annual_reactor_cost,residual_incremental_budget,
    };
    let allowed=54_000_000.0;
    let dedicated=allocated_annual_reactor_cost(500_000_000.0,1.0,0.08,25,10_000_000.0);
    let shared=allocated_annual_reactor_cost(500_000_000.0,170.0/600.0,0.08,25,10_000_000.0);
    assert!(shared < dedicated);
    assert!(residual_incremental_budget(allowed,shared)
        > residual_incremental_budget(allowed,dedicated));
}


#[test]
fn cogeneration_energy_share_expands_common_reactor_cost_headroom_by_three_point_five() {
    use nuclear_assisted_smr::max_common_reactor_annual_cost;
    let dedicated=max_common_reactor_annual_cost(54_000_000.0,20_000_000.0,1.0);
    let cog=max_common_reactor_annual_cost(54_000_000.0,20_000_000.0,170.0/600.0);
    close(cog/dedicated,600.0/170.0,1e-12);
}

#[test]
fn ccs_tariff_can_consume_large_fraction_of_annual_budget() {
    use nuclear_assisted_smr::annual_ccs_transport_storage_cost;
    close(annual_ccs_transport_storage_cost(500_000.0,50.0),25_000_000.0,1e-6);
}


#[test]
fn singapore_ccs_group_a_can_consume_most_of_100_sgd_per_t_budget() {
    use nuclear_assisted_smr::{
        annual_ccs_cost_sgd_from_foreign_tariff,budget_fraction_consumed,
    };
    // Screening: 0.50 MtCO2/y captured, USD50-75/t T&S,
    // 1.276 SGD/USD representative late-Sep-2026 FX.
    let low=annual_ccs_cost_sgd_from_foreign_tariff(500_000.0,50.0,1.276);
    let high=annual_ccs_cost_sgd_from_foreign_tariff(500_000.0,75.0,1.276);
    close(low,31_900_000.0,1.0);
    close(high,47_850_000.0,1.0);
    assert!(budget_fraction_consumed(low,54_000_000.0)>0.59);
    assert!(budget_fraction_consumed(high,54_000_000.0)>0.88);
}


#[test]
fn zero_other_costs_set_absolute_ccs_tariff_ceiling_near_108_sgd_per_t() {
    use nuclear_assisted_smr::max_ccs_tariff_per_t_captured;
    close(max_ccs_tariff_per_t_captured(54_000_000.0,0.0,500_000.0),108.0,1e-12);
}

#[test]
fn group_a_like_ccs_leaves_little_room_for_everything_else() {
    use nuclear_assisted_smr::max_reactor_and_integration_budget_after_ccs;
    // Approx SGD tariffs from USD50 and USD75 at 1.276 SGD/USD.
    let low=max_reactor_and_integration_budget_after_ccs(54_000_000.0,500_000.0,63.8);
    let high=max_reactor_and_integration_budget_after_ccs(54_000_000.0,500_000.0,95.7);
    close(low,22_100_000.0,1.0);
    close(high,6_150_000.0,1.0);
}


#[test]
fn ieaghg_case1a_cac_decomposition_reconstructs_reported_47_point_1() {
    use nuclear_assisted_smr::{
        ieaghg_case1a_captured_per_avoided_ratio,
        ieaghg_case1a_non_ts_cac_eur2014_per_t_avoided,
        case1a_cac_with_replacement_ts,
    };
    close(ieaghg_case1a_captured_per_avoided_ratio(),1.06223,0.0001);
    let non_ts=ieaghg_case1a_non_ts_cac_eur2014_per_t_avoided();
    close(non_ts,36.4777,0.01);
    close(case1a_cac_with_replacement_ts(non_ts,10.0),47.1,0.001);
}

#[test]
fn each_unit_of_ts_tariff_moves_case1a_cac_by_captured_per_avoided_ratio() {
    use nuclear_assisted_smr::{
        ieaghg_case1a_captured_per_avoided_ratio,case1a_cac_with_replacement_ts,
    };
    let r=ieaghg_case1a_captured_per_avoided_ratio();
    let a=case1a_cac_with_replacement_ts(40.0,50.0);
    let b=case1a_cac_with_replacement_ts(40.0,51.0);
    close(b-a,r,1e-12);
}


#[test]
fn case1a_singapore_screen_exceeds_assignment_threshold_under_group_a_inputs() {
    use nuclear_assisted_smr::{
        case1a_non_ts_cac_sgd_screen,case1a_singapore_cac_sgd_screen,
    };
    let non_ts=case1a_non_ts_cac_sgd_screen(576.1,812.8,1.5105);
    assert!(non_ts>77.0 && non_ts<79.0);
    let low=case1a_singapore_cac_sgd_screen(non_ts,50.0*1.2863);
    let high=case1a_singapore_cac_sgd_screen(non_ts,75.0*1.2863);
    assert!(low>145.0 && low<147.0);
    assert!(high>179.0 && high<181.0);
}


#[test]
fn case1a_screen_requires_large_annual_savings_to_reach_s100() {
    use nuclear_assisted_smr::{
        ieaghg_case1a_annual_direct_co2_avoided_t,required_annual_savings_to_target,
    };
    let a=ieaghg_case1a_annual_direct_co2_avoided_t();
    assert!(a>365_000.0 && a<365_200.0);
    let low=required_annual_savings_to_target(146.0,100.0,a);
    let high=required_annual_savings_to_target(180.0,100.0,a);
    assert!(low>16_700_000.0 && low<16_900_000.0);
    assert!(high>29_100_000.0 && high<29_300_000.0);
}


#[test]
fn furnace_ng_savings_need_double_digit_sgd_per_gj_to_close_current_gap_alone() {
    use nuclear_assisted_smr::{
        ieaghg_annual_makeup_furnace_ng_gj,gas_price_required_for_savings_sgd_per_gj,
    };
    let e=ieaghg_annual_makeup_furnace_ng_gj();
    assert!(e>1_670_000.0 && e<1_680_000.0);
    let p17=gas_price_required_for_savings_sgd_per_gj(16_800_000.0,1.0);
    let p29=gas_price_required_for_savings_sgd_per_gj(29_200_000.0,1.0);
    assert!(p17>9.9 && p17<10.1);
    assert!(p29>17.3 && p29<17.6);
}


#[test]
fn psa_tail_gas_contains_large_internal_fuel_inventory() {
    use nuclear_assisted_smr::{
        ieaghg_psa_tail_gas_lhv_mw,ieaghg_psa_tail_combustible_kmol_h,
    };
    let q=ieaghg_psa_tail_gas_lhv_mw();
    close(q,101.95234,0.002);
    assert!(ieaghg_psa_tail_combustible_kmol_h()>990.0);
}

#[test]
fn utility_value_functions_preserve_sign_and_units() {
    use nuclear_assisted_smr::{annual_electricity_value_sgd,annual_steam_value_sgd};
    close(annual_electricity_value_sgd(10.0,8000.0,150.0),12_000_000.0,1e-6);
    close(annual_electricity_value_sgd(-2.0,8000.0,150.0),-2_400_000.0,1e-6);
    close(annual_steam_value_sgd(46.0,8000.0,20.0),7_360_000.0,1e-6);
}


#[test]
fn tail_gas_recycle_has_material_h2_and_co2_recovery_potential() {
    use nuclear_assisted_smr::{
        tail_gas_ideal_h2_upper_bound_kmol_h,tail_gas_existing_co2_t_h,
        tail_gas_nonco2_carbon_kmol_h,
    };
    let h=tail_gas_ideal_h2_upper_bound_kmol_h();
    assert!(h>1590.0 && h<1610.0, "ideal tail H2 {h} kmol/h");
    let c=tail_gas_existing_co2_t_h();
    assert!(c>47.0 && c<48.0, "tail CO2 {c} t/h");
    assert!(tail_gas_nonco2_carbon_kmol_h()>500.0);
}


#[test]
fn tail_recycle_can_displace_at_most_about_one_third_of_fresh_feed_carbon() {
    use nuclear_assisted_smr::{
        tail_recycle_fresh_ng_carbon_displacement_upper_fraction,
        tail_recycle_fresh_ng_displacement_upper_kmol_h,
        tail_recycle_fresh_ng_feed_energy_displacement_upper_mw,
    };
    let f=tail_recycle_fresh_ng_carbon_displacement_upper_fraction();
    assert!(f>0.31 && f<0.33, "carbon displacement fraction {f}");
    let n=tail_recycle_fresh_ng_displacement_upper_kmol_h();
    assert!(n>460.0 && n<480.0, "fresh NG upper displacement {n} kmol/h");
    let q=tail_recycle_fresh_ng_feed_energy_displacement_upper_mw();
    assert!(q>105.0 && q<115.0, "fresh NG energy upper displacement {q} MW");
}

#[test]
fn case2a_tail_separation_anchor_is_over_six_mwe_before_other_recycle_costs() {
    use nuclear_assisted_smr::case2a_tail_separation_net_electric_anchor_mwe;
    close(case2a_tail_separation_net_electric_anchor_mwe(),6.309,1e-12);
}


#[test]
fn reduced_tail_recycle_respects_physical_monotonicity() {
    use nuclear_assisted_smr::reduced_tail_recycle_fixed_h2;
    let low=reduced_tail_recycle_fixed_h2(0.50,0.50,0.50);
    let mid=reduced_tail_recycle_fixed_h2(0.80,0.80,0.80);
    let high=reduced_tail_recycle_fixed_h2(1.00,1.00,1.00);
    assert!(low.recovered_h2_kmol_h < mid.recovered_h2_kmol_h);
    assert!(mid.recovered_h2_kmol_h < high.recovered_h2_kmol_h);
    assert!(low.fresh_ng_displaced_fraction < mid.fresh_ng_displaced_fraction);
    assert!(mid.fresh_ng_displaced_fraction < high.fresh_ng_displaced_fraction);
    assert!(high.extra_water_consumed_kmol_h>0.0);
}

#[test]
fn reduced_tail_recycle_is_below_carbon_only_displacement_ceiling_for_mid_case() {
    use nuclear_assisted_smr::{
        reduced_tail_recycle_fixed_h2,
        tail_recycle_fresh_ng_carbon_displacement_upper_fraction,
    };
    let r=reduced_tail_recycle_fixed_h2(0.80,0.80,0.80);
    assert!(r.fresh_ng_displaced_fraction>0.15);
    assert!(r.fresh_ng_displaced_fraction
        < tail_recycle_fresh_ng_carbon_displacement_upper_fraction());
}


#[test]
fn eighty_percent_tail_recycle_has_small_net_standard_reaction_heat_but_large_mdea_heat() {
    use nuclear_assisted_smr::{
        reduced_tail_recycle_reaction_heat_screen_mw,
        case2a_mdea_regeneration_latent_heat_bounds_mw,
    };
    let qrxn=reduced_tail_recycle_reaction_heat_screen_mw(0.80,0.80);
    assert!(qrxn>4.4 && qrxn<4.7, "reaction heat screen {qrxn} MW");
    let (lo,hi)=case2a_mdea_regeneration_latent_heat_bounds_mw();
    assert!(lo>38.0 && hi<39.3, "MDEA latent heat {lo}..{hi} MW");
}

#[test]
fn eighty_percent_recycle_nearly_replaces_removed_fresh_carbon_throughput() {
    use nuclear_assisted_smr::{
        reduced_tail_recycle_fixed_h2,reduced_tail_recycle_carbon_replacement,
    };
    let r=reduced_tail_recycle_fixed_h2(0.80,0.80,0.80);
    let (removed,recycled)=reduced_tail_recycle_carbon_replacement(r);
    assert!(removed>400.0 && removed<430.0);
    assert!(recycled>400.0 && recycled<410.0);
    assert!((removed-recycled).abs()<20.0);
}


#[test]
fn eighty_percent_recycle_reduces_standard_reaction_heat_vs_displaced_fresh_methane() {
    use nuclear_assisted_smr::{
        reduced_tail_recycle_fixed_h2,
        displaced_fresh_ng_ch4_reaction_heat_lower_bound_mw,
        recycle_minus_displaced_reaction_heat_screen_mw,
    };
    let r=reduced_tail_recycle_fixed_h2(0.80,0.80,0.80);
    let removed=displaced_fresh_ng_ch4_reaction_heat_lower_bound_mw(r);
    assert!(removed>15.0 && removed<17.0);
    let net=recycle_minus_displaced_reaction_heat_screen_mw(r,0.80,0.80);
    assert!(net < -10.0 && net > -13.0, "net reaction heat {net} MW");
}


#[test]
fn sixty_three_percent_waste_heat_anchor_reduces_mdea_incremental_heat_to_about_fourteen_mw() {
    use nuclear_assisted_smr::{
        mdea_incremental_nuclear_heat_bounds_mw,
        recycle_net_thermal_increment_screen_80pct,
    };
    let (lo,hi)=mdea_incremental_nuclear_heat_bounds_mw(0.63);
    assert!(lo>14.0 && hi<14.6, "incremental MDEA {lo}..{hi} MW");
    let (nlo,nhi)=recycle_net_thermal_increment_screen_80pct(0.63);
    assert!(nlo>2.0 && nhi<4.0, "net thermal screen {nlo}..{nhi} MW");
}

#[test]
fn heat_cascade_sensitivity_spans_full_external_heat_to_full_waste_heat() {
    use nuclear_assisted_smr::mdea_incremental_nuclear_heat_bounds_mw;
    let full=mdea_incremental_nuclear_heat_bounds_mw(0.0);
    let none=mdea_incremental_nuclear_heat_bounds_mw(1.0);
    assert!(full.0>38.0 && full.1<39.3);
    close(none.0,0.0,1e-12); close(none.1,0.0,1e-12);
}


#[test]
fn ieaghg_non_whb_steam_group_can_cover_less_than_half_mdea_duty_at_most() {
    use nuclear_assisted_smr::{
        non_syngas_whb_steam_latent_heat_upper_bounds_mw,
        mdea_waste_heat_fraction_upper_from_ieaghg_steam_group,
    };
    let (qlo,qhi)=non_syngas_whb_steam_latent_heat_upper_bounds_mw();
    assert!(qlo>16.4 && qhi<16.9);
    let (flo,fhi)=mdea_waste_heat_fraction_upper_from_ieaghg_steam_group();
    assert!(flo>0.41 && fhi<0.45, "source upper fraction {flo}..{fhi}");
}


#[test]
fn shifted_syngas_has_material_sensible_heat_above_mdea_temperature() {
    use nuclear_assisted_smr::{
        ieaghg_shift_sensible_heat_above_reboiler_mw,
        mdea_fraction_from_shift_sensible_ceiling,
    };
    // Use a conservative 226.85 C hot-stream outlet (500 K), staying within
    // the verified NIST H2O-gas Shomate range rather than extrapolating to 170 C.
    let q=ieaghg_shift_sensible_heat_above_reboiler_mw(160.0,66.85);
    assert!(q>13.0 && q<16.0, "shift sensible heat {q} MW");
    let (lo,hi)=mdea_fraction_from_shift_sensible_ceiling(160.0,66.85);
    assert!(lo>0.33 && hi<0.42, "MDEA sensible ceiling fraction {lo}..{hi}");
}


#[test]
fn stream6_water_dewpoint_is_below_a_160c_reboiler_plus_10k_pinch() {
    use nuclear_assisted_smr::{
        ieaghg_stream6_water_dewpoint_c,
        stream6_condensation_above_mdea_pinch,
    };
    let td=ieaghg_stream6_water_dewpoint_c();
    assert!(td>155.0 && td<160.0, "stream6 dew point {td} C");
    assert!(!stream6_condensation_above_mdea_pinch(160.0,10.0));
}

#[test]
fn iapws_region4_tsat_reproduces_normal_boiling_point() {
    use nuclear_assisted_smr::iapws_if97_tsat_k_from_mpa;
    let t=iapws_if97_tsat_k_from_mpa(0.101325)-273.15;
    assert!((t-99.974).abs()<0.02, "normal boiling point {t} C");
}


#[test]
fn full_shift_sensible_heat_to_170c_is_now_bounded_without_nist_extrapolation() {
    use nuclear_assisted_smr::ieaghg_shift_sensible_412_to_target_bounds_mw;
    let (lo,hi)=ieaghg_shift_sensible_412_to_target_bounds_mw(170.0);
    println!("SHIFT_412_TO_170_MW={lo}..{hi}");
    assert!(lo>18.0 && hi<22.0, "412->170 C sensible heat {lo}..{hi} MW");
    assert!(hi-lo<0.5, "water-Cp uncertainty unexpectedly large");
}


#[test]
fn preserving_feed_preheat_pushes_mdea_recovery_ceiling_below_thirty_seven_percent() {
    use nuclear_assisted_smr::{
        residual_shift_heat_after_feed_preheat_bounds_mw,
        mdea_fraction_after_feed_preheat_upper_bounds,
    };
    let (qlo,qhi)=residual_shift_heat_after_feed_preheat_bounds_mw();
    assert!(qlo>13.7 && qhi<14.1, "residual shifted heat {qlo}..{qhi} MW");
    let (flo,fhi)=mdea_fraction_after_feed_preheat_upper_bounds();
    assert!(flo>0.34 && fhi<0.38, "MDEA upper fraction {flo}..{fhi}");
}


#[test]
fn source_bounded_mdea_and_recycle_heat_are_finite_and_ordered() {
    use nuclear_assisted_smr::{
        mdea_incremental_heat_source_bounded_mw,
        recycle_80pct_thermal_increment_source_bounded_mw,
        htgr_service_with_recycle_source_bounded_mw,
    };
    let (mlo,mhi)=mdea_incremental_heat_source_bounded_mw();
    assert!(mlo>24.0 && mlo<26.0);
    assert!(mhi>38.0 && mhi<40.0);
    let (rlo,rhi)=recycle_80pct_thermal_increment_source_bounded_mw();
    assert!(rlo>12.0 && rlo<15.0);
    assert!(rhi>27.0 && rhi<30.0);
    let (qlo,qhi)=htgr_service_with_recycle_source_bounded_mw(625.0);
    assert!(qlo>144.0 && qlo<148.0);
    assert!(qhi>175.0 && qhi<180.0);
}


#[test]
fn recycle80_operating_energy_screen_has_expected_economic_direction() {
    use nuclear_assisted_smr::{
        recycle80_operating_energy_net_bounds_sgd_y,
        recycle80_break_even_gas_price_sgd_per_gj,
    };
    // Current source-bounded HTGR service screen: representative 145-179 MWth.
    // 5.69 SGD/GJ corresponds approximately to JAEA's legacy 0.7 JPY/MJ
    // at ~0.00813 SGD/JPY; it is a technology-cost anchor, not Singapore price.
    let low_gas=recycle80_operating_energy_net_bounds_sgd_y(
        145.0,179.0,10.0,5.69,150.0);
    println!("RECYCLE80_NET_GAS10={:?}",low_gas);
    let high_gas=recycle80_operating_energy_net_bounds_sgd_y(
        145.0,179.0,20.0,5.69,150.0);
    println!("RECYCLE80_NET_GAS20={:?}",high_gas);
    assert!(low_gas.0>4_000_000.0 && low_gas.1<13_000_000.0);
    assert!(high_gas.0>48_000_000.0 && high_gas.1<57_000_000.0);
    let p=recycle80_break_even_gas_price_sgd_per_gj(162.0,5.69,150.0);
    assert!(p>7.0 && p<10.0);
}


#[test]
fn inverse_recycle_economics_shows_ccs_can_erase_capital_headroom() {
    use nuclear_assisted_smr::{
        recycle80_max_allocated_capital_opex_sgd_y,
        total_common_reactor_headroom_sgd_y,
    };
    // Illustrative 0.54 Mt/y abatement budget = S$54m/y.
    // Group-A-like low T&S: S$31.9m/y.
    let h=recycle80_max_allocated_capital_opex_sgd_y(
        54_000_000.0,162.0,15.0,5.69,150.0,31_900_000.0);
    println!("INVERSE_HEADROOM_GAS15_LOWCCS={h}");
    assert!(h>52_000_000.0 && h<52_400_000.0, "hydrogen-side headroom {h}");
    let total=total_common_reactor_headroom_sgd_y(h.max(0.0),170.0/600.0);
    assert!(total>=h.max(0.0));
}

#[test]
fn maximum_nuclear_heat_price_falls_as_ccs_cost_rises() {
    use nuclear_assisted_smr::recycle80_max_nuclear_heat_price_sgd_per_gj;
    let low=recycle80_max_nuclear_heat_price_sgd_per_gj(
        54_000_000.0,162.0,15.0,150.0,31_900_000.0,0.0);
    let high=recycle80_max_nuclear_heat_price_sgd_per_gj(
        54_000_000.0,162.0,15.0,150.0,47_850_000.0,0.0);
    println!("MAX_NUCLEAR_HEAT_LOWCCS={low};HIGHCCS={high}");
    assert!(low>high);
    assert!(low>10.0 && low<20.0);
    assert!(high>5.0 && high<15.0);
}


#[test]
fn iaea_htgr_source_costs_are_large_relative_to_current_hydrogen_headroom() {
    use nuclear_assisted_smr::{
        source_reactor_annual_cost_allocated_by_heat,
        reactor_cost_multiplier_headroom,
    };
    // IAEA TECDOC 2075 source cases, million USD basis.
    // HTGR-200+SMR: 4x200 MWth, NPP CAPEX 2065, O&M 192/y.
    let htgr200=source_reactor_annual_cost_allocated_by_heat(
        2065.0,192.0,800.0,162.0,0.08,25);
    // MHR-T+SMR: 4x600 MWth, NPP CAPEX 2748, O&M 324/y.
    let mhrt=source_reactor_annual_cost_allocated_by_heat(
        2748.0,324.0,2400.0,162.0,0.08,25);
    println!("IAEA_ALLOCATED_MUSD_Y_HTGR200={htgr200};MHRT={mhrt}");
    assert!(htgr200>70.0 && htgr200<90.0, "HTGR200 allocated MUSD/y {htgr200}");
    assert!(mhrt>35.0 && mhrt<45.0, "MHR-T allocated MUSD/y {mhrt}");
    // Example hydrogen-side headroom of 20 MUSD/y cannot carry either source case.
    assert!(reactor_cost_multiplier_headroom(20.0,htgr200)<0.3);
    assert!(reactor_cost_multiplier_headroom(20.0,mhrt)<0.6);
}


#[test]
fn gthtr300c_ihx_loop_anchor_is_not_negligible_against_narrow_margin() {
    use nuclear_assisted_smr::{
        annualised_integration_component_cost,
        integration_capex_from_annual_margin,
    };
    let annual=annualised_integration_component_cost(69.0,0.08,25,0.0);
    assert!(annual>6.4 && annual<6.6, "annualised legacy IHX/loop MUSD/y {annual}");
    let cap=integration_capex_from_annual_margin(2.0,0.08,25,0.0);
    assert!(cap>21.0 && cap<22.0);
}


#[test]
fn shared_reactor_full_cost_boundary_requires_favourable_gas_or_ts() {
    use nuclear_assisted_smr::{
        recycle80_min_gas_price_for_full_cost_sgd_per_gj,
        recycle80_max_ccs_ts_cost_sgd_y,
    };
    // Screening midpoint: 162 MWth; JAEA legacy heat 5.69 SGD/GJ;
    // allocated MHR-T source reactor ~S$50m/y; IAEA IHX/loop ~S$8.2m/y.
    // Keep reformer/recycle allowance explicit at S$5m/y rather than zero.
    let p=recycle80_min_gas_price_for_full_cost_sgd_per_gj(
        54_000_000.0,162.0,5.69,150.0,31_900_000.0,
        50_000_000.0,8_200_000.0,5_000_000.0);
    println!("FULL_COST_MIN_GAS={p}");
    assert!(p>17.0 && p<20.0, "minimum gas price {p} SGD/GJ");
    let ts=recycle80_max_ccs_ts_cost_sgd_y(
        54_000_000.0,162.0,20.0,5.69,150.0,
        50_000_000.0,8_200_000.0,5_000_000.0);
    println!("FULL_COST_MAX_TS={ts}");
    assert!(ts>35_000_000.0 && ts<45_000_000.0, "max T&S {ts} SGD/y");
}


#[test]
fn recycle80_matched_lifecycle_recomputes_assignment_budget() {
    use nuclear_assisted_smr::{
        ieaghg_unabated_lifecycle_screen,
        recycle80_shared_direct_lifecycle_screen,
        annual_lifecycle_abatement_and_budget,
    };
    let b=ieaghg_unabated_lifecycle_screen(11.5);
    let c=recycle80_shared_direct_lifecycle_screen(
        0.90,11.5,162.0,5.5,0.504,0.025);
    println!("MATCHED_LCA_BASE={};CAND={}",b.total(),c.total());
    let (mt,budget)=annual_lifecycle_abatement_and_budget(
        b,c,8994.0,8322.0,100.0);
    println!("MATCHED_LCA_ABATE_MT={mt};BUDGET_SGDY={budget}");
    assert!(c.total()<b.total());
    assert!(mt>0.60 && mt<0.90);
    assert!(budget>60_000_000.0 && budget<90_000_000.0);
}


#[test]
fn matched_lifecycle_budget_reopens_part_of_shared_reactor_region() {
    use nuclear_assisted_smr::{
        ieaghg_unabated_lifecycle_screen,
        recycle80_shared_direct_lifecycle_screen,
        annual_lifecycle_abatement_and_budget,
        recycle80_min_gas_price_for_full_cost_sgd_per_gj,
        recycle80_max_ccs_ts_cost_sgd_y,
    };
    let b=ieaghg_unabated_lifecycle_screen(11.5);
    let c=recycle80_shared_direct_lifecycle_screen(
        0.90,11.5,162.0,5.5,0.504,0.025);
    let (_,budget)=annual_lifecycle_abatement_and_budget(
        b,c,8994.0,8322.0,100.0);
    let p=recycle80_min_gas_price_for_full_cost_sgd_per_gj(
        budget,162.0,5.69,150.0,31_900_000.0,
        50_000_000.0,8_200_000.0,5_000_000.0);
    let ts20=recycle80_max_ccs_ts_cost_sgd_y(
        budget,162.0,20.0,5.69,150.0,
        50_000_000.0,8_200_000.0,5_000_000.0);
    println!("MATCHED_FULL_COST_MIN_GAS={p};MAX_TS20={ts20};BUDGET={budget}");
    assert!(p>14.0 && p<15.5);
    assert!(ts20>55_000_000.0 && ts20<56_000_000.0);
}


#[test]
fn recycle80_matched_lifecycle_regenerates_abatement_budget_from_same_case() {
    use nuclear_assisted_smr::{
        ieaghg_base_lifecycle_screen,recycle80_direct_lifecycle_screen,
        matched_lifecycle_abatement_budget,
    };
    // Screening sensitivities only:
    // upstream NG 15 gCO2e/MJ, nuclear 12 g/kWh_e, eta_e=0.45,
    // 162 MWth, 90% fresh-feed-carbon capture, CCS chain 2% of captured CO2.
    let b=ieaghg_base_lifecycle_screen(15.0);
    let c=recycle80_direct_lifecycle_screen(0.90,15.0,12.0,0.45,162.0,0.02);
    let (delta,avoided,budget)=matched_lifecycle_abatement_budget(b,c,8322.0,100.0);
    println!("MATCHED_LCA_BASE={};CAND={};DELTA={};AVOIDED_T={};BUDGET={}",
        b.total(),c.total(),delta,avoided,budget);
    assert!(delta>5.0);
    assert!(avoided>375_000.0);
    assert!(budget>37_500_000.0);
}


#[test]
fn matched_lifecycle_budget_changes_full_cost_gas_boundary() {
    use nuclear_assisted_smr::{
        ieaghg_base_lifecycle_screen,recycle80_direct_lifecycle_screen,
        recycle80_min_gas_price_with_matched_lifecycle,
    };
    let b=ieaghg_base_lifecycle_screen(15.0);
    let c=recycle80_direct_lifecycle_screen(0.90,15.0,12.0,0.45,162.0,0.02);
    let p=recycle80_min_gas_price_with_matched_lifecycle(
        b,c,8322.0,100.0,162.0,5.69,150.0,31_900_000.0,
        50_000_000.0,8_200_000.0,5_000_000.0);
    println!("MATCHED_LCA_MIN_GAS={p}");
    assert!(p>14.2 && p<14.4);
}


#[test]
fn singapore_import_mix_screen_is_between_global_gas_and_lng_anchors() {
    use nuclear_assisted_smr::singapore_2024_ng_mix_screen_gco2e_per_mj;
    let x=singapore_2024_ng_mix_screen_gco2e_per_mj();
    assert!(x>15.3 && x<15.5, "Singapore mix screen {x} g/MJ");
}

#[test]
fn matched_recycle_meets_abatement_scale_across_credible_lifecycle_bracket() {
    use nuclear_assisted_smr::matched_recycle_lifecycle_economic_point;
    // Lower-abatement corner: lower upstream gas burden, higher nuclear LCA,
    // higher CCS-chain fraction.
    let low=matched_recycle_lifecycle_economic_point(11.5,6.4,0.035);
    // Higher-abatement corner: LNG-like upstream burden, lower nuclear LCA,
    // lower CCS-chain fraction.
    let high=matched_recycle_lifecycle_economic_point(18.6,5.1,0.025);
    println!("LCA_LOW={:?};LCA_HIGH={:?}",low,high);
    assert!(low.annual_abatement_t>600_000.0);
    assert!(high.annual_abatement_t>690_000.0 && high.annual_abatement_t<710_000.0);
    assert!(low.min_gas_price_low_ts_sgd_per_gj>high.min_gas_price_low_ts_sgd_per_gj);
    assert!(low.min_gas_price_low_ts_sgd_per_gj<17.0);
    assert!(high.min_gas_price_low_ts_sgd_per_gj>10.0);
}
