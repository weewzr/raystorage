# Reads a PDF from stdin into memory, nothing is written to disk.
import pypdf,sys,re,io
data=sys.stdin.buffer.read()
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
print('bytes',len(data),data[:8])
r=pypdf.PdfReader(io.BytesIO(data)); n=len(r.pages); print('pages',n)
pats=sys.argv[1:]
hits={p:[] for p in pats}
for i in range(n):
    try: t=r.pages[i].extract_text() or ''
    except Exception: continue
    t=re.sub(r'\s+',' ',t)
    if i<2: print('p',i+1,':',t[:230])
    for p in pats:
        for m in list(re.finditer(p,t,re.I))[:2]:
            hits[p].append((i+1,t[max(0,m.start()-100):m.end()+170]))
for p,v in hits.items():
    print('==',p,'pages:',sorted(set(a for a,b in v))[:30])
    for a,b in v[:5]: print('   p',a,':',b)
