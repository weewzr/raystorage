"""Render pages of Risoe report 224 (Petersen 1970) from the fetched PDF. Read-only."""
import sys
import pymupdf

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
P = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793596659066-91v1rr.pdf"
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/m2_props/"
d = pymupdf.open(P)
print("pages", d.page_count)
for i in range(d.page_count):
    t = d[i].get_text()
    print(i + 1, len(t), t[:70].replace("\n", " | "))
pages = [int(a) for a in sys.argv[1:]]
for n in pages:
    pix = d[n - 1].get_pixmap(dpi=140)
    pix.save(f"{OUT}pet_p{n:02d}.png")
    print("saved", n, pix.width, pix.height)
