import csv,re
rows=list(csv.DictReader(open('math/eq_register.csv',encoding='utf-8')))
for r in rows:
    blob=' '.join(r.values())
    for m in re.finditer(r'[\w/]+\.(md|py|json)\b|\bQ\d+\b|`|refute|design/|findings/|outram|htgr_ihx|smr_wgs|mea_columns|mea_thermo|econ_lit|screening\.',blob):
        print(r['id'],m.group(0),'::',blob[max(0,m.start()-40):m.end()+30])
for line in open('math/generated/values.tex',encoding='utf-8'):
    m=re.match(r'.newcommand.{2}(\w+).{2}(.*?). % class',line)
    if m and m.group(1)[0]=='c': print(m.group(1),m.group(2),end=' | ')
print()
