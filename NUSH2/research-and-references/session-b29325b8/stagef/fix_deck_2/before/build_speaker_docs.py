#!/usr/bin/env python3
"""Build presentation/NUSH2_Speaker_Script.docx and presentation/NUSH2_QA_Prep.docx with no hand edits.

    python presentation/build_speaker_docs.py
    python presentation/build_speaker_docs.py --pdf    also export both to presentation/render/ for review

Run with the Python that runs build_deck.py (python-pptx, python-docx, lxml). build_deck.py calls this
script after the deck, so one command refreshes deck, script and Q&A sheet together.

The spoken text is the storyboard's speaker notes, the same text the deck carries. The Q&A answers below
hold {{TOKEN}} placeholders only. Both resolve through build_deck.Book, which reads build/numbers.json and
its derived layer, so a token that cannot resolve stops the build. Every result-dependent claim in an
answer is protected by a guard in QA_GUARDS: if the engine moves a number across the line a sentence
depends on, the build stops and names the answer to rewrite. Physical units are written as native Word
math through report/_build/omml.py.
"""
import argparse
import csv
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(ROOT / "report" / "_build"))

import build_deck as bd  # noqa: E402
import omml  # noqa: E402
from docx import Document  # noqa: E402
from docx.enum.table import WD_TABLE_ALIGNMENT  # noqa: E402
from docx.enum.text import WD_ALIGN_PARAGRAPH  # noqa: E402
from docx.oxml import OxmlElement  # noqa: E402
from docx.oxml.ns import qn  # noqa: E402
from docx.shared import Mm, Pt, RGBColor  # noqa: E402

SCRIPT = HERE / "NUSH2_Speaker_Script.docx"
QA = HERE / "NUSH2_QA_Prep.docx"
UNITS = ROOT / "report" / "_build" / "units.csv"
FONT, HEAD_FONT = "Segoe UI", "Segoe UI Semibold"
omml.FONT = FONT
TITLE = "Nuclear-Upgraded Steam Methane Reforming with Carbon Capture for Low-Carbon Hydrogen in Singapore"
# Status of the checks, guarded with G16 of the storyboard: some rows are independent and some fail.
VALIDATION_LINE = ("{{VAL_N_INDEPENDENT}} of {{VAL_N_TOTAL}} checks are independent of the source data, and "
                   "{{VAL_N_INDEPENDENT_PASS}} of those pass. Overall {{VAL_N_PASS}} pass, {{VAL_N_FAIL}} fail, "
                   "{{VAL_N_REPORTED}} are reported with no limit to test against and {{VAL_N_NOT_RUN}} did not run. "
                   "The failures stay recorded as failed.")


# ---------------------------------------------------------------- Q&A content
# Each answer: short answer, body paragraphs or bullets, the line to give if pressed, and the slides to show.
# Prose rules: no em dashes, no rule-of-three phrasing, no "not A but B" constructions.

QA_ITEMS = [
    dict(q="Why {{R_RATED_MWTH|f0}} MWth?",
         short="{{R_RATED_MWTH|f0}} MWth is the rating we hold. The process needs {{R_NEEDED_B_MWTH}} MWth, so {{MARGIN_B_MWTH}} MWth is margin that earns no revenue.",
         body=["The rating comes from the original project basis and is kept with Case B as the headline (decision D03). "
               "The sizing walk starts from a first sum of {{VAL_SIZING_WALK_STEP_0_SIM}} MWth at INL Case A scale, which is "
               "{{VAL_SIZING_WALK_STEP_1_SIM}} MWth at our {{H2_B_NM3H}} Nm3/h. Counting every real load, from the restored purge load "
               "to MEA's reboiler gap, brings the need to {{R_NEEDED_B_MWTH}} MWth, within a range of {{R_BEST_B_MWTH}} to {{R_HIGH_B_MWTH}} MWth.",
               "We keep the rating because the need is uncertain. On Ahn and Lee's heat basis the margin falls to "
               "{{SENS_HEAT_BASIS_143P4_MARGIN_210}} MWth, and at the adverse corner the plant is {{ADVERSE_SHORT_MWTH}} MWth short. "
               "The spare capacity is paid for: full cost is S${{AC_L5_BOAK_25Y}}/t charged at {{R_RATED_MWTH|f0}} MWth against "
               "S${{AC_L5_BOAK_25Y_N1P00_NEED}}/t sized to need, at BOAK over {{LIVES_SHORT|f0}} y.",
               "INL's Case A cannot be the design at this rating. With MEA and recycle it needs {{R_NEEDED_A_MEA_MWTH}} MWth, "
               "{{MARGIN_A_MEA_MWTH|abs}} MWth above the rating, so it stays a reference."],
         pressed="The reboiler gap rests on an SRD of {{SRD_CENTRAL}} GJ/t with no syngas measurement behind it. Across "
                 "{{SRD_LOW}} to {{SRD_HIGH}} GJ/t the need moves from {{SENS_SRD_3P4_R_NEEDED}} to {{SENS_SRD_4P2_R_NEEDED}} MWth.",
         show=["S14", "S15", "S16"]),
    dict(q="Why HTGR instead of a PWR?",
         short="Reforming needs heat near {{SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C}} C, and of the reactor types we compared only a "
               "high-temperature gas-cooled reactor reaches that band.",
         body=["On the process temperature chart, light-water reactors stop far short of the reforming band, which is why we use "
               "a gas-cooled reactor in place of a PWR. A helium-cooled HTGR with a {{ROT}} C outlet gives secondary helium at "
               "{{T_HE_SUPPLY}} C, and {{ROT}} C is INL's only case that removes the fired stage. INL's yield optimum is "
               "{{CTX_INL_OPT_ROT_C}} C, with a fired stage kept.",
               "TRISO coatings are the first barrier to fission product release, provided fuel stays below about "
               "{{CTX_TRISO_LIMIT_C}} C, which leaves headroom above the outlet temperature."],
         pressed="Only test reactors of {{CTX_TEST_REACTOR_MAX_MW}} MW or less have run this hot. Commercial HTR-PM, and Xe-100 "
                 "as our analogue, run near {{CTX_COMMERCIAL_ROT_C}} C, so our {{ROT}} C reactor is generic and unproven at scale.",
         show=["S12", "S13", "S35"]),
    dict(q="Why MEA instead of Selexol?",
         short="MEA is our design basis (decision D25). The evidence is mixed, and our own sensitivity favours Selexol slightly.",
         body=["IEAGHG 2022-07 calls MEA the preferred chemical solvent, while IEAGHG 2017-02 favours aMDEA on shifted syngas, "
               "and INL's reference plant uses Selexol. We run {{W_MEA|pct0}} wt% MEA as the reference and carry Selexol and aMDEA "
               "as sensitivities.",
               "In our model Selexol abates {{SENS_SOLVENT_1_A_T_Y|Mt}} Mt/y at a full cost of S${{SENS_SOLVENT_1_AC_L5_25}}/t, "
               "against {{A_ATM_B_MT}} Mt/y at S${{AC_L5_BOAK_25Y}}/t for MEA, at BOAK over {{LIVES_SHORT|f0}} y. It also leaves "
               "{{SENS_SOLVENT_1_MARGIN_210}} MWth spare against {{MARGIN_B_MWTH}} MWth. aMDEA sits close to Selexol, at "
               "{{SENS_SOLVENT_2_A_T_Y|Mt}} Mt/y and S${{SENS_SOLVENT_2_AC_L5_25}}/t. Selexol would also keep the recycle INL "
               "actually modelled, which removes our largest process risk."],
         pressed="Say plainly that our numbers do not show MEA ahead. MEA stays the basis by design decision D25, and a switch "
                 "to Selexol or aMDEA remains open.",
         show=["S10", "S31"]),
    dict(q="How is MEA regeneration powered?",
         short="By steam, taken in a fixed order, with reactor steam covering only the last gap.",
         body=["The reboiler needs {{REBOILER_B_MW}} MW at an SRD of {{SRD_CENTRAL}} GJ/t. It draws first on the steam freed by "
               "removing Selexol, {{EB_B_REC_F_REB_FROM_SELEXOL}} MW, then on diverted export steam, {{EB_B_REC_F_REB_FROM_EXPORT}} MW, then on "
               "recovered syngas heat, {{EB_B_REC_F_SYNGAS_RECOVERY}} MW. Reactor steam covers the remaining {{REBOILER_GAP_B_MW}} MW, "
               "and that gap is the last step of the sizing walk.",
               "Diverting export steam is a real cost: lost steam export adds S${{LINE_LOST_STEAM_BOAK_25Y_N1P00}}M/y at L5. "
               "Electricity for the MEA pumps and fans comes from the site power block, inside the {{E_PROCESS_B_MWE}} MWe of "
               "process loads."],
         pressed="SRD has no syngas measurement behind it. Across {{SRD_LOW}} to {{SRD_HIGH}} GJ/t the reactor need moves from "
                 "{{SENS_SRD_3P4_R_NEEDED}} to {{SENS_SRD_4P2_R_NEEDED}} MWth, and cost barely moves because the reactor is "
                 "charged at its rating.",
         show=["S15", "B06"]),
    dict(q="Is the system thermally self-sufficient?",
         short="Yes on the central balance: the reactor covers every heat and power load with {{MARGIN_B_MWTH}} MWth to spare.",
         body=["The reactor needs {{R_NEEDED_B_MWTH}} MWth against a {{R_RATED_MWTH|f0}} MWth rating. Unmet thermal duty is "
               "{{EB_B_REC_F_UNMET_THERMAL}} MW and unmet electrical load is {{EB_B_REC_F_UNMET_ELECTRICAL}} MWe. A small power block "
               "supplies the {{E_TOTAL_B_MWE}} MWe site load, {{E_PROCESS_B_MWE}} MWe of it for process loads and the rest for helium "
               "circulators, so grid import at rated size is {{RES_GRID_IMPORT_AT_RATED_MWE|f0}} MWe."],
         pressed="The margin is thin. It falls to {{SENS_HEAT_BASIS_143P4_MARGIN_210}} MWth on Ahn and Lee's heat basis, and the "
                 "adverse corner leaves the plant {{ADVERSE_SHORT_MWTH}} MWth short. Ship liquefaction load for CO2 is unmodelled. "
                 "The balance also rests on the IHX delivering {{T_HE_SUPPLY}} C, which no measured unit has done. Outside the "
                 "{{AVAILABILITY_PCT|f0}}% availability hours there is no hydrogen unless backup firing is added, a sensitivity whose "
                 "furnace capital is unpriced.",
         show=["S13", "S15", "S16", "S20"]),
    dict(q="Why exclude PSA tail gas?",
         short="We no longer exclude it. The PSA stays outside the boundary as an unsimulated block, and its reject now returns "
               "across the boundary to be recycled (decision D04).",
         body=["A recycle compressor feeds the reject to a cryogenic unit. Its methane-rich cut returns to the reformer feed and its "
               "hydrogen goes back towards the PSA, while the nitrogen-rich cut is burned and vented. That stack is the plant's only "
               "residual source: {{ATM_B_T_Y}} t CO2/y, or {{INTENSITY_B_KG_KG}} kg CO2 per kg H2. The PSA purge load is restored "
               "in the power balance.",
               "Recycle is why NUS-H2 takes {{NG_B_MMSCFD}} MMSCFD of gas against {{NG_UNABATED_B_MMSCFD}} MMSCFD for a same-size "
               "unabated steam methane reforming plant."],
         pressed="INL modelled recycle with Selexol, never with MEA. If it fails to transfer, gas rises to "
                 "{{SENS_TAIL_GAS_2_NG_MMSCFD}} MMSCFD and abatement falls to {{SENS_TAIL_GAS_2_A_T_Y|Mt}} Mt/y. Full cost then rises to "
                 "S${{SENS_TAIL_GAS_2_AC_L5_25}}/t. INL also never modelled the larger CO2 slip that MEA sends through the "
                 "cryogenic unit, which stays an open risk.",
         show=["S06", "S09", "S28", "S31"]),
    dict(q="What are the safety barriers?",
         short="TRISO fuel coatings come first, inside the reactor's own barriers, and the IHX is the boundary to the chemical plant.",
         body=["TRISO coatings are the first barrier to fission product release, provided fuel stays below about "
               "{{CTX_TRISO_LIMIT_C}} C. HTTR counts four barriers: coatings, pressure vessel, containment and reactor building. "
               "Tritium is the known exception, since it can leave cracked coatings and also forms in graphite.",
               "The IHX tubes separate the nuclear island from the process helium loop. Secondary helium is held slightly above "
               "primary pressure, so a leak runs towards the reactor, and purification with monitoring manages tritium permeation. "
               "Isolation needs a high-temperature emergency shutoff valve, which our figure omits.",
               "The emergency planning zone follows source term and dose, so only a design-specific safety case can size it, and we "
               "claim no distance. That case must also cover process hazards beside the reactor, such as fire and toxic release."],
         pressed="A thermal buffer for a reformer trip is still open in our design.",
         show=["S18", "S19", "S21"]),
    dict(q="Why Singapore?",
         short="Singapore runs on imported gas and plans for hydrogen, so low-carbon hydrogen made here meets a national need.",
         body=["Natural gas made up {{CTX_GAS_POWER_PCT}}% of the electricity fuel mix in 2024, and the national hydrogen strategy "
               "sees low-carbon hydrogen supplying up to {{CTX_H2_POWER_PCT}}% of power needs by 2050. Hydrogen is already made at "
               "industrial scale on Jurong Island, and SOXAL's plant sets our size of {{H2_B_NM3H}} Nm3/h, used for size only.",
               "Singapore is also building the capability to decide on nuclear energy: the IAEA INIR Phase 1 assessment is "
               "tentatively set for 2027 and covers {{CTX_INIR_AREAS}} areas."],
         pressed="Singapore has made no decision on nuclear energy. CO2 has to be stored abroad under the Singapore-Indonesia "
                 "arrangement, and the accounting across the border is unresolved. For scale only, our abatement equals "
                 "{{NDC_SHARE_LOW_PCT}} to {{NDC_SHARE_HIGH_PCT}}% of Singapore's 2035 target level.",
         show=["S01", "S02", "S22"]),
    dict(q="Why Western Island?",
         short="It is the author's proposal, with no official approval: new land planned for energy and industry, linked to Jurong Island.",
         body=["The new western island from the 2026 National Day Rally is planned for future energy and industry use among others, "
               "with a planned link to Jurong Island, where gas supply and hydrogen users already sit. Our plant is a new build "
               "there, compared with a same-size unabated steam methane reforming plant."],
         pressed="The government page does not mention nuclear. Reclamation runs over coming decades, so the land may not exist at "
                 "our milestone dates, and every link to the island is unpriced.",
         show=["S24", "S25"]),
    dict(q="How realistic is the 2050 roadmap?",
         short="Possible on paper. Every date is illustrative, and each gate opens only if the stage before it delivers.",
         body=["The roadmap has four gated stages: national readiness from 2027, reactor and process-heat qualification from 2030, "
               "project development and an investment decision from 2035, then construction from 2039 towards potential operation "
               "by 2050. The construction window leans on the {{CTX_HTRPM_YEARS}} years HTR-PM took from construction start to "
               "commercial operation."],
         pressed="The weakest links are the IHX and the land. The highest measured IHX secondary temperature in our sources is "
                 "{{CTX_HE_MEASURED_C}} C on a {{CTX_HTTR_IHX_MW}} MW unit, against the {{T_HE_SUPPLY}} C we need, and reclamation "
                 "has no published date. The first plant would also be first of a kind, at a full cost of S${{AC_L5_FOAK_25Y}}/t over "
                 "{{LIVES_SHORT|f0}} y.",
         show=["S22", "S25", "S20"]),
    dict(q="Does the project satisfy the emissions target?",
         short="Yes. Net abatement is {{A_ATM_B_MT}} Mt CO2e/y against a target of more than {{TARGET_ABATEMENT_MT}} Mt/y, "
               "{{ABATEMENT_MULTIPLE}} times the target.",
         body=["This counts direct emissions to atmosphere against a same-size unabated steam methane reforming plant on INL's {{BASELINE_INL_KG_KG}} kg/kg "
               "baseline, a {{REDUCTION_B_PCT}}% cut. The plant captures {{PLANT_CAPTURE_B_PCT}}% of its carbon at "
               "{{PER_PASS_CAPTURE_PCT|f0}}% per pass, and {{RES_CARBON_NEVER_GENERATED_T_Y}} t CO2/y never forms. With the storage chain "
               "burden charged the result is {{A_TS_BURDEN_B_MT}} Mt/y. Cho's and IEAGHG's baselines give {{A_CHO_B_MT}} and "
               "{{A_IEAGHG_B_MT}} Mt/y.",
               "Every stress case with working capture stays above the target: {{SENS_AVAILABILITY_0P65_A_T_Y|Mt}} Mt/y at "
               "{{SWEEP_AVAILABILITY_0P65|pct0}}% availability, and {{SENS_TAIL_GAS_2_A_T_Y|Mt}} Mt/y if recycle fails."],
         pressed="With capture offline only {{RES_CARBON_NEVER_GENERATED_T_Y|Mt}} Mt/y remains, which misses the target, so capture has to work. "
                 "Abatement is counted at the Singapore stack while the CO2 is stored abroad.",
         show=["S28", "S29", "S30", "S31"]),
    dict(q="Does the project satisfy the cost target?",
         short="At central inputs, no: {{TARGET_COST_N_PASS}} of {{TARGET_COST_N_CENTRAL}} central cases reach "
               "S${{TARGET_COST_S_T|f0}}/t. In the named scenario MATURE_60Y, a 10th-of-a-kind plant on INL's BOAK learning "
               "route over {{LIVES_LONG|f0}} y meets it at S${{TARGET_MATURE_COST|f0}}/t when gas costs "
               "S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ, above the S${{GAS_LOW|f0}} to S${{GAS_HIGH|f0}}/GJ range tested.",
         body=["The full cost by the annual ledger is S${{AC_L5_BOAK_25Y}}/t at BOAK over {{LIVES_SHORT|f0}} y and "
               "S${{AC_L5_BOAK_60Y}}/t over {{LIVES_LONG|f0}} y. A first of a kind costs S${{AC_L5_FOAK_25Y}}/t and "
               "S${{AC_L5_FOAK_60Y}}/t. The discounted cash flow agrees within {{DCF_GAP_MAX_ALL_PCT}}% over all "
               "{{DCF_GAP_N_POINTS}} points, and within {{DCF_TEST_MAX_GAP_PCT}}% on the headline grid, at "
               "S${{LAC_DCF_BOAK_25Y}}/t for BOAK over {{LIVES_SHORT|f0}} y. "
               "Hydrogen costs S${{LCOH_NUSH2_BOAK_25Y}}/kg against S${{LCOH_SMR_25Y}}/kg unabated.",
               "No single lever closes the gap. Gas would have to reach S${{LEVER_GAS_L5_BOAK_25Y_S_GJ}}/GJ, about "
               "{{LEVER_GAS_L5_BOAK_25Y_MULTIPLE}} times our central price. Reactor capital would have to fall to "
               "US${{LEVER_OCC_L5_BOAK_25Y_USD_KWT}}/kWt, under INL's low of US${{OCC_LOW}}/kWt. Transport and storage would have "
               "to go negative.",
               "Three conditions together cross the target, and each is an engine run. A 10th-of-a-kind reactor on INL's "
               "BOAK learning route takes BOAK over {{LIVES_SHORT|f0}} y from S${{STEPDOWN_1_L5}}/t to S${{STEPDOWN_2_L5}}/t. A "
               "{{LIVES_LONG|f0}} y life takes it to S${{STEPDOWN_3_L5}}/t. Gas at S${{STEPDOWN_4_GAS_S_GJ}}/GJ then gives "
               "S${{STEPDOWN_4_L5}}/t on the ledger and S${{STEPDOWN_4_DCF}}/t by cash flow, under the target at the "
               "S${{TARGET_MATURE_COST|f0}}/t point. The engine solves that gas price at S${{MATURE_60Y_L5_GAS_NEEDED_S_GJ}}/GJ "
               "on the ledger and S${{MATURE_60Y_DCF_GAS_NEEDED_S_GJ}}/GJ by cash flow, and the slide prints the larger, rounded "
               "up. That price lies above the S${{GAS_LOW|f0}} to S${{GAS_HIGH|f0}}/GJ range tested. At S${{GAS_HIGH|f0}}/GJ, the "
               "top of that range, the scenario gives S${{MATURE_60Y_L5_AT_20}}/t on the ledger and "
               "S${{MATURE_60Y_DCF_AT_20}}/t by cash flow. Higher gas lowers the abatement cost because NUS-H2 burns less gas than the unabated plant."],
         pressed="The S${{TARGET_MATURE_COST|f0}}/t case is a set of conditions on the base, and the base stays the reference: "
                 "S${{AC_L5_BOAK_25Y}}/t on the ledger and S${{LAC_DCF_BOAK_25Y}}/t by cash flow. Do not quote the named step L3 "
                 "as the answer either. L3, at S${{AC_L3_BOAK_25Y}}/t, leaves out real costs of this plant such as construction "
                 "interest and lost steam export, and it still misses.",
         show=["S37", "S38", "S39", "S40", "S41", "S41b"]),
    dict(q="Could public co-funding bring the cost down?",
         short="Yes, at a price to the public budget. Co-funding S${{COFUND_L5_25Y_SM}}M, {{COFUND_L5_25Y_SHARE_PCT}}% of the "
               "S${{COFUND_REACTOR_CAPITAL_SM}}M reactor capital, brings BOAK over {{LIVES_SHORT|f0}} y to "
               "S${{COFUND_TARGET_S_T|f0}}/t on the ledger.",
         body=["The cash flow needs S${{COFUND_DCF_25Y_SM}}M, {{COFUND_DCF_25Y_SHARE_PCT}}%. Over {{LIVES_LONG|f0}} y the "
               "ledger needs S${{COFUND_L5_60Y_SM}}M, {{COFUND_L5_60Y_SHARE_PCT}}%, and the cash flow S${{COFUND_DCF_60Y_SM}}M, "
               "{{COFUND_DCF_60Y_SHARE_PCT}}%. These are engine solves on the BOAK case at n = {{SWEEP_SIZE_EXPONENT_1|f1}} with "
               "{{R_RATED_MWTH|f0}} MWth charged."],
         pressed="Co-funding moves cost to the public budget. The total resource cost stays the same, so it is a policy choice "
                 "and leaves the engineering result unchanged.",
         show=["S39", "QA16"]),
    dict(q="Which assumptions dominate uncertainty?",
         short="Recycle transfer to MEA and the reactor cost treatment move the result most. The IHX temperature decides whether "
               "the design works at all.",
         bullets=["Recycle transfer to MEA. Without it abatement is {{SENS_TAIL_GAS_2_A_T_Y|Mt}} Mt/y and full cost S${{SENS_TAIL_GAS_2_AC_L5_25}}/t, "
                  "against {{A_ATM_B_MT}} Mt/y and S${{AC_L5_BOAK_25Y}}/t.",
                  "Reactor size law. At BOAK over {{LIVES_SHORT|f0}} y, full cost is S${{AC_L5_BOAK_25Y}}/t at n = "
                  "{{SWEEP_SIZE_EXPONENT_1|f1}}, S${{AC_L5_BOAK_25Y_N0P60}}/t at n = {{SWEEP_SIZE_EXPONENT_0P6|f1}} and "
                  "S${{AC_L5_BOAK_25Y_N0P31}}/t at n = {{SWEEP_SIZE_EXPONENT_0P310698|f2}}.",
                  "Build maturity. A first of a kind carries a {{FOAK_CENTRAL|f1}} times premium on reactor capital, which gives "
                  "S${{AC_L5_FOAK_25Y}}/t against S${{AC_L5_BOAK_25Y}}/t. INL's low and high reactor capital give "
                  "S${{SENS_REACTOR_CAPITAL_1456_AC_L5_25}}/t and S${{SENS_REACTOR_CAPITAL_2548_AC_L5_25}}/t.",
                  "Availability. {{AVAILABILITY_PCT|f0}}% has no measured HTGR record behind it. At "
                  "{{SWEEP_AVAILABILITY_0P65|pct0}}% abatement falls to {{SENS_AVAILABILITY_0P65_A_T_Y|Mt}} Mt/y and full cost "
                  "rises to S${{SENS_AVAILABILITY_0P65_AC_L5_25}}/t.",
                  "Gas price. S${{SWEEP_GAS_10|f0}}/GJ gives S${{SENS_GAS_10_AC_L5_25}}/t and S${{SWEEP_GAS_20|f0}}/GJ gives "
                  "S${{SENS_GAS_20_AC_L5_25}}/t. On Ahn and Lee's lower gas saving, full cost is S${{SENS_GAS_SAVING_AHN_0P188038_B_REC_F_CARBON_HELD_AC_L5_25}}/t.",
                  "Capture capital basis. Our capture capital follows IEAGHG's aMDEA plant; an MEA flue gas basis gives "
                  "S${{SENS_CAPTURE_CAPITAL_5P09395_AC_L5_25}}/t.",
                  "IHX approach. At the IAEA's typical {{IHX_APPROACH_K_HIGH}} K approach the reformer pinch falls from "
                  "{{PINCH_CENTRAL_K}} K to {{PINCH_IAEA_K}} K, which puts the no-furnace claim at risk.",
                  "SRD moves the spare margin, from {{SENS_SRD_3P4_MARGIN_210}} to {{SENS_SRD_4P2_MARGIN_210}} MWth, and leaves "
                  "cost almost unchanged."],
         pressed="Each case moves one input at a time. Corner runs carry no probability, so a range here is no confidence interval.",
         show=["S30", "S31", "S36", "B08"]),
]

# A claim in an answer stays true only while its numbers sit on the stated side of a line. A failure stops
# the build and names the answer to rewrite.
QA_GUARDS = [
    ("Q1", "MARGIN_B_MWTH > 0 and R_HIGH_B_MWTH > R_RATED_MWTH and MARGIN_A_MEA_MWTH < 0 and AC_L5_BOAK_25Y_N1P00_NEED < AC_L5_BOAK_25Y"),
    ("Q2", "ROT > SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C and CTX_COMMERCIAL_ROT_C < ROT"),
    ("Q3", "SENS_SOLVENT_1_A_T_Y > A_ATM_B_T and SENS_SOLVENT_1_AC_L5_25 < AC_L5_BOAK_25Y and SENS_SOLVENT_1_MARGIN_210 > MARGIN_B_MWTH"
           " and SENS_SOLVENT_2_A_T_Y > A_ATM_B_T and SENS_SOLVENT_2_AC_L5_25 < AC_L5_BOAK_25Y"),
    ("Q4", "REBOILER_GAP_B_MW > 0 and abs(EB_B_REC_F_REB_FROM_SELEXOL + EB_B_REC_F_REB_FROM_EXPORT + EB_B_REC_F_SYNGAS_RECOVERY + REBOILER_GAP_B_MW"
           " - REBOILER_B_MW) < 0.2 and LINE_LOST_STEAM_BOAK_25Y_N1P00 > 0"),
    ("Q5", "EB_B_REC_F_UNMET_THERMAL == 0 and EB_B_REC_F_UNMET_ELECTRICAL == 0 and RES_GRID_IMPORT_AT_RATED_MWE == 0"
           " and MARGIN_B_MWTH > 0 and ADVERSE_SHORT_MWTH > 0"),
    ("Q6", "NG_B_MMSCFD < NG_UNABATED_B_MMSCFD and SENS_TAIL_GAS_2_NG_MMSCFD > NG_B_MMSCFD and A_NORECYCLE_MT < A_ATM_B_MT"
           " and SENS_TAIL_GAS_2_AC_L5_25 > AC_L5_BOAK_25Y"),
    ("Q10", "CTX_HE_MEASURED_C < T_HE_SUPPLY"),
    ("Q11", "TARGET_ABATEMENT_RESULT == 'met' and A_TS_BURDEN_B_MT > TARGET_ABATEMENT_MT and A_CHO_B_MT > TARGET_ABATEMENT_MT"
            " and A_IEAGHG_B_MT > TARGET_ABATEMENT_MT and SENS_AVAILABILITY_0P65_A_T_Y > TARGET_ABATEMENT"
            " and A_NORECYCLE_MT > TARGET_ABATEMENT_MT and NEVER_GENERATED_MT < TARGET_ABATEMENT_MT"),
    ("Q12", "TARGET_COST_RESULT == 'missed' and TARGET_COST_N_PASS == 0 and LEVER_GAS_L5_BOAK_25Y_S_GJ > GAS_HIGH"
            " and LEVER_OCC_L5_BOAK_25Y_USD_KWT < OCC_LOW and LEVER_TS_L5_BOAK_25Y_S_T < 0 and AC_L3_BOAK_25Y > TARGET_COST"
            " and STEPDOWN_1_L5 > STEPDOWN_2_L5 > STEPDOWN_3_L5 > TARGET_COST > TARGET_MATURE_COST > STEPDOWN_4_L5"
            " and STEPDOWN_4_DCF < TARGET_MATURE_COST and MATURE_60Y_GAS_PRINTED_S_GJ >= MATURE_60Y_L5_GAS_NEEDED_S_GJ"
            " and MATURE_60Y_GAS_PRINTED_S_GJ >= MATURE_60Y_DCF_GAS_NEEDED_S_GJ and NG_B_MMSCFD < NG_UNABATED_B_MMSCFD"
            " and MATURE_60Y_GAS_PRINTED_S_GJ > GAS_HIGH and SWEEP_MATURE_60Y_20_MATURE_60Y_L5 == GAS_HIGH"
            " and MATURE_60Y_L5_AT_20 > TARGET_MATURE_COST and MATURE_60Y_DCF_AT_20 > TARGET_MATURE_COST"),
    ("Q13", "COFUND_L5_25Y_SM < COFUND_REACTOR_CAPITAL_SM and COFUND_L5_60Y_SM < COFUND_L5_25Y_SM"
            " and COFUND_DCF_60Y_SM < COFUND_DCF_25Y_SM and COFUND_TARGET_S_T < TARGET_COST_S_T"),
    ("Q14", "SENS_TAIL_GAS_2_AC_L5_25 > AC_L5_BOAK_25Y and AC_L5_BOAK_25Y_N0P31 > AC_L5_BOAK_25Y_N0P60 > AC_L5_BOAK_25Y"
            " and PINCH_IAEA_K < PINCH_CENTRAL_K and SENS_SRD_3P4_MARGIN_210 > SENS_SRD_4P2_MARGIN_210"),
]

QUICK_REF = [
    ("Hydrogen", "{{H2_B_NM3H}} Nm3/h, {{H2_B_T_Y}} t/y at {{AVAILABILITY_PCT|f0}}% availability"),
    ("Reactor", "{{R_RATED_MWTH|f0}} MWth rated, {{R_NEEDED_B_MWTH}} MWth needed, {{MARGIN_B_MWTH}} MWth spare"),
    ("Process heat", "{{Q_PROCESS_B_MWTH}} MWth at {{T_HE_SUPPLY}} C secondary helium"),
    ("Natural gas", "{{NG_B_MMSCFD}} MMSCFD against {{NG_UNABATED_B_MMSCFD}} MMSCFD for unabated steam methane reforming"),
    ("Net abatement", "{{A_ATM_B_MT}} Mt CO2e/y, {{REDUCTION_B_PCT}}% cut; target {{TARGET_ABATEMENT_RESULT}}"),
    ("Capture", "{{PER_PASS_CAPTURE_PCT|f0}}% per pass, {{PLANT_CAPTURE_B_PCT}}% of plant carbon"),
    ("To atmosphere", "{{ATM_B_T_Y}} t CO2/y, {{INTENSITY_B_KG_KG}} kg CO2 per kg H2"),
    ("Full cost, BOAK", "S${{AC_L5_BOAK_25Y}}/t at {{LIVES_SHORT|f0}} y, S${{AC_L5_BOAK_60Y}}/t at {{LIVES_LONG|f0}} y"),
    ("Full cost, FOAK", "S${{AC_L5_FOAK_25Y}}/t at {{LIVES_SHORT|f0}} y, S${{AC_L5_FOAK_60Y}}/t at {{LIVES_LONG|f0}} y"),
    ("Cost target", "{{TARGET_COST_N_PASS}} of {{TARGET_COST_N_CENTRAL}} central cases reach S${{TARGET_COST_S_T|f0}}/t; "
                    "target {{TARGET_COST_RESULT}} at the base"),
    ("Cost headline", "MATURE_60Y: S${{TARGET_MATURE_COST|f0}}/t for a 10th-of-a-kind plant over {{LIVES_LONG|f0}} y with gas at "
                      "S${{MATURE_60Y_GAS_PRINTED_S_GJ}}/GJ, above the tested S${{GAS_LOW|f0}} to S${{GAS_HIGH|f0}}/GJ: L5 S${{MATURE_60Y_L5_AT_PRINTED}}/t, DCF "
                      "S${{MATURE_60Y_DCF_AT_PRINTED}}/t; central gas S${{GAS_PRICE_S_GJ|f0}}/GJ"),
    ("Co-funding", "S${{COFUND_L5_25Y_SM}}M ({{COFUND_L5_25Y_SHARE_PCT}}%) at {{LIVES_SHORT|f0}} y and "
                   "S${{COFUND_L5_60Y_SM}}M ({{COFUND_L5_60Y_SHARE_PCT}}%) at {{LIVES_LONG|f0}} y bring BOAK to "
                   "S${{COFUND_TARGET_S_T|f0}}/t on L5"),
    ("Hydrogen cost", "S${{LCOH_NUSH2_BOAK_25Y}}/kg at BOAK against S${{LCOH_SMR_25Y}}/kg unabated, {{LIVES_SHORT|f0}} y"),
    ("Reactor capital", "S${{REACTOR_CAPITAL_BOAK_N1P00_SM|f0}}M at BOAK, S${{REACTOR_CAPITAL_FOAK_N1P00_SM|f0}}M at FOAK"),
]


# ---------------------------------------------------------------- units and rich text

def load_units():
    with UNITS.open(encoding="utf-8", newline="") as f:
        return {r["token"]: (r["latex"], r["style"]) for r in csv.DictReader(f)}


UNIT_LATEX = load_units()
SUFFIX = {"MWth": "MWth", "MWe": "MWe", "MW": "MW", "Mt CO2e/y": "MtCO2e/y", "Mt/y": "Mt/y", "t CO2/y": "tCO2/y",
          "t/y": "t/y", "Nm3/h": "Nm3/h", "MMSCFD": "MMSCFD", "GJ/t": "GJ/t", "kg CO2 per kg H2": "kg/kg", "kg/kg": "kg/kg",
          "wt%": "wt%", "K": "K", "C": "degC", "y": "y", "bar": "bar"}
SUFFIX_RE = re.compile(r"(?<=\d)[  ](" + "|".join(re.escape(k) for k in sorted(SUFFIX, key=len, reverse=True)) + r")(?![\w/%])")
MONEY_RE = re.compile(r"(US|S)\$([−-]?[\d,]+(?:\.\d+)?)(M/y|M|/t|/kg|/GJ|/kWt)(?![\w])")
MONEY_UNIT = {("S", "M/y"): "S$M/y", ("S", "M"): "S$M", ("S", "/t"): "S$/t", ("S", "/kg"): "S$/kg", ("S", "/GJ"): "S$/GJ",
              ("US", "/kWt"): "US$/kWt"}
MARK = re.compile(r"\[\[([^|\]]+)(?:\|([^\]]*))?\]\]")


def mark_units(s):
    """Wrap physical units in [[unit]] and priced units in [[unit|value]] markers."""
    s = s.replace("wt%", " wt%").replace("  wt%", " wt%")
    s = MONEY_RE.sub(lambda m: f"[[{MONEY_UNIT[(m.group(1), m.group(3))]}|{m.group(2)}]]" if (m.group(1), m.group(3)) in MONEY_UNIT
                     else m.group(0), s)
    return SUFFIX_RE.sub(lambda m: " [[" + SUFFIX[m.group(1)] + "]]", s)


def unit_latex(unit, value=None):
    latex, style = UNIT_LATEX[unit]
    if style == "suffix":
        return latex
    lead = {"money": r"S\$", "usmoney": r"US\$", "euromoney": r"EUR\ "}[style]
    val = (value or "").replace("-", "−")
    sign = ""
    if val.startswith("−"):
        sign, val = "−", val[1:]
    return "\\mathrm{" + sign + lead + val + latex + "}"


def add_rich(par, text, size=10.5, bold=False, italic=False, color=None, font=FONT, units=True):
    """Plain text with chemical subscripts, plus units as native Word math. units=False keeps the text
    as written, for slide titles that must read as the slide prints them."""
    s = mark_units(text) if units else text
    pos = 0
    for m in MARK.finditer(s):
        add_plain(par, s[pos:m.start()], size, bold, italic, color, font)
        el = omml.omath(unit_latex(m.group(1), m.group(2)), int(round(size * 2)), bold)
        if color:  # match the surrounding text; w:color sits before w:sz in the schema order
            for rpr in el.iter(qn("w:rPr")):
                c = OxmlElement("w:color")
                c.set(qn("w:val"), color)
                sz = rpr.find(qn("w:sz"))
                if sz is not None:
                    sz.addprevious(c)
                else:
                    rpr.append(c)
        par._p.append(el)
        pos = m.end()
    add_plain(par, s[pos:], size, bold, italic, color, font)


def add_plain(par, text, size, bold, italic, color, font):
    if not text:
        return
    for piece, base, _ in bd.segments(bd.tidy(text)):
        r = par.add_run(piece)
        set_font(r, font, size, bold, italic, color)
        if base:
            r.font.subscript = True


def set_font(run, font, size, bold=False, italic=False, color=None):
    f = run.font
    f.name, f.size, f.bold, f.italic = font, Pt(size), bold or None, italic or None
    if color:
        f.color.rgb = RGBColor.from_string(color)
    rpr = run._r.get_or_add_rPr()
    fonts = rpr.find(qn("w:rFonts"))
    if fonts is None:
        fonts = OxmlElement("w:rFonts")
        rpr.insert(0, fonts)
    for a in ("w:ascii", "w:hAnsi", "w:cs", "w:eastAsia"):
        fonts.set(qn(a), font)


# ---------------------------------------------------------------- document scaffolding

def new_doc(title, subtitle):
    doc = Document()
    sec = doc.sections[0]
    sec.page_width, sec.page_height = Mm(210), Mm(297)
    sec.left_margin = sec.right_margin = Mm(20)
    sec.top_margin, sec.bottom_margin = Mm(18), Mm(18)
    normal = doc.styles["Normal"]
    normal.font.name, normal.font.size = FONT, Pt(10.5)
    normal.font.color.rgb = RGBColor.from_string(bd.INK)
    normal.element.get_or_add_rPr()
    rfonts = normal.element.rPr.find(qn("w:rFonts"))
    if rfonts is None:
        rfonts = OxmlElement("w:rFonts")
        normal.element.rPr.insert(0, rfonts)
    for a in ("w:ascii", "w:hAnsi", "w:cs", "w:eastAsia"):
        rfonts.set(qn(a), FONT)
    normal.paragraph_format.space_after = Pt(6)
    normal.paragraph_format.line_spacing = 1.2
    for name, size, color, before in (("Heading 1", 15, bd.HEAD, 14), ("Heading 2", 12, bd.DEEP, 10)):
        st = doc.styles[name]
        st.font.name, st.font.size, st.font.bold = HEAD_FONT, Pt(size), False
        st.font.color.rgb = RGBColor.from_string(color)
        st.element.get_or_add_rPr()
        rf = st.element.rPr.find(qn("w:rFonts"))
        if rf is None:
            rf = OxmlElement("w:rFonts")
            st.element.rPr.insert(0, rf)
        for a in ("w:ascii", "w:hAnsi", "w:cs", "w:eastAsia"):
            rf.set(qn(a), HEAD_FONT)
        for a in ("w:asciiTheme", "w:hAnsiTheme", "w:eastAsiaTheme", "w:cstheme"):
            if rf.get(qn(a)) is not None:
                del rf.attrib[qn(a)]
        st.paragraph_format.space_before = Pt(before)
        st.paragraph_format.space_after = Pt(4)
        st.paragraph_format.keep_with_next = True
    p = doc.add_paragraph()
    set_font(p.add_run(title), BLACK_FONT, 24, color=bd.HEAD)
    p.paragraph_format.space_after = Pt(2)
    p = doc.add_paragraph()
    set_font(p.add_run(subtitle), HEAD_FONT, 12, color=bd.INK)
    p.paragraph_format.space_after = Pt(2)
    footer(sec)
    return doc


BLACK_FONT = "Segoe UI Black"


def footer(sec):
    p = sec.footer.paragraphs[0]
    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    set_font(p.add_run("NUS-H2  "), BLACK_FONT, 8, color=bd.DEEP)
    run = p.add_run()
    set_font(run, FONT, 8, color=bd.MUTED)
    for kind, text in (("begin", None), (None, " PAGE "), ("separate", None), (None, "1"), ("end", None)):
        if kind:
            el = OxmlElement("w:fldChar")
            el.set(qn("w:fldCharType"), kind)
            run._r.append(el)
        elif text.strip() == "PAGE":
            el = OxmlElement("w:instrText")
            el.set(qn("xml:space"), "preserve")
            el.text = text
            run._r.append(el)
        else:
            el = OxmlElement("w:t")
            el.text = text
            run._r.append(el)


def para(doc, text, size=10.5, bold=False, italic=False, color=None, after=6, style=None):
    p = doc.add_paragraph(style=style)
    add_rich(p, text, size, bold, italic, color)
    p.paragraph_format.space_after = Pt(after)
    return p


def shade(cell, fill):
    tcpr = cell._tc.get_or_add_tcPr()
    shd = OxmlElement("w:shd")
    shd.set(qn("w:val"), "clear")
    shd.set(qn("w:color"), "auto")
    shd.set(qn("w:fill"), fill)
    tcpr.append(shd)


def table(doc, head, rows, widths_mm, align=None, size=9.5, verbatim=()):
    t = doc.add_table(rows=1 + len(rows), cols=len(head))
    t.style = doc.styles["Table Grid"]
    t.alignment = WD_TABLE_ALIGNMENT.CENTER
    for j, w in enumerate(widths_mm):
        for row in t.rows:
            row.cells[j].width = Mm(w)
    for i, row in enumerate([head] + rows):
        for j, text in enumerate(row):
            cell = t.cell(i, j)
            p = cell.paragraphs[0]
            p.paragraph_format.space_after = Pt(1)
            p.paragraph_format.space_before = Pt(1)
            p.paragraph_format.line_spacing = 1.0
            if align and align[j] == "r":
                p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
            if i == 0:
                shade(cell, bd.DEEP)
                add_rich(p, text, size, color=bd.WHITE, font=HEAD_FONT)
            else:
                if i % 2 == 0:
                    shade(cell, "F7F4FD")
                add_rich(p, text, size, units=j not in verbatim)
    doc.add_paragraph().paragraph_format.space_after = Pt(2)
    return t


def status_line(book):
    s = bd.status_text(book)
    return "Status of the numbers: " + s[0].lower() + s[1:] + ". "


def fmt_time(seconds):
    s = int(round(seconds))
    return f"{s // 60}:{s % 60:02d}"


def slide_refs(book, ids):
    pages = {s["id"]: i + 1 for i, s in enumerate(book.slides)}
    pages.update({f"QA{k}": len(book.slides) + k for k in range(1, 17)})
    main = [str(pages[i]) for i in ids if i.startswith("S")]
    backup = [str(pages[i]) for i in ids if i.startswith("B")]
    qa = [str(pages[i]) for i in ids if i.startswith("QA")]
    parts = []
    if main:
        parts.append(("Slide " if len(main) == 1 else "Slides ") + join_list(main))
    if backup:
        parts.append(("backup " if len(backup) == 1 else "backups ") + join_list(backup))
    if qa:
        parts.append(("Q&A slide " if len(qa) == 1 else "Q&A slides ") + join_list(qa))
    return "; ".join(parts) + "."


def join_list(items):
    return items[0] if len(items) == 1 else ", ".join(items[:-1]) + " and " + items[-1]


def on_screen(book, s):
    """The slide's large number, from the storyboard's Large number field, when it carries tokens."""
    field = s.get("Large number", "")
    if "`" not in field:
        return ""
    first = re.split(r"(?<=[a-z0-9)%])\. (?=[A-Z])", field.strip(), maxsplit=1)[0].rstrip(".")
    first = re.sub(r", in (?:the chip|equal type)$", "", first)  # layout notes in the storyboard
    first = re.sub(r"^[A-Z][^`:]*: ", "", first)
    return book.r(first.replace("`", ""))


def pacing(book):
    m = re.search(r"at (\d+) words a minute on resolved notes, plus (\d+) s for each slide change", book.text)
    if not m:
        raise SystemExit("pacing rule not found in the storyboard, section 3.3")
    return int(m.group(1)), int(m.group(2))


# ---------------------------------------------------------------- speaker script

def write_pacing(book, rows, total):
    """Refresh the pacing table of storyboard section 3.3 from the resolved notes."""
    acts, order = {}, []
    for s, note, start, dur in rows:
        if s["act"] not in acts:
            acts[s["act"]] = [[], 0, 0.0]
            order.append(s["act"])
        acts[s["act"]][0].append(s["id"])
        acts[s["act"]][1] += len(note.split())
        acts[s["act"]][2] += dur
    lines = ["<!-- pacing -->", "| Act | Slides | Words | Time |", "|---|---|---|---|"]
    for a in order:
        ids, words, dur = acts[a]
        span = ids[0] if len(ids) == 1 else f"{ids[0]} to {ids[-1]}"
        lines.append(f"| {a} | {span} | {words} | {fmt_time(dur)} |")
    lines.append(f"| Total | {len(rows)} slides | {sum(v[1] for v in acts.values())} | {fmt_time(total)} |")
    lines.append("")
    lines.append(f"Last run: {len(book.slides)} storyboard slides plus 16 Q&A slides, {len(book.tok)} tokens loaded, "
                 f"every storyboard guard passed.")
    lines.append("<!-- /pacing -->")
    text = bd.STORYBOARD.read_text(encoding="utf-8")
    new = re.sub(r"<!-- pacing -->.*?<!-- /pacing -->", lambda m: "\n".join(lines), text, flags=re.S)
    if new != text:
        bd.STORYBOARD.write_text(new, encoding="utf-8", newline="\n")


def build_script(book):
    wpm, change = pacing(book)
    main = [s for s in book.slides if s["id"].startswith("S")]
    backup = [s for s in book.slides if s["id"].startswith("B")]
    rows, t = [], 0.0
    for s in main:
        note = bd.resolved_note(book, s)
        dur = len(note.split()) / wpm * 60 + change
        rows.append((s, note, t, dur))
        t += dur
    total = t
    write_pacing(book, rows, total)
    names = [n.strip() for n in book.r(next(s for s in book.slides if s["id"] == "S44")["Slide text"]).split(".") if n.strip()]

    doc = new_doc("NUS-H2 speaker script", TITLE)
    if names != [bd.AUTHOR]:
        raise SystemExit("S44 must name the author alone")
    para(doc, bd.AUTHOR, 10.5, color=bd.MUTED, after=12)
    para(doc, f"The script follows the deck slide by slide, and the words are the deck's speaker notes. Every figure in them "
              f"is read from the model results when the deck is built, so the spoken numbers match the slides. Times assume "
              f"{wpm} words a minute plus {change} s for each slide change, the pacing rule of the storyboard. The "
              f"{len(main)} main slides run {fmt_time(total)} against the 20 minute target, which leaves the 10 minutes of "
              f"questions intact. Backup slides follow the close, and the 16 Q&A slides come last. Both are for questions only.")
    para(doc, status_line(book) + book.r(VALIDATION_LINE) + " Say so if asked.", color=bd.MUTED)

    doc.add_heading("Running order", level=1)
    acts, order = {}, []
    for s, note, start, dur in rows:
        if s["act"] not in acts:
            acts[s["act"]] = [start, 0.0]
            order.append(s["act"])
        acts[s["act"]][1] += dur
    table(doc, ["Slide", "Title", "Act", "Starts", "Length"],
          [[str(i + 1), bd.plain(book.r(s["Title"])), s["act"], fmt_time(start), f"{int(round(dur))} s"]
           for i, (s, note, start, dur) in enumerate(rows)] + [["", "Total", "", "", fmt_time(total)]],
          [14, 86, 34, 18, 18], align=["r", "l", "l", "r", "r"], size=9, verbatim=(1,))

    doc.add_heading("Script", level=1)
    current = None
    for i, (s, note, start, dur) in enumerate(rows):
        if s["act"] != current:
            current = s["act"]
            a_start, a_len = acts[current]
            p = doc.add_paragraph()
            add_rich(p, f"{current.upper()}   {fmt_time(a_start)} to {fmt_time(a_start + a_len)}", 9, color=bd.DEEP, font=HEAD_FONT)
            p.paragraph_format.space_before = Pt(12)
            p.paragraph_format.space_after = Pt(0)
            p.paragraph_format.keep_with_next = True
        h = doc.add_heading(level=2)
        add_rich(h, f"{i + 1}. {bd.plain(book.r(s['Title']))}", 12, color=bd.DEEP, font=HEAD_FONT, units=False)
        meta = f"{fmt_time(start)} to {fmt_time(start + dur)}, {int(round(dur))} s."
        shown = on_screen(book, s)
        if shown:
            meta += f" On screen: {shown}."
        p = para(doc, meta, 9, color=bd.MUTED, after=3)
        p.paragraph_format.keep_with_next = True
        para(doc, note, 11, after=8)

    doc.add_heading("Backup slides, for questions", level=1)
    para(doc, "These slides follow the close and are shown only when a question needs one. Open it, read its line, then "
              "return to the close.",
         color=bd.MUTED)
    for s in backup:
        n = book.slides.index(s) + 1
        h = doc.add_heading(level=2)
        add_rich(h, f"{n}. {bd.plain(book.r(s['Title']))}", 12, color=bd.DEEP, font=HEAD_FONT, units=False)
        para(doc, bd.resolved_note(book, s), 11, after=8)

    doc.add_heading("Q&A slides, at the very back", level=1)
    para(doc, "Sixteen slides in their own format, with no footer and no slide number. Open the one a question needs and "
              "answer from it. Slide 16 carries the engine's co-funding solve.", color=bd.MUTED)
    table(doc, ["Slide", "Question on the slide"], qa_rows(book), [16, 154], align=["r", "l"], size=9.5)

    props(doc, "NUS-H2 speaker script")
    save_scrubbed(doc, SCRIPT)
    return total, len(main)


# ---------------------------------------------------------------- Q&A sheet

def run_qa_guards(book):
    env = book.env()
    env.update({k: v[0] for k, v in book.new.items()})
    env.update({k: v["text"] for k, v in book.tok.items() if k.endswith("_RESULT")})
    env["abs"] = abs
    failed = [f"{qid}: {expr}" for qid, expr in QA_GUARDS if not eval(expr, {"__builtins__": {}}, env)]  # guard table above
    if failed:
        raise SystemExit("Q&A guard failed, rewrite the named answers before building:\n  " + "\n  ".join(failed))
    print(f"Q&A guards: {len(QA_GUARDS)} evaluated, 0 failed")


def build_qa(book):
    run_qa_guards(book)
    doc = new_doc("NUS-H2 Q&A preparation", TITLE)
    para(doc, "Short answers to the questions the brief lists, with the line to give if pressed and the slides to open. "
              "Every figure is read from the model results or the published inputs when this sheet is built.", after=6)
    para(doc, status_line(book) + book.r(VALIDATION_LINE) +
         " Sensitivities move one input at a time, and corner runs carry no probability.", color=bd.MUTED, after=8)

    doc.add_heading("Numbers to have ready", level=1)
    table(doc, ["Item", "Central value"], [[a, book.r(b)] for a, b in QUICK_REF], [44, 126], size=9.5)

    doc.add_heading("Q&A slides at the back of the deck", level=1)
    para(doc, "Each answers one question directly, so open it when that question comes up.", color=bd.MUTED, after=4)
    table(doc, ["Slide", "Question on the slide"], qa_rows(book), [16, 154], align=["r", "l"], size=9.5)

    doc.add_heading("Questions", level=1)
    for i, item in enumerate(QA_ITEMS, 1):
        h = doc.add_heading(level=2)
        add_rich(h, f"Q{i}. {book.r(item['q'])}", 12, color=bd.DEEP, font=HEAD_FONT)
        p = doc.add_paragraph()
        add_rich(p, book.r(item["short"]), 11, bold=True)
        p.paragraph_format.space_after = Pt(4)
        for text in item.get("body", []):
            para(doc, book.r(text), 10.5, after=4)
        for text in item.get("bullets", []):
            p = doc.add_paragraph(style="List Bullet")
            add_rich(p, book.r(text), 10.5)
            p.paragraph_format.space_after = Pt(2)
        p = doc.add_paragraph()
        add_rich(p, "If pressed: ", 10.5, bold=True, color=bd.RED)
        add_rich(p, book.r(item["pressed"]), 10.5)
        p.paragraph_format.space_after = Pt(3)
        p = doc.add_paragraph()
        add_rich(p, "Show: " + slide_refs(book, item["show"]), 9.5, color=bd.MUTED)
        p.paragraph_format.space_after = Pt(6)

    props(doc, "NUS-H2 Q&A preparation")
    save_scrubbed(doc, QA)


def qa_rows(book):
    """Slide numbers and titles of the appended Q&A slides, as the deck build recorded them."""
    pages = json.loads((bd.RENDER / "deck_pages.json").read_text(encoding="utf-8"))
    titles = pages["qa_titles"]
    if len(titles) != 16 or len(pages["ids"]) != len(book.slides) + 16:
        raise SystemExit("deck_pages.json does not match the storyboard; build the deck first")
    return [[str(len(book.slides) + k), t] for k, t in enumerate(titles, 1)]


def save_scrubbed(doc, path):
    """Save, then drop theme font entries that spell a name (build_deck.scrub_zip)."""
    doc.save(path)
    blob, _ = bd.scrub_zip(path.read_bytes())
    path.write_bytes(blob)


def props(doc, title):
    cp = doc.core_properties
    cp.title = title
    cp.subject = TITLE
    cp.author = bd.AUTHOR
    cp.last_modified_by = bd.AUTHOR
    cp.keywords = cp.comments = cp.category = cp.content_status = cp.identifier = cp.version = ""
    cp.revision = 1
    cp.created = cp.modified = bd.datetime.datetime.utcnow().replace(microsecond=0)


# ---------------------------------------------------------------- checks

def scan(path):
    """Read the saved file back: forbidden terms, withdrawn figures, the bare abbreviation, stray tokens,
    em and en dashes, names other than the author's, and every XML part including the core properties."""
    with zipfile.ZipFile(path) as z:
        parts = {n: z.read(n) for n in z.namelist()}
    xml = parts["word/document.xml"].decode("utf-8")
    text = " ".join(re.findall(r"<(?:w|m):t(?: [^>]*)?>([^<]*)</(?:w|m):t>", xml))
    problems = []
    for rx in (bd.FORBIDDEN, bd.WITHDRAWN, bd.BARE_SMR, bd.S96_BASE, bd.EN_DASH):
        for m in rx.finditer(text):
            problems.append(f"{path.name}: {m.group(0)!r} in ...{text[max(0, m.start() - 40):m.end() + 40]}...")
    for pos in bd.name_hits(text):
        problems.append(f"{path.name}: a name other than the author at offset {pos}")
    problems += bd.scan_parts(parts, path.name)
    core = parts["docProps/core.xml"].decode("utf-8")
    if f"<dc:creator>{bd.AUTHOR}</dc:creator>" not in core or f"<cp:lastModifiedBy>{bd.AUTHOR}</cp:lastModifiedBy>" not in core:
        problems.append(f"{path.name}: core properties are not the author's alone")
    if any(n.startswith("word/comments") for n in parts):
        problems.append(f"{path.name}: comments part present")
    n_math = xml.count("<m:oMath")
    return problems, n_math, len(text.split())


def export_pdf(docx, pdf):
    code = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(HERE / "word_run.ps1"),
                           "-InDocx", str(docx), "-OutPdf", str(pdf), "-TimeoutSec", "240"]).returncode
    if code != 0:
        raise SystemExit(f"Word export of {docx.name} failed with code {code}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pdf", action="store_true", help="export both documents to presentation/render/ for review")
    args = ap.parse_args()
    if sys.stdout.encoding.lower() != "utf-8":
        sys.stdout.reconfigure(encoding="utf-8")
    book = bd.Book()
    total, n_main = build_script(book)
    build_qa(book)
    problems = []
    for path in (SCRIPT, QA):
        p, n_math, words = scan(path)
        problems += p
        print(f"{path.name}: {words} words, {n_math} unit math objects")
    print(f"script: {n_main} main slides, {fmt_time(total)}")
    for line in problems:
        print(" -", line)
    if problems:
        raise SystemExit("scan problems above")
    if args.pdf:
        bd.RENDER.mkdir(exist_ok=True)
        for path in (SCRIPT, QA):
            export_pdf(path, bd.RENDER / (path.stem + ".pdf"))


if __name__ == "__main__":
    main()
