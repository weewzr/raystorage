import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
for (rel, p), t in idx.items():
    if "4953394 (1)" in rel and p in (19, 21):
        print("---", p); print(re.sub(r"\s+", " ", t)[:2300])
