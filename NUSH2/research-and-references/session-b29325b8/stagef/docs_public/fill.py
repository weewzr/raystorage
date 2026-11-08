# Fill number placeholders from build/numbers.json and results/Validation_Results.csv.
#   [[tok:NAME]]  token text form      [[tab:NAME]]  token table form
#   [[int:NAME]]  integral value        [[pct:NAME]]  value x 100, integral
#   [[val:FAILED_LIST]], [[val:PASS_INDEPENDENT]]  computed from Validation_Results.csv
# Usage: python fill.py <repo> <template> <output>   or   python fill.py <repo> --inplace <file>...
import csv, json, re, sys
from pathlib import Path
repo = Path(sys.argv[1])
tok = json.loads((repo / "build" / "numbers.json").read_text(encoding="utf-8"))["tokens"]
rows = list(csv.DictReader((repo / "results" / "Validation_Results.csv").open(encoding="utf-8")))
val = {
    "FAILED_LIST": ", ".join("`%s`" % r["test_id"] for r in sorted(rows, key=lambda r: r["test_id"]) if r["status"] == "fail"),
    "PASS_INDEPENDENT": str(sum(1 for r in rows if r["status"] == "pass" and r["data_use"] == "independent")),
}
rx = re.compile(r"\[\[(tok|tab|int|pct|val):([A-Za-z0-9_\-]+)\]\]")
used = {}


def rep(m):
    kind, name = m.group(1), m.group(2)
    if kind == "val":
        used["val:" + name] = (val[name][:60], "Validation_Results.csv")
        return val[name]
    if name not in tok:
        raise SystemExit(f"unknown token {name}")
    t = tok[name]
    if kind in ("int", "pct"):
        x = float(t["value"]) * (100 if kind == "pct" else 1)
        if abs(x - round(x)) > 1e-9:
            raise SystemExit(f"token {name} = {x} is not integral")
        v = f"{int(round(x)):,}"
    else:
        v = t["text" if kind == "tok" else "table"]
    used[name] = (v, t.get("source"))
    return v


if sys.argv[2] == "--inplace":
    pairs = [(Path(f), Path(f)) for f in sys.argv[3:]]
else:
    pairs = [(Path(sys.argv[2]), Path(sys.argv[3]))]
for src, dst in pairs:
    out = rx.sub(rep, src.read_text(encoding="utf-8"))
    left = re.findall(r"\[\[[a-z]+:[^\]]*\]\]", out)
    if left:
        raise SystemExit(f"unfilled: {left[:5]}")
    dst.write_bytes(out.encode("utf-8"))
for k, (v, src) in sorted(used.items()):
    print(f"{k} = {v}  <- {src}")
