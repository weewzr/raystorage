import sys, json, pymupdf
from pptx import Presentation
pptx, pdf, out = sys.argv[1], sys.argv[2], sys.argv[3]
prs = Presentation(pptx)
lines=[]
doc = pymupdf.open(pdf)
for i, (sl, pg) in enumerate(zip(prs.slides, doc), 1):
    lines.append(f"===== SLIDE {i}")
    lines.append(pg.get_text().replace("\n"," | "))
    if sl.has_notes_slide:
        lines.append("NOTE: " + sl.notes_slide.notes_text_frame.text)
open(out,"w",encoding="utf-8").write("\n".join(lines))
