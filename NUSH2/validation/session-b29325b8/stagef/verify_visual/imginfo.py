import sys
import pymupdf
sys.stdout.reconfigure(encoding="utf-8")
doc = pymupdf.open(sys.argv[1]); scale = float(sys.argv[3])
for pno in [int(x) for x in sys.argv[2].split(",")]:
    page = doc[pno - 1]
    print("page", pno)
    for info in page.get_image_info(xrefs=True):
        r = pymupdf.Rect(info["bbox"]) * (1 / scale)
        t = info["transform"]
        print("   xref", info["xref"], "px", info["width"], "x", info["height"], "bbox/scale", tuple(round(v, 1) for v in r), "transform", tuple(round(v, 1) for v in t))
