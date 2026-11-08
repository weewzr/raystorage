import pypdf,sys,re,io
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
f=sys.argv[1]; pats=sys.argv[2:]
r=pypdf.PdfReader(f); n=len(r.pages); print('pages',n)
hits={p:[] for p in pats}
for i in range(n):
    try: t=r.pages[i].extract_text() or ''
    except Exception: continue
    t=re.sub(r'\s+',' ',t)
    if i<2: print('p',i+1,':',t[:260])
    for p in pats:
        for m in list(re.finditer(p,t,re.I))[:2]:
            hits[p].append((i+1,t[max(0,m.start()-100):m.end()+140]))
for p,v in hits.items():
    print('==',p,'pages:',sorted(set(a for a,b in v))[:30])
    for a,b in v[:5]: print('   p',a,':',b)
