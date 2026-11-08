"""Search PDFs for regex patterns. Usage: search.py out.txt pattern pdf_or_dir [pdf_or_dir ...]"""
import os
import re
import sys
import fitz

out, pat = sys.argv[1], re.compile(sys.argv[2], re.I)
paths = []
for p in sys.argv[3:]:
    if os.path.isdir(p):
        for root, _, files in os.walk(p):
            for f in files:
                if f.lower().endswith(".pdf"):
                    paths.append(os.path.join(root, f))
    else:
        paths.append(p)
with open(out, "w", encoding="utf-8") as fo:
    for path in sorted(set(paths)):
        try:
            d = fitz.open(path)
        except Exception as e:
            fo.write(f"!! {path}: {e}\n")
            continue
        for i, pg in enumerate(d):
            t = " ".join(pg.get_text().split())
            for m in pat.finditer(t):
                a, b = max(0, m.start() - 160), min(len(t), m.end() + 160)
                fo.write(f"{os.path.basename(path)} p{i + 1}: ...{t[a:b]}...\n")
