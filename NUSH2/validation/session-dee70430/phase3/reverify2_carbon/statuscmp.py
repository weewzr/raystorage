import csv,sys,collections
a=list(csv.DictReader(open(sys.argv[1],encoding='utf-8')))
b=list(csv.DictReader(open(sys.argv[2],encoding='utf-8')))
print('columns:',list(a[0].keys()))
sc=lambda rows:collections.Counter(r['status'] for r in rows)
print('A',len(a),dict(sc(a))); print('B',len(b),dict(sc(b)))
da={r['test_id']:r for r in a}; db={r['test_id']:r for r in b}
print('ids only in A:',[k for k in da if k not in db][:10])
print('new in B:',len([k for k in db if k not in da]))
flip=[(k,da[k]['status'],db[k]['status']) for k in da if k in db and da[k]['status']!=db[k]['status']]
print('status flips A->B:',flip)
tolchg=[(k,da[k]['tolerance'],db[k]['tolerance']) for k in da if k in db and da[k]['tolerance']!=db[k]['tolerance']]
print('tolerance changes:',tolchg)
fa=sorted(k for k in da if da[k]['status']=='fail'); fb=sorted(k for k in db if db[k]['status']=='fail')
print('fail ids equal:',fa==fb,len(fa),len(fb))
print([k for k in fb if k not in fa])
