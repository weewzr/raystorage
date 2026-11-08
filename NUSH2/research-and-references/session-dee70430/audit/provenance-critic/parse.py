import re, json, sys, collections
P=[];cur=None
for n,line in enumerate(open('engine/data/parameters.toml',encoding='utf-8'),1):
    s=line.rstrip('\n')
    if s.strip().startswith('#') or not s.strip(): continue
    if s.strip()=='[[param]]':
        cur={'_line':n}; P.append(cur); continue
    m=re.match(r'^(\w+)\s*=\s*(.*)$',s)
    if not m: print('UNPARSED',n,s[:80]); continue
    k,v=m.groups()
    if v.startswith('"'): v=json.loads(v) if v.endswith('"') else v
    else:
        try: v=float(v)
        except: pass
    cur[k]=v
json.dump(P,open(sys.argv[1],'w'))
print(len(P))
for k in ['source_status','class','rigour','group']:
    print(k,collections.Counter(p.get(k) for p in P))
