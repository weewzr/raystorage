import fitz, sys, os
W = sys.argv[1]
jobs = [("NUSH2_Final_Report.pdf","rep",110),("NUSH2_Final_Presentation.pdf","deck",None),("qa_src.pdf","qasrc",None),("NUSH2_Speaker_Script.pdf","script",100),("NUSH2_QA_Prep.pdf","qaprep",100)]
for fn, tag, dpi in jobs:
    d = fitz.open(os.path.join(W, fn))
    od = os.path.join(W, "png_"+tag); os.makedirs(od, exist_ok=True)
    sizes=set()
    for i, p in enumerate(d):
        sizes.add((round(p.rect.width,1), round(p.rect.height,1)))
        if dpi is None:
            z = 1300/p.rect.width
            pix = p.get_pixmap(matrix=fitz.Matrix(z,z))
        else:
            pix = p.get_pixmap(dpi=dpi)
        pix.save(os.path.join(od, f"{tag}_{i+1:02d}.png"))
    print(fn, d.page_count, sizes, d.metadata)
