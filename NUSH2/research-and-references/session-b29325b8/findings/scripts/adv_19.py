import pymupdf, sys, re
sys.stdout.reconfigure(encoding="utf-8")
B = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/"
for f in ("webfetch-1793581367866-frs1v7.pdf", "webfetch-1793581368567-10tt5w.pdf"):
    d = pymupdf.open(B + f)
    print("=====", f, d.page_count, d.metadata.get("title"), d.metadata.get("creationDate"))
    print(re.sub(r"\s+", " ", d[0].get_text())[:400])
    n = 0
    for i, pg in enumerate(d):
        t = re.sub(r"\s+", " ", pg.get_text())
        for m in re.finditer(r"(\bSMR\b|steam methane reform|TEV-96|TEV-95|TEV-98|hydrogen price|cost of hydrogen|\$/kg|\$/MMBtu|capacity factor|availability of|\bIRR\b|HTGR capital|\$/kW)", t, re.I):
            n += 1
            if n <= 26:
                print(f"[p{i+1}] ...{t[max(0,m.start()-110):m.end()+150]}...")
    print("hits", n)
