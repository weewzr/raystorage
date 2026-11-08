import sys, pymupdf
pdf = pymupdf.open(sys.argv[1]); out = sys.argv[2]; dpi = int(sys.argv[3]) if len(sys.argv) > 3 else 80
pages = range(len(pdf)) if len(sys.argv) <= 4 else [int(x) - 1 for x in sys.argv[4].split(",")]
for i in pages:
    pdf[i].get_pixmap(dpi=dpi).save(f"{out}/p{i+1:02d}.png")
print(len(pdf), "pages")
