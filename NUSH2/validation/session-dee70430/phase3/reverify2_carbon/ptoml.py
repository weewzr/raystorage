import re
def load(path):
    P={}; cur=None
    for line in open(path,encoding='utf-8'):
        line=line.rstrip('\n')
        if line.strip()=='[[param]]':
            cur={}; continue
        m=re.match(r'^(\w+)\s*=\s*(.*)$',line)
        if m and cur is not None:
            k,v=m.group(1),m.group(2).strip()
            if v.startswith('"'):
                v=v[1:v.rfind('"')]
            else:
                try: v=float(v)
                except: pass
            cur[k]=v
            if k=='id': P[v]=cur
    return P
