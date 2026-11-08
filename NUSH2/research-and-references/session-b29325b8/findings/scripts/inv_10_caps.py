"""Table captions, abstract and limit-related sentences per document, with PDF page numbers."""
import json, os, re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
pref = sys.argv[1:]
for sha in pref:
    fn = [f for f in os.listdir(SP + "/findings/inv_txt") if f.startswith(sha) and f.endswith("_full.txt")][0]
    t = open(SP + "/findings/inv_txt/" + fn, encoding="utf-8").read()
    pages = re.split(r"\n=== PDF p\.(\d+) ===\n", t)
    print("#" * 72, sha, "pages", (len(pages) - 1) // 2)
    seen = set()
    for i in range(1, len(pages), 2):
        pn = int(pages[i]); body = pages[i + 1]
        body1 = re.sub(r"\n(?=\S)", " ", body) if body.count("\n") > 200 and len(body) / max(1, body.count("\n")) < 12 else body
        for m in re.finditer(r"(?m)^\s*(Table|TABLE)\s+(S?[A-Z]?\d+(?:\.\d+)?)[\.:\s\-–—]*(.{0,110})", body1):
            k = (m.group(2))
            if k in seen:
                continue
            seen.add(k)
            print(f"  p{pn} Table {m.group(2)}: {m.group(3).strip()}")
    # highlighted ranges
    for kw in ["abstract", "Abstract", "ABSTRACT"]:
        idx = t.find(kw)
        if idx >= 0:
            print("  ABS:", re.sub(r"\s+", " ", t[idx:idx + 1000]))
            break
    if len(seen) == 0:
        print("  no table caption lines found")
