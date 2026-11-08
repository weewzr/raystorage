import sys, fitz
pdf, out, dpi = sys.argv[1], sys.argv[2], int(sys.argv[3])
doc = fitz.open(pdf)
for p in sys.argv[4:]:
    i = int(p)
    pix = doc[i-1].get_pixmap(dpi=dpi)
    pix.save(f"{out}_p{i:03d}.png")
print(len(doc))
