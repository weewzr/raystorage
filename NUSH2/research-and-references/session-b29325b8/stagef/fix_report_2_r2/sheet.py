import sys
from PIL import Image
files = sys.argv[2:]; ims = [Image.open(f) for f in files]
h = max(i.size[1] for i in ims); w = sum(i.size[0] for i in ims) + 10 * (len(ims) - 1)
s = Image.new("RGB", (w, h), "#888"); x = 0
for i in ims: s.paste(i, (x, 0)); x += i.size[0] + 10
s.save(sys.argv[1])
