import sys; sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import sys,re
import pymupdf as fitz
base='C:/Users/Alf/.claude/projects/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/tool-results/'
for name,pat in [('webfetch-1793584472632-61axzz.pdf',r'1448|1,448|kPa|MPa|m2|m²|伝熱面積|圧力損失|圧損'),
                 ('webfetch-1793584486118-p4rp5g.pdf',r'irculator|循環機|kPa|圧力損失'),
                 ('webfetch-1793584472356-p9hj30.pdf',r'kPa|MPa|ressure drop|irculator|form loss|head'),
                 ('webfetch-1793584472704-4ya78t.pdf',r'kPa|ressure drop|irculator|form loss')]:
    d=fitz.open(base+name)
    print('=====',name,'pages',len(d),d.metadata.get('title'))
    n=0
    print((d[0].get_text() or '')[:300].replace('\n',' | '))
    for i,p in enumerate(d):
        t=p.get_text()
        for line in t.split('\n'):
            if re.search(pat,line):
                print(i+1,line[:170]); n+=1
                if n>60: break
        if n>60: break
