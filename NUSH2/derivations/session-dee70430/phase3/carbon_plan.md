# Plan design:carbon, the independent carbon basis

Read-only study of the worktree at HEAD 2b231dd (branch feature/independent-abatement-subsidy). No cargo command and no engine run was made. Figures marked (py) come from inline Python on committed values; the engine decides the final numbers. The subsidy track has uncommitted edits in dcf.rs, economics.rs, subsidy.rs and tests/m10_subsidy.rs. An earlier plan for 41d8d4b is kept next to this file as carbon_plan_prev_1737.md; this plan replaces it and differs in one structural choice (section 0): the independent path is additive and the anchored functions stay untouched.

## 0. Design rule and departures from the derivation

**Rule.** `Model::balance`, `Model::carbon`, `Model::case_summary` and `Model::emissions` on an anchored `Carbon` keep their code and their results. New `plant_balance` and `plant_summary` return the same structs with `Balance.c` and `Balance.cap_t` replaced by the plant's own stream ledger when `CARBON_BASIS` is 1 and the scenario has a ledger. `Carbon.src` (Anchored or Plant) says which one a number is, and `Model::emissions` branches on it. The flowsheet keeps calling `balance`, so there is no recursion. Consumers that must see the plant's carbon switch to the plant functions: lib.rs:242 (`run`), export.rs:404, 999, 1024, 1122, sens.rs:211, 445, 670, 1296, 1502, and the wasm solve (`case_summary`, `emissions`, `lifecycle_terms`).

Effects. Reactor need (179.0 MWth), spare (31 MWth), the T01 to T13 terms, the gates, drift.rs, the golden and screening rows, `Integrated_Energy_Balance.csv` and `sizing_walk` do not move. Legacy output follows by delegation. Cost: reboiler duty, compression and pump power stay on the anchored captured CO2 (359,777 t/y) while abatement, the T&S tonnage and the capture capital use the plant's 363,277 t/y (+0.97 percent). A reconciliation row prints the reboiler duty that the plant's capture would imply. Whether the energy balance should follow instead is question Q3.

- **DV1 central reference.** Derivation 01 builds E_ref from the plant's own unabated reference (N.7.3), but decision D2 (revised after Phase 2) fixes the central reference as IEAGHG 2017-02 Case 1, 8.99600 kg/kg times annual H2 (`BASELINE_IEAGHG_KG_NM3 / BASELINE_IEAGHG_RHO`). Implement N.9.1 with a tagged reference. Implement N.7.1 and N.7.3 as floors (reference run with eta=0, f=0, r=0, no firing gas) because N.7.2 needs `REF_FURNACE_EFF` and `REF_STEAM_DUTY`, which have no source. The task text lists IEAGHG as a comparison row; D2 contradicts it (Q1).
- **DV2 gas difference.** Rule 1 of N.7 cannot hold: the GAS ledger line keeps INL's 52.5 MMSCFD (economics.rs:254) while the IEAGHG intensity implies 35.09 MMSCFD of reference gas at Case B against 35.87 for INL scaled (-2.2 percent, py). Print the gap as a validation row and leave the cost line alone.
- **DV3 T&S.** The flat `TS_BURDEN` 0.025 t/t gives way to the physical tau_TS of N.9.4 on the independent basis (D7c). It stays for legacy and as a cross-check row.
- No algebraic error was found in N.1 to N.10. The quoted factor 327.695 t/y per kmol/h and the residual ratio 4.3e-12 reproduce.

## (a) Data structures and signatures

**New engine/plant/src/abatement.rs** (add to lib.rs; mirrors subsidy.rs). One field `carbon_params: CarbonParams` in `Model` and one line `CarbonParams::read(r)?` in `build`, so anchored.rs changes stay small.
- `enum CarbonBasis { Legacy, Independent }` from `CARBON_BASIS` (0 or 1, anything else refused). `enum BaselineKind { Ieaghg, Inl, Cho }` from `CARBON_BASELINE`, carrying the ids IEAGHG_9p00, INL_9p21, CHO_9p35 that the ledger already prints.
- `struct CarbonParams { basis, baseline, cryo_f, cryo_r, cond_vent, closure_tol, ts: TsInputs }`. `read` rejects f, r, v outside [0,1] and eta outside (0,1]; it never clamps.
- `struct PlantLedger { run_id, scn, h2_kmol_h, ng_mmscfd, lines: CarbonLedger, unconverted: f64 }`, one flowsheet run at Case A hydrogen (13,154 kg/h), so its lines are already at Case A scale. A process-wide cache `OnceLock<Mutex<BTreeMap<LedgerKey, Arc<PlantLedger>>>>`; the key holds bit patterns of kind, eta, f, r, xi, y_CH4, T, P, S/C and k_source. Solvent parameters stay out of the key because capture is `eta x S11 CO2` and S19 equals S20 when the lean loading is fixed (`solvent_tear = false`, the fallback branch of `solve_at_ng`, flowsheet.rs:1699-1712; a test compares it with FS_B_REC_F to 1e-9). Only Ok results are cached; non-convergence is returned as `EngineError::NotConverged` and never falls back to legacy.
- `impl Model`: `plant_ledger(scn, eta) -> Result<Arc<PlantLedger>>`, `reference_ledger() -> Result<Arc<PlantLedger>>`, `plant_balance(scn, &BalanceOpts) -> Result<Balance>`, `plant_summary(scn, s) -> Result<BTreeMap<&'static str, f64>>`, `ts_parts() -> TsParts` (ship, conditioning, injection, terminal, purge; `total()` is tau_TS), `abatement(&Balance) -> Result<Abatement>`.
- `struct Abatement { src, captured_t_y, stack_t_y, cond_vented_t_y, residual_direct_t_y, ts_emission_t_y, ts_parts, h2_t_y, rows: [BaselineRow; 4], closure: AccountReport }`; `BaselineRow { id, e_ref_t_y, a_direct, a_net, a_h2_direct, a_h2_net, reduction, e_ref_kg_kg }` for IEAGHG, INL, Cho and PLANT_FLOOR.
- `pub(crate) mod closed_form`, each function tagged `/// eq: N.x.y`: `loop_denominator`, `tube_carbon` (N.2.1), `fates` (N.2.2, N.3.1), `gas_rate_from_h2` (N.1.5), `recycle_read_back` (N.4.1), `n2_cut` (N.4.2), `stack_vs_f` (N.4.4), `ts_ratio` (N.9.4). They check the flowsheet and do not replace it.

**carbon.rs.** `enum Status { Modelled, Specified, Unknown, OutOfBoundary }`, `struct Entry { id, tag, inlet: bool, kmol_c_h: Option<f64>, status, to_air: bool }`, `enum Verdict { Closed, Open, Undetermined }`, `struct AccountReport { residual, relative, tol, on_modelled: Verdict, overall: Verdict, unknown: Vec<&'static str> }`, `CarbonLedger::account(v) -> Vec<Entry>`, `closure_report(&[Entry], tol)` built on the existing `close_carbon` (N.6.1). `CarbonLedger::closure()` and its tests stay.

**anchored.rs.** `Carbon` gains `src: CarbonSrc`, `cond` (S10 CO2) and keeps `closure`, which the plant overlay sets to the true ledger residual. The legacy `carbon()` sets `src = Anchored`, `cond = 0`. `LIMITS` (line 480) gains the new ranges. `case_summary` is split into `case_summary_from(scn, s, &Balance)`; `case_summary` passes the anchored balance, `plant_summary` the plant one. Under `src == Plant`, `feed_C_kmol_h` reads `c.f*s` (legacy keeps `f_c6*s`).

**Overlay map (plant_balance).** For ledger `L` and `u = 1` (Case A basis): `c.f = L.feed`, `c.ng = L.ng_mmscfd`, `c.t = L.tube`, `c.co2_abs`, `c.cap = L.captured`, `c.slip`, `c.g = L.tail_gas`, `c.r = L.recycled`, `c.p = L.product`, `c.cond = L.condensate`, `c.unconv = L.tube - L.co2_abs - L.condensate`; `c.burn = L.burned + v L.condensate` (site) and `c.exported = 0`, except Export where `c.burn = 0` and `c.exported = L.burned`; `b.cap_t = kmol_h_to_t_h(c.cap) s`. `BalanceOpts.phi` is ignored on this path; `eta_c` is used.

**emissions.rs.** `Emissions` renames `a_atm` to `a_direct` so that a missed call site fails to compile, and gains `ts_emission`, `a_net`, `cond_vented`, `stack`, `baseline_kind`, `unmodelled: u8`. Legacy: bit-identical values, `a_net = a_direct`, `ts_emission = 0`. Independent: `baseline = kg_kg(kind) x h2_t_y`, `atm = tpy(burn + exported)`, `ts_emission = tau_TS x captured`, `a_net = a_direct - ts_emission` (N.9.1, N.9.4). Each former `.a_atm` is classified as cost denominator (`a_net`: lib.rs:178, dcf.rs:184, sens.rs, export.rs:1020, 1108, 3181, wasm) or display (`a_direct`). `lifecycle_terms` takes `ts_burden` from `ts_ratio` when `b.c.src == Plant`. `sens.rs` `Baseline` offsets become differences to the active baseline (line 215) and `adjust` (line 178) also scales `cond`.

**flowsheet.rs.** `FlowsheetSpec::new` keeps today's inputs. New `ledger_spec(m, kind, eta)` with `LedgerKind { Rec, InlSel, NoRecycle, Reference }`: f and r from `CarbonParams` (Rec, InlSel), f=r=0 (NoRecycle, Reference). `no_cryo()` is `carbon_recycle == 0 && h2_return == 0`; `solve_at_ng` then sets `cryo_w_per_ng = 0` (new specs only). New `NoCapture: CaptureColumns` (S12 = S11, S19 = 0) for Reference. NoRecycle uses `Scenario::RecF` (MEA columns) in the spec, so `validate`, `base_spec`, `suite`, `SUITE_IDS` and every FS_ row stay as they are. `FlowsheetSpec::new` and `suite` take f and r from the parameters on the independent basis; `close_h2_return` still runs for its validation rows. The same row is the live flowsheet of the wasm reply (a `live_flowsheet` cache hit).

**vv and CLI.** `harness::run_all_on(&ParamSet)` with `run_all()` as the embedded wrapper. `nush2 export --basis legacy|independent` (default independent) sets `CARBON_BASIS` on the ParamSet; `json` and `drift` accept the same flag.

## (b) Stream ledger by scenario

| Scenario | Level 3 ledger today | Plan |
|---|---|---|
| B_REC_F, A_REC_F, B_REC_F_MODELLED | FS_B_REC_F, FS_A_REC_F | Kind Rec. The Modelled preset differs in ETA_MEA only; eta is in the cache key. Test: per-H2 ledger of Case B equals Case A to 1e-9 (homogeneity). |
| A_INL_SEL, B_INL_SEL | FS_A_INL_SEL (NG 34 MMSCFD closure run), FS_B_INL_SEL | Kind InlSel on the hydrogen basis, Selexol 0.99 as INL's published specification. INL's printed 1927 and 142 st/d become validation rows that may fail. |
| B_COMB_H | none; `validate` refuses | Kind NoRecycle: f=0, r=0, no cryogenic power, cut equals the whole reject, so all reject carbon is burned on site. |
| B_EXPORT | none | Same ledger as COMB_H; S15 carbon goes to `c.exported` (air, off site). |
| COMB_N (sweep only) | none | No new run: by homogeneity, hold the REC_F plant's own gas per unit time; `h_scale = (ng/H2 of Rec) / (ng/H2 of NoRecycle)`. No INL gas number enters. |
| B_REC_FIX | none | Legacy by definition (stack pinned to INL's value, anchored.rs:983-984). `Carbon.src = Anchored`; status words `legacy_by_definition`. A test asserts no table row mixes bases. |
| PLANT_FLOOR | none | Kind Reference (eta=0, f=0, r=0); E_ref,floor from N.7.3 with n_fuel = 0. Floors (py): K.5 5.458, K.6 5.936, N.7.1 with psi'=omega'=0 6.294 kg/kg; all below 8.996. |

The stack is B = (1-f)G (N.4.1, N.4.4), so f and r carry INL information into every independent result (D8). Fallback if the NoRecycle ledger slips: keep COMB_H, COMB_N and EXPORT on the legacy basis with the `legacy_by_definition` label; `Carbon.src` travels with the number.

## (c) New parameters (engine/data/parameters.toml)

Classes follow params.rs. Source strings avoid absolute paths and the working-note patterns of no_internal_pointers.rs. Each id is read by the model (PARAMS_COMPLETE) and has a wasm control or a FIXED/DROPPED entry.

| id | value | unit | class | source and range |
|---|---|---|---|---|
| CARBON_BASIS | 1 | - | user_decision | D1, D12; 0 legacy |
| CARBON_BASELINE | 0 | - | user_decision | D2; 0 IEAGHG, 1 INL, 2 Cho; independent only |
| CRYO_C_RECOVERY (f) | 0.752178 (12 digits from the J.7 calculation) | 1 | assumed | D8, calibrated to INL Case 6 (J.7). Band 0.729852 to 0.771783 from PHI_RECYCLE 0.79 and 0.76 (py) |
| CRYO_H2_RETURN (r) | closure value to 12 digits (0.921630 at six) | 1 | assumed | D8, `close_h2_return` on INL's 34 and 130 MMSCFD pair. Grid +/-0.05, labelled as having no source |
| COND_CO2_VENT (v) | 1 | 1 | assumed | N.5.2; unknown fate counts as emission; range 0 to 1 |
| CARBON_CLOSURE_TOL | 1e-9 | 1 | assumed | equals ELEMENT_TOL (flowsheet.rs:193); argument in N.6.1 |
| TS_SHIP_G_T_KM | 6.77 | g fuel/t/km | published | T1 Table 4 (10 kt ship; low 3.21 at 100 kt, high 7.07 at 2.5 kt); round trip |
| TS_SHIP_FUEL_MWH_T | 11.63 | MWh/t | published | T2 (marine diesel) |
| TS_SHIP_FUEL_EF | 74.1 | t CO2/TJ | published | T8 Table 3.5.2 (high 77.4, residual oil) |
| TS_SHIP_DIST_KM | 1000 | km | assumed, not_verified | labelled assumption, no repository source; T2 evaluates 200 and 1000 km; band 200 to 2000. T10 is a snippet and supports nothing |
| TS_COND_KWH_T | 19.6 | kWh/t | assumed | T2 Table 4-2, pre-pressurised feed (S20 leaves at 2015 psia, flowsheet.rs:158); low 9.0 (T1 Table 3), high 83.1 (T2, non-pressurised) |
| TS_INJ_KWH_T | 5.0 | kWh/t | assumed | T3, pumping 4 to 5 kWh/t |
| TS_TERMINAL_FRAC | 0.00048 | t/t | assumed | T5, 720 t/y over 1.5 Mt/y |
| TS_PURGE_FRAC | 0.0009 | t/t | assumed | T4, derived upper bound for purge, cool-down and methane slip |
| TS_GRID_EF | 0.4080 | t/MWh | published | E9, 2020 row; legacy GRID_EF (0.402, not_verified) is left alone |

tau_TS = 6.77e-3 x 1000 x 3.102e-3 + 0.0246 x 0.408 + 0.00138 = 0.0324 t/t (py), against T2's 2.0 percent at 1000 km and T4's 2.6 percent for its whole chain. Expected independent results (py, approximate): captured 363.3 kt/y, residual direct 32.5 kt/y (stack 30.8 plus vented condensate 1.7), A_dir 569.5 kt/y, T&S 11.8 kt/y, A_net 557.8 kt/y, 8.51 kg/kg direct and 8.33 net, reduction 94.6 percent, intensity 0.486 kg/kg. With the ledger unchanged, L5 would be about S$197/t against 189.7 (py). Reused: BASELINE_IEAGHG_KG_NM3, BASELINE_IEAGHG_RHO, BASELINE_CHO, CO2_CONV_NOCC, ETA_MEA, PSA_H2_RECOVERY, CH4_IN_H2, AVAILABILITY. Not added: REF_FURNACE_EFF, REF_STEAM_DUTY, MEA_CARBON, CO2_PROC_VENT, GWP_* (no source or no value); the N2 and O2 fractions stay in flowsheet.rs as documented process constants, because moving them changes the signature of `natural_gas` and every mass balance row for no gain.

## (d) Unknown-stream rule

The register U1 to U9 of derivation 01 lives in code (`UNMODELLED`), because a registry value must be a number. U1 (MEA make-up and degradation, inlet and outlet), U2 (vent between S19 and S20) and U6 (start-up, flare, fugitive methane) are `Entry { kmol_c_h: None, status: Unknown }`; the storage leakage U4b likewise. U7 and U8 are Specified zero with their reason, U3 is pure CO2, U5 is OutOfBoundary, U9 is the condensate CO2 (Modelled, fate set by v), and U4a is Modelled through TS_TERMINAL_FRAC and TS_PURGE_FRAC. `closure_report` maps Unknown to `None` and calls `close_carbon`, which returns `None` for any absent flow, so `overall` is Undetermined whenever one entry is unknown. `on_modelled` repeats the sum over Modelled and Specified entries and reports Closed when |rho| <= tau_C. A missing stream is never entered as zero. Annual abatement is labelled `modelled_streams_only` (unknown air outlets can only lower it) and the exports carry the unknown ids beside the residual.

## (e) Basis switch, CLI, wasm, legacy snapshot, CI

- Engine: `CARBON_BASIS` in the ParamSet, so `Inputs.overrides`, the CLI and wasm share one path. `plant_balance` and `plant_summary` return the anchored result when it is 0. In legacy mode the exporter emits no row and no column that 8750b09 lacks (new Emissions_Ledger views, RES rows, validation rows and sensitivity rows are skipped), and `run_all_on` skips the independent rows.
- CLI (main.rs:30): `--basis`, default independent. wasm: a new enum Spec `carbon_basis` in registry.rs (`Target::CarbonBasis`; the request key `basis` already means Case A or B), plus controls `cryo_c_recovery` and `cryo_h2_return` in the Capture group with the bands above. `live_flowsheet` (solve.rs:457) skips `base_closure_cached` when independent. Cost: one live flowsheet is already inside the measured 30 to 42 ms full solve (wasm README line 21); `kpi_only` solves pay one cached ledger per distinct (scenario, eta). Step 0 measures it.
- Snapshot: take the seven tables from `git show 8750b09:results/<file>` into `engine/vv/data/legacy_8750b09/` (about 1.2 MB). Do not copy the current results/: Final_Design_Basis gained 24 input rows and Validation_Results one cell at 2b231dd. `compare_results.py` gets `--subset` (keyed compare; extra rows listed, not failed; keys: id for Final_Design_Basis and Validation_Results, run_id+term_id, run_id+stream_id, the leading identifier columns of Emissions_Ledger, Economic_Ledger and Sensitivity_Results), `--names`, and `--allow-cell Validation_Results.csv:PARAMS_COMPLETE:conditions` (the entry count text). The subsidy track needs the same options; land them once.
- CI (ci.yml, after Test): export with `--basis legacy` to `$RUNNER_TEMP/legacy` and compare with `--subset` to the snapshot; then export with the default basis and compare to results/. A Rust test `legacy_export_matches_snapshot` repeats the keyed compare. Gate for every commit: fmt, clippy -D warnings, test, export plus `compare_results.py`, `check_clean.py`.

## (f) Existing items that encode copied values or labels

Because the anchored functions are untouched, most of these keep passing as they are.

| Where | Encodes | Becomes |
|---|---|---|
| drift.rs:24, 138-145, 244-253 | 578,000 +/-12,000 t/y blocking, on the anchored port | unchanged and still meaningful (the anchored port reproduces the screening value). The independent comparison is a reported Validation row (D10). Finding count 8 (m8_level4.rs:286) holds |
| vv/src/harness.rs:165-262 (SCREENING_, GOLD_, HEADLINE_, L3_ rows), :151-159 | A=578052, 23.2 MMSCFD, 137.7 and the other L3 costs, anchored closure | unchanged, legacy by construction; new `independent_rows()` (pass or reported) keeps `plant_rows` length 59 (plant_goldens.rs:138-150) |
| vv/src/golden.rs:104-136, 209-223, 270-289, 386-489, 709-756; data/engine_goldens.json, freeze_values.json, screening_checks.csv | frozen legacy balances and sweeps | unchanged |
| vv/tests/plant_goldens.rs:50-60, 63-80, 82-92 | A=578052, cost pair, sizing walk | unchanged |
| plant/tests/m7_flowsheet.rs:83-112, 236-251; flowsheet.rs:2233-2345, 2519-2540 | carbon against the anchored ledger (four rows fail), FS_GATE_M7 Failed, f equals `recycle_fraction` | unchanged; failed rows stay failed (D10) |
| plant/tests/m8_level4.rs:159-167, 268-286 | B_COMB_H CLOSURE_H and WGS_RELEASE not_run | unchanged, because the ledger-only runs stay out of SUITE_IDS |
| plant/tests/m9_economics.rs:123-130, 172-180; m10_subsidy.rs:32 | `m.balance` + `emissions().a_atm` compared with sens lever rows | switch to `plant_balance` and `a_net` |
| plant/tests/live_fields.rs:85 | RES_NG_MMSCFD against the live reply | expected value regenerated (23.10 against 23.23 MMSCFD) |
| terms.rs:264 (T13 `b.c.ng`), terms.rs:301, drift.rs:58, 85 | anchored slots of the terms | unchanged (they call `balance`) |
| sens.rs:157-160, 178-190, 211-218; sweeps over PHI_RECYCLE, CO2_CAPTURED_C6, NG_CASE6 | Baseline offsets, adjustments, legacy-only inputs | plant path; legacy-only inputs become inert on the independent basis and are labelled `legacy_input` |
| wasm/src/tests.rs:22-30 (REQUIRED), 347-415, 474 (`INL_9p21` lookup), 773-799, 895, 1069-1080 | KPI parity with results/, control probe | append new KPIs; look-ups use the central baseline id; `every_control_moves_a_kpi` keeps passing (abatement moves) |
| scripts/build_numbers.py:149, 190-194, 617, 749-754; build_site.py:100-102; derivations_tools.py:859-861; check_site.py:80-92 | 578052 +/-0.5 percent, ties to the `INL_9p21` rows, withdrawn tokens | re-point to the central baseline id; expected values from the exporter (Phase 5) |
| docs/FINAL_DESIGN_DECISIONS.md:19, 51, 55, 153, 394, 489, 571; VALIDATION_MATRIX.md:147-149; README.md | INL 9.21 headline | Phase 7 |

Cells of results/ that change on purpose: Emissions_Ledger (all B and A rows and the new views), Final_Design_Basis result rows for the carbon, gas and T&S lines plus the new input rows, Integrated_Mass_Balance anchored carbon lines of the six fixed runs, Economic_Ledger and Sensitivity_Results (denominator, `annual_captured_t`, `ng_plant`, capture capital), Validation_Results (new rows). Integrated_Energy_Balance must stay byte-identical, which is a test.

## (g) Frontend labels (listed only; Phase 6 edits)

The five places: engine/wasm/src/registry.rs:520 (DROPPED "baseline" text); registry.rs:808-812 (KPI labels "Annual abatement, INL baseline" and its four siblings); engine/wasm/src/solve.rs:2042-2043 ("Unabated SMR baseline"); frontend/views.js:385 and :415 (note and RES label; also :386 chart title "t CO2e per year", which breaks decision D4); frontend/tour.js:476-479 (also :465 "Unabated reformer baseline"). Keep every KPI id so the tour audit and the required-KPI block hold. Related: views.js:822-823 and solve.rs:2080-2083 describe the anchored closure ("Residual of the anchored carbon balance at INL scale"); the status map at views.js:26-37 needs the new word. The `<=` against 100 (solve.rs `below`, `met`) belongs to the subsidy track (D3).

## (h) Exported quantities

| Quantity | Emissions_Ledger (view `independent_carbon`; lines without a baseline use baseline_id PLANT) | case_summary key | wasm kpi |
|---|---|---|---|
| Annual captured CO2 (N.9.3) | CAPTURED | captured_t_y | co2_captured_t_y |
| Residual direct emissions (N.5.2) | STACK, COND_VENT, RESIDUAL_DIRECT | atm_t_y | co2_atm_t_y |
| Transport and storage emissions (N.9.4) | TS_EMISSIONS, plus one line per TsParts term | ts_emission_t_y | co2_ts_t_y (new) |
| Net abatement (N.9.4) | ABATEMENT_NET per baseline id | A_net_t_y | abatement_t_y |
| Direct abatement (N.9.1) | ABATEMENT_DIRECT per baseline id | A_t_y | abatement_direct_t_y (new) |
| Abatement per kg H2 (N.10.1) | kg_per_kg_h2 column of both abatement lines | A_net_per_kg | abatement_kg_per_kg (new) |
| Percentage reduction (N.10.2) | REDUCTION | reduction | reduction (new) |
| Emission intensity (N.10.3) | INTENSITY | ci_kg_kg | intensity_kg_kg (new) |
| Carbon balance residual (N.6.1, N.10.4) | CLOSURE_ABS, CLOSURE_REL, CLOSURE_TOL, verdict text in `line`, unknown ids | closure_rel | carbon_residual_rel (new) |

Existing line ids BASELINE, CAPTURED, N2_RICH_CUT and ABATEMENT stay in the `direct_atm` view for continuity. New rows carry the status word `carbon_independent` (row_status and status_source, export.rs:97, 147). New KPIs are appended after index 24 (registry.rs:849). Also written: the three floor rows, the DV2 gas row, the T&S cross-check rows (0.025 legacy ratio, T2 0.9 to 2.0 percent, T4 2.6 percent), `FS_CARBON_VS_ANCHOR_*` unchanged, the f and r band rows (a `cryo_design_rows` beside `cryo_split_rows`, export.rs:3385) and the reboiler reconciliation row.

## (i) Ordered implementation steps (test first)

0. Measure one `flowsheet::solve` and a `kpi_only` solve; print r to 12 digits. No behaviour change.
1. Test `legacy_export_matches_snapshot` (passes at once). Then the snapshot directory, `--subset`, `--names`, `--allow-cell`, the CI step, `CarbonBasis`, `run_all_on` and the CLI flag.
2. Tests `cryo_defaults_equal_calibration` (f and r within 1e-9 of J.7 and the closure), `range_guards_refuse` (derivation test 26), `params_complete`. Then toml entries, `LIMITS`, `CarbonParams`.
3. Tests `eq_n_1_5_gas_rate`, `eq_n_2_1_loop_closed_form` (three tear starts), `eq_n_2_2_stream_vector`, `eq_n_3_1_capture_identity`, `eq_n_4_1_recycle_read_back`, `eq_n_4_2_n2_balance`, `eq_n_4_4_stack_curve`, `eq_n_6_1_verdicts` (a relative loss of 1e-6 in one stream gives Open; any Unknown gives Undetermined), `eq_n_6_2_tear_check`. Then `closed_form` and the carbon.rs account.
4. Tests `eq_n_9_1_hand_ledger`, `eq_n_9_2_decomposition`, `eq_n_10_identities`, `legacy_emissions_bit_identical`, `energy_table_unchanged`. Then the `Carbon` and `Emissions` extension.
5. Tests `ledger_homogeneous_case_a_b`, `ledger_equals_suite_run` (solvent_tear false against FS_B_REC_F), `cache_is_deterministic`, `nonconvergence_is_an_error`. Then `ledger_spec`, `NoCapture`, `no_cryo`, the cache and `plant_ledger`.
6. Tests with the Part B anchors (captured 363,277 t/y, stack 30,778, condensate 1,731, each within 1 t/y), `comb_h_carbon_closes`, `export_site_vs_atm`, `comb_n_h_scale`, `eq_n_7_floors_below_published`, `no_row_mixes_bases`. Then `plant_balance`, `plant_summary` and the overlay.
7. Tests `eq_n_9_4_zero_inputs_equal_direct`, `ts_within_published_range`, `lifecycle_does_not_double_count_ts`. Then the T&S parameters, `ts_parts`, `ts_ratio`.
8. Test `cost_denominator_is_net` (ac_l5 x a_net / 1e6 equals the ladder). Then switch the consumers listed in section 0 and the m9 tests.
9. Tests `emissions_rows_cover_owner_list`, `export_byte_identical`, `legacy_layout_frozen`. Then exporter rows and statuses.
10. Tests `independent_rows_reported`. Then the validation rows: independent against 578,052, INL pair 52.5 and 44.5 MMSCFD (N.7.3, within 0.5 percent), floors, DV2, reboiler reconciliation, f and r read-back.
11. Tests `cli_basis_flag`, `every_control_moves_a_kpi`, `live_matches_export`. Then wasm registry, KPIs and CLI.
12. Regenerate results/, list the changed cells and reasons in the commit message; check that `Integrated_Energy_Balance.csv` is identical.
13. Only if step 0 shows `kpi_only` over budget: a frozen-share closed form for sweeps, labelled approximate, with a test bounding its error.

## (j) Risks and size

- **Silent wrong basis.** A consumer left on `balance` reports legacy carbon. The `a_atm` rename makes missed denominators fail to compile; the unmodified list in section 0 is checked by `no_row_mixes_bases`.
- **Energy and capture size differ by 0.97 percent.** Accepted by design (section 0); Q3.
- **Performance.** A miss costs one flowsheet; the cache and step 0 bound it.
- **Non-convergence at sweep extremes** (ETA_MEA 0.80 to 0.95, f and r grids): rows print `not_converged`.
- **Parallel subsidy edits** to export.rs, economics.rs, dcf.rs, sens.rs, parameters.toml, compare_results.py, registry.rs and no_internal_pointers.rs. Land the shared script options first and rebase often.
- **Derivation tooling.** `/// eq: N.x.y` tags may be ignored or flagged by `derivations_tools.py links` (accepts sections A to L); run `check` after the first tagged function.
- **Content guard** (check_site.py:57-100) rejects tokens such as 99.4 in site data; new numbers near 100 can hit it.
- **T&S** is dominated by distance: A_net runs 563.9 to 550.1 kt/y for 200 to 2000 km (py).
- **INL information that stays:** f, r, S/C, 871 C, PSA recovery 0.88 and Selexol 0.99. Every output labels them process assumptions.
- **Mixed ledgers:** RecFix stays anchored by definition.

Size: about 3,500 changed lines (abatement.rs 650, anchored and emissions 400, flowsheet 250, exporter 450, sens 100, wasm and CLI 300, vv 250, tests 1,100, toml 170, scripts 150), a 1.2 MB snapshot, 20 to 25 commits, 7 to 9 working days.

## Questions for the owner

Q1 central reference (D2 against the task text); Q2 values and bands of f and r; Q3 whether the energy balance follows the plant's captured CO2; Q4 shipping distance, ship size and the electricity factor (grid E9 as in the table, or the nuclear factor for site conditioning); Q5 condensate vent v = 1; Q6 accepting an Undetermined closure verdict in the headline while U1, U2 and U6 are unknown; Q7 REC_FIX shown as a labelled legacy row or dropped from the independent exports; Q8 INL_SEL on the plant ledger or kept as the INL reproduction; Q9 whether GAS, LOST_STEAM and BASE_POWER should follow the plant's reference (rule 1) instead of INL's TEV-953 values; Q10 GWP horizon and CO2e wording for the lifecycle view.

## Owner decisions of 2026-11-06 (these override the open questions Q1 to Q9 above where they overlap)

D13 Shared reference. Cost and carbon must use one conventional-SMR reference. On the independent basis (CARBON_BASIS = 1) both come from IEAGHG 2017-02 Table 2, Base Case (100,000 Nm3/h H2, 8.994 t/h): natural gas 14.212 MJ/Nm3 H2 LHV (feedstock 12.197 plus fuel 2.014, total input 394.77 MW) and CO2 0.8091 kg/Nm3 H2. The GAS line of the ladder and of the DCF takes its unabated gas energy from that table, converted to the plant's own gas through energy (LHV), never through volume. The legacy basis keeps INL 52.5 MMSCFD and 9.21 kg/kg for both. Not shared: LOST_STEAM stays INL-based, because the IEAGHG base case exports 9.918 MWe of power (11.500 gross less 1.216 and 0.366 auxiliaries) and no steam. Report that as a labelled validation row and say in the docs that the export credit is the one item still on INL's basis. The engine reports the gas energy difference between the two references (INL scaled to this plant against IEAGHG) as a row, with no value written in advance.

D14 Cryogenic inputs. f = 0.752178 and r = 0.921630 stay as labelled design assumptions (assumed, calibrated to INL). No published source gives these two numbers for the NUSH2 unit, which splits the PSA reject into a CH4/CO-rich recycle and an N2-rich cut. INL's unit is a fixed assumption (TEV-961 A-16 and A-18; INL publishes no r; the INL URLs returned 404). Do not cite anything as supporting the values. Context-only references, each marked as not supporting f or r: IEAGHG 2017-02 Case 2B (cold box of 3 flash columns plus membrane on PSA tail gas at about 51 mol percent CO2, product CO2 above 99 mol percent), Air Liquide Cryocap H2 (headline claim 60 to 99 percent capture, search snippet and marketing material, page itself not readable by the fetcher), and the Air Liquide CMTC 2019 slides (OSTI 1605737, no recovery numbers in the text layer). Band stays 0.729852 to 0.771783 for f and plus or minus 0.05 for r.

D15 Shipping distance. Central 1000 km stays. It is supported by two named candidate sites, both approximate and unverified in coordinates: Kasawari CCS, Sarawak (Block SK316, about 200 km off Bintulu; great circle Singapore to Bintulu 1059 km, 1218 km with a 1.15 sea-route factor, plus the 200 km offshore leg) and Sunda Asri, offshore West Java (Jakarta as proxy: 892 km great circle, 1026 km with the factor). The Indonesia LOI of 15 February 2024 and Perpres 14/2024 (30 percent of storage capacity open to imported CO2) are the policy basis; no Singapore to Malaysia agreement was found. Add a sweep TS_SHIP_DIST in {200, 600, 1000, 1400, 2000} km and two labelled scenario rows (Malaysia 1400 km, Indonesia 1000 km). Net abatement across that sweep follows the formula in (c); report it from the engine.

D16 Unknown streams. U1 (MEA make-up and degradation carbon), U2 (CO2 vent between S19 and S20), U4b (storage leakage after S20) and U6 (start-up, shutdown, flaring, fugitive methane) are the Unknown entries. U5 is out of boundary, not unknown. Closure verdict stays Undetermined; abatement is printed as an upper bound on modelled streams with the four ids beside it, and any target pass is printed as conditional. No unknown stream is ever entered as zero.

D17 Legacy snapshot of about 1.2 MB is accepted in the repository.
