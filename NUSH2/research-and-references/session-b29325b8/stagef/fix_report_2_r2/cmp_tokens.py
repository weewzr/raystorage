import json, sys
a = json.load(open(sys.argv[1], encoding="utf-8")); b = json.load(open(sys.argv[2], encoding="utf-8"))
ta, tb = a["tokens"], b["tokens"]
changed = [k for k in ta if k in tb and ta[k] != tb[k]]
removed = [k for k in ta if k not in tb]
added = [k for k in tb if k not in ta]
print(f"old {len(ta)} new {len(tb)}: changed {len(changed)}, removed {len(removed)}, added {len(added)}")
for k in changed[:20]: print("CHANGED", k, ta[k], tb[k])
for k in removed[:20]: print("REMOVED", k)
other = [k for k in a if k != "tokens" and a[k] != b.get(k)]
print("other top-level keys differing:", other)
for k in other:
    if isinstance(a[k], list):
        print(k, "old", len(a[k]), "new", len(b[k]))
        sa = {json.dumps(x, sort_keys=True) for x in a[k]}; sb = {json.dumps(x, sort_keys=True) for x in b[k]}
        print("  only old:", list(sa - sb)[:5]); print("  only new:", list(sb - sa)[:8])
    elif isinstance(a[k], dict):
        ks = set(a[k]) | set(b[k])
        print(k, [x for x in ks if a[k].get(x) != b[k].get(x)][:20])
print("ADDED:", " ".join(added))
