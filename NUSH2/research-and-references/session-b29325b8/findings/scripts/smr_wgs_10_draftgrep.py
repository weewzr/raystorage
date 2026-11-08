import re, sys
sys.stdout.reconfigure(encoding="utf-8")
base = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/txt/"
rx = re.compile(r"(water.gas shift|\bWGS\b|\bHTS\b|\bLTS\b|shift reactor|syngas cool|condensate|knock|Xu and|Froment|Hoseinzade|Kraussler|Chinchen|JAERI|mock-?up|effectiveness factor|steam.to.carbon|871|methane conversion|78\.1|Ergun|tail gas|tail-gas|purge)", re.I)
for f in ["final_v1_7.txt", "report_latest.txt", "report_revised_31oct.txt", "chapter3_recs.txt", "proposal.txt", "group_edited.txt", "amine_spec.txt"]:
    print("=====", f)
    t = open(base + f, encoding="utf-8", errors="replace").read().splitlines()
    n = 0
    for ln, line in enumerate(t, 1):
        for m in rx.finditer(line):
            s = max(0, m.start() - 110)
            e = min(len(line), m.end() + 130)
            print(f"L{ln}: ...{line[s:e]}...")
            n += 1
            if n > 26:
                break
        if n > 26:
            break
