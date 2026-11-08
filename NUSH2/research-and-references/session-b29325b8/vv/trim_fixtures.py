"""Trim engine/vv/data/benchmarks to the rows a benchmark reads (lead decision S7).

Reads the read log of an instrumented export:
  TMPVVREAD|<file>|<data-row index>          rows returned by nush2_vv lookups
  TMPFIXREAD|<source>|<series>|<quantity>    rows returned by nush2_units::bench lookups
A units lookup returns the first row with that series and quantity, so the first
such row in file order is kept. Lines are copied verbatim. A file left with no
data row is reported for removal.
"""
import csv
import io
import json
import sys
from collections import defaultdict
from pathlib import Path

repo = Path(sys.argv[1])
vv_log = Path(sys.argv[2])
units_log = Path(sys.argv[3])
apply = len(sys.argv) > 4 and sys.argv[4] == "--apply"
bench = repo / "engine" / "vv" / "data" / "benchmarks"

vv_reads = defaultdict(set)
units_reads = defaultdict(set)
for line in vv_log.read_text(encoding="utf-8", errors="replace").splitlines():
    if line.startswith("TMPVVREAD|"):
        _, name, idx = line.split("|")
        vv_reads[name].add(int(idx))
for line in units_log.read_text(encoding="utf-8", errors="replace").splitlines():
    if line.startswith("TMPFIXREAD|"):
        _, source, series, quantity = line.split("|", 3)
        units_reads[source].add((series, quantity))

report = {}
for path in sorted(bench.glob("*.csv")):
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")
    header, body = lines[0], lines[1:]
    data = [l for l in body if l.strip()]
    rows = [next(csv.reader(io.StringIO(l))) for l in data]
    keep = set()
    if path.name in vv_reads:
        keep |= vv_reads[path.name]
    sources = {r[0] for r in rows}
    for src in sources & set(units_reads):
        for series, quantity in units_reads[src]:
            for i, r in enumerate(rows):
                if r[4] == series and r[5] == quantity:
                    keep.add(i)
                    break
            else:
                raise SystemExit(f"{path.name}: logged read {series} {quantity} not found")
    kept = [data[i] for i in sorted(keep)]
    report[path.name] = {"before": len(data), "after": len(kept),
                         "readers": ("vv " if path.name in vv_reads else "")
                         + ("units" if sources & set(units_reads) else "")}
    if apply:
        if kept:
            path.write_text(header + "\n" + "\n".join(kept) + "\n", encoding="utf-8", newline="\n")
        else:
            path.unlink()
print(json.dumps(report, indent=1))
