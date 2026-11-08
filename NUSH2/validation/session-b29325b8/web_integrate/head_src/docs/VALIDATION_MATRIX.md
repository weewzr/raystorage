# NUS-H2 validation matrix

This file fixes what each benchmark compares, where the literature number comes from, how the engine reproduces it and how close the engine must come. Results reach `results/Validation_Results.csv` when the harness in `engine/vv` runs (`nush2 export`). The Status column of each table summarises that file as exported with this revision: run and pass, run and fail, reported with no limit, or not run with its reason. A benchmark with several rows is counted row by row. Section 19 covers the rows outside the brief's sources and gives the totals with the list of failed rows.

## 1. Rules

- Tolerances are frozen at the commit that adds this file. A later change adds a new row with its reason, and the old row stays.
- A failed benchmark stays recorded as failed. No constant is tuned to pass and no literature value is edited.
- A row is written as `pass` or `fail` only after it ran against a limit. A row printed with no limit is `reported`. A benchmark that cannot run is written as `not_run` with its reason, from section 18 where it is listed there.
- Code tests and literature benchmarks are separate. `cargo test` checks code correctness and regressions and must be green. Literature benchmarks run through `nush2_vv` and write rows with their status. A test of a benchmark asserts that the row is produced, finite and judged against the unchanged limit, never that it passes.
- Numbers come from `engine/vv/data/benchmarks/` and `engine/vv/data/props/`. No literature value is typed into a test. Where a unit crate holds a typed copy of a printed number, a harness row checks the copy against the fixture (T961-FIXTURE, XF-2-TABLES, AR-1-FIXTURE).
- A fixture holds only values a test or benchmark reads, each with its source and page. Where a benchmark does not run, its row carries the single printed target when the matrix names one, and the other values were removed. Removed values stay in the repository history at commit `eaf9327`, which added the fixtures.
- Data use takes one of three values, as in `results/Validation_Results.csv`:

| Value | Meaning |
|---|---|
| `independent` | The number played no part in building or tuning the engine term under test |
| `calibration` | The number was used to set the engine term, so agreement proves the fit and nothing more |
| `same_dataset_disclosed` | The number shares a dataset with an engine term or input without being fitted here, so agreement is verification and earns no validation credit |

Agreement with a value marked `reported_model` or `design` in a fixture compares two calculations. It is a code comparison and earns no credit as experimental validation. Only `measured` rows can validate physics.

Engine calls with no tag exist in the source tree today. A tag such as (M4) names the milestone that delivers the call, and the name is the one in the engine architecture plan. New test names proposed here are marked (new).

### Decisions applied to this record

The project lead settled these open items after the second build stage. Rows and notes cite them by label.

| Decision | Effect on this record |
|---|---|
| S1 | Code tests and literature benchmarks are separate. A benchmark that misses stays a failed row, and its test checks only that the row is produced, finite and judged against the unchanged limit |
| S2 | The low-temperature shift keeps the Zhang K2 by default, because T961-6 passes with it. The NIST Gibbs K2 runs as one sensitivity row, T961-6-CO-NIST |
| S3 | The JAERI A-2 activity at the 2.246 search limit is the single disclosed tuned value. The holdout misses are structural and reported |
| S6 | The repeated Aronu Table 3 row is disclosed in sections 9 and 20 |
| S7 | A fixture holds only the values a test or benchmark reads, each with source and page |
| S9 | XF-1 passes only at the plan's 3-decimal precision, and each row note gives the unrounded ratio |
| S11 | The Level 3 carbon comparison failed, so FS_GATE_M7 is false and terms T04, T09 and T13 stay anchored |
| S12 | The column gate failed, so terms T07 and T08 stay anchored and the SRD in Headline stays at 3.8 GJ/t |

## 2. Fixtures

Each file holds one source. The columns are the same in all of them.

| Column | Content |
|---|---|
| `source` | Citation |
| `locator` | Table or section with its PDF page |
| `read_method` | `table`, `text`, `page_image` or `vector_figure` |
| `data_kind` | `measured`, `reported_model`, `design`, `constant` or `inventory` |
| `series` | Identifier, unique with `quantity`, `locator` and `conditions` |
| `quantity` | Name of the number |
| `value` | The number as printed, with spaces and separators removed |
| `unit` | Unit as printed |
| `read_uncertainty` | The printed plus or minus value, or the figure reading band, in the row's unit. Blank when neither applies |
| `conditions` | Semicolon list of `key=value` |

Rows at `eaf9327` counts the fixture as extracted. Rows now counts it after lead decision S7, which keeps only the values a benchmark reads. The harness tests `fixtures_hold_only_read_values` and `column_rows_and_fixtures` fail if a stored row is never read, and an export from the full `eaf9327` fixtures, with the one row added below, gives the same `Validation_Results.csv`, byte for byte, as the trimmed set. A file that no benchmark reads was removed.

| File | Source | Rows at `eaf9327` | Rows now | Printed page |
|---|---|---|---|---|
| `tev953.csv` | INL TEV-953 Rev. 0, 2010 | 149 | 27 | PDF - 1 |
| `tev961.csv` | INL TEV-961 Rev. 0, 2010 | 285 | 56 | PDF - 1 |
| `xu_froment_1989.csv` | Xu and Froment, AIChE J. 35 (1989) 88-96 | 81 | 28 | PDF + 87 |
| `wgs_chinchen_1984.csv` | Chinchen et al., Appl. Catal. 12 (1984) 69-88 | 161 | 25 | PDF + 68 |
| `wgs_kraussler_2017.csv` | Kraussler et al., Biomass Conv. Bioref. 7 (2017) 153-165 | 109 | 4 | PDF + 152 |
| `zhang_que_chen_2011.csv` | Zhang, Que and Chen, Fluid Phase Equilib. 311 (2011) 67-75 | 108 | Removed | Not checked |
| `aronu_2011.csv` | Aronu et al., Chem. Eng. Sci. 66 (2011) 6393-6406, with corrigendum 78 (2012) 246-247 | 352 | 348 | PDF + 6392, corrigendum PDF p16 is p246 |
| `hartono_2014.csv` | Hartono et al., J. Chem. Eng. Data 59 (2014) 1808-1816 | 280 | 166 | PDF + 1807 |
| `versteeg_1988.csv` | Versteeg and van Swaaij, Chem. Eng. Sci. 43 (1988) 573-585 | 20 | 14 | PDF + 571 |
| `khan_2011.csv` | Khan, Krishnamoorthi and Mahmud, Chem. Eng. Res. Des. 89 (2011) 1600-1608 | 55 | 5 | PDF + 1599 |
| `isa_2017.csv` | Isa et al., Int. J. Greenh. Gas Control 56 (2017) 173-186 | 233 | 1 | PDF + 172 |
| `abdul_halim_2015.csv` | Abdul Halim et al., Ind. Eng. Chem. Res. 54 (2015) 1675-1680 | 64 | 1 | PDF + 1674 |
| `salvinder_2018.csv` | Salvinder et al., Int. J. Greenh. Gas Control 70 (2018) 164-177 | 17 | 11 | PDF + 163 |
| `vinjarapu_2024_parametric.csv` | Vinjarapu et al., Appl. Energy 355 (2024) 122193 | 1658 | 768 | Same as PDF |
| `vinjarapu_2024_split_flow.csv` | Vinjarapu et al., Sep. Purif. Technol. 345 (2024) 127311 | 78 | Removed | Not checked |
| `vinjarapu_2025_interheat.csv` | Vinjarapu et al., Energy 320 (2025) 134973 | 75 | Removed | Not checked |
| `brigman_2014.csv` | Brigman et al., Energy Procedia 63 (2014) 6012-6022 | 68 | 3 | PDF + 6011 |
| `iaea_tecdoc_1236.csv` | IAEA-TECDOC-1236, 2001 | 120 | 45 | PDF - 6 |

The trim added one row: `tev953.csv` now holds the stage 1 steam to carbon ratio of 3.00 from block NG-RFMR (A-42 PDF p76), which T953-1 reads. Section 19 lists the fixtures added after `eaf9327` for the sources outside this list, and `engine/vv/data/props/` holds the property fixtures of the M2 rows, each citing its table and page in comment lines.

### How the extraction was checked

These checks ran on the full extraction at `eaf9327`, before the trim.

| Check | Result |
|---|---|
| Value appears as a digit string on the cited page text layer | 3,742 of 3,750 rows with a text layer |
| Rows the matcher could not place | 8. Five are text layer faults read from the page image (Chinchen 0.334 and 0.10; Xu-Froment 0.2211, 0.4152, 0.0317). Three are `5,160kg/h` and `1,290kg/h` in TECDOC-1236, printed with no space before the unit and confirmed in the text |
| Rows from scans with no text layer | 116 from TEV-961 A-50, A-53, A-55 and A-56, read at up to 330 dpi |
| Rows from vector figures | 47, calibrated on gridlines or tick marks. Brigman Figs 3 and 4 give 33, Khan Fig 3 gives 14 |
| Figure calibration against printed values | Khan Fig 3a base 19.12 against Table 1 19.1 mol%. Fig 3b top 3006 against Table 1 3000 mol/m3. Brigman minima 3.70 and 3.40 against text 3.7 and 3.4 MJ/kg |
| Aronu against the separately typed `engine/mea/tests/data` | All 104 model points and all 36 Table 4 points agree |
| Hartono against the same folder | Density, viscosity and N2O points held there agree, 24, 24 and 20 points |
| TEV power rows sum to the printed total | Within 0.1 MW in the four TEV-961 and four TEV-953 columns tested |
| TEV-961 Case 5 balances | KO drum split closes within 2 lb/h. 630342 lb/h gives 79.42 kg/s. 1562 and 835 F give 850 and 446 C. Composition sums reach 100 within rounding |
| Vinjarapu Table 5 SS1 against steady state 4 of Tables D.13 and D.19 | Lean flow 229 kg/h and SRD 3.73 agree |
| Internal schema | Ten columns, numeric `value`, a PDF page in every locator, no duplicate keys, in all 18 files |

The matcher proves that a number is on the page. It cannot prove the number sits in the right cell, so placement was checked separately, at the level given here.

| Level of placement check | Tables |
|---|---|
| Read cell by cell against the page image | Chinchen 1, 2, 3, 5. Xu-Froment 1, 5, 6. Abdul Halim 3. Isa 7, 8, 9, 10, 12. Kraussler 4, 5. Khan 1. Aronu 3, 4, 5. Versteeg 10. Hartono 2, 3, 10, 11. TEV-953 Table 2. TEV-961 Table 2 and A-50, A-53, A-55, A-56 |
| Read against ordered text from the text layer | Zhang 8, 9, 10, 11. Isa 3. Salvinder 1, 2. Abdul Halim 1. Xu-Froment 7. Hartono 6, 7. TEV-953 A-40, A-45. TECDOC-1236 tables. Vinjarapu split flow and interheating tables |
| Parsed by position, stopping on any malformed row | Vinjarapu D.10 to D.19, with rows 1, 59 and 85 of D.19 read against the page |
| Digit match only | Aronu 6, whose model column is covered by the 104 point comparison. Zhang 7. Kraussler 1, 2, 3 |

Conditions were audited as well as values. One condition inherited from an earlier extraction, zero lean loading for Abdul Halim, is printed nowhere in that paper and was removed.

## 3. What was already known when tolerances were set

Fixing a tolerance before any run has a limit here. Python prototypes and hand arithmetic ran during design, so these limits were set with an expected outcome in view. None of them should be read as a blind test.

| Benchmark | Known beforehand | Source of that knowledge |
|---|---|---|
| AR-1 | Prototype AARD 0.20% | Engine architecture plan section 8 |
| AR-2 | Prototype AARD 21.9% | Same |
| HA-1 | Weiland density AARD 0.51% | Design evidence, Python |
| HA-2 | Weiland viscosity AARD 1.4% | Same |
| XF-1 | Ratios 0.986 to 1.004, computed from the same two tables | Engine architecture plan section 12 |
| T953-2 | Pure methane equilibrium at S/C 3 and 30.3 bar gives 80.3%, about 2.2 points above INL | Design evidence, hand calculation. This row may fail |
| VER-1 | Khan Eq 2 gives 4.419, 5.868, 7.721 against 4.42, 5.87, 7.72 | Hand calculation for this file |
| VER-2 | Khan Eq 2 sits 19.4, 20.0 and 19.2% above Penny and Ritter | Hand calculation for this file. The 298 K row sits on the limit and may fail |
| T953-3, T961-1, T961-2 | Helium duty with cp 5.193 kJ/kg/K reproduces the printed values | Design evidence |
| HA-3 | Khan Eq 8 has no loading term, while measured kH rises by about 70% from loading 0 to 0.5 at 25.2 C | Reading of Eq 8 and Table 11. This row is likely to fail at high loading |

The M2 and M3 tests already in the tree carry limits from the architecture plan, and those limits were not changed here. Their benchmark rows, with status, are listed in section 19.

The approved SRD of 3.8 GJ/t (range 3.4 to 4.2) was chosen with sight of Vinjarapu and Brigman. A comparison of that anchored value with either paper is `same_dataset_disclosed`. A comparison of the modelled STRIP output with them is `independent`, which is why the rows below test STRIP.

## 4. TEV-953

INL model results, Aspen with Gibbs minimisation, with no plant measurement behind them. Every row is a code comparison.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| T953-1 | 1 | HTGR with capture, reformer exit 1600 F, 439 psia (A-45 PDF p79), NG as A-40 | Raw syngas, wet | H2 47.8, CO 9.2, CO2 5.3, N2 0.2, H2O 34.1, CH4 3.5 (A-40 PDF p74) | mol% | `props::equilibrium::reform`, test `inl_a40_raw_syngas` | 0.5 mol% each | `independent` | Run, pass, 6 of 6 rows |
| T953-2 | 1 | Conventional, 871 C, S/C 3 (PDF p18). 439 psia carried over from A-33 and A-45, because the conventional sheets have no text layer and were not read | CH4 conversion | 78.1 (PDF p18) | % | `props::equilibrium::reform`, test `tev953_conversion` (new) | 1.0 point | `independent` | Run, fail. Failed: 80.33 against 78.1% |
| T953-3 | 1 | HTGR with capture, 663671 lb/h, 1292 to 807 F (A-45 PDF p79) | Process heat | 117 (Table 2 PDF p30) | MWt | `props::helium::duty_w`, test `tev953_process_heat` (new) | 1.0%, printed to three figures | `same_dataset_disclosed` | Run, pass |
| T953-4 | 5 | Conventional without capture, 29000 lb/h H2 | CO2 intensity | 3205 short ton/day (Table 2 PDF p30), which gives 9.21 kg/kg by calculation | kg/kg | `plant::emissions`, freeze baseline | 0.1% | `same_dataset_disclosed` | Run, pass |

T953-4 checks the conversion behind the 9.21 baseline, which is an input to the emissions ledger. No benchmark read the steam, water and capture rows of `tev953.csv`, so the S7 trim removed them. A later Level 3 or 4 comparison that needs them takes them from commit `eaf9327` with their citations.

T953-2 failed as section 3 expected. Pure methane at S/C 3 stands in for the conventional feed, and the 2.2 point gap has the size that substitution predicts.

## 5. TEV-961

The local PDF ends at A-59, the end of Case 5. Case 6, the design case, survives only in Table 2 and its continuation. Case 5 is the closest full flowsheet, so it carries the unit tests, and a pass here says nothing direct about Case 6.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| T961-1 | 1 | Five HTGR cases, helium flow and temperatures (Table 2 continued PDF p18) | Process heat demand | 116.9, 132.2, 149.1, 166.6, 176.8 (Table 2 PDF p17) | MWt | `props::helium::duty_w`, test `he_duty_176p8`, and `tev961_table2_reconstruction` (M8) | 0.15% each, 0.2% for 176.8 | `same_dataset_disclosed` | Run: 4 pass, 1 fail. Failed: ROT825C 149.4 against 149.1 MWt |
| T961-2 | 1 | Five HTGR cases, helium flow across return less ideal return | Excess heat | 55.6, 36.5, 22.6, 10.3, 2.0 (PDF p18) | MWt | `props::helium::duty_w`, test `tev961_excess_heat` (new) | 0.5 MWt, since two temperatures printed to 1 K carry 0.4 MWt | `same_dataset_disclosed` | Run, pass, 5 of 5 rows |
| T961-3 | 1 | Case 5, exit 1600 F, 439 psia (A-55 PDF p79) | Raw syngas, wet | H2 48.8, CO 8.7, CO2 4.8, N2 0.7, H2O 33.4, CH4 3.6 (A-50 PDF p74) | mol% | `props::equilibrium::reform`, test `case5_raw_syngas` (new) | 0.5 mol% each | `independent` | Run, pass, 6 of 6 rows |
| T961-4 | 2 | Case 5, 1000 to 1600 F, 349724 lb/h | REFORM-1 plus burner duty | 404.2 + 74.0 MMBtu/h, which is 140.2 MW (A-55 PDF p79) | MW | `units::smr_eq` (M4), test `case5_reformer_enthalpy` | 3% | `independent` | Run, pass |
| T961-5 | 2 | HTS inlet 635 F, steam to dry gas 0.50 | Outlet T, dry CO | 764 F (407 C), 3.01 (A-53 PDF p77) | C, mol% | `units::wgs` (M4), test `wgs_case5_hts` | 10 K, 0.3 point | `independent` | Run: 2 pass, 2 fail. Failed: GHSV-CO 7.063 against 3.01 mol%; GHSV-T 377.5 against 406.7 C. The two GHSV rows record the first basis tried, Chinchen's 5000 1/h test space velocity, and stay failed |
| T961-6 | 2 | LTS inlet 375 F, steam to dry gas 0.37 | Outlet T, dry CO | 415 F (213 C), 0.33 (A-53 PDF p77) | C, mol% | `units::wgs` (M4), test `wgs_case5_lts` | 10 K, 0.1 point | `independent` | Run, pass, 2 of 2 rows. T961-6-CO-NIST reruns the LTS on the NIST Gibbs K2 as a reported sensitivity (decision S2) |
| T961-7 | 2 | Case 5 outlet temperatures as A-56 PDF p80 | Cooler duties | HPS-GEN1 236.3, WGS-EX1 66.7, EX2 24.2, EX3 65.7, EX4 117.1, KO-DRM-1 19.7 (A-55 PDF p79, A-56 PDF p80), which are 69.3, 19.5, 7.1, 19.3, 34.3, 5.8 MW | MMBtu/h | `units::cooler` (M4), test `cooler_case5_duties` | 10% each | `independent` | Run, pass, 6 of 6 rows |
| T961-8 | 2 | KO drum at 104 F, 389 psia | Condensate, dry CO2 | 126553 lb/h, which is 57.4 t/h (A-56 PDF p80). 17.7% (A-53 PDF p77) | t/h, % | `units::flash` (M4), test `flash_case5_condensate` | 3%, 0.3 point | `independent` | Run, pass, 2 of 2 rows |

The T961-7 row is the gate for flipping term T05. If it misses, T05 stays anchored and the row stays failed.

The six preheater duties PRE-HT-1 to PRE-HT-6 on A-55 are left out of the fixture. The scan does not separate the digits 3, 6 and 8 reliably. Helium enthalpy and the sum to 568.4 MMBtu/h suggest values, and the fixture holds no inferred value. The equivalent TEV-953 sheet, A-45, has a text layer; its duties were extracted at `eaf9327` and removed by the trim, since no benchmark reads them.

T961-1 at the 825 C reactor outlet misses its 0.15% limit by 0.03 points (149.37 against 149.1 MWt, 0.18%). The other four cases pass. The row stays failed.

## 6. Xu and Froment

Rate constants enter the engine from Table 5 and Table 6, so nothing here validates the kinetics against experiment. The papers show their experimental rates only in figures.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| XF-1 | 2 | 648 K and 823 K | k1, k2, k3, KCO, KH2, KCH4, KH2O from Table 6 with E and dH | Table 5 and Table 6 PDF p7 | As printed | `units::xf` (M5), test `xf_table5_vs_table6` | Ratio 0.986 to 1.004 | `same_dataset_disclosed` | Run, pass, 7 of 7 rows. Judged on the ratio rounded to 3 decimals, see the paragraph below the table |
| XF-2 | 2 | Reference activity | KH2O preexponential, fresh catalyst factor unused | 1.77e5 (Table 6 PDF p7), 2.246 (PDF p7) | 1 | `units::xf` (M5), test `xf_guards` | Exact | `same_dataset_disclosed` | Run, pass, 2 of 2 rows. XF-2-TABLES, the transcription check of the 21 typed constants, also passes |
| XF-3 | 1 | 298 K and 948 K | Reaction enthalpy I, II, III | 206.1, -41.15, 164.9 and 224.0, -37.3, 187.5 (Table 7 PDF p8) | kJ/mol | `props::species::h_j_mol`, test `xf_table7_enthalpy` (new) | 1.0 kJ/mol at 298 K, 2.0 at 948 K | `independent` | Run: 5 pass, 1 fail. Failed: III-948K 189.8 against 187.5 kJ/mol |
| XF-4 | 2 | Long tube limit | Exit conversion against SMR-EQ | No literature value | points | `units::smr_tube` (M5), test `smr_tube_tends_to_equilibrium` | 0.5 point | `independent` | Run, pass |

XF-1 passes only at the precision of the plan, which prints its limits to 3 decimals, so each ratio is judged after rounding to 3 decimals. Before rounding, two ratios lie outside the limits: k3 at 0.985568 and KH2 at 1.004181. Each row's note gives the unrounded ratio. XF-2-TABLES checks the 21 typed constants behind XF-1 against the fixture.

XF-3 misses at 948 K for reaction III, 189.8 against 187.5 kJ/mol with a 2.0 kJ/mol limit. The row stays failed.

Table 1 PDF p2 sets the range of the data: 773 to 848 K and 3 to 15 bar for reforming. The plant runs about 300 K and a factor of two in pressure beyond it, so the engine must flag every SMR-TUBE result as extrapolated. Activity fitted on JAERI A-2 is `calibration`, and that dataset sits outside this list (section 19).

## 7. Water-gas shift studies

WGS-HTS uses Chinchen pseudo first order kinetics, so Chinchen Table 1 is calibration data. The TEV-961 rows T961-5 and T961-6 are the plant regime tests.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| CH-1 | 2 | 31 bar, 310 to 450 C, feed 80 H2, 10 CO, 10 CO2, steam ratio 1.0 | k, 15-5 and 15-4 pellets | 0.017 to 0.918 and 0.017 to 0.565 (Table 1 PDF p6) | 1/s | `units::wgs` (M4), test `chinchen_k_fit` (new) | AARD 15%, from the 0.090 to 0.114 spread at 350 C | `calibration` | Run, pass |
| CH-2 | 2 | 16 and 21 bar, T up to 350 C, where pore diffusion is absent | k against the 31 bar fit | Table 1 PDF p6 | 1/s | `units::wgs` (M4), test `chinchen_k_pressure` (new) | AARD 25% | `same_dataset_disclosed` | Run, pass |
| CH-3 | 2 | Microreactor, 31.6 bar, GHSV 5000 1/h STP, dry gas to steam 1, crushed 15-5, feed PDF p5 | CO conversion | 9.2 to 19.0 at 310 C, 18.5 to 37.2 at 330 C, 33.0 to 55.8 at 350 C (Table 2 PDF p7) | % | `units::wgs` (M4), test `chinchen_microreactor` (new) | 10 points, close to the 11.4 point half range of all rows at 350 C | `independent` | Not run: no route was built; `wgs::hts` sizes adiabatic beds and the microreactor bed of PDF p5 was not set up |
| KR-1 | 2 | Ambient pressure, OP1 to OP4, inlet as Table 4 PDF p10, GHSV and steam ratio as Table 2 PDF p7 | Outlet CO, dry | 3.4, 4.4, 5.1, 3.4 (Table 4 PDF p10) | vol% | `units::wgs` (M4), test `kraussler_outlet_co` (new) | 2.0 vol%. Kraussler's own worst model error is 1.5 vol% | `independent` | Not run: no route was built; the catalyst differs from Chinchen's and the gas carries H2S and tar |

KR-1 is a weak test. The catalyst differs and the gas carries H2S and tar. Hydrogen was closed by balance and never measured, and OP1 was the point Kraussler used to improve the model. The reactor ran at ambient pressure against about 30 bar in the plant. Kraussler's power law stays unported, because its CO exponent and pressure base do not extrapolate to plant pressure. The KR-1 rows carry the four printed outlet values, and the trim removed the rest of the file.

Chinchen Table 2 row 1 prints one value at 400 C under the 330 C column, marked by footnote d. The fixture at `eaf9327` carried it at 400 C. CH-3 does not run, so the trim removed Table 2, and Tables 3 and 5 with it. See section 18 for why Tables 3 and 5 do not run.

## 8. Zhang, Que and Chen electrolyte NRTL

No engine comparison can run from this source. See Z-1 in section 18. The fixture at `eaf9327` held Table 8 and Table 10 parameters with their standard deviations, plus the data inventory of Tables 7, 9 and 11, which list ranges and deviations with no data points. No benchmark reads them, so the trim removed the file.

The inventory gives context for the VLE tolerances. Zhang's own model reaches AARD 33.5% on Jou and 35.5% on Hilliard (Table 9 PDF p6). The 25% limit on the Aronu rows sits below both figures, though the datasets differ.

## 9. Aronu VLE

The engine implements this model. Its parameters were regressed on these same points, so no row here is independent validation. Tables 3 and 5 come from the corrigendum, which replaces the originals.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| AR-1 | 1 | 15 to 60 mass%, 60 to 120 C, at printed Ptot and loading, SRK plus Poynting | pCO2 model column, 104 points | Tables 3, 5 (PDF p16, p17), Tables 4, 6 (PDF p5, p6) | kPa | `mea::validation::run_model_points`, test `aronu_104_model_columns` | Each point 1.5%, AARD 0.30% | `same_dataset_disclosed` | Run, pass, 105 of 105 rows. Includes the 104 point rows ARONU_104_001 to 104; AR-1-FIXTURE also passes |
| AR-2 | 1 | 30 mass%, 40 to 80 C | Measured pCO2, 36 points | Table 4 PDF p5 | kPa | `mea::pstar::pstar_kpa`, test `aronu_table4_data` | AARD 25% | `same_dataset_disclosed` | Run, pass |
| AR-3 | 1 | All four concentrations, 40 to 80 C | Measured pCO2, 138 points including AR-2 | Tables 3 to 6 | kPa | `mea::pstar::pstar_kpa`, test `aronu_all_measured` (new) | AARD 25%. Aronu reports 24.3% (PDF p1) | `same_dataset_disclosed` | Run, pass |
| AR-4 | 1 | All four concentrations, 60 to 120 C | Measured Ptot, 104 points | Tables 3 to 6 | kPa | `mea::pstar::pstar_kpa` plus `water_partial_pressure_pa`, test `aronu_ptot` (new) | AARD 12%. Aronu reports 11.7% (PDF p1) | `same_dataset_disclosed` | Run, pass |

Table 3 prints the same model value, 487.652 kPa, and the same loading, 0.784, on two rows at 60 C, total pressures 505.3 and 671.9 kPa (corrigendum PDF p16). One of them is a repeat, and the page does not say which. The set keeps the 671.9 kPa row, which sits in loading order, and flags the 505.3 kPa row `dup_printed=1` and excludes it, which gives 104 points from 105. The port agrees better with the excluded row: SRK and Poynting at 505.3 kPa give 487.691 kPa, 0.008% from the printed value, and at 671.9 kPa they give 490.956 kPa, 0.677% from it (test `repeated_table3_row`). The kept row may therefore carry the repeated value. Both readings lie inside the 1.5% point limit, and the choice moves the AR-1 AARD by under 0.007 points. The row AR-1-FIXTURE names the excluded row, and AR-4 leaves it out as well. A pass on AR-1 proves the port. It says nothing about accuracy at plant conditions, where the model is reliable to about 10 bar pCO2.

## 10. Hartono physical properties

Density and viscosity in the engine use Weiland, so Hartono is an out of sample benchmark. Only rows without a superscript reference are this paper's measurements. The text layer glues superscripts onto literature rows, turning 1010.9 with reference 2 into 1010.92, so those rows were excluded.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| HA-1 | 1 | 30 mass%, 20 to 80 C, loading 0 to 0.50, 48 points | Density | 1012.68 to 979.27 unloaded, Table 3 PDF p3 | kg/m3 | `mea::density::density_kg_m3`, test `density_weiland` | AARD 1% | `independent` | Run, pass |
| HA-2 | 1 | 30 mass%, same grid, two repeats, 96 points | Viscosity | 2.874 to 0.740 unloaded, Table 7 PDF p5 | mPa s | `mea::viscosity::viscosity_pa_s`, test `viscosity_weiland` | AARD 3% | `independent` | Run, pass |
| HA-3 | 1 | 30 mass%, 25.2 to 87.1 C, loading 0 to 0.50, 22 points | N2O Henry constant against Khan Eq 8 with the N2O analogy | 4.67 to 25.32, Table 11 PDF p6 | kPa m3/mol | `mea::kinetics::henry_khan_pa_m3_mol`, test `khan_henry_against_hartono_table11` | AARD 30%, set from the 24.3% VLE scatter. The existing test only guards units, ratio 0.2 to 2.0 | `independent` | Run, fail. Failed: AARD 40.41% against the 30% limit |

The M3 tests hold 24 of the 48 density points, 24 viscosity points and 20 of the 22 Henry points. The harness runs the full sets above at the same limits. The 6.2 mass% rows, extracted at `eaf9327`, were removed by the trim: no gate used them, because that concentration lies far from the 30 wt% basis.

HA-3 failed, as section 3 expected. Khan Eq 8 has no loading term, and the measured Henry constant rises with loading.

Hartono's own correlation cannot be rebuilt. See H-1 in section 18.

## 11. Versteeg and van Swaaij

The engine uses the Hikita correlation through Khan Eq 2. Hikita's rows in Table 10 are the same data, so only the other laboratories give an outside view. `mea::kinetics::k2_hikita` returns m3/kmol/s and Table 10 prints m3/mol/s, so the harness divides by 1000.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| VER-1 | 1 | MEA, 293, 298, 303 K, Hikita 1977a rows | k2 | 4.42, 5.87, 7.72 (Table 10 PDF p12) | m3/mol/s | `mea::kinetics::k2_hikita`, test `k2_hikita_rows` (new) | 0.5% | `same_dataset_disclosed` | Run, pass, 3 of 3 rows |
| VER-2 | 1 | MEA, 293, 298, 303 K, other laboratories | k2 | Penny and Ritter 3.70, 4.89, 6.48. Alvarez-Fuster 4.30. Laddha and Danckwerts 5.72. Sada 7.74 (Table 10 PDF p12) | m3/mol/s | `mea::kinetics::k2_hikita`, test `k2_other_laboratories` (new) | 20%, about two standard deviations of the printed values at each temperature | `independent` | Run: 5 pass, 1 fail. Failed: 298K-PennyRitter1983 5.868 against 4.89 m3/mol/s |
| VER-3 | 1 | MEA, same temperatures and pKa | k2 against Eq 20, ln k2 = pKa + 16.26 - 7188/T | Eq 20 PDF p12 | m3/mol/s | `mea::kinetics::k2_hikita`, test `k2_bronsted` (new) | 20% | `independent` | Run, pass, 3 of 3 rows |

## 12. Khan rate-based model

The engine's ABS uses the Khan closure as a set. These runs validated that closure in the original paper, so agreement here is `same_dataset_disclosed` even though nothing is fitted in this project.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| KH-1 | 2 | Runs T22, T13, T15, T19 at 103.15 kPa, Berl saddles, 6.6 m, 0.1 m | CO2 removal | 100.0 each (Table 1 PDF p5) | % | `units::absorber` (M6), test `absorber_khan_table1` | 2 points | `same_dataset_disclosed` | Not run: Khan prints no specific area or critical surface tension for these packings and no local source holds them, so Onda Eqs 5a to 5c cannot run |
| KH-2 | 2 | Run I06, 724.47 kPa, Pall rings, 14.1 m, 1.9 m | CO2 removal | 99.92 (Table 1 PDF p5) | % | `units::absorber` (M6), test `absorber_khan_table1` | 2 points | `same_dataset_disclosed` | Not run: Khan prints no specific area or critical surface tension for these packings and no local source holds them, so Onda Eqs 5a to 5c cannot run |
| KH-3 | 2 | Run T22, 7 heights | Gas CO2 profile | 19.12 to 0.00 (Fig 3a PDF p5, vector read, band 0.10) | mol% | `units::absorber` (M6), test `absorber_khan_t22_profile` (new) | RMS 2.0 mol%. The figure itself shows about this error for Khan's model | `same_dataset_disclosed` | Not run: Khan prints no specific area or critical surface tension for these packings and no local source holds them, so Onda Eqs 5a to 5c cannot run |
| KH-4 | 2 | Run T22, 7 heights | Liquid MEA profile | 343 to 3006 (Fig 3b PDF p5, vector read, band 15) | mol/m3 | `units::absorber` (M6), test `absorber_khan_t22_profile` (new) | RMS 300 mol/m3 | `same_dataset_disclosed` | Not run: Khan prints no specific area or critical surface tension for these packings and no local source holds them, so Onda Eqs 5a to 5c cannot run |

Removal of 100.0 printed for T22 saturates, so KH-1 discriminates little. KH-3 and KH-4 carry the real content. The Aroonwilas run prints no removal value. Its conditions and the Fig 3 profile points were extracted at `eaf9327`, and the trim removed them because no Khan row runs. `khan_2011.csv` keeps the five printed removals that KH-1 and KH-2 carry.

## 13. Isa high-pressure absorption

All Isa runs use 20 wt% pure aqueous MEA once through, on CO2 and CH4 at 1:1. Regeneration was unavailable on the pilot (PDF p7). Table 9 and Fig 7 captions call P a CO2 partial pressure, while Table 3 heads the same numbers "Pressure". The fixture at `eaf9327` kept `P_bar` as printed, and a harness route must state which reading it uses.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| IS-1 | 2 | Pressures up to 50 bar, inlet as Table 3 PDF p4, 1.505 m, 0.145 m, Flexipac 1Y | Y CO2 at column top | 0.05 experiment, 0.055 Isa model (Conclusions PDF p13) | mol CO2/mol CH4 | `units::absorber` (M6), test `absorber_isa_ytop` (new) | 0.05 absolute | `independent` | Not run: no route was built; the absorber would need the Table 3 inlet state at each pressure |

IS-1 is the only Isa benchmark. Table 9 fails its own mass balance and does not run (I-1 in section 18). At `eaf9327` the fixture also held Tables 3, 7, 8, 10 and 12. Tables 7 to 12 are Isa's model outputs, and the engine does not use the index f. IS-1 does not run, so the trim kept only its printed target.

## 14. Abdul Halim high-pressure experiments

Bench column 0.046 m by 2.04 m with Sulzer metal gauze, at 5.0 MPa on 20% CO2 in natural gas. The paper prints no lean loading anywhere, so the harness must state the value it assumes and carry that assumption in the row's explanation.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| AH-1 | 2 | Eight conditions, 1.0 to 4.0 kmol/m3 MEA, 6 heights | Column temperature profile | 30.5 to 46.2 (Table 3 PDF p5) | C | `units::absorber` (M6), test `absorber_halim_profiles` (new) | RMS 5 K. About twice the 2.8 C average error of a pilot model in the design evidence, since wall loss here is unknown | `independent` | Not run: the Sulzer gauze geometry is not in the packing set and the paper prints no lean loading |
| AH-2 | 2 | 1.0 kmol/m3, 19.72 kmol/m2/h, 27 C, liquid load 1.81 to 4.51 m3/m2/h | Rise in CO2 removal | 52.59 (text PDF p4) | % | `units::absorber` (M6), test `absorber_halim_liquid_trend` (new) | Sign, then 20 points | `independent` | Not run: the Sulzer gauze geometry is not in the packing set and the paper prints no lean loading |

Removal and KGav values appear in raster figures and were not read. The text gives KGav in mol/m2/h/kPa while Fig 6 labels mol/m3.h.kPa, so those two rows were stored as printed at `eaf9327`. No gate used them, and the trim removed them with the Table 3 profiles, since AH-1 and AH-2 do not run. The fixture keeps the AH-2 target. The sentence that KGav "has reduced to about 17%" reads as a reduction by 17% against Fig 6, and it also runs no gate.

## 15. Salvinder high-pressure pilot

This is the closest local experiment to the plant in partial pressure: 5.25 bar CO2 by calculation at 10.5 bar, against 4.74 bar at the INL absorber.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| SA-1 | 2 | 10.5 bar, 26 C, 50 L/min gas, 1.04 L/min lean, 20 wt% (Table 1 PDF p4, PDF p2) | CO2 removal | 85.0 pilot. Aspen comparators 84.3 rate based, 55.0 equilibrium (Table 2 PDF p5) | % | `units::absorber` (M6), test `absorber_salvinder_85pct` | 5 points | `independent` | Run, fail. Failed: 72.58 against 85% |

Table 1 prints compositions as "0.5 mol" and "0.8 mol". The fixture keeps the unit as printed and the harness reads them as fractions. This is one data point.

SA-1 fails: the absorber removes 72.6% against the 85.0% measured. The row note states the gas basis assumed (flow read at 0 C and 1 atm) and the gas film left out for want of a local CH4 diffusivity. SA-1 is one of the rows that fail the column gate of terms T07 and T08 (section 19).

## 16. Vinjarapu pilot regeneration

Waste-to-energy flue gas in a 0.217 m column. Inlet CO2 runs 10.5 to 18.5 vol%, with 76 of 85 states between 11 and 14. Solvent is nominally 30 wt% and measured lean MEA runs 24.5 to 31.3 wt%, so each state must run at its own printed concentration. Stream meanings come from the flow diagram, Fig D.43 PDF p28: G3 absorber inlet gas, A1 rich solvent, A3 lean solvent, S1 stripper bottom.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| VI-1 | 2 | 16 states with capture 88 to 92%, inputs from Tables D.10, D.12, D.13, D.17 | SRD | 3.46 to 4.82 (Table D.19 PDF p38) | GJ/t CO2 | `units::stripper` (M6), test `stripper_vinjarapu` | Median absolute error 10%, matching the plan's DICE and Kwak limit | `independent` | Run, pass |
| VI-2 | 2 | All 85 states | SRD | Table D.19 PDF p38 | GJ/t CO2 | `units::stripper` (M6), test `stripper_vinjarapu` | Reported, median absolute error 15% | `independent` | Run, pass. The 85 rows VI-SRD-SS* print each state's SRD with no limit |
| VI-3 | 2 | Table 5 states SS1, SS2, SS3 | Lean loading | 0.16, 0.23, 0.25 (Table 5 PDF p14) | mol/mol | `units::stripper` (M6), test `stripper_vinjarapu` | 0.03, the printed uncertainty (split flow Table 3 PDF p6) | `independent` | Run: 1 pass, 2 fail. Failed: SS2 0.1596 against 0.23 mol/mol; SS3 0.1968 against 0.25 mol/mol |

SRD in D.19 appears to be net of pilot heat loss. RD times 3.6 over FTC2 runs a median 1.17 times SRD, which fits a loss of 2.8 to 2.9 kW (split flow Table 2 PDF p5). The harness must apply the same basis, or the comparison is biased by about 15%.

The stripper model has no heat loss. VI-1 and VI-2 set the printed lean loading and compare the model SRD with D.19, so both sides sit on the net basis. VI-3 runs the stripper at the printed reboiler duty RD, which still includes the pilot loss, so VI-3 carries the gross basis this section warns against. More heat reaches the model solvent than reached the pilot solvent, and the three model lean loadings all sit below the printed values, as that bias predicts. VI-3-SS2 and VI-3-SS3 stay failed. The route was left as built, because a change after the result would act on a known outcome; a net-duty route needs the D.19 SRD times the captured CO2 flow, which the trimmed fixture no longer holds.

Measurement uncertainty is carried in `read_uncertainty` for D table rows. Figures in this paper are raster and were not read.

## 17. Brigman Mongstad and the IAEA HTTR report

### Brigman

CHP flue gas near 3.5 vol% CO2 at about 1 bar. Figure rows come from vector markers with a band of 0.02 MJ/kg and 0.002 in loading. The legend assigns diamonds to 25 C, triangles to anti-foam and a star to the base case. The star is drawn from separate lines and was not extracted, because the text prints it as 4.1.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| BR-1 | 2 | 30 wt%, 24 m, 1.9 bara, 47000 Sm3/h, 25 C, capture 85% | Minimum SRD and its lean loading | 4.0 at 0.25 (PDF p5), curve Fig 3 PDF p6 | MJ/kg CO2 | `units::stripper` (M6), test `stripper_brigman_30wt` (new) | 10% on SRD, 0.05 on loading | `independent` | Not run: needs the absorber and stripper coupled on flue gas near 1 bar with a lean loading sweep; no route was built |
| BR-2 | 2 | 40 wt%, 59000 Sm3/h | Minimum SRD | 3.4 at 0.2 to 0.25 (PDF p5), curve Fig 4 PDF p6 | MJ/kg CO2 | `units::stripper` (M6), test `stripper_brigman_40wt` (new) | 10% | `independent` | Not run: needs the absorber and stripper coupled on flue gas near 1 bar with a lean loading sweep; no route was built |
| BR-3 | 3 | 30 wt%, 18 m against 24 m | SRD, rich loading | 4.5 against 4.0, 0.44 against 0.48 (PDF p7) | MJ/kg CO2, mol/mol | Coupled absorber and stripper (M7), test `brigman_packing` (new) | Sign, then 0.03 on loading | `independent` | Not run: needs the absorber and stripper coupled on flue gas near 1 bar with a lean loading sweep; no route was built |
| BR-4 | 3 | Flue gas 50 C against 25 C | SRD | 5.0 against 4.0 (PDF p9) | MJ/kg CO2 | Coupled absorber and stripper (M7), test `brigman_flue_50c` (new) | Sign only. Brigman attributes the rise mainly to a top pinch, which the design evidence finds absent at 26.8 bar | `independent` | Not run: needs the absorber and stripper coupled on flue gas near 1 bar with a lean loading sweep; no route was built |

### IAEA-TECDOC-1236

This report is a design study and prints no measured data from a coupled HTTR steam reformer. Only the EVA summary in IA-5 reports a test, and it does so in words. Comma decimals in the report are stored as points.

| ID | Level | Conditions | Quantity | Literature value, locator | Units | Engine call | Tolerance | Data use | Status |
|---|---|---|---|---|---|---|---|---|---|
| IA-1 | 1 | Feed CH4 19.77, C2H6 0.17, CO 0.02, H2O 80.05 (PDF p12). 800 C at 41.9 and 20 bar, 860 C at 41.9 bar | Conversion | 66.1, 81.3, 79.8 (Tables 2.1, 2.2 PDF p13) | % | `props::equilibrium::reform`, test `te1236_equilibrium_three_points` | 0.5 point | `independent` | Run, pass, 3 of 3 rows |
| IA-2 | 1 | Same feed, 800, 825, 850, 860 C at 41.9 bar | Wet product composition, conversion at 825, 850 C | Table 2.2 PDF p13. 72.0, 77.6 | vol%, % | `props::equilibrium::reform`, test `te1236_table22` (new) | 0.5 mol%, 0.5 point | `independent` | Run, pass, 22 of 22 rows |
| IA-3 | 1 | 20 bar, 800 C | Wet product composition | CH4 2.83, H2 43.06, CO 6.10, CO2 6.23, H2O 41.77 (Table 2.1 PDF p13) | vol% | `props::equilibrium::reform`, test `te1236_table22` (new) | 0.5 mol% | `independent` | Run, pass, 5 of 5 rows |
| IA-4 | 1 | HTTR reformer, 830 C, 4.5 MPa, S/C 3.5 (PDF p52, p53) | Conversion | About 68 (PDF p53) | % | `props::equilibrium::reform`, test `httr_sr_design` (new) | 5 points, as printed with "about" | `independent` | Run, pass |
| IA-5 | 2 | EVA, helium heated tube, exit | Conversion below equilibrium | 1 to 2.5 (PDF p71) | points | `units::smr_tube` (M5), test `smr_tube_tends_to_equilibrium` | Plausibility, 2.5 points | `independent` | Run, pass |

Equilibrium temperature in Tables 2.1 and 2.2 is printed about 10 K below the product exit temperature (PDF p12). The test uses the equilibrium temperature. The fixture keeps the HTTR text values that IA-4 reads. Table 3-4 values were extracted at `eaf9327`, and the trim removed them because no benchmark reads them. The IHX benchmark `ihx_benchmarks_10_3` depends on Tochio Table 2 and TECDOC-1382, which are not in this list (section 19).

## 18. Benchmarks that cannot run from local sources

| ID | Benchmark wanted | Why it cannot run | What runs in its place | Status |
|---|---|---|---|---|
| C-1 | TEV-961 Case 6 streams | The local PDF ends at A-59, the end of Case 5. The Case 6 calculator pages and stream appendices are absent | T961-1 and T961-2 from Table 2, Case 5 for all unit tests | Not run, reason in this row |
| W-1 | LTS kinetics | No Cu/Zn kinetic source is in the library. WGS-LTS is a 10 K approach assumption | T961-6 tests the outlet only | Not run, reason in this row |
| W-2 | Chinchen effectiveness, Tables 3 and 5 | The engine has no pore diffusion model and none is planned. Chinchen Part III is not local | CH-1 to CH-3 on empirical k | Not run, reason in this row |
| Z-1 | Zhang e-NRTL reproduction | Parameters depend on the Aspen Databank and the activity equations are unprinted. The paper gives no numerical model output to compare with, and Tables 7, 9 and 11 hold ranges and deviations with no points | AR rows, with Zhang as context in section 8 | Not run, reason in this row |
| Z-2 | High pressure VLE from Zhang Figs 9 and 10 | Those figures are raster, so the points cannot be read reliably, and Jou's primary paper is not local | Nothing at 30 wt% above about 1 MPa pCO2. Wong and Dugas are outside this list | Not run, reason in this row |
| X-1 | Xu-Froment rates against their experiments | The papers show rates only in figures, and K1(T), K2(T), K3(T) appear in neither paper | XF-1 to XF-3, which test transcription and enthalpy | Not run, reason in this row |
| H-1 | Hartono correlations | Parameters k1 to k4 are in Hartono 2013, which is not local. Pure component properties are unprinted, and supplement Tables S1 and S2 are not local | Weiland against Hartono data, HA-1 and HA-2 | Not run, reason in this row |
| H-2 | Properties above loading 0.50 | Hartono stops at 0.50. No local source reaches the 0.50 to 0.75 region | None. The plant caps rich loading at 0.50 | Not run, reason in this row |
| H-3 | H2, CH4, CO solubility in MEA | No local source covers them | None. FLASH treats them as insoluble and states so | Not run, reason in this row |
| VER-X | Versteeg Henry constants | Those values are in J. Chem. Eng. Data 33 (1988) 29-34, not local. The local paper is the kinetics paper | Second hand values through Aronu and Dugas | Not run, reason in this row |
| I-1 | Isa rich loading | Table 9 lists 0.056 to 0.195, while Table 3 flows and about 95% removal imply about 0.31. The table fails its own mass balance | IS-1 on gas side Y | Not run, reason in this row |
| I-2 | Isa Fig 10 and Table 8 areas | Fig 10 is raster. Table 8 prints no operating condition for its single area value | IS-1 | Not run, reason in this row |
| A-1 | Abdul Halim removal and KGav | Figures 3 to 6 are raster | AH-1, AH-2 | Not run, reason in this row |
| S-1 | SRD at high pressure | Isa, Abdul Halim and Salvinder all run once through. None measured regeneration | VI and BR rows, all on flue gas, disclosed as a regime gap | Not run, reason in this row |
| S-2 | 30 wt% MEA at high pressure | Isa and Salvinder use 20 wt%, and Abdul Halim stops at 4 kmol/m3 | 20 wt% rows, disclosed as a concentration gap | Not run, reason in this row |
| S-3 | MEA on H2-rich syngas | No local absorber study uses H2-rich gas | None. The absorber carries `ReducedOrder` status at plant conditions | Not run, reason in this row |
| V-1 | Split flow and interheating | Both need a stripper configuration the plant flowsheet does not contain. The two fixtures were removed by the trim and remain at `eaf9327` | None | Not run, reason in this row |
| HT-1 | HTTR heat utilisation performance | TECDOC-1236 gives design intent for the coupled reformer and prints no measured data from it | IA-1 to IA-4 as code comparisons | Not run, reason in this row |
| HT-2 | HTR-10 reformer energy balance | Table 5-3 PDF p88 does not close: 2.325 kg/s across 890 to 600 C gives 3.50 MW against 2.398 MW printed. Process pressure is also unprinted | None. The fixture keeps the row, and HT-2 reports the gap with no gate | Run, reported with no limit: 3.503 MW from the printed flow and temperatures against 2.398 MW printed |

## 19. Benchmarks in the plan outside this list

The architecture test list also names JAERI-Tech 2005-031 (A-2 to A-5), Yin, Tochio, TECDOC-1382, DICE, Kwak, Morgan, Sonderby, Tan and IEAGHG Case 1A. These lie outside the sources the brief names, so this file froze no tolerance for them when it was added. Their limits are the targets of the engine plan, section 12, and M5 and M6 added fixtures for the column sources after `eaf9327`: `tan_2015.csv`, `sonderby_2013.csv`, `dice_2020.csv`, `kwak_2012.csv` and `morgan_2018.csv`, each holding only the values its benchmark reads, with source and page. The JAERI values are typed constants in `nush2_units::jaeri`, each with its sheet and PDF page, and have no fixture. Dugas Table 4.4 sits in `engine/mea/tests/data/`.

The export also carries the property rows of M2 and the plant and flowsheet rows, which check the code against the screening script and the Level 3 flowsheet. All of them are listed here with their status.

| Group | Rows | Source and fixture | Limit | Data use | Status |
|---|---|---|---|---|---|
| PROPS (M2) | 19 | NIST-JANAF, the IAPWS-IF97 release vectors, Petersen 1970, Zhang et al. 2021 Table 1, and the TECDOC-1236 and TEV-953 A-40 equilibrium routes. Fixtures in `engine/vv/data/props/` | The M2 test limits: Shomate Cp 0.1% and S 0.01% relative, H 0.01 kJ/mol; K against JANAF 1%; Zhang against NIST 6%; latent heat 0.057 kJ/mol; helium 7e-4 relative; IF97 1e-8 relative; equilibrium 0.5 point or mol% | `independent`, or `same_dataset_disclosed` for the code verification rows | Run: 15 pass, 1 fail, 3 reported with no limit. Failed: PROPS-SHOMATE-CP, CO Cp at 400 K, 29.30 against 29.342 J/mol/K (0.134% against 0.1%); the printed NIST polynomial itself gives 29.3027, so the miss is the fit's own swing about JANAF. Reported: PROPS-K2-ZHANG-LTS, ratio 0.832 at 486 K, outside the 6% domain, and the TECDOC-1236 rows at 825 and 850 C |
| JA (M5) | 58 | JAERI-Tech 2005-031 sheets A-2 to A-5, typed in `nush2_units::jaeri` | Conversion 0.03; station temperatures, duties and pressure drop reported | A-2 `calibration`, A-3 to A-5 `independent` | Run: 1 pass, 3 fail, 54 reported with no limit. Failed: JA-A-2-X 0.511 against 0.555; JA-A-3-X 0.516 against 0.567; JA-A-5-X 0.506 against 0.548. A-4 passes. The A-2 activity sits at the 2.246 search limit and is the one disclosed tuned value (decision S3); the holdout misses are structural and stay recorded |
| TAN (M6) | 4 | Tan et al. 2015, `tan_2015.csv` | Plan trend on 76, 95 and 100% removal | `independent` | Run: 1 pass, 3 reported with no limit. TAN-TREND passes on the rising trend only. The three removals read 19.8, 26.0 and 47.9% against 76, 95 and 100% |
| SO (M6) | 82 | Sonderby et al. 2013, `sonderby_2013.csv` | Profiles reported, as the plan sets | `independent` | Run, reported with no limit, 82 rows |
| ABS-CASEB (M6) | 5 | Case B absorber feed from the M4 chain; no literature value | 90% capture at rich loading 0.50 or less | `same_dataset_disclosed` | Run: 1 pass (ABS-CASEB-SPEC), 4 reported with no limit (height, diameter, liquid load, rich loading) |
| DICE (M6) | 5 | Nexant and Bechtel 2020 DICE pre-FEED performance report, `dice_2020.csv` | SRD 10% | `independent` | Run: 1 fail, 4 reported with no limit. Failed: DICE-SRD 5.293 against 3.638 GJ/t |
| KW (M6) | 4 | Kwak et al. 2012, `kwak_2012.csv` | SRD 10% | `independent` | Run: 1 fail, 3 reported with no limit. Failed: KW-SRD 4.977 against 3.92 GJ/t; the pilot duty includes its heat loss and the model has none |
| MO (M6) | 6 | Morgan et al. 2018 supporting information, `morgan_2018.csv` | Stripper outlet temperature 1.4 C | `independent` | Run: 2 pass, 1 fail, 3 reported with no limit. Failed: MO-K16-TOUT 121.3 against 122.94 C. K14 and K15 pass; lean loadings are reported |
| Plant (M1) | 59 | `screening.py` through the design goldens and the freeze | The limit of each check | `same_dataset_disclosed` | Run, pass, 59 of 59 rows. These verify the port and validate no physics |
| FS (M7) | 24 | Level 3 flowsheet against the screening ledger and its own closures | Carbon lines 2%; closures 1e-9 in elements and 1e-6 in energy; tear restarts 1e-5; each other row states its own | `same_dataset_disclosed` for the anchor comparisons, `independent` for closures and tears, `calibration` for the H2 return closure | Run: 19 pass, 5 fail. Failed: FS_CARBON_VS_ANCHOR_TUBE 1497.3 against 1574.1 kmol C/h (4.9% low); TAIL_GAS, RECYCLED and BURNED each 19.7% low; FS_GATE_M7. The flowsheet sends 0.826 of the shifted carbon to CO2 against the anchored 0.775, so T04, T09 and T13 stay anchored (decision S11) |
| Tochio, TECDOC-1382 | 0 | No fixture | Plan section 12 | | No row: the IHX and reactor comparisons belong to M8, which has not run |
| Yin, IEAGHG Case 1A | 0 | No fixture | None | | No row. Yin Table 2 enters only as the inlet set of JAERI A-5, and IEAGHG Case 1A enters only as the aMDEA capital of a sensitivity |

The column gate of terms T07 and T08 takes every M6 row with a limit except the Khan rows, which cannot run for want of source data. SA-1, DICE-SRD, KW-SRD, MO-K16-TOUT, VI-3-SS2 and VI-3-SS3 fail, so the gate returns `GateFailed`, T07 and T08 stay anchored, and the SRD in Headline stays at the approved 3.8 GJ/t (decision S12). The same stripper model overstates the DICE and Kwak pilot SRD by 27 to 45%.

As exported with this revision, `results/Validation_Results.csv` holds 613 rows: 310 pass, 22 fail, 243 reported with no limit and 38 not run. The 22 failed rows are DICE-SRD, FS_CARBON_VS_ANCHOR_BURNED, FS_CARBON_VS_ANCHOR_RECYCLED, FS_CARBON_VS_ANCHOR_TAIL_GAS, FS_CARBON_VS_ANCHOR_TUBE, FS_GATE_M7, HA-3, JA-A-2-X, JA-A-3-X, JA-A-5-X, KW-SRD, MO-K16-TOUT, PROPS-SHOMATE-CP, SA-1, T953-2, T961-1-ROT825C, T961-5-GHSV-CO, T961-5-GHSV-T, VER-2-298K-PennyRitter1983, VI-3-SS2, VI-3-SS3 and XF-3-III-948K.

## 20. Anomalies found in the sources

| Source | Finding | Treatment |
|---|---|---|
| Aronu Table 3 | Rows at Ptot 505.3 and 671.9 print the same model value and loading. The page does not say which row is the repeat, and the port agrees better with the 505.3 row (section 9) | The 505.3 row is flagged and excluded, disclosed in section 9 and in the row AR-1-FIXTURE |
| Aronu Tables 3, 5 | The corrigendum replaces both, and also prints 858.45 and 998.77 where the original has more digits | Corrigendum used |
| Hartono Tables 3, 7, 11 | Text layer glues reference superscripts onto values | Literature rows excluded |
| Xu-Froment Table 5 | KH2O lower limit prints 0.0317, far from the mean of 0.4152 given an upper limit of 0.5032 | Stored as `_LL_printed` at `eaf9327`, unused, and removed by the trim |
| Xu-Froment PDF p7 | A(KH2) prints 6.13e-9 in text and 6.12e-9 in Table 6 | Table 6 used |
| TECDOC-1236 Table 3-4 | Tube outer diameter prints 3.2 mm beside a 3.5 mm wall | Left out |
| TECDOC-1236 Table 5-3 | Helium side does not close, see HT-2 | No gate |
| Isa Table 9 | Fails the mass balance, see I-1 | No gate |
| Isa Table 9, Fig 7 | P labelled as CO2 partial pressure while Table 3 heads it Pressure | Stored as printed at `eaf9327`, removed by the trim since IS-1 does not run |
| Abdul Halim PDF p4 | KGav unit differs between text and Fig 6 | No gate |
| Abdul Halim, whole paper | Lean loading is printed nowhere | Harness states its assumption |
| TEV-961 A-55 | Preheater duty labels unreadable in the scan | Left out |
| Vinjarapu Table D.19 | SRD appears net of heat loss | Basis stated in section 16 |
