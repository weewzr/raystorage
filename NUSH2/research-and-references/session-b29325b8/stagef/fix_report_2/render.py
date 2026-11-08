import sys, pymupdf
pdf, out, pages, dpi = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]) if len(sys.argv) > 4 else 80
d = pymupdf.open(pdf)
sel = range(1, len(d) + 1) if pages == "all" else [int(x) for x in pages.split(",")]
for p in sel:
    d[p - 1].get_pixmap(dpi=dpi).save(f"{out}_p{p:02d}.png")
print(len(d), "pages")
