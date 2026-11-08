"""Reconcile the user's Q8 figures with the ledger under the user's Q3, Q4, Q9, Q10 answers. Read-only."""
import sys, importlib.util
from pathlib import Path
S = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)

print("== A. User's arithmetic and the deck ledger (slides 24, 25, 28, 29) ==")
A = 0.585; ceil = 100 * A
ccs = 366224 * 80 / 1e6 * 1.30
fuel = 31.5
remain = ceil - ccs + fuel
print(f"ceiling {ceil:.2f} S$M/y, CCS {ccs:.2f}, fuel avoided {fuel}, remaining {remain:.2f}")
nonrx = ccs - fuel
print(f"non-reactor net cost {nonrx:.2f} S$M/y = {nonrx/A:.1f} S$/t")
for life, rx in ((25, 51.2), (40, 45.8), (60, 44.1)):
    for extra_name, extra in (("7.2 read as S$M/y", 7.2), ("7.2 read as S$/t", 7.2 * A), ("no 7.2", 0.0)):
        tot = nonrx + rx + extra
        print(f"life {life}: {extra_name:20s} total {tot:6.2f} S$M/y -> {tot/A:6.1f} S$/t at 0.585 | {tot/0.578:6.1f} at 0.578")
print("user formula 25y", (100 - 51.9) + 51.2 + 7.2, " 60y", (100 - 51.9) + 44.1 + 7.2)
print("first term in S$/t if done in one unit:", 100 - 51.9 / A)

print("\n== B. One ledger under Q3 A, Q4 A, Q9 A, Q10 A: Case B, recycle, bottom-up capture, T&S 60 ==")
s = m.S_B
b = m.balance("REC_F", s=s)
c = m.carbon("REC_F")
e = m.emissions(c, s)
print("reactor needed", round(b["R"], 1), "abatement keys", {k: round(v, 3) if isinstance(v, float) else v for k, v in e.items()})
Aatm = e["A_atm"]
def row(stage, life, n, R, ts=None):
    r = m.ledger(b, s, stage=stage, life=life, n=n, R=R, ts=ts)
    out = dict(m.ladder_totals(r))
    return r, [v for k, v in out.items()]
print("A_atm", Aatm)
for R in (210.0, b["R"]):
    for n in (1.0, 0.6, m.N_SCALE["NGNP"]):
        for stage in ("FOAK", "BOAK"):
            for life in (25, 40, 60):
                r, cum = row(stage, life, n, R)
                f = 1e6 / Aatm if Aatm > 100 else 1.0
                print(f"R={R:6.1f} n={n:.2f} {stage:5s} life={life}: L0 {cum[0]*f:6.1f} L1 {cum[1]*f:6.1f} L2 {cum[2]*f:6.1f} L3 {cum[3]*f:6.1f} L4 {cum[4]*f:6.1f}")
r, _ = row("BOAK", 25, 1.0, 210.0)
print("\nBOAK 25y n=1 R=210 lines S$M/y:")
for k, v in r.items():
    print(f"  {k:18s} {v:8.2f}")
print("sum to L3", sum(r[k] for k in ["gas","reactor_capital","capture_capital","capture_fom","capture_allin","ts","reactor_om","ihx","licence","decom"]))
print("reactor capex BOAK n=1 R=210 S$M:", m.reactor_capex(210.0, "central", "BOAK", 1.0), " n=0.6:", m.reactor_capex(210.0, "central", "BOAK", 0.6))
print("TS central", m.TS, " N_SCALE", m.N_SCALE, " STAGES", m.STAGES)
