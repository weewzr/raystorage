import sys, os
import pymupdf
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.argv=['x','aronu','1']
src=open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'rp.py'),encoding='utf-8').read().split("key=sys.argv")[0]
exec(src)
for k,f in F.items():
    d=pymupdf.open(f)
    with open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'txt',k+'.txt'),'w',encoding='utf-8') as o:
        for i,p in enumerate(d):
            o.write(f"\n=====PDFPAGE {i+1}=====\n"); o.write(p.get_text())
    print(k,d.page_count)
