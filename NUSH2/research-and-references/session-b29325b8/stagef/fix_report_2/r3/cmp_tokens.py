import json, sys
a = json.load(open(sys.argv[1], encoding="utf-8")); b = json.load(open(sys.argv[2], encoding="utf-8"))
ta, tb = a["tokens"], b["tokens"]
changed = [k for k in ta if k in tb and ta[k] != tb[k]]
removed = [k for k in ta if k not in tb]
added = [k for k in tb if k not in ta]
print(f"old {len(ta)} new {len(tb)}: changed {len(changed)}, removed {len(removed)}, added {len(added)}")
for k in changed[:30]: print("CHANGED", k, ta[k], tb[k])
for k in removed[:30]: print("REMOVED", k)
for k in a:
    if k == "tokens": continue
    if a[k] != b.get(k):
        if isinstance(a[k], list):
            sa = {json.dumps(x, sort_keys=True) for x in a[k]}; sb = {json.dumps(x, sort_keys=True) for x in b[k]}
            print("KEY", k, "list old", len(a[k]), "new", len(b[k]), "only old", len(sa - sb), "only new", len(sb - sa))
            for x in list(sb - sa)[:12]: print("   +", x[:200])
            for x in list(sa - sb)[:12]: print("   -", x[:200])
        elif isinstance(a[k], dict):
            ks = set(a[k]) | set(b[k])
            diff = [x for x in ks if a[k].get(x) != b[k].get(x)]
            print("KEY", k, "dict entries differing", len(diff), diff[:20])
        else:
            print("KEY", k, repr(a[k])[:100], repr(b.get(k))[:100])
for k in b:
    if k not in a: print("NEW TOP KEY", k)
if len(sys.argv) > 3: print("ADDED:", " ".join(added))
