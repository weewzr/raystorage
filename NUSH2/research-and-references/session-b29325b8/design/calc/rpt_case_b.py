"""Case B numbers the report plan needs, from the screening model. Read-only arithmetic; prints only."""
import sys, os, importlib.util
from pathlib import Path
S = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)
s = m.S_B
print("avail", m.AVAIL, "hours", m.HOURS, "s", round(s, 4))

print("\n== Case B walk (same steps as screening s7.1, scaled) ==")
Q = m.Q_PROC * s
steps = [("Deck sum at Case B: 120.8 + 11.6*s/0.397", Q + 11.6 * s / m.ETA_PWR),
         ("Add purge row (17.3*s/0.397)", Q + 17.3 * s / m.ETA_PWR)]
b1 = m.balance("INL_SEL", s=s, circ="none", loss="central"); steps.append(("Add vessel, IHX, duct losses", b1["R"]))
b2 = m.balance("INL_SEL", s=s, circ="central", loss="central"); steps.append(("Add helium circulators", b2["R"]))
b3 = m.balance("REC_F", s=s, srd=0.0001); steps.append(("Swap Selexol for MEA, zero reboiler", b3["R"]))
b4 = m.balance("REC_F", s=s); steps.append(("Add MEA reboiler gap at 3.8", b4["R"]))
prev = None
for n, v in steps:
    print(f"  {n:48s} {v:7.1f}  {'' if prev is None else f'{v-prev:+.1f}'}")
    prev = v
print(f"  Spare at 210: {210-b4['R']:.1f}")

print("\n== Case B heat and power, REC_F central ==")
b = b4
for k in ("q_proc", "q_recyc", "q_ihx", "q_duct", "qv", "q_reb", "avail", "gap", "branch", "w_s", "w_p", "e_circ", "e_proc", "e_total", "e_nuc", "cap_t", "R", "export_kept"):
    print(f"  {k:12s} {b[k]:8.2f}")
print("  stack", {k: round(v, 2) for k, v in b["stack"].items()})
print("  e_rows", {k: round(v, 2) for k, v in b["e_rows"].items()})
hf = b["q_proc"] + b["q_recyc"] + b["q_ihx"] + b["q_duct"] + b["qv"] + b["gap_after_tg"] - (b["w_s"] + b["w_p"])
print("  heat-first load", round(hf, 1), " power from balance at 210 with 39.7%:", round((210 - hf) * m.ETA_PWR, 1))
print("  spare at 210 in MWe equivalent:", round((210 - b["R"]) * m.ETA_PWR, 1))
for nm, kw in (("low (SRD3.4, low circ/loss)", dict(srd=3.4, circ="low", loss="low")),
               ("high (SRD4.2, high, direct)", dict(srd=4.2, circ="high", loss="high", route="direct"))):
    print("  ", nm, round(m.balance("REC_F", s=s, **kw)["R"], 1))

print("\n== Carbon and emissions Case B, REC_F ==")
c = m.carbon("REC_F"); e = m.emissions(c, s)
feed = m.tpy(c["F"], s); prod = m.tpy(c["P"], s)
print({k: round(v) for k, v in e.items()}, "feed", round(feed), "product C", round(prod), "slip", round(m.tpy(c["slip"], s)), "recycled", round(m.tpy(c["R"], s)), "tail gas C", round(m.tpy(c["G"], s)))
h2 = m.H2_B_KGH * m.HOURS / 1000
print("h2 t/y", round(h2), "baseline kg/kg", round(e["baseline"] / h2, 3), "residual kg/kg", round(e["atm"] / h2, 3), "reduction %", round(100 * e["A_atm"] / e["baseline"], 1), "plant capture %", round(100 * e["captured"] / (e["captured"] + e["atm"]), 1))
print("gas saved t/y CO2-equivalent", round(e["baseline"] - feed), "share", round(100 * (e["baseline"] - feed) / e["baseline"], 1))
for nm, ef in (("Cho 9.35", 9.35), ("INL", e["baseline"] / h2), ("IEAGHG 9.00", 9.00)):
    print(f"  baseline {nm:12s} {ef*h2:10,.0f} t/y  A {ef*h2-e['atm']:10,.0f}")
print("per-pass capture sensitivity:")
for eta in (0.80, 0.85, 0.90, 0.95):
    c2 = m.carbon("REC_F", eta=eta); e2 = m.emissions(c2, s); b2x = m.balance("REC_F", s=s, eta_c=eta)
    print(f"  eta {eta:.2f} captured {e2['captured']:9,.0f} atm {e2['atm']:8,.0f} A {e2['A_atm']:9,.0f} plant {100*e2['captured']/(e2['captured']+e2['atm']):.1f}% R {b2x['R']:.1f}")
print("availability scaling of A:")
for av in (0.65, 0.70, 0.75, 0.80, 0.85, 0.90):
    print(f"  {av:.2f} A {e['A_atm']*av/0.85:9,.0f}  H2 {h2*av/0.85:8,.0f}")
print("T&S ship fuel burden 2.5%:", round(0.025 * e["captured"]), "A after", round(e["A_atm"] - 0.025 * e["captured"]))
print("system adjust t/y", round(m.system_adjust(b, s)))

print("\n== Ledger lines, S$M/y, n=1.0, R=210 and R=179, BOAK and FOAK ==")
ceil = 100 * e["A_atm"] / 1e6
print("ceiling at S$100/t:", round(ceil, 2))
def line(stage, life, n, R, ts=None, gas=None, rx=1.0):
    r = m.ledger(b, s, stage=stage, life=life, n=n, R=R, ts=ts, gas=gas, rx_mult=rx)
    return r, m.ladder_totals(r)
keys = ["gas", "reactor_capital", "capture_capital", "capture_fom", "ts", "reactor_om", "ihx", "licence", "decom", "idc", "power_island", "recycle_train", "lost_steam", "baseline_power"]
for stage, life, n, R in (("BOAK", 25, 1.0, 210.0), ("BOAK", 60, 1.0, 210.0), ("FOAK", 25, 1.0, 210.0), ("FOAK", 60, 1.0, 210.0)):
    r, t = line(stage, life, n, R)
    print(stage, life, "n", n, "R", R, {k: round(r[k], 2) for k in keys})
    print("   cum S$M/y", [round(v, 1) for _, v in t], " S$/t", [round(v / e["A_atm"] * 1e6, 1) for _, v in t])
print("\nAll three n, L3, S$/t")
for stage in ("BOAK", "FOAK", "10-OAK INL"):
    for n in (1.0, 0.6, m.N_SCALE["NGNP"]):
        row = []
        for life in (25, 40, 60):
            r, t = line(stage, life, n, 210.0)
            row.append(round(t[3][1] / e["A_atm"] * 1e6, 1))
        print(f"  {stage:11s} n={n:.2f} 210 MWth: {row}", " capex S$M", round(m.reactor_capex(210.0, 'central', stage, n), 0))

print("\nOne at a time around BOAK, 25 y, n = 1.0, R = 210, L3")
r0, t0 = line("BOAK", 25, 1.0, 210.0)
base = t0[3][1]
def show(nm, val):
    print(f"  {nm:40s} {val/e['A_atm']*1e6:7.1f}  {(val-base)/e['A_atm']*1e6:+7.1f}")
show("Central", base)
for nm, kw in (("T&S 23.2", dict(ts=23.2)), ("T&S 112", dict(ts=112.0)), ("Gas 10", dict(gas=10.0)), ("Gas 20", dict(gas=20.0)), ("Reactor capital +30%", dict(rx=1.3)), ("Reactor capital -30%", dict(rx=0.7))):
    show(nm, line("BOAK", 25, 1.0, 210.0, **kw)[1][3][1])
for nm, kw in (("SRD 3.4", dict(srd=3.4)), ("SRD 4.2", dict(srd=4.2))):
    bx = m.balance("REC_F", s=s, **kw)
    show(nm + " (reactor charge held 210)", m.ladder_totals(m.ledger(bx, s, stage="BOAK", life=25, n=1.0, R=210.0))[3][1])
print("  R charged 179.0: S$/t", round(m.ladder_totals(m.ledger(b, s, stage="BOAK", life=25, n=1.0))[3][1] / e["A_atm"] * 1e6, 1), " 60y", round(m.ladder_totals(m.ledger(b, s, stage="BOAK", life=60, n=1.0))[3][1] / e["A_atm"] * 1e6, 1))
print("  Levers to S$100/t at BOAK, 25 y, n=1, R=210: excess S$M/y", round(base - ceil, 2))
gj = (m.NG_CONV * s - m.NG_C6 * s) * m.NG_HHV * m.MMBTU_GJ * m.DAYS / 1e6
print("   gas GJ/y (M)", round(gj, 3), " gas price needed", round(15 + (base - ceil) / gj, 2))
print("   reactor capital+O&M multiple needed", round(1 - (base - ceil) / (r0["reactor_capital"] + r0["reactor_om"]), 3))
print("   T&S needed", round(60 - (base - ceil) / (e["captured"] / 1e6), 1))

print("\nComparator at Case B: IEAGHG fractions applied to the INL baseline, less 2.5% T&S")
for nm, f, cost in (("Syngas only 55.7%", 0.557, 109), ("Syngas+flue 90.0%", 0.90, 162)):
    A = e["baseline"] * f * 0.975
    print(f"  {nm}: A {A:,.0f} t/y, S${cost}/t, S$M/y {A*cost/1e6:.1f}")
