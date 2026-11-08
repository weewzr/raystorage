"""mea_thermo agent: extended V&V of the corrected Aronu prototype. My own calculation.

Convention under test: K_MEA is on an infinite-dilution-in-water basis for MEA.
Checks:
 1. model-column code verification, 15/30/45/60 mass%, with and without SRK + Poynting
 2. AARD against experimental pCO2 and Ptot-free data
 3. N2O-analogy physical solubility against Hartono Tables 10 and 11
 4. Speciation sanity
 5. Reduced correlations (Dugas Eq 5.26) against the same data
 6. Gibbs-Helmholtz heat of absorption
 7. Plant conditions: loading at pCO2 = 378 kPa
"""
import importlib.util
import json
import math

import numpy as np
from scipy.optimize import brentq

S = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/scripts/mea_thermo_euniquac.py"
spec = importlib.util.spec_from_file_location("eu", S)
eu = importlib.util.module_from_spec(spec)
spec.loader.exec_module(eu)

R = 8.314462


def lg_inf(T, z=10.0):
    r, q = eu.R_, eu.Q_
    P = eu.psi(T)
    i, w = eu.iM, eu.iW
    rr, qq = r[i] / r[w], q[i] / q[w]
    rq = rr / qq
    return math.log(rr) + 1 - rr - (z / 2) * q[i] * (math.log(rq) + 1 - rq) + q[i] * (1 - math.log(P[w, i]) - P[i, w])


def opts(T):
    return {"kscale": {"mea": lg_inf(T)}}


def pco2(T, w, a):
    return eu.solve(T, w, a, opts(T))


def srk_phi_co2(T, P, y_co2):
    """SRK fugacity coefficient of CO2 in CO2/H2O vapour, k_ij = 0. Critical data are general knowledge."""
    Tc = np.array([304.12, 647.1])
    Pc = np.array([7.374e6, 22.064e6])
    om = np.array([0.225, 0.344])
    y = np.array([y_co2, 1 - y_co2])
    m = 0.480 + 1.574 * om - 0.176 * om**2
    alpha = (1 + m * (1 - np.sqrt(T / Tc))) ** 2
    a = 0.42748 * R**2 * Tc**2 / Pc * alpha
    b = 0.08664 * R * Tc / Pc
    amix = (y[:, None] * y[None, :] * np.sqrt(a[:, None] * a[None, :])).sum()
    bmix = (y * b).sum()
    A = amix * P / (R * T) ** 2
    B = bmix * P / (R * T)
    roots = np.roots([1, -1, A - B - B**2, -A * B])
    Z = max(r.real for r in roots if abs(r.imag) < 1e-9)
    a0 = (y * np.sqrt(a[0] * a)).sum()
    return math.exp(b[0] / bmix * (Z - 1) - math.log(Z - B) - A / B * (2 * a0 / amix - b[0] / bmix) * math.log(1 + B / Z))


def report_model_col():
    print("\n=== 1. code verification vs Aronu pCO2-model columns ===")
    T3 = {  # 15 mass% corrigendum Table 3 (PDF p16)
        333.15: [(82.00, 95.896, 0.624), (149.3, 133.258, 0.652), (178.9, 155.405, 0.666), (298.4, 234.407, 0.706),
                 (407.5, 292.900, 0.729), (611.7, 470.243, 0.780), (671.9, 487.652, 0.784), (819.7, 858.45, 0.847),
                 (934.1, 998.77, 0.864)],
        353.15: [(138.3, 101.535, 0.551), (194.7, 208.891, 0.605), (288.2, 248.881, 0.620), (412.5, 358.882, 0.654),
                 (495.4, 455.126, 0.678), (598.5, 505.150, 0.689), (689.0, 539.098, 0.696), (792.7, 640.229, 0.715),
                 (898.6, 768.969, 0.736)],
        373.15: [(109.4, 14.623, 0.346), (200.7, 204.957, 0.527), (322.8, 278.634, 0.551), (391.5, 318.213, 0.562),
                 (500.0, 436.601, 0.590), (598.3, 576.507, 0.617), (717.3, 623.174, 0.625), (717.6, 647.446, 0.629),
                 (799.9, 697.913, 0.637), (917.5, 757.845, 0.646)],
        393.15: [(241.0, 93.575, 0.385), (361.8, 182.307, 0.437), (464.2, 321.367, 0.483), (529.8, 400.784, 0.502),
                 (702.6, 629.533, 0.544), (857.2, 737.313, 0.560), (1050.2, 929.845, 0.585)],
    }
    T5 = {  # 45 mass% corrigendum Table 5 (PDF p17)
        353.15: [(56.3, 36.099, 0.503), (151.8, 106.190, 0.534), (322.2, 264.384, 0.567), (422.9, 364.534, 0.580),
                 (509.9, 421.021, 0.586), (627.9, 612.781, 0.602), (714.6, 771.31, 0.612), (868.0, 947.05, 0.621),
                 (1030.4, 1243.31, 0.633)],
        373.15: [(86.6, 4.708, 0.270), (201.7, 96.884, 0.445), (319.4, 208.127, 0.479), (433.3, 238.500, 0.485),
                 (539.0, 341.539, 0.501), (625.4, 389.847, 0.507), (747.2, 434.768, 0.512), (879.5, 574.312, 0.525),
                 (974.6, 611.783, 0.528), (1031.0, 693.481, 0.534)],
        393.15: [(165.1, 3.560, 0.145), (241.5, 106.177, 0.374), (361.5, 250.646, 0.426), (443.7, 335.793, 0.443),
                 (538.7, 490.925, 0.465), (653.1, 592.662, 0.476), (752.2, 667.434, 0.483), (852.7, 750.925, 0.490),
                 (928.9, 887.076, 0.500), (1039.6, 979.386, 0.506)],
    }
    T6 = {  # 60 mass% Table 6 (PDF p6)
        373.15: [(106.3, 30.942, 0.386), (204.9, 78.602, 0.430), (308.6, 126.628, 0.450), (414.6, 226.909, 0.473),
                 (518.3, 279.632, 0.481), (618.4, 438.671, 0.498), (705.0, 529.194, 0.505)],
        393.15: [(196.9, 77.117, 0.346), (306.0, 185.399, 0.398), (412.3, 306.435, 0.426), (537.0, 395.997, 0.440),
                 (700.0, 551.896, 0.458), (963.3, 845.133, 0.481), (1060.2, 945.232, 0.487)],
    }
    all_rows = {}
    for w, tab in ((0.15, T3), (0.30, eu.T4_MODEL), (0.45, T5), (0.60, T6)):
        ratios, ratios_g = [], []
        for T, rows in tab.items():
            for Pt, p, a in rows:
                s = pco2(T, w, a)
                pm = s["pco2_kPa"]
                y = min(pm / Pt, 0.999)
                phi = srk_phi_co2(T, Pt * 1e3, y)
                vinf = 3.5e-5  # m3/mol, assumed, not printed in Aronu
                psat = 1e5 * math.exp(11.6 - 3816.4 / (T - 46.13))
                poy = math.exp(vinf * (Pt * 1e3 - psat) / (R * T))
                pg = pm * poy / phi
                ratios.append(pm / p)
                ratios_g.append(pg / p)
        r = np.array(ratios)
        g = np.array(ratios_g)
        all_rows[w] = dict(n=len(r), aard_ideal=float(100 * np.mean(abs(r - 1))), aard_real=float(100 * np.mean(abs(g - 1))),
                           min_ratio=float(r.min()), max_ratio=float(r.max()), min_real=float(g.min()), max_real=float(g.max()))
        print(f"w={w:.2f} n={len(r)} AARD ideal gas {all_rows[w]['aard_ideal']:.2f}%  ratio {r.min():.3f}-{r.max():.3f}  | "
              f"with SRK+Poynting AARD {all_rows[w]['aard_real']:.2f}% ratio {g.min():.3f}-{g.max():.3f}")
    return all_rows


def report_exp():
    print("\n=== 2. against Aronu Table 4 experimental pCO2, 30 mass% ===")
    res = {}
    allp = []
    for T, rows in eu.T4_EXP.items():
        pairs = []
        for p, a in rows:
            pairs.append((pco2(T, 0.30, a)["pco2_kPa"], p, a))
        aa = 100 * np.mean([abs(m - e) / e for m, e, _ in pairs])
        allp += pairs
        res[T] = aa
        print(f"T={T:.2f} n={len(pairs)} AARD {aa:.1f}%  ratio range {min(m/e for m,e,_ in pairs):.2f}-{max(m/e for m,e,_ in pairs):.2f}")
    print("all", len(allp), "AARD", 100 * np.mean([abs(m - e) / e for m, e, _ in allp]))
    # Dugas Table 4.4 at 7 m (= 30 wt per Dugas Table 4.1), PDF p96
    D = [(313.15, 0.252, 15.7), (313.15, 0.351, 77), (313.15, 0.432, 465), (313.15, 0.496, 4216),
         (333.15, 0.252, 109), (333.15, 0.351, 660), (333.15, 0.432, 3434), (333.15, 0.496, 16157),
         (353.15, 0.271, 1053), (353.15, 0.366, 4443), (373.15, 0.271, 5297), (373.15, 0.366, 19008)]
    print("\nDugas Table 4.4, 7 m MEA, P* in Pa, prototype vs data vs Eq 5.26")
    v1, v2 = [], []
    for T, a, p in D:
        m = pco2(T, 7 * 0.06108 / (1 + 7 * 0.06108), a)["pco2_kPa"] * 1e3
        c = xu(T, a)
        v1.append(m / p)
        v2.append(c / p)
        print(f"T {T:.0f} a {a:.3f} data {p:8.1f} eUNIQ {m:9.1f} ratio {m/p:5.2f} | Eq5.26 {c:9.1f} ratio {c/p:5.2f}")
    print("AARD eUNIQUAC", 100 * np.mean(np.abs(np.array(v1) - 1)), "Eq 5.26", 100 * np.mean(np.abs(np.array(v2) - 1)))
    # Eq 5.26 against Aronu Table 4, in its validity range
    print("\nDugas Eq 5.26 against Aronu 30 mass% Table 4 data")
    for lo, hi in ((0.0, 2.0), (0.2, 0.55)):
        q = []
        for T, rows in eu.T4_EXP.items():
            for p, a in rows:
                if lo <= a <= hi:
                    q.append(xu(T, a) / 1e3 / p)
        q = np.array(q)
        print(f"alpha in [{lo},{hi}] n={len(q)} AARD {100*np.mean(abs(q-1)):.1f}% ratio {q.min():.2f}-{q.max():.2f}")
    return res


def xu(T, a):
    return math.exp(44.2 - 116000 / (8.314 * T) - 29.7 * a + 11600 * a / T + 17.3 * a * a)


def report_n2o():
    print("\n=== 3. apparent CO2 activity from N2O analogy, Hartono Table 11, 30 mass% ===")
    # (T C, alpha, kH kPa m3/mol)
    T11 = [(25.2, 0.0, 4.67), (25.2, 0.20, 5.64), (25.2, 0.40, 7.02), (25.2, 0.50, 7.94),
           (39.5, 0.0, 6.34), (39.5, 0.20, 7.84), (39.5, 0.40, 9.48), (39.5, 0.50, 10.90),
           (49.1, 0.0, 7.57), (49.1, 0.20, 9.68), (49.1, 0.40, 11.05), (49.1, 0.50, 12.83),
           (58.6, 0.0, 9.01), (58.6, 0.20, 11.46), (58.6, 0.40, 13.95), (58.6, 0.50, 16.15),
           (87.1, 0.0, 13.47), (87.1, 0.20, 17.33), (87.1, 0.40, 20.98), (87.1, 0.50, 25.32)]
    ratio = []
    for tc, a, k in T11:
        T = tc + 273.15
        hw_co2 = 2.82e6 * math.exp(-2044 / T)
        hw_n2o = 8.55e6 * math.exp(-2284 / T)
        h_exp = hw_co2 / hw_n2o * k * 1e3  # Pa m3/mol, Aronu Eq 19
        if a == 0:
            continue
        s = pco2(T, 0.30, a)
        x = s["x"]
        n = s["n"]
        rho = 1000.0  # kg/m3 placeholder replaced below
        mw = np.array([18.015, 61.08, 44.01, 19.02, 62.09, 17.01, 61.02, 60.01, 104.08]) * 1e-3
        mass = float((n * mw).sum())
        ntot = float(n.sum())
        # density from Hartono Table 3 trend at 30 mass%: use 1010 to 1100, interpolated linearly on alpha at 40 C
        rho = 1003.45 + (1098.66 - 1003.45) * a / 0.5 - 0.55 * (tc - 40)
        ctot = ntot / (mass / rho)
        h_model = math.exp(s["lng"][eu.iC]) * math.exp(eu.lnK("h", T)) / ctot
        ratio.append(h_model / h_exp)
        print(f"T {tc:5.1f} a {a:.2f} H_app exp {h_exp:8.1f} Pa m3/mol model {h_model:8.1f} ratio {h_model/h_exp:5.2f}  gamma*CO2 {math.exp(s['lng'][eu.iC]):.3f}")
    r = np.array(ratio)
    print("AARD", 100 * np.mean(abs(r - 1)), "ratio", r.min(), r.max())


def report_speciation():
    print("\n=== 4. speciation, 30 mass%, 40 C, mole fractions ===")
    print("alpha  MEA     MEAH+   MEACOO-  HCO3-   CO3--    CO2      OH-      H3O+   pCO2kPa")
    for a in (0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9):
        s = pco2(313.15, 0.30, a)
        x = s["x"]
        print(f"{a:.2f} {x[1]:.5f} {x[4]:.5f} {x[8]:.5f} {x[6]:.5f} {x[7]:.6f} {x[2]:.2e} {x[5]:.2e} {x[3]:.2e} {s['pco2_kPa']:.4g}")


def alpha_at(T, w, p_kPa):
    f = lambda a: math.log(pco2(T, w, a)["pco2_kPa"] / p_kPa)
    return brentq(f, 0.02, 1.4, xtol=1e-4)


def report_heat():
    print("\n=== 6. Gibbs-Helmholtz -dH (kJ/mol CO2), 30 mass%, fixed composition ===")
    print("alpha   40C    60C    80C   100C   120C")
    for a in (0.1, 0.2, 0.3, 0.4, 0.45, 0.5, 0.55, 0.6):
        row = []
        for T in (313.15, 333.15, 353.15, 373.15, 393.15):
            d = 2.0
            lp1 = math.log(pco2(T - d, 0.30, a)["pco2_kPa"])
            lp2 = math.log(pco2(T + d, 0.30, a)["pco2_kPa"])
            dH = -8.314e-3 * (lp2 - lp1) / (1 / (T + d) - 1 / (T - d))
            row.append(-dH)
        print(f"{a:.2f} " + " ".join(f"{-v:6.1f}" for v in row))


def report_plant():
    print("\n=== 7. equilibrium loading at plant pCO2, 30 mass% (prototype extrapolation) ===")
    for p in (378.0, 340.0, 100.0, 45.0, 10.0, 1.0):
        print(f"pCO2 {p:7.1f} kPa: " + "  ".join(f"{t}C {alpha_at(t + 273.15, 0.30, p):.3f}" for t in (35, 40, 50, 60, 70)))
    print("\nEq 5.26 extrapolated: alpha at 378 kPa")
    for t in (40, 50, 60, 70):
        T = t + 273.15
        print(t, round(brentq(lambda a: math.log(xu(T, a) / 378e3), 0.1, 1.2), 3))
    print("\nStripper: lean-end pCO2 at 120 C, alpha 0.25 prototype, Eq 5.26")
    for a in (0.15, 0.20, 0.25, 0.30):
        print(a, round(pco2(393.15, 0.30, a)["pco2_kPa"], 2), round(xu(393.15, a) / 1e3, 2))
    print("\nLean-end equilibrium over 40 C lean solvent, for treated gas 2.15 mol% at 21 bar = 45 kPa")
    for a in (0.15, 0.20, 0.25, 0.30):
        print(a, round(pco2(313.15, 0.30, a)["pco2_kPa"], 4), round(xu(313.15, a) / 1e3, 4))


if __name__ == "__main__":
    a = report_model_col()
    b = report_exp()
    report_n2o()
    report_speciation()
    report_heat()
    report_plant()
