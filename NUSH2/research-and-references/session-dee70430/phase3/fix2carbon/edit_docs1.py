p = "docs/derivation/01_carbon_and_abatement.md"
s = open(p, encoding="utf-8").read()

reps = []

# N.7.3 implementation and validation lines
reps.append((
    "INL's 52.5 MMSCFD is a comparison input of the cost (`CostQuery::ng_smr`) and the legacy value.\n\nValidation test: `eq_n_7_3_reference_identity_on_the_inl_gas_flows` `the_implied_gas_reproduces_inls_gas_when_given_inls_intensity`, `inls_reference_gas_is_a_comparison_with_a_reported_cost_effect` and the validation rows `IND_N73_INL_UNABATED`, `IND_N73_INL_HTGR_NOCC`, `IND_GAS_LINE_UNDER_INL_GAS`, `IND_L5_UNDER_INL_GAS` and `IND_LAC_DCF_UNDER_INL_GAS` in `engine/plant/tests/m11_carbon.rs` and `engine/vv/src/harness.rs`.",
    "INL's 52.5 MMSCFD is a comparison input of the cost (`CostQuery::ng_smr`) and the legacy value. `Model::floor_reference` gives the plant-built reference (the floor of N.7.1 and N.7.3 without firing gas) as a cost comparison: its gas to price through `CostQuery::ng_smr` and its net abatement to divide by. The central result stays on the generic reference intensity of decision D2; `reference_cost_block` in `engine/plant/src/export.rs` prints the cost and the break-even shares under the floor beside the headline (rows `FLOOR_L5_*`, `FLOOR_LAC_DCF_*` and `FLOOR_BREAKEVEN_*` of `Final_Design_Basis.csv`).\n\nValidation test: `eq_n_7_3_reference_identity_on_the_inl_gas_flows` `the_implied_gas_reproduces_inls_gas_when_given_inls_intensity`, `inls_reference_gas_is_a_comparison_with_a_reported_cost_effect`, `rule_1_of_n_7_holds_for_every_central_baseline`, `the_plant_built_floor_is_one_reference_for_gas_and_abatement`, `the_exported_floor_and_system_line_rows_are_the_engine_values`, `the_exported_floor_break_even_shares_meet_the_target_strictly` and the validation rows `IND_N73_INL_UNABATED`, `IND_N73_INL_HTGR_NOCC`, `IND_GAS_LINE_UNDER_INL_GAS`, `IND_L5_UNDER_INL_GAS`, `IND_LAC_DCF_UNDER_INL_GAS`, `IND_L5_UNDER_PLANT_FLOOR`, `IND_LAC_DCF_UNDER_PLANT_FLOOR`, `IND_BREAKEVEN_A_UNDER_PLANT_FLOOR` and `IND_BREAKEVEN_B_UNDER_PLANT_FLOOR` in `engine/plant/tests/m11_carbon.rs` and `engine/vv/src/harness.rs`.",
))

# N.7.4 implementation and validation lines
reps.append((
    "Rust implementation: `closed_form::system_terms` in `engine/plant/src/abatement.rs` and the legacy `Model::system_adjust` in `engine/plant/src/emissions.rs`, which prints the system view.\n\nValidation test: `eq_n_7_4_system_terms_are_added_once` in `engine/plant/tests/m11_carbon.rs`.",
    "Rust implementation: `closed_form::system_terms` in `engine/plant/src/abatement.rs` and the legacy `Model::system_adjust` in `engine/plant/src/emissions.rs`, which prints the system view. The two cost lines that correspond to these terms, LOST_STEAM and BASE_POWER, price the steam export and the purchased power of INL's conventional plant (`EXPORT_STEAM_CONV` and `E_CONV_NOCC`, TEV-953 Table 2) scaled to the plant. They are comparison inputs, because no source gives the steam duty or the power of a plant-built reference. `CostQuery::system_lines` (default true, the ladder as committed) switches both off in `Model::cost_items`, so the ledger and the cash flow read the same cost without them; `Final_Design_Basis.csv` prints that cost beside the headline (rows `NOSYS_*` and `SYSLINES_*`).\n\nValidation test: `eq_n_7_4_system_terms_are_added_once`, `the_two_system_lines_vanish_together_when_they_are_switched_off`, `the_system_lines_are_on_by_default_and_unchanged_on_the_legacy_basis` and the validation rows `IND_L5_WITHOUT_SYSTEM_LINES` and `IND_LAC_DCF_WITHOUT_SYSTEM_LINES` in `engine/plant/tests/m11_carbon.rs` and `engine/vv/src/harness.rs`.",
))

# Rule 1 text: sensitivity points
reps.append((
    "and the saved upstream gas of the lifecycle view (N.9.5) reads the same difference.",
    "and the saved upstream gas of the lifecycle view (N.9.5) reads the same difference. The rule holds at every sensitivity point that changes the baseline: such a point selects the baseline through `CARBON_BASELINE` (`solve` in `engine/plant/src/sens.rs`), so the gas and the abatement follow it together and no point shifts the abatement alone. The tests `rule_1_of_n_7_holds_at_every_sensitivity_point_that_changes_the_baseline` and `the_baseline_sensitivity_group_equals_the_carbon_baseline_group` check the points of the BASELINE and CARBON_BASELINE groups.",
))

# Open issue 2 and 3
reps.append((
    "Until then the reference gas is the gas that the central intensity implies (rule 1 of N.7) or the floor of N.7.1, and INL's gas is a comparison row.",
    "Until then the reference gas is the gas that the central intensity implies (rule 1 of N.7) or the floor of N.7.1, and INL's gas is a comparison row. Status after fix round 2: the central result is the generic reference intensity of decision D2, which includes the firing gas of a real furnace; the floor leaves it out. The headline rows carry the words generic reference intensity, and the cost and the break-even shares under the floor are printed beside the headline. The owner has not yet accepted decision D2 in this form; if it is not accepted, the plant-built reference becomes the centre only after `REF_FURNACE_EFF` and `REF_STEAM_DUTY` have sources.",
))
reps.append((
    "Decide which pair is the headline (L5 with $A_{sys}$, or the ladder without those two lines with $A_{dir}$) and report the other as a second row.",
    "Decide which pair is the headline (L5 with $A_{sys}$, or the ladder without those two lines with $A_{dir}$) and report the other as a second row. Status after fix round 2: the cost without the two lines is printed beside the headline (`NOSYS_*`, `SYSLINES_*`, `IND_L5_WITHOUT_SYSTEM_LINES`), and the two lines are labelled comparison inputs because they price INL's conventional plant; the pairing itself is still the owner's decision.",
))

for a, b in reps:
    assert s.count(a) == 1, a[:60]
    s = s.replace(a, b, 1)
open(p, "w", encoding="utf-8").write(s)
print("ok")
