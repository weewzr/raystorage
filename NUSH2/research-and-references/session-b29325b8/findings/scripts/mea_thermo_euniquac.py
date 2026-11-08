"""mea_thermo agent: prototype of the Aronu et al. (2011) extended UNIQUAC model.

Purpose: test whether the printed tables are sufficient. This is my own calculation.

Printed in Aronu (PDF p4 and p7): Table 2 (K, Henry), Table 7 (r, q), Tables 8 and 9 (u0, uT),
Eqs 10, 14, 15, 16.

NOT printed in Aronu, taken from the cited framework (Thomsen 1997; Thomsen and Rasmussen 1999)
from my own general knowledge, because no local file prints them. They are switchable so the
sensitivity can be measured:
  * full combinatorial, residual and Debye-Hueckel activity expressions
  * Debye-Hueckel A(T) polynomial and b = 1.5
  * coordination number z = 10
  * whether molality in the DH term uses kg water or kg (water + MEA)
"""
import json
import math
import sys

import numpy as np
from scipy.optimize import root

SP = ["H2O", "MEA", "CO2", "H3O", "MEAH", "OH", "HCO3", "CO3", "MEACOO"]
iW, iM, iC, iH, iMH, iOH, iB, iCA, iCB = range(9)
Z = np.array([0, 0, 0, 1, 1, -1, -1, -2, -1], float)
R_ = np.array([0.9200, 4.2800, 5.7410, 0.13779, 1.0241, 9.3973, 9.1571, 9.7452, 1.0741])
Q_ = np.array([1.4000, 4.2800, 6.0806, 1e-15, 2.5150, 8.8171, 6.3461, 6.4614, 0.1106])
MW_W = 0.01801528
MW_MEA = 0.06108

# Aronu Table 8, lower triangle, read from the page image (signs verified on the image).
U0 = [
    [0.0],
    [173.9645, 414.6924],
    [-151.4573, 87.5583, 40.5176],
    [1.0e4, 1.0e9, 1.0e9, 0.0],
    [-20.7732, 310.1293, 30.8035, 1.0e9, 0.0],
    [600.4952, 1.0e9, 2500.0, 1.0e9, 1.0e9, 1562.8810],
    [517.0278, 655.0881, 597.9726, 1.0e9, 732.7007, 2500.0, 743.6159],
    [361.3877, 1.0e9, 2500.0, 1.0e9, 1.0e9, 1588.0250, 719.159, 1458.3440],
    [2758.3852, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1.0e9, 1500.0],
]
# Aronu Table 9
UT = [
    [0.0],
    [0.8027, 0.6647],
    [6.0908, 4.6660, 13.6290],
    [0.0, 0.0, 0.0, 0.0],
    [-1.9174, 0.1213, 7.3541, 0.0, 0.0],
    [8.5455, 0.0, 0.0, 0.0, 0.0, 5.6169],
    [6.9504, 15.2488, 5.8077, 0.0, 2.8863, 0.0, 17.1148],
    [3.3516, 0.0, 0.0, 0.0, 0.0, 2.7496, 2.6115, -1.3448],
    [16.9192, 0, 0, 0, 0, 0, 0, 0, 0.0],
]


def sym(L):
    M = np.zeros((9, 9))
    for i, row in enumerate(L):
        for j, v in enumerate(row):
            M[i, j] = v
            M[j, i] = v
    return M


U0M = sym(U0)
UTM = sym(UT)

# Aronu Table 2: ln K or ln H = C1 + C2/T + C3 ln T + C4 T
KPAR = {
    "w": (132.899, -13445.90, -22.4773, 0.0),
    "co2": (231.465, -12092.10, -36.7816, 0.0),
    "hco3": (216.049, -12431.70, -35.4819, 0.0),
    "mea": (-4.9074, -6166.12, 0.0, -0.00098482),
    "meacoo": (2.8898, -3635.09, 0.0, 0.0),
    "h": (170.7126, -8477.711, -21.95743, 0.005781),
}


def lnK(key, T):
    a, b, c, d = KPAR[key]
    return a + b / T + c * math.log(T) + d * T


def psi(T):
    u = U0M + UTM * (T - 298.15)
    uii = np.diag(u)
    # psi[k, i] = exp(-(u_ki - u_ii)/T)
    return np.exp(np.clip(-(u - uii[None, :]) / T, -700, 700))


def A_dh(T):
    t = T - 273.15
    return 1.131 + 1.335e-3 * t + 1.164e-5 * t * t


def ln_gamma(x, T, opts, nw_over_nsolv=None):
    """Returns ln gamma with: symmetric for H2O and MEA, unsymmetric (inf dil in water) for the rest."""
    z = opts.get("z", 10.0)
    r, q = R_, Q_
    P = psi(T)
    xr = x * r
    xq = x * q
    phi_over_x = r / xr.sum()
    theta = xq / xq.sum()
    phi_over_theta = (r / xr.sum()) / (q / xq.sum())
    lnC = np.log(phi_over_x) + 1 - phi_over_x - (z / 2) * q * (np.log(phi_over_theta) + 1 - phi_over_theta)
    S = P.T @ theta  # S[i] = sum_k theta_k psi_ki
    term = np.zeros(9)
    for i in range(9):
        term[i] = np.sum(theta * P[i, :] / S)
    lnR = q * (1 - np.log(S) - term)
    lng = lnC + lnR
    # infinite dilution in water for solutes
    rr = r / r[iW]
    qq = q / q[iW]
    rq = rr / qq
    lnC_inf = np.log(rr) + 1 - rr - (z / 2) * q * (np.log(rq) + 1 - rq)
    lnR_inf = q * (1 - np.log(P[iW, :]) - P[:, iW])
    solutes = [iC, iH, iMH, iOH, iB, iCA, iCB]
    for i in solutes:
        lng[i] -= lnC_inf[i] + lnR_inf[i]
    # Debye-Hueckel, molality on chosen solvent basis
    b = opts.get("b", 1.5)
    A = opts.get("A_scale", 1.0) * A_dh(T)
    if opts.get("dh_basis", "water") == "water":
        kg = x[iW] * MW_W
    else:
        kg = x[iW] * MW_W + x[iM] * MW_MEA
    m = x / kg
    I = 0.5 * np.sum(m * Z * Z)
    sI = math.sqrt(max(I, 0.0))
    if opts.get("dh", True):
        lng_dh = np.zeros(9)
        for i in solutes:
            lng_dh[i] = -Z[i] ** 2 * A * sI / (1 + b * sI)
        xb = b * sI
        if xb > 1e-6:
            sigma = 3 / xb**3 * (1 + xb - 1 / (1 + xb) - 2 * math.log(1 + xb))
        else:
            sigma = 1.0
        lng_dh[iW] = (2.0 / 3.0) * MW_W * A * I**1.5 * sigma
        if opts.get("dh_basis", "water") != "water":
            lng_dh[iM] = (2.0 / 3.0) * MW_MEA * A * I**1.5 * sigma
        lng = lng + lng_dh
    return lng


def ideal_guess(alpha, n_w):
    a = min(alpha, 0.95)
    if a <= 0.5:
        carb, hco3 = 2 * a * 0.97, 2 * a * 0.0 + a * 0.03
        carb = a * 0.97
        mh = carb + hco3
        mea = max(1 - mh - carb, 1e-4)
    else:
        hco3 = (a - 0.5) * 1.5
        carb = max(a - hco3, 1e-3)
        mh = carb + hco3
        mea = max(1 - mh - carb, 1e-4)
    co3 = max(hco3 * 0.02, 1e-6)
    co2 = 1e-6 + 1e-4 * a
    n = np.array([n_w, mea, co2, 1e-11, mh, 1e-7, hco3, co3, carb])
    return n


def residuals(lnn, T, alpha, n_w0, opts):
    n = np.exp(lnn)
    x = n / n.sum()
    lng = ln_gamma(x, T, opts)
    lna = lng + np.log(x)
    kscale = opts.get("kscale", {})
    r = np.zeros(9)
    r[0] = lna[iH] + lna[iOH] - 2 * lna[iW] - lnK("w", T)
    r[1] = lna[iH] + lna[iB] - lna[iC] - 2 * lna[iW] - lnK("co2", T)
    r[2] = lna[iH] + lna[iCA] - lna[iB] - lna[iW] - lnK("hco3", T)
    r[3] = lna[iH] + lna[iM] - lna[iMH] - lna[iW] - lnK("mea", T) - kscale.get("mea", 0.0)
    r[4] = lna[iM] + lna[iB] - lna[iCB] - lna[iW] - lnK("meacoo", T) - kscale.get("meacoo", 0.0)
    # balances per mol apparent MEA, scaled
    r[5] = (n[iM] + n[iMH] + n[iCB] - 1.0) * 100
    r[6] = (n[iC] + n[iB] + n[iCA] + n[iCB] - alpha) * 100
    # hydrogen balance
    H0 = 2 * n_w0 + 7
    H = 2 * n[iW] + 7 * n[iM] + 3 * n[iH] + 8 * n[iMH] + n[iOH] + n[iB] + 6 * n[iCB]
    r[7] = (H - H0)
    r[8] = (n[iH] + n[iMH] - n[iOH] - n[iB] - 2 * n[iCA] - n[iCB]) * 100
    return r


def solve(T, w_mea, alpha, opts=None, x0=None):
    opts = opts or {}
    n_w0 = ((1 - w_mea) / 18.01528) / (w_mea / 61.08)
    g = ideal_guess(alpha, n_w0) if x0 is None else x0
    lnn = np.log(g)
    best = None
    for method in ("hybr", "lm"):
        sol = root(residuals, lnn, args=(T, alpha, n_w0, opts), method=method, tol=1e-10)
        rn = np.max(np.abs(residuals(sol.x, T, alpha, n_w0, opts)))
        if best is None or rn < best[1]:
            best = (sol.x, rn)
        if rn < 1e-7:
            break
        lnn = best[0]
    n = np.exp(best[0])
    x = n / n.sum()
    lng = ln_gamma(x, T, opts)
    pco2 = math.exp(lng[iC]) * x[iC] * math.exp(lnK("h", T)) / 1000.0
    return {"n": n, "x": x, "lng": lng, "pco2_kPa": pco2, "resid": best[1]}


# Benchmarks transcribed from Aronu Table 4 (PDF p5 image, 30 mass%)
T4_EXP = {
    313.15: [(0.0016, 0.102), (0.0123, 0.206), (0.0246, 0.250), (0.0603, 0.337), (0.0851, 0.353),
             (0.1835, 0.401), (0.2928, 0.417), (0.3188, 0.421), (0.3809, 0.433), (0.5702, 0.447),
             (1.0662, 0.464), (1.8326, 0.476), (1.8278, 0.477), (2.3193, 0.485), (2.8577, 0.489),
             (8.5583, 0.516), (11.812, 0.524)],
    333.15: [(0.0045, 0.053), (0.0154, 0.105), (0.0427, 0.162), (0.1348, 0.244), (0.3015, 0.303),
             (0.6436, 0.360), (1.0970, 0.393), (2.5014, 0.428), (13.558, 0.491)],
    353.15: [(0.0056, 0.017), (0.0219, 0.040), (0.0557, 0.075), (0.1406, 0.122), (0.2485, 0.155),
             (0.6137, 0.216), (1.2538, 0.271), (3.7522, 0.347), (7.9387, 0.400), (8.3031, 0.398)],
}
# Model column (Aronu's own model output): (Ptot, pCO2 model, alpha)
T4_MODEL = {
    373.15: [(103.0, 14.464, 0.344), (130.0, 42.199, 0.409), (170.2, 79.197, 0.443), (174.6, 114.232, 0.462),
             (254.6, 181.397, 0.486), (308.8, 272.872, 0.508), (338.8, 277.827, 0.509), (407.5, 361.398, 0.524),
             (525.5, 478.990, 0.541)],
    393.15: [(214.8, 38.534, 0.311), (254.3, 81.379, 0.364), (351.9, 228.102, 0.432), (469.9, 374.623, 0.464),
             (556.8, 429.659, 0.473), (662.7, 529.618, 0.487), (691.7, 578.233, 0.493), (767.7, 630.539, 0.499)],
}


def aard(pairs):
    return 100 * sum(abs(a - b) / b for a, b in pairs) / len(pairs)


def bias(pairs):
    return sum(math.log(a / b) for a, b in pairs) / len(pairs)


def run(opts, label, verbose=False):
    out = {"label": label}
    exp = []
    for T, rows in T4_EXP.items():
        for p, a in rows:
            s = solve(T, 0.30, a, opts)
            exp.append((s["pco2_kPa"], p, T, a, s["resid"]))
            if verbose:
                print(f"  exp T={T:.2f} a={a:.3f} pexp={p:9.4f} pmod={s['pco2_kPa']:10.4f} ratio={s['pco2_kPa']/p:6.2f} r={s['resid']:.1e}")
    mod = []
    for T, rows in T4_MODEL.items():
        for Pt, p, a in rows:
            s = solve(T, 0.30, a, opts)
            mod.append((s["pco2_kPa"], p, T, a, s["resid"]))
            if verbose:
                print(f"  modelcol T={T:.2f} a={a:.3f} paronu={p:9.3f} pmine={s['pco2_kPa']:10.3f} ratio={s['pco2_kPa']/p:6.2f} r={s['resid']:.1e}")
    out["AARD_vs_exp_%"] = aard([(a, b) for a, b, *_ in exp])
    out["mean_ln_ratio_exp"] = bias([(a, b) for a, b, *_ in exp])
    out["AARD_vs_Aronu_model_column_%"] = aard([(a, b) for a, b, *_ in mod])
    out["mean_ln_ratio_model"] = bias([(a, b) for a, b, *_ in mod])
    out["max_resid"] = max(max(r[4] for r in exp), max(r[4] for r in mod))
    return out


if __name__ == "__main__":
    print("K sanity at 298.15 K (ln K):", {k: round(lnK(k, 298.15), 3) for k in KPAR})
    print("H at 298.15 K, Pa:", math.exp(lnK("h", 298.15)))
    variants = [
        ("A water-basis DH, z=10, b=1.5", {}),
        ("B water+MEA basis DH", {"dh_basis": "mix"}),
        ("C no DH term", {"dh": False}),
    ]
    res = []
    for lab, o in variants:
        r = run(o, lab, verbose=(lab[0] == "A"))
        res.append(r)
        print(json.dumps(r, indent=1))
    with open("C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/mea_thermo_euniquac_results.json", "w") as f:
        json.dump(res, f, indent=1)
