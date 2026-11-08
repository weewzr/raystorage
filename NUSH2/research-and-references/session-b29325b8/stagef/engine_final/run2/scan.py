import csv, re, sys, collections
from pathlib import Path
root = Path(sys.argv[1])
pat = re.compile(sys.argv[2], re.I)
hits = collections.Counter()
for f in sorted((root/'results').glob('*.csv')):
    with open(f, newline='', encoding='utf-8') as fh:
        rd = csv.reader(fh)
        head = next(rd)
        for row in rd:
            for h, c in zip(head, row):
                for m in pat.finditer(c):
                    s = c[max(0, m.start()-80):m.end()+60]
                    hits[(f.name, h, m.group(0), s)] += 1
seen = set()
for (fn, h, g, s), n in sorted(hits.items()):
    key = (fn, h, s)
    if key in seen: continue
    seen.add(key)
    print(f"{fn}|{h}|{n}|{g}| ...{s}...")
