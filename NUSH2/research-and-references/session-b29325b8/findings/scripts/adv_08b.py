import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
def pg(name, n, a=0, b=2500):
    for (rel, p), t in idx.items():
        if name in rel and p == n:
            t2 = re.sub(r"[ \t]+", " ", t)
            t2 = re.sub(r"\n\s*", " | ", t2)
            print(f"--- {rel[-46:]} p{n} ---\n{t2[a:b]}\n")
            return
pg("TE_1382", 168, 1500, 2400)
pg("TE_1382", 169, 0, 2600)
pg("P1772", 23, 0, 2400)
pg("P1772", 38, 0, 1800)
