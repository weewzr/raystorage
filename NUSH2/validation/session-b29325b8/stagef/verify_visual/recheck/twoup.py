import pymupdf as fitz, sys, os
W = sys.argv[1]
d = fitz.open(os.path.join(W, "NUSH2_Final_Report.pdf"))
od = os.path.join(W, "rep2up"); os.makedirs(od, exist_ok=True)
H = 1500
for k in range(0, d.page_count, 2):
    pix = []
    for p in d.pages(k, min(k+2, d.page_count)):
        z = H / p.rect.height
        pix.append(p.get_pixmap(matrix=fitz.Matrix(z, z)))
    w = sum(x.width for x in pix) + 20 * (len(pix)-1)
    out = fitz.Pixmap(fitz.csRGB, fitz.IRect(0, 0, w, H), False); out.clear_with(160)
    x = 0
    for px in pix:
        px.set_origin(x, 0); out.copy(px, px.irect); x += px.width + 20
    out.save(os.path.join(od, f"rep_{k+1:02d}_{k+2:02d}.png"))
print("done")
