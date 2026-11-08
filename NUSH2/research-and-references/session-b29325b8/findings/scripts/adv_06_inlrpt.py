import pymupdf, re, sys, pickle
sys.stdout.reconfigure(encoding="utf-8")
p = r"C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793580492930-ryxsf0.pdf"
d = pymupdf.open(p)
print("pages", d.page_count, d.metadata.get("title"), d.metadata.get("creationDate"))
for n in (2, 5, 6, 7, 47):
    t = d[n - 1].get_text()
    print(f"\n=========== RPT PDF p{n} ===========\n{t[:4200]}")
pats = r"(learning|BOAK|beginning of a kind|first-of-a-kind|tenth|10-?OAK|owner|licens|decommission|interest during)"
hits = 0
for i, pg in enumerate(d):
    t = re.sub(r"\s+", " ", pg.get_text())
    for m in re.finditer(r"(BOAK|10-?OAK|beginning of a kind|interest during|Plant Licensing|Decommissioning|Owner.{0,3}s? Cost)", t):
        hits += 1
        if hits < 60:
            print(f"[p{i+1}] ...{t[max(0,m.start()-110):m.end()+130]}...")
print("hits", hits)
