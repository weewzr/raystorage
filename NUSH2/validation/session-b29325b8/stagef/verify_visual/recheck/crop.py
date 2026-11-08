import pymupdf as fitz, sys
# usage: crop.py pdf page(1-based) x0 y0 x1 y1 dpi out
pdf, pg, x0, y0, x1, y1, dpi, out = sys.argv[1], int(sys.argv[2]), *map(float, sys.argv[3:7]), int(sys.argv[7]), sys.argv[8]
d = fitz.open(pdf); p = d[pg-1]
p.get_pixmap(dpi=dpi, clip=fitz.Rect(x0,y0,x1,y1)).save(out)
