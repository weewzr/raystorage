"""mea_thermo agent: diagnose the constant offset against Aronu's model column.
Tests reference-state conventions for MEA and the DH term. My own calculation.
"""
import importlib.util
import json
import math

import numpy as np
from scipy.optimize import least_squares

S = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/scripts/mea_thermo_euniquac.py"
spec = importlib.util.spec_from_file_location("eu", S)
eu = importlib.util.module_from_spec(spec)
spec.loader.exec_module(eu)


def ln_gamma_inf_mea(T, z=10.0):
    r, q = eu.R_, eu.Q_
    P = eu.psi(T)
    i, w = eu.iM, eu.iW
    rr, qq = r[i] / r[w], q[i] / q[w]
    rq = rr / qq
    C = math.log(rr) + 1 - rr - (z / 2) * q[i] * (math.log(rq) + 1 - rq)
    Rr = q[i] * (1 - math.log(P[w, i]) - P[i, w])
    return C + Rr


print("ln gamma_inf MEA in water (model), T K:")
for T in (313.15, 333.15, 353.15, 373.15, 393.15):
    print(T, round(ln_gamma_inf_mea(T), 4), "gamma_inf", round(math.exp(ln_gamma_inf_mea(T)), 4))


def col(opts):
    out = []
    for T, rows in eu.T4_MODEL.items():
        for Pt, p, a in rows:
            s = eu.solve(T, 0.30, a, opts)
            out.append((T, a, p, s["pco2_kPa"], s["resid"]))
    return out


def report(label, f):
    rows = []
    for T, table in eu.T4_MODEL.items():
        for Pt, p, a in table:
            o = f(T)
            s = eu.solve(T, 0.30, a, o)
            rows.append((T, a, p, s["pco2_kPa"]))
    ratios = [r[3] / r[2] for r in rows]
    a373 = [x for x, r in zip(ratios, rows) if r[0] == 373.15]
    a393 = [x for x, r in zip(ratios, rows) if r[0] == 393.15]
    ar = 100 * np.mean([abs(x - 1) for x in ratios])
    print(f"{label:58s} ratio373 {min(a373):.3f}-{max(a373):.3f}  ratio393 {min(a393):.3f}-{max(a393):.3f}  AARD {ar:.2f}%")
    return rows


# kscale adds to ln K in residuals. Using an infinite-dilution reference for MEA in a K while the
# activity is symmetric is equivalent to adding ln gamma_inf(MEA) to that ln K.
report("as printed (symmetric MEA)", lambda T: {})
report("K_MEA referenced to inf. dil. (+ln g_inf)", lambda T: {"kscale": {"mea": ln_gamma_inf_mea(T)}})
report("K_MEACOO referenced to inf. dil.", lambda T: {"kscale": {"meacoo": ln_gamma_inf_mea(T)}})
report("both Ks referenced to inf. dil.", lambda T: {"kscale": {"mea": ln_gamma_inf_mea(T), "meacoo": ln_gamma_inf_mea(T)}})
report("mixed DH basis, K_MEA inf. dil.", lambda T: {"dh_basis": "mix", "kscale": {"mea": ln_gamma_inf_mea(T)}})
for s in (0.8, 0.9, 1.1, 1.2, 1.3, 1.5):
    report(f"A_DH scaled by {s}", lambda T: {"A_scale": s})
for b in (1.0, 1.2, 1.8, 2.0):
    report(f"b = {b}", lambda T: {"b": b})
report("z = 0 combinatorial-staverman off", lambda T: {"z": 1e-9})
