import sys
import pymupdf
from PIL import Image, ImageDraw
src = pymupdf.open(sys.argv[1]); deck = pymupdf.open(sys.argv[2])
sp, dp = int(sys.argv[3]), int(sys.argv[4]); out = sys.argv[5]
x0, y0, x1, y1 = (float(v) for v in sys.argv[6:10])
r1 = pymupdf.Rect(x0, y0, x1, y1); r2 = r1 * 1.5
z = 4
a = src[sp - 1].get_pixmap(matrix=pymupdf.Matrix(z, z), clip=r1, alpha=False)
b = deck[dp - 1].get_pixmap(matrix=pymupdf.Matrix(z / 1.5, z / 1.5), clip=r2, alpha=False)
A = Image.frombytes("RGB", (a.width, a.height), a.samples); B = Image.frombytes("RGB", (b.width, b.height), b.samples)
S = Image.new("RGB", (A.width + B.width + 10, max(A.height, B.height)), (255, 0, 0))
S.paste(A, (0, 0)); S.paste(B, (A.width + 10, 0)); S.save(out); print(A.size, B.size)
