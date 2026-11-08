import pymupdf as fitz, sys, os
W = sys.argv[1]
src = fitz.open(os.path.join(W, "qa_src.pdf")); dk = fitz.open(os.path.join(W, "NUSH2_Final_Presentation.pdf"))
od = os.path.join(W, "pairs"); os.makedirs(od, exist_ok=True)
for i in range(16):
    a = src[i]; b = dk[56 + i]
    wa = 900
    pa = a.get_pixmap(matrix=fitz.Matrix(wa / a.rect.width, wa / a.rect.width))
    pb = b.get_pixmap(matrix=fitz.Matrix(wa / b.rect.width, wa / b.rect.width))
    h = max(pa.height, pb.height)
    out = fitz.Pixmap(fitz.csRGB, fitz.IRect(0, 0, wa * 2 + 20, h), False)
    out.clear_with(200)
    pa.set_origin(0, 0); pb.set_origin(wa + 20, 0)
    out.copy(pa, pa.irect); out.copy(pb, pb.irect)
    out.save(os.path.join(od, f"pair_{i+1:02d}.png"))
print("ok")
