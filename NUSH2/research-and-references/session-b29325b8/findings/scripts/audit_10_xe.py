"""Read-only: text search in the downloaded X-energy investor deck (held in the tool-results folder)."""
import re, sys
from pathlib import Path
import pymupdf
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = Path(r"C:\Users\Alf\.claude\projects\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tool-results\webfetch-1793576514245-5faaup.pdf")
d = pymupdf.open(p)
print(len(d), d.metadata)
pat = re.compile(r"750|565|200 ?MW|80 ?MW|320 ?MW|/kW|kWe|\$\s?\d[\d,\.]*\s?(/|per)|2,900|9,200|5,250|overnight|FOAK|NOAK|billion|outlet", re.I)
n = 0
for i, pg in enumerate(d):
    t = pg.get_text("text")
    tt = " ".join(t.split())
    hits = [m for m in pat.finditer(tt)]
    print(f"[p{i+1}] chars={len(tt)} imgs={len(pg.get_images())} hits={len(hits)} :: {tt[:110]}")
    seen = 0
    last = -100
    for m in hits:
        if m.start() - last < 90:
            continue
        last = m.start()
        print("     ...", tt[max(0, m.start()-90):m.end()+110])
        seen += 1
        if seen >= 5:
            break
