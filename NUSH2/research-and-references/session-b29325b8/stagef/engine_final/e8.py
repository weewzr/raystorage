import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "scripts/build_numbers.py"

rep(F, """    SENS_GAS_10_AC_L5_25                      sweep, test value, metric; _DELTA is against central
""", """    SENS_GAS_10_AC_L5_25                      sweep, test value, metric; _DELTA is against central
    MATURE_60Y_L5_AT_PRINTED, ..._GAS_PRINTED  the MATURE_60Y solves, points and printed gas price
    STEPDOWN_1_L5 to STEPDOWN_4_DCF           the step-down bars, with _DELTA, _GAS_S_GJ, _LABEL
    COFUND_L5_25Y_SHARE_PCT, COFUND_L5_25Y_SM co-funding at 25 y and 60 y, both methods
""")

rep(F, """    units = {"A_T_Y": "t/y", "R_NEEDED": "MWth", "MARGIN_210": "MWth", "NG_MMSCFD": "MMSCFD"}""",
    """    units = {"A_T_Y": "t/y", "R_NEEDED": "MWth", "MARGIN_210": "MWth", "NG_MMSCFD": "MMSCFD",
             "GAS_NEEDED": "S$/GJ", "GAS_PRINTED": "S$/GJ", "COFUND_SHARE": "-",
             "COFUND_S_M": "S$M", "REACTOR_CAPITAL_S_M": "S$M"}""")

rep(F, """def add_validation(tok, rows):""", '''def add_mature(tok, rows, checks):
    """MATURE_60Y, STEPDOWN and COFUNDING tokens by name, each one row of
    Sensitivity_Results.csv, so no builder does arithmetic on them. Checks that the
    step-down bars equal their source rows and that the printed gas point sits at
    or below the mature-scenario target on both methods."""
    fams = ("MATURE_60Y", "STEPDOWN", "COFUNDING")
    sens = [r for r in rows if r["sweep_id"] in fams]

    def one(sweep, run, metric, test=None):
        hit = [r for r in sens if r["sweep_id"] == sweep and r["run_id"] == run
               and r["metric"] == metric and (test is None or close(num(r["test_value"]), test))]
        if len(hit) != 1:
            raise BuildError(f"{sweep} {run} {metric} {test}: {len(hit)} rows")
        return hit[0]

    def src(r, col="result"):
        return (f"Sensitivity_Results.csv:{r['sweep_id']}:{r['run_id']}:"
                f"{tag_test(num(r['test_value']))}:{r['metric']}:{col}")

    def put(name, r, unit, col="result", scale=1.0):
        tok.add(name, num(r[col]) * scale, unit, src(r, col), r["status"])
        return tok.value(name)

    x = put("MATURE_60Y_GAS_PRINTED_S_GJ", one("MATURE_60Y", "MATURE_60Y", "GAS_PRINTED"), "S$/GJ")
    needed = []
    for m in ("L5", "DCF"):
        run = f"MATURE_60Y_{m}"
        g = one("MATURE_60Y", run, "GAS_NEEDED")
        needed.append(put(f"MATURE_60Y_{m}_GAS_NEEDED_S_GJ", g, "S$/GJ"))
        central = num(g["base_value"])
        tok.add("MATURE_60Y_GAS_CENTRAL_S_GJ", central, "S$/GJ", src(g, "base_value"), g["status"])
        put(f"MATURE_60Y_{m}_AT_CENTRAL", one("MATURE_60Y", run, "COST_S_PER_T", central), "S$/t")
        put(f"MATURE_60Y_{m}_AT_20", one("MATURE_60Y", run, "COST_S_PER_T", 20.0), "S$/t")
        put(f"MATURE_60Y_{m}_AT_PRINTED", one("MATURE_60Y", run, "COST_S_PER_T", x), "S$/t")
    if not close(x, math.ceil(max(needed) * 10.0 - 1e-9) / 10.0, 1e-9):
        raise BuildError(f"printed gas price {x} is not the larger solve {max(needed)} rounded up")
    if not close(tok.value("MATURE_60Y_GAS_CENTRAL_S_GJ"), tok.value("GAS_CENTRAL"), 1e-9):
        raise BuildError("MATURE_60Y central gas price differs from GAS_CENTRAL")
    target = tok.value("TARGET_MATURE_COST")
    for m in ("L5", "DCF"):
        if tok.value(f"MATURE_60Y_{m}_AT_PRINTED") > target:
            raise BuildError(f"MATURE_60Y {m} at the printed gas price is above {target}")

    steps = sorted({r["run_id"] for r in sens if r["sweep_id"] == "STEPDOWN"})
    if [s.split("_")[1] for s in steps] != ["1", "2", "3", "4"]:
        raise BuildError(f"STEPDOWN runs {steps}")
    for run in steps:
        n = run.split("_")[1]
        for m, metric in (("L5", "AC_L5"), ("DCF", "LAC_DCF")):
            r = one("STEPDOWN", run, metric)
            put(f"STEPDOWN_{n}_{m}", r, "S$/t")
            put(f"STEPDOWN_{n}_{m}_DELTA", r, "S$/t", "delta")
        put(f"STEPDOWN_{n}_GAS_S_GJ", r, "S$/GJ", "test_value")
        tok.add(f"STEPDOWN_{n}_LABEL", r["parameter"], "", src(r, "parameter"))
    sources = {
        "1": ("AC_L5_BOAK_25Y", "LAC_DCF_BOAK_25Y"),
        "2": ("AC_L5_10OAK_P46_25Y", "LAC_DCF_10OAK_P46_25Y"),
        "3": ("AC_L5_10OAK_P46_60Y", "LAC_DCF_10OAK_P46_60Y"),
        "4": ("MATURE_60Y_L5_AT_PRINTED", "MATURE_60Y_DCF_AT_PRINTED"),
    }
    wrong = []
    for n, (l5, dcf) in sources.items():
        for m, ref in (("L5", l5), ("DCF", dcf)):
            if not close(tok.value(f"STEPDOWN_{n}_{m}"), tok.value(ref), 1e-6):
                wrong.append(f"STEPDOWN_{n}_{m} {tok.value(f'STEPDOWN_{n}_{m}')} against {ref} {tok.value(ref)}")
    for m in ("L5", "DCF"):
        if not close(tok.value(f"STEPDOWN_3_{m}"), tok.value(f"MATURE_60Y_{m}_AT_CENTRAL"), 1e-6):
            wrong.append(f"STEPDOWN_3_{m} against MATURE_60Y_{m}_AT_CENTRAL")
    if wrong:
        raise BuildError("step-down bars differ from their source rows: " + "; ".join(wrong))
    checks.append({"name": "stepdown_bars_equal_source_rows", "state": "ok",
                   "detail": f"8 bars against the ledger grid and MATURE_60Y; printed gas {x} S$/GJ, "
                             f"L5 {tok.value('MATURE_60Y_L5_AT_PRINTED'):.2f} and DCF "
                             f"{tok.value('MATURE_60Y_DCF_AT_PRINTED'):.2f} at or below {target:g}"})

    capital = []
    for m in ("L5", "DCF"):
        for life in (25, 60):
            run = f"{m}_BOAK_{life}Y"
            s = one("COFUNDING", run, "COFUND_SHARE")
            put(f"COFUND_{m}_{life}Y_SHARE", s, "-")
            put(f"COFUND_{m}_{life}Y_SHARE_PCT", s, "%", scale=100.0)
            put(f"COFUND_{m}_{life}Y_SM", one("COFUNDING", run, "COFUND_S_M"), "S$M")
            c = one("COFUNDING", run, "REACTOR_CAPITAL_S_M")
            capital.append(num(c["result"]))
            tok.add("COFUND_REACTOR_CAPITAL_SM", num(c["result"]), "S$M", src(c), c["status"])
            tok.add("COFUND_TARGET_S_T", num(s["test_value"]), "S$/t", src(s, "test_value"), s["status"])
    if not all(close(v, capital[0], 1e-9) for v in capital):
        raise BuildError(f"co-funding runs charge different reactor capital: {capital}")
    if not close(tok.value("COFUND_TARGET_S_T"), target, 1e-9):
        raise BuildError("co-funding target differs from TARGET_MATURE_COST")
    if not close(capital[0], tok.value("REACTOR_CAPITAL_BOAK_N1P00_SM"), 1e-5):
        raise BuildError("co-funding reactor capital differs from the ledger's BOAK reactor capital")


def add_validation(tok, rows):''')

rep(F, """    add_derived(tok, tables, checks)
    statuses = run_checks(tok, tables, checks)""", """    add_derived(tok, tables, checks)
    add_mature(tok, tables["Sensitivity_Results.csv"], checks)
    statuses = run_checks(tok, tables, checks)""")
rep(F, """import csv
import hashlib
import json
import sys""", """import csv
import hashlib
import json
import math
import sys""")
print("ok")
