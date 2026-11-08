import json, sys
sf = sys.argv[1]
old = json.load(open(sf + "/resolved_old.json", encoding="utf-8"))["tokens"]
new = json.load(open(sf + "/resolved_new.json", encoding="utf-8"))["tokens"]
used = open(sys.argv[2], encoding="utf-8").read().split()
ch = 0
for k in used:
    o, n = old.get(k), new.get(k)
    if n is None:
        print("MISSING in new:", k); ch += 1; continue
    if o is None:
        print("NEW only:", k); continue
    diffs = [f for f in ("text", "table", "unit") if o.get(f) != n.get(f)]
    if diffs:
        ch += 1
        print(k, {f: (o.get(f), n.get(f)) for f in diffs})
print("used", len(used), "changed", ch)
