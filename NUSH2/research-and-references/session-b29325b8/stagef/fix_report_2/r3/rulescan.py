"""Glyphs whose ink comes within CLEAR pt of a horizontal table rule, per page. Ink is estimated
from each glyph's baseline: the bottom is the baseline, or 0.22 em below it for a descender; the top
is 0.68 em above it for digits, capitals and ascenders and 0.46 em for other lowercase letters.
Checks rules below the glyph (subscripts, descenders) and rules above it (superscripts, capitals)."""
import sys, pymupdf
pdf = pymupdf.open(sys.argv[1])
CLEAR = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
DESC, ASC = set("gjpqy"), set("0123456789bdfhklt()[]/ABCDEFGHIJKLMNOPQRSTUVWXYZ")
tot = 0
for pno, page in enumerate(pdf, 1):
    rules = []
    for d in page.get_drawings():
        for it in d["items"]:
            if it[0] == "l":
                p, q = it[1], it[2]
                if abs(p.y - q.y) < 0.5 and abs(p.x - q.x) > 20:
                    rules.append((min(p.x, q.x), max(p.x, q.x), p.y, p.y))
            elif it[0] == "re":
                r = it[1]
                if r.height < 1.6 and r.width > 20:
                    rules.append((r.x0, r.x1, r.y0, r.y1))
    if not rules:
        continue
    hits = []
    for b in page.get_text("rawdict")["blocks"]:
        for l in b.get("lines", []):
            for s in l["spans"]:
                size = s["size"]
                for ch in s["chars"]:
                    c = ch["c"]
                    if c.isspace() or c in "−-–_.,":
                        continue
                    x0, _, x1, _ = ch["bbox"]
                    base = ch["origin"][1]
                    bot = base + (0.22 * size if c in DESC else 0.0)
                    top = base - (0.68 if c in ASC else 0.46) * size
                    for rx0, rx1, ry0, ry1 in rules:
                        if not (x1 > rx0 + 0.5 and x0 < rx1 - 0.5):
                            continue
                        below = ry0 - bot   # rule under the glyph
                        above = top - ry1   # rule over the glyph
                        if -2.0 < below < CLEAR:
                            hits.append(("below", c, round(size, 1), round(below, 2), round(ry0, 1)))
                        elif -2.0 < above < CLEAR:
                            hits.append(("above", c, round(size, 1), round(above, 2), round(ry0, 1)))
    if hits:
        small = [h for h in hits if h[2] < 10.5]
        print(f"p{pno}: {len(hits)} glyphs within {CLEAR} pt of a rule, {len(small)} of them scripts: {hits[:8]}")
    tot += len(hits)
print("total", tot)
