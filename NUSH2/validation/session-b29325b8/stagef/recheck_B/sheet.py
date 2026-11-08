"""sheet.py PDF OUTPREFIX DPI PER_SHEET COLS [first last]: render pages and tile them."""
import sys
from pathlib import Path
import pymupdf
from PIL import Image

pdf, pre, dpi, per, cols = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
doc = pymupdf.open(pdf)
first = int(sys.argv[6]) if len(sys.argv) > 6 else 1
last = int(sys.argv[7]) if len(sys.argv) > 7 else doc.page_count
pages = list(range(first, last + 1))
for k in range(0, len(pages), per):
    grp = pages[k:k + per]
    ims = []
    for p in grp:
        pix = doc[p - 1].get_pixmap(dpi=dpi)
        ims.append(Image.frombytes("RGB", (pix.width, pix.height), pix.samples))
    w = max(i.width for i in ims)
    h = max(i.height for i in ims)
    rows = (len(ims) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * w + (cols - 1) * 8, rows * h + (rows - 1) * 8), (120, 120, 120))
    for j, im in enumerate(ims):
        sheet.paste(im, ((j % cols) * (w + 8), (j // cols) * (h + 8)))
    out = f"{pre}_{grp[0]:02d}_{grp[-1]:02d}.png"
    sheet.save(out)
    print("wrote", out)
