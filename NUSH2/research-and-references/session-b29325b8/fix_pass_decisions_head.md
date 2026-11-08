# NUS-H2 final design decisions

Value labels: **[P]** published, **[A]** assumed, **[C]** calculated by the engine, **[U]** author decision. Every [C] value in this file is read from `results/` as exported with this revision, through `build/numbers.json`, on the Headline preset unless a line names another. The evidence state of each model is in `docs/VALIDATION_MATRIX.md`, and no module carries the author's sign-off yet (section 21 there).

Sources are cited by document, table and page: TEV-953 and TEV-961 are the INL technical evaluations of the HTGR-integrated SMR (TEV-961 Rev. 0, 2010, "Sensitivity of Hydrogen Production..."), INL/RPT-23-72972 is the INL microreactor and HTGR cost report, and IEAGHG 2017-02 is the IEAGHG study of SMR hydrogen with CO2 capture. Card numbers such as J.26 refer to `math/eq_register.csv` and the derivations.

## 1. Headline

Case B, Headline preset: term T05 runs on its gated model and every other term stays on its anchored value (section on drift below).

| Item | Value | Label |
|---|---|---|
| Basis | Case B, 100,000 Nm3/h H2, 66,925 t H2/y at 85% availability | [U], [C] |
| Reactor | 210 MWth rated, 178.5 MWth needed, 31.5 MWth spare held as margin with no revenue. The high corner of card J.27 needs 214.1 MWth, 4.1 MWth above the rating | [U], [C] |
| Process heat | 120.8 MWth at 900 C helium | [P] scaled |
| Natural gas | 23.2 MMSCFD with the PSA reject recycled across the boundary | [C] |
| Net abatement | 578,052 t/y (0.578 Mt/y) direct to atmosphere on the INL baseline of 9.21 kg/kg. Bounds 587,436 t/y on Cho's 9.35 and 563,745 t/y on IEAGHG's 8.996 kg/kg | [C] |
| Capture | 90% per pass, 90.4% of plant carbon | [A], [C] |
| Full cost, BOAK, n = 1.0, 210 MWth charged | L5 S$189.7/t at 25 y and S$165.6/t at 60 y. LAC_DCF S$190.1/t and S$166.0/t | [C] |
| Full cost, FOAK, same basis | L5 S$277.9/t and S$241.7/t. LAC_DCF S$278.4/t and S$242.2/t | [C] |
| Named step L3, BOAK | S$137.7/t at 25 y and S$117.2/t at 60 y | [C] |
| Named step L3, FOAK | S$209.9/t and S$179.5/t | [C] |
| LCOH, 25 y and 60 y | Unabated SMR S$3.618/kg and S$3.619/kg. NUS-H2 BOAK S$5.260/kg and S$5.053/kg, FOAK S$6.023/kg and S$5.711/kg | [C] |
| Target test | Abatement target of 250,000 t/y met. No computed central case reaches S$100/t on L5 or the DCF (0 of 8). The lowest L5 in the n = 1.0 family is 10OAK_P46 over 60 y with the reactor charged at its need, S$108.5/t | [C] |

The ladder prints beneath the full cost, with L3 as a named step, so the reader sees what each item adds (section 6). L5 and the DCF are two independent methods on one cost set; the largest gap between them over all 72 DCF points is 0.47% of L5 (10OAK_P46, 40 y, charged at need), 0.42% over the points charged at 210 MWth and 0.20% on the 12-point headline grid, inside the 10% test.

**Named scenario MATURE_60Y.** The author's stacked case: tenth of a kind on INL's BOAK learning route (stage 10OAK_P46, INL/RPT-23-72972 PDF p46), 60 y, n = 1.0, 210 MWth charged. L5 is S$128.2/t at the central S$15/GJ gas price and S$99.6/t at S$20/GJ (DCF S$128.5/t and S$99.9/t). The gas price that brings it to the author's mature-scenario target of S$96/t (parameter `TARGET_MATURE_COST`, a target the engine solves to) is S$20.63/GJ on L5 and S$20.68/GJ on the DCF. The base result above stays the reference everywhere else. Rows: sweep `MATURE_60Y` in `results/Sensitivity_Results.csv`.

**Co-funding.** Public funding of reactor capital that brings BOAK, n = 1.0, 210 MWth charged to the same target. Of the S$742.8M reactor capital, it takes 65.0% on L5 and 65.1% on the DCF at 25 y (S$483.0M and S$483.6M), and 56.0% and 56.2% at 60 y (S$416.0M and S$417.1M). Rows: sweep `COFUNDING`.

Case A, 130 MMSCFD, stays as the INL reference. With MEA and recycle it needs 260.7 MWth, 50.7 MWth above 210 MWth, so Case A is printed as a reference and never as a 210 MWth plant.

## 2. Decision register

| ID | Decision | Basis | Evidence |
|---|---|---|---|
| D01 | Outputs describe the product only | Author decision | |
| D02 | Work runs in the order of the build plan, starting from this record | Author decision | |
| D03 | Case B at 210 MWth is the headline and Case A the INL reference. Abatement is 0.578 Mt/y. The spare 31.5 MWth earns nothing | Author decision | Run B_REC_F; sweep HEAT_BASIS gives 6.5 MWth spare on Ahn Table 6's 143.4 MWth |
| D04 | PSA stays an unsimulated external block. Its reject returns as a stream crossing the boundary. The purge load of TEV-961 Table 2 is restored, scaled and lifted for the CO2 slip; row PUMP_COMPRESS carries it | Author decision | TEV-961 PDF p15, A-16 PDF p40, A-18 PDF p42; TEV-953 Table 2 PDF p30 (44.5 MMSCFD with no recycle) |
| D05 | Site is the proposed Western Island, labelled as the author's proposal with no official approval. New-build at SOXAL scale against a same-size unabated SMR. Central hydrogen runs in the 85% hours only. Backup firing is a sensitivity | Author decision | Air Liquide releases 2008, 2011; gov.sg National Day Rally 2026 (no nuclear mention); IAEA NP-T-4.3 PDF p23 on backup |
| D06 | Boundary follows Figure 2: CO2 compressor and ship offshore storage inside the dashed line, PSA outside | Author decision | Test `crosses_boundary_follows_figure_2` |
| D07 | Headline is direct emissions to atmosphere on INL's 9.21 kg/kg, with Cho 9.35 and IEAGHG 8.996 kg/kg as bounds. Per-pass and plant capture print together. Lifecycle is a sensitivity | Author decision | TEV-953 Table 2 PDF p30; Cho 2022 abstract; IEAGHG 2017-02 Table 2 PDF p16 |
| D08 | The cost is built by two independent methods on one cost set: the annual incremental ledger L0 to L5 and a discounted cash flow of the unabated SMR and NUS-H2 at equal H2 output. The headline is the full cost, L5 and the DCF, with the reactor charged at 210 MWth, n = 1.0, at BOAK and FOAK, at 25 y and 60 y. L3 prints beneath it as a named step | Author decision | Section 6; `results/Economic_Ledger.csv` |
| D09 | INL recipe, n = 1.0, is central. n = 0.6 and n = 0.31 print beside it at equal prominence. Learning route is stated each time: PDF p46 from BOAK, PDF p47 from FOAK | Author decision | INL/RPT-23-72972 PDF p27, p45 to p47 |
| D10 | Capture cost is bottom-up capital plus fixed O&M, with energy in the heat and power balance. T&S S$60/t central, S$23.2 low, S$112 high | Author decision | IEAGHG 2017-02 Tables 2 and 3 |
| D11 | One labelled SMR plus CCS comparator: IEAGHG Case 3, MEA on flue gas, 90.0% overall, at its own 100,000 Nm3/h | Author decision | IEAGHG 2017-02 Tables 2 to 5; section 6.5 |
| D12 | Retired | | |
| D13 | The report takes every number from `build/numbers.json` | Author decision | Section 7.2 |
| D14 | Retired | | |
| D15 | The deck is produced as a .pptx from the same numbers | Author decision | Section 7.2 |
| D16 | Deck delivery: one idea per slide with large numbers, and a closing summary | Author decision | |
| D17 | Independent build under MIT. No OUTRAM PARK code is copied. OUTRAM PARK is credited as a reference | Author decision | OUTRAM PARK is AGPL-3.0; section "Concepts drawn from OUTRAM PARK" |
| D18 | The repository holds code, derivations, results, docs and the four final files (report PDF and DOCX, deck PPTX and PDF). Builders, speaker notes and the Q&A sheet stay local. The author is the only person named | Author decision | `.gitignore` |
| D19 | Third-party papers and slides leave the earlier repository by a normal commit. GitHub stays minimal | Author decision | |
| D20 | Tectonic builds the derivations PDF | Author decision | `math/` |
| D21 | TEV-961 is INL's "Sensitivity of Hydrogen Production..." report | Author decision | TEV-961 title page |
| D22 | Title: Nuclear-Upgraded Steam Methane Reforming with Carbon Capture for Low-Carbon Hydrogen in Singapore | Author decision | |
| D23 | Reactor is a generic 925 C HTGR. Xe-100 is a labelled analogue only | Author decision | X-energy deck p13, p15, p16; TEV-961 PDF p14 |
| D24 | Sizing walks from 206.0 MWth at 39.7%, counting circulators, losses and the reboiler gap | Author decision | Section 5 |
| D25 | Solvent is 30 wt% MEA, with Selexol and aMDEA sensitivities | Author decision | IEAGHG 2022-07 PDF p44, where the literature is mixed |
| D26 | SRD 3.8 GJ/t, range 3.4 to 4.2, an assumption with no syngas measurement behind it | Author decision | `docs/VALIDATION_MATRIX.md` section 16 and decision S12 |
| D27 | Reboiler heat order: freed Selexol steam, export steam, syngas above 130 C, then reactor steam | Author decision | TEV-961 A-57 PDF p81; `REB_FROM_` rows of `results/Integrated_Energy_Balance.csv` |
| D28 | Absorber 40 C and 26.8 bar, rich loading capped at 0.50 | Author decision | TEV-961 A-53 PDF p77 |
| D29 | Reformer 871 C disclosed as INL's equilibrium assumption, with 825 and 800 C run through the flowsheet | Author decision | TEV-961 PDF p14; decision S8, sweep REFORMER_T |
| D30 | Solvent thermodynamics use Aronu extended UNIQUAC with the K_MEA shift | Author decision | Aronu et al. 2011; rows ARONU_104_001 to ARONU_104_104 |
| D31 | A Reference Frame opens the derivations and sits on a backup slide | Author decision | `math/sections/00_reference_frame.tex` |
| D32 | Power conversion 39.7% central, 47.5% a labelled sensitivity | Evidence | TEV-961 Table 2 note 3; INL/EXT-11-21537 PDF p19, p20 |
| D33 | HTS plus LTS as in INL. Xu-Froment Table 6 kinetics with K(T) from Zhang Table 1. 44.5 MMSCFD is the no-recycle reference | Evidence | Xu and Froment 1989 Part I Table 6; Zhang et al. 2021 Table 1; TEV-953 Table 2 |

## 3. Corrections that override earlier notes

| Earlier claim | Correction | Source |
|---|---|---|
| Learning from BOAK is an error | Both routes are published. US$1,539/kWt is the p46 route, US$2,462/kWt the p47 route | INL/RPT-23-72972 PDF p46, p47 |
| n = 0.6 is central | Now a sensitivity. n = 0.31 is the ratio of two single-unit FOAK estimates, so it carries that label | INL/RPT-23-72972 PDF p25, p27, p39, p45 |
| Amine spec overstates carbon by 33% | With recycle, captured CO2 at Case A is about 1,871 st/d, which the engine's 526,544 t/y at Case A MEA reproduces | Run A_REC_F, `RES_CAPTURED_T_Y` |
| 230 MWth closes Case A | Case A with MEA and recycle needs 260.7 MWth | `RES_REACTOR_NEEDED_MWTH`, Case A MEA |
| 85% has no HTGR support | The library holds assumed 90 to 92%, with no measured HTGR availability | IAEA NP-T-4.3 PDF p38; INL/RPT-23-72972 PDF p30 |
| 1.6x premium has no source | INL medium FOAK premium. Only US$5,250/kWe has no source | INL/RPT-23-72972 Table 1, Table 11 |
| Table A10 gas-rate label | Cite the row as 700 C helium, TEV-961 725 C ROT, equal to TEV-953 750 C ROT | TEV-961 PDF p14 note b |
| Syngas cooling comes from a process simulator file | None exists locally. The TEV-961 Case 5 cooler duties are used, and the M4 chain reproduces them (rows T961-7) | TEV-961 Case 5 sheets |

## 4. Design basis

Engine values on the Headline preset. Case B is Case A extensive items x 0.6833, with the reactor resized. "A INL" is TEV-961 Case 6 as published with Selexol (run A_INL_SEL). "A MEA" is the same plant with 30 wt% MEA and recycle kept (run A_REC_F).

### 4.1 Production

| Parameter | Case B | A INL | A MEA | Unit | Label, source |
|---|---|---|---|---|---|
| Purified H2 after external PSA | 100,000 | 130 MMSCFD | 130 MMSCFD | Nm3/h | [U]; [P] TEV-961 Table 2 |
| H2 mass rate | 8,988 | 13,154 | 13,154 | kg/h | [C]; [P] 29,000 lb/h, TEV-953 PDF p16 |
| Availability | 85 | 85 | 85 | % | [A] |
| Operating hours, days | 7,446, 310.25 | same | same | h/y, d/y | [C] |
| Annual H2 | 66,925 | 97,946 | 97,946 | t/y | [C] |
| PSA H2 recovery | 88 | 88 | 88 | % | [P] external, unsimulated, TEV-961 A-53 |
| H2 at PSA inlet | 10,214 | 14,948 | 14,948 | kg/h | [C] |
| H2 in PSA reject | 1,226 | 1,794 | 1,794 | kg/h | [C] |
| Raw syngas, reformer outlet | 108.0 | | | t/h | [C] Level 3 run FS_B_REC_F, stream S05 |
| H2-rich gas leaving capture | 19.6 | | | t/h | [C] FS_B_REC_F, stream S12 |
| Fresh natural gas | 23.2 | 34.0 | 34.0 | MMSCFD | [P] x s |
| Unabated SMR gas | 35.9 | 52.5 | 52.5 | MMSCFD | [P] TEV-953 Table 2 |
| No-recycle reference gas | 30.4 | 44.5 | 44.5 | MMSCFD | [P] TEV-953 Table 2 |
| Natural gas HHV | 1,044 | same | same | Btu/scf | [P] TEV-961 A-50 |
| Process water | 508 | 743 | 743 | gpm | [P] x s; MEA make-up unpriced |

Plant-level H2 recovery exceeds 88% because the cryogenic unit returns H2 ahead of the PSA. INL publishes no value for it; the flowsheet calculates the return fraction, 0.921630, by closure on INL's natural gas and hydrogen pair.

### 4.2 Conditions

| Parameter | Value, all cases | Label, source |
|---|---|---|
| Reactor outlet | 925 C | [P] TEV-961 Table 2 |
| Secondary helium supply, return | 900 C, 466 C | [P] TEV-961 Table 2, cont. |
| Primary return | 491 C, 25 K approach | [C] |
| Helium pressure | 7.0 MPa central, 5.15 MPa sensitivity | [A] |
| Reformer outlet | 871 C, 30.3 bar | [P] TEV-961 PDF p14; Case 5 pressure, A-55 |
| Shift | HTS in 335 C, LTS in 191 C | [P] Case 5 |
| Absorber | 40 C, 26.8 bar | [P] A-53, Case 5 |
| MEA | 30 wt%, rich 0.50, lean 0.25 | [A] |
| Stripper | 2 bar | [A] |
| CO2 product | 138.9 bar, 84.4 C in INL's compressor basis | [P] A-53; see X8 |

### 4.3 Carbon and emissions at 85%

| Parameter | Case B | A INL | A MEA | Unit | Label |
|---|---|---|---|---|---|
| Feed carbon | 1,219 | 1,784 | 1,784 | kmol C/h | [C] |
| Tube carbon | 1,574 | 2,157 | 2,304 | kmol C/h | [C] |
| CO2 at absorber | 1,220 | 1,672 | 1,785 | kmol/h | [C] |
| Captured | 1,098 | 1,655 | 1,607 | kmol/h | [C] |
| Absorber slip, recycled | 122 | 17 | 179 | kmol/h | [C] |
| Tail gas carbon crossing the boundary | 472 | 496 | 690 | kmol C/h | [C] |
| Of which recycled | 355 | 373 | 519 | kmol C/h | [C] |
| Of which burned, N2-rich cut | 117 | 123 | 171 | kmol C/h | [C] |
| Carbon in H2 product | 4.4 | 6.5 | 6.5 | kmol C/h | [C] 0.1 mol% CH4 |
| Baseline | 616,361 | 902,060 | 902,060 | t/y | [C] TEV-953 Table 2 |
| Captured | 359,777 | 542,362 | 526,544 | tCO2/y | [C] |
| To atmosphere | 38,309 | 40,248 | 56,066 | tCO2/y | [C] |
| Net abatement | 578,052 | 861,812 | 845,994 | tCO2e/y | [C] |
| Bound, Cho 9.35 | 587,436 | 875,547 | 859,729 | tCO2e/y | [C] |
| Bound, IEAGHG 8.996 | 563,745 | 840,874 | 825,056 | tCO2e/y | [C] unrounded, IEAGHG 2017-02 Table 2 |
| Per-pass capture | 90 | 99 | 90 | % | [A]; [P] Selexol |
| Plant capture | 90.4 | 93.1 | 90.4 | % | [C] |
| Reduction against unabated SMR | 93.8 | 95.5 | 93.8 | % | [C] |
| Direct intensity | 0.57 | 0.41 | 0.57 | kg/kg H2 | [C] |
| Baseline carbon never generated | 218,274 | | | tCO2/y | [C] heat plus recycle |

Residual emissions come from the burned N2-rich cut, and absorber slip returns through the reformer. The cryogenic unit is INL's fixed-assumption block (TEV-961 A-16, A-18), drawn for Selexol's slip; with MEA at Case A the absorber slip is ten times larger (179 against 17 kmol/h), so 23.2 MMSCFD carries that open risk. The Modelled preset's carbon ledger is written as run `B_REC_F_MODELLED` in `results/Integrated_Mass_Balance.csv` and `results/Emissions_Ledger.csv`.

### 4.4 Heat

| Term | Case B | A INL | A MEA | Unit | Label |
|---|---|---|---|---|---|
| Process heat to helium | 120.80 | 176.80 | 176.80 | MWth | [P] x s, covers preheat as well as reforming |
| CO2 slip sensible heat | 0.88 | 0 | 1.28 | MWth | [C] RWGS excluded |
| Secondary helium flow | 54.02 | 78.49 | 79.06 | kg/s | [P], [C] |
| Primary helium flow at need | 79.17 | 107.52 | 115.65 | kg/s | [C] |
| Reboiler duty | 51.00 | 0 | 74.64 | MW | [C] 3.8 GJ/t |
| Freed Selexol steam | 12.73 | 0 | 18.63 | MW | [C] Case 5 proxy |
| Export steam diverted | 16.76 | 0 | 24.53 | MW | [P] 83.7 MMBtu/h x s |
| Syngas above 130 C | 8.39 | 0 | 12.27 | MW | [C] term T05, Case 5 chain on the plant CO2 scale |
| Reboiler gap, reactor steam | 13.12 | 0 | 19.21 | MW | [C] |
| IHX, duct, vessel losses | 0.97, 0.31, 2.66 | 1.41, 0.45, 3.26 | 1.42, 0.45, 3.42 | MW | [C], [A] |
| Circulator shaft heat returned | 6.70 | 9.40 | 9.80 | MW | [C] |
| Power branch incl. gap | 59.57 | 69.88 | 87.16 | MWth | [C] |
| **Reactor needed** | **178.49** | **242.41** | **260.74** | MWth | [C] |
| Range, best to high | 155.78 to 214.09 | 224.84 to 279.11 | 227.55 to 313.32 | MWth | [C] card J.27 |
| Against 210 MWth | +31.51 | -32.41 | -50.74 | MWth | [C] |
| Spare on Ahn's heat basis | +6.54 | | | MWth | [C] sweep HEAT_BASIS |

INL itself quotes 178.8 MWth for Case 6 because it imports 16.5 MWe (TEV-961 Table 2, cont.). Open terms that move Case B: a secondary helium drop of 414 kPa adds 6.21 MWth (sweep SEC_DP), and a 39.7% already net of primary circulation removes 6.48 MWth (sweep ETA_NET_OF_PRIMARY). On the plant's own Level 3 syngas (7.904 MW above 130 C in place of the gated 8.387 MW) the reboiler gap grows by 0.48 MW and the need by 0.38 MWth (sweep T05_BASIS). At the high corner the need is 214.09 MWth; with the reactor held at 210 MWth the heat duties are met and 1.62 MWe of electricity is imported (rows `DRIFT_HEADLINE_R_HIGH_` of `results/Final_Design_Basis.csv`).

### 4.5 Electricity

| Load | Case B | A INL | A MEA | Unit | Row |
|---|---|---|---|---|---|
| Reformer train | 0.82 | 1.20 | 1.20 | MWe | REFORMING_POWER |
| Gas purification | 0.01 | 2.10 | 0.02 | MWe | PURIFICATION_POWER, PSA only under MEA |
| Steam system | 0.20 | 0.30 | 0.30 | MWe | STEAM_SYSTEM_POWER |
| Water treatment | 1.02 | 1.50 | 1.50 | MWe | WATER_POWER |
| Lean pump, CO2 compression and purge recovery | 9.44 | 12.10 | 13.81 | MWe | PUMP_COMPRESS |
| Cooling towers and MEA fans | 0.83 | 0.10 | 1.22 | MWe | COOLING |
| Process subtotal | 12.33 | 17.30 | 18.05 | MWe | RES_E_PROCESS_MWE |
| Secondary circulator | 3.48 | 5.06 | 5.10 | MWe | RES_E_CIRC_SEC_MWE, no measured basis |
| Primary circulator | 3.96 | 5.38 | 5.79 | MWe | RES_E_CIRC_PRIM_MWE, may double count, see 4.4 |
| **Total, generated from reactor heat** | **19.78** | **27.74** | **28.94** | MWe | POWER_GENERATED |
| Grid import at rated size | 0 | 12.87 | 22.10 | MWe | UNMET_ELECTRICAL |

The rows of each run close: POWER_GENERATED plus TAIL_GAS_POWER equals the process rows plus HE_CIRCULATION (test `eq_j_24_energy_groups_close_on_their_rows`). BRANCH_EXTRACTION_WORK, the back-pressure work on the reboiler steam, is a part of POWER_GENERATED and of the power branch, so it is never added to the electricity side.

### 4.6 Cost inputs

| Input | Low | Central | High | Unit | Label, source |
|---|---|---|---|---|---|
| Reactor OCC, no turbine | 1,456 | 2,184 | 2,548 | US$2019/kWt | [P] low, high; [C] central, INL/RPT-23-72972 PDF p47 |
| FOAK premium | 1.3 | 1.6 | 2.1 | x BOAK | [P] Table 1 |
| Learning | | 10 | | % per doubling | [P] Table 1 |
| Size exponent | 0.31 | 1.0 | 0.6 | n, printed together | [U] |
| Reactor OPEX with fuel | 6 | 10 | 14 | US$2019/MWth-h | [P] low, high; [C] central |
| Discount rate | 5 | 8 | 10 | % | [A] |
| Lives | 25 | 40 | 60 | y, printed pair is 25 and 60 | [A] |
| Gas price | 10 | 15 | 20 | S$/GJ HHV | [P] central; [A] range |
| T&S | 23.2 | 60 | 112 | S$/tCO2 | [U] |
| Capture capital | 1.74 | 2.04 | 5.09 | S$M per t/h | [C] IEAGHG Case 1A aMDEA, Case 3 MEA high |
| Capture fixed O&M | 2.7 | 2.7 | 6 | % per y | [A]; [P] Ahn Table 3 high |
| IHX and loop capital, at 170 MW | 13.7 | 129.7 | 207.5 | S$M | [P] via earlier notes, primary sources unopened |
| Extra licensing | 0 | 50 | 137.5 | US$M | [P] proxy |
| Decommissioning | 750 | 1,000 | 1,250 | US$2022/kWe | [P] LWR history |
| Recycle and cryo train, Case A | 0 | 71.5 | 307.5 | S$M | [C] proxy, no published cost |
| FX and escalation | | 1.3065, 1.6195, 2.322 | | S$/US$; S$2025/US$2019; S$2025/EUR2014 | [P], [C] |

## 5. Sizing walk, MWth

The walk is the screening derivation of the rating on the Anchored preset, held by the golden test `sizing_walk`. Its last line moves to 178.5 MWth on the Headline preset, where term T05 runs on its gated model.

| Step | Case B | Case A, MEA |
|---|---|---|
| 176.8 + 11.6/0.397 | 206.0 at 130 MMSCFD | 206.0 |
| Same sum at Case B | 140.8 | n/a |
| Restore purge load | 150.6 | 220.4 |
| Add vessel, IHX and duct losses | 154.3 | 225.3 |
| Add helium circulators | 165.9 | 242.4 |
| Swap Selexol for MEA, zero reboiler heat | 166.2 | 242.8 |
| Add reboiler gap at 3.8 GJ/t, Anchored | 179.0 | 261.5 |
| Headline need | 178.5 | 260.7 |
| Rated | 210.0 | 210.0 |
| Margin, Headline | +31.5 | -50.7 |

On the Headline preset, SRD 3.4 and 4.2 GJ/t move Case B to 174.1 and 182.9 MWth (sweep SRD).

## 6. Ledger

Two independent methods feed one result (D08).

**Method 1, annual incremental ledger.** The L0 to L5 ladder below, in `engine/plant/src/economics.rs`. All lines are annual differences against the same-size unabated SMR. Case B, BOAK, n = 1.0, 210 MWth charged, 578,052 t/y.

**Method 2, discounted cash flow.** `engine/plant/src/dcf.rs` runs year-by-year real cash flows at 8% for two plants of equal H2 output on a common horizon: the unabated SMR and NUS-H2.

- Construction: 5.5 y uniform spend for the reactor, the INL NZW09 base case, then operation at 85% availability.
- Horizon: the reactor life, 25 y, 40 y or 60 y. The items with a 25 y life (capture plant, IHX, recycle train) and the SMR island are replaced in full every 25 y of operation. The unused life of the last replacement is credited at the horizon, straight line, in that year's cash flow.
- Decommissioning is paid at end of life from cash flow, with no sinking fund.
- Absolute plant costs: both plants carry the SMR island capital and fixed O&M from IEAGHG 2017-02 Base Case, re-based with the escalation in section 4.6. NUS-H2 carries it unchanged, on the assumption that reformer differences cancel. The capital basis is Total Capital Requirement (Table 3, PDF p17) because it carries its own IDC. Fixed O&M, raw water and chemicals with catalysts come from Table 4 (PDF p18), and the two variable lines scale from Table 4's 95% availability to 85%.
- Credits: export steam and purchased power per the L5 lines.

Outputs: LCOH of each plant, NPV of costs over NPV of H2, in S$/kg; and the levelised abatement cost LAC_DCF, NPV of the cost difference over NPV of tonnes abated, in S$/t, with tonnes discounted at the same rate. `LAC_DCF` against L5 is a test: a difference over 10% would need a named term in `docs/`. The largest difference is 0.47% (section 1).

**History.** An earlier deck ledger mixed S$/t with S$M/y and rested on inputs that D04, D09 and D10 replace. It is withdrawn, and none of its figures is used here.

### 6.1 Central lines, S$M/y

| Set | Line | 25 y | 60 y |
|---|---|---|---|
| L0 | Gas saved, 12.6 MMSCFD at S$15/GJ | -64.80 | -64.80 |
| L0 | Reactor capital, S$742.8M | 69.58 | 60.01 |
| L0 | Capture capital | 9.25 | 9.25 |
| L0 | Capture fixed O&M | 2.67 | 2.67 |
| L0 | T&S, S$60/t on 359,777 t/y | 21.59 | 21.59 |
| L1 | Reactor O&M and fuel | 25.32 | 25.32 |
| L2 | IHX and secondary loop | 8.36 | 8.36 |
| L3 | Extra licensing | 6.12 | 5.28 |
| L3 | Decommissioning | 1.50 | 0.09 |
| | **L3 total** | **79.60** | **67.77** |
| | **L3, S$/t** | **137.70** | **117.24** |
| L4 | IDC on reactor | 13.75 | 11.86 |
| L4 | Power island add-back | 1.62 | 1.40 |
| L4 | Recycle and cryo train | 5.89 | 5.89 |
| | L4, S$/t | 174.49 | 150.38 |
| L5 | Lost steam export | 15.23 | 15.23 |
| L5 | Baseline grid power avoided | -6.41 | -6.41 |
| | **L5 total** | **109.69** | **95.75** |
| | **L5, S$/t** | **189.75** | **165.64** |
| DCF | LAC_DCF, S$/t | 190.06 | 165.97 |

L5 is the full cost set under D08, and L3 stays in the table as a named step. L4 and L5 carry the recycle train and the diverted export steam, which D04 and D10 make real costs of this plant.

### 6.2 Stage, exponent and life, 210 MWth charged, S$/t

L5, then L3 in brackets, at 25 / 40 / 60 y.

| Stage | n = 1.0 | n = 0.6 | n = 0.31 |
|---|---|---|---|
| FOAK | 277.9 / 250.3 / 241.7 (209.9 / 186.8 / 179.5) | 411.0 / 370.8 / 358.2 (323.1 / 289.4 / 278.9) | 548.8 / 495.5 / 478.8 (440.3 / 395.7 / 381.8) |
| BOAK | 189.7 / 171.4 / 165.6 (137.7 / 122.1 / 117.2) | 277.7 / 251.4 / 243.2 (213.2 / 191.1 / 184.1) | 368.7 / 334.3 / 323.6 (291.4 / 262.4 / 253.4) |
| 10OAK, p46 route | 146.3 / 132.5 / 128.2 (102.1 / 90.3 / 86.6) | 212.1 / 192.7 / 186.6 (159.1 / 142.6 / 137.5) | 280.1 / 255.0 / 247.1 (218.1 / 196.8 / 190.2) |
| 10OAK, p47 route | 208.5 / 188.1 / 181.8 (153.0 / 135.8 / 130.5) | 306.0 / 276.8 / 267.7 (236.6 / 212.0 / 204.3) | 407.0 / 368.5 / 356.6 (323.0 / 290.7 / 280.7) |

Charging the 178.5 MWth the process needs, n = 1.0, L5 at 25 / 40 / 60 y: BOAK 161.2 / 145.3 / 140.4, FOAK 236.4 / 212.6 / 205.3, 10OAK p46 124.1 / 112.2 / 108.5, 10OAK p47 177.1 / 159.6 / 154.2. Case A, each charged at its own need, n = 1.0, L5 at 25 and 60 y: A MEA BOAK 157.5 and 137.2, FOAK 232.6 and 202.0; A INL BOAK 133.5 and 114.9, FOAK 202.1 and 174.0.

### 6.3 Sensitivities, L5, BOAK, 25 y, S$/t

Central 189.7.

| Change | Result | Change | Result |
|---|---|---|---|
| T&S 23.2 / 112 | 166.8 / 222.1 | Gas S$10 / S$20 per GJ | 218.3 / 161.2 |
| Reactor OCC 1,456 / 2,548 US$/kWt | 141.7 / 213.8 | Reactor capital -30% / +30% | 146.5 / 233.0 |
| Capture capital 1.74 / 5.09 S$M per t/h (Case 3 MEA high) | 186.7 / 220.5 | Capture fixed O&M 6% | 195.4 |
| Gas saving at Ahn 18.8%, carbon held / carbon follows | 242.0 / 261.1 | Baseline Cho / IEAGHG | 186.7 / 194.6 |
| Availability 70% / 90% | 233.8 / 178.3 | Discount rate 5% / 10% | 136.4 / 231.4 |
| Reactor charged at the 178.5 MWth need | 161.2 | Heat basis on Ahn's 143.4 MWth | 192.6 |
| Reformer exit 825 / 800 C through the flowsheet | 204.8 / 217.0 | Condensate CO2 vented (decision S14) | 190.0 |

SRD moves the margin and leaves this cost almost unchanged (189.72 and 189.77 at 3.4 and 4.2 GJ/t), because the reactor is charged at 210 MWth.

Abatement sensitivities, t/y: per-pass capture 80%, 85% and 95% give 564,180, 571,388 and 584,231. Availability 70% and 90% give 476,043 and 612,055. Cryogenic carbon held at INL's flow (run B_REC_FIX) gives 588,860. No recycle with H2 held (B_COMB_H) gives 467,353 at 30.4 MMSCFD. The system view with lost steam gives 0.528 Mt/y, the T&S burden 0.569 Mt/y, and the lifecycle 0.566 Mt/y with burdens only and 0.616 Mt/y with saved upstream gas. Vented condensate CO2 gives 576,321 (decision S14). Reformer exit 825 and 800 C give 559,036 and 544,468; the cryogenic recycle share of the reject carbon is held at INL's 0.752 (card J.7) in those runs, so the burned cut takes the extra reject carbon.

Backup firing over the 1,314 outage hours adds 11,810 t H2/y. Firing unabated emits 108,770 t/y, which matches the unabated SMR over the same hours and adds no abatement. Firing on a conventional train with capture emits 29,017 t/y and adds 79,753 t/y; the card K.20 heater emits 44,476 t/y and adds 64,293 t/y. Furnace capital is unpriced.

### 6.5 Comparator

IEAGHG 2017-02 Case 3 at 100,000 Nm3/h and 85%: 90.0% overall capture, 536,335 t/y against its own 8.996 kg/kg baseline, S$162.1/t at T&S S$23.2/t (EUR 10/t as published), S$203.0/t at S$60/t and S$260.8/t at S$112/t. Gas price and availability differ from this ledger, so the row is labelled indicative.

### 6.6 Distance to S$100/t, one lever at a time

Recomputed on the full cost set, L5 and the DCF, n = 1.0, 210 MWth charged (sweep LEVER_100, card L.31). At L5, BOAK, 25 y the ceiling is S$57.8M/y and the ledger is S$109.7M/y.

| Case | Cost, S$/t | Gas price needed, S$/GJ | Reactor OCC needed, US$2019/kWt | T&S needed, S$/t |
|---|---|---|---|---|
| L5, BOAK, 25 y | 189.7 | 30.70 | 824 | -84.2 |
| L5, BOAK, 60 y | 165.6 | 26.48 | 1,031 | -45.5 |
| L5, FOAK, 25 y | 277.9 | 46.13 | 499 | -225.9 |
| L5, FOAK, 60 y | 241.7 | 39.79 | 628 | -167.7 |
| DCF, BOAK, 25 y | 190.1 | 30.76 | 823 | -84.7 |
| DCF, BOAK, 60 y | 166.0 | 26.54 | 1,028 | -46.0 |
| DCF, FOAK, 25 y | 278.4 | 46.21 | 498 | -226.7 |
| DCF, FOAK, 60 y | 242.2 | 39.88 | 626 | -168.5 |

A T&S price below zero means that lever cannot close the gap alone. The reactor charged at its need with a 40 y life gives S$145.3/t on L5 and S$145.9/t on the DCF at BOAK, and S$212.6/t and S$213.4/t at FOAK, against the 25 y rated cost.

## 7. Single calculation database

### 7.1 One source file

`engine/data/parameters.toml` holds every input number. No other file may hold a project input. Unit conversions and shared physical constants live in `engine/props/src/units.rs`; species data sit in `engine/props/src/species.rs`.

```toml
[[param]]
id = "H2_B_NM3H"          # unique, UPPER_SNAKE
value = 100000.0
unit = "Nm3/h"
low = 100000.0            # optional
high = 100000.0           # optional
class = "published"       # published | assumed | calculated | user_decision
rigour = "fixed_assumption"  # rigorous | reduced_order | empirical | fixed_assumption | external_boundary
source = "Air Liquide release, 15 Jan 2008"
locator = "para 1"
source_status = "primary" # primary | via_reader | via_old_repo | not_verified
decision = "D03"
```

### 7.2 Flow

| Stage | Reads | Writes | Direction rule |
|---|---|---|---|
| Engine | `parameters.toml`, embedded with `include_str!` | model state | Native and wasm builds share one embedded file |
| Exporter, `cargo run --release -p nush2_cli -- export --out results` | engine | seven CSVs in `results/` | Only writer of `results/` |
| Number map, `scripts/build_numbers.py` | seven CSVs | `build/numbers.json`, git-ignored | Python never reads the TOML |
| Report and deck builders | `build/numbers.json` | local `report/`, `presentation/` | Text carries tokens such as `{{A_ATM_B_MT}}` |
| Derivations | `build/numbers.json` | `math/` worked examples | Same tokens |
| Sensitivity sweep | `parameters.toml` low and high | `results/Sensitivity_Results.csv` | Overrides stay in memory |
| Web simulator | wasm with the embedded TOML | browser only | Defaults equal the CSV central row |

The release export and a dev-profile export agree within `scripts/compare_results.py`. The condition text of the flowsheet closure rows leaves out the tear pass count, which follows round-off in the natural gas secant and differed between the two profiles.

### 7.3 Run definitions

Exporter runs fixed run ids: `B_REC_F` (headline), `A_INL_SEL`, `A_REC_F`, `B_REC_FIX`, `B_COMB_H`, `B_EXPORT`. `B_REC_F_ANCHORED` and `B_REC_F_MODELLED` repeat Case B REC_F on the other two presets. Ledger defaults are n = 1.0, 210 MWth charged, BOAK, 25 and 60 y. Case A runs charge their own needed rating.

### 7.4 Rounding

t/y to the tonne, Mt/y to 3 dp, MWth and MWe to 1 dp in text and 2 dp in tables, S$/t and S$M/y to 1 dp in text and 2 dp in ledgers. Report units use the `$\mathrm{...}$` form so that they become native Word math.

### 7.5 Engine replaces screening

| Quantity | Screening value | Engine, Headline | Tolerance before review |
|---|---|---|---|
| H2 Case B | 66,925 t/y | 66,925 t/y | 1 t/y |
| Net abatement Case B | 578,052 t/y | 578,052 t/y | 0.5% |
| Reactor needed Case B | 179.0 MWth | 178.5 MWth | 1.0 MWth |
| L3 cost, BOAK, 25 y | S$137.7/t | S$137.7/t | S$1.0/t |

All four sit inside tolerance. The reactor need moves by 0.55 MWth because term T05 runs on its gated model (section on drift). No parameter was tuned to return to a screening value.

### 7.6 Guards in CI and local checks

| Guard | Pass condition |
|---|---|
| `results_fresh` | A release re-export agrees with committed `results/` under `scripts/compare_results.py` |
| `params_complete` | Every entry has class, rigour and source; every id is read at least once |
| `no_stale_numbers` | No superseded value in deliverables outside rows marked Case A reference |
| `forbidden_text` | `scripts/check_clean.py` finds no local absolute path, identifier pattern or withdrawn figure. It has no rule for names, so a separate pre-push scan outside the repository gates every push |

## 8. Results files

All files are UTF-8 CSV with one header row, SI or stated units, no merged cells. Every calculated row names its equation card, and the last column of every table is `status`.

### `results/Final_Design_Basis.csv`
One row per parameter, inputs and key results.

| Column | Type | Meaning |
|---|---|---|
| id | text | Parameter id, equals the TOML id for inputs |
| group | text | production, conditions, carbon, heat, power, nuclear, cost, drift |
| parameter | text | Plain name |
| case_b, case_a_inl, case_a_mea | number | Values as in section 4 |
| unit | text | |
| low, high | number | Blank when not applicable |
| class, rigour | enum | |
| source, locator, source_status | text | |
| equation | text | Card number, blank for inputs |
| decision | text | D number |
| status | enum | `input`, `anchored_port`, `gated_T05` and the other status words below |

Result families: `RES_` (plant results), `AC_L3_`, `AC_L5_`, `LAC_DCF_` and `LCOH_` (the cost grid at 25 and 60 y), `TERM_` (each term's anchored, modelled and used values), `L4_` (heat recovery, IHX and circulators) and `DRIFT_` (the section 7 drift values on each preset, and the high corner against the rating). `TERM_T06_MODELLED` holds the modelled SRD of the failed M6 gate; `scripts/build_numbers.py` keeps it out of the tokens (decision S12).

### `results/Integrated_Mass_Balance.csv`
One row per stream per run.

| Column | Type | Meaning |
|---|---|---|
| run_id | text | Section 7.3, plus the `FS_` Level 3 runs |
| stream_id, stream_name | text | |
| from_unit, to_unit | text | |
| crosses_boundary | bool | True when exactly one end of the stream sits inside the dashed line of Figure 2; the PSA, the battery limits, the product, the atmosphere and off-site users sit outside |
| t_c, p_bar | number | |
| total_kg_h, total_kmol_h | number | Every species the row holds |
| h2, co, co2, ch4, c2plus, c2h6, c3h8, nc4h10, nc5h12, n2, h2o, h2o_liquid, mea | number | kmol/h |
| carbon_kmol_h | number | |
| class, rigour, source | text | `external_boundary` for PSA streams |

Anchored runs write `H2_PRODUCT`, `CO2_TO_STORAGE`, `N2_RICH_CUT`, `NG_FEED`, `CRYO_H2_RETURN`, `PSA_FEED`, `PSA_REJECT_RETURN`, `RECYCLE_TO_REFORMER` and `TAIL_GAS_EXPORT`. Every node with streams in and out closes on carbon, and on H2 where each stream at the node carries its H2 (test `eq_j_11_anchored_nodes_close`). Only the recycle runs hold a cryogenic unit: where the reject is burned or exported it leaves the PSA once, by that route, and `CRYO_H2_RETURN` and `PSA_REJECT_RETURN` are zero. Each Level 3 run adds `COMBUSTION_O2`, the stoichiometric O2 of the N2-rich cut burner from the battery limit, so its envelope closes on mass and oxygen from the table alone. Closing rows `CLOSURE_CARBON`, `CLOSURE_H`, `CLOSURE_MASS` (and `CLOSURE_N`, `CLOSURE_O` for the Level 3 runs) hold residuals per run.

### `results/Integrated_Energy_Balance.csv`
One row per term per run.

| Column | Type | Meaning |
|---|---|---|
| run_id, term_id, term | text | |
| side | enum | source, sink |
| kind | enum | heat, shaft, electricity, enthalpy |
| value_mw, low_mw, high_mw | number | |
| temperature_c | number | Heat grade, blank for electricity |
| class, rigour, source, equation | text | |
| closure_group | enum | `reactor`, `reboiler`, `electricity`, `envelope`, `part:POWER_BRANCH`, `memo` |

Rows of one closure group close on their own: in `reactor`, `reboiler`, `electricity` and `envelope` the sources equal the sinks, and the rows of `part:POWER_BRANCH` add up to `POWER_BRANCH`. A `memo` row repeats or annotates a value held in a group and enters no sum. The reactor heat budget: `HTGR_HEAT` plus `CIRC_SHAFT_HEAT` equals `IHX_DUTY`, `IHX_LOSS`, `DUCT_LOSS`, `VESSEL_LOSS` and `POWER_BRANCH`. The reboiler: `MEA_REGEN` equals the `REB_FROM_` rows in the approved order, each with its grade in `temperature_c`. Electricity: `POWER_GENERATED` and `TAIL_GAS_POWER` equal `HE_CIRCULATION`, `PUMP_COMPRESS`, `COOLING`, `REFORMING_POWER`, `PURIFICATION_POWER`, `STEAM_SYSTEM_POWER` and `WATER_POWER`. Each Level 3 run (`FS_` ids) prints its process heat, shift release, syngas heat above 130 C and reboiler duty as `memo` rows, and in the `envelope` group every unit duty, signed, with the formation-basis enthalpy flow of each envelope stream (`H_S01` and so on). Duties set by closure carry `fixed_assumption`, so for those blocks the envelope closure is bookkeeping.

### `results/Emissions_Ledger.csv`
One row per line, per run, per baseline. Run `B_REC_F_MODELLED` repeats Case B REC_F on the Modelled preset.

| Column | Type | Meaning |
|---|---|---|
| run_id | text | |
| baseline_id | enum | INL_9p21, CHO_9p35, IEAGHG_9p00 (the unrounded 8.995997 kg/kg) |
| view | enum | direct_atm, system, lifecycle_burdens, lifecycle_upstream |
| line_id, line | text | `BASELINE`, `CAPTURED`, `N2_RICH_CUT`, `ABATEMENT`, `TS_BURDEN`, `UPSTREAM_CREDIT`, `NUCLEAR_LC` |
| t_per_y, kg_per_kg_h2 | number | |
| per_pass_capture, plant_capture | number | Printed together |
| class, source, equation | text | |

### `results/Economic_Ledger.csv`
One row per line, per stage, life, exponent and charged size.

| Column | Type | Meaning |
|---|---|---|
| run_id | text | |
| stage | enum | FOAK, BOAK, 10OAK_P46, 10OAK_P47 |
| life_y | int | 25, 40, 60 |
| n | number | 1.0, 0.6, 0.31 |
| reactor_charged_mwth | number | 210 or the need |
| level | enum | L0 to L5, and DCF |
| line_id, line | text | Ledger lines `GAS`, `REACTOR_CAP`, `CAPTURE_CAP`, `CAPTURE_FOM`, `TS`, `REACTOR_OM`, `IHX`, `LICENCE`, `DECOM`, `IDC`, `POWER_ISLAND`, `RECYCLE_TRAIN`, `LOST_STEAM`, `BASE_POWER`; DCF rows below |
| value_sgdm_y | number | Ledger lines |
| low, high | number | |
| cum_sgdm_y, cum_s_per_t | number | Cumulative to this line |
| abatement_t_y | number | Denominator used |
| class, source, source_status, equation | text | |
| dcf_value | number | Value of a DCF row, blank on ledger lines |
| dcf_unit | text | Unit of `dcf_value` |

DCF rows, level `DCF`: `LCOH_SMR` and `LCOH_NUSH2` in S$/kg, `LAC_DCF` in S$/t, `LAC_L5_GAP` (LAC_DCF less L5, S$/t), `LAC_L5_GAP_REL` (that gap over L5, against the 0.10 limit) and the named gap terms `GAP_` at n = 1.0. The source of the LCOH rows states the limit of the IEAGHG island, whose capital includes its own power island.

### `results/Sensitivity_Results.csv`
One row per run and metric.

| Column | Type | Meaning |
|---|---|---|
| sweep_id, run_id | text | |
| parameter, unit | text | |
| base_value, test_value | number | |
| metric | text | A_T_Y, AC_L3_25, AC_L3_60, AC_L5_25, AC_L5_60, LAC_DCF_25, LAC_DCF_60, R_NEEDED, MARGIN_210, NG_MMSCFD; the `FS_` metrics of flowsheet points; GAP_MW in T05_BASIS; the solved values of LEVER_100, MATURE_60Y and COFUNDING |
| result, delta | number | delta from central, or from the target for a solved cost |
| class, note | text | Corner runs say they carry no probability |

Sweeps beyond the freeze list: `T05_BASIS` (term T05 on the flowsheet's own syngas), `MATURE_60Y`, `COFUNDING`, `LEVER_100`, `CRYO_H2_SPLIT`, `REFORMER_T`, `LTS_K_SOURCE`, `CONDENSATE_CO2` and `BACKUP_FIRING`.

### `results/Validation_Results.csv`
One row per benchmark.

| Column | Type | Meaning |
|---|---|---|
| test_id | text | |
| level | int | Model hierarchy level 1 to 6 |
| subject | text | |
| source, locator | text | Paper, page or table |
| conditions | text | Operating conditions |
| literature, simulated | number | |
| unit | text | |
| abs_error, rel_error, tolerance | number | |
| status | enum | pass, fail, reported, not_run |
| data_use | enum | independent, calibration, same_dataset_disclosed |
| explanation, commit | text | |

`reported` marks a value printed with no limit, such as a station temperature or a profile point; it is neither a pass nor a fail. A benchmark judged on a range or a trend states its limit in `explanation` and leaves `tolerance` blank. Closure, tear and code-check rows carry `same_dataset_disclosed`, so they earn no validation credit. The harness in `engine/vv` writes every row, and `docs/VALIDATION_MATRIX.md` gives each benchmark's limit and status.

## 9. Conflicts created by the decisions

| ID | Clash | Effect | Default applied | Author call |
|---|---|---|---|---|
| X1 | The earlier deck ledger rests on inputs that D04, D09 and D10 replace | Closed | Ledger withdrawn, D08 | None |
| X2 | The earlier deck sum mixes S$/t with S$M/y | Closed | Ledger withdrawn, D08 | None |
| X3 | L3 counts the recycle energy and omits the train (S$5.9M/y), and diverts export steam without pricing it (S$15.2M/y) | Closed. L4 S$174.5/t, L5 S$189.7/t at 25 y | Full cost set is the headline with L3 a named step, D08 | None |
| X4 | The first and only plant on the roadmap is FOAK, and the headline pair is BOAK | L5 S$277.9/t against S$189.7/t at 25 y | FOAK and BOAK print side by side | Choose |
| X5 | Equal prominence of n = 0.31, which is two FOAK estimates | L5 span S$189.7 to S$368.7/t at 25 y | Printed at equal size with that label | None |
| X6 | Charging 210 MWth while using 178.5 MWth | +S$28.6/t on L5 at 25 y | Central stays at 210 MWth; 178.5 MWth prints as the feasible alternative | None |
| X7 | Margin depends on the heat basis | 31.5 MWth on INL heat, 6.5 MWth on Ahn heat, -4.1 MWth at the high corner | Print all three | None |
| X8 | Ship storage against INL compression: 4.25 MWe compresses to 138.9 bar for a pipeline, while ship transport needs liquefaction | Load is unmodelled for ship. DICE compression runs 26% higher | INL load scaled, labelled, open item | None |
| X9 | Storage sits inside the line, while the T&S burden sits in the lifecycle sensitivity | -8,995 t/y gives 0.569 Mt/y | 0.578 stays headline, 0.569 prints beneath it | Confirm |
| X10 | Storage is offshore and inside the line, while the plant sits in Singapore | Accounting across the border is unsettled | Abatement counted at the Singapore stack; the report states the dependency on the Singapore-Indonesia arrangement | None |
| X11 | The figure sent tail gas away and had no cryogenic unit or stack | Figure must change while D06 says follow it | Keep the dashed line. Edit the arrow, add the compressor and cryogenic block and the N2-rich stack, change 176.8 to 120.8 | None |
| X12 | 19.8 MWe needs a turbine island, and the deck crossed out the turbine | Co-generation text contradicts that slide | The cross applies to the condensing power plant only; a small power block is added to the figure; S$1.6M/y sits at L4 | None |
| X13 | New-build against wording that retires an existing Jurong source | New-build retires no source | Wording changes; SOXAL sets scale only | None |
| X14 | Share of 2035 emissions | The plant runs near 2050 on the roadmap, and 45 to 50 Mt is an NDC target | 1.2% to 1.3% prints as a size comparison only | None |
| X15 | Reclamation runs over "coming decades", with no date, while construction starts in 2039 | Land may not exist at that milestone. The pipeline links are unpriced | Dates labelled illustrative; link costs listed as omitted | None |
| X16 | Backup firing needs a furnace that the ledger omits | Capital unpriced | Sensitivity only, capital flagged unpriced | None |
| X17 | Comparator on IEAGHG 8.996, headline on INL 9.21 | Abatement bases differ | 8.996 is D07's lower bound, row labelled | None |
| X18 | Bottom-up capture capital is IEAGHG Case 1A aMDEA, while the solvent is MEA | MEA basis adds S$30.8/t on L5 | Keep with label; Case 3 in sensitivities | None |
| X19 | S$60/t T&S is assumed, and ERIA's US$43 to 86/t is transport only | A ship chain could exceed S$60/t | Print low and high beside central | None |
| X20 | 23.2 MMSCFD assumes the recycle transfers to MEA | Falls towards 467,353 t/y and 30.4 MMSCFD if it fails | Disclosed limit, sensitivity row | None |
| X24 | Lifecycle: the deck counts burdens only, an earlier version credits saved upstream gas | 0.566 against 0.616 Mt/y | Both print; the higher one is labelled | None |

## 10. Open items

Nothing here blocks the build, since each has a default in section 9.

1. Does BOAK stay the headline pair, or does FOAK lead? (X4)
2. Does 0.578 Mt/y stay the headline with 0.569 beneath it? (X9)
3. Sources to supply, in order of effect: TEV-962, TEV-954, TEV-981, the TEV-961 Case 6 sheets, IAEA-TECDOC-1682, GA-A24265, Cho 2022 full text, the ETC report.

## Drift between headline and modelled terms

`nush2 drift` runs Case B REC_F on the three presets and reads the section 7 limits. A term leaves its anchored value only where the `GATE_` row of its term passes in `results/Validation_Results.csv`. The V&V harness writes those rows in the same export from the rows behind each gate, and the exporter stops when they differ from the table the engine records. The figures below are the CLI output at this revision. The same values sit in the `DRIFT_`, `TERM_` and `L4_` rows of `results/Final_Design_Basis.csv`, and `results/Integrated_Energy_Balance.csv` repeats Case B REC_F on each preset.

### Presets at Case B REC_F

| Preset | Reactor need, MWth | Spare against 210 MWth | Net abatement, t/y | Reboiler gap, MW | Electricity, MWe |
|---|---|---|---|---|---|
| Anchored | 179.040 | 30.960 | 578,052 | 13.823 | 19.791 |
| Headline | 178.494 | 31.506 | 578,052 | 13.123 | 19.778 |
| Modelled | 178.475 | 31.525 | 577,933 | 13.106 | 19.776 |

### Terms

| Term | Name | Unit | Anchored | Modelled | Headline uses | Modelled uses | Gate | Gate outcome | Modelled slot |
|---|---|---|---|---|---|---|---|---|---|
| T01 | Carbon split after shift | - | 0.7750 | 0.7776 | 0.7750 (anchored) | 0.7776 (model) | M4 | passed | SMR-EQ, HTS and LTS on TEV-961 Case 5, bed sized to the IEAGHG approach |
| T02 | Per-pass capture | - | 0.9000 | 0.9000 | 0.9000 (anchored) | 0.9000 (anchored) | M6 | failed | absorber solved to 90% per pass on the RSP-250 stand-in geometry |
| T03 | Process heat | MW | 120.80 | 109.65 | 120.80 (anchored) | 120.80 (anchored) | M4 | not_run | Level 3 preheat plus reformer at Case B REC_F; upstream preheat incomplete |
| T04 | Recycle slip heat | MW | 0.8769 | 1.6500 | 0.8769 (anchored) | 0.8769 (anchored) | M7 | failed | process heat of REC_F less INL_SEL at the same H2, Level 3 flowsheet |
| T05 | Syngas above 130 C | MW | 7.6862 | 8.3867 | 8.3867 (model) | 8.3867 (model) | M4 | passed | Case 5 chain from SMR-EQ, 149 to 130 C with condensation, on the plant CO2 scale |
| T06 | Specific reboiler duty | GJ/t | 3.8000 | 3.6461 | 3.8000 (anchored) | 3.8000 (anchored) | M6 | failed | equilibrium-stage stripper and regeneration train on the Case B rich solvent |
| T07 | Lean pump | MWe | 0.8691 | 0.9092 | 0.8691 (anchored) | 0.8691 (anchored) | M6 | failed | lean pump on the M6 solvent flow and density |
| T08 | MEA cooling load | MWe | 0.7650 | 0.7257 | 0.7650 (anchored) | 0.7650 (anchored) | M6 | failed | lean cooler plus condenser duty at the anchored k |
| T09 | CO2 compression | MWe | 4.2455 | 3.3732 | 4.2455 (anchored) | 4.2455 (anchored) | M7 | failed | isothermal ideal-gas limit, MW of shaft work, a cross-check on another basis |
| T10 | Secondary circulator pressure drop | kPa | 200.00 | 91.1036 | 200.00 (anchored) | 200.00 (anchored) | M8 | not_run | smooth-wall Blasius on the HA reformer annulus at 199 tubes plus HTTR-size IHX tubes; a lower bound |
| T11 | Primary circulator pressure drop | kPa | 150.00 | none | 150.00 (anchored) | 150.00 (anchored) | none | no gate | no model |
| T12 | Losses | MW | 3.9460 | none | 3.9460 (anchored) | 3.9460 (anchored) | none | no gate | no model |
| T13 | Natural gas | MMSCFD | 23.2316 | 23.0986 | 23.2316 (anchored) | 23.2316 (anchored) | M7 | failed | natural gas for 8,988 kg/h H2 with the S03 tear, Level 3 flowsheet |

### Section 7 checks

| Check | Limit | Value | Raised |
|---|---|---|---|
| Headline R above 210 MWth | R > 210 MWth | 178.49 MWth | No |
| Modelled R above 210 MWth | R > 210 MWth | 178.47 MWth | No |
| Headline high-corner R above the 210 MWth rating | High-corner R > 210 MWth | 214.09 MWth, +4.09 MWth; with the reactor held at 210 MWth the unmet heat is 0.00 MWth and the electricity import 1.62 MWe | Yes, not blocking |
| Modelled abatement against 0.578 Mt/y | Off by more than 0.012 Mt/y | 0.5779 Mt/y, -0.0001 Mt/y | No |
| Modelled SRD | Outside 3.4 to 4.2 GJ/t | 3.646 GJ/t, gate failed | No |
| 90% reachable at rich loading 0.50 | Above 0.50, or packing above 40 m | 15.16 m at rich loading 0.500 | No |
| Modelled NG for 8,988 kg/h | Off 23.2 MMSCFD by over 5% | 23.099 MMSCFD, -0.44% | No |
| Headline spare | Outside 27 to 33 MWth | 31.51 MWth | No |

Neither blocking check is raised. The high corner of card J.27 needs 4.09 MWth more than the rating; at that corner the rated reactor still covers the heat duties and imports 1.62 MWe, so the finding is stated in the report and changes no input.

### Notes on the drift

Headline flips T05 alone. Its gate is the six TEV-961 Case 5 cooler duties (T961-7), and all of them pass. The modelled 8.387 MW replaces 7.686 MW, so the reactor steam the reboiler draws falls by 0.70 MW and the reactor need by 0.55 MWth. Carbon does not move, so Headline abatement equals Anchored. The gate validates the Case 5 chain on Case 5 only; on the plant's own Level 3 EX3 outlet the syngas above 130 C is 7.904 MW, and sweep T05_BASIS prints the need at that value, 178.87 MWth, beside the gated 178.49 MWth.

T04, T07 and T08 are on the Headline flip list and stay anchored. T04 waits on FS_GATE_M7, which failed on the carbon lines (decision S11). T07 and T08 wait on the M6 column gate, which failed on SA-1, DICE-SRD, KW-SRD, MO-K16-TOUT, VI-3-SS2 and VI-3-SS3 (decision S12). The modelled SRD of 3.646 GJ/t sits in this table and in the validation rows only, with that gate failure beside it: the same stripper model overstates the DICE pre-FEED design SRD and the Kwak pilot SRD by 27 to 45%, so 3.8 GJ/t stays in Headline. The capture island now closes on energy across its hand-offs (section 19 of the validation matrix), which moved the modelled SRD from 3.650 to 3.646 GJ/t and T08 from 0.6897 to 0.7257 MWe.

Modelled also takes T01, whose M4 gate passed on the raw syngas rows and on the shift outlet rows of the sized basis. A higher shift conversion lowers Modelled abatement, 577,933 against 578,052 t/y: the anchored port re-derives INL's Case 6 recycle share from the carbon split, so the higher split moves tail-gas carbon from the recycle to the burner. The Level 3 flowsheet gives a split of 0.8256 at Case B REC_F against the anchored 0.775, the gap behind the failed `carbon_matches_anchor`. Routing the reject H2 that the cryogenic unit does not return to the PSA into the recycle (decision S11, sweep `CRYO_H2_SPLIT`) moves that split to 0.8231 and tube carbon down by 11.4 kmol/h, so the carbon lines stay outside 2% of the anchor and the default split stands.

T03 has no complete replacing model. The Level 3 preheat plus reformer duty is 109.65 MW at Case B against the 120.8 MW basis, and the upstream preheat rests on Case 6 states the library lacks. T09 holds the isothermal shaft-work limit, which cannot replace the INL electricity row. T10 holds 91.1 kPa at the 199 tubes of the HA design and 38.8 kPa and 6.2 kPa at the 327 and 1079 tubes the S8 finding names; fins, coil curvature, headers and ducts are left out, so these are lower bounds, and no measured helium-side drop exists to gate them.

Heat recovery on real flows. The M6 reboiler sits at 121.0 C. LP steam at 307 F (152.8 C) and export steam at 489 F (253.9 C) from the INL steam system (TEV-961 A-57, PDF p81) clear it by 31.8 K and 132.9 K. The syngas cut at 130 C clears it by 9.0 K and is the pinch of the stack. The 200 F hot water loop sits below the reboiler and is excluded. No minimum approach is published in the library, so the check judges the sign only.

IHX and circulators on the Headline duty of 121.68 MW: a balanced 25 K approach needs UA 4.867 MW/K at NTU 17.36, which is 57.3 HTTR-size modules, and 12,168 m2 at 400 W/m2/K. The 760 W/m2/K upper case of the engine plan has no primary page in the library, so its 6,404 m2 is an unsourced reporting case. Helium flows from the duty (card B.9) are 53.97 kg/s secondary and 79.17 kg/s primary, against the 54.02 kg/s the anchored port scales from INL; the 0.1% gap is the c_p of 5195 against the INL-implied 5190 J/kg/K. The duct loss on pipe diameter (card B.14) is 0.373 MW against the 0.307 MW of the port's linear scaling; T12 stays anchored, so the balance keeps 0.307 MW.

