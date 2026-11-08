"""Automated text-geometry checks on a PDF.
Usage: textcheck.py <pdf> <min_pt> [scale]
Flags: spans outside the page box, spans below min_pt (after scale), overlapping lines from different blocks,
and images that extend past the page or are drawn with a distorted aspect ratio."""
import sys
import pymupdf
sys.stdout.reconfigure(encoding="utf-8")

pdf, min_pt = sys.argv[1], float(sys.argv[2])
scale = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
doc = pymupdf.open(pdf)
small, outside, overlaps, imgs = [], [], [], []
for pno, page in enumerate(doc, 1):
    pr = page.rect
    d = page.get_text("dict")
    lines = []
    for bi, b in enumerate(d["blocks"]):
        if b["type"] != 0:
            continue
        for li, l in enumerate(b["lines"]):
            txt = "".join(s["text"] for s in l["spans"]).strip()
            if not txt:
                continue
            bb = pymupdf.Rect(l["bbox"])
            lines.append((bi, li, bb, txt, l["spans"]))
            for s in l["spans"]:
                t = s["text"].strip()
                if not t:
                    continue
                sz = s["size"] * scale
                if sz < min_pt:
                    small.append((pno, round(sz, 1), t[:60]))
                r = pymupdf.Rect(s["bbox"])
                if r.x0 < pr.x0 - 1 or r.y0 < pr.y0 - 1 or r.x1 > pr.x1 + 1 or r.y1 > pr.y1 + 1:
                    outside.append((pno, tuple(round(v) for v in r), t[:60]))
    for i in range(len(lines)):
        for j in range(i + 1, len(lines)):
            a, b = lines[i], lines[j]
            if a[0] == b[0]:
                continue
            inter = a[2] & b[2]
            if inter.is_empty:
                continue
            area = inter.width * inter.height
            mn = min(a[2].width * a[2].height, b[2].width * b[2].height)
            if mn > 0 and area / mn > 0.25:
                overlaps.append((pno, round(area / mn, 2), a[3][:40], b[3][:40]))
    for info in page.get_image_info(xrefs=True):
        r = pymupdf.Rect(info["bbox"])
        w, h = info["width"], info["height"]
        if r.width > 0 and r.height > 0 and w and h:
            ar_draw = r.width / r.height
            ar_src = w / h
            dist = ar_draw / ar_src
            # rotation-free transform check
            if abs(dist - 1) > 0.03:
                imgs.append((pno, "aspect", round(dist, 3), w, h, tuple(round(v) for v in r)))
        if r.x0 < pr.x0 - 1 or r.y0 < pr.y0 - 1 or r.x1 > pr.x1 + 1 or r.y1 > pr.y1 + 1:
            imgs.append((pno, "outside", w, h, tuple(round(v) for v in r)))
print("pages", doc.page_count)
print("SMALL (<%.1f pt):" % min_pt, len(small))
for x in small:
    print("  ", x)
print("OUTSIDE:", len(outside))
for x in outside:
    print("  ", x)
print("OVERLAPS:", len(overlaps))
for x in overlaps:
    print("  ", x)
print("IMAGES:", len(imgs))
for x in imgs:
    print("  ", x)
