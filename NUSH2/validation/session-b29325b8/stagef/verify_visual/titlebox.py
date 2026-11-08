import sys, pymupdf
sys.stdout.reconfigure(encoding="utf-8")
d = pymupdf.open(sys.argv[1]); off = int(sys.argv[2])
for pno in range(off, off + 16):
    p = d[pno - 1]
    spans = [s for b in p.get_text("dict")["blocks"] if b["type"] == 0 for l in b["lines"] for s in l["spans"] if s["text"].strip()]
    qa = [s for s in spans if s["text"].strip() == "Q&A"]
    big = max(spans, key=lambda s: s["size"])
    tl = [s for s in spans if abs(s["size"] - big["size"]) < 0.5 and s["bbox"][1] < 130 * (p.rect.width / 960)]
    x1 = max(s["bbox"][2] for s in tl); y1 = max(s["bbox"][3] for s in tl)
    qx = qa[0]["bbox"][0] if qa else None
    # vertical divider: drawings near qx
    lines = [dr for dr in p.get_drawings() if dr["rect"].width < 4 and dr["rect"].height > 20 and dr["rect"].y0 < 100 * (p.rect.width / 960)]
    dx = min((dr["rect"].x0 for dr in lines), default=None)
    print(pno, "title size", round(big["size"], 1), "lines", len({round(s['bbox'][1]) for s in tl}), "title right", round(x1, 1), "divider x", round(dx, 1) if dx else None, "gap", round(dx - x1, 1) if dx else None, "| Q&A x", round(qx, 1) if qx else None)
