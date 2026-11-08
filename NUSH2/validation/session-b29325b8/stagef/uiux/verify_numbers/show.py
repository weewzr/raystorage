import json, sys
d = json.load(open(sys.argv[1], encoding="utf-8"))
for s in d["steps"]:
    print(f"=== {s['i']}. {s['title']}")
    for k in ("tags", "warn", "scen"):
        if s[k]: print(f"  [{k}]", " | ".join(s[k]))
    for t in s["text"]: print("  T:", t)
    for e in s["eq"]: print("  EQ:", e)
    for f in s["figs"]: print("  FIG:", f)
    for st in s["status"]: print("  STATUS:", st)
    for n in s["notes"]: print("  NOTE:", n)
    for i in s["inset"]: print("  INSET:", i["cap"], "| svg text:", i["svgText"])
    if s["media"]: print("  MEDIA:", s["media"])
    print("  LIT:", s["lit"], "MOVING:", s["moving"])
    if len(sys.argv) > 2:
        for n in s["nums"]: print("   NUM", n["ctx"], n["path"] or n["row"], n["u"], n["d"], "shown", n["shown"], "raw", n["raw"])
