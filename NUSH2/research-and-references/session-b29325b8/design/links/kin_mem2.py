import urllib.request,io,sys,re
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
import pypdf
UA={'User-Agent':'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36','Accept':'*/*','Range':'bytes=0-60000000'}
u=sys.argv[1]; pages=[int(x) for x in sys.argv[2].split(',')] if sys.argv[2]!='-' else []; pats=sys.argv[3:]
data=urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=240).read()
print('bytes',len(data))
r=pypdf.PdfReader(io.BytesIO(data)); n=len(r.pages); print('pages',n)
for p in pages:
    print('--- PDF p',p); print(re.sub(r'\s+',' ',r.pages[p-1].extract_text() or '')[:1100])
hits={p:[] for p in pats}
for i in range(n):
    if not pats: break
    try: t=r.pages[i].extract_text() or ''
    except Exception: continue
    t=re.sub(r'\s+',' ',t)
    for p in pats:
        for m in list(re.finditer(p,t,re.I))[:2]:
            hits[p].append((i+1,t[max(0,m.start()-90):m.end()+130]))
for p,v in hits.items():
    print('==',p,'pages:',sorted(set(a for a,b in v))[:40])
    for a,b in v[:5]: print('   p',a,':',b)
