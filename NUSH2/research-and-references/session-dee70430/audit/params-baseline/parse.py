import re, collections, json, sys
ps = []
cur = None
for no, raw in enumerate(open('engine/data/parameters.toml', encoding='utf8'), 1):
    l = raw.strip()
    if not l or l.startswith('#'):
        continue
    if l == '[[param]]':
        cur = {'_line': no}
        ps.append(cur)
        continue
    k, v = l.split('=', 1)
    k = k.strip()
    v = v.strip()
    if v.startswith('"'):
        m = re.match(r'"((?:[^"\\]|\\.)*)"', v)
        cur[k] = m.group(1)
    else:
        cur[k] = float(v.split('#')[0])
json.dump(ps, open(sys.argv[1], 'w'))
print(len(ps))
for k in ['class', 'rigour', 'source_status', 'group']:
    print(k, collections.Counter(p.get(k) for p in ps).most_common())
keys = collections.Counter()
for p in ps:
    keys.update(x for x in p if x != '_line')
print(keys)
ids = collections.Counter(p['id'] for p in ps)
print('dups', [i for i, c in ids.items() if c > 1])
print('with decision', sum(1 for p in ps if p.get('decision')))
