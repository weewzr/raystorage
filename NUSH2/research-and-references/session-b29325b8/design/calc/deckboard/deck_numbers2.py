"""Case B deck numbers on the answered basis (Q3, Q4, Q7, Q9, Q10). Read-only. Run twice with NUSH2_AVAIL for availability."""
import sys, importlib.util, os
from pathlib import Path
sys.stdout.reconfigure(encoding='utf-8')
S = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)
s = m.S_B
b = m.balance("REC_F", s=s)
c = m.carbon("REC_F")
e = m.emissions(c, s)
A = e['A_atm']
H2 = m.H2_B_KGH*m.HOURS/1000
print(f"H2 {H2:.1f} t/y, hours {m.HOURS}")
print(f"baseline {e['baseline']:.0f}, atm {e['atm']:.0f}, captured {e['captured']:.0f}, A {A:.0f}, red {A/e['baseline']*100:.2f}%, CI {e['atm']/H2:.3f} kg/kg, ratio {A/250000:.2f}")
print('plant capture', e['captured']/(e['captured']+e['atm'])*100, 'feed C t/y', m.tpy(c['F'],s))
print('NG Case B', 34.0*s, 'NG conv', 52.5*s, 'saved', (52.5-34.0)*s)
print('rows', {k:round(v,3) for k,v in b['e_rows'].items()}, 'e_proc',b['e_proc'],'circ',b['e_circ'],'tot',b['e_total'])
print('stack', {k:round(v,2) for k,v in b['stack'].items()}, 'avail',b['avail'],'reb',b['q_reb'],'gap',b['gap'])
print('R walk: q_proc',b['q_proc'],'recyc',b['q_recyc'],'ihx',b['q_ihx'],'duct',b['q_duct'],'vessel',b['qv'],'branch',b['branch'],'wsum',b['w_s']+b['w_p'],'R',b['R'], 'spare',210-b['R'])
# bounds on baseline
for nm,bl in (('INL',m.BASE_INL),('Cho',9.35),('IEAGHG',m.BASE_IEAGHG)):
    print(nm, bl, 'baseline', bl*H2, 'A', bl*H2-e['atm'])
# capture sensitivity
for eta in (0.80,0.85,0.90,0.95,0.99):
    c2=m.carbon("REC_F",eta=eta); e2=m.emissions(c2,s); b2=m.balance("REC_F",s=s,eta_c=eta)
    print(f"eta {eta}: A {e2['A_atm']:.0f}, atm {e2['atm']:.0f}, cap {e2['captured']:.0f}, plant {e2['captured']/(e2['captured']+e2['atm'])*100:.1f}%, R {b2['R']:.1f}")
# no capture cases
base=e['baseline']
print('recycle, no capture: A', base-m.tpy(c['F'],s), ' heat only INL no-capture 2713 st/d:', (m.CO2_CONV-m.CO2_HTGR_NOCC if hasattr(m,'CO2_HTGR_NOCC') else 3205-2713)*m.ST*m.DAYS*s)
for scn in ('COMB_H','EXPORT','INL_SEL','REC_FIX'):
    c3=m.carbon(scn); e3=m.emissions(c3,s); b3=m.balance(scn,s=s)
    print(scn,'A',round(e3['A_atm']),'R',round(b3['R'],1),'NG',c3['NG']*s)
for r in (179.04,210.0):
    print('R',r,'system',m.system_adjust(b,s),'life',m.lifecycle_adjust(b,s,r),' A life',A+m.lifecycle_adjust(b,s,r),' A sys',A+m.system_adjust(b,s))
R=210.0
def ac(**kw):
    rows=m.ledger(b,s,**kw); t=m.ladder_totals(rows); return [v*1e6/A for _,v in t], rows
print('Central n=1 R=210')
for stage in ('FOAK','10-OAK INL','BOAK','10-OAK v1_7'):
  for life in (25,40,60):
    v,_=ac(stage=stage,life=life,n=1.0,R=R); print(stage,life,[round(x,1) for x in v])
v,rows=ac(stage='BOAK',life=25,n=1.0,R=R)
print({k:round(x,2) for k,x in rows.items()})
v,rows=ac(stage='BOAK',life=60,n=1.0,R=R)
print({k:round(x,2) for k,x in rows.items()})
print('capex BOAK n=1 R=210', m.reactor_capex(210,'central','BOAK',1.0),'FOAK',m.reactor_capex(210,'central','FOAK',1.0), 'R179', m.reactor_capex(b['R'],'central','BOAK',1.0))
print('--- sensitivities at BOAK 25y n=1 R=210 L3')
base3=ac(stage='BOAK',life=25,n=1.0,R=R)[0][3]
print('base',base3)
for nm,kw in (('TS23.2',dict(ts=23.2)),('TS112',dict(ts=112.0)),('gas10',dict(gas=10.0)),('gas20',dict(gas=20.0)),('n0.6',dict(n=0.6)),('n0.31',dict(n=m.N_SCALE['NGNP'])),('rx+30',dict(rx_mult=1.3)),('rx-30',dict(rx_mult=0.7)),('capt case3',dict(capt_k=m.CAPT_K['high'])),('fom6',dict(fom=0.06)),('R179',dict(R=b['R'])),('NG Ahn',dict(ng_override=42.63))):
    k=dict(stage='BOAK',life=25,n=1.0,R=R); k.update(kw)
    v=ac(**k)[0]; print(nm, round(v[3],1), round(v[3]-base3,1))
for nm,kw in (('n0.6 60',dict(n=0.6,life=60)),('n0.31 60',dict(n=m.N_SCALE['NGNP'],life=60)),('n1 60',dict(life=60))):
    k=dict(stage='BOAK',life=25,n=1.0,R=R); k.update(kw); print(nm, round(ac(**k)[0][3],1))
for nm,kw in (('SRD3.4',dict(srd=3.4)),('SRD4.2',dict(srd=4.2))):
    b2=m.balance("REC_F",s=s,**kw); rows=m.ledger(b2,s,stage='BOAK',life=25,n=1.0,R=R); t=m.ladder_totals(rows); print(nm,'R',round(b2['R'],1),round(t[3][1]*1e6/A,1))
# ceiling
dC=ac(stage='BOAK',life=25,n=1.0,R=R)[0][3]*A/1e6
print('dC L3 25y',dC,'ceiling',A*100/1e6,'excess',dC-A*100/1e6)
dC60=ac(stage='BOAK',life=60,n=1.0,R=R)[0][3]*A/1e6
print('dC L3 60y',dC60,'excess',dC60-A*100/1e6)
# gas price needed; reactor multiple needed at 25 y and 60 y
gj=(m.NG_CONV-34.0)*s*m.NG_HHV*m.MMBTU_GJ*m.DAYS/1e6
print('gas saving S$M per S$/GJ',gj,'gas needed 25y',15+(dC-A*100/1e6)/gj,'60y',15+(dC60-A*100/1e6)/gj)
print('BASE_INL',m.BASE_INL, 'K', m.CAPT_K, 'F2019',m.F2019)
