"""Case B numbers for the deck storyboard, from screening.py. Read-only; prints to stdout."""
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
print('s',s,'H2 t/y', m.H2_B_KGH*m.HOURS/1000 if hasattr(m,'H2_B_KGH') else None)
print('balance keys', list(b.keys())[:80])
for k,v in b.items():
    if isinstance(v,(int,float)): print(f'  {k}: {v:.3f}')
print('emissions', e)
print('carbon', {k:(round(v,2) if isinstance(v,float) else v) for k,v in c.items()})
