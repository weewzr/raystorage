import pymupdf as fitz, sys, hashlib, os
d = fitz.open(sys.argv[1])
media = sys.argv[2]
# hash map of media files
hm = {}
for fn in os.listdir(media):
    p = os.path.join(media, fn)
    try:
        pm = fitz.Pixmap(p)
        hm[(pm.width, pm.height)] = hm.get((pm.width, pm.height), []) + [fn]
    except Exception as e:
        pass
for i, p in enumerate(d):
    for info in p.get_image_info(xrefs=True):
        w, h = info["width"], info["height"]
        bb = fitz.Rect(info["bbox"])
        sx = bb.width / w * 300 / 72  # ratio of placed size to 300-dpi native size
        print(f"p{i+1} xref{info['xref']} px {w}x{h} placed {bb.width:.1f}x{bb.height:.1f}pt ({bb.width/72*25.4:.1f} mm) bbox y {bb.y0:.0f}-{bb.y1:.0f} scale_vs_300dpi {sx:.3f} aspect_px {w/h:.3f} aspect_box {bb.width/bb.height:.3f} match {hm.get((w,h))}")
