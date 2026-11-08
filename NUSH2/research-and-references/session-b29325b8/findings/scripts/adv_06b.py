import pymupdf, sys
sys.stdout.reconfigure(encoding="utf-8")
p = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793580492930-ryxsf0.pdf"
d = pymupdf.open(p)
for n in (46, 47, 48):
    print(f"=== p{n} ===")
    print(d[n-1].get_text())
