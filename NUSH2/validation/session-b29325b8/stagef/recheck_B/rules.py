"""Glyph-to-rule scan: small-font glyphs (sub/superscripts) whose ink box comes within GAP pt of a
horizontal table rule. Uses char bboxes from rawdict (font-size based) shrunk to the glyph's
ascender/descender, plus the drawn horizontal lines and thin rects."""
import sys
import pymupdf

path = sys.argv[1]
GAP = float(sys.argv[2]) if len(sys.argv) > 2 else 0.8
doc = pymupdf.open(path)
total = 0
for pno, page in enumerate(doc, 1):
    rules = []
    for d in page.get_drawings():
        for it in d["items"]:
            if it[0] == "l":
                a, b = it[1], it[2]
                if abs(a.y - b.y) < 0.3 and abs(a.x - b.x) > 20:
                    rules.append((min(a.x, b.x), max(a.x, b.x), a.y))
            elif it[0] == "re":
                r = it[1]
                if r.height < 1.6 and r.width > 20:
                    rules.append((r.x0, r.x1, (r.y0 + r.y1) / 2))
    if not rules:
        continue
    raw = page.get_text("rawdict")
    hits = []
    for b in raw["blocks"]:
        for l in b.get("lines", []):
            for s in l["spans"]:
                if s["size"] > 9.5:
                    continue
                for ch in s["chars"]:
                    c = ch["c"]
                    if c.isspace():
                        continue
                    x0, y0, x1, y1 = ch["bbox"]
                    oy = ch["origin"][1]
                    # ink extent: baseline +/- typical ascent/descent of the glyph
                    desc = 0.22 * s["size"] if c in "gjpqy" else 0.0
                    ink_bot = oy + desc
                    ink_top = oy - 0.72 * s["size"]
                    for rx0, rx1, ry in rules:
                        if rx0 - 1 <= (x0 + x1) / 2 <= rx1 + 1:
                            gap = min(abs(ry - ink_bot), abs(ink_top - ry))
                            if (ink_top - GAP) <= ry <= (ink_bot + GAP):
                                hits.append((round(ry, 1), c, round(s["size"], 1), round(gap, 2), round(x0)))
    if hits:
        total += len(hits)
        print(f"p{pno}: {len(hits)} {hits[:8]}")
print("total", total)
