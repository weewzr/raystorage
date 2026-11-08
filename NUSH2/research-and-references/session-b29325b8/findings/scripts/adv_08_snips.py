import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
def pg(name, n, a=0, b=2500):
    for (rel, p), t in idx.items():
        if name in rel and p == n:
            t2 = re.sub(r"[ \t]+", " ", t)
            t2 = re.sub(r"\n\s*\n+", "\n", t2)
            print(f"--- {rel[-50:]} p{n} ---\n{t2[a:b]}\n")
            return
    print("NOT FOUND", name, n)
def grep(name, pt, ctx=130, mx=14):
    pat = re.compile(pt, re.I)
    c = 0
    for (rel, p), t in idx.items():
        if name not in rel:
            continue
        tt = re.sub(r"\s+", " ", t)
        for m in pat.finditer(tt):
            c += 1
            if c <= mx:
                print(f"[{rel[-34:]} p{p}] ...{tt[max(0,m.start()-ctx):m.end()+ctx]}...")
    print(f"== {name} / {pt}: {c} hits\n")
# P1772 availability section
grep("P1772", r"Availability and reliability", 40, 3)
grep("P1772", r"\b(9\d|8\d)\s?%.{0,50}availab|availab.{0,60}\b(9\d|8\d)\s?%|availability.{0,60}(required|target|requirement)", 160, 8)
# helium cp
grep("HTGR_SMR_MEA", r"519\d\b|5\.19\d?\b", 60, 6)
grep("1-s2.0-S0360319917332548", r"519\d|5\.19|helium.{0,40}(J/|kJ)", 80, 6)
grep("Modeling of the Helium", r"519\d|5\.19|heat capacity|specific heat", 100, 6)
grep("TE_1382", r"519\d|5\.19\d?\b", 80, 6)
grep("te_1236", r"519\d|5\.19\d?\b", 80, 6)
# TECDOC-1382 heat balance / circulator
grep("TE_1382", r"(PGC|gas circulator|helium circulator|circulator).{0,90}(kW|MW)|(kW|MW).{0,60}(PGC|gas circulator|helium circulator)", 140, 16)
pg("TE_1382", 163, 0, 1800)
pg("TE_1382", 164, 0, 1800)
pg("TE_1382", 168, 0, 1700)
grep("TE_1382", r"pressure (drop|loss)", 120, 10)
grep("Thermal Performance of Intermediate", r"pressure (drop|loss)", 120, 6)
