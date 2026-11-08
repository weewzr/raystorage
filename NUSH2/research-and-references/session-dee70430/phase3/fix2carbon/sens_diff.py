import csv
import sys

S = sys.argv[1]
old = list(csv.DictReader(open(f"{S}/results_before/Sensitivity_Results.csv", encoding="utf-8")))
new = list(csv.DictReader(open("results/Sensitivity_Results.csv", encoding="utf-8")))
key = lambda r: (r["sweep_id"], r["run_id"], r["test_value"], r["metric"])
o = {key(r): r for r in old}
n = {key(r): r for r in new}
assert set(o) == set(n), "row sets differ"
changed = {}
for k in o:
    for col in ("base_value", "result", "delta", "note", "status", "unit", "parameter"):
        a, b = o[k][col], n[k][col]
        same = a == b
        if not same:
            try:
                same = abs(float(a) - float(b)) <= max(1e-6, 1e-9 * max(abs(float(a)), abs(float(b))))
            except ValueError:
                same = False
        if not same:
            changed.setdefault((k[0], col), []).append(k)
for (sweep, col), ks in sorted(changed.items()):
    runs = sorted({k[1] for k in ks})
    print(sweep, col, len(ks), runs)
print("old vs new AC_L5_25 and A_T_Y per baseline run")
for run in ("B_REC_F_BASE_IEAGHG", "B_REC_F_BASE_INL", "B_REC_F_BASE_CHO"):
    for metric in ("AC_L3_25", "AC_L5_25", "LAC_DCF_25", "A_T_Y"):
        ko = [k for k in o if k[0] == "BASELINE" and k[1] == run and k[3] == metric][0]
        print(run, metric, o[ko]["result"], "->", n[ko]["result"])
