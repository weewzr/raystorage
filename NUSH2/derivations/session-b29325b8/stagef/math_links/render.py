"""Render the pages of a PDF that hold given phrases, and print PDF metadata.

Usage: python -I render.py <pdf> <out dir> <phrase> [<phrase> ...]
"""
import sys
from pathlib import Path

import fitz  # pymupdf

pdf, out = Path(sys.argv[1]), Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
doc = fitz.open(pdf)
print("pages", doc.page_count, "metadata", doc.metadata)
fonts = set()
for page in doc:
    for f in page.get_fonts():
        fonts.add((f[3], f[1]))
print("fonts", len(fonts), "non-embedded", [f for f in fonts if f[1] in ("", "n/a")])
done = set()
for phrase in sys.argv[3:]:
    hits = [p.number for p in doc if p.search_for(phrase)]
    print(repr(phrase), "pages", [h + 1 for h in hits])
    for h in hits[:1]:
        if h in done:
            continue
        done.add(h)
        pix = doc[h].get_pixmap(dpi=110)
        pix.save(out / f"p{h + 1:03d}.png")
print("rendered", sorted(d + 1 for d in done))
