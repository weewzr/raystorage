"""Read-only: print selected pages of the X-energy deck and render p16 for visual check."""
import sys
from pathlib import Path
import pymupdf
sys.stdout.reconfigure(encoding="utf8", errors="replace")
p = Path(r"C:\Users\Alf\.claude\projects\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tool-results\webfetch-1793576514245-5faaup.pdf")
OUT = Path(r"C:\Users\Alf\AppData\Local\Temp\claude\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\scratchpad\findings\render")
d = pymupdf.open(p)
for n in [14, 16, 37, 38]:
    print(f"\n##### X-energy deck p{n}\n", " ".join(d[n-1].get_text("text").split()))
d[15].get_pixmap(dpi=110).save(OUT / "audit_xenergy_p16.png")
d[36].get_pixmap(dpi=110).save(OUT / "audit_xenergy_p37.png")
for i, pg in enumerate(d):
    t = pg.get_text("text")
    for k in ["750", "200 MW", "200MW", "950", "helium", "Helium"]:
        if k in t:
            j = t.find(k)
            print(f"p{i+1} {k!r}: ", " ".join(t[max(0,j-60):j+80].split()))
