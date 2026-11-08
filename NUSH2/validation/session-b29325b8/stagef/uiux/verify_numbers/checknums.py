import json, sys, glob, os
rows = json.load(open(sys.argv[2], encoding="utf-8"))["rows"]
def fmt(v, u, d):
    x = v * 100 if u == "-" else v
    return f"{x:,.{d}f}"
tot = bad = 0
moved = {}
for f in sorted(glob.glob(os.path.join(sys.argv[1], "walk_*.json"))):
    d = json.load(open(f, encoding="utf-8"))
    name = os.path.basename(f)[5:-5]
    for s in d["steps"]:
        for n in s["nums"]:
            tot += 1
            if n["row"]:
                exp = fmt(rows[n["row"]]["value"], n["u"], int(n["d"]))
            elif isinstance(n["raw"], (int, float)):
                exp = fmt(n["raw"], n["u"], int(n["d"]))
            else:
                exp = "n/a"
            if exp != n["shown"]:
                bad += 1
                print("MISMATCH", name, s["i"], n["path"] or n["row"], n["shown"], exp)
            moved.setdefault((s["i"], n["path"] or n["row"]), {})[name] = n["shown"]
print("numbers checked", tot, "mismatches", bad)
print("--- numbers that differ between default, rating100 and case_a")
for k, v in sorted(moved.items()):
    a, b, c = v.get("default"), v.get("rating100"), v.get("case_a")
    if len({a, b, c}) > 1:
        print(k, "default", a, "| rating100", b, "| case_a", c)
