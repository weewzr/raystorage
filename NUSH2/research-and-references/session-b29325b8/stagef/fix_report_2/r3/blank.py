"""Per page: top and bottom of body content (text, images, drawings) above the footer, against the
body area of a 20 mm margin page. Prints blank height at the bottom and the largest inner gap."""
import sys, pymupdf
pdf = pymupdf.open(sys.argv[1])
MARGIN = 20 / 25.4 * 72
for i, page in enumerate(pdf):
    W, H = page.rect.width, page.rect.height
    top, bottom = MARGIN, H - MARGIN
    boxes = []
    for b in page.get_text("dict")["blocks"]:
        r = pymupdf.Rect(b["bbox"])
        if r.y0 > bottom - 2:  # footer
            continue
        if b["type"] == 0 and not "".join(s["text"] for l in b["lines"] for s in l["spans"]).strip():
            continue
        boxes.append(r)
    for d in page.get_drawings():
        r = d["rect"]
        if r.y0 > bottom - 2 or r.height > H * 0.9:
            continue
        boxes.append(r)
    if not boxes:
        print(f"p{i+1}: empty"); continue
    ys = sorted((r.y0, r.y1) for r in boxes)
    cmax, gap, at = ys[0][1], 0, None
    for y0, y1 in ys[1:]:
        if y0 - cmax > gap:
            gap, at = y0 - cmax, cmax
        cmax = max(cmax, y1)
    end = max(r.y1 for r in boxes)
    body = bottom - top
    print(f"p{i+1} {'L' if W > H else 'P'}: content {min(r.y0 for r in boxes):.0f}-{end:.0f}, blank at bottom {bottom-end:.0f} pt ({100*(bottom-end)/body:.0f}%), largest inner gap {gap:.0f} pt at y={at if at is None else round(at)}")
