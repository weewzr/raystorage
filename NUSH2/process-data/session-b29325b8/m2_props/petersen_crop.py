"""Crop the Petersen tables at high resolution. Read-only."""
import sys
import pymupdf

P = "C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/webfetch-1793596659066-91v1rr.pdf"
OUT = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/m2_props/"
d = pymupdf.open(P)
for n in (43, 44, 45):
    pg = d[n - 1]
    r = pg.rect
    # Rows from about 500 C to 950 C sit in the middle of each table.
    clip = pymupdf.Rect(r.x0, r.y0 + r.height * 0.0, r.x1, r.y0 + r.height * 0.34)
    pg.get_pixmap(dpi=230, clip=clip).save(f"{OUT}pet_t{n}_a.png")
    clip = pymupdf.Rect(r.x0, r.y0 + r.height * 0.34, r.x1, r.y0 + r.height * 0.68)
    pg.get_pixmap(dpi=230, clip=clip).save(f"{OUT}pet_t{n}_b.png")
print("ok")
