"""crop.py PDF PAGE x0 y0 x1 y1 DPI OUT  (PDF points)  or  crop.py PDF PAGE full DPI OUT"""
import sys
import pymupdf

pdf, pno = sys.argv[1], int(sys.argv[2])
doc = pymupdf.open(pdf)
page = doc[pno - 1]
if sys.argv[3] == "full":
    dpi, out = int(sys.argv[4]), sys.argv[5]
    page.get_pixmap(dpi=dpi).save(out)
else:
    x0, y0, x1, y1 = map(float, sys.argv[3:7])
    dpi, out = int(sys.argv[7]), sys.argv[8]
    page.get_pixmap(dpi=dpi, clip=pymupdf.Rect(x0, y0, x1, y1)).save(out)
print("wrote", out)
