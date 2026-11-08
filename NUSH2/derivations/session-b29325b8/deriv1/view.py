"""Render pages of the scratch build. Usage: view.py [page ...]"""
import re
import sys
import pymupdf as fitz

B = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/deriv1/"
d = fitz.open(B + "build/NUSH2_First_Principles_Derivations.pdf")
print("pages", d.page_count)
heads = {}
for i, page in enumerate(d):
    text = page.get_text()
    for key in ("Table of variables", "Numbered equation list", "Assumptions and parameter ranges", "Variables\n",
                "A. Conservation", "B. HTGR", "C. Intermediate", "D. Steam", "E. Water", "References"):
        if key in text and key not in heads and i > 1:
            heads[key] = i + 1
print(heads)
for n in sys.argv[1:]:
    p = int(n)
    fn = B + f"img/out_{p}.png"
    d[p - 1].get_pixmap(dpi=80).save(fn)
    print(fn)
