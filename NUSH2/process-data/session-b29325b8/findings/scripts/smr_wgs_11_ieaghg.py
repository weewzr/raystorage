import re, sys, json, os
sys.stdout.reconfigure(encoding="utf-8")
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/smr_wgs_txt/all"
idx = json.load(open(os.path.join(OUT, "_index.json"), encoding="utf-8"))
needle = sys.argv[1]
pat = sys.argv[2]
maxn = int(sys.argv[3]) if len(sys.argv) > 3 else 40
for k, v in idx.items():
    if needle.lower() in v["path"].lower() and "(1)" not in v["path"]:
        print("FILE", v["path"], k, v.get("pages"))
        t = open(os.path.join(OUT, k + ".txt"), encoding="utf-8", errors="replace").read()
        pages = re.split(r"\n\n=== PDF page (\d+) of \d+ ===\n", t)
        n = 0
        for i in range(1, len(pages), 2):
            body = re.sub(r"\s+", " ", pages[i + 1])
            for m in re.finditer(pat, body, re.I):
                s = max(0, m.start() - 170)
                e = min(len(body), m.end() + 230)
                print(f"p{pages[i]}: {body[s:e]}")
                n += 1
                if n >= maxn:
                    break
            if n >= maxn:
                break
        break
