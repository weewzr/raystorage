"""Query the page-level index: adv_q.py 'regex' [max_hits_per_file] [context_chars] [path_filter]"""
import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
pat = re.compile(sys.argv[1], re.I)
mx = int(sys.argv[2]) if len(sys.argv) > 2 else 3
ctx = int(sys.argv[3]) if len(sys.argv) > 3 else 110
flt = sys.argv[4] if len(sys.argv) > 4 else ""
excl = re.compile(r"Implementation-of-Nuclear|Claude/|^HTGR_SMR_MEA_CCS_Research_GitHub") if (len(sys.argv) > 5 and sys.argv[5] == "x") else None
cnt = {}
tot = 0
for (rel, pg), t in idx.items():
    if flt and flt.lower() not in rel.lower():
        continue
    if excl and excl.search(rel):
        continue
    tt = re.sub(r"\s+", " ", t)
    for m in pat.finditer(tt):
        tot += 1
        c = cnt.get(rel, 0)
        cnt[rel] = c + 1
        if c < mx:
            s = max(0, m.start() - ctx)
            e = min(len(tt), m.end() + ctx)
            print(f"[{rel[-60:]} p{pg}] ...{tt[s:e]}...")
print("TOTAL", tot, "in", len(cnt), "files")
