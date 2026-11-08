import sys,ssl,urllib.request,re
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import pymupdf as fitz
UA="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
ctx=ssl._create_unverified_context()
def get(u):
    r=urllib.request.Request(u,headers={"User-Agent":UA})
    return urllib.request.urlopen(r,timeout=200,context=ctx).read()
b=get("https://www-pub.iaea.org/MTCD/Publications/PDF/Pub1577_web.pdf")
d=fitz.open(stream=b,filetype="pdf")
t=d[210].get_text(); i=t.find('Maximum pressure'); print(t[i:])
print('--- tables on the page'); 
try:
    for tb in d[210].find_tables().tables:
        for row in tb.extract(): print(row)
except Exception as e: print(e)
print('--- reference [331]')
for n in range(380,len(d)):
    x=d[n].get_text()
    for m in re.finditer(r'\[33[0-3]\][^\[]{0,220}',x): print(n+1,m.group(0).replace('\n',' '))
print(d[0].get_text()[:200]); print(d[2].get_text()[:300])
