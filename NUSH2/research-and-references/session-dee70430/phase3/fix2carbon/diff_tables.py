"""Keyed cell-level difference of two result folders, summarised per table and column.

Reads only exported CSV text. Rows are keyed by the leading identifier columns of each
table (the same widths as the legacy snapshot test, plus an occurrence counter), so an
inserted row does not shift the others.
"""
import csv
import sys
from collections import Counter, defaultdict

KEY_WIDTH = {
    "Economic_Ledger.csv": 7,
    "Emissions_Ledger.csv": 4,
    "Final_Design_Basis.csv": 1,
    "Integrated_Energy_Balance.csv": 2,
    "Integrated_Mass_Balance.csv": 2,
    "Sensitivity_Results.csv": 7,
    "Subsidy_Scenarios.csv": 11,
    "Validation_Results.csv": 1,
}


def load(path, width):
    with open(path, encoding="utf-8", newline="") as fh:
        rows = list(csv.reader(fh))
    head, body = rows[0], rows[1:]
    seen = Counter()
    keyed = {}
    for r in body:
        k = tuple(r[:width])
        seen[k] += 1
        keyed[(k, seen[k])] = r
    return head, keyed


def same(a, b):
    if a == b:
        return True
    try:
        x, y = float(a), float(b)
    except ValueError:
        return False
    return abs(x - y) <= max(1e-6, 1e-9 * max(abs(x), abs(y)))


def main(old, new):
    for name, width in KEY_WIDTH.items():
        h0, k0 = load(f"{old}/{name}", width)
        h1, k1 = load(f"{new}/{name}", width)
        if h0 != h1:
            print(f"{name}: HEADER CHANGED")
            continue
        added = [k for k in k1 if k not in k0]
        removed = [k for k in k0 if k not in k1]
        by_col = defaultdict(int)
        for k, r0 in k0.items():
            r1 = k1.get(k)
            if r1 is None:
                continue
            for i, (a, b) in enumerate(zip(r0, r1)):
                if not same(a, b):
                    by_col[h0[i]] += 1
        total = sum(by_col.values())
        print(f"{name}: rows {len(k0)} -> {len(k1)}, added {len(added)}, removed {len(removed)}, changed cells {total}")
        for col, n in sorted(by_col.items(), key=lambda x: -x[1]):
            print(f"    {col}: {n}")
        if name == "Final_Design_Basis.csv" or name == "Validation_Results.csv":
            ids = Counter(k[0][0].split("_")[0] + "_" + k[0][0].split("_")[1] if "_" in k[0][0] else k[0][0] for k in added)
            print("    added id prefixes:", dict(ids))
        if removed:
            print("    removed sample:", removed[:3])


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
