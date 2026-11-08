import json, os, sys, re
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
scan = json.load(open(SP + "/findings/inv_pdf_scan.json", encoding="utf-8"))
limit = int(sys.argv[1]) if len(sys.argv) > 1 else 1100
pat = sys.argv[2:]
for r in sorted(scan, key=lambda x: x["rels"][0].lower()):
    if pat and not any(p.lower() in " ".join(r["rels"]).lower() or r["sha"].startswith(p) for p in pat):
        continue
    print("#" * 80)
    print(r["sha"][:10], r["rels"])
    print("pages", r.get("pages"), "meta", {k: r["meta"][k] for k in r.get("meta", {}) if k in ("title", "author", "subject", "keywords")})
    print("dois", r.get("dois_head"))
    t = open(SP + "/findings/inv_txt/" + r["sha"][:10] + ".txt", encoding="utf-8").read()
    t = t.split("=====\n", 1)[1]
    t = re.sub(r"[ \t]+", " ", t)
    t = re.sub(r"\n\s*\n+", "\n", t)
    print(t[:limit])
