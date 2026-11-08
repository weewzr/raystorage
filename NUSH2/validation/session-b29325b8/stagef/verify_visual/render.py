"""Render a PDF to PNGs (one per page) plus paired sheets. Usage: render.py <pdf> <outdir> <width_px> [pairs]"""
import sys
from pathlib import Path
import pymupdf
from PIL import Image, ImageDraw

pdf, out, width = sys.argv[1], Path(sys.argv[2]), int(sys.argv[3])
pairs = len(sys.argv) > 4 and sys.argv[4] == "pairs"
out.mkdir(parents=True, exist_ok=True)
doc = pymupdf.open(pdf)
print(f"{pdf}: {doc.page_count} pages, page0 {doc[0].rect}")
paths = []
for i, page in enumerate(doc, 1):
    z = width / page.rect.width
    p = out / f"p{i:02d}.png"
    page.get_pixmap(matrix=pymupdf.Matrix(z, z), alpha=False).save(p)
    paths.append(p)
if pairs:
    for s in range(0, len(paths), 2):
        ims = [Image.open(p).convert("RGB") for p in paths[s:s + 2]]
        w = max(im.width for im in ims)
        h = sum(im.height for im in ims) + 8 * (len(ims) - 1)
        sheet = Image.new("RGB", (w, h), (255, 0, 0))
        y = 0
        for p, im in zip(paths[s:s + 2], ims):
            sheet.paste(im, (0, y))
            ImageDraw.Draw(sheet).text((4, y + 2), p.stem, fill=(255, 0, 0))
            y += im.height + 8
        sheet.save(out / f"pair_{s // 2 + 1:02d}.png")
