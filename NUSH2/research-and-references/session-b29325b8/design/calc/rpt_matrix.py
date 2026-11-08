import sys, importlib.util
from pathlib import Path
S = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)
s = m.S_B
b = m.balance("REC_F", s=s); c = m.carbon("REC_F"); e = m.emissions(c, s); A = e["A_atm"]
print("L3 S$/t, R=210: stage, n: 25y, 60y | L4 25y, 60y")
for st in ("FOAK", "BOAK", "10-OAK INL", "10-OAK v1_7"):
    for n in (1.0, 0.6, m.N_SCALE["NGNP"]):
        v = []
        for life in (25, 60):
            t = m.ladder_totals(m.ledger(b, s, stage=st, life=life, n=n, R=210.0))
            v.append((t[3][1]/A*1e6, t[4][1]/A*1e6))
        print(f"{st:12s} n={n:.2f}: {v[0][0]:6.1f} {v[1][0]:6.1f} | {v[0][1]:6.1f} {v[1][1]:6.1f}")
print("Case A REC_F, R=261.5 own need, n=1 BOAK:", [round(m.ladder_totals(m.ledger(m.balance('REC_F'), 1.0, stage='BOAK', life=l, n=1.0))[3][1]/m.emissions(c,1.0)['A_atm']*1e6,1) for l in (25,60)])
print("Selexol Case B R, S$/t n=1 BOAK 25y R=210:", round(m.balance('INL_SEL', s=s)['R'],1), round(m.ladder_totals(m.ledger(m.balance('INL_SEL', s=s), s, stage='BOAK', life=25, n=1.0, R=210.0))[3][1]/m.emissions(m.carbon('INL_SEL'),s)['A_atm']*1e6,1))
b2=m.balance('COMB_H', s=s); e2=m.emissions(m.carbon('COMB_H'), s)
print("COMB_H Case B R", round(b2['R'],1), 'A', round(e2['A_atm']), 'NG', round(44.5*s,1), 'S$/t BOAK n=1 25y R=210', round(m.ladder_totals(m.ledger(b2, s, stage='BOAK', life=25, n=1.0, R=210.0))[3][1]/e2['A_atm']*1e6,1))
h=66925; extra=h*0.15/0.85; print('backup H2 t/y',round(extra),' at baseline 9.21:',round(extra*9.21))
