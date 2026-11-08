"""Render one PDF page to PNG. Usage: render.py pdf page(1-based) out.png [zoom] [clip x0,y0,x1,y1 in fractions]"""
import sys
import fitz

pdf, page, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
zoom = float(sys.argv[4]) if len(sys.argv) > 4 else 2.0
d = fitz.open(pdf)
pg = d[page - 1]
clip = None
if len(sys.argv) > 5:
    x0, y0, x1, y1 = [float(v) for v in sys.argv[5].split(",")]
    r = pg.rect
    clip = fitz.Rect(r.x0 + x0 * r.width, r.y0 + y0 * r.height, r.x0 + x1 * r.width, r.y0 + y1 * r.height)
pix = pg.get_pixmap(matrix=fitz.Matrix(zoom, zoom), clip=clip)
pix.save(out)
print(out, pix.width, pix.height)
