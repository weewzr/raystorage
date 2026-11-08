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
t=d[6].get_text(); print('--- HPLPB p7 tail'); print(t[-1500:])
t=d[7].get_text(); i=t.find('voltage'); print('--- HPLPB p8 around table', i); print(t[max(0,i-300):i+700] if i>=0 else '')
print('--- p7 table search')
for pg in (5,6,7):
    for w in d[pg].get_text().split('\n'):
        if 'Table' in w or '表' in w[:3]: print(pg+1,w[:120])
b=get("https://downloads.unido.org/ot/47/91/4791816/10001-15000_11046.pdf")
d=fitz.open(stream=b,filetype="pdf")
for pg in (2,3,4,5):
    print('--- UNIDO p',pg+1); print(d[pg].get_text()[:900])
print('--- UNIDO last pages')
print(d[13].get_text()[:500]); print(d[len(d)-1].get_text()[:300])
