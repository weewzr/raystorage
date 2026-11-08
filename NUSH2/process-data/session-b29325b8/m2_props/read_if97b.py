"""Print and render the IF97 verification tables. Read-only."""
import sys
import pymupdf

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
P = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793596923231-onsx2y.pdf"
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/m2_props/"
d = pymupdf.open(P)
for n in (9, 17, 34, 36):
    t = d[n - 1].get_text()
    i = t.find("Thermodynamic property values") if n in (9, 17) else t.find("Saturation")
    print("=== PDF p", n)
    print(t[i:i + 1000])
    d[n - 1].get_pixmap(dpi=110).save(f"{OUT}if97_p{n}.png")
