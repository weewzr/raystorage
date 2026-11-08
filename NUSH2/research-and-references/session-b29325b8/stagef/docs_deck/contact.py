import sys
from pathlib import Path
from PIL import Image, ImageDraw
files = [Path(a) for a in sys.argv[2:]]
cols = 4
cell = 480
rows = (len(files) + cols - 1) // cols
sheet = Image.new("RGB", (cols * cell, rows * (cell + 20)), "white")
for i, f in enumerate(files):
    im = Image.open(f).convert("RGB")
    print(f.name, im.size)
    im.thumbnail((cell - 10, cell - 10))
    x, y = (i % cols) * cell, (i // cols) * (cell + 20)
    sheet.paste(im, (x + 5, y + 20))
    ImageDraw.Draw(sheet).text((x + 5, y + 4), f.name, fill=(200, 0, 0))
sheet.save(sys.argv[1])
