import sys, glob, re
from PIL import Image, ExifTags
for f in sys.argv[1:]:
    for p in sorted(glob.glob(f)):
        im = Image.open(p)
        info = {k: (v if isinstance(v, (int, float, str, tuple)) else type(v).__name__) for k, v in im.info.items() if k not in ("dpi", "jfif", "jfif_version", "jfif_unit", "jfif_density", "gamma", "progressive", "progression", "adobe", "adobe_transform", "srgb")}
        ex = im.getexif()
        tags = {ExifTags.TAGS.get(k, k): (str(v)[:60]) for k, v in ex.items()}
        gps = ex.get_ifd(0x8825) if ex else {}
        raw = open(p, "rb").read()
        xmp = b"<x:xmpmeta" in raw or b"http://ns.adobe.com/xap" in raw
        if tags or gps or info or xmp:
            print(p.split("/")[-1].split("\\")[-1], "| info", {k: str(v)[:80] for k, v in info.items()}, "| exif", tags, "| gps", dict(gps), "| xmp", xmp)
print("checked")
