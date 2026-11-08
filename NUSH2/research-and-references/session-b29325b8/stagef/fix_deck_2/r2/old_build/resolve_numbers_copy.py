#!/usr/bin/env python3
"""Plan step 5. Read build/numbers.json and write numbers_resolved.json for the report.

Every token, its value and its printed text come from build/numbers.json unchanged: the
document forms of results (unit scalings, sums, ratios, validation counts) and the cited
published constants are tokens of scripts/build_numbers.py, so nothing here computes or
types a printed number. A drift log compares the engine values with the screening values
of the plan; it is never printed in the report.

The deck runs a copy of this file with NUSH2_NUMBERS and NUSH2_RESOLVED set, so it stays
self-contained.
"""
import csv
import json
import os
import sys
from pathlib import Path

B = Path(__file__).resolve().parent
ROOT = B.parents[1]
SRC = Path(os.environ.get("NUSH2_NUMBERS", ROOT / "build" / "numbers.json"))
OUT = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))
DRIFT = B / "drift.csv"

SCREENING = [
    ("H2_B_T_Y", 66925.0, 0.005), ("Q_PROCESS_B_MWTH", 120.8, 0.005), ("NG_B_MMSCFD", 23.2, 0.005),
    ("R_RATED_MWTH", 210.0, 0.005), ("R_NEEDED_B_MWTH", 179.0, 0.005), ("MARGIN_B_MWTH", 31.0, 0.005),
    ("A_ATM_B_MT", 0.578, 0.005), ("AC_L5_BOAK_25Y", 189.8, 0.01), ("AC_L5_BOAK_60Y", 165.6, 0.01),
    ("AC_L3_BOAK_25Y", 137.7, 0.01), ("AC_L3_BOAK_60Y", 117.2, 0.01),
]

# Known causes of drift over tolerance, from the engine design record. Logged only, never printed.
CAUSES = {"MARGIN_B_MWTH": "Headline runs term T05 on its gated model (syngas above 130 C), which lowers the reactor need "
                           "from the anchored 179.04 to 178.49 MWth (docs/FINAL_DESIGN_DECISIONS.md, drift section)"}

# Tokens the report and the deck print that once lived in this file. They now come from
# numbers.json, so a stale numbers.json stops the build here instead of in a builder.
DOCUMENT_FORMS = ["ABATEMENT_MULTIPLE", "NDC_SHARE_LOW_PCT", "NDC_SHARE_HIGH_PCT", "PRODUCT_C_T_Y", "FEED_C_T_Y",
                  "GAS_SAVED_C_T_Y", "NEVER_GENERATED_MT", "ADVERSE_SHORT_MWTH", "OCC_10OAK_P46_USD_KWT",
                  "CAPTURE_K_1A_SM_TH", "COMPARATOR_TS_EUR", "OUTAGE_H_Y", "VALLEVEL_1_N", "VALPCT_DICE_SRD",
                  "LIT_HTTR_OUTLET_C"]


def main():
    data = json.loads(SRC.read_text(encoding="utf-8"))
    t = data["tokens"]
    lost = [name for name in DOCUMENT_FORMS if name not in t]
    if lost:
        raise SystemExit(f"numbers.json lacks {lost}; rebuild it with scripts/build_numbers.py")

    def v(name):
        val = t[name]["value"]
        if val is None:
            raise SystemExit(f"token {name} has no value")
        return val

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
    print(f"resolved {len(t)} tokens from numbers.json, none added; drift rows over tolerance: {over}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
