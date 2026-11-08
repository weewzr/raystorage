import sys
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import pymupdf as fitz
base='C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/'
d=fitz.open(base+'webfetch-1793584472632-61axzz.pdf')
for i in [1,2,3]:
    print('--- Kato p',i+1); print(d[i].get_text()[:3600])
d=fitz.open(base+'webfetch-1793584486118-p4rp5g.pdf')
print('--- Fuj p8'); t=d[7].get_text(); print(len(t)); print(t[:3500])
