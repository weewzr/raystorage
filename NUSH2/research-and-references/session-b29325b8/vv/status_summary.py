"""Status per matrix ID from a Validation_Results.csv."""
import csv
import re
import sys
from collections import defaultdict

rows = list(csv.DictReader(open(sys.argv[1], encoding="utf-8")))
ids = sys.argv[2:] if len(sys.argv) > 2 else None

def family(tid):
    m = re.match(r"^(T953-\d|T961-\d|XF-\d|CH-\d|KR-\d|AR-\d|HA-\d|VER-\d|VER-X|KH-\d|IS-\d|AH-\d|SA-\d|VI-\d|BR-\d|IA-\d|HT-\d|C-1|W-\d|Z-\d|X-1|H-\d|I-\d|A-1|S-\d|V-1)", tid)
    if m:
        return m.group(1)
    if tid.startswith("ARONU_104"):
        return "AR-1"
    return tid.split("-")[0] if "-" in tid else tid

fam = defaultdict(list)
for r in rows:
    fam[family(r["test_id"])].append(r)

for k in sorted(fam):
    if ids and k not in ids:
        continue
    rs = fam[k]
    c = defaultdict(int)
    for r in rs:
        c[r["status"]] += 1
    fails = [r["test_id"] for r in rs if r["status"] == "fail"]
    print(f"{k}: n={len(rs)} " + " ".join(f"{s}={n}" for s, n in sorted(c.items())) + (f" FAIL {fails}" if fails else ""))
