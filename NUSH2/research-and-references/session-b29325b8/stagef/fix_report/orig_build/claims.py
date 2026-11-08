#!/usr/bin/env python3
"""Sentences in the prose that assert a direction or an order. Each is tested against the
current numbers, so a rebuild on new results flags any sentence the numbers no longer support."""
import math


def checks(v, tok):
    """v(name) gives a token value. Returns (claim, passed) pairs."""
    target = v("TARGET_COST")
    central = [v(f"{m}_{s}_{y}Y") for m in ("AC_L5", "LAC_DCF") for s in ("BOAK", "FOAK") for y in (25, 60)]
    base = v("AC_L5_BOAK_25Y")
    eb = lambda term: v(f"EB_B_REC_F_{term}")

    # One-at-a-time rows of the uncertainty table, as distances from the base.
    unc = {name: abs(v(key) - base) for name, key in (
        ("ts_low", "SENS_TS_23P2_AC_L5_25"), ("ts_high", "SENS_TS_112_AC_L5_25"), ("gas_low", "SENS_GAS_10_AC_L5_25"),
        ("gas_high", "SENS_GAS_20_AC_L5_25"), ("occ_low", "SENS_REACTOR_CAPITAL_1456_AC_L5_25"),
        ("occ_high", "SENS_REACTOR_CAPITAL_2548_AC_L5_25"), ("av65", "SENS_AVAILABILITY_0P65_AC_L5_25"),
        ("av90", "SENS_AVAILABILITY_0P9_AC_L5_25"), ("ahn", "SENS_GAS_SAVING_AHN_0P188038_B_REC_F_CARBON_HELD_AC_L5_25"),
        ("cap_mea", "SENS_CAPTURE_CAPITAL_5P09395_AC_L5_25"), ("disc5", "SENS_DISCOUNT_0P05_AC_L5_25"),
        ("disc10", "SENS_DISCOUNT_0P1_AC_L5_25"), ("srd_low", "SENS_SRD_3P4_AC_L5_25"), ("srd_high", "SENS_SRD_4P2_AC_L5_25"),
        ("selexol", "SENS_SOLVENT_1_AC_L5_25"), ("amdea", "SENS_SOLVENT_2_AC_L5_25"), ("cho", "SENS_BASELINE_9P35_AC_L5_25"),
        ("ieaghg", "SENS_BASELINE_8P996_AC_L5_25"), ("ref825", "SENS_REFORMER_T_825_AC_L5_25"),
        ("ref800", "SENS_REFORMER_T_800_AC_L5_25"), ("need", "SENS_CHARGED_178P494_AC_L5_25"),
        ("norecycle", "SENS_TAIL_GAS_2_AC_L5_25"), ("n060", "AC_L5_BOAK_25Y_N0P60"))}
    leaders = ("norecycle", "n060")
    followers = ("av65", "disc5", "ahn")
    rest = [k for k in unc if k not in leaders + followers + ("need",)]

    heat_sum = (v("Q_PROCESS_B_MWTH") + v("RES_SLIP_HEAT_MW") + eb("IHX_LOSS") + eb("DUCT_LOSS") + eb("VESSEL_LOSS")
                + eb("BRANCH_TO_REBOILER") + eb("BRANCH_EXTRACTION_WORK") + eb("BRANCH_TO_TURBINE") - eb("CIRC_SHAFT_HEAT"))
    power_sum = sum(eb(k) for k in ("PUMP_COMPRESS", "COOLING", "REFORMING_POWER", "PURIFICATION_POWER", "STEAM_SYSTEM_POWER",
                                    "WATER_POWER", "HE_CIRCULATION", "TAIL_GAS_POWER"))
    reb_sum = sum(eb(k) for k in ("REB_FROM_SELEXOL", "REB_FROM_EXPORT", "REB_FROM_SYNGAS", "REB_FROM_REACTOR"))
    steps_l5 = [v(f"STEPDOWN_{i}_L5") for i in range(1, 5)]
    steps_dcf = [v(f"STEPDOWN_{i}_DCF") for i in range(1, 5)]
    solve = max(v("MATURE_60Y_L5_GAS_NEEDED_S_GJ"), v("MATURE_60Y_DCF_GAS_NEEDED_S_GJ"))

    return [
        ("Cost target missed in every central case", v("TARGET_COST_N_PASS") == 0 and min(central) > target),
        ("Abatement target met", v("A_ATM_B_T") > v("TARGET_ABATEMENT")),
        ("DCF within 10% of L5", v("DCF_TEST_MAX_GAP_PCT") <= 10),
        ("L3 below full cost, so L3 omits real costs", v("AC_L3_BOAK_25Y") < base),
        ("Gas credit offsets most of the reactor capital line", abs(v("LINE_GAS_BOAK_25Y_N1P00")) > 0.5 * v("LINE_REACTOR_CAP_BOAK_25Y_N1P00")),
        ("T&S and reactor O&M take the cost past the target", v("CUM_CAPTURE_FOM_BOAK_25Y_N1P00") < target < v("CUM_REACTOR_OM_BOAK_25Y_N1P00")),
        ("Gas lever above the tested high", v("LEVER_GAS_L5_BOAK_25Y_S_GJ") > v("GAS_HIGH")),
        ("Reactor lever below INL low", v("LEVER_OCC_L5_BOAK_25Y_USD_KWT") < v("OCC_LOW")),
        ("T&S lever negative", v("LEVER_TS_L5_BOAK_25Y_S_T") < 0),
        ("Step-down base equals the headline", abs(steps_l5[0] - base) < 1e-6 and abs(steps_dcf[0] - v("LAC_DCF_BOAK_25Y")) < 1e-6),
        ("Each step-down step lowers the cost by both methods", all(a > b for a, b in zip(steps_l5, steps_l5[1:]))
         and all(a > b for a, b in zip(steps_dcf, steps_dcf[1:]))),
        ("Only the last step-down step is below the target", min(steps_l5[:3] + steps_dcf[:3]) > target
         and max(steps_l5[3], steps_dcf[3]) < target),
        ("MATURE_60Y at the printed price meets the solve point", max(steps_l5[3], steps_dcf[3]) <= v("TARGET_MATURE_COST") + 1e-9),
        ("Printed gas price is the larger solve rounded up to 0.1",
         abs(math.ceil(solve * 10 - 1e-9) / 10 - v("STEPDOWN_4_GAS_S_GJ")) < 1e-9 and abs(v("STEPDOWN_4_GAS_S_GJ") - v("MATURE_60Y_GAS_PRINTED_S_GJ")) < 1e-9),
        ("Printed gas price is above the central price", v("STEPDOWN_4_GAS_S_GJ") > v("STEPDOWN_1_GAS_S_GJ") == v("GAS_CENTRAL")),
        ("Dearer gas lowers the cost because NUS-H2 burns less gas",
         v("NG_B_MMSCFD") < v("NG_UNABATED_B_MMSCFD") and v("SENS_GAS_20_AC_L5_25") < base < v("SENS_GAS_10_AC_L5_25")
         and v("MATURE_60Y_L5_AT_20") < v("MATURE_60Y_L5_AT_CENTRAL")),
        ("Comparator between BOAK and FOAK at matched T&S", base < v("RES_COMPARATOR_CAC_TS60_S_T") < v("AC_L5_FOAK_25Y")),
        ("Spare is positive at the rating", v("MARGIN_B_MWTH") > 0),
        ("Spare is smaller on Ahn and Lee's heat basis", 0 < v("SENS_HEAT_BASIS_143P4_MARGIN_210") < v("MARGIN_B_MWTH")),
        ("Gated T05 lowers the need below the anchored walk", v("EB_B_REC_F_ANCHORED_HTGR_HEAT") > v("R_NEEDED_B_MWTH")
         and abs(v("VAL_SIZING_WALK_STEP_6_SIM") - v("EB_B_REC_F_ANCHORED_HTGR_HEAT")) < 0.01),
        ("Adverse corner above the rating, heat still met, power imported", v("DRIFT_HEADLINE_R_HIGH_MWTH") > v("R_RATED_MWTH")
         and v("DRIFT_HEADLINE_R_HIGH_UNMET_HEAT_MWTH") == 0 and v("DRIFT_HEADLINE_R_HIGH_UNMET_E_MWE") > 0),
        ("Case A above the rating", v("R_NEEDED_A_MEA_MWTH") > v("R_RATED_MWTH")),
        ("Heat column sums to the reactor need", abs(heat_sum - v("R_NEEDED_B_MWTH")) < 1e-3),
        ("Electricity column sums to the power generated", abs(power_sum - eb("POWER_GENERATED")) < 1e-3),
        ("Reboiler stack sums to the reboiler duty", abs(reb_sum - eb("MEA_REGEN")) < 1e-3),
        ("Extraction work and turbine heat below generated power over efficiency",
         eb("BRANCH_EXTRACTION_WORK") + eb("BRANCH_TO_TURBINE") < eb("POWER_GENERATED") / v("ETA_POWER")),
        ("Capture offline below the abatement target", v("RES_CARBON_NEVER_GENERATED_T_Y") < v("TARGET_ABATEMENT")),
        ("All working capture and recycle cases above target",
         min(v("A_CAP80_MT"), v("A_AV65_MT"), v("A_NORECYCLE_MT"), v("A_SYSTEM_B_MT"), v("A_REF800_MT"),
             v("A_CONDENSATE_MT")) > v("TARGET_ABATEMENT_MT")),
        ("Cooler reformer outlet lowers abatement and burns more carbon",
         v("SENS_REFORMER_T_800_A_T_Y") < v("SENS_REFORMER_T_825_A_T_Y") < v("SENS_REFORMER_T_871_A_T_Y")
         and v("SENS_REFORMER_T_800_FS_EMITTED_C_KMOL_H") > v("SENS_REFORMER_T_825_FS_EMITTED_C_KMOL_H") > v("SENS_REFORMER_T_871_FS_EMITTED_C_KMOL_H")),
        ("Unabated backup adds no abatement, backup with capture adds some",
         v("SENS_BACKUP_FIRING_1_A_EXTRA_T_Y") == 0 and v("SENS_BACKUP_FIRING_2_A_EXTRA_T_Y") > 0),
        ("SRD leaves the cost almost unchanged", abs(v("SENS_SRD_4P2_AC_L5_25") - v("SENS_SRD_3P4_AC_L5_25")) < 1.0),
        ("Recycle and size treatment lead, availability next, discount swing close to the gas saving",
         min(unc[k] for k in leaders) > unc["av65"] > max(unc["disc5"], unc["ahn"])
         and min(unc["disc5"], unc["ahn"]) > max(unc[k] for k in rest)
         and abs(unc["disc5"] - unc["ahn"]) < 0.1 * unc["ahn"]),
        ("Charging only the need lowers the cost", v("SENS_CHARGED_178P494_AC_L5_25") < base),
        ("Stripper overstates both SRD benchmarks, column gate failed",
         v("VAL_DICE-SRD_SIM") > v("VAL_DICE-SRD_LIT") and v("VAL_KW-SRD_SIM") > v("VAL_KW-SRD_LIT")
         and all(tok[f"VAL_GATE_{x}_STATUS"]["value"] == "fail" for x in ("T02", "T06", "T07", "T08"))),
        ("Absorber pilot removal below its benchmark", v("VAL_SA-1_SIM") < v("VAL_SA-1_LIT")),
        ("Flowsheet carbon below the anchor, flowsheet gate failed",
         v("VAL_FS_CARBON_VS_ANCHOR_TUBE_SIM") < v("VAL_FS_CARBON_VS_ANCHOR_TUBE_LIT")
         and v("VAL_FS_CARBON_VS_ANCHOR_TAIL_GAS_SIM") < v("VAL_FS_CARBON_VS_ANCHOR_TAIL_GAS_LIT")
         and all(tok[f"VAL_GATE_{x}_STATUS"]["value"] == "fail" for x in ("T04", "T09", "T13"))),
        ("Gates T01 and T05 passed, T03 and T10 not run",
         all(tok[f"VAL_GATE_{x}_STATUS"]["value"] == "pass" for x in ("T01", "T05"))
         and all(tok[f"VAL_GATE_{x}_STATUS"]["value"] == "not_run" for x in ("T03", "T10"))),
        ("LCOH premium positive", v("LCOH_PREMIUM_BOAK_25Y") > 0),
        ("Carbon, hydrogen and mass residuals print as zero",
         all(abs(v(k)) < 0.05 for k in ("MB_B_REC_F_CLOSURE_CARBON_C_KMOL_H", "MB_B_REC_F_CLOSURE_H_KMOL_H", "MB_B_REC_F_CLOSURE_MASS_KG_H"))),
    ]
