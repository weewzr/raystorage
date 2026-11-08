import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
for (rel, p), t in idx.items():
    if "P1772" in rel and p == 23:
        tt = re.sub(r"\s+", " ", t)
        i = tt.find("The supply has to be reliable")
        print(tt[i:i+900])
