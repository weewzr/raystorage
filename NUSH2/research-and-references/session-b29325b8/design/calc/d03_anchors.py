"""Arithmetic anchors for design/03_derivations_outline.md. Read-only import of screening.py."""
import importlib.util
import json
import math
import sys
from pathlib import Path

S = Path(__file__).resolve().parent.parent.parent
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec)
sys.modules["screening"] = m
spec.loader.exec_module(m)

s = m.S_B
print("s", s, "H2 kg/h", m.H2_B_KGH, "t/y", m.H2_B_KGH * m.HOURS / 1000, "hours", m.HOURS, "days", m.DAYS)
print("NG B MMSCFD", 34.0 * s, "Case A kmol C/h feed", m.F_C6, "P_C", m.P_C, "phi_NR", m.PHI_NR)
print("BASE_INL kg/kg", m.BASE_INL)
b = m.balance("REC_F", s=s)
c = b["c"]
e = m.emissions(c, s)
print("REC_F B reactor", b["R"], "q_proc", b["q_proc"], "q_recyc", b["q_recyc"], "q_reb", b["q_reb"], "avail", b["avail"], "gap", b["gap"])
print("stack", b["stack"])
print("q_ihx", b["q_ihx"], "q_duct", b["q_duct"], "qv", b["qv"], "w_s", b["w_s"], "w_p", b["w_p"], "e_circ", b["e_circ"])
print("e_rows", b["e_rows"], "e_proc", b["e_proc"], "e_total", b["e_total"], "branch", b["branch"], "export_kept MMBtu/h", b["export_kept"])
print("carbon A", {k: (round(v, 2) if isinstance(v, float) else v) for k, v in c.items()})
print("carbon scaled kmol C/h", {k: round(c[k] * s, 2) for k in ("F", "T", "co2_abs", "cap", "G", "R", "burn", "P", "slip", "unconv")})
print("cap t/h", b["cap_t"], "emissions", {k: round(v, 1) for k, v in e.items()})
print("plant capture", c["cap"] / c["F"], "per pass", c["cap"] / c["co2_abs"])
solv = c["cap"] * s / (m.RICH - m.LEAN) * 61.08 / m.W_MEA / 3600.0
print("solvent kg/s", solv, "t/h", solv * 3.6, "pCO2 at abs bar", 26.8 * 0.177)
print("baseline t/y", e["baseline"], "A_atm", e["A_atm"], "bounds Cho", (9.35 - m.BASE_INL) * m.H2_B_KGH * m.HOURS / 1000 + e["A_atm"], "IEAGHG", (m.BASE_IEAGHG - m.BASE_INL) * m.H2_B_KGH * m.HOURS / 1000 + e["A_atm"])
print("system adj", m.system_adjust(b, s), "lifecycle adj at R", m.lifecycle_adjust(b, s, b["R"]))
print("CRF", m.crf(.08, 25), m.crf(.08, 40), m.crf(.08, 60), "SFF25", m.sff(.08, 25), "LF10", m.LF10, "IDC", m.idc_factor())
for R in (210.0, b["R"]):
    for n in (1.0, 0.6, m.N_SCALE["NGNP"]):
        for st in ("BOAK", "FOAK"):
            for life in (25, 60):
                r = m.ledger(b, s, stage=st, life=life, n=n, R=R)
                cum = [v for _, v in m.ladder_totals(r)]
                print(f"R {R:6.1f} n {n:.2f} {st} {life}y L0 {cum[0]*1e6/e['A_atm']:.1f} L3 {cum[3]*1e6/e['A_atm']:.1f} L4 {cum[4]*1e6/e['A_atm']:.1f} L5 {cum[5]*1e6/e['A_atm']:.1f}")
r = m.ledger(b, s, stage="BOAK", life=25, n=1.0, R=210.0)
print({k: round(v, 3) for k, v in r.items()})
print("capex BOAK n1 R210", m.reactor_capex(210.0, "central", "BOAK", 1.0), "n.6", m.reactor_capex(210.0, "central", "BOAK", 0.6), "n.31", m.reactor_capex(210.0, "central", "BOAK", m.N_SCALE["NGNP"]))
print("n_NGNP", m.N_SCALE["NGNP"])
# helium checks
print("He s-scaled flow", m.HE_FLOW * b["q_proc"] / m.Q_PROC, "primary flow at R=179", 179e6 / m.CP_HE / (925 - 491), "at 210", 210e6 / m.CP_HE / (925 - 491))
print("UA balanced 120.8/25", 120.8 / 25, "x HTTR", 120.8 / 25 / 0.085, "NTU", 120.8e6 / 25 / (m.HE_FLOW * s * 5195))
print("dTref sens: eff", (900 - 466) / (925 - 466))
# Q8 corrected units
A = 0.585075
for a in (0.585, 0.578):
    ceil = 100 * a
for a, name in ((0.585, "deck"), (e["A_atm"] / 1e6, "headline")):
    nonrx = 38.09 - 31.5
    for life, rx in ((25, 51.2), (60, 44.1)):
        print(name, life, (nonrx + rx + 7.2) / a)
# 925/900/466 enthalpy
print("Q_proc check", 78.49 * 5190 * 434 / 1e6, 78.49 * 5195 * 434 / 1e6)
print("Case B 5.7 scaled", 5.7 * s, "with slip factor", b["e_rows"]["purge"])
print("11.8 proc A", 17.3 * s)
