import sys, difflib
from pathlib import Path
import pymupdf
from PIL import Image, ImageChops, ImageStat
sys.stdout.reconfigure(encoding="utf-8")
V = Path(sys.argv[1])
src = pymupdf.open(V / "qa_src.pdf"); deck = pymupdf.open(V / "deck.pdf")
for i in range(16):
    a = Image.open(V / "qa_png" / f"p{i+1:02d}.png").convert("L").resize((480, 270))
    b = Image.open(V / "deck_png" / f"p{57+i:02d}.png").convert("L").resize((480, 270))
    d = ImageChops.difference(a, b)
    frac = sum(1 for p in d.getdata() if p > 40) / (480 * 270)
    ta = src[i].get_text().split(); tb = deck[56 + i].get_text().split()
    sm = difflib.SequenceMatcher(None, ta, tb)
    changes = [(op, " ".join(ta[i1:i2]), " ".join(tb[j1:j2])) for op, i1, i2, j1, j2 in sm.get_opcodes() if op != "equal"]
    print(f"Q&A {i+1:2d} -> deck {57+i}: pixel diff {frac:.3f}; text changes {len(changes)}")
    for c in changes[:12]:
        print("    ", c[0], "|", c[1][:90], "=>", c[2][:90])
