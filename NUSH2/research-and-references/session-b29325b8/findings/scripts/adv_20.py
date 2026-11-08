import pymupdf, sys, re
sys.stdout.reconfigure(encoding="utf-8")
d = pymupdf.open("C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793581431025-ns0tdf.pdf")
print(d.page_count)
n = 0
for i, pg in enumerate(d):
    t = re.sub(r"\s+", " ", pg.get_text())
    for m in re.finditer(r"(MWt\b|MWth|MW\(t\)|thermal power|\b750\b|\b565\b|°C|outlet temperature|\$\d[\d,.]*\s?/\s?kW)", t):
        n += 1
        if n <= 22:
            print(f"[p{i+1}] ...{t[max(0,m.start()-70):m.end()+80]}...")
print("hits", n)
