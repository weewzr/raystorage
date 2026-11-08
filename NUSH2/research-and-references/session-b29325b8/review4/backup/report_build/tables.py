#!/usr/bin/env python3
"""Table specifications. Every cell is inline markup, so every number is a token.

A spec is a dict: caption, widths in mm, header (list of cell lists), rows (list of cell lists),
align per column (l, r, c), bold (row indices), rule (row indices with a line above), keep (keep
the table on one page).
"""
import csv
from collections import OrderedDict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def t(name, fmt="t"):
    return "{{" + name + "|" + fmt + "}}"


def ledger_rows():
    return [("L0", "Gas saved", "GAS"), ("L0", "Reactor capital", "REACTOR_CAP"), ("L0", "Capture capital", "CAPTURE_CAP"),
            ("L0", "Capture fixed O&M", "CAPTURE_FOM"), ("L0", "CO₂ transport and storage", "TS"),
            ("L1", "Reactor O&M and fuel", "REACTOR_OM"), ("L2", "IHX and secondary loop", "IHX"),
            ("L3", "Extra licensing", "LICENCE"), ("L3", "Decommissioning", "DECOM"),
            ("L4", "Interest during construction", "IDC"), ("L4", "Power island", "POWER_ISLAND"),
            ("L4", "Recycle and cryogenic train", "RECYCLE_TRAIN"), ("L5", "Lost steam export", "LOST_STEAM"),
            ("L5", "Baseline purchased power avoided", "BASE_POWER")]


def build():
    specs = {}
    keys = ("BOAK_25Y", "BOAK_60Y", "FOAK_25Y", "FOAK_60Y")
    head4 = ["BOAK,\n25~[[y]]", "BOAK,\n60~[[y]]", "FOAK,\n25~[[y]]", "FOAK,\n60~[[y]]"]

    specs["t_basis"] = dict(
        caption="Headline Case B at the SOXAL rate beside the INL reference Case A, at {{AVAILABILITY_PCT|f0}}% availability.",
        widths=[62, 32, 33, 32], align="lrrr", keep=True,
        header=[["Quantity", "Case B, headline", "Case A, INL Selexol", "Case A, with MEA"]],
        rows=[["H₂ product, [[kg/h]]", t("H2_B_KG_H"), t("RES_H2_KG_H_AINL"), t("RES_H2_KG_H_AMEA")],
              ["Annual H₂, [[tH2/y]]", t("H2_B_T_Y"), t("H2_A_T_Y"), t("RES_H2_T_Y_AMEA")],
              ["Natural gas, [[MMSCFD]]", t("RES_NG_MMSCFD"), t("RES_NG_MMSCFD_AINL"), t("RES_NG_MMSCFD_AMEA")],
              ["Process heat, [[MWth]]", t("RES_PROCESS_HEAT_MW"), t("RES_PROCESS_HEAT_MW_AINL"), t("RES_PROCESS_HEAT_MW_AMEA")],
              ["Process electricity, [[MWe]]", t("RES_E_PROCESS_MWE"), t("RES_E_PROCESS_MWE_AINL"), t("RES_E_PROCESS_MWE_AMEA")],
              ["Helium circulators, [[MWe]]", t("E_CIRC_TOTAL_B_MWE"), t("E_CIRC_TOTAL_AINL_MWE"), t("E_CIRC_TOTAL_AMEA_MWE")],
              ["Reactor needed, [[MWth]]", t("RES_REACTOR_NEEDED_MWTH"), t("RES_REACTOR_NEEDED_MWTH_AINL"), t("RES_REACTOR_NEEDED_MWTH_AMEA")],
              ["Rating, [[MWth]]", t("REACTOR_RATED_MWTH"), t("REACTOR_RATED_MWTH_AINL"), t("REACTOR_RATED_MWTH_AMEA")],
              ["Surplus or shortfall, [[MWth]]", t("RES_MARGIN_210_MWTH"), t("RES_MARGIN_210_MWTH_AINL"), t("RES_MARGIN_210_MWTH_AMEA")],
              ["Captured CO₂, [[tCO2/y]]", t("RES_CAPTURED_T_Y"), t("RES_CAPTURED_T_Y_AINL"), t("RES_CAPTURED_T_Y_AMEA")],
              ["Net abatement, [[tCO2e/y]]", t("RES_ABATEMENT_T_Y"), t("RES_ABATEMENT_T_Y_AINL"), t("RES_ABATEMENT_T_Y_AMEA")]])

    specs["t_heat"] = dict(
        caption="Heat and power balance, Case B central. Purge recovery is Case 6 fuel-gas recycle and nitrogen removal equipment, split not published; INL's {{E_PURGE|f1}}~[[MWe]] is at Case A [@r8].",
        widths=[56, 20, 63, 20], align="lrlr", keep=True,
        header=[["Heat term", "[[MWth]]", "Electricity term", "[[MWe]]"]],
        rows=[["Process heat to helium", t("Q_PROCESS_B_MWTH"), "Lean pump, CO₂ compression, purge recovery", t("EB_B_REC_F_PUMP_COMPRESS")],
              ["CO₂ slip sensible heat", t("RES_SLIP_HEAT_MW"), "Cooling towers and MEA fans", t("EB_B_REC_F_COOLING")],
              ["IHX loss", t("EB_B_REC_F_IHX_LOSS"), "Other process loads", t("E_OTHER_B_MWE")],
              ["Duct and vessel loss, less circulator heat", t("NET_DUCT_VESSEL_SHAFT_MWTH"), "Process subtotal", t("E_PROCESS_B_MWE")],
              ["Reboiler gap", t("REBOILER_GAP_B_MW"), "Helium circulators", t("EB_B_REC_F_HE_CIRCULATION")],
              ["Heat to electricity", t("HEAT_TO_POWER_MWTH"), "Total generated on site", t("E_TOTAL_B_MWE")],
              ["Reactor needed", t("R_NEEDED_B_MWTH"), "Unmet electricity", t("EB_B_REC_F_UNMET_ELECTRICAL")],
              ["Rating", t("R_RATED_MWTH"), "Unmet heat at rating", t("EB_B_REC_F_UNMET_THERMAL")],
              ["Spare, no revenue", t("MARGIN_B_MWTH"), "", ""]],
        bold=[6])

    specs["t_risk"] = dict(
        caption="Principal risks and constraints.",
        widths=[29, 67, 63], align="lll", keep=False,
        header=[["Area", "Risk or constraint", "Mitigation or requirement"]],
        rows=[["Technology", "No commercial nuclear process heat for hydrogen; IHX unqualified for 900 to 925[[degC]] helium.", "Overseas demonstration and component qualification before any Singapore commitment."],
              ["Hot-end approach", "INL's 29~[[K]] approach is untested; JAERI measured 102 to 125~[[K]] [@jaeri2005].", "Test the approach during IHX qualification."],
              ["Recycle with MEA", "Ten times INL's CO₂ slip enters the cryogenic unit, unmodelled by INL [@r8].", "Pilot the recycle; without it abatement is {{A_NORECYCLE_MT|f3}}~[[Mt/y]]."],
              ["Outage and backup", "No furnace, so hydrogen stops in reactor outages.", "Offtake for {{AVAILABILITY_PCT|f0}}% of hours, or a priced backup furnace."],
              ["Safety and siting", "A dose-based, design-specific EPZ must be justified on a compact site.", "Safety case and EPZ analysis before site approval."],
              ["Regulation", "No licensing pathway exists for a reactor supplying industrial heat.", "INIR review, then one coordinated pathway."],
              ["Economics", "Central full cost is {{AC_L5_BOAK_60Y|u}} to {{AC_L5_FOAK_25Y|u}}, above the target.", "Overseas replication and contracted costs before investment (Section 4.7)."],
              ["Storage and offtake", "Storage depends on Singapore-Indonesia cooperation; offtake is uncontracted.", "Contracted CO₂ chain and hydrogen offtake before investment."],
              ["Storage accounting", "Treatment of offshore storage across the border is unresolved.", "Settle under the Singapore-Indonesia arrangement [@r30]."]])

    specs["t_carbon"] = dict(
        caption="Carbon ledger, Case B, direct emissions to atmosphere on the INL baseline.",
        widths=[105, 34], align="lr", keep=True, center=True,
        header=[["Line", "[[tCO2e/y]]"]],
        rows=[["Same-size unabated SMR baseline", t("BASELINE_B_T_Y")],
              ["Less gas saved by nuclear heat and recycle", t("GAS_SAVED_C_T_Y")],
              ["Feed carbon, fresh natural gas", t("FEED_C_T_Y")],
              ["Captured and stored", t("CAPTURED_B_T_Y")],
              ["Carbon in product methane, not counted at the plant", t("PRODUCT_C_T_Y")],
              ["To atmosphere, burned nitrogen-rich cut", t("ATM_B_T_Y")],
              ["Net abatement, baseline less atmosphere", t("A_ATM_B_T")]],
        bold=[6], rule=[6])

    specs["t_absens"] = dict(
        caption="Net abatement sensitivities, [[MtCO2e/y]]. The target is {{TARGET_ABATEMENT_MT|f2}}.",
        widths=[62, 17, 63, 17], align="lrlr", keep=True,
        header=[["Case", "Result", "Case", "Result"]],
        rows=[["Per-pass capture 80%", t("A_CAP80_MT"), "Baseline, Cho {{BASELINE_CHO_KG_KG|f2}}", t("A_CHO_B_MT")],
              ["Per-pass capture 85%", t("A_CAP85_MT"), "Baseline, IEAGHG {{BASELINE_IEAGHG_KG_KG|f2}}", t("A_IEAGHG_B_MT")],
              ["Per-pass capture 95%", t("A_CAP95_MT"), "Ship fuel burden counted", t("A_TS_BURDEN_B_MT")],
              ["Availability 65%", t("A_AV65_MT"), "Lifecycle, burdens only", t("A_LC_BURDENS_B_MT")],
              ["Availability 75%", t("A_AV75_MT"), "Lifecycle, with saved upstream gas", t("A_LC_UPSTREAM_B_MT")],
              ["Availability 90%", t("A_AV90_MT"), "System view, with lost steam", t("A_SYSTEM_B_MT")],
              ["Selexol reference", t("A_SELEXOL_MT"), "No recycle, tail gas burned", t("A_NORECYCLE_MT")],
              ["aMDEA", t("A_AMDEA_MT"), "Capture offline", t("NEVER_GENERATED_MT")]])

    specs["t_head"] = dict(
        caption="Headline rebuilt cost by both methods, $n = 1.0$, {{R_RATED_MWTH|f0}}~[[MWth]] charged, against the unabated SMR.",
        widths=[63, 24, 24, 24, 24], align="lrrrr", keep=True,
        header=[["Measure"] + head4],
        rows=[["Full cost, ledger L5, [[S$/tCO2e]]"] + [t("AC_L5_" + k) for k in keys],
              ["Full cost, DCF LAC, [[S$/tCO2e]]"] + [t("LAC_DCF_" + k) for k in keys],
              ["DCF less L5, %"] + [t("LAC_L5_GAP_PCT_" + k, "f2") for k in keys],
              ["Over the target, L5, [[S$/tCO2e]]"] + [t("RES_TARGET_COST_EXCESS_L5_" + k) for k in keys],
              ["LCOH, NUS-H₂, [[S$/kg]]"] + [t("LCOH_NUSH2_" + k) for k in keys],
              ["LCOH, unabated SMR, [[S$/kg]]"] + [t("LCOH_SMR_" + k) for k in keys],
              ["LCOH premium, [[S$/kg]]"] + [t("LCOH_PREMIUM_" + k) for k in keys],
              ["Named step L3, [[S$/tCO2e]]"] + [t("AC_L3_" + k) for k in keys]],
        bold=[0, 1], rule=[4, 7])

    rows = []
    bold, rule = [], []
    for lvl, name, line in ledger_rows():
        rows.append([lvl, name] + [t(f"LINE_{line}_{k}_N1P00") for k in keys])
        if line == "DECOM":
            rule.append(len(rows))
            bold.extend([len(rows), len(rows) + 1])
            rows.append(["L3", "Named step, [[S$M/y]]"] + [t("COST_L3_" + k) for k in keys])
            rows.append(["", "Named step, [[S$/tCO2e]]"] + [t("AC_L3_" + k) for k in keys])
    rule.append(len(rows))
    bold.extend([len(rows), len(rows) + 1])
    rows.append(["L5", "Full cost, [[S$M/y]]"] + [t("COST_L5_" + k) for k in keys])
    rows.append(["", "Full cost, [[S$/tCO2e]]"] + [t("AC_L5_" + k) for k in keys])
    specs["t_ladder"] = dict(
        caption="Cost ladder, annual lines in [[S$M/y]] against the unabated SMR, $n = 1.0$, {{R_RATED_MWTH|f0}}~[[MWth]] charged, {{A_ATM_B_MT|u:MtCO2e/y}}. IHX, licensing and recycle train are proxies.",
        widths=[14, 61, 21, 21, 21, 21], align="llrrrr", keep=True,
        header=[["Level", "Line"] + head4], rows=rows, bold=bold, rule=rule)

    def pair(stage, n):
        return "{{AC_L5_%s_25Y_%s}} / {{AC_L5_%s_60Y_%s}}" % (stage, n, stage, n)

    grid_rows = []
    for label, stage in (("FOAK", "FOAK"), ("Tenth unit, from FOAK", "10OAK_P47"), ("BOAK", "BOAK"), ("Tenth unit, from BOAK", "10OAK_P46")):
        grid_rows.append([label] + [pair(stage, n) for n in ("N1P00", "N0P60", "N0P31")])
    specs["t_grid"] = dict(
        caption="Rebuilt full cost L5 in [[S$/tCO2e]] at 25~[[y]] / 60~[[y]], {{R_RATED_MWTH|f0}}~[[MWth]] charged, for the three size treatments.",
        widths=[47, 37, 37, 37], align="lccc", keep=True,
        header=[["Stage", "$n = 1.0$", "$n = 0.6$", "$n = 0.31$"]], rows=grid_rows)

    specs["t_unc"] = dict(
        caption="One-at-a-time changes to full cost L5, [[S$/tCO2e]], around {{AC_L5_BOAK_25Y|f1}} at BOAK, 25~[[y]], $n = 1.0$.",
        widths=[62, 17, 63, 17], align="lrlr", keep=True,
        header=[["Change", "Result", "Change", "Result"]],
        rows=[["T&S {{TS_LOW|f1|u}}", t("SENS_TS_23P2_AC_L5_25"), "T&S {{TS_HIGH|f0|u}}", t("SENS_TS_112_AC_L5_25")],
              ["Gas {{GAS_LOW|f0|u}}", t("SENS_GAS_10_AC_L5_25"), "Gas {{GAS_HIGH|f0|u}}", t("SENS_GAS_20_AC_L5_25")],
              ["Reactor, INL low", t("AC_L5_OCC_LOW_25Y"), "Reactor, INL high", t("AC_L5_OCC_HIGH_25Y")],
              ["Availability 65%", t("SENS_AVAILABILITY_0P65_AC_L5_25"), "Availability 90%", t("SENS_AVAILABILITY_0P9_AC_L5_25")],
              ["Gas saving at {{GAS_SAVING_AHN_PCT}}%", t("AC_L5_GAS_AHN_25Y"), "Capture capital, Case 3 MEA", t("AC_L5_CAPTURE_MEA_25Y")],
              ["SRD {{SRD_LOW|f1}}", t("SENS_SRD_3P4_AC_L5_25"), "SRD {{SRD_HIGH|f1}}", t("SENS_SRD_4P2_AC_L5_25")],
              ["Selexol reference", t("SENS_SOLVENT_1_AC_L5_25"), "aMDEA", t("SENS_SOLVENT_2_AC_L5_25")],
              ["Baseline, Cho", t("SENS_BASELINE_9P35_AC_L5_25"), "Baseline, IEAGHG", t("SENS_BASELINE_8P996_AC_L5_25")],
              ["Reactor sized to need", t("SENS_CHARGED_179P04_AC_L5_25"), "No recycle, {{A_NORECYCLE_MT|f3}}~[[Mt/y]]", t("SENS_TAIL_GAS_2_AC_L5_25")]])

    specs["t_gap"] = dict(
        caption="Value each lever alone would need to reach {{TARGET_COST_S_T|f0|u}} at BOAK, $n = 1.0$.",
        widths=[36, 25, 25, 25, 48], align="lrrrl", keep=True,
        header=[["Lever", "L5, 25~[[y]]", "L5, 60~[[y]]", "L3, 25~[[y]]", "Reference"]],
        rows=[["Gas, [[S$/GJ]]", t("LEVER_GAS_L5_BOAK_25Y_S_GJ"), t("LEVER_GAS_L5_BOAK_60Y_S_GJ"), t("LEVER_GAS_L3_BOAK_25Y_S_GJ"), "Central {{GAS_CENTRAL|f0}}, high {{GAS_HIGH|f0}}"],
              ["Reactor, [[US$/kWt]]", t("LEVER_OCC_L5_BOAK_25Y_USD_KWT"), t("LEVER_OCC_L5_BOAK_60Y_USD_KWT"), t("LEVER_OCC_L3_BOAK_25Y_USD_KWT"), "Central {{OCC_CENTRAL|f0}}, low {{OCC_LOW|f0}}"],
              ["T&S, [[S$/tCO2]]", t("LEVER_TS_L5_BOAK_25Y_S_T", "f1"), t("LEVER_TS_L5_BOAK_60Y_S_T", "f1"), t("LEVER_TS_L3_BOAK_25Y_S_T", "f1"), "Central {{TS_CENTRAL|f0}}, low {{TS_LOW|f1}}"]])

    # Appendix A
    specs["t_a1"] = dict(
        caption="Design basis. Published values are from INL TEV-961 [@r8] and TEV-953 [@r10]; assumed and calculated values are labelled.",
        widths=[72, 28, 28, 31], align="lrrl", keep=False,
        header=[["Parameter", "Case B", "Case A", "Label"]],
        rows=[["Reactor outlet, [[degC]]", t("ROT", "f0"), t("ROT_AINL", "f0"), "Published"],
              ["Secondary helium supply, [[degC]]", t("T_HE_SUPPLY", "f0"), t("T_HE_SUPPLY_AINL", "f0"), "Published"],
              ["Secondary helium return, [[degC]]", t("T_HE_RETURN", "f0"), t("T_HE_RETURN_AINL", "f0"), "Published"],
              ["Secondary helium flow, [[kg/s]]", t("HE_SEC_B_KG_S"), t("HE_FLOW", "f2"), "Calculated; published"],
              ["Primary helium flow at need, [[kg/s]]", t("HE_PRIM_B_KG_S"), t("RES_HE_PRIM_KG_S_AMEA"), "Calculated"],
              ["Reformer outlet, [[degC]]", t("SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C", "f0"), t("SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C", "f0"), "Published"],
              ["Process heat, [[MWth]]", t("Q_PROCESS_B_MWTH"), t("Q_PROCESS_A_MWTH"), "Scaled; published"],
              ["Electricity demand, [[MWe]]", t("E_TOTAL_B_MWE"), t("RES_E_TOTAL_MWE_AMEA"), "Calculated, with MEA"],
              ["Power conversion efficiency, %", t("ETA_POWER_PCT", "f1"), t("ETA_POWER_PCT", "f1"), "Published, interpolated"],
              ["PSA recovery, %", t("PSA_RECOVERY_PCT", "f0"), t("PSA_RECOVERY_PCT", "f0"), "Published, external"],
              ["Availability, %", t("AVAILABILITY_PCT", "f0"), t("AVAILABILITY_PCT", "f0"), "Assumed"],
              ["Proposed CO₂ capture", "30~[[wt%]] MEA", "30~[[wt%]] MEA", "Assumed"],
              ["INL CO₂ removal", "n/a", "Selexol", "Published"],
              ["SRD, [[GJ/tCO2]]", t("SRD_CENTRAL", "f1"), t("SRD_CENTRAL", "f1"), "Assumed"],
              ["Rich and lean loading, [[mol/mol]]", "{{LOADING_RICH|f2}}, {{LOADING_LEAN|f2}}", "{{LOADING_RICH|f2}}, {{LOADING_LEAN|f2}}", "Assumed"],
              ["Reactor rating, [[MWth]]", t("R_RATED_MWTH"), t("R_RATED_MWTH"), "User decision"]])

    specs["t_walk"] = dict(
        caption="Sizing walk in [[MWth]]. Steps not exported by the results are marked n/e.",
        widths=[89, 35, 35], align="lrr", keep=True,
        header=[["Step", "Case B", "Case A, MEA"]],
        rows=[["INL's rate: process heat plus power at {{ETA_POWER_PCT|f1}}%, no purge load", t("VAL_SIZING_WALK_STEP_0_SIM"), t("VAL_SIZING_WALK_STEP_0_SIM")],
              ["Same sum at Case B", t("VAL_SIZING_WALK_STEP_1_SIM"), "n/a"],
              ["Restore purge load", t("VAL_SIZING_WALK_STEP_2_SIM"), t("VAL_SCREENING_COMPLETE_REACTOR_MWTH_SIM")],
              ["Add vessel, IHX and duct losses", t("VAL_SIZING_WALK_STEP_3_SIM"), "n/e"],
              ["Add helium circulators", t("VAL_SIZING_WALK_STEP_4_SIM"), t("R_NEEDED_A_INL_MWTH")],
              ["Swap Selexol for MEA, zero reboiler heat", t("VAL_SIZING_WALK_STEP_5_SIM"), "n/e"],
              ["Add reboiler gap at {{SRD_CENTRAL|f1}}~[[GJ/tCO2]]", t("VAL_SIZING_WALK_STEP_6_SIM"), t("R_NEEDED_A_MEA_MWTH")],
              ["Rated", t("R_RATED_MWTH"), t("R_RATED_MWTH")],
              ["Margin", t("MARGIN_B_MWTH"), t("MARGIN_A_MEA_MWTH")]],
        bold=[6], rule=[7])

    # Appendix B
    rows = []
    mb = [r for r in csv.DictReader(open(ROOT / "results" / "Integrated_Mass_Balance.csv", encoding="utf-8")) if r["run_id"] == "B_REC_F"]
    for r in mb:
        sid = r["stream_id"]
        if sid.startswith("CLOSURE") and sid != "CLOSURE_CARBON":
            continue
        name = r["stream_name"].replace("CO2", "CO₂").replace("H2", "H₂")
        rows.append([name, "yes" if r["crosses_boundary"] == "true" else "no", t(f"MB_B_REC_F_{sid}_C_KMOL_H", "f1")])
    specs["t_b_carbon"] = dict(
        caption="Carbon streams, Case B, from the integrated mass balance, kmol C per hour.",
        widths=[97, 31, 31], align="lcr", keep=True,
        header=[["Stream", "Crosses boundary", "Carbon"]], rows=rows)

    specs["t_b_steam"] = dict(
        caption="Reboiler heat stack, Case B central, [[MW]].",
        widths=[110, 30], align="lr", keep=True, center=True,
        header=[["Source, in order of use", "Duty"]],
        rows=[["Freed Selexol steam", t("SELEXOL_FREED_B_MW")], ["Export steam diverted", t("EXPORT_STEAM_B_MW")],
              ["Syngas heat above 130[[degC]]", t("EB_B_REC_F_SYNGAS_RECOVERY")], ["Reactor steam, the reboiler gap", t("REBOILER_GAP_B_MW")],
              ["Reboiler duty at {{SRD_CENTRAL|f1}}~[[GJ/tCO2]]", t("REBOILER_B_MW")]],
        bold=[4], rule=[4])

    # Appendix C
    base = "EM_B_REC_F_INL_9P21"
    rows = [["Direct", "Unabated SMR baseline", t(f"{base}_DIRECT_ATM_BASELINE_T_Y"), t(f"{base}_DIRECT_ATM_BASELINE_KG_KG")],
            ["Direct", "Captured and stored", t(f"{base}_DIRECT_ATM_CAPTURED_T_Y"), t(f"{base}_DIRECT_ATM_CAPTURED_KG_KG")],
            ["Direct", "Residual to atmosphere", t(f"{base}_DIRECT_ATM_N2_RICH_CUT_T_Y"), t(f"{base}_DIRECT_ATM_N2_RICH_CUT_KG_KG")],
            ["Direct", "Net abatement", t(f"{base}_DIRECT_ATM_ABATEMENT_T_Y"), t(f"{base}_DIRECT_ATM_ABATEMENT_KG_KG")],
            ["Lifecycle, burdens", "Nuclear lifecycle burden", t(f"{base}_LIFECYCLE_BURDENS_NUCLEAR_LC_T_Y"), t(f"{base}_LIFECYCLE_BURDENS_NUCLEAR_LC_KG_KG")],
            ["Lifecycle, burdens", "T&S burden", t(f"{base}_LIFECYCLE_BURDENS_TS_BURDEN_T_Y"), t(f"{base}_LIFECYCLE_BURDENS_TS_BURDEN_KG_KG")],
            ["Lifecycle, burdens", "Net abatement", t(f"{base}_LIFECYCLE_BURDENS_ABATEMENT_T_Y"), t(f"{base}_LIFECYCLE_BURDENS_ABATEMENT_KG_KG")],
            ["Lifecycle, upstream", "Saved upstream gas", t(f"{base}_LIFECYCLE_UPSTREAM_UPSTREAM_CREDIT_T_Y"), t(f"{base}_LIFECYCLE_UPSTREAM_UPSTREAM_CREDIT_KG_KG")],
            ["Lifecycle, upstream", "Net abatement", t(f"{base}_LIFECYCLE_UPSTREAM_ABATEMENT_T_Y"), t(f"{base}_LIFECYCLE_UPSTREAM_ABATEMENT_KG_KG")],
            ["System", "Net abatement with lost steam", t(f"{base}_SYSTEM_ABATEMENT_T_Y"), t(f"{base}_SYSTEM_ABATEMENT_KG_KG")]]
    specs["t_c_steps"] = dict(
        caption="Emissions ledger, Case B on the INL baseline. Lifecycle rows are sensitivities and stay out of the headline.",
        widths=[42, 57, 30, 30], align="llrr", keep=True,
        header=[["View", "Line", "[[tCO2e/y]]", "[[kg/kg]]"]], rows=rows)

    # Appendix D
    specs["t_d_inputs"] = dict(
        caption="Cost inputs. Label P is published, A assumed, C calculated, U a project decision.",
        widths=[64, 19, 21, 19, 36], align="lrrrl", keep=False,
        header=[["Input", "Low", "Central", "High", "Label, source"]],
        rows=[["Reactor overnight cost, [[US$2019/kWt]]", t("OCC_LOW", "f0"), t("OCC_CENTRAL", "f0"), t("OCC_HIGH", "f0"), "P, C [@inlcost]"],
              ["FOAK premium, multiple of BOAK", t("FOAK_LOW", "f1"), t("FOAK_CENTRAL", "f1"), t("FOAK_HIGH", "f1"), "P [@inlcost]"],
              ["Reactor O&M and fuel, [[US$/MWth-h]]", t("OPEX_LOW", "f0"), t("OPEX_CENTRAL", "f0"), t("OPEX_HIGH", "f0"), "P, C [@inlcost]"],
              ["Gas, [[S$/GJ]], higher heating value", t("GAS_LOW", "f0"), t("GAS_CENTRAL", "f0"), t("GAS_HIGH", "f0"), "P, A [@fredgas]"],
              ["T&S, [[S$/tCO2]]", t("TS_LOW", "f1"), t("TS_CENTRAL", "f0"), t("TS_HIGH", "f0"), "U [@r5,eria2024]"],
              ["Capture capital, [[S$M per t/h]]", "n/a", t("CAPTURE_K_1A_SM_TH", "f2"), t("CAPTURE_K_3_SM_TH", "f2"), "C [@r5]"],
              ["Capture fixed O&M, % of capital", "n/a", t("CAPTURE_FOM_PCT", "f1"), t("CAPTURE_FOM_HIGH_PCT", "f1"), "A, P [@ahn2026]"],
              ["IHX and loop line, [[S$M/y]]", t("LINE_IHX_BOAK_25Y_N1P00_LOW"), t("LINE_IHX_BOAK_25Y_N1P00"), t("LINE_IHX_BOAK_25Y_N1P00_HIGH"), "Proxy, primary not verified"],
              ["Licensing line, 25~[[y]], [[S$M/y]]", t("LINE_LICENCE_BOAK_25Y_N1P00_LOW"), t("LINE_LICENCE_BOAK_25Y_N1P00"), t("LINE_LICENCE_BOAK_25Y_N1P00_HIGH"), "Proxy, primary not verified"],
              ["Decommissioning, [[US$/kWe]]", t("DECOM_LOW", "f0"), t("DECOM_CENTRAL", "f0"), t("DECOM_HIGH", "f0"), "P [@nzw09]"],
              ["Recycle train line, [[S$M/y]]", t("LINE_RECYCLE_TRAIN_BOAK_25Y_N1P00_LOW"), t("LINE_RECYCLE_TRAIN_BOAK_25Y_N1P00"), t("LINE_RECYCLE_TRAIN_BOAK_25Y_N1P00_HIGH"), "Proxy from [@r5]"],
              ["Discount rate, %", "n/a", t("DISCOUNT_PCT", "f0"), "n/a", "A [@r5]"],
              ["Availability, %", t("AVAILABILITY_LOW_PCT", "f0"), t("AVAILABILITY_PCT", "f0"), t("AVAILABILITY_HIGH_PCT", "f0"), "A"]])

    rows = []
    for label, stage, n in (("BOAK, $n = 1.0$", "BOAK", "N1P00"), ("BOAK, $n = 0.6$", "BOAK", "N0P60"), ("BOAK, $n = 0.31$", "BOAK", "N0P31"),
                            ("FOAK, $n = 1.0$", "FOAK", "N1P00"), ("FOAK, $n = 0.6$", "FOAK", "N0P60"), ("FOAK, $n = 0.31$", "FOAK", "N0P31"),
                            ("Tenth, from BOAK, $n = 1.0$", "10OAK_P46", "N1P00"), ("Tenth, from FOAK, $n = 1.0$", "10OAK_P47", "N1P00")):
        rows.append([label, t(f"REACTOR_CAPITAL_{stage}_{n}_SM"), t(f"LINE_REACTOR_CAP_{stage}_25Y_{n}"), t(f"LINE_REACTOR_CAP_{stage}_60Y_{n}")])
    specs["t_d_reactor"] = dict(
        caption="Reactor capital at {{R_RATED_MWTH|f0}}~[[MWth]] and its annual lines.",
        widths=[73, 30, 28, 28], align="lrrr", keep=True,
        header=[["Stage and size treatment", "Capital, [[S$M]]", "25~[[y]], [[S$M/y]]", "60~[[y]], [[S$M/y]]"]], rows=rows)

    rows = []
    for stage, nm in (("BOAK", "BOAK"), ("FOAK", "FOAK")):
        for n, ln in (("N1P00", "1.0"), ("N0P60", "0.6"), ("N0P31", "0.31")):
            k25, k60 = f"{stage}_25Y_{n}", f"{stage}_60Y_{n}"
            rows.append([f"{nm}, $n = {ln}$", "{{AC_L3_%s}} / {{AC_L3_%s}}" % (k25, k60), "{{AC_L5_%s}} / {{AC_L5_%s}}" % (k25, k60),
                         "{{LAC_DCF_%s}} / {{LAC_DCF_%s}}" % (k25, k60), "{{LCOH_NUSH2_%s}} / {{LCOH_NUSH2_%s}}" % (k25, k60)])
    rows.append(["Sized to need, BOAK, $n = 1.0$", "{{AC_L3_BOAK_25Y_N1P00_NEED}} / {{AC_L3_BOAK_60Y_N1P00_NEED}}",
                 "{{AC_L5_BOAK_25Y_N1P00_NEED}} / {{AC_L5_BOAK_60Y_N1P00_NEED}}", "n/e", "n/e"])
    rows.append(["Case A at its own need, BOAK", "{{AC_L3_A_REC_F_BOAK_25Y}} / {{AC_L3_A_REC_F_BOAK_60Y}}",
                 "{{AC_L5_A_REC_F_BOAK_25Y}} / {{AC_L5_A_REC_F_BOAK_60Y}}", "n/e", "n/e"])
    specs["t_d_matrix"] = dict(
        caption="Stage, life and size matrix at 25~[[y]] / 60~[[y]]. Costs are [[S$/tCO2e]], LCOH is [[S$/kg]]. Cells marked n/e are not exported.",
        widths=[53, 27, 27, 27, 25], align="lcccc", keep=True,
        header=[["Case", "L3", "L5", "DCF LAC", "LCOH"]], rows=rows)

    specs["t_d_fx"] = dict(
        caption="Currency and escalation.",
        widths=[110, 35], align="lr", keep=True, center=True,
        header=[["Factor", "Value"]],
        rows=[["Singapore dollars per US dollar, 2025 average [@fredfx]", t("FX", "f4")],
              ["2025 Singapore dollars per 2019 US dollar [@fredgdp]", t("F_2019USD_TO_2025SGD", "f4")],
              ["2025 Singapore dollars per 2014 euro [@fredeur]", t("F_2014EUR_TO_2025SGD", "f3")]])

    specs["t_comp"] = dict(
        caption="SMR plus CCS comparator, indicative only, since gas price and scale differ.",
        widths=[79, 40, 40], align="lrr", keep=True,
        header=[["Item", "IEAGHG Case 3", "NUS-H₂, BOAK, 25~[[y]]"]],
        rows=[["Capture route", "MEA on flue gas", "MEA on syngas"],
              ["Overall capture, %", t("COMPARATOR_CAPTURE_PCT", "f1"), t("PLANT_CAPTURE_B_PCT", "f1")],
              ["Abatement, [[MtCO2e/y]]", t("COMPARATOR_A_MT"), t("A_ATM_B_MT")],
              ["Baseline, [[kg/kg]]", t("BASELINE_IEAGHG_KG_KG", "f2"), t("BASELINE_INL_KG_KG", "f2")],
              ["Cost at low T&S, [[S$/tCO2e]]", t("COMPARATOR_S_T", "f1"), t("SENS_TS_23P2_AC_L5_25", "f1")],
              ["Cost at {{TS_CENTRAL_S_T|f0|u}}, [[S$/tCO2e]]", t("COMPARATOR_TS60_S_T", "f1"), t("AC_L5_BOAK_25Y", "f1")]])

    # Appendix E, built from the validation rows.
    groups = OrderedDict([("HEADLINE_", "Headline values against screening"), ("SIZING_WALK_", "Sizing walk"),
                          ("SCREENING_", "Screening reproduction checks"), ("GOLD_", "Golden grids of ledger and balance"),
                          ("LAC_DCF_VS_L5_", "DCF against the L5 ledger"), ("", "Other engine checks")])
    val = list(csv.DictReader(open(ROOT / "results" / "Validation_Results.csv", encoding="utf-8")))
    seen = {}
    for r in val:
        for pre in groups:
            if r["test_id"].startswith(pre):
                seen.setdefault(pre, []).append(r)
                break
    rows = []
    for pre, label in groups.items():
        grp = seen.get(pre, [])
        if not grp:
            continue
        safe = (pre or "OTHER").strip("_")
        rows.append([label, "{{VALGROUP_%s_N}}" % safe, "{{VALGROUP_%s_PASS}}" % safe, "{{VALGROUP_%s_MAXREL}}" % safe, "{{VALGROUP_%s_USE}}" % safe])
    specs["t_e_val"] = dict(
        caption="Recorded checks, generated from the validation results. Maximum error is relative, in percent.",
        widths=[74, 15, 15, 20, 35], align="lrrrl", keep=True,
        header=[["Group", "Rows", "Pass", "Max error", "Data use"]], rows=rows)

    return specs
