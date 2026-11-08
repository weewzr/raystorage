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
print('--- NP-T-4.2 p211'); print(d[210].get_text()[:2600])
print('--- p210 tail'); print(d[209].get_text()[-900:])
print('printed page guess'); print(d[210].get_text()[-60:])
b=get("https://publications.ieaghg.org/technicalreports/2017-TR3%20Reference%20data%20and%20Supporting%20Literature%20Reviews%20for%20SMR%20Based%20Hydrogen%20Production%20with%20CCS.pdf")
d=fitz.open(stream=b,filetype="pdf")
print('--- TR3 p1'); print(d[0].get_text()[:300])
print('--- TR3 p79'); t=d[78].get_text(); i=t.find('From an energy'); print(t[max(0,i-1400):i+700])
print('--- TR3 p96'); t=d[95].get_text(); print(t[:1300])
print('--- TR3 p124'); t=d[123].get_text(); i=t.find('amine reboiler'); print(t[max(0,i-900):i+500])
