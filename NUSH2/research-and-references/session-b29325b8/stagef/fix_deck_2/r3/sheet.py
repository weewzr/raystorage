import sys, pymupdf
from PIL import Image, ImageDraw
from pathlib import Path
pdf, out = sys.argv[1], sys.argv[2]; pages = [int(a) for a in sys.argv[3:]]
d = pymupdf.open(pdf); W = 1000
ims = []
for p in pages:
    pg = d[p-1]; z = W / pg.rect.width
    pm = pg.get_pixmap(matrix=pymupdf.Matrix(z, z), alpha=False)
    ims.append((p, Image.frombytes("RGB", (pm.width, pm.height), pm.samples)))
h = ims[0][1].height
sheet = Image.new("RGB", (2 * W + 10, 2 * h + 10), "white")
for k, (p, im) in enumerate(ims[:4]):
    x, y = (k % 2) * (W + 10), (k // 2) * (h + 10)
    sheet.paste(im, (x, y)); ImageDraw.Draw(sheet).text((x + 4, y + 2), f"p{p}", fill=(220, 0, 0))
sheet.save(out); print(out)
