import sys
from PIL import Image
out, files = sys.argv[1], sys.argv[2:]
ims = [Image.open(f).convert("RGB") for f in files]
h = max(i.height for i in ims); w = sum(i.width for i in ims) + 10 * (len(ims) - 1)
s = Image.new("RGB", (w, h), "white"); x = 0
for i in ims:
    s.paste(i, (x, 0)); x += i.width + 10
s.save(out)
