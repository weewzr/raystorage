# Plan design:carbon, the independent carbon basis

Read-only study of branch feature/independent-abatement-subsidy at 41d8d4b. No cargo command and no engine run was made. Numbers marked (py) come from inline Python on committed values (results CSVs, parameters.toml, the reference table); the engine decides the final values. Line numbers refer to 41d8d4b. The subsidy track has uncommitted edits in the same worktree (anchored.rs, lib.rs, parameters.toml, a new subsidy.rs), so lines in anchored.rs shift by a few.

## 0. Departures from the derivation, to record before coding

- **DV1 central reference.** Derivation 01 defines A_dir = E_ref - E_plant with E_ref from the plant's own unabated reference (N.7.3). Decision D2 (revised after Phase 2) fixes the central E_ref as the IEAGHG intensity (BASELINE_IEAGHG_KG_NM3 / BASELINE_IEAGHG_RHO = 8.99600 kg/kg) times annual H2. Implement N.9.1 with a tagged reference. Implement N.7.1 and N.7.3 as floors only, because N.7.2 needs REF_FURNACE_EFF and REF_STEAM_DUTY and no source exists for either. Floors (py): K.5 5.458, K.6 5.936, N.7.1 with psi'=omega'=0 (1223.99 kmol/h) 6.294 kg/kg. The published 8.996 must sit above all of them.
- **DV2 gas difference.** N.7 rule 1 (one source for the gas difference) cannot hold. The GAS cost line keeps INL's 52.5 MMSCFD (economics.rs:254). The IEAGHG intensity implies 35.09 MMSCFD of reference gas at Case B against 35.87 for INL scaled (-2.17 percent, py). Report a validation row and leave the cost line alone.
- **DV3 T&S ratio.** The flat TS_BURDEN 0.025 t/t gives way to the physical tau_TS of N.9.4 on the independent basis (D7c). It stays for the legacy basis and as a cross-check row.
- No algebraic error found in N.1 to N.10; the quoted factors (327.695 t/y per kmol/h, residual ratio 4.3e-12) reproduce.

## (a) Data structures and signatures

**New `engine/plant/src/carbon_account.rs`** (add to lib.rs). Follow the subsidy track's pattern: one field `carbon_params: CarbonParams` in `Model` and one line `CarbonParams::read(r)?` in `build`, so anchored.rs changes stay small and `unread_params` still passes.

- `CarbonParams { basis, baseline_kind, cryo_f, cryo_r, cond_vent, closure_tol, ts: TsInputs }`.
- `enum CarbonBasis { Legacy, Independent }` (`from_param(f64)` refuses anything but 0 or 1), `enum BaselineKind { Inl, Ieaghg, FleetAverage }` with the ids INL_9p21, IEAGHG_9p00, CHO_9p35 that the Emissions_Ledger already prints.
- `LineStatus { Modelled, Specified, Unknown, OutOfBoundary }`, `Fate { Storage, Air, AirElsewhere, Product, Water, Internal }`, `Line { id, tag, inlet, kmol_c_h: Option<f64>, status, fate, note }`, `Verdict { Closed, Open, Undetermined }`.
- `ClosureReport { lines, residual_kmol_c_h, relative, tol, on_modelled, overall, unknown }` from `closure_report(lines, tol)` (N.6.1, built on `carbon::close_carbon`).
- `const UNMODELLED: [(&str, &str, &str); 9]`, the register U1 to U9.
- Pure closed forms, each doc-commented `/// eq: N.x.y`: `loop_denominator`, `tube_carbon` (N.2.1), `fates` (N.2.2, N.3.1), `gas_rate_from_h2` (N.1.5), `recycle_read_back` (N.4.1), `n2_cut` (N.4.2), `stack_vs_recycle` (N.4.4), `ts_burden_per_t` (N.9.4). They check the numerical flowsheet; they do not replace it.
- `IndepLedger { scn, h2_kg_h, ng_mmscfd, carbon: CarbonLedger, run_id }` and a process-wide cache `OnceLock<Mutex<BTreeMap<IndepKey, Arc<IndepLedger>>>>`. The key holds bit patterns of scn, eta, f, r, xi, pi, T, P, S/C, k_source and the feed basis; BTreeMap keeps hash order out of the output.

**anchored.rs.**
- `LIMITS` (line 477) gains CRYO_C_RECOVERY, CRYO_H2_RETURN, COND_CO2_VENT and CARBON_BASIS (0 to 1, refused, never clamped).
- `Carbon` gains `cond` (S10 CO2, Case A scale) and `basis`. Legacy sets cond = 0. On the independent basis `closure` is the true residual of the ledger lines.
- `Model::carbon` stays the legacy producer, unchanged. New: `carbon_independent(scn, eta) -> Result<Carbon>` (flowsheet ledger divided by s_b, so Case A scale and `tpy(c, s)` keep working), `carbon_active(scn, phi, eta)` (dispatch on the basis; RecFix always legacy), `balance_on(basis, scn, &BalanceOpts)` with `balance` = `balance_on(self.basis, ..)`, `legacy_view() -> Cow<Model>`, `embedded_legacy()`, `carbon_report(scn, s)`.
- In `balance`, `inl` (line 1113) comes from the basis of `c`. Mixed bases would corrupt `t_ratio` and `extra_co2`.
- `case_summary` (line 1407): `feed_C_kmol_h` becomes `c.f*s` (was `f_c6*s`) and `ng_mmscfd` reads `c.ng*s` (was `ng_c6*s`, line 1371). New keys `A_net_t_y`, `A_direct_t_y`, `ts_t_y`, `cond_vent_t_y`, `a_per_kg_net`, `a_per_kg_direct`, `closure_rel`.

**emissions.rs.** `Emissions` gains `ts`, `a_net`, `cond_vent`, `stack`, `baseline_kind`. `Model::emissions(&Carbon, s)` branches on `c.basis`:
- Legacy: bit-identical to today, `a_net = a_atm`, `ts = 0`.
- Independent: baseline = intensity(kind) x h2_t_y; site = tpy(burn) + v x tpy(cond); atm = site + tpy(exported); `ts = tau_TS x captured`; `a_atm = baseline - atm`; `a_net = a_atm - ts`.
- `lifecycle_terms` uses `ts` on the independent basis so T&S is not charged twice. `vent_condensate` (K.8) stays legacy; the S14 sensitivity becomes a sweep of COND_CO2_VENT.
- Cost denominators move from `.a_atm` to `.a_net` (decision D9): lib.rs:177, dcf.rs:198, export.rs:1020, 1108, 3181, sens.rs:214, 446, 555, 606, 1297, 1503, drift.rs:142, wasm solve.rs, vv/golden.rs. Legacy is unaffected.
- sens.rs `Baseline` deltas (line 215) become differences to the active baseline, and `adjust` (line 178) must scale `cond` with `atm`.

**flowsheet.rs.**
- `FlowsheetSpec` gains `tail_gas: TailGas { Cryo, BurnOnSite, Export }`. `FlowsheetSpec::new` keeps today's behaviour; new `with_cryo(m, case, scn, basis)` reads f and r from the parameters.
- `CarbonLedger` gains `unconverted` (S11 carbon not as CO2, feeds `Carbon.unconv`) and `ng_mmscfd`.
- `validate` (line 390) accepts CombH, CombN, Export with f=0, r=0 and still refuses RecFix. `capture_for` (line 1465) maps them to `AnchoredMea`. `base_spec` and `suite` use `with_cryo` on the independent basis and still run `close_h2_return` for reporting.
- The flowsheet works on `m.legacy_view()`: `solve_at_ng` (line 1692) and `carbon_vs_anchor` (line 2287) call `balance_on(Legacy, ..)`. This breaks the recursion balance -> carbon_independent -> solve -> balance and leaves every flowsheet energy row unchanged.

**lib.rs.** `PlantSolution` gains `carbon_report`; `cost_point` and `run` read `a_net`.

## (b) Stream ledger by scenario

| Run | Level 3 ledger today | Plan |
|---|---|---|
| B_REC_F, B_REC_F_MODELLED | FS_B_REC_F | Independent source. The Modelled preset differs in ETA_MEA only, covered by the cache key. |
| A_REC_F | FS_A_REC_F | Independent. Test: FS_B_REC_F / s_b equals FS_A_REC_F to 1e-9 (flowsheet is homogeneous of degree one). |
| A_INL_SEL, B_INL_SEL | FS_A_INL_SEL (34 MMSCFD closure run), FS_B_INL_SEL | Independent from FS_B_INL_SEL / s_b; Selexol 0.99 labelled as INL's published specification. FS_A_INL_SEL remains the calibration run, legacy. |
| B_COMB_H, B_EXPORT, COMB_N (sweep only) | none; `validate` refuses | Extend the run: f=0, r=0, no cryogenic power (the balance purge row is 0), cut = whole reject. COMB_H and EXPORT on the H2 basis; COMB_N on natural gas at ng_c6 x s_b with h_scale = H2 out / H2 target. Export differs in accounting only (S15 carbon goes to `Carbon.exported`). The REC-CRYO record needs a flag saying no cryogenic unit exists. |
| B_REC_FIX | none | Legacy by definition: its stack is pinned to INL's value (anchored.rs:979). Label `legacy_by_definition` in status and source. |

The stack is B = (1-f) G, and the cut holds only 8.0 mol percent N2 now (N.4.2). So f and r carry INL information into every independent result. They are design inputs (D8), reported with a band and checked by the read-back identity N.4.1.

Fallback if the combustion extension slips: keep COMB_H, COMB_N and EXPORT on the legacy basis with the RecFix label; `Carbon.basis` travels with the number, so mixed bases are legal.

## (c) New parameters (engine/data/parameters.toml)

Classes follow params.rs. Source strings must avoid absolute paths, "user decision" with a space and working-note names (no_internal_pointers.rs, check_clean.py). A source that cites docs/derivation/01_carbon_and_abatement.md needs that path added to `PUBLIC_MD` in no_internal_pointers.rs, which the subsidy track is also editing.

| id | value | unit | class, status | source |
|---|---|---|---|---|
| CARBON_BASIS | 1 | - | user_decision, primary | D1, D12; 0 legacy, 1 independent |
| CARBON_BASELINE | 1 | - | user_decision, primary | D2; 0 INL, 1 IEAGHG, 2 fleet average; independent basis only |
| CRYO_C_RECOVERY (f) | 0.752178065 | 1 | assumed, via_reader | D8; J.7 on CO2_CAPTURED_C6, ETA_SELEXOL, PHI_RECYCLE (py). Low 0.729852, high 0.771783 from PHI_RECYCLE 0.79 and 0.76 (py) |
| CRYO_H2_RETURN (r) | closure value to 12 digits, read from a test print (0.921630 at six) | 1 | assumed, via_reader | D8; `close_h2_return` on INL's 34 and 130 MMSCFD pair. Sweep grid plus or minus 0.05, labelled as having no source |
| COND_CO2_VENT (v) | 1 | 1 | assumed, via_reader | N.5.2; unknown fate counts as emission; low 0, high 1 |
| CARBON_CLOSURE_TOL | 1e-9 | 1 | assumed, primary | equals ELEMENT_TOL (flowsheet.rs:193); argument in N.6.1 |
| TS_COND_KWH_T | 19.6 | kWh/t | assumed, primary | T2 Table 4-2 (PDF p26), medium pressure, pre-pressurised feed; S20 leaves at 2015 psia (flowsheet.rs:158). Low 9.0 (T1 Table 3, 15 barg from 90 bara), high 83.1 (T2, non-pressurised) |
| TS_INJ_KWH_T | 5.0 | kWh/t | assumed, primary | T3 (PDF p59), pumping 4 to 5 kWh/t, upper end |
| TS_SHIP_G_T_KM | 6.77 | g fuel/t/km | published, primary | T1 Table 4 (PDF p9), 10 kt ship; low 3.21 (100 kt), high 7.07 (2.5 kt). Round-trip basis checked: T2 gives 263 MWh/day, 15 knots, 10 kt, 11.63 MWh/t, which is 3.39 g/t/km one way and 6.78 round trip (py) |
| TS_SHIP_FUEL_MWH_T | 11.63 | MWh/t | published, primary | T2, marine diesel |
| TS_SHIP_FUEL_EF | 74.1 | t CO2/TJ | published, primary | T8 Table 3.5.2, gas and diesel oil; high 77.4 (residual fuel oil) |
| TS_SHIP_DIST_KM | 1000 | km | assumed, not_verified | Labelled assumption with no source in the repository. T2 evaluates 200 and 1000 km. T10 is a search snippet and supports nothing. Low 200, high 2000 |
| TS_PURGE_FRAC | 0.0009 | t/t | assumed, primary | T4 derived upper bound for purge, cool-down and methane slip |
| TS_TERMINAL_FRAC | 0.00048 | t/t | assumed, primary | T5 operating emissions 720 t/y over 1.5 Mt/y |
| TS_GRID_EF | 0.4080 | t/MWh | published, primary | E9, 2020 row; legacy GRID_EF (0.402, not_verified) is left alone |

tau_TS central = 0.0014 + 0.0210 + 0.0246 x 0.408 = 0.0324 t/t (py); T2 gives 2.0 percent at 1000 km and T4 2.6 percent for its whole chain. Expected independent results (py, approximate): captured 363.3 kt/y, residual direct 32.5 kt/y, A_dir 569.5 kt/y, T&S 11.8 kt/y, A_net 557.8 kt/y, 8.51 kg/kg direct and 8.33 net, reduction 94.6 percent, intensity 0.486 kg/kg.

Baselines reuse BASELINE_IEAGHG_KG_NM3, BASELINE_IEAGHG_RHO, BASELINE_CHO and CO2_CONV_NOCC. The flowsheet's INL set points (871 C, S/C 3.00, 439 psia, N2 and O2 fractions) stay in flowsheet.rs as process assumptions (finding P06); moving them is optional.

## (d) Unknown-stream rule

U1 (MEA make-up and degradation), U2 (vent between S19 and S20), U4b (storage leakage after injection) and U6 (start-up, flaring, fugitive methane) have no sourced value. Each is a `Line` with `kmol_c_h: None` and status Unknown. A parameter cannot hold "unknown" (the parser needs a number), so the register lives in code (`UNMODELLED`). A stream leaves it only when a sourced parameter and a code change arrive together. U7 and U8 are Specified zero (product specification, flash assumption) and say so. U3 is pure CO2, U5 is OutOfBoundary, U9 is the condensate CO2 (Modelled, fate set by v), and U4a (logistics loss) is Modelled through TS_PURGE_FRAC and TS_TERMINAL_FRAC.

`closure_report` returns `on_modelled` (Closed when |rho| <= tol on Modelled and Specified lines, else Open) and `overall` (Undetermined whenever any line is Unknown). Missing carbon is never added as zero.

Unknown air outlets can only lower abatement, so A_dir and A_net are labelled `modelled_streams_only` and treated as upper bounds. The exporter adds `headroom_unknown_t_y`: the unknown air emission that could be absorbed before L5 reaches S$100/t. It turns an unknown into a decision number without inventing a value. A pass at the target is printed as conditional on the unknown list.

## (e) Basis switch, CLI, wasm and the legacy snapshot

- Registry parameter `CARBON_BASIS` in the ParamSet, so `Inputs.overrides`, the CLI and wasm share one path.
- CLI (cli/src/main.rs:30, 167): `export --basis legacy|independent`, default independent, sets the parameter on `ParamSet::embedded()`; `json` and `drift` take the flag. `nush2_vv::harness::run_all` takes the basis: the pinned plant rows run on `embedded_legacy`, new independent rows only when independent. In legacy mode the exporter adds no row and no column that 8750b09 lacked (new Emissions_Ledger views, RES rows, validation rows and input rows are skipped).
- wasm: a new enum Spec `carbon_basis` (Target::Registry("CARBON_BASIS")) in registry.rs. The request key `basis` already means Case A or B; do not reuse it. `live_flowsheet` (solve.rs:457) uses `with_cryo` and skips `base_closure_cached` when independent. One full solve already contains one live flowsheet in 30 to 42 ms (wasm README line 21), which bounds the cost; `native_solve_time_is_recorded` (tests.rs:968) prints the kpi_only median after the change.
- Snapshot: copy the seven `results/*.csv` (unchanged since 8750b09: `git diff 8750b09 HEAD -- results engine` is empty) to `engine/vv/data/legacy_8750b09/` (1.2 MB). `compare_results.py` is strict on header and row count, so add `--subset` (keyed compare, extra rows listed and not failed; keys: id for Final_Design_Basis and Validation_Results, run_id+stream_id, run_id+term_id, the first four columns of Emissions_Ledger, the first seven of Economic_Ledger and Sensitivity_Results) and `--names` to restrict to the seven legacy tables. The subsidy track needs both options; land them once.
- CI (`.github/workflows/ci.yml`, after Test): export with `--basis legacy` to `$RUNNER_TEMP/legacy` and run `compare_results.py --subset --names ... engine/vv/data/legacy_8750b09 $RUNNER_TEMP/legacy`; then export with the default basis and run `compare_results.py results $RUNNER_TEMP/indep`. A Rust test `legacy_export_matches_snapshot` repeats the keyed compare in `cargo test`.
- Failure policy: a flowsheet that does not converge returns `EngineError::NotConverged`. It never falls back to legacy. Sweep rows print `not_converged` (sens.rs:51).

## (f) Existing items that encode copied values or labels

| Where | Encodes | Becomes |
|---|---|---|
| drift.rs:24-25, 243-253 | 578,000 +/-12,000 t/y, blocking | blocking on legacy only; non-blocking reported finding on independent (D10). Finding count stays 8 (m8_level4.rs:281) |
| vv/tests/plant_goldens.rs:50-60, 63-80, 126-147 | A=578052, L3 costs, 59 plant rows | `embedded_legacy`; independent rows added; count constant updated |
| vv/src/harness.rs:243-258, 297, 329 | HEADLINE_ABATEMENT and L3 rows, closure, PARAMS_COMPLETE | stay legacy; independent counterparts reported |
| vv/src/golden.rs:106-136, 221-223, 270-289, 386-389, 456-489, 709-756; data/engine_goldens.json, freeze_values.json | frozen legacy balances and sweeps | `embedded_legacy`; JSON untouched |
| plant/tests/m7_flowsheet.rs:83-112, 236-251; flowsheet.rs:2232-2345, 2519-2540, 2565-2570 | carbon vs anchor (fails by design), FS_GATE_M7 (failed), f equals `recycle_fraction` | `embedded_legacy`; the gate and its four failing rows stay failed (D10) |
| flowsheet.rs `bad_specs_are_refused` | CombH refused | CombH accepted; RecFix is the refusal case |
| plant/tests/m8_level4.rs:260-280, 368-420 | B_COMB_H CLOSURE_H "not_run"; anchored node closure | status test changes with the extension; add an S10 row in `anchored_mass_run` (export.rs:1830) so node closure holds |
| plant/tests/m9_economics.rs:123-230, 310; live_fields.rs:85 | a_atm targets; NG against RES_NG_MMSCFD | read `a_net` and `c.ng` |
| wasm/src/tests.rs:378-415, 895, 1057-1115 (MAP 21, NO_CELL 11), 1231 | KPIs against results; control probe | arrays resized; the new control must move a required KPI (abatement does) |
| scripts/build_numbers.py:149, 190-200, 618-628, 750-753; build_site.py:100-103 | 578052 +/-0.5 percent; ties to the INL_9p21 row | tie to the central baseline id; expected value from the exporter |
| docs/VALIDATION_MATRIX.md:147-149, docs/FINAL_DESIGN_DECISIONS.md:19, 55, 150-153, 571; README.md:15-17 | INL 9.21 headline | Phase 7 |

After the default flip, classify every other failure by one rule. A test about the anchored port or the INL ledger uses `embedded_legacy`. A structure test keeps `embedded`. A number that legitimately moves gets an expected value taken from the exporter and named by equation id.

## (g) Frontend labels (listed only; Phase 6 edits)

The five places the Phase 1 review names: wasm/src/registry.rs:520 (DROPPED "baseline" text), registry.rs:808-812 (KPI label "Annual abatement, INL baseline"), wasm/src/solve.rs:2042 ("Unabated SMR baseline"), frontend/views.js:385 and :415, frontend/tour.js:476-479 (also :465). Keep every KPI id so the tour audit and the required-KPI block hold. Related: README.md:17; views.js:28 (status map needs `carbon_independent`); views.js:822-823 and 835-847 with solve.rs:2080-2083 ("Residual of the anchored carbon balance at INL scale"); build_numbers.py token BASELINE_INL_KG_KG. The cost verdict uses `<=` against 100 (solve.rs `below`, `met`) and D3 needs `<`; that belongs to the subsidy track and must be agreed once.

## (h) Exported quantities

| Quantity | Emissions_Ledger (view `independent_carbon`; baseline-free lines use baseline_id PLANT) | Final_Design_Basis | wasm kpi |
|---|---|---|---|
| Annual captured CO2 | CAPTURED | RES_CAPTURED_T_Y | co2_captured_t_y |
| Residual direct emissions | STACK, COND_VENT, RESIDUAL_DIRECT | RES_ATM_T_Y | co2_atm_t_y |
| Transport and storage emissions | TS_EMISSIONS | RES_TS_EMISSIONS_T_Y (new) | co2_ts_t_y (new) |
| Net abatement | ABATEMENT_NET per baseline | RES_ABATEMENT_T_Y | abatement_t_y |
| Direct abatement | ABATEMENT_DIRECT per baseline | RES_ABATEMENT_DIRECT_T_Y (new) | abatement_direct_t_y (new) |
| Abatement per kg H2 | kg_per_kg_h2 column of both abatement lines | RES_ABATEMENT_KG_KG (new) | abatement_kg_per_kg (new) |
| Percentage reduction | REDUCTION | RES_REDUCTION | reduction (new) |
| Emission intensity | INTENSITY | RES_INTENSITY_KG_KG | intensity_kg_kg (new) |
| Carbon balance residual | CLOSURE_REL, verdict text in `line` | RES_CARBON_CLOSURE_REL (new) | `balances.closure` rewritten: residual, relative, tolerance, both verdicts, unknown ids |

Status word `carbon_independent` with source "engine independent carbon basis" goes into `row_status` and `status_source` (export.rs:97, 147). New KPI ids are appended after index 24 (registry.rs:849). Also write headroom_unknown_t_y, the three floor rows, the DV2 reference-gas row and the T&S cross-check rows (T2 0.9 to 2.0 percent, T4 2.6 percent).

## (i) Ordered steps, test first

Gate for every commit: fmt, clippy -D warnings, `cargo test --workspace`, export plus `compare_results.py`, `check_clean.py`.

0. Measure one flowsheet solve; print r to 12 digits. No behaviour change.
1. Test `legacy_export_matches_snapshot` (passes at once). Then snapshot, `--subset`, `--names`, CI step, `CarbonBasis` (Legacy only) and `embedded_legacy`.
2. Tests `cryo_defaults_equal_calibration` (f and r within 1e-9 of the calibrations), `range_guards_refuse` (N.26), `params_complete`. Then toml entries, `LIMITS`, `CarbonParams`.
3. Tests for the pure forms, each on the current FS_B_REC_F run: `eq_n_1_5_gas_rate`, `eq_n_2_1_loop_closed_form` (three tear starts), `eq_n_2_2_stream_vector`, `eq_n_3_1_capture_identity`, `eq_n_4_1_recycle_read_back`, `eq_n_4_2_n2_balance`, `eq_n_4_4_stack_curve`, `eq_n_6_1_verdicts` (a relative loss of 1e-6 in one stream gives Open). Then the pure part of `carbon_account.rs`.
4. Tests `eq_n_9_1_hand_ledger`, `eq_n_9_2_decomposition` (exact gap k(sum eps_out_other - sum eps_in)), `eq_n_10_intensity_identity`, `legacy_emissions_bit_identical`. Then `Carbon` and `Emissions` extension with the `c.basis` branch.
5. Tests `indep_ledger_homogeneous`, `eq_n_6_2_tear_check`, `cache_is_deterministic`, `nonconvergence_is_an_error`. Then `with_cryo`, `TailGas`, the `CarbonLedger` fields, `legacy_view`, provider and cache.
6. Tests with the Part B anchors (captured 363,277 t/y, stack 30,778, condensate 1,731, each within 1 t/y), `eq_n_7_floors_below_published`, `balance_uses_one_basis`. Then `carbon_active`, `balance_on`, `case_summary`, the default flip and the reclassification of section (f).
7. Tests `eq_n_9_4_zero_inputs_equal_direct`, `ts_chain_within_published_range`, `lifecycle_does_not_double_count_ts`. Then the T&S parameters and `ts_burden_per_t`.
8. Test `cost_denominator_is_net` (ac_l5 x a_net / 1e6 equals the ladder). Then switch every consumer and `drift.rs`.
9. Tests `emissions_rows_cover_owner_list`, `export_byte_identical`, `legacy_layout_frozen`. Then exporter rows, S10 row, status word, closure rows.
10. Tests `independent_rows_reported` and the count update. Then harness rows: floors, DV2 row, T&S cross-checks; `FS_CARBON_VS_ANCHOR_*` unchanged and still failed.
11. Tests `cli_basis_flag`, `every_control_moves_a_kpi`, `live_matches_export`. Then CLI and wasm.
12. Tests `comb_h_carbon_closes`, `export_site_vs_atm`, `comb_n_h_scale`. Then the flowsheet extension.
13. Regenerate `results/`; list changed cells and reasons in the commit message; touch goldens only as section (f) allows.
14. Only if step 0 shows kpi_only over budget: a frozen-share closed form for sweeps, labelled approximate, with a test bounding its error against the flowsheet over the registry ranges.

## (j) Risks and size

- **Default flip churn:** about 60 call sites of `Model::embedded()` and `balance`. Mitigation: `embedded_legacy`, the classification rule, the snapshot test.
- **Heat and power follow the new carbon.** `balance` consumes the independent captured CO2, so reboiler duty and compression power rise by 0.97 percent. Reactor need and spare shift slightly; re-read the drift ranges (spare 27 to 33 MWth, NG within 5 percent of 23.2). The alternative overlays emissions and cost items only and leaves the balance legacy, which prices captured tonnes that differ from the abated tonnes. Recommended: the first.
- **Performance:** every independent solve needs a flowsheet; cache plus step 0 measurement.
- **Non-convergence at sweep extremes** (ETA_MEA 0.80 to 0.95, f and r grids): `not_converged` rows.
- **Parallel subsidy edits** to export.rs, economics.rs, sens.rs, parameters.toml, compare_results.py, wasm registry and no_internal_pointers.rs: land shared options first, rebase often.
- **Mixed bases in one table** (RecFix): status and source words carry it; a test checks no row mixes.
- **Content guard** (check_site.py:57-100) rejects tokens such as 99.4 in site data; new outputs near 100 can hit it.
- **T&S:** distance dominates (A_net runs 563.9 to 550.1 kt/y for 200 to 2000 km, py); the round-trip factor of two rests on the T2 cross-check.
- **Residual INL information:** f, r, S/C, 871 C, PSA recovery 0.88 and Selexol 0.99 stay INL-sourced; label them process assumptions in every output.

Size: about 4,000 changed lines (new module 700, anchored and emissions 350, flowsheet 250, exporter 450, sens and drift 200, wasm and CLI 300, vv 300, tests 1,200, toml 170), a 1.2 MB snapshot, 25 to 30 commits, 8 to 10 working days.
