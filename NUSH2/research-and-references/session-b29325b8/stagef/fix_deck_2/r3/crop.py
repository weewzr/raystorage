import sys, pymupdf
# crop.py pdf page x0 y0 x1 y1 dpi out   (coordinates in a 1600-px-wide frame)
pdf, page, x0, y0, x1, y1, dpi, out = sys.argv[1], int(sys.argv[2]), *map(float, sys.argv[3:7]), int(sys.argv[7]), sys.argv[8]
d = pymupdf.open(pdf); pg = d[page-1]; s = pg.rect.width / 1600
clip = pymupdf.Rect(x0*s, y0*s, x1*s, y1*s)
pg.get_pixmap(matrix=pymupdf.Matrix(dpi/72, dpi/72), clip=clip, alpha=False).save(out)
print(out)
