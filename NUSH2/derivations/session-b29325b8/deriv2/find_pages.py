"""Find PDF pages holding given card titles and render them. Usage: find_pages.py pdf outdir phrase..."""
import sys
import pymupdf as fitz

pdf, out = sys.argv[1], sys.argv[2]
doc = fitz.open(pdf)
for phrase in sys.argv[3:]:
    for i in range(doc.page_count):
        if phrase in doc[i].get_text():
            fn = f"{out}/p{i + 1}.png"
            doc[i].get_pixmap(dpi=80).save(fn)
            print(phrase, "->", i + 1, fn)
            break
    else:
        print(phrase, "-> not found")
