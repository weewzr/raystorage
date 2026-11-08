import json,sys
sys.stdout.reconfigure(encoding='utf-8')
from datetime import datetime
p=sys.argv[1]; n=int(sys.argv[2]) if len(sys.argv)>2 else 6
rows=[]
for l in open(p,encoding='utf-8'):
    try: d=json.loads(l)
    except: continue
    ts=d.get('timestamp')
    if ts: rows.append((datetime.fromisoformat(ts.replace('Z','+00:00')),d))
print(len(rows), rows[0][0], rows[-1][0])
prev=None
for t,d in rows:
    if prev and (t-prev).total_seconds()>300: print('gap',prev,'->',t,round((t-prev).total_seconds()/60),'min')
    prev=t
for t,d in rows[-n:]:
    m=d.get('message',{}); c=m.get('content')
    if isinstance(c,list):
        for x in c:
            ty=x.get('type')
            if ty=='text': print(t,'TEXT',x['text'][:700])
            elif ty=='tool_use': print(t,'TOOL',json.dumps(x.get('input'))[:500])
            elif ty=='tool_result': print(t,'RES',json.dumps(x.get('content'))[:500])
    else: print(t,str(c)[:300])
