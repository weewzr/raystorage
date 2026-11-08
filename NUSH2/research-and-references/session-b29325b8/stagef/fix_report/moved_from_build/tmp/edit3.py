s=open('units.csv',encoding='utf-8').read()
lines=s.split('\n')
out=[]
for l in lines:
    if l.startswith('S$/GJ,'): l='S$/GJ,/GJ,money'
    elif l.startswith('S$/kg,'): l='S$/kg,/kg,money'
    elif l.startswith('S$M/y,'): l='S$M/y,M/y,money'
    elif l.startswith('S$/kWh,'): l='S$/kWh,/kWh,money'
    elif l.startswith('S$M per t/h,'): l=r'S$M per t/h,M\ per\ t/h,money'
    elif l.startswith('US$/MWth-h,'): l='US$/MWth-h,/MWh_{th},usmoney'
    out.append(l)
open('units.csv','w',encoding='utf-8',newline='\n').write('\n'.join(out))
s=open('tables.py',encoding='utf-8').read()
a='"Pilot the recycle; without it abatement falls to {{A_NORECYCLE_MT|u:MtCO2e/y}}."'
b='"Pilot the recycle; without it abatement is {{A_NORECYCLE_MT|f3}}~[[Mt/y]]."'
assert a in s
s=s.replace(a,b)
open('tables.py','w',encoding='utf-8',newline='\n').write(s)
s=open('qa_report.py',encoding='utf-8').read()
a='f"modal pitch {modal} pt, {off} of {total_pitch} pitches away from it: {pitch.most_common(6)}")'
b='f"modal pitch {modal} pt, {off} of {total_pitch} pitches more than 0.5 pt from it, largest {max(pitch) if pitch else None}: {sorted(pitch.items())}")'
assert a in s
s=s.replace(a,b)
open('qa_report.py','w',encoding='utf-8',newline='\n').write(s)
