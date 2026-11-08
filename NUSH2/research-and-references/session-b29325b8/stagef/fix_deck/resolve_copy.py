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

# Known causes of drift over tolerance, from the engine design record. Logged only, never printed.
CAUSES = {"MARGIN_B_MWTH": "Headline runs term T05 on its gated model (syngas above 130 C), which lowers the reactor need "
                           "from the anchored 179.04 to 178.49 MWth (docs/FINAL_DESIGN_DECISIONS.md, drift section)"}


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
    add("NEVER_GENERATED_MT", v("RES_CARBON_NEVER_GENERATED_T_Y") * 1e-6, "Mt/y", 3)
    add("CARBON_REACHING_ATM_OR_STORE_T_Y", capt + atm, "t/y", 0)
    add("GAS_SAVING_INL_PCT", 100 * v("RES_GAS_SAVING_INL"), "%", 1)
    add("GAS_SAVING_AHN_PCT", 100 * v("RES_GAS_SAVING_AHN"), "%", 1)
    for name, tok in (("A_NORECYCLE_MT", "SENS_TAIL_GAS_2_A_T_Y"), ("A_NOHEAT_N_MT", "SENS_TAIL_GAS_3_A_T_Y"),
                      ("A_AV65_MT", "SENS_AVAILABILITY_0P65_A_T_Y"), ("A_AV75_MT", "SENS_AVAILABILITY_0P75_A_T_Y"),
                      ("A_AV90_MT", "SENS_AVAILABILITY_0P9_A_T_Y"), ("A_CAP80_MT", "SENS_CAPTURE_0P8_A_T_Y"),
                      ("A_CAP85_MT", "SENS_CAPTURE_0P85_A_T_Y"), ("A_CAP95_MT", "SENS_CAPTURE_0P95_A_T_Y"),
                      ("A_SELEXOL_MT", "SENS_SOLVENT_1_A_T_Y"), ("A_AMDEA_MT", "SENS_SOLVENT_2_A_T_Y"),
                      ("A_REF825_MT", "SENS_REFORMER_T_825_A_T_Y"), ("A_REF800_MT", "SENS_REFORMER_T_800_A_T_Y"),
                      ("A_CONDENSATE_MT", "SENS_CONDENSATE_CO2_1_A_T_Y"), ("A_BACKUP_CCS_MT", "SENS_BACKUP_FIRING_2_A_T_Y")):
        add(name, v(tok) * 1e-6, "Mt/y", 3)

    # Reactor need against the rating. The heat and power rows themselves are engine rows
    # (Integrated_Energy_Balance.csv), printed directly, so no residual is computed here.
    add("ADVERSE_SHORT_MWTH", v("R_HIGH_B_MWTH") - v("R_RATED_MWTH"), "MWth", 1, 2)
    add("CASE_A_SHORT_MWTH", -v("MARGIN_A_MEA_MWTH"), "MWth", 1, 2)
    add("E_CIRC_TOTAL_AINL_MWE", v("RES_E_CIRC_PRIM_MWE_AINL") + v("RES_E_CIRC_SEC_MWE_AINL"), "MWe", 1, 2)
    add("E_CIRC_TOTAL_AMEA_MWE", v("RES_E_CIRC_PRIM_MWE_AMEA") + v("RES_E_CIRC_SEC_MWE_AMEA"), "MWe", 1, 2)
    scale = v("H2_B_KG_H") / v("H2_A_KG_H")
    add("SCALE_S", scale, "x", 4)
    add("T05_NEED_SHIFT_MWTH", v("EB_B_REC_F_ANCHORED_HTGR_HEAT") - v("R_NEEDED_B_MWTH"), "MWth", 2, 2)

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

    # Comparator, IEAGHG Case 3: the engine rows, in Mt/y and percent for prose.
    add("COMPARATOR_A_MT", v("RES_COMPARATOR_ABATEMENT_T_Y") * 1e-6, "Mt/y", 3)
    add("COMPARATOR_CAPTURE_PCT", 100 * v("COMPARATOR_OVERALL_CAPTURE"), "%", 1)
    add("COMPARATOR_TS_EUR", v("TS_LOW") / eur, "EUR/t", 0)

    # Operator view and backup firing.
    add("TAX_NET_BOAK_25Y", v("AC_L5_BOAK_25Y") - CARBON_TAX_S_T, "S$/t", 1)
    add("TAX_NET_FOAK_25Y", v("AC_L5_FOAK_25Y") - CARBON_TAX_S_T, "S$/t", 1)
    add("OUTAGE_H_Y", HOURS_YEAR - v("HOURS_Y"), "h/y", 0)

    # Percent forms of inputs, for prose.
    add("ETA_POWER_PCT", 100 * v("ETA_POWER"), "%", 1)
    add("LEARNING_PCT", 100 * v("LEARNING"), "%", 0)
    add("CAPTURE_FOM_PCT", 100 * v("CAPTURE_FOM"), "%", 1)
    add("CAPTURE_FOM_HIGH_PCT", 100 * v("CAPTURE_FOM_HIGH"), "%", 1)
    add("DISCOUNT_PCT", 100 * v("DISCOUNT"), "%", 0)
    add("CARBON_TAX", CARBON_TAX_S_T, "S$/t", 0)
    add("CARBON_TAX_2030_LOW", 50.0, "S$/t", 0)
    add("CARBON_TAX_2030_HIGH", 80.0, "S$/t", 0)
    add("AVAILABILITY_LOW_PCT", 100 * min(x["test_value"] for x in data["sweeps"]["AVAILABILITY"]["tests"]), "%", 0)
    add("AVAILABILITY_HIGH_PCT", 100 * max(x["test_value"] for x in data["sweeps"]["AVAILABILITY"]["tests"]), "%", 0)

    # Validation counts for Appendix E, by level, read from the results file itself. Counting only.
    val = list(csv.DictReader(open(ROOT / "results" / "Validation_Results.csv", encoding="utf-8")))
    for lvl in sorted({r["level"] for r in val}):
        rows = [r for r in val if r["level"] == lvl]
        add(f"VALLEVEL_{lvl}_N", float(len(rows)), "rows", 0)
        for st in ("pass", "fail", "reported", "not_run"):
            add(f"VALLEVEL_{lvl}_{st.upper()}", float(sum(r["status"] == st for r in rows)), "rows", 0)
        add(f"VALLEVEL_{lvl}_INDEPENDENT", float(sum(r["data_use"] == "independent" for r in rows)), "rows", 0)
        add(f"VALLEVEL_{lvl}_INDEP_PASS", float(sum(r["data_use"] == "independent" and r["status"] == "pass" for r in rows)), "rows", 0)
    add("VAL_N_INDEP_PASS", float(sum(r["data_use"] == "independent" and r["status"] == "pass" for r in val)), "rows", 0)
    # Percent forms of the column benchmark errors that decide the M6 gate.
    for name in ("DICE-SRD", "KW-SRD", "SA-1", "FS_CARBON_VS_ANCHOR_TUBE", "FS_CARBON_VS_ANCHOR_TAIL_GAS"):
        add(f"VALPCT_{name.replace('-', '_')}", 100 * abs(v(f"VAL_{name}_REL")), "%", 0 if "-" in name else 1)

    with DRIFT.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["token", "engine", "screening", "rel_diff", "tol", "state", "cause"])
        over = 0
        for name, screen, tol in SCREENING:
            rel = abs(v(name) - screen) / abs(screen)
            state = "ok" if rel <= tol else "over"
            over += state == "over"
            w.writerow([name, v(name), screen, f"{rel:.5f}", tol, state, CAUSES.get(name, "") if state == "over" else ""])
    OUT.write_text(json.dumps({"sources": data["sources"], "status_rows": data["status_rows"], "checks": data["checks"],
                               "units_tex": data["units_tex"], "sweeps": data["sweeps"], "tokens": t},
                              ensure_ascii=False, indent=0), encoding="utf-8")
    print(f"resolved {len(t)} tokens, drift rows over tolerance: {over}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
