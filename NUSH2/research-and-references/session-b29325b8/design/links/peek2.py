import sys,ssl,urllib.request
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
import pymupdf as fitz
UA="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
ctx=ssl._create_unverified_context()
def get(u):
    r=urllib.request.Request(u,headers={"User-Agent":UA})
    return urllib.request.urlopen(r,timeout=120,context=ctx).read()
b=get("https://www.hplpb.com.cn/en/article/pdf/preview/10.11884/HPLPB202436.230444.pdf")
d=fitz.open(stream=b,filetype="pdf")
print(d[0].get_text()[:900])
for pg in (3,5,7):
    t=d[pg].get_text()
    print('--- HPLPB page',pg+1)
    print(t[:2600] if pg!=7 else t[:3000])
b=get("https://downloads.unido.org/ot/47/91/4791816/10001-15000_11046.pdf")
d=fitz.open(stream=b,filetype="pdf")
print('--- UNIDO p1-3'); 
for pg in (0,1,2): print(d[pg].get_text()[:700])
print('--- UNIDO p7'); print(d[6].get_text()[:2200])
print('--- UNIDO p12'); print(d[11].get_text()[:3000])
print('--- UNIDO p13'); print(d[12].get_text()[:1200])
