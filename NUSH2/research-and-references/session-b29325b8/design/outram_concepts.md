# OUTRAM PARK concept review for the NUS-H2 engine

Written 2026-11-02. Read-only review. Nothing in either OUTRAM clone was built or run, and nothing in the NUS-H2 repository was changed.

## How to use this memo

The user asked that the NUS-H2 engine be held to the same concepts and checks as the OUTRAM PARK Rust code, at the level of ideas and tests. OUTRAM PARK is AGPL-3.0-only. Copyright, and with it the AGPL, attaches to the expression of the code. Ideas and equations taken from primary sources are free to use with credit. So this memo holds no OUTRAM code or transcribed pseudocode, and it names OUTRAM files only to cite where an idea was read. The agent that applies it works from this memo and the primary sources only, and never opens the OUTRAM files. Each adopted concept is credited in docs as "concept after OUTRAM PARK (AGPL-3.0), reference only".

SP below means the session scratchpad `C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad`. No SP path belongs in a committed file.

On the user's question about pseudocode: describing a concept and citing it is ordinary scholarship. A step-by-step transcription of a specific function's control flow can carry the code's expression, so this memo replaces that with acceptance tests and primary-source equations, which give the same benefit.

Clone read: `outram-park-backend`, branch `main`, commit `9af23fdb1` (2026-10-21). The `develop` branch is far ahead and was not reviewed. Citations below are `path:line` relative to that clone root. NUS-H2 paths are relative to the repository root, branch `develop`, as read on 2026-11-02 (head `f796752`).

Each item gives: the NUS-H2 module, the concept in my words, the primary source where OUTRAM names one, the OUTRAM citation, what I saw in NUS-H2 by reading (no engine test was run for this memo), and an acceptance criterion with a tolerance or pass rule. Where an item touches a Stage 2 decision (S1 to S14), the decision stands and the item says so.

Evidence tags: [R] read in a file, [D] a claim in OUTRAM's own documents, not confirmed by me, [C] my own calculation (script path given).

## 0. Maturity and human sign-off of the OUTRAM crates drawn on

OUTRAM keeps three separate records, and they must not be merged into one claim of quality:

1. A numeric maturity in each `Cargo.toml` under `[package.metadata.kovan]`: 0 concept, 1 AI draft, 2 AI V&V, 3 human reviewed, 4 human V&V. It is the crate's lowest part, with higher-rated modules listed separately (`crates/dover/Cargo.toml:13-28`).
2. A maintainer declaration of "mature" for the API usability gate, which is about dogfooding the interface and is stated in a crate's own `CLAUDE.md` with its bar and evidence class (`docs/claude-md/api-design-and-maturity.md:40-90`).
3. A "Bookkeeping status" block in each crate README with two axes, V&V and human interface, that only the maintainer may flip (`docs/claude-md/bookkeeping-and-api-docs.md:40-60`; `docs/claude-md/human-review.md:39`).

| Crate (version in tree) | Cargo maturity | Maintainer API declaration | V&V axis (README) | Interface axis (README) | What this means for citing it |
|---|---|---|---|---|---|
| `dover` 0.0.1 | 1, AI draft (`crates/dover/Cargo.toml:28`) | Not declared (`crates/dover/CLAUDE.md:162-166`) | Not yet manually checked (`crates/dover/README.md:147`) | Not yet manually checked (`:148`) | Every test is verification by its own header (`crates/dover/tests/smr_cstr.rs:1-8`); kinetics are placeholders (`crates/dover/CLAUDE.md:133-141`). Cite only its documented pitfalls. |
| `outram-park-fork-dwsim-libs` 0.0.4 | 2, AI V&V (`Cargo.toml:30`) | Declared mature 2026-09-13; bar is agreement with upstream DWSIM to 4 significant figures, evidence class cross-code, with the flash layer recorded as open on that date (`crates/outram-park-fork-dwsim-libs/CLAUDE.md:38-48`) | Not yet manually checked (`README.md:74`) | Not yet manually checked (`README.md:75`) | Its coverage matrix says no row reaches validated (`docs/upstream-port-coverage.md:45`). Its code-to-code records are [D]. |
| `outram-park-fork-coolprop` 0.1.3 | 2 (`Cargo.toml:40`) | No crate `CLAUDE.md` found, so no declaration found | Not yet manually checked (`README.md:41`) | Not yet manually checked (`README.md:42`) | Helium data file is MIT-headed CoolProp output (`src/fluids/helium.rs:1-3`); transport and EOS engine files are AGPL. |
| `tampines-steam-tables` 0.3.0 | 2 for the crate; IF97 region, backward-equation and transport modules at 3, "reviewed by the maintainer (maintainer, 2026-10-06)" (`Cargo.toml:47-111`) | Declared mature 2026-09-05, with a bar that differs by code path (`CLAUDE.md:7-37`) | Not yet manually checked (`README.md:16`) | Not yet manually checked (`README.md:17`) | Human review of IF97 modules is recorded; human V&V sign-off is not. |
| `tuas_boussinesq_solver` 0.1.8 | 4, human V&V (`Cargo.toml:33`) | Declared mature 2026-09-05 (`CLAUDE.md:7`) | Cleared by the maintainer 2026-07-21, CIET validation, peer reviewed, doi:10.1016/j.jandt.2025.03.006 (`README.md:18`) | Not yet manually checked; status INCOMPLETE (`README.md:19-21`) | The only crate with a human V&V sign-off. It has no helium. |
| Workspace as a whole | n/a | n/a | Banner: all code unverified and untrusted until a specific V&V case shows otherwise (`README.md:3-4`) | n/a | |

Concepts below that come from `dover` or the DWSIM fork are pitfalls OUTRAM documents about its own draft code. They are cited as documented findings only.

## A. Audit questions from the human-corrections catalogue

`docs/human-corrections-to-ai-work.md` logs defects where the AI output was plausible, passed its own tests, and was wrong in a way a practitioner saw. Its eleven recurring failure modes are at `:376-430`. Items 1 to 11 turn each class into a question for our engine.

### 1. Boundary conditions sit at the end where each stream enters, and each is justified

- Module: `engine/units/src/smr_tube.rs` (helium enters at z = L, process gas at z = 0, bayonet return), `engine/units/src/absorber.rs` (card H.25), `engine/units/src/stripper.rs`, `engine/units/src/regen.rs` (rich-lean exchanger, I.10).
- Concept: an advected stream carries its own inlet state into the domain; the domain does not choose it. A formulation that lets the interior value set the inflow is self-referential: a setter test passes, and the global balance cannot even be posed. A counterflow exchanger needs all four terminal states, two prescribed and two computed. The check is to state each governing equation with its boundary conditions and justify each physically.
- Primary source: card A.1 (Bird, Stewart and Lightfoot 2002) for the balance form. OUTRAM names no paper for this entry.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:32-106` (entry), `:381-384` (failure mode 1).
- NUS-H2 state: module headers state stream directions; I did not find a test that checks each prescribed inlet at its own end after convergence.
- Acceptance: for every two-point problem (tube, absorber, stripper, exchanger), a test named `eq_x_n_inlets_met_at_their_own_end` checks that each prescribed inlet value holds at its own boundary in the converged solution to 1e-9 relative, and that no outlet value equals an input field. The module `//!` header lists every boundary condition with a one-line physical reason. Pass: both parts, every module.

### 2. A conserved-quantity test on every unit, separate from any setter or API test

- Module: `engine/units/src/gas.rs` (`closure`), `engine/plant/src/flowsheet.rs` (cards A.2, A.5, A.8), `engine/units/src/smr_tube.rs`, `engine/units/src/wgs.rs`.
- Concept: a global balance catches defect classes nobody has thought of; a test of one API call catches only the defect already known. For the helium-heated tube this means the helium side and the process side together.
- Primary source: card A.8, steady-state first law, $\sum \dot n_{out} h_{out} - \sum \dot n_{in} h_{in} = \dot Q_{in}$.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:121-133`; failure mode 2 at `:385-387`.
- NUS-H2 state: `closure()` exists and units report it; tests `straight_tube_closes` and `bayonet_tube_closes_and_meets_at_the_tip` exist in `smr_tube.rs`. I did not check whether the tube closure includes the helium duty, or whether sweep and sensitivity rows report closure.
- Acceptance: for every unit, and for the Case B flowsheet at each S8 reformer exit (871, 825, 800 C): energy residual divided by the largest term at most 1e-6; element residuals for C, H, O, N at most 1e-9 relative; for SMR-TUBE, helium duty $\dot m_{He} c_p (T_{He,in}-T_{He,out})$ equals process enthalpy rise plus stated wall and loss terms to 1e-6 relative. Every sensitivity row carries its closure value. Pass: all residuals inside the limits.

### 3. Test the regime where the model is used: SRK correction at absorber pressure

- Module: `engine/mea/src/pstar.rs` (`srk_phi_co2`, card G.21), `engine/units/src/solvent.rs` (`pstar_gas_pa`), called from `engine/units/src/absorber.rs:517` with the absorber pressure `P_ABS_PA = 26.8e5` (`engine/units/src/columns.rs:32`).
- Concept: a correction that is right in its validation regime gets reused where its assumptions fail. The SRK fugacity coefficient is computed for a CO2 and H2O binary with $k_{ij}=0$. That binary matches the Aronu vapour (CO2 over the solvent). In the plant absorber the gas is mostly H2 at 26.8 bar, while the code treats every non-CO2 mole as water vapour.
- Primary source: Soave (1972) SRK equation, not named by OUTRAM. OUTRAM's related finding: a cubic EOS with missing binary data gave $K_{CO_2}$ off by +531% on a sour gas at 5 MPa (`crates/outram-park-fork-dwsim-libs/verification_and_validation/sour_water_vs_upstream_dwsim.md:208,218`), and failure mode 3 "untested regimes" (`docs/human-corrections-to-ai-work.md:388-390`).
- NUS-H2 state [C]: I re-evaluated the same SRK form with the same critical constants, mixing rule and largest-root choice in my own Python (`SP/findings/scripts/srk_phi_absorber_check.py`, not engine code, engine not run). At 313.15 K and 26.8 bar the CO2 and H2O binary has one real root, Z about 0.025 (liquid-like), and $\phi_{CO_2}$ = 563 at $y_{CO_2}$ = 1e-4, 485 at 0.01, 81 at 0.15. With H2 as the partner the same form gives Z about 1.01 and $\phi_{CO_2}$ = 0.96 to 0.98. At 1 bar both partners give about 0.998. The existing test `phi_tends_to_one_at_low_pressure` runs at 1 kPa. If the engine reproduces my numbers, p* in the plant absorber is divided by roughly 80 to 560.
- Acceptance: a test `eq_g_21_phi_at_absorber_pressure` evaluates the coefficient the absorber actually uses at 313.15 K, 26.8e5 Pa and the Case B gas, and requires a vapour root (Z > 0.3) and $0.90 \le \phi_{CO_2} \le 1.05$. A validation row reports $\phi$ at the absorber top and bottom. Write the predicted sign and size before any fix (p* rises at the rich end, so height or solvent rate rises at fixed capture) and record the movement of ABS-CASEB rows and drift-table entries afterwards. S12 is unchanged: SRD and T07, T08 stay anchored. Pass: test green and movement recorded.

### 4. Check what the engine already has before adding a second implementation

- Module: whole engine. Examples: the gas constant (item 28), and SRK, where card G.21 names `props::mixture::srk_phi` while the only SRK code is in `mea::pstar`. Root finding is also written twice (`engine/units/src/solve.rs` says `nush2_numerics` has no Brent, and `engine/mea/src/solve.rs` holds one).
- Concept: two implementations of one thing drift apart silently. Search by the domain noun before writing.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:391-393` (failure mode 4); `CLAUDE.md:353-400`.
- Acceptance: one definition per physical constant and per numerical routine across `engine/`. A vv test greps for a second definition of any name in a short list kept in the test, and fails when the second one lacks a doc line naming why it must differ. Pass: zero undocumented duplicates.

### 5. Precise-looking numbers invite trust they have not earned

- Module: `engine/plant/src/export.rs`, `engine/wasm/src/registry.rs`, and the numbers path to `build/numbers.json`.
- Concept: polished output and many digits reduce scrutiny. Digits printed should follow the least precise input.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:151-196` (an overlay that contradicted sources already in the repository), failure mode 5 at `:394-395`.
- Acceptance: results CSVs keep full precision for traceability; every number rendered for a reader passes through one formatter with a significant-figure rule declared per quantity (for example 3 significant figures for S$/t and Mt/y). A test renders every report-facing number and fails on any value with more digits than its rule. Pass: zero violations.

### 6. Name the governing model first; cost blow-up is evidence about the formulation

- Module: every file in `engine/units/src/` (each header already carries a Class line: Rigorous, ReducedOrder or Empirical).
- Concept: fidelity is a choice of formulation to justify. When a sizing or run cost jumps by an order of magnitude, ask whether the formulation or an assumption is wrong before working around it. Adding a bracket or a correction each time a model misses is the sign to stop and question the premise.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:200-267` (pebble bed as porous medium), failure modes 6, 7 and 8 at `:396-416`.
- NUS-H2 state: S8 records that the HA tube cannot reach 871 C with 900 C helium even at 5000 tubes. S8 is decided; this item asks only that the row note gives the physical reason (the hot-end approach the counterflow tube would need) so a reader sees the result is a property of the assumption set.
- Acceptance: every `engine/units/src/*.rs` header names its governing equations and a one-sentence reason for the class. Each row whose result exceeds the reference design by more than a factor of 10 in equipment count or area carries a note naming the limiting assumption. Pass: grep finds a Class line and reason in every units module; the S8 rows carry the note.

### 7. Correct physics is the default; ablation is an explicit act and defaults are pinned by tests

- Module: `engine/units/src/smr_tube.rs` (radiation term, card D.32; `KSource` default), `engine/units/src/solvent.rs` (Poynting and SRK applied in the absorber), `engine/units/src/jaeri.rs` (bayonet arrangement), `engine/plant/src/flowsheet.rs` (S11 recycle variant and S14 condensate venting are sensitivities, defaults unchanged).
- Concept: a term behind an off-by-default flag is a term the model does not have. A test that constructs through the ordinary path should assert the term is present. Seeing a symbol in the code does not show it is active.
- OUTRAM citation: `CLAUDE.md:150-195`; convenience constructors at `docs/human-corrections-to-ai-work.md:417-420`.
- Acceptance: one test per default, built through the public constructor: radiation contributes a nonzero share to the bed-side coefficient at 1073 K; `KSource::Zhang` is the default and NIST appears only in the S2 sensitivity row; JAERI benchmarks run with the bayonet set; the S11 and S14 variants appear only as named sensitivity rows. Pass: all pinned; any default change re-runs and records every V&V row that depended on it.

### 8. Reproduce the reference's geometry before comparing any outlet value

- Module: `engine/units/src/jaeri.rs`, `engine/units/src/tube_ht.rs`, `engine/units/src/tube_case_b.rs` (HA Table 6 tube), future `units::ihx` (cards C.13, C.14).
- Concept: a benchmark carries its geometry. A geometric assertion (volume or area of each region) fails before any physics is computed when the model's geometry differs from the reference.
- Primary source: JAERI-Tech 2005-031 Table 3.1 and sheet A-2-b (as `jaeri.rs` already cites); Hoseinzade and Adams 2017 Table 6.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:271-331` (test at `:324-331`), failure mode 10 at `:421-424`.
- Acceptance: a geometry check per benchmark tube reproduces catalyst volume, heated length, bed annulus area, finned outer area and helium gap to the last printed digit of the source. A benchmark row is written only when its geometry check passes; otherwise the row is `not_run` with the reason. Pass: all geometry checks green.

### 9. A faithful port reproduces the simplifications of what it ports

- Module: `engine/units/src/flash.rs` (H2, CH4, CO held insoluble as one group), `engine/units/src/wgs.rs` (Chinchen steam ratio 1.0 against about 0.5, no correction), `engine/mea/src/aronu.rs` and `engine/mea/src/pstar.rs` (prototype choices such as the assumed $v_\infty$ and the SRK binary).
- Concept: agreement with the code or model being ported certifies the translation, and every test written against it confirms its lumped parameters. Draw or tabulate the whole population grouped by the dimension the physics depends on, and ask of each shared parameter whether the physics shares it. Draw the zeros as labelled off paths.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:335-372`, failure mode 11 at `:425-430`; `docs/claude-md/visual-review-first.md:1-66`.
- Acceptance: a test prints a review table (not a shipped CSV) listing, for every separation and every species, the parameter that sets its fate and the source of that parameter. Any parameter shared by a group without a source or a stated reason fails the test. Pass: zero unexplained shared parameters.

### 10. Label every check as verification or validation

- Module: `engine/vv/src/rowkit.rs` and `results/Validation_Results.csv` (`data_use` column).
- Concept: verification asks whether the code implements its model; validation asks whether the model represents measured reality for its purpose. A comparison of code with the same paper's own model column, or with its own correlation table, is verification.
- Primary source: OUTRAM's statement follows the usual V&V definitions; it names no standard.
- OUTRAM citation: `VERIFICATION_AND_VALIDATION.md:18-50`; `crates/dover/tests/smr_cstr.rs:1-8`.
- NUS-H2 state: `helium_petersen_points` (Petersen against Petersen's table) and the Aronu 104-point comparison against Aronu's model column (`engine/mea/src/validation.rs:31`) are verification by this definition; the `tube_ht` reruns of JAERI's printed worked sheet are the same kind.
- Acceptance: a vv test reads the CSV and fails if any row whose literature value is a model output or a worked calculation printed in the same source carries `data_use = independent`. A correlation table counts as a model output. Pass: zero mislabels.

### 11. Claims shown to readers must agree with the evidence already in the repository

- Module: `engine/wasm/src/registry.rs`, `engine/plant/src/export.rs`.
- Concept: the HTGR overlay entry shows output contradicting cited sources that sat in the same repository. The check that would have caught it asserts that no claim is shown where the evidence contradicts it.
- OUTRAM citation: `docs/human-corrections-to-ai-work.md:151-196` (test idea at `:191-196`).
- Acceptance: a test enumerates every site and export label that states a validation state or a target ("meets", "passes", "validated", an 871 C reformer exit, a S$/t target) and requires each to map to a Validation_Results row with `status = pass` or to carry its disclosure note (for example S8 or S12). Pass: zero unbacked claims.

## B. Verification and validation practice

### 12. Every V&V record states method and result, with date and commit

- Module: `engine/vv/src/report.rs`, `docs/VALIDATION_MATRIX.md`.
- Concept: a check documented only by what it does cannot be judged without re-running it. The record holds inputs, reference with page, tolerance and pass rule, the measured number, the date, the commit, and an interpretation. Machine-written records and human-reviewed records are kept apart.
- OUTRAM citation: `CLAUDE.md:308-328`; `crates/tampines-steam-tables/verification_and_validation/README.md:7-92`.
- NUS-H2 state: the CSV already carries source, locator, conditions, tolerance, status and commit.
- Acceptance: every row with `status = fail` has an explanation that states the measured value against the frozen tolerance and what the miss implies; every row has a non-empty commit. Pass: a vv test finds zero empty explanations on fail rows and zero empty commits.

### 13. Split each benchmark miss into a solver part and a model part

- Module: `engine/vv/src/units_bench.rs`, `engine/units/src/jaeri.rs`, `engine/units/src/case5.rs`, `engine/vv/src/mea_bench.rs`.
- Concept: two comparisons answer different questions. First hand the engine the reference's own intermediate state (for example a measured U or a printed outlet temperature) so only the solver is compared; then run as a user would, and the remaining gap is the measured cost of the engine's simplifications. The second gap is pinned within a band and is never gated to zero.
- OUTRAM citation: `crates/outram-park-fork-dwsim-libs/verification_and_validation/reactors_vs_upstream_dwsim.md:74-98`.
- NUS-H2 state: `jaeri_measured_u_sensitivity_is_reported` (`engine/units/tests/m5_acceptance.rs:264`) is one half of this pattern for JAERI.
- Acceptance: for each Level 2 benchmark that fails (JAERI A-2 to A-5 under S3, SA-1 and the other S12 gate rows), a paired row exists with the reference intermediate imposed. The row note states which share of the miss each part carries. No tolerance changes. Pass: pairs exist for every failing Level 2 row.

### 14. Measure the cost of each documented simplification as a sensitivity row

- Module: `engine/props/src/equilibrium.rs` (ideal gas at the reformer outlet), `engine/units/src/wgs.rs` (ideal gas, Chinchen steam ratio), `engine/units/src/smr_tube.rs` (pseudo-homogeneous bed), `engine/plant/src/sens.rs`.
- Concept: each simplification gets a one-at-a-time ablation with its effect recorded, whatever its size. For orientation only, OUTRAM's code-to-code records [D] give: constant volumetric flow overstated CH4 conversion by 9.6% in a CSTR (upstream flow grew 19.0%) and CO by 6.0% in a PFR; ideal fugacity coefficients moved the SMR extent at 900 K by +0.32% at 10 bar and +1.1% at 30 bar.
- OUTRAM citation: `reactors_vs_upstream_dwsim.md:113-150`; `crates/outram-park-fork-dwsim-libs/src/reactors/equilibrium_reactor.rs:38-50`; `crates/outram-park-fork-dwsim-libs/docs/upstream-port-coverage.md:181` (row R7).
- Acceptance: one sensitivity row per listed simplification at Case B, each reporting the change in CH4 conversion or CO slip. The fugacity row runs only if critical constants for all six gas species come from a cited page; if not, it is recorded `not_run` with that reason. The default model stays as it is. Pass: rows produced and finite, or `not_run` with a reason.

### 15. A reference must pass its own balance before it is used

- Module: `engine/vv/src/fixture.rs`, `engine/units/src/jaeri.rs`, `engine/units/src/case5.rs`.
- Concept: published references and reference software contain defects; OUTRAM found upstream defects that changed answers by more than 1% without warning (carbon lost in a conversion penalty, a PFR integrating 99% of each segment). Experimental data is checked against its own heat or mass balance within the stated measurement band before use, as TUAS does with a 5% heat balance on the Du et al. data.
- OUTRAM citation: `reactors_vs_upstream_dwsim.md:140-150`; `crates/tuas_boussinesq_solver/src/lib/pre_built_components/shell_and_tube_heat_exchanger/tests/hitec_to_yd325_du_hx/set_a.rs:540-541`.
- NUS-H2 state: `docs/VALIDATION_MATRIX.md` section 20 records source anomalies, and Isa Table 9 is excluded for failing its own balance.
- Acceptance: each benchmark source used has a self-consistency row computed from the same pages (JAERI sheet helium duty against process duty; TEV-961 stream sheet mass closure). The benchmark row is produced only when its source row passes the source's stated band, and `not_run` otherwise. Pass: every used source has its self-consistency row.

### 16. Runs are reproducible, and every golden move is logged

- Module: `engine/wasm/src/solve.rs`, `engine/cli/src/main.rs`, `engine/vv/tests/plant_goldens.rs`.
- Concept: bit identity is a yes or no check that needs no human judgement. Where it is out of reach, a difference below 1e-8 relative is acceptable only with its cause understood (a different `libm`, a changed operation order). A golden value may move only with a named cause, recorded with old and new values. Nondeterministic iteration order (a hash map) is a defect.
- OUTRAM citation: `CLAUDE.md:329-351`; `crates/tampines-steam-tables/verification_and_validation/determinism_golden_moves.md:1-5`; the hash-map case at `CLAUDE.md:126-131`.
- NUS-H2 state: `eq_h_26_repeat_run_is_identical` covers the absorber; `BTreeMap` is used in `mea::pstar`.
- Acceptance: (a) the full export run twice in one process and in two processes gives byte-identical CSVs apart from the commit column; (b) the KPIs from the wasm build, run in the browser check of Stage 5, match the results CSVs to 1e-8 relative or better, and any nonzero difference is listed with its cause; (c) per the task rules goldens are never edited to pass, and any deliberate change of a golden by the lead goes into a log with cause, old value and new value. Pass: (a) and (b) green, (c) log exists.

### 17. Run uncalibrated first, then calibrate, and ablate what was calibrated

- Module: `engine/units/src/smr_tube.rs` (lumped activity, card D.21), `engine/units/src/jaeri.rs`.
- Concept: never reason backwards from a target. Derive every coefficient from outside the comparison and record the uncalibrated miss; only then calibrate, state what was adjusted and over what range, and turn each calibrated parameter back to its derived value to show how much of the agreement it carries. A fitted value that lands on its search bound is evidence about the formulation. Write the predicted sign and magnitude before a run.
- OUTRAM citation: `CLAUDE.md:82-134` (process first) and `:204-243` (uncalibrated, then calibrated, then ablation); `docs/claude-md/human-review.md:20` (predictions before runs).
- NUS-H2 state: S3 accepts the A-2 activity at the 2.246 search limit as the single disclosed tuned value; `jaeri_a2_fit_stops_at_the_upper_limit` exists.
- Acceptance: rows for A-2 to A-5 with the activity at its uncalibrated value (1.0, Table 6 constants as printed) are recorded beside the fitted rows, each with the frozen tolerance and pass computed against it. The fitted row note states that the value sits at its bound. Pass: uncalibrated rows exist and are finite; no tolerance changed.

## C. Equilibrium and reaction thermodynamics (DOVER and the DWSIM fork)

### 18. Kp against Kc, and the standard-state pressure

- Module: `engine/props/src/equilibrium.rs` (cards D.7 to D.12), `engine/props/src/mixture.rs` (`conc`, card A.14), `engine/units/src/xf.rs`, `engine/units/src/wgs.rs`.
- Concept: $K = \exp(-\Delta G^\circ(T)/RT)$ is dimensionless on partial pressures referred to $P^\circ$. A concentration-basis rate law equilibrates at $K_c = K_p\,(P^\circ/RT)^{\Delta n}$. For reforming $\Delta n = +2$; at 1123.15 K and $P^\circ$ = 1 bar the factor is about 115 $(\mathrm{mol\ m^{-3}})^2$. For the shift $\Delta n = 0$ and the two coincide. The error is silent: the solver converges to a plausible number. Two related findings in the DWSIM port: a fugacity basis that omitted the division by $P^\circ$ moved an SMR extent by +129% at 10 bar before the fix, and taking $P^\circ$ as 1 atm against data tabulated at 1 bar moved methane by +1.3% (20 bar) to +2.7% (1 bar).
- Primary source: the identity follows from the ideal-gas law; DOVER cites NIST-JANAF (Chase 1998) and CODATA for the formation data.
- OUTRAM citation: `crates/dover/CLAUDE.md:83-86`; `crates/dover/src/smr.rs:286-329`; `crates/dover/tests/smr_cstr.rs:89-111`; `crates/outram-park-fork-dwsim-libs/src/reactions.rs:146-150`; `crates/outram-park-fork-dwsim-libs/src/reactors/equilibrium_reactor.rs:24-50`; `crates/outram-park-fork-dwsim-libs/src/reactors/gibbs_reactor.rs:35-39`.
- NUS-H2 state: rates in `xf.rs` and `wgs.rs` are on partial pressures in bar with $K_1$ in $\mathrm{bar^2}$, so the identity bites only if a gas-phase path mixes a concentration with a pressure-basis K.
- Acceptance: (a) every K function's doc names its standard state and unit, and a test reproduces the printed source value at one temperature after unit conversion to 1e-12 relative (Zhang Table 1 in $\mathrm{MPa^2}$, HA in $\mathrm{kPa^2}$); (b) a test shows $P^\circ$ is used: the NIST-route equilibrium at Case B solved with $P^\circ$ = 101325 Pa differs from the 1 bar result, and the ratio of the two $K_1$ values equals $(101325/100000)^2$ to 1e-12; (c) a grep test fails if any gas-phase rate in `engine/units/src/` uses `mixture::conc` together with a K from `equilibrium`. Pass: all three.

### 19. The temperature dependence of K carries the heat-capacity integrals

- Module: `engine/props/src/equilibrium.rs` (`KSource::NistGibbs`), `engine/props/src/shomate.rs`.
- Concept: holding $\Delta H^\circ$ and $\Delta S^\circ$ at their 298.15 K values (van 't Hoff with constants) drops $\int \Delta c_p\,dT$ and $\int \Delta c_p/T\,dT$. With DOVER's own data this put $K_1$ at 1123.15 K low by a factor of 11.9 against Hoseinzade and Adams (2017) eq. 51 to 52, and gave an equilibrium CH4 conversion of 0.529 against 0.842 [C, `SP/findings/outram.md` section 6.1].
- Primary source: $\Delta G^\circ(T) = \Delta H^\circ(T) - T\Delta S^\circ(T)$ with $\Delta H^\circ(T) = \Delta H^\circ_{298} + \int_{298}^{T}\Delta c_p\,dT$ and $\Delta S^\circ(T) = \Delta S^\circ_{298} + \int_{298}^{T}\Delta c_p/T\,dT$ (Kirchhoff); data from the NIST WebBook entries `species.rs` already cites.
- OUTRAM citation: `crates/dover/src/smr.rs:270-280`.
- NUS-H2 state: `k_nist_vs_janaf_log_kf` and `k_zhang_vs_nist_within_6pct` exist in `engine/props/tests/m2_acceptance.rs`.
- Acceptance: a guard test computes $K_1$ at 1123.15 K from the same species table both ways and requires the full route to exceed the constant-$\Delta H^\circ$, $\Delta S^\circ$ route by more than a factor of 5, so a regression to the constant form fails. Pass: ratio above 5.

### 20. The equilibrium limit belongs to thermodynamics; kinetics set only the approach

- Module: `engine/units/src/xf.rs`, `engine/units/src/smr_tube.rs`, `engine/units/src/wgs.rs`.
- Concept: the rate law's equilibrium must equal the K the equilibrium module uses. So rates vanish at the equilibrium composition, and a long isothermal bed converges to the equilibrium result whatever the activity. Stale constants in sweeps are item 26.
- Primary source: Xu and Froment (1989) Part I Eqs. 3 to 5, whose driving forces are $p_{CH_4}p_{H_2O} - p_{H_2}^3 p_{CO}/K_1$, $p_{CO}p_{H_2O} - p_{H_2}p_{CO_2}/K_2$ and $p_{CH_4}p_{H_2O}^2 - p_{H_2}^4 p_{CO_2}/K_3$ with $K_3 = K_1K_2$.
- OUTRAM citation: `crates/dover/src/smr.rs:46-72`, `:353-400`; `crates/dover/tests/smr_cstr.rs:114-146`, `:192-242`, `:372-418`.
- NUS-H2 state: `rates_vanish_at_equilibrium` (`xf.rs:317`), `smr_tube_tends_to_equilibrium` (`m5_acceptance.rs:159`), `hts_closes_and_a_huge_bed_reaches_equilibrium` (`wgs.rs:417`) exist.
- Acceptance: extend the vanishing-rate test to a grid of 700 to 1200 K by 100 K and 15 to 40 bar by 5 bar for both `KSource` values: each driving force divided by its first term at most 1e-9. Long-bed test, on the isothermal tube that `smr_tube_tends_to_equilibrium` already builds: CH4 conversion changes by less than 1e-4 absolute when the activity is multiplied by 10, and again when it is divided by 10 with the tube made ten times longer. Pass: both.

### 21. Two independent reactions, and the hard ceilings

- Module: `engine/units/src/smr_eq.rs`, `engine/plant/src/flowsheet.rs` (card A.6).
- Concept: the overall reaction CH4 + 2 H2O = CO2 + 4 H2 is the sum of the other two; listing it as a third extent makes the Jacobian singular. Simple invariants such as the stoichiometric hydrogen ceiling catch gross errors cheaply.
- OUTRAM citation: `crates/dover/src/smr.rs:32-44`; `crates/dover/tests/smr_cstr.rs:420-447`.
- NUS-H2 state: `eq_a_6_reformer_has_two_independent_reactions` exists.
- Acceptance: in every reformer, shift and flowsheet run, $\dot n_{H_2,out} \le \dot n_{H_2,in} + \sum_n (3n+1)\,\dot n_{C_nH_{2n+2},in}$; every $y_i \ge 0$ and $|\sum y_i - 1| \le 10^{-12}$; reformer duty $> 0$. Pass: all runs.

### 22. Le Chatelier is tested where it applies

- Module: `engine/units/src/smr_eq.rs`, `engine/units/src/smr_tube.rs`.
- Concept: the trends Le Chatelier predicts for reforming hold at equilibrium. A rate-limited or heat-transfer-limited unit may run the other way, and that is recorded as a finding. A test asserting the equilibrium trend on a kinetic unit fails for the test's own reason.
- OUTRAM citation: `crates/dover/tests/smr_cstr.rs:264-370`.
- Acceptance: SMR-EQ at the Case B feed: CH4 conversion strictly decreasing over 15, 20, 25, 30, 35, 40 bar; strictly increasing over 973.15 to 1173.15 K in 40 K steps; strictly increasing over S/C 2.5 to 4.0 in 0.3 steps. For SMR-TUBE the same sweeps are printed as a finding without a monotonic assertion. Pass: strict monotonicity at every SMR-EQ grid point.

### 23. Formation-basis and reference-state traps

- Module: `engine/props/src/species.rs`, `engine/props/src/shomate.rs` (cards A.9, A.10), `engine/props/src/steam.rs`, every unit that adds an IF97 enthalpy to a gas-phase balance (`engine/units/src/cooler.rs`, `engine/units/src/flash.rs`, `engine/units/src/regen.rs`).
- Concept: reaction entropies come from absolute (third-law) entropies, and mixing in an entropy of formation is wrong by about the entropy of H2 (about 130 $\mathrm{J\ mol^{-1}\ K^{-1}}$). The feed uses the gas-phase formation enthalpy of water; the liquid and gas values differ by the latent heat, 44.0 kJ/mol at 298.15 K. IF97 enthalpies sit on their own zero (liquid at the triple point), so they enter a formation-basis balance only through a constant offset. Reaction enthalpies are summed from the species table so they cannot disagree with it.
- Primary source: NIST-JANAF (Chase 1998) and CODATA key values as named by DOVER; NUS-H2 cites the NIST WebBook rows of the same origin.
- OUTRAM citation: `crates/dover/src/species.rs:34-62`, `:196-198`, `:220-224`; `crates/dover/tests/smr_cstr.rs:35-62`.
- NUS-H2 state: `latent_heat_298k` and `h2o_branch_continuity_500k` exist and cover part of this.
- Acceptance: (a) $\Delta H^\circ_{298}$ and $\Delta S^\circ_{298}$ of R1 and R2 summed from `species` equal the values computed by hand from the cited WebBook entries to 0.1 kJ/mol and 0.1 $\mathrm{J\ mol^{-1}\ K^{-1}}$ (expected near +206 and -41 kJ/mol); (b) for liquid water, the formation-basis enthalpy used by the units minus the IF97 enthalpy times the molar mass is constant within 1 J/mol from 300 to 450 K, which shows a single offset. Pass: both.

### 24. Conditioning, iteration budgets and step limits

- Module: `engine/units/src/newton.rs`, `engine/units/src/smr_tube.rs` (shooting), `engine/numerics/src/lib.rs` (`fixed_point`), `engine/plant/src/sens.rs`, `engine/wasm/src/solve.rs`.
- Concept: an extreme rate constant can make a residual unsolvable because the residual norm has a local minimum that is not a root; raising the iteration cap fixes none of those cases. An iteration budget is set from a measured grid of cases, and changing it never loosens the tolerance. An unbounded Newton step can drive a flow negative into a clamp where the finite-difference Jacobian carries no information; OUTRAM reports that upstream DWSIM limits a step to consuming at most 80% of any species present.
- OUTRAM citation: `crates/dover/CLAUDE.md:91-111`; `crates/dover/src/smr.rs:74-112` and `:183-198` (a 936-point grid, 0 failures, worst case 20,000 iterations, budget set at the measured worst case).
- NUS-H2 state: `newton.rs` returns `NotConverged` with no best-guess state. A grep of `src/` folders (in-file test modules included) finds 18 `.max(0.0)` and 18 `.clamp(` calls, some inside residuals (`engine/units/src/absorber.rs:513,516`).
- Acceptance: (a) every sweep reports converged and failed counts, and a sweep passes with zero failures on its declared grid; (b) each iteration cap is documented with the worst count measured on a stated grid, and the cap is at least that count; (c) a test counts clamp activations at the converged state of the Case B solve and requires zero, or a doc line per clamp saying why it is safe; (d) updates of mole numbers and loadings use a fraction-to-boundary step limit so iterates stay non-negative without a clamp. Pass: (a) to (d).

### 25. Integrators reach the end of the domain

- Module: `engine/units/src/smr_tube.rs` (adaptive step, `STEP_TOL = 2.0e-5`), `engine/units/src/wgs.rs` (plug flow), `engine/units/src/absorber.rs` (grid, card H.24).
- Concept: an integrator that stops short of the domain end changes answers without warning; OUTRAM found upstream DWSIM's PFR integrating 99% of each segment, worth +1.9% on one case.
- OUTRAM citation: `reactors_vs_upstream_dwsim.md:134`, `:146-150`.
- Acceptance: the final abscissa equals the specified length to 1e-12 relative; integrating 0 to L/2 then L/2 to L agrees with one pass 0 to L to 1e-9 relative on outlet state; halving `STEP_TOL` moves the tube exit temperature by less than 0.1 K and CH4 conversion by less than 1e-3 absolute. Pass: all three.

### 26. Sweeps and repeated solves share no stale state

- Module: `engine/plant/src/sens.rs`, `engine/wasm/src/solve.rs`, `engine/units/src/solvent.rs` (cached PStarTable, S4).
- Concept: a constant built for one temperature and reused across a sweep is right only at the point it was built for.
- OUTRAM citation: `crates/dover/src/smr.rs:397-400`.
- Acceptance: the 825 C reformer case solved fresh equals the same case solved after an 871 C solve in the same process, bit for bit; the same holds for a 30 mass% MEA case run after a 40 mass% direct-solve case. Pass: bit-identical.

## D. Helium properties

### 27. Test helium transport where the plant runs, against an independent source

- Module: `engine/props/src/helium.rs` (cards B.1 to B.4; Petersen, Riso Report 224, 1970).
- Concept: OUTRAM's helium viscosity follows Arp, McCarty and Friend, NIST Technical Note 1334 (1998), and its conductivity follows Hands and Arp, with the near-critical enhancement (3.5 to 12 K) left out. Its literature checks run only at 300 K and 400 K, at 2% on viscosity and 5% on conductivity; its EOS checks compare pressure from $(T,\rho)$ with CoolProp's tabulated triple and critical states, a reference-state-independent code-to-code check. Nothing is tested at HTGR temperature. Our helium module is tested against Petersen's own table (`helium_petersen_points`, 7e-4), which is verification of the transcription.
- Primary source: NIST TN-1334 (Arp, McCarty and Friend 1998) and Hands and Arp, as named by OUTRAM; Petersen 1970 for the engine. TN-1334 is absent from `SP/findings/inventory_master.csv` (grep on "1334", "helium", "Arp" found no entry), and CoolProp Python is not installed on this machine.
- OUTRAM citation: `crates/outram-park-fork-coolprop/src/transport.rs:425`, `:655`, `:902-929`; `crates/outram-park-fork-coolprop/tests/transport_vle.rs:52-67`; `crates/outram-park-fork-coolprop/tests/helium_reference.rs:1-17`.
- Acceptance: independent rows for viscosity and conductivity at 700, 900, 1100 and 1200 K and 50 bar, with tolerance frozen before the run at twice the standard deviation Petersen states (viscosity $0.0015\,T$ %, conductivity $0.0035\,T$ %, so 3.5% and 8.2% at 1173 K). Values read from the TN-1334 tables carry `data_use = independent`; values from CoolProp carry `code-to-code`. Also assert $c_p$ = 5195 $\mathrm{J\ kg^{-1}\ K^{-1}}$ lies within 0.1% of the monatomic ideal value $\tfrac52 R/M$ = 5193.2. Pass: rows produced with pass computed against the frozen tolerance; if no source is obtained, the rows are `not_run` with that reason.

### 28. One gas constant per purpose, with its source

- Module: `engine/props/src/units.rs` (`R_GAS` = $N_A k_B$, exact), `engine/mea/src/pstar.rs:21` (`R_GAS = 8.314462`, the prototype's value, also used by `engine/units/src/absorber.rs:56`), `engine/units/src/xf.rs:30` (`R_KJ_MOL_K = 8.314e-3`), `engine/props/src/helium.rs` (Petersen's 2077.3 $\mathrm{J\ kg^{-1}\ K^{-1}}$).
- Concept: OUTRAM's DWSIM port carries a truncated 8.314 in its rate laws and CODATA in its thermodynamics; the CoolProp helium file carries 8.3144598. An Arrhenius exponent near 40 turns a 5.6e-5 relative difference in R into about 2e-3 in k. The rule: thermodynamic quantities use the exact R; a rate law uses the R its constants were fitted with, and the doc cites the page that shows it.
- OUTRAM citation: `crates/dover/CLAUDE.md:87-90`; `crates/dover/src/smr.rs:152-180`; `crates/outram-park-fork-coolprop/src/fluids/helium.rs:17`.
- Acceptance: a test asserts `mea::pstar::R_GAS` equals `props::units::R_GAS` to 1e-6 relative or the constant carries a doc reason; `xf.rs` cites the Xu and Froment page that fixes its R, or switches to the exact value with the change in $k_1$ at 800 K printed (expected near 2e-3 relative for an activation energy of 240 kJ/mol). Pass: every R in `engine/` is exact or carries a cited reason.

## E. IF97 practice

### 29. Test IF97 by code path, and test the region edges the plant touches

- Module: `engine/props/src/steam.rs` (forward $(p,T)$ and saturation calls through `seuif97`; no backward $(p,h)$ or $(p,s)$ call in the engine by grep).
- Concept: OUTRAM tests IF97 regions against the release's verification values at 1e-9 relative (1e-8 on entropy), states a separate and looser bar for backward equations (5e-5) and flash interfaces, and warns that conflating the paths overstates the code. It found the forward Region 1 equation and a backward $T(p,h)$ correlation disagree about the 273.15 K floor by 17.7 mK at 4 MPa, enough to push a state out of range and panic; the defect is kept as a test that fails the day it is fixed. A heated-tube test asserts a discrete maximum principle: no cell can fall below its feed enthalpy when every wall is at or above the feed temperature, with a 1 kJ/kg allowance for pressure work.
- Primary source: IAPWS R7-97(2012), IF97, verification tables. The release is not in the local inventory.
- OUTRAM citation: `crates/tampines-steam-tables/CLAUDE.md:12-37`; `crates/tampines-steam-tables/src/region_1_subcooled_liquid/tests/set_a_region_1_300_kelvin_3_mpa.rs:15-74`; `crates/tampines-steam-tables/tests/boundary_273_15_repro.rs:1-26`, `:66-107`; `crates/tampines-steam-tables/tests/heated_tube_never_undercuts_its_feedwater.rs:2-47`.
- NUS-H2 state: `if97_release_vectors` (42 values at 1e-8 from release Tables 5, 15, 35 and 36), `psat_vs_tsat_backward`, `liquid_enthalpy_refuses_vapour_states` and `out_of_range_is_an_error` exist.
- Acceptance: (a) at each pressure the plant uses for steam (reboiler steam, export steam, condensate) `state()` at $T_{sat}(p) \pm 1$ mK returns the phase that side of the line implies, or a Range error, and never the other phase; (b) the dilute Region 2 path used below 500 K documents its domain and is tested at its lower edge; (c) if a backward call is ever added, a round-trip test uses the permissible inconsistency stated in the release, read from the page; (d) no doc quotes IF97 accuracy for a quantity the engine derives from it. Pass: (a), (b), (d) now, (c) when triggered.

## F. Heat exchanger V&V practice (TUAS)

### 30. Validate an exchanger in layers before comparing outlet temperatures

- Module: `engine/units/src/tube_ht.rs` (JAERI A-1 and A-2), future `units::ihx` (cards C.1 to C.19), `engine/units/src/regen.rs` (rich-lean exchanger, I.10), `engine/units/src/cooler.rs`.
- Concept: the TUAS validation against Du et al. (2018) checks, in order: geometry (shell hydraulic diameter within 3% of the paper); fluid properties at the paper's temperatures (1%); dimensionless groups Re, bulk and wall Pr, and Nu per data point against the paper's printed values (1%); each thermal resistance computed independently (wall term checked against a hand value); U backed out from measured data as $U = \bar Q/(A\,\Delta T_{lm})$ and split into resistances to find which film is off; the data's own hot-side and cold-side heat balance within the 5% the experiment states; and node refinement up to 25 nodes, with 12 found sufficient.
- Primary source: Du, He, Qiu, Liang and Zhou (2018), Int. Commun. Heat Mass Transf. 96, 61-68, as named by TUAS; the resistance sum $1/U_o = (1/h_i)(d_o/d_i) + d_o\ln(d_o/d_i)/(2\lambda_w) + 1/h_o$ as quoted there.
- OUTRAM citation: `crates/tuas_boussinesq_solver/src/lib/pre_built_components/shell_and_tube_heat_exchanger/tests/hitec_to_yd325_du_hx/mod.rs:1-58`; `.../dimension_checks.rs:1-5`; `.../heat_transfer_correlations_checks.rs:1-15`; `.../set_a.rs:96-97`, `:540-545`, `:835-841`.
- NUS-H2 state: `tube_ht.rs` reruns JAERI's printed worked sheet (`bed_coefficient_reruns_the_printed_sheet`, `overall_coefficient_reruns_the_printed_sheet`, `helium_film_coefficient_reruns_the_printed_sheet`).
- Acceptance: for each exchanger benchmark, the layers above run as separate tests, each at the precision the source prints. The outlet comparison row is labelled validation only when every lower layer passes. Every exchanger asserts the bound $\min(T_{h,in},T_{c,in}) \le T_{out} \le \max(T_{h,in},T_{c,in})$ for both streams, effectiveness at most 1, and no temperature cross at any node of the tube ($T_{He} > T_{bed}$ everywhere for a heated tube). Pass: all layers and bounds green.

### 31. Tolerance comes from the reference; regression baselines are a separate layer

- Module: `engine/vv/src/bench.rs`, `engine/vv/src/golden.rs`, `docs/VALIDATION_MATRIX.md`.
- Concept: TUAS sets its benchmark band from the reference publication's own agreement with experiment (SAM within 6.65% on primary flow and 6.76% on DRACS flow) and keeps tight regression baselines apart from it (0.1% on flows, 0.01 K on temperatures). When a model change pushed four points past their bands, TUAS widened those bands per point with written reasons and kept the global band. NUS-H2 rule S1 freezes tolerances, so only the documentation half is adopted: a miss stays a fail with its margin stated.
- OUTRAM citation: `crates/tuas_boussinesq_solver/verification_and_validation/coupled_dracs_pipe38_K17p8_vs_ciet_experiment.md:25-31`, `:99-133`.
- Acceptance: every new benchmark row added after this memo names where its tolerance comes from in `docs/VALIDATION_MATRIX.md` before it runs, for example the source's stated uncertainty or its printed precision. A vv test fails if a tolerance in the CSV differs from the matrix value. Pass: zero differences.

## G. Units, error handling and range guards

### 32. Units are named at every API boundary

- Module: public functions in `engine/props`, `engine/units`, `engine/plant`; `engine/props/src/units.rs` (SI with mol/s and W internally; project units only at edges).
- Concept: OUTRAM requires a typed quantity wherever a physical value crosses an API, named aliases for compound units, and doc comments that still spell out units, ranges and assumptions. NUS-H2 uses plain `f64` with unit suffixes (`t_k`, `p_pa`, `k1_zhang_bar2`); the same goal can be checked by a lint.
- OUTRAM citation: `CLAUDE.md:716-717`; `docs/claude-md/api-design-and-maturity.md:15-25`.
- Acceptance: a vv test parses public function signatures and fails on any physical parameter or return without a unit suffix or a doc line naming its unit; the list of allowed suffixes lives in one place. Equilibrium constants carry their unit in the name or doc ($\mathrm{bar^2}$, dimensionless). Pass: zero violations or a listed exception with a reason.

### 33. No reachable input panics, and compiling for wasm32 does not show that it runs

- Module: `engine/wasm/src/solve.rs`, `engine/wasm/src/registry.rs`, `engine/numerics/src/lib.rs` (`EngineError`: "User input never reaches a panic").
- Concept: OUTRAM's own wasm gate states that compiling does not imply running; its notes record a timer call that panics at run time on `wasm32-unknown-unknown`, and a steam-table bound check that panics on a state a transient can reach.
- OUTRAM citation: `SP/findings/outram.md` section 7 (`scripts/check-wasm.sh` header and the timer calls it cites); `crates/tampines-steam-tables/tests/boundary_273_15_repro.rs:28-50`.
- NUS-H2 state: `rust_refuses_out_of_range_and_unknown_input` and `set_refuses_non_finite` exist.
- Acceptance: (a) for every registry parameter, a solve at min, max, min minus 1e-9 relative, max plus 1e-9 relative, NaN, and both infinities, each under `catch_unwind`, gives zero panics, and every Err names the parameter; (b) a grep gate over crates built for wasm32 fails on `std::time::Instant`, `SystemTime`, `thread::spawn` or `std::fs` outside `#[cfg(test)]`; (c) the Stage 5 browser check runs one full solve. Pass: all three.

### 34. Every correlation states its domain, and every guard follows a declared policy

- Module: `engine/props/src/helium.rs` (hard Range error outside 273 to 1800 K and 1 to 100 bar), `engine/props/src/transport.rs` (flag outside 359 to 483 C), `engine/mea/src/pstar.rs` and `engine/units/src/solvent.rs` (notes such as `pco2_above_reliable` and `cold_extrapolated`; PStarTable only at or below 30 mass%, S4), `engine/units/src/xf.rs` (`check` refuses any set other than Table 6).
- Concept: OUTRAM requires each public function's doc to give valid input ranges and assumptions; its correlations record what they omit (helium conductivity has no near-critical term). OUTRAM's porting rule adds that a missing limit is a more common port defect than a wrong formula, since formulas get checked line by line and control flow does not.
- OUTRAM citation: `docs/claude-md/api-design-and-maturity.md:15-18`; `CLAUDE.md:259-262`; `crates/outram-park-fork-coolprop/src/transport.rs:655`.
- Acceptance: each correlation's doc cites its source domain with page and declares its policy (error, or value with flag). A table-driven test calls each correlation just inside each edge (Ok) and 1e-6 relative outside (Err or flag, matching the declared policy). No flagged value reaches a Headline number unless the row note carries the flag. Pass: all edges behave as declared.

## H. Bookkeeping of traceability, maturity and sign-off

### 35. The equation register and the code agree in both directions

- Module: `math/eq_register.csv`, all of `engine/`.
- Concept: theory, implementation, verification and validation should be traceable in both directions. A doc claim the code contradicts is a defect to fix in the same change, and a dated "still true" stamp proves only when someone last looked.
- OUTRAM citation: `VERIFICATION_AND_VALIDATION.md:117-136`; `CLAUDE.md:402-445`.
- NUS-H2 state [C, approximate name grep run 2026-11-02]: of 299 register rows, 77 `engine_fn` references resolve to an item of that name in the named module and 222 do not. Examples: B.1 names `props::helium::rho` where the code has `density_kg_m3`; G.21 names `props::mixture::srk_phi` where the code has `mea::pstar::srk_phi_co2`; the modules `units::ihx`, `units::htgr`, `units::circulator`, `plant::heatrec`, `plant::utility` and `numerics::ode` are absent. "eq: X.n" doc lines appear in 5 files and cover 58 cards; about 25 cards have a test named `eq_x_n_...`. Test names mix padding (`eq_a_2_...`, `eq_h_06_...`) while the register writes `eq_a_02`. The grep can miss items whose path differs from the register's, so the counts are an upper bound on mismatches. They describe the tree at `f796752`; by the end of this review HEAD was `950bcdf` and another agent had an untracked `engine/units/src/circulator.rs`, so rerun the count before acting on it.
- Acceptance: a vv test parses `eq_register.csv` and requires each `engine_fn` to resolve to an item that carries "eq: X.n", or the row to be marked as not built in an explicit column; every test named `eq_x_n_...` maps to a register row under one padding convention. Pass: zero unresolved and zero unmarked rows.

### 36. Keep the evidence state and the author's sign-off apart from the model class, and overstate neither

- Module: crate READMEs or a status section in `docs/VALIDATION_MATRIX.md`; module headers in `engine/`.
- Concept: OUTRAM keeps the records listed in section 0 apart, and only the maintainer may flip a sign-off. Superseded bars stay in the record, and corrections are shown with the old value and a dated note. An agent may propose a status and never sets it. For NUS-H2 the user is the only person who can sign off, so every module defaults to "not yet checked by the author".
- OUTRAM citation: `crates/dover/Cargo.toml:13-28`; `docs/claude-md/api-design-and-maturity.md:40-90`; `docs/claude-md/bookkeeping-and-api-docs.md:40-60`; `docs/claude-md/human-review.md:32`, `:39-53`; `CLAUDE.md:536-538`.
- Acceptance: one table per crate with three columns: class (Rigorous, ReducedOrder, Empirical), evidence state from Validation_Results (verification only, benchmark pass, benchmark fail, not run), and author sign-off (default not yet checked; only the user changes it). A vv test fails if any doc, report text or site label calls a module "validated" while its rows are fail or not run, or while its sign-off is unchecked. Pass: zero contradictions.

## Primary sources OUTRAM names that bear on NUS-H2

- NIST-JANAF Thermochemical Tables, 4th ed. (Chase 1998) and CODATA key values: formation data (`crates/dover/src/species.rs:36-40`).
- Arp, McCarty and Friend (1998), NIST Technical Note 1334: helium viscosity (`crates/outram-park-fork-coolprop/src/transport.rs:902-903`).
- Hands and Arp: helium thermal conductivity (`crates/outram-park-fork-coolprop/src/transport.rs:926-929`).
- IAPWS-IF97 verification values (`crates/tampines-steam-tables/CLAUDE.md:25-29`).
- Du et al. (2018), shell-and-tube molten salt exchanger data (`.../hitec_to_yd325_du_hx/mod.rs:5-8`).
- Zou et al. (2019, 2021) SAM and CIET; Zweibaum (2015) (`coupled_dracs_pipe38_K17p8_vs_ciet_experiment.md:33-44`).
- DWSIM 9.0.5.0 at commit `1abf72d1`, as reference software for code-to-code work (`reactors_vs_upstream_dwsim.md:100-111`).
- Xu and Froment (1989), named by DOVER and deliberately not embedded there (`crates/dover/src/smr.rs:66-72`); NUS-H2 already uses it from the paper.

## What OUTRAM does not cover

No amine, capture, PSA or CO2 conditioning capability exists in OUTRAM (`SP/findings/outram.md` section 4); its MEA file is an incompressible property fit with no CO2 loading. Nothing in this memo bears on the MEA thermodynamics or the column models beyond items 3, 9, 10 and 24.

## Priority for the applying agent

1. Item 3 (SRK at 26.8 bar): my calculation suggests a large error in plant-absorber p*. Confirm it with an engine test first and record the prediction before any fix.
2. Items 35 and 36 (traceability and status): cheap, and they stop overstated claims.
3. Items 2, 18, 20, 24, 25 and 30: conservation, standard state, consistency and numerical checks.
4. The rest as time allows, each adding rows or tests without changing a frozen tolerance or golden value.
