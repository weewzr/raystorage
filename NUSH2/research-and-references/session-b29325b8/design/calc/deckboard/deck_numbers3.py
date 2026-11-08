"""Extra snapshot values for the storyboard. Read-only."""
import sys, importlib.util
from pathlib import Path
sys.stdout.reconfigure(encoding='utf-8')
S = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)
s=m.S_B; b=m.balance("REC_F",s=s); c=m.carbon("REC_F"); e=m.emissions(c,s); A=e['A_atm']
for n,nm in ((1.0,'1.0'),(0.6,'0.6'),(m.N_SCALE['NGNP'],'0.31')):
    print('n',nm,'BOAK capex',round(m.reactor_capex(210,'central','BOAK',n),1),'FOAK',round(m.reactor_capex(210,'central','FOAK',n),1))
    for st in ('BOAK','FOAK'):
        print('  ',st,[round(dict(enumerate(m.ladder_totals(m.ledger(b,s,stage=st,life=l,n=n,R=210.0))))[3][1]*1e6/A,1) for l in (25,40,60)])
print('R179 n1 BOAK',[round(m.ladder_totals(m.ledger(b,s,stage='BOAK',life=l,n=1.0,R=b['R']))[3][1]*1e6/A,1) for l in (25,40,60)])
print('product C t/y',m.tpy(c['P'],s),'closure',m.tpy(c['F'],s)-e['captured']-e['atm'])
print('ship burden',e['captured']*m.TS_BURDEN, (A-e['captured']*m.TS_BURDEN))
print('ngsave pct',(52.5-34)/52.5*100,'H2 Nm3 ratio',100000/145119)
print('solvent t/h case B',c['cap']/(m.RICH-m.LEAN)*61.08/m.W_MEA/1000*s)
print('purge scaled simple',5.7*s,'model',b['e_rows']['purge'])
print('S',s,'1.2-1.3 pct',A/50e6*100,A/45e6*100)
print('walk: gap',b['gap'],'elec branch',b['branch']-b['gap'],'recyc',b['q_recyc'])
# COMB_N lowest
c4=m.carbon('COMB_N'); e4=m.emissions(c4,s); print('COMB_N A',e4['A_atm'])
