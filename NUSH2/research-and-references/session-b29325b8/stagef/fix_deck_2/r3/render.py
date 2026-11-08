import sys, pymupdf
from pathlib import Path
pdf, out = sys.argv[1], Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
width = int(sys.argv[3]); pages = [int(a) for a in sys.argv[4:]]
d = pymupdf.open(pdf)
for p in pages:
    pg = d[p-1]; z = width / pg.rect.width
    pg.get_pixmap(matrix=pymupdf.Matrix(z, z), alpha=False).save(out / f"s{p:02d}.png")
print("rendered", pages)
