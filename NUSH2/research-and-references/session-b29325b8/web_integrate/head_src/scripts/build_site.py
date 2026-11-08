#!/usr/bin/env python3
"""Assemble the static site in _site/ from frontend/, the wasm module and the
seven results CSVs. Python stdlib only.

Steps:
 1. Delete and recreate the output folder.
 2. Copy the frontend files git knows about (tracked, or untracked and not
    ignored), leaving out any wasm file there.
 3. Copy the wasm module to nush2.wasm. Fail over 3 MB, or when the module does
    not carry the commit id given.
 4. Write data/*.json built from results/*.csv and copy the CSVs to data/csv/.
    Every number in the JSON is read from a CSV cell; nothing is computed here
    beyond counting rows.
 5. Replace __COMMIT__ in HTML, JS, CSS and SVG files. The frontend uses it as a
    ?v= stamp on every local resource, so a new commit never meets a cached
    file of the old one.
 6. Write build.json with commit, run id, UTC time and file hashes.
 7. Check that every local resource the HTML, CSS and JS name exists, is
    relative and carries the ?v= stamp. Fail over 6 MB in all.

Usage:
  python scripts/build_site.py --wasm frontend/nush2.wasm --results results \
      --commit "$(git rev-parse HEAD)" --run-id 0 --out _site [--test-log target/test.log]
Exit 0 on success, 1 on any failure.
"""
import argparse
import csv
import hashlib
import io
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
RESULTS = (
    "Economic_Ledger.csv",
    "Emissions_Ledger.csv",
    "Final_Design_Basis.csv",
    "Integrated_Energy_Balance.csv",
    "Integrated_Mass_Balance.csv",
    "Sensitivity_Results.csv",
    "Validation_Results.csv",
)
MAX_WASM = 3 * 1024 * 1024
MAX_SITE = 6 * 1024 * 1024
PLACEHOLDER = "__COMMIT__"
STAMPED = (".html", ".js", ".mjs", ".css", ".svg")
STAMP_TYPES = (".js", ".mjs", ".css", ".svg", ".json", ".wasm", ".csv")
HEADLINE_RUN = "B_REC_F"
COST_ID = re.compile(r"^(AC_L3|AC_L5|LAC_DCF|LCOH_NUSH2|LCOH_SMR)_([A-Z0-9]+)_(\d+)Y_N(\d)P(\d\d)$")
EXCESS_ID = re.compile(r"^RES_TARGET_COST_EXCESS_L5_([A-Z0-9]+)_(\d+)Y$")
COST_FIELD = {"AC_L3": "ac_l3", "AC_L5": "ac_l5", "LAC_DCF": "lac_dcf",
              "LCOH_NUSH2": "lcoh_nush2", "LCOH_SMR": "lcoh_smr"}
INT = re.compile(r"^[+-]?\d+$")
NUM = re.compile(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
# Native reference for the browser parity check: KPI id of the solve reply,
# then the CSV cell the native export wrote for the same quantity. These are
# the cells engine/wasm/src/tests.rs reads in default_solve_matches_the_published_results.
FDB = "Final_Design_Basis.csv"
EML = "Emissions_Ledger.csv"
KPI_CELLS = [
    ("rating_mwth", FDB, {"id": "REACTOR_RATED_MWTH"}, "case_b"),
    ("reactor_charged_mwth", FDB, {"id": "REACTOR_RATED_MWTH"}, "case_b"),
    ("reactor_required_mwth", FDB, {"id": "RES_REACTOR_NEEDED_MWTH"}, "case_b"),
    ("surplus_mwth", FDB, {"id": "RES_MARGIN_210_MWTH"}, "case_b"),
    ("q_process_mwth", FDB, {"id": "RES_PROCESS_HEAT_MW"}, "case_b"),
    ("h2_rich_kg_h", FDB, {"id": "RES_PSA_H2_FEED_KG_H"}, "case_b"),
    ("h2_product_t_y", FDB, {"id": "RES_H2_T_Y"}, "case_b"),
    ("ng_mmscfd", FDB, {"id": "RES_NG_MMSCFD"}, "case_b"),
    ("co2_captured_t_y", FDB, {"id": "RES_CAPTURED_T_Y"}, "case_b"),
    ("co2_atm_t_y", FDB, {"id": "RES_ATM_T_Y"}, "case_b"),
    ("capture_per_pass", FDB, {"id": "ETA_MEA"}, "case_b"),
    ("capture_plant", FDB, {"id": "RES_PLANT_CAPTURE"}, "case_b"),
    ("q_reboiler_mw", FDB, {"id": "RES_REBOILER_MW"}, "case_b"),
    ("reboiler_gap_mw", FDB, {"id": "RES_REBOILER_GAP_MW"}, "case_b"),
    ("e_aux_mwe", FDB, {"id": "RES_E_TOTAL_MWE"}, "case_b"),
    ("unmet_power_mwe", FDB, {"id": "RES_GRID_IMPORT_AT_RATED_MWE"}, "case_b"),
    ("abatement_t_y", FDB, {"id": "RES_ABATEMENT_T_Y"}, "case_b"),
    ("abatement_cho_t_y", FDB, {"id": "RES_ABATEMENT_CHO_T_Y"}, "case_b"),
    # Unrounded IEAGHG factor, as the ledger prints it (build order section 5 item 3).
    ("abatement_ieaghg_t_y", EML, {"run_id": HEADLINE_RUN, "baseline_id": "IEAGHG_9p00",
                                   "view": "direct_atm", "line_id": "ABATEMENT"}, "t_per_y"),
    ("abatement_system_t_y", EML, {"run_id": HEADLINE_RUN, "baseline_id": "INL_9p21",
                                   "view": "system", "line_id": "ABATEMENT"}, "t_per_y"),
    ("abatement_lifecycle_t_y", EML, {"run_id": HEADLINE_RUN, "baseline_id": "INL_9p21",
                                      "view": "lifecycle_upstream", "line_id": "ABATEMENT"}, "t_per_y"),
    ("solvent_t_h", FDB, {"id": "RES_SOLVENT_T_H"}, "case_b"),
    ("cost_s_per_t", FDB, {"id": "AC_L5_BOAK_25Y_N1P00"}, "case_b"),
    ("cost_l3_s_per_t", FDB, {"id": "AC_L3_BOAK_25Y_N1P00"}, "case_b"),
    ("cost_lac_dcf_s_per_t", FDB, {"id": "LAC_DCF_BOAK_25Y_N1P00"}, "case_b"),
    ("lcoh_nush2_s_per_kg", FDB, {"id": "LCOH_NUSH2_BOAK_25Y_N1P00"}, "case_b"),
    ("lcoh_smr_s_per_kg", FDB, {"id": "LCOH_SMR_BOAK_25Y_N1P00"}, "case_b"),
    ("target_abatement_t_y", FDB, {"id": "TARGET_ABATEMENT"}, "case_b"),
    ("target_cost_s_per_t", FDB, {"id": "TARGET_COST"}, "case_b"),
]
KPI_UNMAPPED = {
    "h2_rich_kmol_h": "Final_Design_Basis.csv prints the PSA H2 feed in kg/h only",
    "unmet_heat_mwth": "the engine has no unmet thermal demand field",
}


class BuildError(Exception):
    pass


# ------------------------------------------------------------------ CSV

def read_csv(path):
    text = path.read_text(encoding="utf-8")
    rows = list(csv.reader(io.StringIO(text)))
    if not rows:
        raise BuildError(f"{path.name} is empty")
    header = rows[0]
    out = []
    for i, r in enumerate(rows[1:], start=2):
        if not r:
            continue
        if len(r) != len(header):
            raise BuildError(f"{path.name} line {i}: {len(r)} cells against {len(header)} in the header")
        out.append(dict(zip(header, r)))
    return header, out


def typed(v):
    """CSV cell to a JSON value: empty is null, numbers are numbers, the rest stays text."""
    if v == "":
        return None
    if v in ("true", "false"):
        return v == "true"
    if INT.match(v):
        return int(v)
    if NUM.match(v):
        return float(v)
    return v


def typed_row(row, keys=None):
    return {k: typed(row[k]) for k in (keys or row.keys())}


def as_num(v, what):
    t = typed(v)
    if isinstance(t, bool) or not isinstance(t, (int, float)):
        raise BuildError(f"{what}: {v!r} is not a number")
    return t


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def find(rows, match, what):
    hits = [r for r in rows if all(cell_eq(r.get(k, ""), v) for k, v in match.items())]
    if len(hits) != 1:
        raise BuildError(f"{what}: {len(hits)} rows match {match}, expected one")
    return hits[0]


def cell_eq(cell, want):
    if isinstance(want, float):
        return NUM.match(cell) is not None and abs(float(cell) - want) <= 1e-9 * max(1.0, abs(want))
    return cell == want


# ------------------------------------------------------------------ data files

def sensitivity(rows, meta):
    sweeps, index = [], {}
    for r in rows:
        key = (r["sweep_id"], r["run_id"], r["metric"])
        if key not in index:
            index[key] = len(sweeps)
            sweeps.append({
                "sweep_id": r["sweep_id"], "run_id": r["run_id"], "parameter": r["parameter"],
                "unit": r["unit"], "base_value": typed(r["base_value"]), "metric": r["metric"],
                "class": r["class"], "note": r["note"], "points": [],
            })
        sweeps[index[key]]["points"].append({
            "test_value": typed(r["test_value"]), "result": typed(r["result"]),
            "delta": typed(r["delta"]), "status": r["status"],
        })
    return {"schema": 1, **meta, "rows": len(rows), "sweeps": sweeps}


def test_counts(log):
    if log is None:
        return None, "no test log was given to this build"
    if not log.is_file():
        return None, f"test log {log.name} is missing"
    suites, cur = [], None
    run_rx = re.compile(r"^\s*Running (?:unittests )?(\S+)(?: \((?:.*[\\/])?([^\\/)]+)\))?")
    res_rx = re.compile(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored")
    for line in log.read_text(encoding="utf-8", errors="replace").splitlines():
        m = run_rx.match(line)
        if m:
            name = m.group(2) or m.group(1)
            cur = re.sub(r"-[0-9a-f]{16}(\.exe)?$", "", name)
            continue
        m = res_rx.search(line)
        if m:
            suites.append({"suite": cur, "result": m.group(1), "passed": int(m.group(2)),
                           "failed": int(m.group(3)), "ignored": int(m.group(4))})
            cur = None
    if not suites:
        return None, "the test log holds no test result line"
    total = {k: sum(s[k] for s in suites) for k in ("passed", "failed", "ignored")}
    return {"suites": suites, **total}, ""


def validation(rows, meta, log):
    def count(key):
        out = {}
        for r in rows:
            out[r[key]] = out.get(r[key], 0) + 1
        return dict(sorted(out.items()))

    tests, why = test_counts(log)
    return {
        "schema": 1, **meta,
        "counts": {"total": len(rows), "by_status": count("status"), "by_level": count("level"),
                   "by_data_use": count("data_use")},
        "tests": tests, "tests_reason": why,
        "rows": [typed_row(r) for r in rows],
    }


def reference(tables):
    _, fdb = tables[FDB]
    kpi = []
    for kid, name, match, col in KPI_CELLS:
        _, rows = tables[name]
        r = find(rows, match, f"{kid} in {name}")
        kpi.append({"id": kid, "value": as_num(r[col], f"{kid} in {name}"), "file": name,
                    "row": match, "column": col, "unit": r.get("unit", "")})
    ahn_heat = as_num(find(fdb, {"id": "Q_PROCESS_AHN_CASE_B"}, "Ahn process heat")["case_b"], "Ahn process heat")
    _, sens = tables["Sensitivity_Results.csv"]
    match = {"sweep_id": "HEAT_BASIS", "run_id": HEADLINE_RUN, "metric": "MARGIN_210",
             "test_value": float(ahn_heat)}
    r = find(sens, match, "surplus_ahn_mwth")
    kpi.append({"id": "surplus_ahn_mwth", "value": as_num(r["result"], "surplus_ahn_mwth"),
                "file": "Sensitivity_Results.csv", "row": match, "column": "result", "unit": "MWth"})

    cost = {}
    for r in fdb:
        m = COST_ID.match(r["id"])
        if not m:
            continue
        metric, stage, life, a, b = m.groups()
        key = (stage, int(life), f"{a}.{b}")
        row = cost.setdefault(key, {"stage": stage, "life_y": int(life), "n": f"{a}.{b}",
                                    "charged": "rating", "label": r["parameter"]})
        row[COST_FIELD[metric]] = as_num(r["case_b"], r["id"])
        if metric == "AC_L5":
            row["label"] = r["parameter"]  # the headline row is named after its L5 cell
    excess = []
    for r in fdb:
        m = EXCESS_ID.match(r["id"])
        if m:
            v = as_num(r["case_b"], r["id"])
            excess.append({"stage": m.group(1), "life_y": int(m.group(2)), "excess_s_per_t": v,
                           "missed": v > 0, "label": r["parameter"]})
    target_cost = find(fdb, {"id": "TARGET_COST"}, "target cost")
    target_a = find(fdb, {"id": "TARGET_ABATEMENT"}, "target abatement")
    margin = find(fdb, {"id": "RES_TARGET_ABATEMENT_MARGIN_T_Y"}, "abatement margin")
    _, imb = tables["Integrated_Mass_Balance.csv"]
    closure = find(imb, {"run_id": HEADLINE_RUN, "stream_id": "CLOSURE_CARBON"}, "carbon closure")
    case_cols = ["id", "group", "parameter", "case_b", "case_a_inl", "case_a_mea", "unit", "equation", "status"]
    return {
        "schema": 1,
        "run_id": HEADLINE_RUN,
        "basis": "Case B, recycle, Headline preset; every value is a cell of results/*.csv written by the native engine",
        "tolerance": {"rel": 1e-6, "abs": 1e-6,
                      "basis": "relative 1e-6 between the browser solve and the native export; the absolute floor is one unit in the sixth decimal the CSVs print"},
        "kpi": kpi,
        "kpi_unmapped": [{"id": k, "reason": v} for k, v in KPI_UNMAPPED.items()],
        "cost_headline": sorted(cost.values(), key=lambda x: (x["stage"], x["life_y"], -float(x["n"]))),
        "targets": {
            "cost_s_per_t": as_num(target_cost["case_b"], "target cost"),
            "cost_source": target_cost["source"],
            "abatement_t_y": as_num(target_a["case_b"], "target abatement"),
            "abatement_source": target_a["source"],
            "abatement_margin_t_y": as_num(margin["case_b"], "abatement margin"),
            "cost_excess_l5": excess,
        },
        "closure": {"carbon_kmol_h": typed(closure["carbon_kmol_h"]), "file": "Integrated_Mass_Balance.csv",
                    "status": closure["status"]},
        "case_columns": [typed_row(r, case_cols) for r in fdb if r["status"] != "input"],
    }


def provenance(tables, files, commit, run_id):
    _, fdb = tables[FDB]

    def count(key, rows):
        out = {}
        for r in rows:
            out[r[key]] = out.get(r[key], 0) + 1
        return dict(sorted(out.items()))

    keep = ["id", "group", "parameter", "case_b", "case_a_inl", "case_a_mea", "unit", "low", "high",
            "class", "rigour", "source", "locator", "source_status", "decision"]
    terms = [typed_row(r, ["id", "parameter", "case_b", "unit", "source", "status"])
             for r in fdb if r["id"].startswith("TERM_")]
    return {
        "schema": 1, "commit": commit, "run_id": run_id,
        "results": files,
        "assumptions": [typed_row(r, keep) for r in fdb if r["status"] == "input"],
        "terms": terms,
        "counts": {"class": count("class", fdb), "rigour": count("rigour", fdb), "status": count("status", fdb)},
    }


# ------------------------------------------------------------------ frontend

def frontend_files(src):
    """Files git would publish from src: tracked, or untracked and not ignored."""
    try:
        out = subprocess.run(["git", "-C", str(ROOT), "ls-files", "--cached", "--others", "--exclude-standard",
                              "--", str(src.relative_to(ROOT))], capture_output=True, text=True, check=True).stdout
        names = [ROOT / n for n in out.splitlines() if n]
    except (subprocess.CalledProcessError, FileNotFoundError, ValueError):
        names = [p for p in src.rglob("*") if p.is_file()]
    keep = []
    for p in names:
        if not p.is_file():
            continue  # listed by git but deleted in the working tree
        rel = p.relative_to(src)
        if any(part.startswith(".") or part == "__pycache__" for part in rel.parts):
            continue
        if p.suffix == ".wasm":
            continue
        keep.append(rel)
    return sorted(keep)


class Refs(HTMLParser):
    RESOURCE = {("script", "src"), ("link", "href"), ("img", "src"), ("source", "src"), ("iframe", "src"),
                ("embed", "src"), ("object", "data"), ("video", "src"), ("audio", "src"), ("track", "src")}

    def __init__(self):
        super().__init__()
        self.refs = []

    def handle_starttag(self, tag, attrs):
        for k, v in attrs:
            if v is None:
                continue
            if (tag, k) in self.RESOURCE:
                self.refs.append((tag, k, v, True))
            elif k in ("href", "src", "action", "data", "poster"):
                self.refs.append((tag, k, v, False))


JS_REF = re.compile(r"""(?:\bfrom\s*|\bimport\s*\(\s*|^\s*import\s+|new\s+Worker\s*\(\s*|\bfetch\s*\(\s*|new\s+URL\s*\(\s*|importScripts\s*\(\s*)(["'`])([^"'`\n]+)\1""", re.M)
CSS_REF = re.compile(r"""url\(\s*["']?([^"')]+)["']?\s*\)|@import\s+["']([^"']+)["']""")


def is_absolute(url):
    s = urlsplit(url)
    return bool(s.scheme) or url.startswith("//")


def check_local(base_dir, site, url, where, stamp_required, problems):
    if url.startswith(("#", "data:", "mailto:", "javascript:")) or "${" in url:
        return
    if is_absolute(url):
        problems.append(f"{where}: resource from an absolute URL {url}")
        return
    s = urlsplit(url)
    if not s.path:
        return
    target = (base_dir / s.path).resolve()
    try:
        target.relative_to(site.resolve())
    except ValueError:
        problems.append(f"{where}: {url} points outside the site")
        return
    if not target.is_file():
        problems.append(f"{where}: {url} does not exist in the site")
    if stamp_required and target.suffix in STAMP_TYPES and "v=" not in s.query:
        problems.append(f"{where}: {url} has no ?v= stamp")


def check_refs(site, commit):
    problems = []
    for p in sorted(site.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(site).as_posix()
        if p.suffix == ".html":
            parser = Refs()
            parser.feed(p.read_text(encoding="utf-8"))
            for tag, attr, url, resource in parser.refs:
                if resource or not is_absolute(url):
                    check_local(p.parent, site, url, f"{rel} <{tag} {attr}>", resource, problems)
        elif p.suffix in (".js", ".mjs"):
            for m in JS_REF.finditer(p.read_text(encoding="utf-8")):
                check_local(p.parent, site, m.group(2), f"{rel} script reference", True, problems)
        elif p.suffix == ".css":
            for m in CSS_REF.finditer(p.read_text(encoding="utf-8")):
                check_local(p.parent, site, m.group(1) or m.group(2), f"{rel} url()", False, problems)
    stamps = set()
    for p in site.rglob("*"):
        if p.suffix in STAMPED:
            stamps.update(re.findall(r"\?v=([^\"'`&#)\s]*)", p.read_text(encoding="utf-8")))
    if stamps - {commit}:
        problems.append(f"?v= stamps other than the commit: {sorted(stamps - {commit})}")
    return problems


# ------------------------------------------------------------------ main

def write_json(path, obj):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, ensure_ascii=False, allow_nan=False, separators=(",", ":")) + "\n",
                    encoding="utf-8")


def git_head():
    try:
        return subprocess.run(["git", "-C", str(ROOT), "rev-parse", "HEAD"], capture_output=True, text=True,
                              check=True).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "local"


def build(args):
    commit = args.commit or git_head()
    if not re.fullmatch(r"[0-9a-f]{7,40}|local", commit):
        raise BuildError(f"commit {commit!r} is neither a hex id nor 'local'")
    run_id = str(args.run_id)
    if not re.fullmatch(r"[0-9]+|local", run_id):
        raise BuildError(f"run id {run_id!r} is not a number")
    src, out, res = args.frontend.resolve(), args.out.resolve(), args.results.resolve()
    if out in (ROOT, src, res) or ROOT.resolve() == out or out in ROOT.parents:
        raise BuildError(f"refusing to replace {out}")

    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    files = frontend_files(src)
    if not any(f.as_posix() == "index.html" for f in files):
        raise BuildError(f"{src} has no index.html")
    for rel in files:
        dst = out / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src / rel, dst)

    wasm = args.wasm.resolve()
    if not wasm.is_file():
        raise BuildError(f"wasm module {args.wasm} is missing; run scripts/build_wasm.py")
    data = wasm.read_bytes()
    if data[:4] != b"\0asm":
        raise BuildError(f"{args.wasm} is not a wasm module")
    if len(data) > MAX_WASM:
        raise BuildError(f"wasm is {len(data)} bytes, over {MAX_WASM}")
    if commit != "local" and commit.encode() not in data:
        raise BuildError(f"the wasm module does not carry commit {commit}; build it with GIT_SHA={commit}")
    (out / "nush2.wasm").write_bytes(data)

    tables, listing = {}, []
    for name in RESULTS:
        p = res / name
        if not p.is_file():
            raise BuildError(f"results file {name} is missing")
        tables[name] = read_csv(p)
        (out / "data" / "csv").mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, out / "data" / "csv" / name)
        listing.append({"file": name, "bytes": p.stat().st_size, "sha256": sha256(p),
                        "rows": len(tables[name][1]), "columns": tables[name][0]})
    meta = lambda name: {"source": f"results/{name}", "sha256": sha256(res / name)}  # noqa: E731
    write_json(out / "data" / "sensitivity.json",
               sensitivity(tables["Sensitivity_Results.csv"][1], meta("Sensitivity_Results.csv")))
    write_json(out / "data" / "validation.json",
               validation(tables["Validation_Results.csv"][1], meta("Validation_Results.csv"), args.test_log))
    write_json(out / "data" / "reference.json", reference(tables))
    write_json(out / "data" / "provenance.json", provenance(tables, listing, commit, run_id))

    for p in out.rglob("*"):
        if p.is_file() and p.suffix in STAMPED:
            text = p.read_text(encoding="utf-8")
            if PLACEHOLDER in text:
                p.write_text(text.replace(PLACEHOLDER, commit), encoding="utf-8")

    # build.json first, since the page fetches it and the reference check
    # below wants every fetched file present.
    write_json(out / "build.json", {
        "schema": 1, "commit": commit, "run_id": run_id,
        "built_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "wasm": {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()},
        "results": {x["file"]: x["sha256"] for x in listing},
    })

    problems = check_refs(out, commit)
    left = [p.relative_to(out).as_posix() for p in out.rglob("*")
            if p.is_file() and p.suffix in STAMPED and PLACEHOLDER in p.read_text(encoding="utf-8")]
    if left:
        problems.append(f"placeholder left in {left}")
    if problems:
        raise BuildError("site references:\n  " + "\n  ".join(problems))

    total = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    if total > MAX_SITE:
        raise BuildError(f"site is {total} bytes, over {MAX_SITE}")
    count = sum(1 for p in out.rglob("*") if p.is_file())
    print(f"built {out.name}: {count} files, {total} bytes, wasm {len(data)} bytes, commit {commit}, run {run_id}")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="Assemble _site/ from frontend/, the wasm module and results/*.csv")
    ap.add_argument("--frontend", type=Path, default=ROOT / "frontend")
    ap.add_argument("--wasm", type=Path, default=ROOT / "frontend" / "nush2.wasm")
    ap.add_argument("--results", type=Path, default=ROOT / "results")
    ap.add_argument("--commit", default=None, help="commit id the wasm module reports; default git HEAD")
    ap.add_argument("--run-id", default="local", help="CI run id, or 'local'")
    ap.add_argument("--test-log", type=Path, default=None, help="cargo test output, for the test counts")
    ap.add_argument("--out", type=Path, default=ROOT / "_site")
    args = ap.parse_args(argv)
    try:
        return build(args)
    except BuildError as e:
        print(f"FAIL: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
