import sys, fitz, os
# usage: render.py pdf outprefix dpi pages...
pdf, prefix, dpi = sys.argv[1], sys.argv[2], int(sys.argv[3])
doc = fitz.open(pdf)
for p in sys.argv[4:]:
    n = int(p)
    pix = doc[n-1].get_pixmap(dpi=dpi)
    out = f"{prefix}_p{n:03d}.png"
    pix.save(out)
    print(out, pix.width, pix.height)
