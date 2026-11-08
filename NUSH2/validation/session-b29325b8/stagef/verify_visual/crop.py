import sys
import pymupdf
sys.stdout.reconfigure(encoding="utf-8")
pdf, pno, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
x0, y0, x1, y1 = (float(v) for v in sys.argv[4:8])
zoom = float(sys.argv[8]) if len(sys.argv) > 8 else 3
doc = pymupdf.open(pdf)
page = doc[pno - 1]
clip = pymupdf.Rect(x0, y0, x1, y1)
page.get_pixmap(matrix=pymupdf.Matrix(zoom, zoom), clip=clip, alpha=False).save(out)
print("saved", out)
