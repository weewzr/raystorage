import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
def pg(name, n, a=0, b=2500):
    for (rel, p), t in idx.items():
        if name in rel and p == n:
            print(f"--- {rel[-40:]} p{n} ---\n{re.sub(chr(10)+'+', ' | ', re.sub(r'[ 	]+', ' ', t))[a:b]}\n")
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
                print(f"[{rel[-22:]} p{p}] ...{tt[max(0,m.start()-ctx):m.end()+ctx]}...")
    print(f"== {pt}: {c} hits\n")
N = "4953394 (1)"
pg(N, 1, 0, 300)
pg(N, 16, 0, 1500)
pg(N, 20, 0, 1900)
grep(N, r"net (thermal |plant |power |cycle )?efficienc|circulator|Carnot", 170, 8)
grep(N, r"does not warrant|optimum ROT|optimum reactor outlet", 170, 5)
pg("P1772", 44, 0, 1500)
grep("P1772", r"HTR.PM|HTR PM", 100, 6)
