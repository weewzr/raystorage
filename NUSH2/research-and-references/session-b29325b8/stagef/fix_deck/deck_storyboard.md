# 06 NUS-H2 deck storyboard

Written 2026-11-02 after stage 1, against `build/numbers.json` and the report's derived layer. It feeds `presentation/build_deck.py`, which writes `presentation/NUSH2_Final_Presentation.pptx` and `.pdf`. The builders, this file, the speaker script and the Q&A sheet stay local. This file holds no typed result. Every result is a token, so a re-run of the builder refreshes the deck with no hand edits.

Authority: `USER_ANSWERS.md`, then `10_build_order.md`, then the freeze. Rows in `01_value_changes.csv` for old slides 3, 24 to 29 and the old reactor-capital-only backup row are replaced by build order section 2, so they are not carried here. The report text in `report/_build/content/` sets the wording of the boundary, the cost story and the stacked route, so deck and report say the same thing.

Checker: `design/calc/deckboard/check_storyboard.py`. It resolves every token, runs every guard, scans the prose and prices the speaker notes against the clock. Results of its last run are in section 3.3.

## 1. Delivery pattern taken from the recording

Structure and delivery come from the FREESTRYDE pitch. No content comes from it. Times below are the frame stamps on `tl_0.jpg` to `tl_2.jpg`.

| Recording | Beat | What it does | NUS-H2 slides |
|---|---|---|---|
| 20 to 34 s | Human hook | Presenter speaks to the audience in second person, no data on screen | S01, S02 |
| 36 to 38 s | One big number | The condition, one large figure, nothing else | S03 |
| 40 to 46 s | Name reveal | Product name alone on the slide, team introduces itself | S04 |
| 48 s | What it is | One sentence, one picture | S07 |
| 52 to 88 s | Proof points | Each claim is one slide, same layout, bold coloured figure, picture left | S08 to S10, S13 |
| 90 to 98 s | Gathered claims | Claims recapped, then the benefit to the user | S17 |
| 100 to 106 s | Expert voice | Quotation card, "do not take my word for it" | S11 |
| 108 to 114 s | Market | Size and growth, then positioning against rivals | S32 |
| 116 to 132 s | Money | Price and revenue model as large figures | S37 to S39 |
| 134 to 160 s | Growth chart | Stepwise bars with two large figures | S41 |
| 162 to 184 s | People | Names and faces | S44 |
| 186 to 196 s | Closing recap | One slide in columns, stays up while thanks are spoken | S45 |

The recording shows no risks, boundary or method. The brief requires them, so those acts sit between the proof points and the market beat and use the same near-identical slide rhythm. The pace is slower than the recording, at about 26 s a slide against about 7 s, because this talk runs 20 minutes.

Taken from the recording: pale gradient, bold coloured headline words, one big numeral, one visual left and text right, each new point arriving as the next near-identical slide, one closing recap in columns. Left behind: its blue palette and its product claims. NUS-H2 stays white and purple.

## 2. Build contract

### 2.1 Tokens

Slide text and notes carry `{{TOKEN}}` or `{{TOKEN|filter}}`. A bare token prints the `text` field. Filters read the `value` field.

| Filter | Result |
|---|---|
| `f0` to `f4` | Fixed decimals with thousands separators, as in the report's grammar |
| `Mt` | Value x 1e-6, 3 decimals |
| `pct0`, `pct1` | Value x 100, 0 or 1 decimal |
| `abs` | Absolute value at the token's own text decimals |

| Prefix | Source | Rule |
|---|---|---|
| none | `build/numbers.json` tokens, or `report/_build/numbers_resolved.json` for derived tokens | The builder runs `report/_build/resolve_numbers.py` first and reads its output, so deck and report share one derived layer |
| `SWEEP_` | `numbers.json` `sweeps`, the `test_value` whose prefix is `SENS_` plus the rest of the name | Input value of a sensitivity run |
| `NEW_` | Formula in section 6.1 | Arithmetic on tokens only |
| `CTX_` | Table in section 6.2 | Published fact with a citation, never a model result |

Years, reference numbers, table and page numbers stay literal. `[[word]]` in a title marks the bold coloured headline words. Text renders CO2, H2 and N2 with subscript digits, except the name NUS-H2, which stays as written. Any token that resolves to `n/a` or `not run` stops the build.

### 2.2 Layout

Slide size stays 18288000 x 10287000 EMU, 20 x 11.25 in. Positions are in inches. Every main slide uses one of these, so that a new point looks like the next frame of the same slide.

| Template | Use | Geometry |
|---|---|---|
| T1 Hook | S01 to S04 | Picture bleeds left, x 0 to 8.6. Right column as T2 |
| T2 Number | Default | Visual panel x 0.8, y 0.9, w 9.0, h 9.45, white with a 1 pt lavender line. Kicker x 10.6, y 0.9. Headline y 1.45, h 2.2. Numeral y 3.9, h 3.2. Unit line y 7.1. Support text y 7.75, h 2.2. Source tag y 10.2 |
| T2b Two numbers | Pairs that must print at equal size | As T2, two numerals side by side |
| T3 Figure | Full figures | Headline top, visual x 0.8, y 2.0, w 18.4, h 8.1. Number chip top right |
| T4 Columns | S42, S45 | Four cards, w 4.4, gap 0.27, y 2.2 to 9.9. Four is deliberate: the prose rule bars the recording's three |
| T5 Table | Backup | Table x 0.8, y 2.0, w 18.4, minimum 18 pt |

Palette, sampled from the deck renders. Headline words `#1400AC`. Deep purple kicker and rules `#4D3583`. Ink `#1B1A2E`. Muted text `#5B5870`. Gradient white `#FFFFFF` to pale lavender `#EFEAFB` at 135 degrees. The old dark economics section goes, so every slide sits on the gradient. Red `#B3261E` appears only on a missed target or a limit banner.

Type: Segoe UI Semibold headlines at 44 pt, Segoe UI Black numerals at 120 to 140 pt, Segoe UI body at 26 pt or more, source tags at 13 pt. Heading Now and Atkinson are not installed here, and Segoe UI is. The PDF is exported by PowerPoint COM on the application object the builder opened, closed in a finally block.

Footer: NUS-H2 wordmark left, page number right, and on any slide that prints a model result a status tag from `S_STATUS` below. A slide carries at most one visual, one numeral group and 25 words of support text.

`S_STATUS`: when `STATUS_ROWS_ANCHORED_PORT` is above zero, "Reduced-order balances anchored on INL cases". Otherwise "Engine result". `S43` swaps its second sentence by the same test.

## 3. Coverage

### 3.1 Brief narrative

| Brief item | Slides |
|---|---|
| 1 Project title | S04 |
| 2 Singapore problem statement | S01 to S03 |
| 3 Scope and limitations | S05, S06 |
| 4 Proposed solution | S07 to S11 |
| 5 Why nuclear | S12, S13 |
| 6 Engineering basis and sizing | S14 to S16 |
| 7 Benefits of integration | S17 |
| 8 Risks and trade-offs | S18 to S23 |
| 9 Implementation location | S24 |
| 10 Implementation roadmap | S25 |
| 11 Emissions boundary | S26 |
| 12 Emissions methodology | S27 |
| 13 Emissions results | S28, S29 |
| 14 Sensitivity analysis | S30, S31 |
| 15 Economics methodology | S32, S33 |
| 16 Cost assumptions | S34 |
| 17 Reactor cost scenarios | S35, S36 |
| 18 Abatement cost results | S37 to S41b |
| 19 Overall feasibility | S42, S43 |
| 20 Conclusions | S45 |
| 21 References and backup | B01 to B10 |

The title moves from first to fourth so that the name lands after the hook, as in the recording. The brief calls its order indicative. The team slide S44 is added from the recording.

### 3.2 Labels that must appear on the slide

| Slide | Label |
|---|---|
| S14 | INL reference. Separate from the proposed design |
| S24 | Team's proposal. No official approval |
| S25 | Illustrative dates |
| S35 | Generic HTGR. Xe-100 is an analogue only, with no vendor quotation |
| S41 | Base full cost printed beside the mature point |
| B03 | Indicative. Gas price and availability differ from this ledger |

### 3.3 Pacing and last check

Computed by the checker at 135 words a minute on resolved notes, plus 2 s for each slide change. Target is about 20 minutes, with 10 minutes of questions afterwards.

<!-- pacing -->
| Act | Slides | Words | Time |
|---|---|---|---|
| Hook | S01 to S03 | 94 | 0:48 |
| Name reveal | S04 | 38 | 0:19 |
| Scope | S05 to S06 | 119 | 0:57 |
| What it is | S07 | 64 | 0:30 |
| Proof | S08 to S10 | 159 | 1:17 |
| Expert voice | S11 | 58 | 0:28 |
| Why nuclear | S12 to S13 | 109 | 0:52 |
| Sizing | S14 to S16 | 168 | 1:21 |
| Benefits | S17 | 49 | 0:24 |
| Safety | S18 to S21 | 250 | 1:59 |
| Regulation | S22 | 54 | 0:26 |
| Community | S23 | 30 | 0:15 |
| Location | S24 | 59 | 0:28 |
| Roadmap | S25 | 58 | 0:28 |
| Emissions boundary | S26 | 59 | 0:28 |
| Emissions method | S27 | 61 | 0:29 |
| Emissions result | S28 to S29 | 89 | 0:44 |
| Sensitivity | S30 to S31 | 104 | 0:50 |
| Market | S32 | 65 | 0:31 |
| Economics method | S33 | 63 | 0:30 |
| Cost assumptions | S34 | 56 | 0:27 |
| Reactor cost | S35 to S36 | 133 | 1:03 |
| Money | S37 | 60 | 0:29 |
| Abatement cost | S38 to S41 | 220 | 1:46 |
| Growth chart | S41b | 52 | 0:25 |
| Feasibility | S42 | 78 | 0:37 |
| Limits | S43 | 54 | 0:26 |
| People | S44 | 9 | 0:06 |
| Close | S45 | 77 | 0:36 |
| Total | 46 slides | 2489 | 19:58 |

Last run: 56 storyboard slides plus 16 Q&A slides, 11700 tokens loaded, every storyboard guard passed.
<!-- /pacing -->

### 3.4 Old slides

| Old | Content | Goes to |
|---|---|---|
| 1 | Title and names | S04. Identifiers removed, title reworded |
| 2 | Problem statement | S01 to S03 |
| 3 | Scope and targets | S05, S06 |
| 4 | Proposed solution figure | S07, edited figure |
| 5 | Why nuclear | S12, S13, S18 |
| 6 | Basis and sizing | S14, S15 |
| 7 | Benefits | S17, with the X12 caption |
| 8 | Risks and trade-offs | S18 to S23, wording corrected |
| 9 | Location | S24 |
| 10 | Roadmap | S25, rebuilt figure |
| 11 | Template slide | Deleted |
| 12, 13 | Boundary, equations | S26 to S28 |
| 14 | Baselines | S06, S27. The stream split is retired |
| 15 | Abatement results | S28, S29 |
| 16 | Sensitivity | S30, S31 |
| 17 | Economics divider | Dropped, with its stock image |
| 18 | Scenario and scale | S06. Logo removed |
| 19, 20 | Older flowsheet | Replaced by S07 |
| 21 | Abatement cost equation | S33 |
| 22 | Ceiling | S40, B05 |
| 23 | Capture cost exhibit | Dropped. It prints a withdrawn figure |
| 24 to 29 | Withdrawn ledger | None of it stays. The topics return from results at S09, S34 to S41 and B02 |
| 30 | Thank you | Spoken over S45 |
| 31 | References | B09, B10, Nelson report number corrected |
| 32 to 37 | Template slides | Deleted |

## 4. Main deck

### S01 | T1 | Hook | Brief 2
**Title:** Switch on a [[light]] tonight
**One idea:** Singapore's electricity runs on imported natural gas.
**Large number:** `{{CTX_GAS_POWER_PCT}}%` natural gas in the electricity fuel mix, 2024.
**Visual:** `pptx:image1.jpeg`, the chimney from old slide 1, left bleed.
**Slide text:** Natural gas in Singapore's electricity fuel mix, 2024.
**Speaker note:** Switch on a light in Singapore tonight. In 2024, natural gas made up {{CTX_GAS_POWER_PCT}} percent of the fuel behind that electricity, and we import all of it.
**Source:** `r1`; `r2`

### S02 | T1 | Hook | Brief 2
**Title:** Hydrogen could carry [[up to half]]
**One idea:** National strategy sees hydrogen as a possible route to much of our future power.
**Large number:** up to `{{CTX_H2_POWER_PCT}}%` of power needs by 2050.
**Visual:** `pptx:image45.jpeg`, the industrial skyline from old slide 30, left bleed.
**Slide text:** Low-carbon hydrogen, a possible share of power needs by 2050.
**Speaker note:** The national hydrogen strategy points one way out. Low-carbon hydrogen could supply up to {{CTX_H2_POWER_PCT}} percent of our power needs by 2050. Everything then rests on the word low-carbon.
**Source:** `r3`

### S03 | T1 | Hook | Brief 2
**Title:** Today's hydrogen carries a [[carbon bill]]
**One idea:** A conventional reformer emits this much CO2 for every kilogram of hydrogen.
**Large number:** `{{BASELINE_INL_KG_KG}}` kg CO2 per kg H2.
**Visual:** `pptx:image45.jpeg`, tight crop on the stacks.
**Slide text:** Direct emissions of an unabated steam methane reformer, INL basis.
**Speaker note:** Steam methane reforming is the established route. It burns gas for heat, and it strips carbon from the gas it converts. On Idaho National Laboratory's numbers, every kilogram of hydrogen leaves {{BASELINE_INL_KG_KG}} kilograms of CO2 in the air.
**Source:** `r10` Table 2, PDF p30

### S04 | T1 | Name reveal | Brief 1
**Title:** [[NUS-H2]]
**One idea:** The name, and its author.
**Large number:** None. The name takes the numeral's place, as in the recording.
**Visual:** `pptx:image1.jpeg`, same crop as S01.
**Slide text:** Nuclear-Upgraded Steam Methane Reforming with Carbon Capture for Low-Carbon Hydrogen in Singapore. Wee Zhern Ray.
**Speaker note:** This is NUS-H2: Nuclear-Upgraded Steam Methane Reforming with Carbon Capture, for low-carbon hydrogen in Singapore. I am Wee Zhern Ray, and the question here is how far a conventional reformer can go once a reactor replaces its furnace.
**Source:** Decision D22

### S05 | T2b | Scope | Brief 3
**Title:** [[Two targets]], one straight answer
**One idea:** The project sets two tests, and we report both results now.
**Large number:** `{{TARGET_ABATEMENT_MT}}` Mt/y and `S${{TARGET_COST_S_T|f0}}/t`.
**Visual:** None. Two native target cards fill the panel, each stamped with its result token.
**Slide text:** Abatement target: {{TARGET_ABATEMENT_RESULT}}. Cost target: {{TARGET_COST_RESULT}}.
**Speaker note:** Our project sets two tests for a solution at scale in Singapore: abate more than {{TARGET_ABATEMENT_MT}} million tonnes of CO2e a year, at under S${{TARGET_COST_S_T|f0}} a tonne. Here is the answer up front. The abatement target is {{TARGET_ABATEMENT_RESULT}}. The cost target is {{TARGET_COST_RESULT}} at central inputs. The rest of this talk shows by how much, and what would close the gap.
**Source:** Project guideline s1

### S06 | T2 | Scope | Brief 3
**Title:** One plant at [[real industrial scale]]
**One idea:** One new-build plant, steady state, with the PSA left as an external block.
**Large number:** `{{H2_B_NM3H}}` Nm3/h of purified hydrogen.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_PSA`.
**Slide text:** {{H2_B_T_Y}} t H2/y at {{AVAILABILITY_PCT|f0}}% availability. PSA reject returns across the boundary.
**Speaker note:** We size one new-build plant at {{H2_B_NM3H}} normal cubic metres of hydrogen an hour, the scale of SOXAL's plant on Jurong Island, used for size only. At {{AVAILABILITY_PCT|f0}} percent availability that is {{H2_B_T_Y}} tonnes a year. The model is steady state. The PSA stays an external, unsimulated block, and its reject returns across our boundary as a stream.
**Source:** `airliquide2008`; `airliquide2011`; Decisions D04, D05

### S07 | T3 | What it is | Brief 4
**Title:** A reformer heated by [[helium]]
**One idea:** A reactor replaces the fired furnace, and MEA captures CO2 from shifted syngas.
**Large number:** `{{R_RATED_MWTH|f0}}` MWth reactor, in the chip.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, full figure, replacing `pptx:image5.png`.
**Slide text:** Generic HTGR, helium-heated steam methane reforming, MEA capture, ship offshore storage.
**Speaker note:** Here is the plant. On the left, a generic high-temperature gas-cooled reactor rated at {{R_RATED_MWTH|f0}} megawatts thermal, with helium leaving at {{ROT}} degrees. An intermediate heat exchanger, the IHX, passes that heat to a separate helium loop at {{T_HE_SUPPLY}} degrees, which heats the reformer tubes. Shift and MEA capture follow, then CO2 compression and ship offshore storage. The PSA sits outside the dashed line.
**Source:** `r8` Table 2; Decisions D06, D23

### S08 | T2 | Proof | Brief 4
**Title:** Helium heat [[replaces the furnace]]
**One idea:** All process heat comes from helium, so no natural gas burns for reforming heat.
**Large number:** `{{Q_PROCESS_B_MWTH}}` MWth at `{{T_HE_SUPPLY}}` C.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_IHX_SMR`.
**Slide text:** Covers reforming and preheat. Helium returns at {{T_HE_RETURN}} C.
**Speaker note:** First proof point. The process needs {{Q_PROCESS_B_MWTH}} megawatts of heat, covering reforming and preheat, at {{T_HE_SUPPLY}} degrees. In INL's study, {{ROT}} degrees is the only case that removes the fired stage. The reformer outlet of {{SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C}} degrees is INL's equilibrium assumption, and we disclose it as one.
**Source:** `r8` printed p12 to p13, p21, PDF p14

### S09 | T2b | Proof | Brief 4
**Title:** [[Less gas]] for the same hydrogen
**One idea:** NUS-H2 takes less gas than a same-size unabated steam methane reforming plant, if recycle transfers to MEA.
**Large number:** `{{NG_B_MMSCFD}}` against `{{NG_UNABATED_B_MMSCFD}}` MMSCFD.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_RECYCLE`.
**Slide text:** {{NG_SAVED_B_MMSCFD}} MMSCFD saved. Depends on recycle transferring from Selexol to MEA.
**Speaker note:** Second proof point. A same-size unabated reformer takes {{NG_UNABATED_B_MMSCFD}} million standard cubic feet of gas a day. NUS-H2 takes {{NG_B_MMSCFD}}, because no gas burns for reforming heat and PSA reject recycles. The published evidence disagrees on this saving: {{GAS_SAVING_INL_PCT}} percent on INL's basis, {{GAS_SAVING_AHN_PCT}} percent in Ahn and Lee. INL also never modelled recycle with MEA, and we come back to that.
**Source:** `r10` Table 2, PDF p30; `ahn2026`; Decision D33

### S10 | T2b | Proof | Brief 4
**Title:** Capture, [[counted two ways]]
**One idea:** Per-pass and plant capture always print together.
**Large number:** `{{PER_PASS_CAPTURE_PCT|f0}}%` per pass and `{{PLANT_CAPTURE_B_PCT}}%` of plant carbon.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_CAPTURE`.
**Slide text:** {{W_MEA|pct0}} wt% MEA. {{CAPTURED_B_MT}} Mt CO2/y to storage. Per-pass value is a project assumption.
**Speaker note:** Third proof point. The absorber takes {{PER_PASS_CAPTURE_PCT|f0}} percent of the CO2 on each pass, a project assumption, using {{W_MEA|pct0}} weight percent MEA. Absorber slip recycles through the reformer, so the whole plant captures {{PLANT_CAPTURE_B_PCT}} percent of its carbon. We print both numbers together so the two never get mixed up.
**Source:** Decisions D07, D25; `ieaghg2022` PDF p44

### S11 | T2 | Expert voice | Brief 4
**Title:** In [[INL's]] own words
**One idea:** The anchor for this design is published INL analysis, with its limits.
**Large number:** `{{CTX_INL_GAS_CUT_PCT}}%` less gas than INL's conventional plant with capture, at `{{ROT}}` C.
**Visual:** None. Native quotation card: "Increasing the HTGR ROT significantly reduces natural gas consumption." R. Wood, INL TEV-961, 2010.
**Slide text:** INL has not reviewed NUS-H2.
**Speaker note:** You need not take our word for this. INL modelled reactor heat for reforming in 2010, and the line on screen is theirs. At {{ROT}} degrees they found {{CTX_INL_GAS_CUT_PCT}} percent less gas than a conventional plant with capture. INL used Selexol, and it has never reviewed NUS-H2. We borrow its method as our anchor, along with its limits.
**Source:** `r8` Executive Summary, PDF p5

### S12 | T2 | Why nuclear | Brief 5
**Title:** Why nuclear: the [[temperature fits]]
**One idea:** Of the reactor types on this chart, only an HTGR reaches the reforming band.
**Large number:** `{{ROT}}` C reactor outlet.
**Visual:** `rm:v17_image2.png`, the report's process temperature chart, cropped below its red label, replacing `pptx:image8.png`. A native label carries the reformer outlet token.
**Slide text:** Reformer outlet {{SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C}} C. Only test reactors have run this hot.
**Speaker note:** Why nuclear? Reforming sits in the hot band on this chart. Light-water reactors stop far short of it, which is why we use a gas-cooled reactor in place of a PWR. INL's yield optimum is {{CTX_INL_OPT_ROT_C}} degrees with a fired stage kept, and {{ROT}} is its only case without one. The catch: only test reactors of {{CTX_TEST_REACTOR_MAX_MW}} megawatts or less have run this hot, and commercial HTR-PM runs near {{CTX_COMMERCIAL_ROT_C}}.
**Source:** `r6`; `d_nelson2011` PDF p9, p21; `iaeanpt43` printed p23; `r11`; `wnn2023`

### S13 | T2 | Why nuclear | Brief 5
**Title:** [[Heat and power]] from one reactor
**One idea:** Fission gives heat with no combustion, and a small power block runs the site.
**Large number:** `{{E_TOTAL_B_MWE}}` MWe site load, grid import `{{RES_GRID_IMPORT_AT_RATED_MWE|f0}}`.
**Visual:** `pptx:image7.png`, the fission diagram from old slide 5.
**Slide text:** Process loads {{E_PROCESS_B_MWE}} MWe. Helium circulators add the rest.
**Speaker note:** Fission gives us heat with no combustion at the reformer. The same reactor also runs the site: a small power block supplies {{E_TOTAL_B_MWE}} megawatts electric for process loads and helium circulators, so grid import at rated size is {{RES_GRID_IMPORT_AT_RATED_MWE|f0}}.
**Source:** `d_nuclearpower`; Decision D24; `Integrated_Energy_Balance.csv`

### S14 | T2 | Sizing | Brief 6
**Title:** [[INL reference]]: Case A
**One idea:** INL's published case anchors the sizing and stays a reference only.
**Large number:** `{{H2_A_MMSCFD_AINL|f0}}` MMSCFD hydrogen, INL reference.
**Visual:** `pptx:image9.png`, captioned "Case A, INL reference (Wood, 2010)".
**Slide text:** INL reference. Separate from the proposed {{R_RATED_MWTH|f0}} MWth design.
**Speaker note:** Sizing starts from INL's Case 6, at {{H2_A_MMSCFD_AINL|f0}} million standard cubic feet of hydrogen a day. We call it Case A, and it stays a reference, separate from our design. With MEA and recycle, Case A would need {{R_NEEDED_A_MEA_MWTH}} megawatts thermal, {{CASE_A_SHORT_MWTH}} above our rating. So we size the plant to fit.
**Source:** `r8` Table 2; Decision D03

### S15 | T2 | Sizing | Brief 6
**Title:** The walk to [[{{R_NEEDED_B_MWTH}} MWth]]
**One idea:** Counting every real load gives the reactor the process needs.
**Large number:** `{{R_NEEDED_B_MWTH}}` MWth needed, range `{{R_BEST_B_MWTH}}` to `{{R_HIGH_B_MWTH}}`.
**Visual:** New chart `CH1 Chart_Sizing_Walk`, section 7.3. It replaces the old table `pptx:image10.png`.
**Slide text:** SRD of {{SRD_CENTRAL}} GJ/t is an assumption with no syngas measurement.
**Speaker note:** So we scale to Case B and walk the reactor size. The team's first sum was {{VAL_SIZING_WALK_STEP_0_SIM}} at Case A scale, which is {{VAL_SIZING_WALK_STEP_1_SIM}} at ours. Restoring the purge load and counting losses and circulators brings it to {{VAL_SIZING_WALK_STEP_4_SIM}}. MEA's reboiler gap takes the need to {{R_NEEDED_B_MWTH}}. That reboiler demand, {{SRD_CENTRAL}} gigajoules per tonne, has no syngas measurement behind it.
**Source:** Freeze section 5; Decisions D24, D26; `Validation_Results.csv` SIZING_WALK rows

### S16 | T2 | Sizing | Brief 6
**Title:** A margin that [[earns nothing]]
**One idea:** The rating leaves spare heat, which costs money and is thinner than it looks.
**Large number:** `{{MARGIN_B_MWTH}}` MWth spare, no revenue.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_NUCLEAR`.
**Slide text:** {{SENS_HEAT_BASIS_143P4_MARGIN_210}} MWth on Ahn and Lee's heat basis. {{ADVERSE_SHORT_MWTH}} MWth short at the adverse corner.
**Speaker note:** We keep the {{R_RATED_MWTH|f0}} megawatt rating, so {{MARGIN_B_MWTH}} megawatts sit spare and earn nothing. We still pay for them: full cost is S${{AC_L5_BOAK_25Y}} a tonne as charged, against S${{AC_L5_BOAK_25Y_N1P00_NEED}} sized to need. The margin is also thin. On Ahn and Lee's heat basis it falls to {{SENS_HEAT_BASIS_143P4_MARGIN_210}}, and at our adverse corner the need reaches {{R_HIGH_B_MWTH}}.
**Source:** Decisions D03, X6, X7; `ahn2026` Table 6

### S17 | T2 | Benefits | Brief 7
**Title:** Integration [[removes equipment]]
**One idea:** Each plant sheds equipment the other makes redundant.
**Large number:** `{{RES_CARBON_NEVER_GENERATED_T_Y}}` t CO2/y never formed.
**Visual:** `pptx:image12.png` with `pptx:image11.png` inset, from old slide 7. Caption: "Condensing power plant removed. A small power block stays for site loads."
**Slide text:** The fired furnace goes. So does the large condensing power plant.
**Speaker note:** Join the two plants and each one sheds equipment. The reformer loses its fired furnace. The reactor loses its large condensing power plant and keeps a small power block for site loads. With the furnace gone and fuel gas recycled, {{RES_CARBON_NEVER_GENERATED_T_Y}} tonnes of baseline CO2 a year never forms.
**Source:** `r8`; Decision X12; `Emissions_Ledger.csv`

### S18 | T2 | Safety | Brief 8
**Title:** TRISO: the [[first barrier]]
**One idea:** TRISO coatings are the first barrier to fission product release, within a temperature limit.
**Large number:** `{{CTX_TRISO_LIMIT_C}}` C fuel limit.
**Visual:** `pptx:image6.png`, the TRISO particle from old slide 5, at no more than 5.6 in wide, or redrawn natively with its five labelled layers.
**Slide text:** TRISO coatings are the first barrier to fission product release, provided fuel stays below about {{CTX_TRISO_LIMIT_C}} C.
**Speaker note:** Now the risks, starting with safety. TRISO coatings are the first barrier to fission product release, provided fuel stays below about {{CTX_TRISO_LIMIT_C}} degrees. HTTR counts four barriers: coatings, pressure vessel, containment and reactor building. Tritium is the known exception, since it can leave cracked coatings and also forms in graphite.
**Source:** `d_tecdoc1236` printed p14, p24, p54, p104; `d_inltriso`

### S19 | T2 | Safety | Brief 8
**Title:** The IHX: the [[boundary]] we must isolate
**One idea:** The IHX is the first boundary between the nuclear island and the process loop.
**Large number:** None. The words "first boundary" take the numeral's place, since isolation is a design duty with no project figure.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_IHX`.
**Slide text:** Secondary helium held slightly above primary pressure. Isolation valves are required and the figure omits them.
**Speaker note:** The IHX is the first boundary between the nuclear island and the process loop. Its tubes are a pressure boundary, and they remove nothing. Secondary helium is held slightly above primary pressure, so a leak runs towards the reactor. Purification and monitoring manage tritium permeation. Isolation needs a high-temperature emergency shutoff valve, which our figure omits. A thermal buffer for a reformer trip is also still open in our design.
**Source:** `iaeanpt43` printed p11, p33; `d_tecdoc1382` printed p4; `d_tecdoc1236` printed p54 to p55, p59; `d_tochio2011` PDF p9; `d_sato2006` PDF p11

### S20 | T2b | Safety | Brief 8
**Title:** [[{{T_HE_SUPPLY}} C]] helium is still unproven
**One idea:** No measured IHX has delivered this temperature, and the no-furnace claim depends on it.
**Large number:** `{{CTX_HE_MEASURED_C}}` C measured against `{{T_HE_SUPPLY}}` C needed.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_IHX`, same frame as S19.
**Slide text:** {{IHX_APPROACH_K}} K approach is INL's assumption. At {{IHX_APPROACH_K_HIGH}} K the reformer pinch falls to {{NEW_PINCH_IAEA_K}} K.
**Speaker note:** This is our largest technical gap. The highest measured IHX secondary temperature in our sources is {{CTX_HE_MEASURED_C}} degrees, on a {{CTX_HTTR_IHX_MW}} megawatt HTTR unit. We need {{T_HE_SUPPLY}} degrees at {{NEW_IHX_DUTY_RATIO}} times that duty, and no source we hold shows an IHX heating a chemical plant. Our {{IHX_APPROACH_K}} kelvin approach is INL's assumption. At {{IHX_APPROACH_K_HIGH}} kelvin, the IAEA's typical value, the reformer pinch falls from {{NEW_PINCH_CENTRAL_K}} to {{NEW_PINCH_IAEA_K}} kelvin, and the no-furnace claim is at risk.
**Source:** `iaeanpt43` printed p14; `d_tochio2011` PDF p4, p8; `r8` printed p10; `d_sato2006` PDF p1

### S21 | T2 | Safety | Brief 8
**Title:** Emergency zone: set by a [[safety case]]
**One idea:** A smaller zone may be justified, and only a design-specific safety case can show it.
**Large number:** None. The words "no distance claimed" take the numeral's place.
**Visual:** `rm:v17_image3.png`, the report's dose-frequency chart. Caption: "Method example from the Xe-100 application. Its distance belongs to that case."
**Slide text:** A low-power TRISO HTGR may justify a smaller EPZ, subject to a design-specific safety case.
**Speaker note:** Emergency planning matters most in a country our size. A low-power TRISO HTGR may justify a smaller emergency planning zone, subject to a design-specific safety case. The zone follows source term and dose, so we claim no distance. The same case must also cover process hazards beside the reactor, such as fire and toxic release.
**Source:** `iaeanpt43` printed p18, p32 to p35; `r13`; `r17`

### S22 | T2 | Regulation | Brief 8
**Title:** Singapore is [[building capability]]
**One idea:** Capability is growing, and no decision on nuclear energy has been made.
**Large number:** `2027`, INIR Phase 1 assessment, tentative date, `{{CTX_INIR_AREAS}}` areas.
**Visual:** None. Native timeline strip showing the May 2026 INIR announcement, the October 2026 manpower schemes and 2027.
**Slide text:** Singapore has made no decision on nuclear energy.
**Speaker note:** On regulation and people, Singapore is building the capability to decide. The IAEA's INIR Phase 1 assessment is tentatively set for 2027 and covers {{CTX_INIR_AREAS}} areas. On 26 October 2026, METI and EMA launched manpower schemes for public officers and selected companies. Government is clear that no decision on nuclear energy has been made.
**Source:** `r21`; `emanmdp`; `metilecture` para 50

### S23 | T2 | Community | Brief 8
**Title:** Acceptance has to be [[earned]]
**One idea:** Public understanding comes before any site decision.
**Large number:** None. This slide makes a statement with no figure.
**Visual:** None. Native panel: SNRSI public lectures now, then public understanding ahead of any site decision. The social-post screenshot `pptx:image13.png` is dropped (section 9 item 6).
**Slide text:** SNRSI public lectures are open to anyone, including students.
**Speaker note:** Public acceptance is a precondition. The Singapore Nuclear Research and Safety Institute runs public lectures open to anyone, including students. That outreach has to run long before any site decision.
**Source:** `r23`; `r24`; `r28`

### S24 | T2 | Location | Brief 9
**Title:** Proposed site: [[Western Island]]
**One idea:** The site is the team's proposal, with no official approval.
**Large number:** None. A banner reads "Team's proposal. No official approval."
**Visual:** `pptx:image14.jpeg`, the new western island map, with its Prime Minister's Office and Straits Times credit kept.
**Slide text:** Links to the island are unpriced. Land may not exist at the milestone dates.
**Speaker note:** Our proposed site is the new western island from the 2026 National Day Rally, linked to Jurong Island's industry. This is the team's proposal. It has no official approval, and the government page does not mention nuclear. Reclamation runs over coming decades, so land may not exist at our milestone dates, and the links to the island are unpriced.
**Source:** `govndr`; `britcham`; Decisions D05, X15

### S25 | T3 | Roadmap | Brief 10
**Title:** [[2027 to 2050]], every date illustrative
**One idea:** Four gated stages lead to potential operation by 2050.
**Large number:** `2050`, potential operation, illustrative, in the chip.
**Visual:** `fig:Figure_Roadmap_NUS_H2.png`, full figure, replacing `pptx:image15.png`.
**Slide text:** Illustrative dates.
**Speaker note:** Our roadmap runs from 2027 to 2050 in four stages, and every date is illustrative. National readiness comes first, then reactor and process-heat qualification, then project development and an investment decision, then construction. The 2039 start leans on the {{CTX_HTRPM_YEARS}} years HTR-PM took from construction to commercial operation. Each gate opens only if the stage before it delivers.
**Source:** `r12`; `r21`; `metilecture`

### S26 | T3 | Emissions boundary | Brief 11
**Title:** Where we [[draw the line]]
**One idea:** The dashed line in Figure 2 is the accounting boundary.
**Large number:** None. The dashed line is the message, drawn heavier in the figure.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, full figure, with a native overlay tracing the dashed boundary.
**Slide text:** Inside: reactor to ship offshore storage. Outside: PSA, upstream gas, construction.
**Speaker note:** Emissions now, and the boundary comes first. Inside the dashed line sit the reactor, helium loops, reformer, shift, capture, compression and ship offshore storage. The PSA sits outside, with its reject crossing back in. Upstream gas and construction are excluded. We count abatement at the Singapore stack. Storage abroad rests on the Singapore-Indonesia arrangement, and its accounting is unresolved.
**Source:** Decisions D06, X10; `r30`

### S27 | T2 | Emissions method | Brief 12
**Title:** Same hydrogen, [[two plants]]
**One idea:** Net abatement is a mass balance against a same-size unabated steam methane reforming plant.
**Large number:** `{{BASELINE_B_T_Y}}` t CO2/y baseline.
**Visual:** None. Native equation: net abatement equals baseline emissions less emissions to atmosphere.
**Slide text:** {{BASELINE_INL_KG_KG}} kg/kg on INL's basis, bounded by {{BASELINE_IEAGHG_KG_KG}} and {{BASELINE_CHO_KG_KG}}.
**Speaker note:** The method is a mass balance at equal hydrogen. The comparator is a same-size new-build unabated steam methane reforming plant at {{BASELINE_INL_KG_KG}} kilograms per kilogram, or {{BASELINE_B_T_Y}} tonnes a year. Cho's {{BASELINE_CHO_KG_KG}}, from {{CTX_CHO_PLANTS}} US plants, and IEAGHG's {{BASELINE_IEAGHG_KG_KG}} bound it. Net abatement is that baseline less what reaches the atmosphere, and hydrogen runs only in the {{AVAILABILITY_PCT|f0}} percent availability hours.
**Source:** `r10` Table 2, PDF p30; `cho2022`; `r5` Table 2; Decisions D05, D07

### S28 | T2 | Emissions result | Brief 13
**Title:** What still [[reaches the air]]
**One idea:** The only residual is the burned nitrogen-rich cut.
**Large number:** `{{ATM_B_T_Y}}` t CO2/y to atmosphere.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_STACK`.
**Slide text:** {{INTENSITY_B_KG_KG}} kg CO2 per kg H2. {{CAPTURED_B_T_Y}} t/y goes to storage.
**Speaker note:** Absorber slip recycles through the reformer, so the residual comes from one place: the burned nitrogen-rich cut at the cryogenic unit. That stack emits {{ATM_B_T_Y}} tonnes a year, {{INTENSITY_B_KG_KG}} kilograms per kilogram of hydrogen. Another {{CAPTURED_B_T_Y}} tonnes goes to storage.
**Source:** `Emissions_Ledger.csv`; Decision D04

### S29 | T2 | Emissions result | Brief 13
**Title:** Net abatement [[clears the target]]
**One idea:** The plant abates more than twice the target on this boundary.
**Large number:** `{{A_ATM_B_MT}}` Mt CO2e/y, `{{REDUCTION_B_PCT}}%` below unabated steam methane reforming.
**Visual:** None. Native bars: baseline, residual, target line.
**Slide text:** {{A_TS_BURDEN_B_MT}} Mt/y with the storage chain burden charged. Target {{TARGET_ABATEMENT_MT}} Mt/y: {{TARGET_ABATEMENT_RESULT}}.
**Speaker note:** Baseline less residual gives {{A_ATM_B_MT}} million tonnes a year, a {{REDUCTION_B_PCT}} percent cut against that unabated plant. That is {{ABATEMENT_MULTIPLE}} times the target of {{TARGET_ABATEMENT_MT}}. Charge the storage chain's burden and it reads {{A_TS_BURDEN_B_MT}}. For scale only, the result equals {{NDC_SHARE_LOW_PCT}} to {{NDC_SHARE_HIGH_PCT}} percent of Singapore's 2035 target level.
**Source:** `Emissions_Ledger.csv`; `nccsndc`; Decisions X9, X14

### S30 | T2 | Sensitivity | Brief 14
**Title:** [[Stress tests]] hold above target
**One idea:** Every case with working capture and recycle stays above the target.
**Large number:** `{{SENS_AVAILABILITY_0P65_A_T_Y|Mt}}` Mt/y at `{{SWEEP_AVAILABILITY_0P65|pct0}}%` availability.
**Visual:** New chart `CH2 Chart_Abatement_Sensitivity`, section 7.3.
**Slide text:** One input at a time. Corner runs carry no probability.
**Speaker note:** We stressed one input at a time. At {{SWEEP_CAPTURE_0P8|pct0}} percent per-pass capture we get {{SENS_CAPTURE_0P8_A_T_Y|Mt}}. At {{SWEEP_AVAILABILITY_0P65|pct0}} percent availability, {{SENS_AVAILABILITY_0P65_A_T_Y|Mt}}. Lifecycle burdens give {{A_LC_BURDENS_B_MT}}, and counting lost steam export gives {{A_SYSTEM_B_MT}}. All clear the target. These corner runs carry no probability, and {{AVAILABILITY_PCT|f0}} percent has no measured HTGR record behind it.
**Source:** `Sensitivity_Results.csv`; `Emissions_Ledger.csv`; `iaeanpt43` printed p38

### S31 | T2 | Sensitivity | Brief 14
**Title:** The assumption that [[matters most]]
**One idea:** Abatement rests on working capture and on recycle transferring to MEA.
**Large number:** `{{A_NORECYCLE_MT}}` Mt/y if recycle fails.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png`, crop `CROP_RECYCLE`, same frame as S09.
**Slide text:** Gas rises to {{SENS_TAIL_GAS_2_NG_MMSCFD}} MMSCFD. With capture offline, only {{NEVER_GENERATED_MT}} Mt/y remains.
**Speaker note:** One assumption matters most. If recycle fails to transfer from INL's Selexol case to MEA, and we hold hydrogen output, abatement falls to {{A_NORECYCLE_MT}} and gas rises to {{SENS_TAIL_GAS_2_NG_MMSCFD}}. That still clears the target. With capture offline, only the carbon that never forms counts, {{NEVER_GENERATED_MT}}, which misses it. So capture has to work.
**Source:** Decision X20; `Sensitivity_Results.csv` TAIL_GAS rows

### S32 | T2 | Market | Brief 15
**Title:** What the [[market]] compares us with
**One idea:** The carbon tax is the market's yardstick, and offtake is still open.
**Large number:** `S${{CTX_CARBON_TAX_NOW}}/t` carbon tax, 2026 to 2027.
**Visual:** None. Native strip of the tax path to the 2030 band, set against the BOAK cost at the same scale.
**Slide text:** A stated view of S${{CTX_CARBON_TAX_2030_LOW}} to S${{CTX_CARBON_TAX_2030_HIGH}}/t by 2030. No offtake is contracted.
**Speaker note:** What does the market compare us with? Singapore's carbon tax is S${{CTX_CARBON_TAX_NOW}} a tonne in 2026 and 2027, with a stated view to S${{CTX_CARBON_TAX_2030_LOW}} to S${{CTX_CARBON_TAX_2030_HIGH}} by 2030. Tax is a transfer, so it stays outside our cost metric. No hydrogen offtake is contracted, and there is competition: Air Liquide and Aster have an MoU on autothermal reforming with up to {{CTX_ATR_CAPTURE_PCT}} percent capture here.
**Source:** `nccstax`; `aster2025`

### S33 | T2 | Economics method | Brief 15
**Title:** Cost, [[rebuilt two ways]]
**One idea:** An annual ledger and a discounted cash flow feed one result.
**Large number:** `{{DCF_TEST_MAX_GAP_PCT}}%` largest gap between the methods.
**Visual:** `pptx:image33.png`, the abatement cost equation from old slide 21.
**Slide text:** Equal hydrogen output, {{DISCOUNT|pct0}}% real discount rate. Shared inputs, so agreement tests the arithmetic.
**Speaker note:** Now the cost. We use two methods at equal hydrogen output. An annual ledger adds each cost difference against the unabated steam methane reforming plant. A discounted cash flow runs both plants year by year at {{DISCOUNT|pct0}} percent. They differ by at most {{DCF_TEST_MAX_GAP_PCT}} percent. Both share one set of inputs, so that agreement tests our annualisation and does not validate the inputs.
**Source:** Build order section 2; `DCF_TEST_RESULT`

### S34 | T5 | Cost assumptions | Brief 16
**Title:** The inputs that [[drive cost]]
**One idea:** A handful of inputs carry the result, and each has a range.
**Large number:** `S${{GAS_PRICE_S_GJ|f0}}/GJ` gas, the largest credit.
**Visual:** None. Native table with low, central and high for gas, T&S and reactor capital.
**Slide text:** Gas {{GAS_LOW|f0}}, {{GAS_PRICE_S_GJ|f0}}, {{GAS_HIGH|f0}} S$/GJ. T&S {{TS_LOW|f1}}, {{TS_CENTRAL_S_T|f0}}, {{TS_HIGH|f0}} S$/t. Reactor {{OCC_LOW}}, {{OCC_CENTRAL_USD_KWT}}, {{OCC_HIGH}} US$/kWt.
**Speaker note:** These inputs drive the result. Gas is S${{GAS_PRICE_S_GJ|f0}} per gigajoule. Transport and storage is S${{TS_CENTRAL_S_T|f0}} a tonne, between S${{TS_LOW|f1}} and S${{TS_HIGH|f0}}, and no Singapore-Indonesia tariff has been published. Reactor capital is US${{OCC_CENTRAL_USD_KWT}} per kilowatt thermal. Capture capital is bottom-up on an aMDEA basis, so an MEA plant could cost more. The full table is in backup.
**Source:** `inlcost` PDF p47; `r5` Tables 2, 3; `fredgas`; `eria2024`; `metipq`; Decision D10, X18, X19

### S35 | T2b | Reactor cost | Brief 17
**Title:** Reactor capital: [[first plant]] against later plants
**One idea:** A first plant costs far more than a build between first and Nth of a kind.
**Large number:** `S${{REACTOR_CAPITAL_BOAK_N1P00_SM|f0}}M` BOAK and `S${{REACTOR_CAPITAL_FOAK_N1P00_SM|f0}}M` FOAK.
**Visual:** `pptx:image39.png`, the Xe-100 render from old slide 26, at no more than 3.2 in wide, captioned "Analogue only".
**Slide text:** Generic HTGR. Xe-100 is an analogue only, with no vendor quotation.
**Speaker note:** We price a generic {{ROT}} degree HTGR on INL's recipe, charged at {{R_RATED_MWTH|f0}} megawatts. BOAK is INL's term for a build between a first and an Nth of a kind, and it gives S${{REACTOR_CAPITAL_BOAK_N1P00_SM|f0}} million. A first of a kind, FOAK, carries a {{FOAK_CENTRAL|f1}} times premium, S${{REACTOR_CAPITAL_FOAK_N1P00_SM|f0}} million, and that is what a single plant would be. The Xe-100 shown is an analogue only, with a {{CTX_COMMERCIAL_ROT_C}} degree outlet and no vendor quotation.
**Source:** `inlcost` Table 1, PDF p5, p47; `xenergy`; Decisions D09, D23, X4

### S36 | T3 | Reactor cost | Brief 17
**Title:** The [[scale law]] moves everything
**One idea:** Three size treatments print at equal weight, because none is settled.
**Large number:** `S${{AC_L5_BOAK_25Y}}`, `S${{AC_L5_BOAK_25Y_N0P60}}` and `S${{AC_L5_BOAK_25Y_N0P31}}` per tonne, equal size.
**Visual:** `fig:Chart_Cost_Grid_L5.png`, full figure.
**Slide text:** n = {{SWEEP_SIZE_EXPONENT_1|f1}} INL recipe, n = {{SWEEP_SIZE_EXPONENT_0P6|f1}} team assumption, n = {{SWEEP_SIZE_EXPONENT_0P310698|f2}} ratio of two FOAK estimates.
**Speaker note:** How a {{REACTOR_COST_REF_SIZE|f0}} megawatt estimate scales down to {{R_RATED_MWTH|f0}} is uncertain, so we print three treatments at equal size. n of {{SWEEP_SIZE_EXPONENT_1|f1}} is INL's recipe and our central case. n of {{SWEEP_SIZE_EXPONENT_0P6|f1}} is a team assumption. n of {{SWEEP_SIZE_EXPONENT_0P310698|f2}} is the ratio of two first-of-a-kind estimates. At BOAK and {{LIVES_SHORT|f0}} years, full cost runs from S${{AC_L5_BOAK_25Y}} to S${{AC_L5_BOAK_25Y_N0P31}} a tonne.
**Source:** `inlcost` PDF p25, p27, p39, p45; Decisions D09, X5

### S37 | T2b | Money | Brief 18
**Title:** Hydrogen cost, [[both plants]]
**One idea:** NUS-H2 hydrogen costs more than unabated hydrogen, and this is the premium.
**Large number:** `S${{LCOH_SMR_25Y}}/kg` unabated steam methane reforming and `S${{LCOH_NUSH2_BOAK_25Y}}/kg` NUS-H2, BOAK, `{{LIVES_SHORT|f0}}` y.
**Visual:** None. Native paired bars of LCOH for both plants at BOAK and FOAK, `{{LIVES_SHORT|f0}}` and `{{LIVES_LONG|f0}}` y.
**Slide text:** FOAK S${{LCOH_NUSH2_FOAK_25Y}}/kg. At {{LIVES_LONG|f0}} y: BOAK S${{LCOH_NUSH2_BOAK_60Y}}/kg, FOAK S${{LCOH_NUSH2_FOAK_60Y}}/kg.
**Speaker note:** Here is the money. Unabated hydrogen costs S${{LCOH_SMR_25Y}} a kilogram. Over {{LIVES_SHORT|f0}} years NUS-H2 costs S${{LCOH_NUSH2_BOAK_25Y}} at BOAK and S${{LCOH_NUSH2_FOAK_25Y}} at FOAK. Over {{LIVES_LONG|f0}} years, S${{LCOH_NUSH2_BOAK_60Y}} and S${{LCOH_NUSH2_FOAK_60Y}}. That is a BOAK premium of S${{LCOH_PREMIUM_BOAK_25Y}} a kilogram, which a buyer would have to carry. Both plants carry the same reformer island from IEAGHG, on the assumption that reformer differences cancel.
**Source:** `Economic_Ledger.csv` DCF rows; `r5` Tables 3, 4, PDF p17, p18

### S38 | T3 | Abatement cost | Brief 18
**Title:** Every real cost is [[on the ladder]]
**One idea:** The ladder shows what each item adds, with L3 as a named step.
**Large number:** `S${{AC_L5_BOAK_25Y}}/t`, rebuilt full cost, in the chip.
**Visual:** `fig:Chart_Cost_Ledger_25y.png`, full figure.
**Slide text:** BOAK, {{LIVES_SHORT|f0}} y, n = {{SWEEP_SIZE_EXPONENT_1|f1}}, {{R_RATED_MWTH|f0}} MWth charged. Named step L3: S${{AC_L3_BOAK_25Y}}/t.
**Speaker note:** This ladder shows what each line adds at BOAK over {{LIVES_SHORT|f0}} years. Gas saved pays back S${{LINE_GAS_BOAK_25Y_N1P00|abs}} million a year. Reactor capital costs S${{LINE_REACTOR_CAP_BOAK_25Y_N1P00}} million. Stop at the named step L3 and you read S${{AC_L3_BOAK_25Y}} a tonne. Construction interest and lost steam export are real costs of this plant too, so the full figure is S${{AC_L5_BOAK_25Y}}.
**Source:** `Economic_Ledger.csv`; Decision D08

### S39 | T2 | Abatement cost | Brief 18
**Title:** Levelised abatement cost: [[target {{TARGET_COST_RESULT}}]]
**One idea:** No central case reaches the target, by either method.
**Large number:** Four numerals at equal size: `S${{LAC_DCF_BOAK_25Y}}`, `S${{LAC_DCF_BOAK_60Y}}`, `S${{LAC_DCF_FOAK_25Y}}`, `S${{LAC_DCF_FOAK_60Y}}` per tonne.
**Visual:** None. The four numerals fill the panel as a grid of stage against life, with the ledger values beneath in small type: {{AC_L5_BOAK_25Y}}, {{AC_L5_BOAK_60Y}}, {{AC_L5_FOAK_25Y}}, {{AC_L5_FOAK_60Y}}.
**Slide text:** {{TARGET_COST_N_PASS}} of {{TARGET_COST_N_CENTRAL}} central cases reach S${{TARGET_COST_S_T|f0}}/t.
**Speaker note:** Here is the levelised abatement cost from the cash flow. BOAK is S${{LAC_DCF_BOAK_25Y}} at {{LIVES_SHORT|f0}} years and S${{LAC_DCF_BOAK_60Y}} at {{LIVES_LONG|f0}}. FOAK is S${{LAC_DCF_FOAK_25Y}} and S${{LAC_DCF_FOAK_60Y}}. Against S${{TARGET_COST_S_T|f0}}, the target is {{TARGET_COST_RESULT}}: {{TARGET_COST_N_PASS}} of {{TARGET_COST_N_CENTRAL}} central cases pass.
**Source:** `Economic_Ledger.csv`; Decisions D08, X4

### S40 | T5 | Abatement cost | Brief 18
**Title:** What [[S${{TARGET_COST_S_T|f0}}/t]] would take
**One idea:** No single lever closes the gap inside its tested range.
**Large number:** `US${{LEVER_OCC_L5_BOAK_25Y_USD_KWT}}/kWt` needed, against INL's low of `US${{OCC_LOW}}/kWt`.
**Visual:** None. Native table of levers, needed value at `{{LIVES_SHORT|f0}}` and `{{LIVES_LONG|f0}}` y, tested range.
**Slide text:** Gas {{LEVER_GAS_L5_BOAK_25Y_S_GJ}}, {{LEVER_GAS_L5_BOAK_60Y_S_GJ}} S$/GJ, tested to {{GAS_HIGH|f0}}. Reactor {{LEVER_OCC_L5_BOAK_25Y_USD_KWT}}, {{LEVER_OCC_L5_BOAK_60Y_USD_KWT}} US$/kWt, tested from {{OCC_LOW}}. T&S {{LEVER_TS_L5_BOAK_25Y_S_T}}, {{LEVER_TS_L5_BOAK_60Y_S_T}} S$/t, tested from {{TS_LOW|f1}}. Sized to need: {{AC_L5_BOAK_25Y_N1P00_NEED}}, {{AC_L5_BOAK_60Y_N1P00_NEED}} S$/t reached.
**Speaker note:** What would S${{TARGET_COST_S_T|f0}} take? One lever at a time, at BOAK and {{LIVES_SHORT|f0}} years, we are S${{EXCESS_L5_BOAK_25Y_SM_Y}} million a year over the ceiling. Gas would have to reach S${{LEVER_GAS_L5_BOAK_25Y_S_GJ}} per gigajoule, {{NEW_GAS_LEVER_MULTIPLE}} times our central price. Reactor cost would have to fall to US${{LEVER_OCC_L5_BOAK_25Y_USD_KWT}}, under INL's low of {{OCC_LOW}}. Transport and storage would have to go negative. Sizing to need reaches only S${{AC_L5_BOAK_25Y_N1P00_NEED}}. No single lever closes it.
**Source:** `LEVER_` and `EXCESS_` tokens; report section 4.7

### S41 | T4 | Abatement cost | Brief 18
**Title:** [[S${{TARGET_MATURE_COST|f0}}/t]]: S${{TARGET_COST_S_T|f0}}/t target met for a 10th-of-a-kind plant over {{LIVES_LONG|f0}} years with gas at S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ
**One idea:** A mature plant over a long life meets the target when gas is dear, and the base full cost stays on the same slide.
**Large number:** `S${{MATURE_60Y_L5_AT_PRINTED}}/t` on the ledger and `S${{MATURE_60Y_DCF_AT_PRINTED}}/t` by cash flow, at `S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ` against the central `S${{GAS_PRICE_S_GJ|f0}}/GJ`.
**Visual:** None. Native panel with the two numerals and the two gas prices, and a card on the right with the base full cost, BOAK, {{LIVES_SHORT|f0}} y, n = {{SWEEP_SIZE_EXPONENT_1|f1}}, {{R_RATED_MWTH|f0}} MWth charged, by both methods.
**Slide text:** Higher gas prices lower the abatement cost, because NUS-H2 burns less gas than an unabated steam methane reforming plant of the same output.
**Speaker note:** A 10th-of-a-kind plant run for {{LIVES_LONG|f0}} years meets the S${{TARGET_COST_S_T|f0}} target at S${{TARGET_MATURE_COST|f0}} a tonne when gas costs S${{MATURE_60Y_GAS_PRINTED_S_GJ}} a gigajoule: S${{MATURE_60Y_L5_AT_PRINTED}} on the ledger and S${{MATURE_60Y_DCF_AT_PRINTED}} by cash flow. Dearer gas lowers the cost because NUS-H2 burns less gas than the unabated plant. The base stays on screen: BOAK over {{LIVES_SHORT|f0}} years costs S${{AC_L5_BOAK_25Y}} and S${{LAC_DCF_BOAK_25Y}}, and misses.
**Source:** `Sensitivity_Results.csv` MATURE_60Y rows; `Economic_Ledger.csv`; `inlcost` PDF p46

### S41b | T3 | Growth chart | Brief 18
**Title:** How the cost [[steps down]] to the target
**One idea:** Three conditions, each an engine run, take the base full cost under the target.
**Large number:** `S${{STEPDOWN_1_L5}}/t` at the base and `S${{STEPDOWN_4_L5}}/t` after the three steps.
**Visual:** `fig:Chart_Cost_Stepdown.png`, full figure, built from the STEPDOWN rows, with the S${{TARGET_COST_S_T|f0}}/t target line and the S${{TARGET_MATURE_COST|f0}}/t point. It takes the place of the recording's growth chart.
**Slide text:** The reactor steps cut S${{STEPDOWN_2_L5_DELTA|abs}}/t for a 10th-of-a-kind unit and S${{STEPDOWN_3_L5_DELTA|abs}}/t for a {{LIVES_LONG|f0}} year life. Gas at S${{STEPDOWN_4_GAS_S_GJ}}/GJ then cuts S${{STEPDOWN_4_L5_DELTA|abs}}/t.
**Speaker note:** Every bar is an engine run, ledger first. From the base of S${{STEPDOWN_1_L5}}, a 10th-of-a-kind reactor and a {{LIVES_LONG|f0}} year life take off S${{STEPDOWN_2_L5_DELTA|abs}} and S${{STEPDOWN_3_L5_DELTA|abs}}. Gas at S${{STEPDOWN_4_GAS_S_GJ}} then takes off S${{STEPDOWN_4_L5_DELTA|abs}}, to S${{STEPDOWN_4_L5}}, under the line at the S${{TARGET_MATURE_COST|f0}} point. These are conditions on the base, which stays our reference.
**Source:** `Sensitivity_Results.csv` STEPDOWN rows; `inlcost` PDF p46

### S42 | T4 | Feasibility | Brief 19
**Title:** Overall [[feasibility]]
**One idea:** Carbon works on this boundary. Cost meets the target only for a mature plant.
**Large number:** Abatement `{{TARGET_ABATEMENT_RESULT}}`, cost `{{TARGET_COST_RESULT}}` at the base, in equal type.
**Visual:** None. Four native cards: Carbon, Cost, Technology, Site and policy.
**Slide text:** Carbon: {{A_ATM_B_MT}} Mt/y. Cost: S${{AC_L5_BOAK_60Y}} to S${{AC_L5_FOAK_25Y}}/t at central inputs, S${{TARGET_MATURE_COST|f0}}/t for a 10th-of-a-kind plant over {{LIVES_LONG|f0}} y with gas at S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ. Technology: IHX unqualified. Site and policy: no decision.
**Speaker note:** So is it feasible? On carbon, yes, on this boundary. On cost, the target is {{TARGET_COST_RESULT}} at central inputs and met only in the mature case. On technology, the plant rests on an IHX nobody has built at this duty. On siting, it rests on land and a national decision that do not yet exist. We read NUS-H2 as a credible carbon route whose cost closes only for a mature plant, and overseas demonstration has to come first.
**Source:** Sections above; report section 5

### S43 | T2 | Limits | Brief 19
**Title:** What this model has [[yet to show]]
**One idea:** Part of the checks now stand on independent data, and some of them fail.
**Large number:** `{{VAL_N_INDEPENDENT}}` of `{{VAL_N_TOTAL}}` checks are independent of the source data.
**Visual:** None. Native list in the left panel: recycle transfer to MEA, ship liquefaction load, capture capital basis, SRD on syngas, HTGR availability, site links, backup firing capital, offshore accounting.
**Slide text:** {{VAL_N_PASS}} rows pass and {{VAL_N_FAIL}} fail, and the failures stay recorded. {{STATUS_ROWS_NOT_RUN}} results rows did not run.
**Speaker note:** Now the limits of our own numbers. Many results are reduced-order balances anchored on INL cases. Of {{VAL_N_TOTAL}} validation rows, {{VAL_N_INDEPENDENT}} are independent of the source data. {{VAL_N_PASS}} pass and {{VAL_N_FAIL}} fail, and we keep the failures on the record. The list on screen is what the next stage of work has to close.
**Source:** `Validation_Results.csv`; `docs/FINAL_DESIGN_DECISIONS.md` sections 4, 9, X8, X16, X18

### S44 | T2 | People | Brief added
**Title:** The [[author]]
**One idea:** The author's name, as the recording's people slide without faces.
**Large number:** None. The name takes the numeral's place.
**Visual:** None. The name on the gradient.
**Slide text:** Wee Zhern Ray.
**Speaker note:** NUS-H2 is presented by its author, Wee Zhern Ray.
**Source:** 

### S45 | T4 | Close | Brief 20
**Title:** NUS-H2 [[in one slide]]
**One idea:** One recap in columns, left up through questions.
**Large number:** One numeral in each column: `{{R_RATED_MWTH|f0}}` MWth, `{{A_ATM_B_MT}}` Mt/y, `S${{AC_L5_BOAK_25Y}}/t`, and the year `2050`.
**Visual:** `fig:Figure_2_NUS_H2_210MWth.png` as a thumbnail in the first column.
**Slide text:** What: helium-heated steam methane reforming with MEA capture. Carbon: {{REDUCTION_B_PCT}}% cut, target {{TARGET_ABATEMENT_RESULT}}. Cost: BOAK S${{AC_L5_BOAK_25Y}}/t over {{LIVES_SHORT|f0}} y, target {{TARGET_COST_RESULT}} at the base, met at S${{TARGET_MATURE_COST|f0}}/t for a 10th-of-a-kind plant over {{LIVES_LONG|f0}} y with gas at S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ. Next: IHX qualification and a design-specific safety case, illustrative path to 2050.
**Speaker note:** To close. NUS-H2 heats a reformer with a {{R_RATED_MWTH|f0}} megawatt HTGR and captures CO2 with MEA. It abates {{A_ATM_B_MT}} million tonnes a year, so the carbon target is {{TARGET_ABATEMENT_RESULT}}. Abatement costs S${{AC_L5_BOAK_25Y}} a tonne at BOAK and S${{AC_L5_FOAK_25Y}} at FOAK, so the cost target is {{TARGET_COST_RESULT}} at the base. A 10th-of-a-kind plant over {{LIVES_LONG|f0}} years meets it at S${{TARGET_MATURE_COST|f0}} with gas at S${{MATURE_60Y_GAS_PRINTED_S_GJ}}. The next steps are IHX qualification overseas and a design-specific safety case. Thank you.
**Source:** Sections above

## 5. Backup slides

### B01 | T5 | Reference Frame | Brief 21
**Title:** Reference Frame: [[variables]]
**One idea:** Every symbol in the talk, with its central value.
**Large number:** None. Table slide.
**Visual:** None. Native table.
**Slide text:**
| Symbol | Definition | Units | Central |
|---|---|---|---|
| $\dot m_{H_2}$ | Annual hydrogen | t/y | {{H2_B_T_Y}} |
| $a$ | Availability | % | {{AVAILABILITY_PCT|f0}} |
| $T_{ROT}$ | Reactor outlet | C | {{ROT}} |
| $T_{He}$ | Secondary helium supply | C | {{T_HE_SUPPLY}} |
| $R_{rated}$ | Reactor rating | MWth | {{R_RATED_MWTH|f0}} |
| $R_{need}$ | Reactor needed | MWth | {{R_NEEDED_B_MWTH}} |
| $Q_{proc}$ | Process heat | MWth | {{Q_PROCESS_B_MWTH}} |
| $E_{site}$ | Site electrical load | MWe | {{E_TOTAL_B_MWE}} |
| $e_{base}$ | Baseline intensity | kg CO2/kg H2 | {{BASELINE_INL_KG_KG}} |
| $E_{base}$ | Baseline emissions | t/y | {{BASELINE_B_T_Y}} |
| $E_{atm}$ | Emissions to atmosphere | t/y | {{ATM_B_T_Y}} |
| $A$ | Net abatement | Mt/y | {{A_ATM_B_MT}} |
| $\eta_{pass}$ | Per-pass capture | % | {{PER_PASS_CAPTURE_PCT|f0}} |
| $\eta_{plant}$ | Plant capture | % | {{PLANT_CAPTURE_B_PCT}} |
| $q_{reb}$ | Specific reboiler duty | GJ/t | {{SRD_CENTRAL}} |
| $r$ | Real discount rate | % | {{DISCOUNT|pct0}} |
| $n$ | Reactor size exponent | none | {{SWEEP_SIZE_EXPONENT_1|f1}} |
| $L$ | Learning rate per doubling | % | {{LEARNING|pct0}} |
**Speaker note:** Backup. The symbols used in the talk, with today's central values.
**Source:** `Final_Design_Basis.csv`

### B02 | T5 | Reference Frame | Brief 21
**Title:** Reference Frame: [[equations]]
**One idea:** The equations behind the headline numbers, with their ranges.
**Large number:** None. Equation list.
**Visual:** None. Native equations, numbered.
**Slide text:** (1) $A = E_{base} - E_{atm}$. (2) $E_{base} = e_{base}\,\dot m_{H_2}$. (3) $\eta_{plant} = m_{cap} / (m_{cap} + E_{atm})$. (4) $R_{need} = Q_{IHX} + Q_{loss} + Q_{power} - W_{shaft}$. (5) $Q = \dot m\,c_p\,\Delta T$. (6) $\mathrm{CRF} = r / (1 - (1 + r)^{-N})$. (7) $\mathrm{AC} = \Delta C / A$. (8) $\mathrm{LCOH}_x = \mathrm{NPV}(c_x) / \mathrm{NPV}(m_{H_2})$. (9) $\mathrm{LAC} = \mathrm{NPV}(\Delta c) / \mathrm{NPV}(A)$. (10) $C(S) = C_{ref}\,(S / S_{ref})^n$. (11) $C_N = C_1\,(1 - L)^{\log_2 N}$. Ranges: SRD {{SRD_LOW}} to {{SRD_HIGH}} GJ/t, life {{LIVES_SHORT|f0}} to {{LIVES_LONG|f0}} y, {{FOAK_LOW|f1}} to {{FOAK_HIGH|f1}} FOAK premium. CRF at {{LIVES_SHORT|f0}} y is {{CRF_25Y}} and at {{LIVES_LONG|f0}} y is {{CRF_60Y}}.
**Speaker note:** Backup. These are the equations behind the headline numbers. Learning from FOAK and from BOAK are both published, and each use states its route.
**Source:** `math/` derivations; `inlcost` PDF p46, p47

### B03 | T5 | Comparator | Brief 21
**Title:** Steam methane reforming plus CCS [[comparator]]
**One idea:** One labelled comparator, indicative only.
**Large number:** `S${{RES_COMPARATOR_CAC_TS60_S_T}}/t` IEAGHG Case 3 at matched T&S.
**Visual:** None. Native table.
**Slide text:**
| Item | NUS-H2 | IEAGHG Case 3 |
|---|---|---|
| Capture route | MEA on shifted syngas | MEA on flue gas |
| Overall capture | {{PLANT_CAPTURE_B_PCT}}% | {{COMPARATOR_CAPTURE_PCT}}% |
| Baseline, kg/kg | {{BASELINE_INL_KG_KG}} | {{BASELINE_IEAGHG_KG_KG}} |
| Abatement, Mt/y | {{A_ATM_B_MT}} | {{COMPARATOR_A_MT}} |
| Cost as published, S$/t | not applicable | {{RES_COMPARATOR_CAC_TS23P2_S_T}} |
| Cost at T&S S${{TS_CENTRAL_S_T|f0}}/t, S$/t | BOAK {{AC_L5_BOAK_25Y}}, FOAK {{AC_L5_FOAK_25Y}} | {{RES_COMPARATOR_CAC_TS60_S_T}} |
Indicative. Gas price and availability differ from this ledger.
**Speaker note:** Backup. IEAGHG Case 3 at matched transport and storage sits above our BOAK case and below our FOAK case, so NUS-H2 competes only once a reactor is past first of a kind. Its gas price and availability differ from this ledger, so read this as indicative.
**Source:** `r5` Tables 2 to 5; Decisions D11, X17

### B04 | T3 | Cost ladder | Brief 21
**Title:** Cost ladder at [[{{LIVES_LONG|f0}} y]]
**One idea:** The same ladder over the longer life.
**Large number:** `S${{AC_L5_BOAK_60Y}}/t`, in the chip.
**Visual:** `fig:Chart_Cost_Ledger_60y.png`, full figure.
**Slide text:** BOAK, {{LIVES_LONG|f0}} y, n = {{SWEEP_SIZE_EXPONENT_1|f1}}, {{R_RATED_MWTH|f0}} MWth charged.
**Speaker note:** Backup. The same ladder over {{LIVES_LONG|f0}} years. Longer life changes only the reactor-linked lines.
**Source:** `Economic_Ledger.csv`

### B05 | T5 | Cost ladder | Brief 21
**Title:** Cost ladder, [[every level]]
**One idea:** L0 to L5 for both stages and both lives, with the ceiling.
**Large number:** `S${{CEILING_SM_Y}}M/y` ceiling at the cost target.
**Visual:** None. Native table.
**Slide text:**
| S$/t | BOAK {{LIVES_SHORT|f0}} y | BOAK {{LIVES_LONG|f0}} y | FOAK {{LIVES_SHORT|f0}} y | FOAK {{LIVES_LONG|f0}} y |
|---|---|---|---|---|
| L0 gas, capital, capture, T&S | {{AC_L0_BOAK_25Y}} | {{AC_L0_BOAK_60Y}} | {{AC_L0_FOAK_25Y}} | {{AC_L0_FOAK_60Y}} |
| L1 reactor O&M and fuel | {{AC_L1_BOAK_25Y}} | {{AC_L1_BOAK_60Y}} | {{AC_L1_FOAK_25Y}} | {{AC_L1_FOAK_60Y}} |
| L2 IHX and loop | {{AC_L2_BOAK_25Y}} | {{AC_L2_BOAK_60Y}} | {{AC_L2_FOAK_25Y}} | {{AC_L2_FOAK_60Y}} |
| L3 licensing, decommissioning | {{AC_L3_BOAK_25Y}} | {{AC_L3_BOAK_60Y}} | {{AC_L3_FOAK_25Y}} | {{AC_L3_FOAK_60Y}} |
| L4 interest, power island, recycle train | {{AC_L4_BOAK_25Y}} | {{AC_L4_BOAK_60Y}} | {{AC_L4_FOAK_25Y}} | {{AC_L4_FOAK_60Y}} |
| L5 lost steam, avoided grid power | {{AC_L5_BOAK_25Y}} | {{AC_L5_BOAK_60Y}} | {{AC_L5_FOAK_25Y}} | {{AC_L5_FOAK_60Y}} |
| DCF levelised abatement cost | {{LAC_DCF_BOAK_25Y}} | {{LAC_DCF_BOAK_60Y}} | {{LAC_DCF_FOAK_25Y}} | {{LAC_DCF_FOAK_60Y}} |
**Speaker note:** Backup. Each row adds the named items. L5 and the cash flow are the headline, and L3 stays as a named step.
**Source:** `Economic_Ledger.csv`

### B06 | T5 | Assumptions | Brief 21
**Title:** Assumptions: [[plant]]
**One idea:** The plant basis, with each value's status.
**Large number:** None. Table slide.
**Visual:** None. Native table.
**Slide text:**
| Item | Value | Status |
|---|---|---|
| Reactor outlet, IHX approach | {{ROT}} C, {{IHX_APPROACH_K}} K | INL assumption, {{IHX_APPROACH_K_HIGH}} K tested |
| Secondary helium | {{T_HE_SUPPLY}} C out, {{T_HE_RETURN}} C back | INL |
| Reformer outlet | {{SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C}} C | INL equilibrium assumption |
| Power conversion | {{ETA_POWER|pct1}}% | INL, {{ETA_POWER_ALT|pct1}}% tested |
| MEA | {{W_MEA|pct0}} wt%, rich {{LOADING_RICH|f2}}, lean {{LOADING_LEAN|f2}} | Project assumption |
| Absorber, stripper | {{P_ABSORBER|f1}} bar, {{P_STRIPPER|f0}} bar | INL, amine specification |
| SRD | {{SRD_CENTRAL}} GJ/t, {{SRD_LOW}} to {{SRD_HIGH}} | Assumed from flue gas pilots |
| Availability | {{AVAILABILITY_PCT|f0}}% | Assumed, no measured HTGR record |
**Speaker note:** Backup. Each plant value with its status. INL values are published, and the rest are assumptions we label as such.
**Source:** `Final_Design_Basis.csv`; `r8`; Decisions D26 to D33

### B07 | T5 | Assumptions | Brief 21
**Title:** Assumptions: [[cost]]
**One idea:** Every cost input with its range.
**Large number:** None. Table slide.
**Visual:** None. Native table.
**Slide text:**
| Input | Low | Central | High | Unit |
|---|---|---|---|---|
| Reactor overnight capital | {{OCC_LOW}} | {{OCC_CENTRAL_USD_KWT}} | {{OCC_HIGH}} | US$2019/kWt |
| FOAK premium | {{FOAK_LOW|f1}} | {{FOAK_CENTRAL|f1}} | {{FOAK_HIGH|f1}} | x BOAK |
| Reactor O&M and fuel | {{OPEX_LOW|f0}} | {{OPEX_CENTRAL|f0}} | {{OPEX_HIGH|f0}} | US$2019/MWth-h |
| Gas | {{GAS_LOW|f0}} | {{GAS_PRICE_S_GJ|f0}} | {{GAS_HIGH|f0}} | S$/GJ |
| T&S | {{TS_LOW|f1}} | {{TS_CENTRAL_S_T|f0}} | {{TS_HIGH|f0}} | S$/t |
| Extra licensing | {{LICENCE_LOW|f0}} | {{LICENCE_CENTRAL|f0}} | {{LICENCE_CENTRAL_HIGH|f1}} | US$M |
| Decommissioning | {{DECOM_LOW|f0}} | {{DECOM_CENTRAL|f0}} | {{DECOM_HIGH|f0}} | US$2022/kWe |
Discount {{DISCOUNT|pct0}}%. FX S${{FX|f4}}/US$. Construction {{CONSTRUCTION_YEARS|f1}} y.
**Speaker note:** Backup. Every cost input with its range. Several lines are labelled proxies, including IHX and licensing, which rest on primary sources not verified here.
**Source:** `inlcost` Table 1; `r5`; `fredfx`; `fredgas`; `eria2024`; Decision D10

### B08 | T5 | Assumptions | Brief 21
**Title:** Cost [[sensitivities]]
**One idea:** One input at a time around BOAK, {{LIVES_SHORT|f0}} y, on the full cost.
**Large number:** None. Table slide.
**Visual:** None. Native table.
**Slide text:**
| Change | Full cost, S$/t | Change | Full cost, S$/t |
|---|---|---|---|
| T&S S${{SWEEP_TS_23P2|f1}} | {{SENS_TS_23P2_AC_L5_25}} | T&S S${{SWEEP_TS_112|f0}} | {{SENS_TS_112_AC_L5_25}} |
| Gas S${{SWEEP_GAS_10|f0}}/GJ | {{SENS_GAS_10_AC_L5_25}} | Gas S${{SWEEP_GAS_20|f0}}/GJ | {{SENS_GAS_20_AC_L5_25}} |
| Availability {{SWEEP_AVAILABILITY_0P65|pct0}}% | {{SENS_AVAILABILITY_0P65_AC_L5_25}} | Availability {{SWEEP_AVAILABILITY_0P9|pct0}}% | {{SENS_AVAILABILITY_0P9_AC_L5_25}} |
| Reactor capital low | {{SENS_REACTOR_CAPITAL_1456_AC_L5_25}} | Reactor capital high | {{SENS_REACTOR_CAPITAL_2548_AC_L5_25}} |
| Gas saving on Ahn and Lee, carbon held | {{SENS_GAS_SAVING_AHN_0P188038_B_REC_F_CARBON_HELD_AC_L5_25}} | Capture capital on MEA flue gas | {{SENS_CAPTURE_CAPITAL_5P09395_AC_L5_25}} |
| Sized to need | {{AC_L5_BOAK_25Y_N1P00_NEED}} | No recycle | {{SENS_TAIL_GAS_2_AC_L5_25}} |
**Speaker note:** Backup. Reactor cost treatment and the transfer of recycle move the result most. SRD moves the spare at a fixed rating and leaves cost almost unchanged. Corner runs carry no probability.
**Source:** `Sensitivity_Results.csv`; report section 4.5

### B09 | T5 | References | Brief 21
**Title:** [[References]]: Singapore and policy
**One idea:** Sources for the Singapore context.
**Large number:** None. Reference list.
**Visual:** None. Native list, 16 pt, hyperlinks live.
**Slide text:** Text of `r1`, `r2`, `r3`, `r12`, `r13`, `r21`, `r23`, `r24`, `r28`, `r30`, `emanmdp`, `metilecture`, `govndr`, `britcham`, `nccstax`, `nccsndc`, `airliquide2008`, `airliquide2011`, `aster2025`, `metipq`, from `report/_build/refs.csv`.
**Speaker note:** Backup. Sources for the Singapore context.
**Source:** `report/_build/refs.csv`

### B10 | T5 | References | Brief 21
**Title:** [[References]]: engineering and cost
**One idea:** Sources for the engineering and cost basis.
**Large number:** None. Reference list.
**Visual:** None. Native list, 16 pt, hyperlinks live.
**Slide text:** Text of `r5`, `r6`, `r8`, `r10`, `r11`, `r17`, `cho2022`, `ahn2026`, `inlcost`, `iaeanpt43`, `ieaghg2022`, `wnn2023`, `xenergy`, `eria2024`, `fredfx`, `fredgas`, from `report/_build/refs.csv`, plus `d_tecdoc1236`, `d_tecdoc1382`, `d_tochio2011`, `d_sato2006`, `d_nelson2011`, `d_inltriso`, `d_nuclearpower`, `d_outram` from section 6.3.
**Speaker note:** Backup. Sources for the engineering and cost basis, with OUTRAM PARK credited as a reference.
**Source:** `report/_build/refs.csv`; section 6.3

## 6. Token gaps and context constants

### 6.1 Tokens to add

Arithmetic on existing tokens. Add them to `report/_build/resolve_numbers.py`, or to the deck builder's own derived step, under these names. The checker evaluates each formula.

| Token | Formula | Decimals | Unit | Slides |
|---|---|---|---|---|
| NEW_PINCH_CENTRAL_K | ROT - IHX_APPROACH_K - SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C | 0 | K | S20 |
| NEW_PINCH_IAEA_K | ROT - IHX_APPROACH_K_HIGH - SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C | 0 | K | S20 |
| NEW_IHX_DUTY_RATIO | EB_B_REC_F_IHX_DUTY / CTX_HTTR_IHX_MW | 0 | x | S20 |
| NEW_GAS_LEVER_MULTIPLE | LEVER_GAS_L5_BOAK_25Y_S_GJ / GAS_PRICE_S_GJ | 1 | x | S40 |
| NEW_UNITS_FOAK_TO_BOAK | 2 ** (log(FOAK_CENTRAL) / -log(1 - LEARNING)) | 0 | units | Q&A sheet |

### 6.2 Published context constants

These are published facts behind the story. None is a model result and none has a token, so they cannot reach the deck through `results/`. Recommended home: `engine/data/parameters.toml` as published inputs, so that they reach `numbers.json` like any other. Until then the builder reads them from `presentation/deck_context.csv`, generated from this table, with no other copy.

| Constant | Value | Meaning | Source |
|---|---|---|---|
| CTX_GAS_POWER_PCT | 94 | Natural gas share of electricity fuel, 2024 | `r1`, `r2` |
| CTX_H2_POWER_PCT | 50 | Hydrogen's possible share of power needs by 2050 | `r3` |
| CTX_INL_GAS_CUT_PCT | 30.9 | INL gas reduction at 925 C ROT | `r8` PDF p5 |
| CTX_INL_OPT_ROT_C | 875 | INL optimum ROT for steam methane reforming | `d_nelson2011` PDF p9, p21 |
| CTX_TEST_REACTOR_MAX_MW | 46 | Largest reactor to reach 950 C, AVR | `iaeanpt43` printed p23 |
| CTX_COMMERCIAL_ROT_C | 750 | HTR-PM and Xe-100 outlet | `wnn2023`, `xenergy` |
| CTX_TRISO_LIMIT_C | 1600 | Accident limit for TRISO fuel | `d_tecdoc1236` printed p24, p104 |
| CTX_HE_MEASURED_C | 859 | Highest measured IHX secondary temperature | `iaeanpt43` printed p14 |
| CTX_HTTR_IHX_MW | 10 | HTTR IHX duty | `d_tochio2011` PDF p4, p8 |
| CTX_INIR_AREAS | 19 | Areas covered by INIR Phase 1 | `r21` |
| CTX_HTRPM_YEARS | 11 | HTR-PM construction start to commercial operation | `r12` |
| CTX_CHO_PLANTS | 33 | US steam methane reforming facilities in Cho | `cho2022` |
| CTX_CARBON_TAX_NOW | 45 | Carbon tax, S$/t, 2026 to 2027 | `nccstax` |
| CTX_CARBON_TAX_2030_LOW | 50 | Stated 2030 band, low | `nccstax` |
| CTX_CARBON_TAX_2030_HIGH | 80 | Stated 2030 band, high | `nccstax` |
| CTX_ATR_CAPTURE_PCT | 99 | Stated capture, Air Liquide and Aster MoU | `aster2025` |

### 6.3 References the report list does not hold

Text taken from each document's own title page, or from the old deck where noted. Two INL links returned 404 on 2 November, so `r8` and `r10` print with no URL, as `refs.csv` already does.

| Key | Text |
|---|---|
| d_tecdoc1236 | International Atomic Energy Agency. Design and evaluation of heat utilization systems for the high temperature engineering test reactor. IAEA-TECDOC-1236, Vienna, August 2001. |
| d_tecdoc1382 | International Atomic Energy Agency. Evaluation of high temperature gas cooled reactor performance: Benchmark analysis related to initial testing of the HTTR and HTR-10. IAEA-TECDOC-1382, Vienna, November 2003. |
| d_tochio2011 | D. Tochio and S. Nakagawa. Thermal Performance of Intermediate Heat Exchanger during High-Temperature Continuous Operation in HTTR. Journal of Nuclear Science and Technology 48:11 (2011) 1361-1368. https://doi.org/10.1080/18811248.2011.9711828 |
| d_sato2006 | H. Sato, H. Ohashi, Y. Inaba, T. Nishihara, K. Hayashi and Y. Inagaki. Evaluation of Hydrogen Production System Coupling with HTTR Using Dynamic Analysis Code. Transactions of the Atomic Energy Society of Japan 5:4 (2006) 292-304. |
| d_nelson2011 | L.O. Nelson, A. Gandrik, M. McKellar, M. Patterson, E. Robertson and R. Wood. Optimum Reactor Outlet Temperatures for High Temperature Gas-Cooled Reactors Integrated with Industrial Processes. INL/EXT-11-21537, Idaho National Laboratory, 2011. https://inldigitallibrary.inl.gov/content/uploads/50/2026/04/4953394.pdf |
| d_inltriso | Idaho National Laboratory. TRISO fuel qualification. https://art.inl.gov/programs/triso-fuel-qualification/ |
| d_nuclearpower | Nuclear-Power.com. Nuclear fission. https://www.nuclear-power.com/nuclear-power/fission/ |
| d_outram | OUTRAM PARK, open-source nuclear simulation suite, credited as a reference. https://github.com/theodoreOnzGit/outram-park |

## 7. Media

Slide pixels: 1920 wide on 20 in, so 96 px per inch. "1:1 width" is the widest placement with no upscaling.

### 7.1 Prefixes

| Prefix | Location |
|---|---|
| `pptx:` | `ppt/media/` inside the source deck, located by `design/calc/deckboard/media_map.py`. The builder extracts each file to `presentation/media/` |
| `fig:` | `presentation/figures/`, built from tokens by `build_figures.py` |
| `rm:` | `design/calc/r_media/`, the report's figures. The builder copies them to `presentation/media/` |

### 7.2 What is used, and what goes

| File | px | 1:1 width, in | Use | Note |
|---|---|---|---|---|
| `pptx:image1.jpeg` | 2800 x 3500 | 29.2 | S01, S04 | Cover chimney |
| `pptx:image45.jpeg` | 4748 x 2671 | 49.5 | S02, S03 | From the old thank-you slide |
| `pptx:image6.png` | 270 x 174 | 2.8 | S18 | Soft above 5.6 in, redraw natively if it looks poor |
| `pptx:image7.png` | 845 x 363 | 8.8 | S13 | |
| `pptx:image9.png` | 402 x 183 | 4.2 | S14 | Published INL figures inside the picture, Case A only |
| `pptx:image11.png` | 1152 x 583 | 12.0 | S17 | Crossed power plant, caption X12 |
| `pptx:image12.png` | 1152 x 704 | 12.0 | S17 | Crossed furnace. The builder paints out its corner heading, which carries the bare abbreviation |
| `pptx:image13.png` | 360 x 346 | 3.8 | None | Dropped, section 9 item 6 |
| `pptx:image14.jpeg` | 912 x 952 | 9.5 | S24 | Keep the credit line |
| `pptx:image33.png` | 1032 x 268 | 10.8 | S33 | Equation only |
| `pptx:image39.png` | 250 x 500 | 2.6 | S35 | Analogue only |
| `rm:v17_image2.png` | 1496 x 814 | 15.6 | S12 | Crop top 7% to lose the words "in this report" |
| `rm:v17_image3.png` | 1364 x 792 | 14.2 | S21 | Method example |
| `fig:Figure_2_NUS_H2_210MWth.png` | 3000 x 1449 | 31.3 | S06 to S10, S16, S19, S20, S26, S28, S31, S45 | Edited figure |
| `fig:Figure_Roadmap_NUS_H2.png` | 3000 x 1008 | 31.3 | S25 | Carries "Illustrative dates" |
| `fig:Chart_Cost_Ledger_25y.png` | 3000 x 2100 | 31.3 | S38 | |
| `fig:Chart_Cost_Ledger_60y.png` | 3000 x 2100 | 31.3 | B04 | |
| `fig:Chart_Cost_Grid_L5.png` | 3000 x 1800 | 31.3 | S36 | |

Crops of `Figure_2_NUS_H2_210MWth.png`, as fractions of width and height, left, top, right, bottom. Previewed in `design/calc/deckboard/fig2_crops.png`.

| Crop | Box | Shows |
|---|---|---|
| CROP_NUCLEAR | 0.00, 0.00, 0.31, 1.00 | Reactor, IHX, power block |
| CROP_IHX_SMR | 0.15, 0.30, 0.43, 0.98 | IHX and reformer |
| CROP_IHX | 0.15, 0.37, 0.32, 0.72 | IHX with both helium labels |
| CROP_RECYCLE | 0.37, 0.00, 1.00, 0.32 | Recycle, cryogenic unit, PSA |
| CROP_CAPTURE | 0.55, 0.30, 1.00, 0.98 | Absorber, stripper, storage |
| CROP_STACK | 0.37, 0.00, 0.70, 0.31 | N2-rich stack and cryogenic unit |
| CROP_PSA | 0.76, 0.02, 1.00, 0.30 | PSA outside the line |

Not reused. Superseded by edited figures: `image5`, `image15`, `image32`. Superseded table: `image10`. Removed by a user decision: `image31`, the logo. Carry withdrawn or superseded figures: `image35` to `image37`, `image41`, `image43`. Formula pictures from withdrawn slides, typeset natively in B02 instead: `image34`, `image42`, `image44`. Lower-resolution duplicate: `image8`. Stock pictures with no technical content: `image4`, `image29`, `image30`, `image38`, `image40`. Everything from slides 11 and 32 to 37.

### 7.3 New charts

No existing file shows these. Add them to `presentation/figures/build_figures.py` or the deck builder, drawn from tokens only, with the target line on each.

| Chart | Slide | Tokens | Form |
|---|---|---|---|
| CH1 Chart_Sizing_Walk | S15 | `VAL_SIZING_WALK_STEP_0_SIM` to `_6_SIM`, `R_RATED_MWTH` | Stepped bars. Labels: first sum at Case A scale, scaled to Case B, purge load restored, losses, circulators, Selexol swapped for MEA, reboiler gap. Rating as a line |
| CH2 Chart_Abatement_Sensitivity | S30 | `A_ATM_B_MT`, `SENS_CAPTURE_0P8_A_T_Y`, `SENS_AVAILABILITY_0P65_A_T_Y`, `A_CHO_B_MT`, `A_IEAGHG_B_MT`, `A_LC_BURDENS_B_MT`, `A_LC_UPSTREAM_B_MT`, `A_SYSTEM_B_MT`, `TARGET_ABATEMENT_MT` | Horizontal bars with the target line |
| CH3 Chart_Cost_Stepdown | S41b | `STEPDOWN_1` to `STEPDOWN_4` L5, DCF, delta and gas tokens, `TARGET_COST`, `TARGET_MATURE_COST`, `MATURE_60Y_GAS_PRINTED_S_GJ` | Built by `build_figures.py`. Four bars falling left to right, L5 printed with the DCF value beside it, the S$100/t target dashed and the S$96/t point marked. It replaces the withdrawn route chart |

## 8. Guards

Prose states a direction, so it can go stale when the engine moves. The builder evaluates each guard on token values. A failure stops the build and names the slide to rewrite, so no slide can print a claim its numbers no longer support.

| Guard | Condition | Protects |
|---|---|---|
| G01 | TARGET_ABATEMENT_RESULT == "met" and ABATEMENT_MULTIPLE > 2 | S05, S29, S42, S45 |
| G02 | TARGET_COST_RESULT == "missed" and TARGET_COST_N_PASS == 0 | S05, S39, S42, S45. No S$100/t claim at the base unless a central case passes; S41 states its conditions in its title |
| G03 | MARGIN_B_MWTH > 0 and R_HIGH_B_MWTH > R_RATED_MWTH | S16 |
| G04 | MARGIN_A_MEA_MWTH < 0 | S14 |
| G05 | RES_GRID_IMPORT_AT_RATED_MWE == 0 | S13 |
| G06 | SENS_CAPTURE_0P8_A_T_Y > TARGET_ABATEMENT and SENS_AVAILABILITY_0P65_A_T_Y > TARGET_ABATEMENT and A_LC_BURDENS_B_MT > TARGET_ABATEMENT_MT and A_SYSTEM_B_MT > TARGET_ABATEMENT_MT | S30 |
| G07 | A_NORECYCLE_MT > TARGET_ABATEMENT_MT and NEVER_GENERATED_MT < TARGET_ABATEMENT_MT | S31 |
| G08 | DCF_TEST_RESULT == "within 10%" | S33 |
| G09 | LCOH_NUSH2_BOAK_25Y > LCOH_SMR_25Y and LCOH_NUSH2_FOAK_25Y > LCOH_NUSH2_BOAK_25Y | S37 |
| G10 | AC_L3_BOAK_25Y < AC_L5_BOAK_25Y | S38 |
| G11 | LEVER_GAS_L5_BOAK_25Y_S_GJ > GAS_HIGH and LEVER_GAS_L5_BOAK_60Y_S_GJ > GAS_HIGH | S40 |
| G12 | LEVER_OCC_L5_BOAK_25Y_USD_KWT < OCC_LOW and LEVER_OCC_L5_BOAK_60Y_USD_KWT < OCC_LOW | S40 |
| G13 | LEVER_TS_L5_BOAK_25Y_S_T < 0 and LEVER_TS_L5_BOAK_60Y_S_T < 0 | S40 |
| G14 | AC_L5_BOAK_25Y_N1P00_NEED > TARGET_COST and AC_L5_BOAK_60Y_N1P00_NEED > TARGET_COST | S40 |
| G15 | STEPDOWN_1_L5 > STEPDOWN_2_L5 > STEPDOWN_3_L5 > TARGET_COST > TARGET_MATURE_COST > STEPDOWN_4_L5 and STEPDOWN_1_DCF > STEPDOWN_2_DCF > STEPDOWN_3_DCF > TARGET_COST > TARGET_MATURE_COST > STEPDOWN_4_DCF and abs(STEPDOWN_1_L5 - AC_L5_BOAK_25Y) < 0.05 and abs(STEPDOWN_1_DCF - LAC_DCF_BOAK_25Y) < 0.05 and STEPDOWN_1_GAS_S_GJ == GAS_PRICE_S_GJ | S41b. Each step lowers the cost, only the last crosses the target, and the base is the BOAK 25 y result |
| G20 | abs(STEPDOWN_4_L5 - MATURE_60Y_L5_AT_PRINTED) < 0.05 and abs(STEPDOWN_4_DCF - MATURE_60Y_DCF_AT_PRINTED) < 0.05 and abs(STEPDOWN_4_GAS_S_GJ - MATURE_60Y_GAS_PRINTED_S_GJ) < 1e-9 and MATURE_60Y_GAS_PRINTED_S_GJ >= MATURE_60Y_L5_GAS_NEEDED_S_GJ and MATURE_60Y_GAS_PRINTED_S_GJ >= MATURE_60Y_DCF_GAS_NEEDED_S_GJ and MATURE_60Y_GAS_PRINTED_S_GJ - MATURE_60Y_L5_GAS_NEEDED_S_GJ < 0.1 and MATURE_60Y_GAS_PRINTED_S_GJ - MATURE_60Y_DCF_GAS_NEEDED_S_GJ < 0.1 | S41, S41b. One row family, and the printed price is the larger solve rounded up to 0.1 |
| G21 | MATURE_60Y_L5_AT_PRINTED <= TARGET_MATURE_COST and MATURE_60Y_DCF_AT_PRINTED <= TARGET_MATURE_COST and TARGET_MATURE_COST < TARGET_COST and MATURE_60Y_GAS_PRINTED_S_GJ > GAS_PRICE_S_GJ and MATURE_60Y_L5_AT_20 < MATURE_60Y_L5_AT_CENTRAL and MATURE_60Y_DCF_AT_20 < MATURE_60Y_DCF_AT_CENTRAL and NG_B_MMSCFD < NG_UNABATED_B_MMSCFD and AC_L5_BOAK_25Y > TARGET_COST and LAC_DCF_BOAK_25Y > TARGET_COST | S41, S42, S45. The mature point meets the target, higher gas lowers the cost, and the base misses |
| G16 | VAL_N_INDEPENDENT > 0 and VAL_N_FAIL > 0 and VAL_N_PASS > VAL_N_FAIL and STATUS_ROWS_ANCHORED_PORT > 0 | S43, the footer status tag. When no row fails, or none is independent, rewrite S43's slide text and note |
| G17 | AC_L5_BOAK_25Y < RES_COMPARATOR_CAC_TS60_S_T and RES_COMPARATOR_CAC_TS60_S_T < AC_L5_FOAK_25Y | B03 |
| G18 | NG_B_MMSCFD < NG_UNABATED_B_MMSCFD and GAS_SAVING_AHN_PCT < GAS_SAVING_INL_PCT | S09 |
| G19 | NEW_PINCH_IAEA_K < NEW_PINCH_CENTRAL_K | S20 |

## 9. Open items

1. **Tokens.** Section 6.1 lists four tokens to add. Section 6.2 lists 16 published constants with no route through `results/`. The storyboard treats them as a single table until they get a home in `parameters.toml`.
2. **Report tokens.** The deck leans on the derived layer in `report/_build/resolve_numbers.py`. If that script renames a token, the checker fails by name, which is the intended behaviour.
3. **Results not available.** The freeze lists reformer 825 and 800 C sensitivities, and `Sensitivity_Results.csv` has no such sweep. Backup firing has report-side tokens only. The deck prints neither as a result. S08 discloses the 871 C assumption without a sensitivity.
4. **Headline pair.** BOAK and FOAK print side by side on S35, S37, S39 and S45 under X4. If the user wants FOAK to lead, only their order changes. Freeze section 10 item 1 stays open.
5. **Headline abatement.** The headline number stays with the storage burden beneath it on S29, per X9. Freeze section 10 item 2 stays open.
6. **`image13`.** Closed. The social-post screenshot is dropped, and S23 carries a native panel.
11. **Q&A backup slides.** The 16 slides of `presentation/inputs/qa_backup_source.pptx` follow B10 in their own format, built by `presentation/qa_backup.py`. Their pictures are kept as supplied; slide 16 is rebuilt from the COFUNDING rows.
7. **S21 chart.** The report prints this figure with its 400 m axis. `htgr_ihx.md` found no library PDF behind that distance, while the report cites `r17`, which is outside the library. The slide caption ties the distance to the Xe-100 case and claims none for NUS-H2.
8. **Cho.** Only the abstract was read, so S27 uses the headline figure and plant count only.
9. **S30 and the 85% line.** The freeze's 90 to 92% HTGR figures are assumed values in NP-T-4.3, so the note says there is no measured record.
10. **Expert voice.** The S11 quotation is a sentence from `r8`. It is the only quotation in the deck, and it is the team's choice of sentence. INL has not reviewed this work, and the slide says so.
