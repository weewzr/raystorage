"""Side-by-side: source slide i (left) and deck slide 56+i (right)."""
import sys
from pathlib import Path
from PIL import Image, ImageDraw
V = Path(sys.argv[1])
for i in range(1, 17):
    a = Image.open(V / "qa_png" / f"p{i:02d}.png").convert("RGB")
    b = Image.open(V / "deck_png" / f"p{56 + i:02d}.png").convert("RGB")
    w = 960
    a = a.resize((w, int(a.height * w / a.width)))
    b = b.resize((w, int(b.height * w / b.width)))
    s = Image.new("RGB", (2 * w + 10, max(a.height, b.height)), (255, 0, 0))
    s.paste(a, (0, 0)); s.paste(b, (w + 10, 0))
    d = ImageDraw.Draw(s)
    d.text((4, 2), f"SRC {i}", fill=(255, 0, 0)); d.text((w + 14, 2), f"DECK {56 + i}", fill=(255, 0, 0))
    s.save(V / f"sbs_{i:02d}.png")
print("ok")
