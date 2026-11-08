import pymupdf, sys, re
sys.stdout.reconfigure(encoding="utf-8")
p = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793580492930-ryxsf0.pdf"
d = pymupdf.open(p)
for i, pg in enumerate(d):
    t = re.sub(r"\s+", " ", pg.get_text())
    for m in re.finditer(r"(economies of scale|economy of scale|scaling|reactor size|smaller reactor|SMRs? (are|have|cost)|per-?kWe.{0,40}size|size of the reactor|Plant Licensing|licens(e|ing) (cost|fee)|\b12\s+Site|capacity factor)", t, re.I):
        print(f"[p{i+1}] ...{t[max(0,m.start()-160):m.end()+200]}...")
print("=== p30 ===")
print(re.sub(r"\s+", " ", d[29].get_text())[:1800])
print("=== p45 ===")
print(re.sub(r"\s+", " ", d[44].get_text())[:900])
print("=== p40 learning ===")
print(re.sub(r"\s+", " ", d[39].get_text())[600:2300])
