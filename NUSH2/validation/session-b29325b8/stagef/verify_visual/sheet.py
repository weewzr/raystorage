import sys, zipfile, io
from PIL import Image, ImageDraw
z = zipfile.ZipFile(sys.argv[1]); out = sys.argv[2]
names = [n for n in z.namelist() if n.startswith("ppt/media/qa_image")]
names.sort(key=lambda n: int(''.join(c for c in n.split('/')[-1] if c.isdigit())))
tiles = []
for n in names:
    im = Image.open(io.BytesIO(z.read(n))).convert("RGB"); im.thumbnail((380, 380))
    t = Image.new("RGB", (390, 400), "white"); t.paste(im, (5, 15)); ImageDraw.Draw(t).text((5, 1), n.split('/')[-1], fill=(255, 0, 0)); tiles.append(t)
cols = 4; rows = (len(tiles) + cols - 1) // cols
S = Image.new("RGB", (cols * 390, rows * 400), "white")
for i, t in enumerate(tiles): S.paste(t, ((i % cols) * 390, (i // cols) * 400))
S.save(out); print(len(tiles))
