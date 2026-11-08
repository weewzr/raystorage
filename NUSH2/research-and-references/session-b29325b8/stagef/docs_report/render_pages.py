#!/usr/bin/env python3
"""Render the PDF to PNG in tmp/pages for inspection, plus contact sheets of six pages each."""
import shutil
import sys
from pathlib import Path

import pymupdf
from PIL import Image

B = Path(__file__).resolve().parent
PDF = B.parent / "NUSH2_Final_Report.pdf"
OUT = B / "tmp" / "pages"


def main():
    dpi = int(sys.argv[1]) if len(sys.argv) > 1 else 72
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir(parents=True)
    doc = pymupdf.open(PDF)
    files = []
    for i, page in enumerate(doc):
        p = OUT / f"p{i + 1:02d}.png"
        page.get_pixmap(dpi=dpi).save(p)
        files.append(p)
    for s in range(0, len(files), 8):
        ims = [Image.open(f) for f in files[s:s + 8]]
        w, h = ims[0].size
        sheet = Image.new("RGB", (w * 4 + 15, h * 2 + 5), "gray")
        for k, im in enumerate(ims):
            sheet.paste(im, ((k % 4) * (w + 5), (k // 4) * (h + 5)))
        sheet.save(OUT / f"sheet_{s // 8}.png")
    print(len(files), "pages")


if __name__ == "__main__":
    main()
