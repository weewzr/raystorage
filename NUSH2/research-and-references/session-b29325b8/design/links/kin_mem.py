# In-memory PDF keyword scan: nothing is written to disk.
import urllib.request,io,sys,re
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
import pypdf
UA={'User-Agent':'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36','Accept':'*/*'}
u=sys.argv[1]; pats=sys.argv[2:]
data=urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=240).read()
print('bytes',len(data))
r=pypdf.PdfReader(io.BytesIO(data)); n=len(r.pages); print('pages',n)
hits={p:[] for p in pats}
for i in range(n):
    try: t=r.pages[i].extract_text() or ''
    except Exception: continue
    t=re.sub(r'\s+',' ',t)
    if i<3: print('p',i+1,':',t[:220])
    for p in pats:
        for m in list(re.finditer(p,t,re.I))[:2]:
            hits[p].append((i+1,t[max(0,m.start()-90):m.end()+130]))
for p,v in hits.items():
    print('==',p,'pages:',sorted(set(a for a,b in v))[:40])
    for a,b in v[:6]: print('   p',a,':',b)
