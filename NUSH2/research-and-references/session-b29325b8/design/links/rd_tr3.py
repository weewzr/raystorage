import sys,re
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import pymupdf as fitz
base='C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/'
d=fitz.open(base+'webfetch-1793584472632-61axzz.pdf')
t=d[7].get_text(); i=t.find('参考'); print(t[max(0,i-100):i+1800] if i>=0 else t[-1500:])
print('kPa hits across paper:')
for n,p in enumerate(d):
    for m in re.finditer(r'[^\n]*(kPa|圧力損失)[^\n]*',p.get_text()): print(n+1,m.group(0)[:120])
d2=fitz.open(base+'webfetch-1793584486118-p4rp5g.pdf')
print('Fujikawa IHX / circulator mentions:')
for n,p in enumerate(d2):
    t=p.get_text()
    for m in re.finditer(r'[^\n]*(循環機|中間熱交換器|IHX|差圧|圧損|kPa)[^\n]*',t): print(n+1,m.group(0)[:110])
