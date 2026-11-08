#!/usr/bin/env python3
"""Sentences in the prose that assert a direction or an order. Each is tested against the
current numbers, so a rebuild on new results flags any sentence the numbers no longer support."""


def checks(v, tok):
    """v(name) gives a token value. Returns (claim, passed) pairs."""
    target = v("TARGET_COST")
    central = [v(f"{m}_{s}_{y}Y") for m in ("AC_L5", "LAC_DCF") for s in ("BOAK", "FOAK") for y in (25, 60)]
    gas_cum = v("CUM_GAS_BOAK_25Y_N1P00") + v("CUM_REACTOR_CAP_BOAK_25Y_N1P00") - v("CUM_GAS_BOAK_25Y_N1P00")
    return [
        ("Cost target missed in every central case", v("TARGET_COST_N_PASS") == 0 and max(central) >= min(central) > target),
        ("Abatement target met", v("A_ATM_B_T") > v("TARGET_ABATEMENT")),
        ("DCF within 10% of L5", v("DCF_TEST_MAX_GAP_PCT") <= 10),
        ("L3 below full cost, so L3 omits real costs", v("AC_L3_BOAK_25Y") < v("AC_L5_BOAK_25Y")),
        ("Gas credit offsets most of the reactor capital line", abs(v("LINE_GAS_BOAK_25Y_N1P00")) > 0.5 * v("LINE_REACTOR_CAP_BOAK_25Y_N1P00")),
        ("T&S and reactor O&M take the cost past the target", v("CUM_CAPTURE_FOM_BOAK_25Y_N1P00") < target < v("CUM_REACTOR_OM_BOAK_25Y_N1P00")),
        ("Gas lever above the tested high", v("LEVER_GAS_L5_BOAK_25Y_S_GJ") > v("GAS_HIGH")),
        ("Reactor lever below INL low", v("LEVER_OCC_L5_BOAK_25Y_USD_KWT") < v("OCC_LOW")),
        ("T&S lever negative", v("LEVER_TS_L5_BOAK_25Y_S_T") < 0),
        ("Tenth unit sized to need at 60 y still misses", v("ROUTE_2") > target),
        ("Route with gas or T&S closes", v("ROUTE_3_GAS") < target and v("ROUTE_3_TS") < target),
        ("FOAK stack still misses", v("ROUTE_FOAK_STACK") > target),
        ("Comparator between BOAK and FOAK at matched T&S", v("AC_L5_BOAK_25Y") < v("COMPARATOR_TS60_S_T") < v("AC_L5_FOAK_25Y")),
        ("Spare is positive at the rating", v("MARGIN_B_MWTH") > 0),
        ("Adverse corner above the rating", v("R_HIGH_B_MWTH") > v("R_RATED_MWTH")),
        ("Case A above the rating", v("R_NEEDED_A_MEA_MWTH") > v("R_RATED_MWTH")),
        ("Capture offline below the abatement target", v("NEVER_GENERATED_T_Y") < v("TARGET_ABATEMENT")),
        ("All working capture and recycle cases above target",
         min(v("A_CAP80_MT"), v("A_AV65_MT"), v("A_NORECYCLE_MT"), v("A_SYSTEM_B_MT")) > v("TARGET_ABATEMENT_MT")),
        ("SRD leaves the cost almost unchanged",
         abs(v("SENS_SRD_4P2_AC_L5_25") - v("SENS_SRD_3P4_AC_L5_25")) < 1.0),
        ("Size treatment and recycle move the cost more than availability",
         v("AC_L5_BOAK_25Y_N0P60") - v("AC_L5_BOAK_25Y") > v("SENS_AVAILABILITY_0P65_AC_L5_25") - v("AC_L5_BOAK_25Y")
         and v("SENS_TAIL_GAS_2_AC_L5_25") - v("AC_L5_BOAK_25Y") > v("SENS_AVAILABILITY_0P65_AC_L5_25") - v("AC_L5_BOAK_25Y")),
        ("Availability moves the cost more than the Ahn gas saving is near",
         v("AC_L5_GAS_AHN_25Y") - v("AC_L5_BOAK_25Y") > v("SENS_TS_112_AC_L5_25") - v("AC_L5_BOAK_25Y")),
        ("Reformer temperature sensitivity not run", not any(k.startswith("SENS_REFORMER") for k in tok)),
        ("Rate-based columns not in the validation file", not any("ABSORBER" in k or "STRIPPER" in k for k in tok if k.startswith("VAL_"))),
        ("Results are anchored ports with no independent benchmark",
         v("STATUS_ROWS_ANCHORED_PORT") > 0 and v("VAL_N_INDEPENDENT") == 0),
    ]
