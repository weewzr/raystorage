import sys,re
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import pymupdf as fitz
base='C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/'
d=fitz.open(base+'webfetch-1793584472632-61axzz.pdf')
t=d[0].get_text(); i=t.find('圧力損失'); print(t[max(0,i-260):i+160])
d=fitz.open(base+'webfetch-1793584472704-4ya78t.pdf')
for pg in (9,12,16): print('--- RELAP slide',pg+1); print(d[pg].get_text()[:700])
