p = "docs/ABATEMENT_SUBSIDY_LOG.md"
s = open(p, encoding="utf-8").read()

# D2 status note
a = "INL (9.21) and Cho (9.35) stay as comparison rows, the target verdict is reported under each, and the intensity built from the plant's own feed is kept as a floor check."
b = a + " Status after fix round 2 (below): the owner has not yet accepted D2 in this form. Wherever the headline is shown it now says generic reference intensity, and the cost and break-even shares under the plant-built floor are printed beside it."
assert s.count(a) == 1
s = s.replace(a, b, 1)

entry = """## Phase 3, fix round 2: carbon track (findings of the second independent verification)

Three major findings were fixed on the existing build. Nothing else was changed: the central abatement, the L0 to L5 lines, the discounted cash flow and every other table value are as they were.

Finding 1, the reference side of the headline is a generic intensity (decision D2). The verifier offered two routes: obtain the owner's acceptance of D2 and carry the wording generic reference intensity, or make the plant-built reference the centre. The second route needs a furnace efficiency and a steam duty for the reference (`REF_FURNACE_EFF`, `REF_STEAM_DUTY`), for which the repository holds no source, so it was not taken. D2 is a recorded decision and the task fixes it as the centre, so it stays. The owner's acceptance of D2 in this form is not on record and is listed below as open.

- Wording: the cost rows of `Final_Design_Basis.csv` (`AC_L5_*`, `AC_L3_*`, `LAC_DCF_*`), the three headline abatement rows, the central rows of `Emissions_Ledger.csv`, the GAS line of `Economic_Ledger.csv`, the source text of every row of `Subsidy_Scenarios.csv` and the labels of the five headline indicators of the wasm reply say generic reference intensity. The legacy export keeps its committed text.
- Beside the headline: `Model::floor_reference` gives the plant-built reference (the floor of N.7.1 and N.7.3, no firing gas) as a cost comparison, its gas for `CostQuery::ng_smr` and its net abatement. `Final_Design_Basis.csv` gains `FLOOR_L5_*`, `FLOOR_LAC_DCF_*` and `FLOOR_BREAKEVEN_{A,B}_{L5,DCF,BOTH}_*` for the four stages at 25 y and 60 y (default timing), and `Validation_Results.csv` gains the reported rows `IND_L5_UNDER_PLANT_FLOOR`, `IND_LAC_DCF_UNDER_PLANT_FLOOR`, `IND_BREAKEVEN_A_UNDER_PLANT_FLOOR` and `IND_BREAKEVEN_B_UNDER_PLANT_FLOOR`.
- Values for B_REC_F, BOAK, 25 y, n = 1.0, as the engine prints them in `Final_Design_Basis.csv`: L5 203.180333 S$/t against 283.076719 under the floor, LAC_DCF 203.500904 against 283.448388, the net abatement 481,083.692586 t/y against 557,767.415516 (reference intensity 7.850175 against 8.995997 kg/kg). The break-even share of definition B (progressive, both methods) is 0.515856 against 0.788615 under the floor. Definition A (0.691256 at the centre) has no break-even share under the floor: even the largest share leaves the cost at or above S$100/t, at every stage and both lives. The verifier's hand figure of 283.1 S$/t for L5 under the floor is confirmed.

Finding 2, LOST_STEAM and BASE_POWER price INL's conventional plant. `EXPORT_STEAM_CONV` and `E_CONV_NOCC` stay as published INL values (TEV-953 Table 2), and no source exists for a plant-built reference duty, so the lines are labelled and reported with and without.

- `CostQuery::system_lines` (default true, the ladder as committed) switches the two lines off in `Model::cost_items`: the reference then exports what the plant exports and buys no power. The ledger and the cash flow read the same items, so both drop the same two lines.
- Labels: on the independent export the source text of the two registry inputs in `Final_Design_Basis.csv` gains the role comparison input, and the line text of LOST_STEAM and BASE_POWER in `Economic_Ledger.csv` says INL conventional plant, comparison input. The registry entries themselves are untouched, because their text belongs to the legacy snapshot; the class and status words (published, primary) keep describing where the numbers come from.
- Beside the headline: `NOSYS_L5_*`, `NOSYS_LAC_DCF_*`, `SYSLINES_L5_*` and `SYSLINES_LAC_DCF_*` in `Final_Design_Basis.csv` and the reported rows `IND_L5_WITHOUT_SYSTEM_LINES` and `IND_LAC_DCF_WITHOUT_SYSTEM_LINES`. Values for the same case: L5 without the two lines 187.361277 S$/t, LAC_DCF 187.681848 S$/t, so the two lines add 15.819055 S$/t to each method (7.8 percent of L5). The verifier's 15.82 S$/t is confirmed.

Finding 3, the BASELINE sensitivity group shifted the abatement alone. `solve` in `engine/plant/src/sens.rs` now selects the baseline of a BASELINE point through `CARBON_BASELINE` on the plant ledger, so the reference gas of the cost follows the baseline (rule 1 of N.7); the old offset of the abatement stays for the anchored carbon, where it keeps the committed legacy values. The `base_value` column prints the central intensity of the basis (8.995997 kg/kg), where it printed INL's 9.209770.

- Effect, B_REC_F, 25 y: AC_L5_25 for the INL point moves from 198.099114 to 190.645146 S$/t and for the Cho point from 194.901764 to 182.757397; AC_L3_25 from 145.501046 to 138.047078 and from 143.152637 to 131.008270; LAC_DCF_25 from 198.411668 to 190.957700 and from 195.209273 to 183.064906. The 60 y metrics move the same way. The IEAGHG point is the central run and does not move. The three points now equal the three points of the CARBON_BASELINE group metric by metric.

Cells of `results/` that changed, and why:

- `Economic_Ledger.csv`: 272 `line` texts (the GAS line of the runs on the plant ledger, and LOST_STEAM and BASE_POWER of every run). No value changed.
- `Emissions_Ledger.csv`: 24 `line` texts (the four central lines of each of the six runs). No value changed.
- `Final_Design_Basis.csv`: 39 `parameter` texts (the 36 cost rows and three headline abatement rows), 2 `source` texts (the two comparison inputs) and 96 new result rows (`FLOOR_*`, `NOSYS_*`, `SYSLINES_*`). No existing value changed.
- `Sensitivity_Results.csv`: 30 `base_value` cells of the BASELINE group, and 12 `result` and 12 `delta` cells (six cost metrics for the INL point and for the Cho point). Finding 3.
- `Subsidy_Scenarios.csv`: 352 `source` texts. No value changed.
- `Validation_Results.csv`: six new reported rows. No existing row changed.
- `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` are byte-identical.

New tests: `rule_1_of_n_7_holds_at_every_sensitivity_point_that_changes_the_baseline` and `legacy_baseline_points_keep_the_committed_offsets` (in `sens.rs`), `rule_1_of_n_7_holds_for_every_central_baseline`, `the_baseline_sensitivity_group_equals_the_carbon_baseline_group`, `the_two_system_lines_vanish_together_when_they_are_switched_off`, `the_system_lines_are_on_by_default_and_unchanged_on_the_legacy_basis`, `the_plant_built_floor_is_one_reference_for_gas_and_abatement`, `the_exported_floor_and_system_line_rows_are_the_engine_values`, `the_exported_floor_break_even_shares_meet_the_target_strictly` and `the_headline_rows_say_generic_reference_intensity_and_the_inputs_say_comparison` (in `m11_carbon.rs`), `the_floor_and_system_line_rows_are_reported_beside_the_independent_headline` (in `benchmark_rows.rs`) and `headline_indicator_labels_name_the_generic_reference_intensity` (wasm). No tolerance was changed.

Outstanding after this round:

- The owner has to accept decision D2 in its present form, or give sources for `REF_FURNACE_EFF` and `REF_STEAM_DUTY` so that the plant-built reference can become the centre. Until then the floor rows are the range that must be read with the headline: at the floor the cost without a grant is above S$100/t at every stage and life, and definition A cannot reach the target with any share.
- LOST_STEAM and BASE_POWER still price INL's conventional plant. They are labelled and reported with and without, and the pairing of the cost level with the carbon view (open issue 3 of the derivation) is the owner's decision.
- The frontend and the site scripts still describe the INL headline and do not yet show the floor rows (Phases 5 and 6).

"""

marker = "## Phase status"
assert s.count(marker) == 1
s = s.replace(marker, entry + marker, 1)
open(p, "w", encoding="utf-8").write(s)
print("ok")
