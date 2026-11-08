"""Find pages by text and render chosen pages of the derivations PDF to PNG."""
import sys
import pymupdf

pdf, out = sys.argv[1], sys.argv[2]
doc = pymupdf.open(pdf)
print("pages", doc.page_count)
if sys.argv[3] == "find":
    for needle in sys.argv[4:]:
        hits = [i + 1 for i, p in enumerate(doc) if needle in p.get_text()]
        print(f"{needle!r}: {hits[:12]}")
else:
    for n in sys.argv[4:]:
        page = doc[int(n) - 1]
        pix = page.get_pixmap(dpi=int(sys.argv[3]))
        path = f"{out}/p{int(n):03d}.png"
        pix.save(path)
        print(path)
