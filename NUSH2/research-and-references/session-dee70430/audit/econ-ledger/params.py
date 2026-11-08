import re, sys, json
def load(path):
    ps=[];cur=None
    for ln,line in enumerate(open(path,encoding='utf-8'),1):
        line=line.rstrip('\n')
        if line.strip()=='[[param]]':
            cur={'_line':ln};ps.append(cur);continue
        m=re.match(r'^(\w+)\s*=\s*(.*)$',line)
        if m and cur is not None:
            k,v=m.groups()
            v=v.strip()
            if v.startswith('"') and v.endswith('"'): v=v[1:-1]
            else:
                try: v=float(v)
                except: pass
            cur[k]=v
    return ps
if __name__=='__main__':
    ps=load(sys.argv[1])
    pat=re.compile(sys.argv[2])
    w=int(sys.argv[3]) if len(sys.argv)>3 else 220
    for p in ps:
        if pat.search(p['id']):
            print(f"L{p['_line']} {p['id']}={p.get('value')} {p.get('unit')} lo={p.get('low')} hi={p.get('high')} [{p.get('class')}/{p.get('source_status')}] {str(p.get('source'))[:w]} @ {p.get('locator')} dec={p.get('decision')}")
