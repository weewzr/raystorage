import sys, pymupdf
from pathlib import Path
pdf, out, width = sys.argv[1], Path(sys.argv[2]), int(sys.argv[3]) if len(sys.argv) > 3 else 1600
pages = {int(a) for a in sys.argv[4:]} if len(sys.argv) > 4 else None
out.mkdir(parents=True, exist_ok=True)
doc = pymupdf.open(pdf)
for i, p in enumerate(doc, 1):
    if pages and i not in pages:
        continue
    z = width / p.rect.width
    p.get_pixmap(matrix=pymupdf.Matrix(z, z), alpha=False).save(out / f"p{i:02d}.png")
print(len(doc), "pages", doc[0].rect)
