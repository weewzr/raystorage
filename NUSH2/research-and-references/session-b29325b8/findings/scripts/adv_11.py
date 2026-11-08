import pymupdf, sys, re
sys.stdout.reconfigure(encoding="utf-8")
p = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793580780217-2i4fqu.pdf"
d = pymupdf.open(p)
print(d.page_count)
for i, pg in enumerate(d):
    print(f"--- p{i+1}")
    print(re.sub(r"[ \t]+", " ", pg.get_text()))
