"""Search the cached library text for shift catalyst, water VLE and condensate terms."""
import os, re, json, sys
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/smr_wgs_txt/all"
idx = json.load(open(os.path.join(OUT, "_index.json"), encoding="utf-8"))
patterns = {
    "LTS": r"(low[- ]temperature (water[- ]gas )?shift|\bLTS\b|\bLT[- ]?shift|LT WGS|LTSR|LT-WGS)",
    "CuZn": r"(Cu[/-]?ZnO?|Cu[- /]Zn|copper[- ]zinc|copper[- ]based|copper oxide|CuO[/-]ZnO|Cu/ZnO/Al)",
    "HTS": r"(high[- ]temperature (water[- ]gas )?shift|\bHTS\b|\bHT[- ]?shift|HT WGS|HTSR)",
    "MTS": r"(medium[- ]temperature shift|\bMTS\b)",
    "water_VLE": r"(Antoine|IAPWS|vapou?r pressure of water|saturation pressure of water|steam tables?|Wagner equation|water vapou?r pressure)",
    "condensate": r"(condensate|knock[- ]?out|KO drum|flash drum|process condensate)",
    "syngas_cooler": r"(syngas cooler|process gas (boiler|cooler)|waste heat boiler|WHB\b|reformed gas boiler|heat recovery steam|shift effluent|gas cooler)",
    "Henry": r"(Henry.s (law|constant)|Sechenov|solubility of (H2|hydrogen|CO2|carbon dioxide) in water)",
}
res = {k: {} for k in patterns}
for key, v in idx.items():
    if "error" in v:
        continue
    t = open(os.path.join(OUT, key + ".txt"), encoding="utf-8", errors="replace").read()
    pages = re.split(r"\n\n=== PDF page (\d+) of \d+ ===\n", t)
    for name, pat in patterns.items():
        rx = re.compile(pat, re.I)
        for i in range(1, len(pages), 2):
            pn = int(pages[i])
            body = pages[i + 1]
            for m in rx.finditer(body):
                s = max(0, m.start() - 90)
                e = min(len(body), m.end() + 110)
                snip = re.sub(r"\s+", " ", body[s:e])
                res[name].setdefault(v["path"], []).append((pn, snip))

out = os.path.join(os.path.dirname(OUT), "library_grep.txt")
with open(out, "w", encoding="utf-8") as g:
    for name, d in res.items():
        g.write(f"\n########## {name}  files={len(d)}\n")
        for p, hits in sorted(d.items(), key=lambda kv: -len(kv[1])):
            g.write(f"\n--- {p}  hits={len(hits)}\n")
            for pn, s in hits[:6]:
                g.write(f"   p{pn}: {s}\n")
print(out)
for name, d in res.items():
    print(name, "files", len(d), "total", sum(len(h) for h in d.values()))
