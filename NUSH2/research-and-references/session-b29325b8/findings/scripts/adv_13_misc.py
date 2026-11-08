import sys, re, pickle, pymupdf, os, zipfile
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
def pg(name, n, a=0, b=2500):
    for (rel, p), t in idx.items():
        if name in rel and p == n:
            print(f"--- {rel[-46:]} p{n} ---\n{re.sub(chr(10)+'+', ' | ', re.sub(r'[ 	]+', ' ', t))[a:b]}\n")
            return
    print("NOT FOUND", name, n)
def grep(name, pt, ctx=130, mx=10):
    pat = re.compile(pt, re.I)
    c = 0
    for (rel, p), t in idx.items():
        if name not in rel:
            continue
        tt = re.sub(r"\s+", " ", t)
        for m in pat.finditer(tt):
            c += 1
            if c <= mx:
                print(f"[{rel[-30:]} p{p}] ...{tt[max(0,m.start()-ctx):m.end()+ctx]}...")
    print(f"== {name} / {pt}: {c} hits\n")
# INL-21537: Table 2 and Table 4, efficiency basis
pg("4953394.pdf", 1, 0, 500)
pg("4953394.pdf", 16, 0, 2300)
pg("4953394.pdf", 20, 0, 2600)
grep("4953394.pdf", r"net (thermal |plant |power |cycle )?efficienc|gross|circulator|Carnot", 150, 10)
grep("4953394.pdf", r"optimum|does not warrant", 150, 6)
# Availability and cost in the TEVs
for nm in ("HTGR-Integrated Hydrogen", "Sensitivity of Hydrogen"):
    grep(nm, r"availabilit|on-?stream|capacity factor|\bhours? per year|hr/yr|h/y\b", 90, 6)
    grep(nm, r"\$\s?\d|US\$|\bcost\b|\bcosts\b", 70, 6)
# HTR-PM in library
grep("P1772", r"HTR-?PM", 150, 8)
grep("P1772", r"AVR.{0,120}950|950.{0,120}AVR", 120, 4)
grep("Thermodynamic analysis and optimization", r"750|reactor outlet|outlet temperature", 120, 5)
