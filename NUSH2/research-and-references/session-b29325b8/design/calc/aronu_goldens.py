"""Port goldens from the Python Aronu prototype for the Rust acceptance test. Own calculation, no new model.
Output: design/calc/aronu_port_golden.csv and aronu_model_columns.csv (the 104 published targets as used by mea_thermo_final.py)."""
import csv, importlib.util, math, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
S = HERE.parent.parent / "findings/scripts/mea_thermo_validate.py"
spec = importlib.util.spec_from_file_location("v", S)
v = importlib.util.module_from_spec(spec); spec.loader.exec_module(v)
eu = v.eu
F = HERE.parent.parent / "findings/scripts/mea_thermo_final.py"
spec2 = importlib.util.spec_from_file_location("fin", F)
fin = importlib.util.module_from_spec(spec2); spec2.loader.exec_module(fin)

rows = []
for w in (0.15, 0.30, 0.45):
    for T in (313.15, 333.15, 353.15, 393.15):
        for a in (0.10, 0.25, 0.40, 0.50, 0.60):
            s = v.pco2(T, w, a)
            rows.append([w, T, a, s["pco2_kPa"], s["resid"], v.lg_inf(T)] + list(s["x"]) + list(s["lng"]))
hdr = ["w_mea", "T_K", "alpha", "pco2_kPa_ideal", "resid", "lngamma_inf_MEA"] + [f"x_{n}" for n in eu.SP] + [f"lng_{n}" for n in eu.SP]
with open(HERE / "aronu_port_golden.csv", "w", newline="") as f:
    wr = csv.writer(f); wr.writerow(hdr)
    for r in rows:
        wr.writerow([f"{x:.10g}" for x in r])
print("port points", len(rows), "max resid", max(r[4] for r in rows))
print("sample 30 wt%, 40 C, alpha 0.50: p* kPa", round(v.pco2(313.15, 0.30, 0.50)["pco2_kPa"], 3))
print("alpha at 378 kPa 40 C", round(v.alpha_at(313.15, 0.30, 378.0), 3), " at 45 kPa", round(v.alpha_at(313.15, 0.30, 45.0), 3))
print("lngamma_inf MEA at 313.15", round(v.lg_inf(313.15), 3), "at 393.15", round(v.lg_inf(393.15), 3))
