#!/usr/bin/env python3
"""Number map: read the seven results CSVs, write build/numbers.json.

This file is the only route by which a result reaches a document. It is a pure
function of results/*.csv: no timestamp, no commit id, no TOML, so two runs on the
same CSVs give identical bytes. Builders read tokens[NAME]["text"] for prose and
tokens[NAME]["table"] for tables, and never type a result.

Usage:
    python scripts/build_numbers.py           write build/numbers.json and a grep index
    python scripts/build_numbers.py --verify  exit 1 if numbers.json no longer matches results/
    python scripts/build_numbers.py --self-test  exit 1 if a metric unit is unlisted or wrong

Token families, all in tokens{}:
    every row of Final_Design_Basis.csv       ID (Case B), ID_AINL, ID_AMEA, ID_LOW, ID_HIGH
    AC_L3_BOAK_25Y_N1P00[_NEED]               cumulative S$/t at a level; _NEED charges the need
    COST_L3_BOAK_25Y_N1P00[_NEED]             the same in S$M/y
    LINE_GAS_BOAK_25Y_N1P00[_NEED]            one ledger line, S$M/y; CUM_ is its cumulative S$/t
    AC_L5_BOAK_25Y                            central short form: n = 1.0, 210 MWth charged
    LAC_DCF_, LCOH_NUSH2_, LCOH_SMR_          DCF results, grid and short forms
    AC_L5_A_REC_F_BOAK_25Y                    other fixed runs, n = 1.0, own charged size
    EB_B_REC_F_HTGR_HEAT, MB_..., EM_...      energy, mass and emissions rows
    SENS_GAS_10_AC_L5_25                      sweep, test value, metric; _DELTA is against central
    MATURE_60Y_L5_AT_PRINTED, ..._GAS_PRINTED  the MATURE_60Y solves, points and printed gas price
    STEPDOWN_1_L5 to STEPDOWN_4_DCF           the step-down bars, with _DELTA, _GAS_S_GJ, _LABEL
    COFUND_L5_25Y_SHARE_PCT, COFUND_L5_25Y_SM co-funding at 25 y and 60 y, both methods
    VAL_N_PASS, VAL_<TEST>_SIM, ...           validation counts and rows
    LEVER_, EXCESS_, BUDGET_, NUCLEAR_ITEMS_  distance to the cost target, recomputed
    A_ATM_B_MT, Q_PROCESS_B_MWTH, ...         named aliases in ALIASES below

A row the engine did not run has value null and text "not run". Exit 1, with nothing
written, if a CSV, a column, an alias or an arithmetic identity fails. Soft findings
go to checks[] in the JSON with state "warn" and are printed.
"""
import csv
import hashlib
import json
import math
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RESULTS = ROOT / "results"
OUT_DIR = ROOT / "build"
OUT_JSON = OUT_DIR / "numbers.json"
OUT_INDEX = OUT_DIR / "numbers_tokens.csv"

CSV_COLUMNS = {
    "Final_Design_Basis.csv": {"id", "group", "parameter", "case_b", "case_a_inl", "case_a_mea",
                               "unit", "low", "high", "class", "status"},
    "Integrated_Mass_Balance.csv": {"run_id", "stream_id", "total_kg_h", "total_kmol_h",
                                    "carbon_kmol_h", "status"},
    "Integrated_Energy_Balance.csv": {"run_id", "term_id", "value_mw", "low_mw", "high_mw", "status"},
    "Emissions_Ledger.csv": {"run_id", "baseline_id", "view", "line_id", "t_per_y", "kg_per_kg_h2",
                             "per_pass_capture", "plant_capture", "status"},
    "Economic_Ledger.csv": {"run_id", "stage", "life_y", "n", "reactor_charged_mwth", "level",
                            "line_id", "value_sgdm_y", "low", "high", "cum_sgdm_y", "cum_s_per_t",
                            "abatement_t_y", "dcf_value", "dcf_unit", "status"},
    "Sensitivity_Results.csv": {"sweep_id", "run_id", "parameter", "unit", "base_value",
                                "test_value", "metric", "result", "delta", "status"},
    "Validation_Results.csv": {"test_id", "level", "literature", "simulated", "unit", "rel_error",
                               "status", "data_use"},
}

# Decimal places (text, table). Freeze 7.4 fixes t/y, Mt/y, MWth, MWe, S$/t and S$M/y.
# The rest extend that rule to units it does not name.
PLACES = {
    "t/y": (0, 0), "Mt/y": (3, 3), "MWth": (1, 2), "MWe": (1, 2), "MW": (1, 2),
    "S$/t": (1, 2), "S$M/y": (1, 2), "S$/tCO2": (1, 2), "S$/kg": (2, 2), "S$M": (1, 1),
    "%": (1, 1), "kg/kg": (2, 2), "MMSCFD": (1, 1), "kg/s": (1, 1), "kmol/h": (0, 1),
    "kg/h": (0, 0), "t/h": (1, 1), "Nm3/h": (0, 0), "h/y": (0, 0), "d/y": (2, 2),
    "S$/GJ": (1, 1), "US$2019/kWt": (0, 0), "GJ/t": (1, 1), "gpm": (0, 0), "CRF": (4, 4),
    "rows": (0, 0), "x": (2, 2),
}

# Report units as the $\mathrm{...}$ body, so that they become native Word math.
UNIT_TEX = {
    "t/y": r"\mathrm{t\ y^{-1}}", "Mt/y": r"\mathrm{Mt\ y^{-1}}", "MWth": r"\mathrm{MW_{th}}",
    "MWe": r"\mathrm{MW_{e}}", "MW": r"\mathrm{MW}", "S$/t": r"\mathrm{S\$/tCO_{2}e}",
    "S$/tCO2": r"\mathrm{S\$/tCO_{2}}", "S$M/y": r"\mathrm{S\$M\ y^{-1}}", "S$/kg": r"\mathrm{S\$\ kg^{-1}}",
    "S$M": r"\mathrm{S\$M}", "%": r"\mathrm{\%}", "kg/kg": r"\mathrm{kg\ CO_{2}\ kg^{-1}\ H_{2}}",
    "MMSCFD": r"\mathrm{MMSCFD}", "kg/s": r"\mathrm{kg\ s^{-1}}", "kmol/h": r"\mathrm{kmol\ h^{-1}}",
    "kg/h": r"\mathrm{kg\ h^{-1}}", "Nm3/h": r"\mathrm{Nm^{3}\ h^{-1}}", "h/y": r"\mathrm{h\ y^{-1}}",
    "d/y": r"\mathrm{d\ y^{-1}}", "S$/GJ": r"\mathrm{S\$\ GJ^{-1}}", "GJ/t": r"\mathrm{GJ\ t^{-1}}",
    "US$2019/kWt": r"\mathrm{US\$_{2019}\ kW_{t}^{-1}}",
}

# Named aliases: token, Final_Design_Basis id, column, unit, scale.
B, AI, AM = "case_b", "case_a_inl", "case_a_mea"
ALIASES = [
    # Production
    ("H2_B_NM3H", "H2_B", B, "Nm3/h", 1), ("H2_B_KG_H", "RES_H2_KG_H", B, "kg/h", 1),
    ("H2_A_KG_H", "RES_H2_KG_H", AI, "kg/h", 1), ("H2_B_T_Y", "RES_H2_T_Y", B, "t/y", 1),
    ("H2_A_T_Y", "RES_H2_T_Y", AI, "t/y", 1), ("AVAILABILITY_PCT", "AVAILABILITY", B, "%", 100),
    ("HOURS_Y", "RES_HOURS", B, "h/y", 1), ("DAYS_Y", "RES_DAYS", B, "d/y", 1),
    ("PSA_RECOVERY_PCT", "PSA_H2_RECOVERY", B, "%", 100),
    ("NG_B_MMSCFD", "RES_NG_MMSCFD", B, "MMSCFD", 1), ("NG_A_MMSCFD", "RES_NG_MMSCFD", AI, "MMSCFD", 1),
    ("NG_UNABATED_B_MMSCFD", "RES_NG_UNABATED_MMSCFD", B, "MMSCFD", 1),
    ("NG_UNABATED_A_MMSCFD", "RES_NG_UNABATED_MMSCFD", AI, "MMSCFD", 1),
    ("NG_NORECYCLE_B_MMSCFD", "RES_NG_NORECYCLE_MMSCFD", B, "MMSCFD", 1),
    # Carbon, 85% availability
    ("BASELINE_B_T_Y", "RES_BASELINE_T_Y", B, "t/y", 1), ("BASELINE_A_T_Y", "RES_BASELINE_T_Y", AI, "t/y", 1),
    ("CAPTURED_B_T_Y", "RES_CAPTURED_T_Y", B, "t/y", 1), ("CAPTURED_B_MT", "RES_CAPTURED_T_Y", B, "Mt/y", 1e-6),
    ("ATM_B_T_Y", "RES_ATM_T_Y", B, "t/y", 1), ("ATM_A_MEA_T_Y", "RES_ATM_T_Y", AM, "t/y", 1),
    ("A_ATM_B_T", "RES_ABATEMENT_T_Y", B, "t/y", 1), ("A_ATM_B_MT", "RES_ABATEMENT_T_Y", B, "Mt/y", 1e-6),
    ("A_ATM_A_INL_T", "RES_ABATEMENT_T_Y", AI, "t/y", 1), ("A_ATM_A_MEA_T", "RES_ABATEMENT_T_Y", AM, "t/y", 1),
    ("A_CHO_B_T", "RES_ABATEMENT_CHO_T_Y", B, "t/y", 1), ("A_CHO_B_MT", "RES_ABATEMENT_CHO_T_Y", B, "Mt/y", 1e-6),
    ("A_IEAGHG_B_T", "RES_ABATEMENT_IEAGHG_T_Y", B, "t/y", 1),
    ("A_IEAGHG_B_MT", "RES_ABATEMENT_IEAGHG_T_Y", B, "Mt/y", 1e-6),
    ("PER_PASS_CAPTURE_PCT", "ETA_MEA", B, "%", 100), ("PLANT_CAPTURE_B_PCT", "RES_PLANT_CAPTURE", B, "%", 100),
    ("PLANT_CAPTURE_A_INL_PCT", "RES_PLANT_CAPTURE", AI, "%", 100), ("REDUCTION_B_PCT", "RES_REDUCTION", B, "%", 100),
    ("INTENSITY_B_KG_KG", "RES_INTENSITY_KG_KG", B, "kg/kg", 1), ("TAILGAS_C_B_KMOL_H", "RES_TAILGAS_C_KMOL_H", B, "kmol/h", 1),
    ("RECYCLED_C_B_KMOL_H", "RES_RECYCLED_C_KMOL_H", B, "kmol/h", 1),
    ("BURNED_C_B_KMOL_H", "RES_BURNED_C_KMOL_H", B, "kmol/h", 1),
    # Heat
    ("R_RATED_MWTH", "REACTOR_RATED_MWTH", B, "MWth", 1), ("R_NEEDED_B_MWTH", "RES_REACTOR_NEEDED_MWTH", B, "MWth", 1),
    ("R_NEEDED_A_INL_MWTH", "RES_REACTOR_NEEDED_MWTH", AI, "MWth", 1),
    ("R_NEEDED_A_MEA_MWTH", "RES_REACTOR_NEEDED_MWTH", AM, "MWth", 1),
    ("R_BEST_B_MWTH", "RES_REACTOR_BEST_MWTH", B, "MWth", 1), ("R_HIGH_B_MWTH", "RES_REACTOR_HIGH_MWTH", B, "MWth", 1),
    ("MARGIN_B_MWTH", "RES_MARGIN_210_MWTH", B, "MWth", 1), ("MARGIN_A_MEA_MWTH", "RES_MARGIN_210_MWTH", AM, "MWth", 1),
    ("Q_PROCESS_B_MWTH", "RES_PROCESS_HEAT_MW", B, "MWth", 1), ("Q_PROCESS_A_MWTH", "RES_PROCESS_HEAT_MW", AI, "MWth", 1),
    ("REBOILER_B_MW", "RES_REBOILER_MW", B, "MW", 1), ("REBOILER_GAP_B_MW", "RES_REBOILER_GAP_MW", B, "MW", 1),
    ("POWER_BRANCH_B_MWTH", "RES_POWER_BRANCH_MWTH", B, "MWth", 1),
    ("HE_SEC_B_KG_S", "RES_HE_SEC_KG_S", B, "kg/s", 1), ("HE_PRIM_B_KG_S", "RES_HE_PRIM_KG_S", B, "kg/s", 1),
    # Power
    ("E_TOTAL_B_MWE", "RES_E_TOTAL_MWE", B, "MWe", 1), ("E_TOTAL_A_MEA_MWE", "RES_E_TOTAL_MWE", AM, "MWe", 1),
    ("E_PROCESS_B_MWE", "RES_E_PROCESS_MWE", B, "MWe", 1), ("E_CIRC_PRIM_B_MWE", "RES_E_CIRC_PRIM_MWE", B, "MWe", 1),
    ("E_CIRC_SEC_B_MWE", "RES_E_CIRC_SEC_MWE", B, "MWe", 1),
    ("GRID_IMPORT_A_MEA_MWE", "RES_GRID_IMPORT_AT_RATED_MWE", AM, "MWe", 1),
    # Plant cost anchors and targets
    ("SMR_ISLAND_CAPITAL_SM", "RES_SMR_ISLAND_CAPITAL_SM", B, "S$M", 1),
    ("SMR_ISLAND_FOM_SM_Y", "RES_SMR_ISLAND_FOM_SM_Y", B, "S$M/y", 1),
    ("TARGET_COST_S_T", "TARGET_COST", B, "S$/t", 1), ("TARGET_ABATEMENT_T_Y", "TARGET_ABATEMENT", B, "t/y", 1),
    ("TARGET_ABATEMENT_MT", "TARGET_ABATEMENT", B, "Mt/y", 1e-6),
    ("ABATEMENT_ABOVE_TARGET_T_Y", "RES_TARGET_ABATEMENT_MARGIN_T_Y", B, "t/y", 1),
    ("GAS_PRICE_S_GJ", "GAS_CENTRAL", B, "S$/GJ", 1), ("TS_CENTRAL_S_T", "TS_CENTRAL", B, "S$/tCO2", 1),
    ("OCC_CENTRAL_USD_KWT", "OCC_CENTRAL", B, "US$2019/kWt", 1),
]

# Freeze 7.5 screening values and tolerances. Used only to report drift, never printed.
SCREENING = [
    ("H2 Case B, t/y", "RES_H2_T_Y", 66925.0, 1.0, False),
    ("Net abatement Case B, t/y", "RES_ABATEMENT_T_Y", 578052.0, 0.005, True),
    ("Reactor needed Case B, MWth", "RES_REACTOR_NEEDED_MWTH", 179.0, 1.0, False),
    ("L3 cost, BOAK, 25 y, S$/t", "AC_L3_BOAK_25Y_N1P00", 137.7, 1.0, False),
]

# One quantity written in two results files. A mismatch means the CSVs came from
# different exports or the exporter disagrees with itself, so the build stops.
IDENTITIES = [
    ("RES_ABATEMENT_T_Y", "EM_B_REC_F_INL_9P21_DIRECT_ATM_ABATEMENT_T_Y"),
    ("RES_ABATEMENT_T_Y", "LEDGER_ABATEMENT_B_T_Y"),
    ("RES_ABATEMENT_T_Y", "SENS_CHARGED_210_A_T_Y"),
    ("RES_CAPTURED_T_Y", "EM_B_REC_F_INL_9P21_DIRECT_ATM_CAPTURED_T_Y"),
    ("RES_ATM_T_Y", "EM_B_REC_F_INL_9P21_DIRECT_ATM_N2_RICH_CUT_T_Y"),
    ("RES_REACTOR_NEEDED_MWTH", "EB_B_REC_F_HTGR_HEAT"),
    ("RES_REACTOR_NEEDED_MWTH", "SENS_CHARGED_210_R_NEEDED"),
    ("RES_MARGIN_210_MWTH", "EB_B_REC_F_SPARE_MARGIN"),
    ("RES_E_TOTAL_MWE", "EB_B_REC_F_POWER_GENERATED"),
    ("RES_REBOILER_MW", "EB_B_REC_F_MEA_REGEN"),
    ("RES_CAPTURED_KMOL_H", "MB_B_REC_F_CO2_TO_STORAGE_C_KMOL_H"),
    ("RES_TAILGAS_C_KMOL_H", "MB_B_REC_F_PSA_REJECT_RETURN_C_KMOL_H"),
    ("RES_RECYCLED_C_KMOL_H", "MB_B_REC_F_RECYCLE_TO_REFORMER_C_KMOL_H"),
    ("RES_BURNED_C_KMOL_H", "MB_B_REC_F_N2_RICH_CUT_C_KMOL_H"),
    ("AC_L3_BOAK_25Y", "SENS_CHARGED_210_AC_L3_25"),
    ("AC_L3_BOAK_60Y", "SENS_CHARGED_210_AC_L3_60"),
    ("AC_L5_BOAK_25Y", "SENS_CHARGED_210_AC_L5_25"),
    ("AC_L5_BOAK_60Y", "SENS_CHARGED_210_AC_L5_60"),
]

NUCLEAR_LINES = ("REACTOR_CAP", "REACTOR_OM", "IHX", "LICENCE", "DECOM", "IDC", "POWER_ISLAND")
RATED_TOL_MWTH = 0.01
CLOSE_REL = 1e-6


class BuildError(Exception):
    pass


def num(s):
    """Float from a CSV cell, or None for a blank or non-finite cell."""
    if s is None or s.strip() == "":
        return None
    try:
        v = float(s)
    except ValueError:
        return None
    return v if v == v and abs(v) != float("inf") else None


def close(a, b, rel=CLOSE_REL):
    return abs(a - b) <= rel * max(1.0, abs(a), abs(b))


def fmt(v, dp):
    r = round(v, dp)
    return f"{(0.0 if r == 0 else r):,.{dp}f}"


def fmt_auto(v):
    return f"{v:,.0f}" if abs(v) >= 1000 else f"{v:.4g}"


def tag_n(n):
    return f"N{n:.2f}".replace(".", "P")


def tag_test(v):
    return f"{v:.6g}".replace(".", "P").replace("-", "M").replace("+", "")


class Tokens:
    def __init__(self):
        self.t = {}
        self.agreed = 0

    def add(self, name, value, unit, source, status="", derived=False, places=None):
        if name in self.t:
            old = self.t[name]["value"]
            same = old == value or (isinstance(old, float) and isinstance(value, float) and close(old, value))
            if not same:
                raise BuildError(f"token {name} defined twice: {old} from {self.t[name]['source']}, "
                                 f"{value} from {source}")
            if self.t[name]["source"].split(":")[0] != source.split(":")[0]:
                self.agreed += 1
            return
        if value is None:
            text = table = "not run" if status == "not_run" else "n/a"
        elif isinstance(value, str):
            text = table = value
        elif places or unit in PLACES:
            text, table = (fmt(value, dp) for dp in (places or PLACES[unit]))
        else:
            text = table = fmt_auto(value)
        entry = {"value": value, "text": text, "table": table, "unit": unit, "source": source}
        if status:
            entry["status"] = status
        if derived:
            entry["derived"] = True
        self.t[name] = entry

    def value(self, name):
        if name not in self.t:
            raise BuildError(f"missing token {name}")
        return self.t[name]["value"]


def read_csvs():
    tables, sources = {}, {}
    missing = [n for n in CSV_COLUMNS if not (RESULTS / n).is_file()]
    if missing:
        raise BuildError(f"missing results files: {missing}")
    for name, need in CSV_COLUMNS.items():
        raw = (RESULTS / name).read_bytes()
        rows = list(csv.DictReader(raw.decode("utf-8").splitlines()))
        if not rows:
            raise BuildError(f"{name} has no rows")
        lost = need - set(rows[0])
        if lost:
            raise BuildError(f"{name} lacks columns {sorted(lost)}")
        tables[name] = rows
        sources[name] = {"sha256": hashlib.sha256(raw).hexdigest(), "rows": len(rows)}
    return tables, sources


# Rows kept out of the tokens. Decision S12: the modelled SRD of the failed M6
# gate may appear only in the drift table and the validation rows, with the gate
# failure beside it, so no builder can print it on its own.
NOT_TOKENS = {"TERM_T06_MODELLED"}


def add_basis(tok, rows):
    """Every parameter and key result. The bare id is Case B."""
    for r in rows:
        if r["id"] in NOT_TOKENS:
            continue
        src = f"Final_Design_Basis.csv:{r['id']}"
        unit, status = r["unit"], r["status"]
        for suffix, col in (("", B), ("_AINL", AI), ("_AMEA", AM), ("_LOW", "low"), ("_HIGH", "high")):
            v = num(r[col])
            if v is not None:
                tok.add(r["id"] + suffix, v, unit, f"{src}:{col}", status)


def add_aliases(tok, rows):
    by_id = {r["id"]: r for r in rows}
    lost = []
    for name, rid, col, unit, scale in ALIASES:
        r = by_id.get(rid)
        v = num(r[col]) if r else None
        if v is None:
            lost.append(f"{name} from {rid}:{col}")
            continue
        tok.add(name, v * scale, unit, f"Final_Design_Basis.csv:{rid}:{col}", r["status"])
    if lost:
        raise BuildError("aliases with no value: " + "; ".join(lost))


def ledger_key(r, rated):
    charged = num(r["reactor_charged_mwth"])
    need = "" if abs(charged - rated) <= RATED_TOL_MWTH else "_NEED"
    return f"{r['stage']}_{r['life_y']}Y_{tag_n(num(r['n']))}{need}"


def add_ledger(tok, rows, rated, checks):
    """L0 to L5 for the headline run on its whole grid, other runs at n = 1.0, then DCF rows."""
    groups = {}
    for r in rows:
        groups.setdefault((r["run_id"], ledger_key(r, rated)), []).append(r)
    worst = 0.0
    for (run, key), grp in groups.items():
        src = f"Economic_Ledger.csv:{run}:{key}"
        status = grp[0]["status"]
        lines = [r for r in grp if r["level"] != "DCF"]
        total, last = 0.0, {}
        for r in lines:
            v = num(r["value_sgdm_y"])
            cum, ac, ab = num(r["cum_sgdm_y"]), num(r["cum_s_per_t"]), num(r["abatement_t_y"])
            total += v
            worst = max(worst, abs(total - cum), abs(cum * 1e6 / ab - ac))
            last[r["level"]] = r
            if run == "B_REC_F":
                tok.add("LEDGER_ABATEMENT_B_T_Y", ab, "t/y", "Economic_Ledger.csv:B_REC_F:abatement_t_y", status)
                tok.add(f"LINE_{r['line_id']}_{key}", v, "S$M/y", f"{src}:{r['line_id']}", status)
                if not key.endswith("_NEED") and "N1P00" in key:
                    tok.add(f"LINE_{r['line_id']}_{key}_LOW", num(r["low"]), "S$M/y", f"{src}:{r['line_id']}:low", status)
                    tok.add(f"LINE_{r['line_id']}_{key}_HIGH", num(r["high"]), "S$M/y", f"{src}:{r['line_id']}:high", status)
                    tok.add(f"CUM_{r['line_id']}_{key}", ac, "S$/t", f"{src}:{r['line_id']}:cum_s_per_t", status)
        for lvl, r in last.items():
            ac, cum = num(r["cum_s_per_t"]), num(r["cum_sgdm_y"])
            if run == "B_REC_F":
                tok.add(f"AC_{lvl}_{key}", ac, "S$/t", f"{src}:{lvl}", status)
                tok.add(f"COST_{lvl}_{key}", cum, "S$M/y", f"{src}:{lvl}", status)
                if key.endswith("_N1P00"):
                    short = key[: -len("_N1P00")]
                    tok.add(f"AC_{lvl}_{short}", ac, "S$/t", f"{src}:{lvl}", status)
                    tok.add(f"COST_{lvl}_{short}", cum, "S$M/y", f"{src}:{lvl}", status)
            else:
                short = key[: key.index("_N")]
                tok.add(f"AC_{lvl}_{run}_{short}", ac, "S$/t", f"{src}:{lvl}", status)
        for r in grp:
            if r["level"] != "DCF":
                continue
            v, unit = num(r["dcf_value"]), r["dcf_unit"]
            tok.add(f"{r['line_id']}_{key}", v, unit, f"{src}:{r['line_id']}", status)
            if key.endswith("_N1P00"):
                short = key[: -len("_N1P00")]
                tok.add(f"{r['line_id']}_{short}", v, unit, f"{src}:{r['line_id']}", status)
                if r["line_id"] == "LCOH_SMR":
                    tok.add(f"LCOH_SMR_{r['life_y']}Y", v, unit, f"{src}:{r['line_id']}", status)
    if worst > 1e-3:
        raise BuildError(f"ledger cumulative columns disagree with the line values by {worst:.2e}")
    checks.append({"name": "ledger_cumulative_closure", "state": "ok",
                   "detail": f"largest residual {worst:.2e} over all ledger keys"})


def add_rows(tok, rows, prefix, keyfn, fields, file):
    """Generic table to token map. A row the engine skipped gives a null token."""
    for r in rows:
        key = keyfn(r)
        status = r.get("status", "")
        src = f"{file}:{key}"
        wrote = False
        for suffix, col, unit in fields(r):
            v = num(r[col])
            if v is not None:
                tok.add(f"{prefix}_{key}{suffix}", v, unit, f"{src}:{col}", status)
                wrote = True
        if not wrote:
            tok.add(f"{prefix}_{key}", None, "", src, status)


def energy_fields(r):
    u = {"electricity": "MWe"}.get(r.get("kind", ""), "MW")
    return (("", "value_mw", u), ("_LOW", "low_mw", u), ("_HIGH", "high_mw", u))


def mass_fields(r):
    return (("_KG_H", "total_kg_h", "kg/h"), ("_KMOL_H", "total_kmol_h", "kmol/h"),
            ("_H2_KMOL_H", "h2", "kmol/h"), ("_C_KMOL_H", "carbon_kmol_h", "kmol/h"),
            ("_T_C", "t_c", "C"), ("_P_BAR", "p_bar", "bar"))


def emission_fields(r):
    return (("_T_Y", "t_per_y", "t/y"), ("_KG_KG", "kg_per_kg_h2", "kg/kg"))


# Unit of each Sensitivity_Results.csv metric, one entry per metric. A metric
# missing here stops the build (sens_unit), so no token takes a unit it does not
# have. S$/t is the abatement cost per tonne CO2e; the T&S price is per tonne
# CO2 stored, as TS_CENTRAL is.
SENS_UNITS = {
    **{m: "S$/t" for m in ("AC_L3", "AC_L3_25", "AC_L3_60", "AC_L5", "AC_L5_25", "AC_L5_60",
                           "LAC_DCF", "LAC_DCF_25", "LAC_DCF_60", "COST_S_PER_T")},
    "TS_NEEDED": "S$/tCO2",
    **{m: "t/y" for m in ("A_T_Y", "A_EXTRA_T_Y", "H2_EXTRA_T_Y", "EMIT_BACKUP_T_Y")},
    **{m: "kmol/h" for m in ("BURNED_C_KMOL_H", "CAPTURED_KMOL_H", "FS_CAPTURED_KMOL_H",
                             "FS_EMITTED_C_KMOL_H", "RECYCLED_C_KMOL_H", "TAIL_GAS_C_KMOL_H",
                             "TUBE_C_KMOL_H")},
    "R_NEEDED": "MWth", "MARGIN_210": "MWth",
    "FS_PROCESS_HEAT_MW": "MW", "PROCESS_HEAT_MW": "MW", "GAP_MW": "MW",
    "NG_MMSCFD": "MMSCFD", "FS_NG_MMSCFD": "MMSCFD",
    "GAS_NEEDED": "S$/GJ", "GAS_PRINTED": "S$/GJ",
    "COFUND_SHARE": "-", "COFUND_S_M": "S$M", "REACTOR_CAPITAL_S_M": "S$M",
    "OCC_NEEDED": "US$2019/kWt",
    "LCOH_NUSH2": "S$/kg", "LCOH_SMR": "S$/kg",
    "PHI_AFTER_SHIFT": "-",
}


def sens_unit(metric):
    """Unit of a Sensitivity_Results.csv metric; an unlisted metric stops the build."""
    if metric not in SENS_UNITS:
        raise BuildError(f"Sensitivity_Results.csv metric {metric} has no unit in SENS_UNITS")
    return SENS_UNITS[metric]


def add_sensitivity(tok, rows):
    seen = Counter((r["sweep_id"], tag_test(num(r["test_value"])), r["metric"]) for r in rows)
    sweeps = {}
    for r in rows:
        tag = tag_test(num(r["test_value"]))
        head = f"SENS_{r['sweep_id']}_{tag}"
        if seen[(r["sweep_id"], tag, r["metric"])] > 1:
            head += f"_{r['run_id']}"
        unit = sens_unit(r["metric"])
        src = f"Sensitivity_Results.csv:{r['sweep_id']}:{r['run_id']}:{tag}:{r['metric']}"
        tok.add(f"{head}_{r['metric']}", num(r["result"]), unit, src, r["status"])
        tok.add(f"{head}_{r['metric']}_DELTA", num(r["delta"]), unit, src + ":delta", r["status"])
        s = sweeps.setdefault(r["sweep_id"], {"parameter": r["parameter"], "unit": r["unit"],
                                              "base_value": num(r["base_value"]), "tests": {}})
        s["tests"][head] = {"test_value": num(r["test_value"]), "run_id": r["run_id"], "parameter": r["parameter"]}
    return {k: {**v, "tests": [{"prefix": p, **t} for p, t in v["tests"].items()]} for k, v in sweeps.items()}


def add_mature(tok, rows, checks):
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

    def put(name, r, unit, col="result", scale=1.0, places=None):
        tok.add(name, num(r[col]) * scale, unit, src(r, col), r["status"], places=places)
        return tok.value(name)

    x = put("MATURE_60Y_GAS_PRINTED_S_GJ", one("MATURE_60Y", "MATURE_60Y", "GAS_PRINTED"), "S$/GJ")
    needed = []
    for m in ("L5", "DCF"):
        run = f"MATURE_60Y_{m}"
        g = one("MATURE_60Y", run, "GAS_NEEDED")
        # The two solves print to S$0.01/GJ, so the printed price can be read beside them.
        needed.append(put(f"MATURE_60Y_{m}_GAS_NEEDED_S_GJ", g, "S$/GJ", places=(2, 2)))
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


def add_validation(tok, rows):
    def count(col, value):
        return float(sum(r[col] == value for r in rows))

    src = "Validation_Results.csv"
    tok.add("VAL_N_TOTAL", float(len(rows)), "rows", src, derived=True)
    for name, col, value in (("VAL_N_PASS", "status", "pass"), ("VAL_N_FAIL", "status", "fail"),
                             ("VAL_N_NOT_RUN", "status", "not_run"),
                             ("VAL_N_REPORTED", "status", "reported"),
                             ("VAL_N_INDEPENDENT", "data_use", "independent"),
                             ("VAL_N_CALIBRATION", "data_use", "calibration"),
                             ("VAL_N_SAME_DATASET", "data_use", "same_dataset_disclosed")):
        tok.add(name, count(col, value), "rows", f"{src}:{col}={value}", derived=True)
    for status in ("pass", "fail", "reported", "not_run"):
        n = float(sum(r["data_use"] == "independent" and r["status"] == status for r in rows))
        tok.add(f"VAL_N_INDEPENDENT_{status.upper()}", n, "rows",
                f"{src}:data_use=independent,status={status}", derived=True)
    for r in rows:
        key = r["test_id"].upper()
        tok.add(f"VAL_{key}_STATUS", r["status"], "", f"{src}:{r['test_id']}:status")
        tok.add(f"VAL_{key}_DATA_USE", r["data_use"], "", f"{src}:{r['test_id']}:data_use")
        for suffix, col in (("_LIT", "literature"), ("_SIM", "simulated"), ("_REL", "rel_error")):
            v = num(r[col])
            if v is not None:
                tok.add(f"VAL_{key}{suffix}", v, r["unit"] if col != "rel_error" else "", f"{src}:{r['test_id']}:{col}")


def add_derived(tok, tables, checks):
    """Arithmetic on results only. Nothing here reads an input file or adds physics."""
    d = dict(derived=True)
    ac = tok.value
    # Gas saved and carbon views the freeze prints beneath the headline.
    tok.add("NG_SAVED_B_MMSCFD", ac("NG_UNABATED_B_MMSCFD") - ac("NG_B_MMSCFD"), "MMSCFD", "NG_UNABATED_B_MMSCFD - NG_B_MMSCFD", **d)
    base = "EM_B_REC_F_INL_9P21"
    direct = ac(f"{base}_DIRECT_ATM_ABATEMENT_T_Y")
    burden = ac(f"{base}_LIFECYCLE_BURDENS_TS_BURDEN_T_Y")
    tok.add("A_TS_BURDEN_B_T", direct + burden, "t/y", "direct abatement + TS_BURDEN line", **d)
    tok.add("A_TS_BURDEN_B_MT", (direct + burden) * 1e-6, "Mt/y", "direct abatement + TS_BURDEN line", **d)
    for name, view in (("A_LC_BURDENS_B_MT", "LIFECYCLE_BURDENS"), ("A_LC_UPSTREAM_B_MT", "LIFECYCLE_UPSTREAM"), ("A_SYSTEM_B_MT", "SYSTEM")):
        tok.add(name, ac(f"{base}_{view}_ABATEMENT_T_Y") * 1e-6, "Mt/y", f"{base}_{view}_ABATEMENT_T_Y / 1e6", **d)
    tok.add("BASELINE_INL_KG_KG", ac(f"{base}_DIRECT_ATM_BASELINE_KG_KG"), "kg/kg", f"{base}_DIRECT_ATM_BASELINE_KG_KG", **d)
    tok.add("BASELINE_CHO_KG_KG", ac("BASELINE_CHO"), "kg/kg", "BASELINE_CHO", **d)
    # One unrounded IEAGHG value everywhere.
    # Three decimals, so the one IEAGHG value prints as 8.996 and never as a rounded 9.00.
    tok.add("BASELINE_IEAGHG_KG_KG", ac("RES_BASELINE_IEAGHG_KG_KG"), "kg/kg", "RES_BASELINE_IEAGHG_KG_KG",
            places=(3, 3), **d)

    # Reformer pinch: reactor outlet less the IHX approach less the syngas temperature at the
    # reformer exit, at the central approach and at the high end of its range.
    for name, appr in (("PINCH_CENTRAL_K", "IHX_APPROACH_K"), ("PINCH_IAEA_K", "IHX_APPROACH_K_HIGH")):
        tok.add(name, ac("ROT") - ac(appr) - ac("SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C"), "K",
                f"ROT - {appr} - SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C", places=(0, 0), **d)
    # Units from FOAK to BOAK on the learning rate: 2 to the power log(FOAK) / -log(1 - LEARNING).
    tok.add("UNITS_FOAK_TO_BOAK", 2 ** (math.log(ac("FOAK_CENTRAL")) / -math.log(1 - ac("LEARNING"))), "units",
            "2 ** (log(FOAK_CENTRAL) / -log(1 - LEARNING))", places=(0, 0), **d)

    # Capital recovery factors and reactor overnight capital recovered from the 25 y line.
    r_disc = ac("DISCOUNT")
    crf = {}
    for life in (25, 40, 60):
        crf[life] = r_disc / (1 - (1 + r_disc) ** -life)
        tok.add(f"CRF_{life}Y", crf[life], "CRF", f"DISCOUNT, {life} y", **d)
    for stage in ("BOAK", "FOAK", "10OAK_P46", "10OAK_P47"):
        for n in ("N1P00", "N0P60", "N0P31"):
            tok.add(f"REACTOR_CAPITAL_{stage}_{n}_SM", ac(f"LINE_REACTOR_CAP_{stage}_25Y_{n}") / crf[25], "S$M",
                    f"LINE_REACTOR_CAP_{stage}_25Y_{n} / CRF_25Y", **d)

    # DCF test: LAC_DCF against L5, percent of L5.
    worst = 0.0
    for stage in ("BOAK", "FOAK"):
        for life in (25, 60):
            for n in ("N1P00", "N0P60", "N0P31"):
                key = f"{stage}_{life}Y_{n}"
                gap = 100 * (ac(f"LAC_DCF_{key}") - ac(f"AC_L5_{key}")) / ac(f"AC_L5_{key}")
                tok.add(f"LAC_L5_GAP_PCT_{key}", gap, "%", f"(LAC_DCF_{key} - AC_L5_{key}) / AC_L5_{key}", **d)
                if not close(ac(f"LAC_L5_GAP_{key}"), ac(f"LAC_DCF_{key}") - ac(f"AC_L5_{key}"), 1e-5):
                    raise BuildError(f"LAC_L5_GAP row for {key} disagrees with LAC_DCF less L5")
                worst = max(worst, abs(gap))
                if n == "N1P00":
                    tok.add(f"LAC_L5_GAP_PCT_{stage}_{life}Y", gap, "%", f"LAC_L5_GAP_PCT_{key}", **d)
                    prem = ac(f"LCOH_NUSH2_{stage}_{life}Y") - ac(f"LCOH_SMR_{life}Y")
                    tok.add(f"LCOH_PREMIUM_{stage}_{life}Y", prem, "S$/kg", f"LCOH_NUSH2_{stage}_{life}Y - LCOH_SMR_{life}Y", **d)
    tok.add("DCF_TEST_MAX_GAP_PCT", worst, "%", "largest absolute LAC_L5_GAP_PCT on the grid", **d)
    # The same gap over every DCF point the ledger carries, all stages, lives, exponents and sizes.
    gaps = [(abs(num(r["dcf_value"])), r) for r in tables["Economic_Ledger.csv"] if r["line_id"] == "LAC_L5_GAP_REL"]
    if not gaps:
        raise BuildError("Economic_Ledger.csv has no LAC_L5_GAP_REL rows")
    top, row = max(gaps, key=lambda g: g[0])
    tok.add("DCF_GAP_MAX_ALL_PCT", 100 * top, "%",
            f"Economic_Ledger.csv:LAC_L5_GAP_REL, largest of {len(gaps)} points "
            f"({row['stage']}, {row['life_y']} y, n {row['n']}, {row['reactor_charged_mwth']} MWth)",
            places=(2, 2), **d)
    tok.add("DCF_GAP_N_POINTS", float(len(gaps)), "points", "Economic_Ledger.csv:LAC_L5_GAP_REL rows", **d)
    tok.add("DCF_TEST_RESULT", "within 10%" if worst <= 10 else "over 10%", "", "DCF_TEST_MAX_GAP_PCT against 10%", **d)
    checks.append({"name": "dcf_vs_l5", "state": "ok" if worst <= 10 else "warn", "detail": f"largest gap {worst:.3f}% of L5"})

    # Target tests over the central cases: BOAK and FOAK, 25 and 60 y, L5 and DCF.
    target = ac("TARGET_COST")
    central = [ac(f"{m}_{s}_{y}Y") for m in ("AC_L5", "LAC_DCF") for s in ("BOAK", "FOAK") for y in (25, 60)]
    passes = sum(v <= target for v in central)
    tok.add("TARGET_COST_N_CENTRAL", float(len(central)), "rows", "L5 and DCF, BOAK and FOAK, 25 and 60 y", **d)
    tok.add("TARGET_COST_N_PASS", float(passes), "rows", "central cases at or below TARGET_COST", **d)
    tok.add("TARGET_COST_RESULT", "met" if passes == len(central) else ("met in part" if passes else "missed"), "", "TARGET_COST_N_PASS", **d)
    tok.add("TARGET_ABATEMENT_RESULT", "met" if ac("RES_ABATEMENT_T_Y") >= ac("TARGET_ABATEMENT") else "missed", "", "RES_ABATEMENT_T_Y against TARGET_ABATEMENT", **d)

    # Distance to the cost target, one lever at a time, for L3 and the full cost L5.
    abatement = ac("RES_ABATEMENT_T_Y")
    ceiling = target * abatement * 1e-6
    tok.add("CEILING_SM_Y", ceiling, "S$M/y", "TARGET_COST x RES_ABATEMENT_T_Y", **d)
    sens = {}
    for r in tables["Sensitivity_Results.csv"]:
        if r["sweep_id"] == "GAS":
            sens.setdefault(r["metric"], []).append((num(r["test_value"]), num(r["result"])))
    gas0, ts0, occ0 = ac("GAS_CENTRAL"), ac("TS_CENTRAL"), ac("OCC_CENTRAL")
    for stage in ("BOAK", "FOAK"):
        for life in (25, 60):
            for lvl in ("L3", "L5"):
                key, tag = f"{stage}_{life}Y_N1P00", f"{lvl}_{stage}_{life}Y"
                excess = ac(f"COST_{lvl}_{key}") - ceiling
                tok.add(f"EXCESS_{tag}_SM_Y", excess, "S$M/y", f"COST_{lvl}_{key} - CEILING_SM_Y", **d)
                nuclear = sum(ac(f"LINE_{ln}_{key}") for ln in NUCLEAR_LINES
                              if ln not in ("IDC", "POWER_ISLAND") or lvl == "L5")
                tok.add(f"NUCLEAR_ITEMS_{tag}_SM_Y", nuclear, "S$M/y", "sum of the nuclear ledger lines to this level", **d)
                tok.add(f"BUDGET_NUCLEAR_{tag}_SM_Y", nuclear - excess, "S$M/y", "ceiling less the non-nuclear lines", **d)
                scaled = ac(f"LINE_REACTOR_CAP_{key}") + (ac(f"LINE_IDC_{key}") if lvl == "L5" else 0.0)
                occ = occ0 * (1 - excess / scaled)
                tok.add(f"LEVER_OCC_{tag}_USD_KWT", occ, "US$2019/kWt",
                        "OCC_CENTRAL x (1 - excess / capital lines); same stage premium applies", **d)
                tok.add(f"LEVER_TS_{tag}_S_T", ts0 - excess * 1e6 / ac("RES_CAPTURED_T_Y"), "S$/tCO2",
                        "TS_CENTRAL - excess / captured", **d)
                pts = sorted(sens.get(f"AC_{lvl}_{life}", []))
                if stage == "BOAK" and len(pts) >= 2:
                    slope = (pts[-1][1] - pts[0][1]) / (pts[-1][0] - pts[0][0])
                    needed = gas0 + (target - ac(f"AC_{lvl}_{key}")) / slope
                    tok.add(f"LEVER_GAS_{tag}_S_GJ", needed, "S$/GJ",
                            f"linear in the GAS sweep, metric AC_{lvl}_{life}", **d)
                    tok.add(f"LEVER_GAS_{tag}_MULTIPLE", needed / gas0, "x",
                            f"LEVER_GAS_{tag}_S_GJ / GAS_CENTRAL", places=(1, 2), **d)


def run_checks(tok, tables, checks):
    # A number written in two files must match, or Tokens.add has already refused it.
    wrong = [f"{a} {tok.value(a):.6f} against {b} {tok.value(b):.6f}" for a, b in IDENTITIES
             if not close(tok.value(a), tok.value(b), 1e-5)]
    # The product stream holds its CH4 as well, so H2 is checked on its species.
    h2_kg_h = tok.value("MB_B_REC_F_H2_PRODUCT_H2_KMOL_H") * 2.01588
    if not close(tok.value("RES_H2_KG_H"), h2_kg_h, 1e-5):
        wrong.append(f"RES_H2_KG_H {tok.value('RES_H2_KG_H'):.6f} against H2 in H2_PRODUCT {h2_kg_h:.6f}")
    if wrong:
        raise BuildError("results files disagree: " + "; ".join(wrong))
    checks.append({"name": "same_number_in_two_files", "state": "ok",
                   "detail": f"{tok.agreed} shared tokens and {len(IDENTITIES)} named identities agree"})
    for stage, life in (("BOAK", 25), ("BOAK", 60), ("FOAK", 25), ("FOAK", 60)):
        excess = tok.value(f"AC_L5_{stage}_{life}Y") - tok.value("TARGET_COST")
        if not close(excess, tok.value(f"RES_TARGET_COST_EXCESS_L5_{stage}_{life}Y"), 1e-5):
            raise BuildError(f"target excess row for {stage} {life} y disagrees with AC_L5")
    em = tok.value("EM_B_REC_F_IEAGHG_9P00_DIRECT_ATM_ABATEMENT_T_Y")
    basis = tok.value("RES_ABATEMENT_IEAGHG_T_Y")
    checks.append({"name": "ieaghg_bound_two_values", "state": "ok" if close(em, basis, 1e-4) else "warn",
                   "detail": f"Final_Design_Basis {basis:,.0f} t/y against Emissions_Ledger {em:,.0f} t/y"})
    for label, token, screen, tol, relative in SCREENING:
        v = tok.value(token)
        lim = tol * screen if relative else tol
        checks.append({"name": f"drift: {label}", "state": "ok" if abs(v - screen) <= lim else "warn",
                       "detail": f"engine {v:,.2f}, screening {screen:,.2f}, tolerance {lim:,.2f}"})
    statuses = Counter()
    for name, rows in tables.items():
        statuses.update(r.get("status", "") for r in rows if name != "Validation_Results.csv")
    for s, c in sorted(statuses.items()):
        tok.add(f"STATUS_ROWS_{s.upper()}", float(c), "rows", "status column of the six results tables", derived=True)
    return dict(sorted(statuses.items()))


def build():
    tables, sources = read_csvs()
    basis = tables["Final_Design_Basis.csv"]
    tok, checks = Tokens(), []
    add_basis(tok, basis)
    add_aliases(tok, basis)
    rated = tok.value("REACTOR_RATED_MWTH")
    add_ledger(tok, tables["Economic_Ledger.csv"], rated, checks)
    add_rows(tok, tables["Integrated_Energy_Balance.csv"], "EB", lambda r: f"{r['run_id']}_{r['term_id']}",
             energy_fields, "Integrated_Energy_Balance.csv")
    add_rows(tok, tables["Integrated_Mass_Balance.csv"], "MB", lambda r: f"{r['run_id']}_{r['stream_id']}",
             mass_fields, "Integrated_Mass_Balance.csv")
    add_rows(tok, tables["Emissions_Ledger.csv"], "EM",
             lambda r: f"{r['run_id']}_{r['baseline_id'].upper()}_{r['view'].upper()}_{r['line_id']}",
             emission_fields, "Emissions_Ledger.csv")
    sweeps = add_sensitivity(tok, tables["Sensitivity_Results.csv"])
    add_validation(tok, tables["Validation_Results.csv"])
    add_derived(tok, tables, checks)
    add_mature(tok, tables["Sensitivity_Results.csv"], checks)
    statuses = run_checks(tok, tables, checks)
    used_units = {t["unit"] for t in tok.t.values()}
    return {
        "schema_version": 1,
        "sources": sources,
        "status_rows": statuses,
        "units_tex": {u: UNIT_TEX[u] for u in sorted(used_units) if u in UNIT_TEX},
        "checks": checks,
        "sweeps": sweeps,
        "tokens": dict(sorted(tok.t.items())),
    }


def verify():
    if not OUT_JSON.is_file():
        print("build/numbers.json does not exist")
        return 1
    have = json.loads(OUT_JSON.read_text(encoding="utf-8"))["sources"]
    _, now = read_csvs()
    stale = [n for n in now if have.get(n, {}).get("sha256") != now[n]["sha256"]]
    print("stale: " + ", ".join(stale) if stale else "numbers.json matches results/")
    return 1 if stale else 0


def self_test():
    """Unit map test. An unlisted metric must stop the build, and every
    sensitivity token built from results/ must carry its metric's unit."""
    failures = []

    def check(ok, what):
        print(f"{'PASS' if ok else 'FAIL'} {what}")
        if not ok:
            failures.append(what)

    row = {"sweep_id": "X", "run_id": "R", "parameter": "p", "unit": "u", "base_value": "1",
           "test_value": "2", "metric": "UNLISTED_METRIC", "result": "3", "delta": "1", "status": "s"}
    try:
        add_sensitivity(Tokens(), [row])
        check(False, "an unlisted metric stops the build")
    except BuildError:
        check(True, "an unlisted metric stops the build")
    for metric, unit in (("AC_L5_25", "S$/t"), ("LAC_DCF_60", "S$/t"), ("TS_NEEDED", "S$/tCO2"),
                         ("LCOH_NUSH2", "S$/kg"), ("OCC_NEEDED", "US$2019/kWt"),
                         ("EMIT_BACKUP_T_Y", "t/y"), ("TUBE_C_KMOL_H", "kmol/h"), ("GAP_MW", "MW"),
                         ("FS_NG_MMSCFD", "MMSCFD"), ("PHI_AFTER_SHIFT", "-")):
        check(sens_unit(metric) == unit, f"{metric} carries {unit}")
    data = build()
    cost = {m for m, u in SENS_UNITS.items() if u == "S$/t"}
    wrong, seen = [], set()
    for name, t in data["tokens"].items():
        parts = t["source"].split(":")
        if not (name.startswith("SENS_") and parts[0] == "Sensitivity_Results.csv"):
            continue
        metric = parts[4]
        seen.add(metric)
        if t["unit"] != sens_unit(metric) or (t["unit"] == "S$/t" and metric not in cost):
            wrong.append(f"{name} {t['unit']}")
    check(not wrong, f"every SENS token from results/ carries its metric's unit ({len(wrong)} wrong)")
    for w in wrong[:10]:
        print(f"  {w}")
    check(seen <= set(SENS_UNITS), f"{len(seen)} metrics in results/, all listed")
    print(f"self test: {len(failures)} failed")
    return 1 if failures else 0


def main():
    try:
        if "--verify" in sys.argv[1:]:
            return verify()
        if "--self-test" in sys.argv[1:]:
            return self_test()
        data = build()
    except BuildError as e:
        print(f"build_numbers: {e}", file=sys.stderr)
        return 1
    OUT_DIR.mkdir(exist_ok=True)
    OUT_JSON.write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    with OUT_INDEX.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f, lineterminator="\n")
        w.writerow(["token", "text", "table", "unit", "status", "source"])
        for name, t in data["tokens"].items():
            w.writerow([name, t["text"], t["table"], t["unit"], t.get("status", ""), t["source"]])
    print(f"wrote {OUT_JSON.relative_to(ROOT).as_posix()}: {len(data['tokens'])} tokens")
    print("row status: " + ", ".join(f"{k or 'none'} {v}" for k, v in data["status_rows"].items()))
    for c in data["checks"]:
        if c["state"] != "ok":
            print(f"WARN {c['name']}: {c['detail']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
