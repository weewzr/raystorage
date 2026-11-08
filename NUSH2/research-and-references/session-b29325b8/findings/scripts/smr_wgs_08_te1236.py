import re, sys
p = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/smr_wgs_txt/te1236.txt"
t = open(p, encoding="utf-8").read()
pages = re.split(r"\n\n=== PDF page (\d+) of \d+ ===\n", t)
kw = {
    "EVA": r"\bEVA\b|EVA-|ADAM|KVK",
    "shift": r"\bshift\b",
    "catalyst tube": r"catalyst tube|reformer tube",
    "heat flux": r"kW/m",
    "fin": r"\bfin(s|ned)?\b",
    "pressure": r"pressure drop|pressure loss",
    "conversion": r"conversion",
    "Ni catalyst": r"Ni/|Ni-based|nickel|catalyst",
    "hydrogen rate": r"Nm3/h|Nm³/h|Nm 3",
}
for i in range(1, len(pages), 2):
    n = int(pages[i])
    b = pages[i + 1]
    c = {k: len(re.findall(v, b, re.I)) for k, v in kw.items()}
    if c["EVA"] or (c["catalyst tube"] > 0) or (c["shift"] > 0 and c["conversion"] > 0):
        print(n, {k: v for k, v in c.items() if v})
