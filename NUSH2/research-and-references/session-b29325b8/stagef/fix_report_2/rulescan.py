"""Small script spans (sub and superscripts) whose baseline or descender sits within 1 pt of a
horizontal rule below them, per page. Also reports the blank space under the last text on each page."""
import sys, pymupdf
pdf = pymupdf.open(sys.argv[1])
first, last = int(sys.argv[2]), int(sys.argv[3])
tot = 0
for pno in range(first, last + 1):
    page = pdf[pno - 1]
    rules = []
    for d in page.get_drawings():
        for it in d["items"]:
            if it[0] == "l":
                p, q = it[1], it[2]
                if abs(p.y - q.y) < 0.5 and abs(p.x - q.x) > 20:
                    rules.append((min(p.x, q.x), max(p.x, q.x), p.y))
            elif it[0] == "re":
                r = it[1]
                if r.height < 1.6 and r.width > 20:
                    rules.append((r.x0, r.x1, r.y0))
    hits = []
    for b in page.get_text("rawdict")["blocks"]:
        for l in b.get("lines", []):
            for s in l["spans"]:
                if s["size"] > 10.5 or not s["chars"]:
                    continue
                for ch in s["chars"]:
                    c = ch["c"]
                    if c.isspace():
                        continue
                    x0, y0, x1, y1 = ch["bbox"]
                    base = ch["origin"][1]
                    ink = base + (0.22 * s["size"] if c in "gjpqy" else 0.0)
                    for rx0, rx1, ry in rules:
                        if x1 > rx0 and x0 < rx1 and -0.5 < ry - ink < 1.0:
                            hits.append((c, round(ry - ink, 2), round(ry, 1)))
    blocks = [b for b in page.get_text("blocks") if b[3] < page.rect.height - 50]
    bottom = max((b[3] for b in blocks), default=0)
    print(f"p{pno}: {len(hits)} script glyphs within 1 pt of a rule {hits[:6]}; last text at y={bottom:.0f}")
    tot += len(hits)
print("total", tot)
