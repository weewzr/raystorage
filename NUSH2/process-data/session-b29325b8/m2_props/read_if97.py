"""Find the IF97 verification tables in the fetched release. Read-only."""
import re
import sys
import pymupdf

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
P = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793596923231-onsx2y.pdf"
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/m2_props/"
d = pymupdf.open(P)
print("pages", d.page_count)
print(d[0].get_text()[:400])
for i in range(d.page_count):
    t = d[i].get_text()
    for m in re.finditer(r"Table (5|15|35|36)\.?[^\n]{0,110}", t):
        print("PDF p", i + 1, m.group(0))
for n in (10, 17, 34, 35, 36):
    pass
