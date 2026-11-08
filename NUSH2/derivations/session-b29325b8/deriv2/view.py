import sys
import pymupdf as fitz
B="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/deriv2/"
d=fitz.open(B+"build/out/NUSH2_First_Principles_Derivations.pdf")
for n in sys.argv[1:]:
    p=int(n); fn=B+f"img/out_{p}.png"
    d[p-1].get_pixmap(dpi=85).save(fn); print(fn)
