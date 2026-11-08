import sys
from PIL import Image
for f in sys.argv[1:]:
    im = Image.open(f)
    ex = im.getexif()
    info = {k: v for k, v in im.info.items() if k not in ("dpi", "jfif", "jfif_version", "jfif_unit", "jfif_density", "progressive", "progression", "gamma", "srgb")}
    print(f.split("/")[-1], im.format, im.size, "exif tags:", len(ex), dict(list(ex.items())[:6]) if ex else "", "info:", {k: (str(v)[:60]) for k, v in info.items()})
