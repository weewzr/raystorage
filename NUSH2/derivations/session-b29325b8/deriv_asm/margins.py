"""Report text spans that cross the page margins, and pages with unresolved markers."""
import sys
import pymupdf

doc = pymupdf.open(sys.argv[1])
left, right = 56.7 - 2.0, 595.3 - 56.7 + 2.0
bad = []
markers = []
for i, page in enumerate(doc):
    d = page.get_text("dict")
    for b in d["blocks"]:
        for line in b.get("lines", []):
            for s in line["spans"]:
                x0, y0, x1, y1 = s["bbox"]
                if (x1 > right or x0 < left) and s["text"].strip():
                    bad.append((i + 1, round(x0, 1), round(x1, 1), s["text"][:50]))
    text = page.get_text()
    for m in ("??", "[?]", "pending", "next build pass"):
        if m in text:
            markers.append((i + 1, m))
print("pages", doc.page_count)
print("spans outside margins:", len(bad))
for row in bad[:30]:
    print(row)
print("markers:", markers[:20])
