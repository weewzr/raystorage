"""Render PDF pages to PNG and make side-by-side sheets of three pages."""
import sys, pymupdf
from PIL import Image
pdf = pymupdf.open(sys.argv[1]); out = sys.argv[2]; dpi = int(sys.argv[3])
pages = [int(x) for x in sys.argv[4].split(",")] if len(sys.argv) > 4 else list(range(1, len(pdf) + 1))
files = []
for n in pages:
    f = f"{out}/p{n:02d}.png"; pdf[n - 1].get_pixmap(dpi=dpi).save(f); files.append((n, f))
for k in range(0, len(files), 3):
    ims = [Image.open(f) for _, f in files[k:k + 3]]
    h = max(i.height for i in ims); w = sum(i.width for i in ims) + 12 * (len(ims) - 1)
    s = Image.new("RGB", (w, h), (120, 120, 120)); x = 0
    for i in ims:
        s.paste(i, (x, 0)); x += i.width + 12
    s.save(f"{out}/s_{files[k][0]:02d}_{files[min(k + 2, len(files) - 1)][0]:02d}.png")
print("rendered", len(files))
