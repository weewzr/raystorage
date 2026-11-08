import json, sys
def walk(r, p):
    v = r
    for k in p.split("."):
        if isinstance(v, list) and not k.isdigit():
            v = next((e for e in v if isinstance(e, dict) and e.get("id") == k), None)
        elif isinstance(v, dict):
            v = v.get(k)
        elif isinstance(v, list):
            v = v[int(k)]
        else:
            return None
    return v
for name in sys.argv[2:]:
    d = json.load(open(f"{sys.argv[1]}/walk_{name}.json", encoding="utf-8"))
    r = d["reply"]
    print("=====", name)
    print(" targets:", json.dumps(r.get("targets"))[:900])
    print(" feasibility:", json.dumps(r.get("feasibility"))[:600])
    be = r["balances"]["energy"]
    keys = be.keys() if isinstance(be, dict) else [e.get("id") for e in be]
    for k in keys:
        v = be[k] if isinstance(be, dict) else next(e for e in be if e.get("id") == k)
        print("  energy", k, json.dumps(v)[:200])
    bp = r["balances"]["power"]
    for k in (bp.keys() if isinstance(bp, dict) else []):
        print("  power", k, json.dumps(bp[k])[:200])
    print("  units.power", json.dumps(r["units"]["power"])[:1500])
