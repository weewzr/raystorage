"""Golden values for the Rust port of screening.py, plus swap sensitivities. Read-only on project files.
Output: design/calc/engine_goldens.json. Run with the scratchpad venv python."""
import importlib.util, json, sys, copy
from pathlib import Path

HERE = Path(__file__).resolve().parent
S = HERE.parent.parent
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)

def scal(d):
    return {k: v for k, v in d.items() if isinstance(v, (int, float))}

out = {"constants": {"s_B": m.S_B, "H2_A_kgh": m.H2_A_KGH, "H2_B_kgh": m.H2_B_KGH, "hours": m.HOURS, "days": m.DAYS,
                     "F_C6": m.F_C6, "P_C": m.P_C, "C_per_mol": m.C_PER_MOL, "BASE_INL": m.BASE_INL, "PHI_NR": m.PHI_NR,
                     "LF10": m.LF10, "IDC": m.idc_factor(), "CRF": {str(n): m.crf(m.RATE, n) for n in (25, 40, 60)},
                     "CAPT_K": m.CAPT_K, "RECYCLE_CAPEX_A": m.RECYCLE_CAPEX_A, "n_NGNP": m.N_SCALE["NGNP"]}}
out["cases"] = {}
for bn, s in m.BASES.items():
    for scn in ("INL_SEL", "REC_F", "REC_FIX", "COMB_H", "EXPORT"):
        c = m.carbon(scn); e = m.emissions(c, s); b = m.balance(scn, s=s)
        rows = {}
        for stage in ("FOAK", "BOAK", "10-OAK INL", "10-OAK v1_7"):
            for n in (1.0, 0.6, m.N_SCALE["NGNP"]):
                for life in (25, 40, 60):
                    for R in (210.0, b["R"]):
                        l = m.ledger(b, s, stage=stage, life=life, n=n, R=R)
                        cum = [v for _, v in m.ladder_totals(l)]
                        rows[f"{stage}|n{n:.2f}|y{life}|R{R:.1f}"] = cum
        out["cases"][f"{bn}|{scn}"] = {"carbon": scal(c), "emissions": e, "balance": scal(b), "stack": b["stack"], "e_rows": b["e_rows"], "ladder_SM_y": rows}

# Headline Case B REC_F sensitivities: what each planned term swap can do to the headline
s = m.S_B
base = m.balance("REC_F", s=s)
A0 = m.emissions(m.carbon("REC_F"), s)["A_atm"]
R0 = base["R"]
sw = {"base": {"R": R0, "A": A0}}
for phi in (0.76, 0.79):
    b = m.balance("REC_F", s=s, phi=phi); A = m.emissions(m.carbon("REC_F", phi=phi), s)["A_atm"]
    sw[f"phi {phi}"] = {"R": b["R"], "A": A}
for eta in (0.85, 0.95):
    b = m.balance("REC_F", s=s, eta_c=eta); A = m.emissions(m.carbon("REC_F", eta=eta), s)["A_atm"]
    sw[f"eta {eta}"] = {"R": b["R"], "A": A}
for srd in (3.4, 4.2):
    b = m.balance("REC_F", s=s, srd=srd)
    sw[f"SRD {srd}"] = {"R": b["R"], "A": A0}
for ck in ("none", "low", "central_5p15", "high"):
    b = m.balance("REC_F", s=s, circ=ck)
    sw[f"circ {ck}"] = {"R": b["R"], "A": A0}
for lk in ("none", "low", "high"):
    b = m.balance("REC_F", s=s, loss=lk)
    sw[f"loss {lk}"] = {"R": b["R"], "A": A0}

# Process heat basis. Scale helium flow with duty so circulator load follows.
Q0, H0 = m.Q_PROC, m.HE_FLOW
for lab, q_s in (("Q -5%", 0.95 * Q0 * s), ("Q +5%", 1.05 * Q0 * s), ("Q Ahn 143.4", 143.4)):
    m.Q_PROC = q_s / s
    m.HE_FLOW = H0 * (q_s / s) / Q0
    b = m.balance("REC_F", s=s)
    sw[lab] = {"R": b["R"], "A": A0, "q_proc": b["q_proc"], "He_kg_s": H0 * (q_s / s) / Q0 * s}
m.Q_PROC, m.HE_FLOW = Q0, H0

out["swap_sensitivity_case_B_REC_F"] = sw
with open(HERE / "engine_goldens.json", "w") as f:
    json.dump(out, f, indent=1, default=float)

b = base
print("Case B REC_F balance")
for k in ("q_proc", "q_recyc", "q_reb", "avail", "gap", "q_ihx", "q_duct", "qv", "w_s", "w_p", "e_circ", "e_proc", "e_total", "branch", "R", "cap_t", "export_kept"):
    print(f"  {k:12s} {b[k]:.4f}")
print("  stack", {k: round(v, 3) for k, v in b["stack"].items()})
print("  e_rows", {k: round(v, 3) for k, v in b["e_rows"].items()})
c = m.carbon("REC_F")
print("carbon Case A REC_F", {k: round(v, 3) for k, v in c.items() if isinstance(v, float)})
print("swap table: label, R MWth, dR, A t/y, dA")
for k, v in sw.items():
    print(f"  {k:16s} R {v['R']:7.2f} dR {v['R']-R0:+6.2f}  A {v['A']:10.0f} dA {v['A']-A0:+8.0f}" + (f"  q {v['q_proc']:.1f} He {v['He_kg_s']:.2f}" if 'q_proc' in v else ""))
print("wrote", HERE / "engine_goldens.json")
