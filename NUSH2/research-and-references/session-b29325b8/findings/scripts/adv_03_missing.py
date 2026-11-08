import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
Q = {
 "TEV-981": r"TEV-?981",
 "Purge Recovery": r"purge recovery",
 "cryogenic": r"cryogenic (separation|unit)",
 "helium cp 5195": r"5\.?19[35]\s*(k?J|\b)|5193|5195|5\.19[35]\s*kJ",
 "helium mu 3.674": r"3\.674|2\.682|Petersen",
 "helium visc correlation": r"(viscosity|thermal conductivity) of helium|helium (viscosity|thermal conductivity)|KTA\s*3102|T\^?0\.7",
 "Cu/Zn LTS": r"Cu/ZnO|CuO/ZnO|Cu-?Zn|copper[- ]zinc|Cu/Zn|low[- ]temperature (water[- ]gas )?shift|LT[- ]?shift|\bLTS\b",
 "H2 solubility": r"(solubility|henry).{0,40}(hydrogen|\bH2\b)|(hydrogen|\bH2\b).{0,40}(solubility|henry)",
 "co-absorption": r"co-?absor",
 "Xe-100": r"Xe-?100|X-?energy",
 "Cho": r"Cho et al|Strezov|9\.35\s*kg",
 "Sheng": r"Sheng\b",
 "ETC": r"Energy Transitions Commission|ETC \(20",
 "capacity factor": r"capacity factor|availability factor|on-?stream (factor|time)|operating factor|plant availability",
 "GTHTR300C": r"GTHTR300C",
 "IDC": r"interest during construction",
}
EX = re.compile(r"^Implementation-of-Nuclear|^Claude/|^HTGR_SMR_MEA_CCS_Research_GitHub|\.css$|\.js$")
for name, p in Q.items():
    pat = re.compile(p, re.I)
    res = {}
    for (rel, pg), t in idx.items():
        if EX.search(rel):
            continue
        tt = re.sub(r"\s+", " ", t)
        ms = list(pat.finditer(tt))
        if ms:
            res.setdefault(rel, []).append((pg, len(ms), tt[max(0, ms[0].start() - 80):ms[0].end() + 110]))
    print(f"\n#### {name}: {len(res)} non-repo files")
    for rel, v in sorted(res.items(), key=lambda kv: -sum(x[1] for x in kv[1]))[:12]:
        n = sum(x[1] for x in v)
        print(f"  {rel[-70:]} hits={n} pages={[x[0] for x in v][:10]}")
        print(f"     e.g. p{v[0][0]}: {v[0][2]}")
