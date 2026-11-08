"""Headline reference values for the site acceptance table. Read-only use of screening.py."""
import sys, importlib.util, json
from pathlib import Path
S = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("screening", S / "findings/scripts/screening.py")
m = importlib.util.module_from_spec(spec); sys.modules["screening"] = m; spec.loader.exec_module(m)
s = m.S_B
b = m.balance("REC_F", s=s)
c = m.carbon("REC_F")
e = m.emissions(c, s)
out = {
 "s": s, "H2_t_y": m.H2_B_KGH*m.HOURS/1000, "H2_kg_h": m.H2_B_KGH,
 "NG_MMSCFD": c["NG"]*s, "R_needed": b["R"], "surplus_210": 210-b["R"],
 "q_proc": b["q_proc"], "q_recyc": b["q_recyc"], "q_reb": b["q_reb"], "gap": b["gap"], "avail": b["avail"], "stack": b["stack"],
 "branch": b["branch"], "qv": b["qv"], "q_ihx": b["q_ihx"], "q_duct": b["q_duct"], "w_s": b["w_s"], "w_p": b["w_p"],
 "e_rows": b["e_rows"], "e_proc": b["e_proc"], "e_circ": b["e_circ"], "e_total": b["e_total"],
 "cap_t_h": b["cap_t"], "captured_t_y": e["captured"], "atm_t_y": e["atm"], "baseline_t_y": e["baseline"], "A_atm": e["A_atm"],
 "plant_capture": e["captured"]/(e["captured"]+e["atm"]),
 "He_sec_kg_s": m.HE_FLOW*(b["q_proc"]+b["q_recyc"])/m.Q_PROC,
 "He_prim_kg_s": 210e6/m.CP_HE/(m.ROT-(m.T_HE_R+25.0)),
 "He_prim_kg_s_R179": b["R"]*1e6/m.CP_HE/(m.ROT-(m.T_HE_R+25.0)),
 "solvent_t_h": (c["cap"]/(m.RICH-m.LEAN)*61.08/m.W_MEA/1000)*s,
 "tail_gas_C_kmol_h": c["G"]*s, "recycled_C_kmol_h": c["R"]*s, "burned_C_kmol_h": c["burn"]*s, "feed_C": c["F"]*s, "tube_C": c["T"]*s, "co2_abs": c["co2_abs"]*s, "slip": c["slip"]*s, "unconv": c["unconv"]*s,
 "H2_t_h_rich_at_88pct": m.H2_B_KGH/m.PSA_REC/1000,
}
for k,v in list(out.items()):
    if isinstance(v,float): out[k]=round(v,4)
    if isinstance(v,dict): out[k]={a:round(x,3) for a,x in v.items()}
print(json.dumps(out,indent=1))
A = e["A_atm"]
print("cost per t (L3), S$/tCO2e")
for R in (210.0, b["R"]):
  for n,lab in ((1.0,"1.0"),(0.6,"0.6"),(m.N_SCALE["NGNP"],"0.31")):
    for stage in ("FOAK","10-OAK INL","BOAK"):
      row=[]
      for life in (25,40,60):
        r=m.ledger(b,s,stage=stage,life=life,n=n,R=R)
        cum=dict(m.ladder_totals(r))
        vals=list(cum.values())
        row.append(round(vals[3]*1e6/A,1))
      print(f"R={R:6.1f} n={lab:>4} {stage:10s} 25/40/60: {row}")
print("ledger lines n=1 BOAK R=210 25y")
r=m.ledger(b,s,stage="BOAK",life=25,n=1.0,R=210.0)
for k,v in r.items(): print(f"  {k:16s}{v:9.2f}")
# Ahn heat basis
