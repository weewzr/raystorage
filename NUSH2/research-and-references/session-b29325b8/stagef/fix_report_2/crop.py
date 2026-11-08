import sys, pymupdf
pdf, page, x0, y0, x1, y1, dpi, out = sys.argv[1], int(sys.argv[2]), *map(float, sys.argv[3:7]), int(sys.argv[7]), sys.argv[8]
d = pymupdf.open(pdf)
d[page - 1].get_pixmap(dpi=dpi, clip=pymupdf.Rect(x0, y0, x1, y1)).save(out)
