import json,re,sys
ps=json.load(open(sys.argv[1]))
pat=re.compile(sys.argv[2],re.I)
for p in ps:
    blob=' '.join(str(p.get(k,'')) for k in ('id','group','name','unit'))
    if pat.search(blob):
        print(p['_line'],p['id'],p['value'],p['unit'],p.get('low'),p.get('high'),p['group'],p['class'],p['source_status'],p.get('decision',''),'|',p['source'][:110])
