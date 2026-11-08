#!/usr/bin/env python3
"""Plan step 5. Read build/numbers.json and write numbers_resolved.json for the report.

Derived tokens here are arithmetic on the exported results and published constants held in
literals.csv. Nothing here adds physics or types a result. A drift log compares the engine
values with the screening values of the plan; it is never printed in the report.
"""
import csv
import json
import math
import os
import sys
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parents[1]
SRC = Path(os.environ.get("NUSH2_NUMBERS", ROOT / "build" / "numbers.json"))
OUT = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))
DRIFT = B / "drift.csv"

MMBTU_H_TO_MW = 0.293071
HOURS_YEAR = 8760.0
NDC_LOW_MT, NDC_HIGH_MT = 45.0, 50.0
CARBON_TAX_S_T = 45.0

SCREENING = [
    ("H2_B_T_Y", 66925.0, 0.005), ("Q_PROCESS_B_MWTH", 120.8, 0.005), ("NG_B_MMSCFD", 23.2, 0.005),
    ("R_RATED_MWTH", 210.0, 0.005), ("R_NEEDED_B_MWTH", 179.0, 0.005), ("MARGIN_B_MWTH", 31.0, 0.005),
    ("A_ATM_B_MT", 0.578, 0.005), ("AC_L5_BOAK_25Y", 189.8, 0.01), ("AC_L5_BOAK_60Y", 165.6, 0.01),
    ("AC_L3_BOAK_25Y", 137.7, 0.01), ("AC_L3_BOAK_60Y", 117.2, 0.01),
]


def grouped(v, dp):
    r = round(v, dp)
    return f"{(0.0 if r == 0 else r):,.{dp}f}"


def main():
    data = json.loads(SRC.read_text(encoding="utf-8"))
    t = data["tokens"]

    def v(name):
        val = t[name]["value"]
        if val is None:
            raise SystemExit(f"token {name} has no value")
        return val

    def add(name, value, unit, dp, tdp=None, source="derived in report/_build/resolve_numbers.py"):
        t[name] = {"value": value, "text": grouped(value, dp), "table": grouped(value, dp if tdp is None else tdp),
                   "unit": unit, "source": source, "derived": True}

    A, TGT = v("A_ATM_B_T"), v("TARGET_ABATEMENT")
    add("ABATEMENT_MULTIPLE", A / TGT, "x", 1)
    add("NDC_SHARE_LOW_PCT", 100 * A * 1e-6 / NDC_HIGH_MT, "%", 1)
    add("NDC_SHARE_HIGH_PCT", 100 * A * 1e-6 / NDC_LOW_MT, "%", 1)

    # Carbon bridge, Table of net abatement.
    base, capt, atm = v("BASELINE_B_T_Y"), v("CAPTURED_B_T_Y"), v("ATM_B_T_Y")
    t_per_kmol = capt / v("RES_CAPTURED_KMOL_H")
    product = v("RES_PRODUCT_C_KMOL_H") * t_per_kmol
    feed = capt + atm + product
    add("PRODUCT_C_T_Y", product, "t/y", 0)
    add("FEED_C_T_Y", feed, "t/y", 0)
    add("GAS_SAVED_C_T_Y", base - feed, "t/y", 0)
    add("NEVER_GENERATED_T_Y", base - capt - atm, "t/y", 0)
    add("NEVER_GENERATED_MT", (base - capt - atm) * 1e-6, "Mt/y", 3)
    add("CARBON_REACHING_ATM_OR_STORE_T_Y", capt + atm, "t/y", 0)
    add("GAS_SAVING_INL_PCT", 100 * (1 - v("NG_B_MMSCFD") / v("NG_UNABATED_B_MMSCFD")), "%", 1)
    ahn = 1 - v("NG_AHN_EQUIVALENT_NG_KG_H") / v("NG_AHN_EQUIVALENT_BASE_KG_H")
    add("GAS_SAVING_AHN_PCT", 100 * ahn, "%", 1)
    for name, tok in (("A_NORECYCLE_MT", "SENS_TAIL_GAS_2_A_T_Y"), ("A_NOHEAT_N_MT", "SENS_TAIL_GAS_3_A_T_Y"),
                      ("A_AV65_MT", "SENS_AVAILABILITY_0P65_A_T_Y"), ("A_AV75_MT", "SENS_AVAILABILITY_0P75_A_T_Y"),
                      ("A_AV90_MT", "SENS_AVAILABILITY_0P9_A_T_Y"), ("A_CAP80_MT", "SENS_CAPTURE_0P8_A_T_Y"),
                      ("A_CAP85_MT", "SENS_CAPTURE_0P85_A_T_Y"), ("A_CAP95_MT", "SENS_CAPTURE_0P95_A_T_Y"),
                      ("A_SELEXOL_MT", "SENS_SOLVENT_1_A_T_Y"), ("A_AMDEA_MT", "SENS_SOLVENT_2_A_T_Y")):
        add(name, v(tok) * 1e-6, "Mt/y", 3)

    # Heat balance rows. The net term closes the printed rows on the reactor need.
    need, q_proc = v("R_NEEDED_B_MWTH"), v("Q_PROCESS_B_MWTH")
    slip, ihx = v("RES_SLIP_HEAT_MW"), v("EB_B_REC_F_IHX_LOSS")
    branch, gap = v("POWER_BRANCH_B_MWTH"), v("REBOILER_GAP_B_MW")
    add("HEAT_TO_POWER_MWTH", branch - gap, "MWth", 1, 2)
    add("NET_DUCT_VESSEL_SHAFT_MWTH", need - (q_proc + slip + ihx + branch), "MWth", 1, 2)
    add("ADVERSE_SHORT_MWTH", v("R_HIGH_B_MWTH") - v("R_RATED_MWTH"), "MWth", 1, 2)
    add("CASE_A_SHORT_MWTH", -v("MARGIN_A_MEA_MWTH"), "MWth", 1, 2)
    add("E_CIRC_TOTAL_B_MWE", v("E_CIRC_PRIM_B_MWE") + v("E_CIRC_SEC_B_MWE"), "MWe", 1, 2)
    add("E_CIRC_TOTAL_AINL_MWE", v("RES_E_CIRC_PRIM_MWE_AINL") + v("RES_E_CIRC_SEC_MWE_AINL"), "MWe", 1, 2)
    add("E_CIRC_TOTAL_AMEA_MWE", v("RES_E_CIRC_PRIM_MWE_AMEA") + v("RES_E_CIRC_SEC_MWE_AMEA"), "MWe", 1, 2)
    scale = v("H2_B_KG_H") / v("H2_A_KG_H")
    add("SCALE_S", scale, "x", 4)
    export = v("EXPORT_STEAM_C6") * MMBTU_H_TO_MW * scale
    add("EXPORT_STEAM_B_MW", export, "MW", 1, 2)
    add("SELEXOL_FREED_B_MW", v("EB_B_REC_F_STEAM_AVAILABLE") - v("EB_B_REC_F_SYNGAS_RECOVERY") - export, "MW", 1, 2)
    # Sum of the four engine rows, so no residual is computed here.
    add("E_OTHER_B_MWE", sum(v(f"EB_B_REC_F_{k}") for k in ("REFORMING_POWER", "PURIFICATION_POWER", "STEAM_SYSTEM_POWER", "WATER_POWER")), "MWe", 1, 2)

    # Reactor unit costs. Learning to the tenth unit on both published routes.
    occ = v("OCC_CENTRAL")
    add("OCC_FOAK_USD_KWT", occ * v("FOAK_CENTRAL"), "US$2019/kWt", 0)
    learn = (1 - v("LEARNING")) ** math.log2(v("LEARNING_UNITS"))
    add("OCC_10OAK_P46_USD_KWT", occ * learn, "US$2019/kWt", 0)
    add("OCC_10OAK_P47_USD_KWT", occ * v("FOAK_CENTRAL") * learn, "US$2019/kWt", 0)

    # Capture capital per t/h of CO2 on the two IEAGHG cases.
    eur = v("F_2014EUR_TO_2025SGD")
    add("CAPTURE_K_1A_SM_TH", v("CAPTURE_K_1A_DTCR_EURM") * eur / v("CAPTURE_K_1A_T_H"), "S$M per t/h", 2)
    add("CAPTURE_K_3_SM_TH", v("CAPTURE_K_3_DTCR_EURM") * eur / v("CAPTURE_K_3_T_H"), "S$M per t/h", 2)

    # Ledger lines combined at BOAK. Each line is additive, so these are sums of ledger terms.
    for life in (25, 60):
        key = f"BOAK_{life}Y_N1P00"
        cost, ac = v(f"COST_L5_{key}"), 1e6 / A
        cap = v(f"LINE_REACTOR_CAP_{key}")
        idc = v(f"LINE_IDC_{key}")
        lo = v(f"LINE_REACTOR_CAP_{key}_LOW") + v(f"LINE_IDC_{key}_LOW")
        hi = v(f"LINE_REACTOR_CAP_{key}_HIGH") + v(f"LINE_IDC_{key}_HIGH")
        add(f"AC_L5_OCC_LOW_{life}Y", (cost + lo - cap - idc) * ac, "S$/t", 1, 2)
        add(f"AC_L5_OCC_HIGH_{life}Y", (cost + hi - cap - idc) * ac, "S$/t", 1, 2)
        gas = abs(v(f"LINE_GAS_{key}"))
        add(f"AC_L5_GAS_AHN_{life}Y", (cost + gas * (1 - ahn / (1 - v("NG_B_MMSCFD") / v("NG_UNABATED_B_MMSCFD")))) * ac, "S$/t", 1, 2)
        k3 = v("CAPTURE_K_3_SM_TH") / v("CAPTURE_K_1A_SM_TH")
        add(f"AC_L5_CAPTURE_MEA_{life}Y", (cost + (k3 - 1) * (v(f"LINE_CAPTURE_CAP_{key}") + v(f"LINE_CAPTURE_FOM_{key}"))) * ac, "S$/t", 1, 2)

    # Stacked route at 60 y. Gas and T&S lines do not depend on stage or size, so deltas add.
    add("ROUTE_0", v("AC_L5_BOAK_60Y"), "S$/t", 1, 2)
    add("ROUTE_1", v("AC_L5_10OAK_P46_60Y_N1P00"), "S$/t", 1, 2)
    add("ROUTE_2", v("AC_L5_10OAK_P46_60Y_N1P00_NEED"), "S$/t", 1, 2)
    add("ROUTE_3_GAS", v("AC_L5_10OAK_P46_60Y_N1P00_NEED") + v("SENS_GAS_20_AC_L5_60_DELTA"), "S$/t", 1, 2)
    add("ROUTE_3_TS", v("AC_L5_10OAK_P46_60Y_N1P00_NEED") + v("SENS_TS_23P2_AC_L5_60_DELTA"), "S$/t", 1, 2)
    add("ROUTE_3_BOTH", v("ROUTE_3_GAS") + v("SENS_TS_23P2_AC_L5_60_DELTA"), "S$/t", 1, 2)
    add("ROUTE_FOAK_STACK", v("AC_L5_FOAK_60Y_N1P00_NEED") + v("SENS_GAS_20_AC_L5_60_DELTA") + v("SENS_TS_23P2_AC_L5_60_DELTA"), "S$/t", 1, 2)
    add("GAS_LEVER_MULTIPLE", v("LEVER_GAS_L5_BOAK_25Y_S_GJ") / v("GAS_CENTRAL"), "x", 1)

    # Comparator, IEAGHG Case 3 at its own rate and this report's availability.
    cac = v("COMPARATOR_CAC_EUR") * eur
    avoided = v("BASELINE_IEAGHG_KG_NM3") - v("COMPARATOR_EMIT_CASE3_KG_NM3")
    add("COMPARATOR_S_T", cac, "S$/t", 1)
    add("COMPARATOR_TS60_S_T", cac + (v("TS_CENTRAL") - v("TS_LOW")) * v("COMPARATOR_CAPTURED_CASE3_KG_NM3") / avoided, "S$/t", 1)
    add("COMPARATOR_A_MT", v("H2_B") * v("HOURS_Y") * avoided * 1e-3 * 1e-6, "Mt/y", 3)
    add("COMPARATOR_T_H", v("CAPTURE_K_3_T_H"), "t/h", 1)
    add("COMPARATOR_CAPTURE_PCT", 100 * v("COMPARATOR_OVERALL_CAPTURE"), "%", 1)

    # Operator view and backup firing.
    add("TAX_NET_BOAK_25Y", v("AC_L5_BOAK_25Y") - CARBON_TAX_S_T, "S$/t", 1)
    add("TAX_NET_FOAK_25Y", v("AC_L5_FOAK_25Y") - CARBON_TAX_S_T, "S$/t", 1)
    outage = HOURS_YEAR - v("HOURS_Y")
    add("OUTAGE_H_Y", outage, "h/y", 0)
    add("BACKUP_H2_T_Y", v("H2_B_KG_H") * outage * 1e-3, "t/y", 0)
    add("BACKUP_CO2_T_Y", v("H2_B_KG_H") * outage * 1e-3 * v("BASELINE_INL_KG_KG"), "t/y", 0)

    # Percent forms of inputs, for prose.
    add("ETA_POWER_PCT", 100 * v("ETA_POWER"), "%", 1)
    add("LEARNING_PCT", 100 * v("LEARNING"), "%", 0)
    add("CAPTURE_FOM_PCT", 100 * v("CAPTURE_FOM"), "%", 1)
    add("CAPTURE_FOM_HIGH_PCT", 100 * v("CAPTURE_FOM_HIGH"), "%", 1)
    add("DISCOUNT_PCT", 100 * v("DISCOUNT"), "%", 0)
    add("CARBON_TAX", CARBON_TAX_S_T, "S$/t", 0)
    add("CARBON_TAX_2030_LOW", 50.0, "S$/t", 0)
    add("CARBON_TAX_2030_HIGH", 80.0, "S$/t", 0)
    add("COMPARATOR_TS_EUR", v("TS_LOW") / eur, "EUR/t", 0)
    add("AVAILABILITY_LOW_PCT", 100 * min(x["test_value"] for x in data["sweeps"]["AVAILABILITY"]["tests"]), "%", 0)
    add("AVAILABILITY_HIGH_PCT", 100 * max(x["test_value"] for x in data["sweeps"]["AVAILABILITY"]["tests"]), "%", 0)

    # Validation groups for Appendix E, from the VAL_ tokens only.
    prefixes = ("HEADLINE_", "SIZING_WALK_", "SCREENING_", "GOLD_", "LAC_DCF_VS_L5_")
    tests = sorted(k[len("VAL_"):-len("_STATUS")] for k in t if k.startswith("VAL_") and k.endswith("_STATUS"))
    use_words = {"same_dataset_disclosed": "same dataset", "independent": "independent", "calibration": "calibration"}
    for pre in prefixes + ("",):
        safe = (pre or "OTHER").strip("_")
        group = [k for k in tests if (k.startswith(pre) if pre else not k.startswith(prefixes))]
        if not group:
            continue
        rels = [abs(t[f"VAL_{k}_REL"]["value"]) for k in group if f"VAL_{k}_REL" in t and t[f"VAL_{k}_REL"]["value"] is not None]
        add(f"VALGROUP_{safe}_N", float(len(group)), "rows", 0)
        add(f"VALGROUP_{safe}_PASS", float(sum(t[f"VAL_{k}_STATUS"]["value"] == "pass" for k in group)), "rows", 0)
        add(f"VALGROUP_{safe}_MAXREL", 100 * max(rels) if rels else 0.0, "%", 4)
        uses = sorted({use_words.get(t[f"VAL_{k}_DATA_USE"]["value"], t[f"VAL_{k}_DATA_USE"]["value"]) for k in group})
        t[f"VALGROUP_{safe}_USE"] = {"value": ", ".join(uses), "text": ", ".join(uses), "table": ", ".join(uses), "unit": "", "source": "derived", "derived": True}

    # Model status words.
    add("VAL_MAX_REL_PCT", 100 * max(abs(x["value"]) for k, x in t.items() if k.startswith("VAL_") and k.endswith("_REL") and x["value"] is not None), "%", 2)

    with DRIFT.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["token", "engine", "screening", "rel_diff", "tol", "state"])
        over = 0
        for name, screen, tol in SCREENING:
            rel = abs(v(name) - screen) / abs(screen)
            state = "ok" if rel <= tol else "over"
            over += state == "over"
            w.writerow([name, v(name), screen, f"{rel:.5f}", tol, state])
    OUT.write_text(json.dumps({"sources": data["sources"], "status_rows": data["status_rows"], "checks": data["checks"],
                               "units_tex": data["units_tex"], "sweeps": data["sweeps"], "tokens": t},
                              ensure_ascii=False, indent=0), encoding="utf-8")
    print(f"resolved {len(t)} tokens, drift rows over tolerance: {over}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
