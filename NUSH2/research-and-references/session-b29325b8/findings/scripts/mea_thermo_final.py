"""mea_thermo agent: final corrected V&V set. My own calculations. Prints everything the report quotes."""
import importlib.util
import json
import math

import numpy as np
from scipy.optimize import brentq

S = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/scripts/mea_thermo_validate.py"
spec = importlib.util.spec_from_file_location("v", S)
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
eu = v.eu
R = 8.314462
VINF = 3.5e-5  # m3/mol, CO2 infinite-dilution partial molar volume, assumed. Aronu cites Brelvi-O'Connell, no value printed.


def psat_w(T):
    return 1e5 * math.exp(11.6 - 3816.4 / (T - 46.13))  # bar Antoine form, general knowledge


def code_verif():
    print("\n=== 1. code verification against pCO2-model columns (corrected Table 5 temperature assignment) ===")
    T3 = {
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
    T5 = {
        333.15: [(56.3, 36.099, 0.503), (151.8, 106.190, 0.534), (322.2, 264.384, 0.567), (422.9, 364.534, 0.580),
                 (509.9, 421.021, 0.586), (627.9, 612.781, 0.602), (714.6, 771.31, 0.612), (868.0, 947.05, 0.621),
                 (1030.4, 1243.31, 0.633)],
        353.15: [(97.0, 59.190, 0.479), (251.9, 157.604, 0.512), (342.5, 279.468, 0.533), (446.1, 352.012, 0.542),
                 (523.8, 419.045, 0.549), (624.4, 509.029, 0.557), (714.8, 615.796, 0.565), (842.1, 725.466, 0.572),
                 (941.5, 872.797, 0.580)],
        373.15: [(86.6, 4.708, 0.270), (201.7, 96.884, 0.445), (319.4, 208.127, 0.479), (433.3, 238.500, 0.485),
                 (539.0, 341.539, 0.501), (625.4, 389.847, 0.507), (747.2, 434.768, 0.512), (879.5, 574.312, 0.525),
                 (974.6, 611.783, 0.528), (1031.0, 693.481, 0.534)],
        393.15: [(165.1, 3.560, 0.145), (241.5, 106.177, 0.374), (361.5, 250.646, 0.426), (443.7, 335.793, 0.443),
                 (538.7, 490.925, 0.465), (653.1, 592.662, 0.476), (752.2, 667.434, 0.483), (852.7, 750.925, 0.490),
                 (928.9, 887.076, 0.500), (1039.6, 979.386, 0.506)],
    }
    T6 = {
        373.15: [(106.3, 30.942, 0.386), (204.9, 78.602, 0.430), (308.6, 126.628, 0.450), (414.6, 226.909, 0.473),
                 (518.3, 279.632, 0.481), (618.4, 438.671, 0.498), (705.0, 529.194, 0.505)],
        393.15: [(196.9, 77.117, 0.346), (306.0, 185.399, 0.398), (412.3, 306.435, 0.426), (537.0, 395.997, 0.440),
                 (700.0, 551.896, 0.458), (963.3, 845.133, 0.481), (1060.2, 945.232, 0.487)],
    }
    tot = []
    for w, tab in ((0.15, T3), (0.30, eu.T4_MODEL), (0.45, T5), (0.60, T6)):
        rr, gg = [], []
        for T, rows in tab.items():
            for Pt, p, a in rows:
                s = v.pco2(T, w, a)
                pm = s["pco2_kPa"]
                y = min(pm / Pt, 0.98)
                phi = v.srk_phi_co2(T, Pt * 1e3, y)
                poy = math.exp(VINF * (Pt * 1e3 - psat_w(T)) / (R * T))
                pg = pm * poy / phi
                rr.append(pm / p)
                gg.append(pg / p)
                if abs(pg / p - 1) > 0.02:
                    print(f"   outlier w={w} T={T} a={a} Ptot={Pt} paronu={p} mine={pg:.2f} ratio={pg/p:.3f}")
        r, g = np.array(rr), np.array(gg)
        tot += gg
        print(f"w={w:.2f} n={len(r):3d}  ideal gas AARD {100*np.mean(abs(r-1)):.2f}% [{r.min():.3f},{r.max():.3f}]   SRK+Poynting AARD {100*np.mean(abs(g-1)):.2f}% [{g.min():.3f},{g.max():.3f}]")
    t = np.array(tot)
    print(f"all n={len(t)} AARD {100*np.mean(abs(t-1)):.3f}% median abs dev {100*np.median(abs(t-1)):.3f}%")


def n2o():
    print("\n=== 3. N2O analogy, Aronu Eq 19 direct: H_app = gamma*CO2 x H_w,CO2 (Eq 17), Hartono Table 11 ===")
    T11 = [(25.2, 0.20, 5.64), (25.2, 0.40, 7.02), (25.2, 0.50, 7.94), (39.5, 0.20, 7.84), (39.5, 0.40, 9.48),
           (39.5, 0.50, 10.90), (49.1, 0.20, 9.68), (49.1, 0.40, 11.05), (49.1, 0.50, 12.83), (58.6, 0.20, 11.46),
           (58.6, 0.40, 13.95), (58.6, 0.50, 16.15), (87.1, 0.20, 17.33), (87.1, 0.40, 20.98), (87.1, 0.50, 25.32)]
    out = []
    for tc, a, k in T11:
        T = tc + 273.15
        hw_co2 = 2.82e6 * math.exp(-2044 / T)
        hw_n2o = 8.55e6 * math.exp(-2284 / T)
        hexp = hw_co2 / hw_n2o * k * 1e3
        g = math.exp(v.pco2(T, 0.30, a)["lng"][eu.iC])
        out.append(g * hw_co2 / hexp)
    o = np.array(out)
    print(f"n={len(o)} AARD {100*np.mean(abs(o-1)):.1f}% ratio [{o.min():.3f},{o.max():.3f}] (Aronu quotes 2.7% on Hartono 2009 data)")
    print("Mole-fraction versus concentration basis: total molar density of loaded 30 wt% solution vs water")
    s = v.pco2(313.15, 0.30, 0.5)
    n = s["n"]
    mw = np.array([18.015, 61.08, 44.01, 19.02, 62.09, 17.01, 61.02, 60.01, 104.08]) * 1e-3
    rho = 1098.66
    ctot = n.sum() / ((n * mw).sum() / rho)
    print(f"  c_total at 40 C alpha 0.5 = {ctot/1000:.1f} kmol/m3 vs water 55.1 kmol/m3, ratio {55100/ctot:.2f}")


def hartono_checks():
    print("\n=== Hartono Eq 12-13 applied to measured unloaded kH, 30 mass% ===")
    x1 = (0.30 / 61.08) / (0.30 / 61.08 + 0.70 / 18.015)
    print("x1 CO2-free", round(x1, 4), "Hartono Table 3 mole fraction check at alpha 0.5 x3 0.0544 ->", round(0.5 * x1 / (1 + 0.5 * x1), 4))
    un = {25.2: 4.67, 39.5: 6.34, 49.1: 7.57, 58.6: 9.01, 87.1: 13.47}
    rows = [(25.2, 0.2, 5.64), (25.2, 0.4, 7.02), (25.2, 0.5, 7.94), (39.5, 0.2, 7.84), (39.5, 0.4, 9.48), (39.5, 0.5, 10.90),
            (49.1, 0.2, 9.68), (49.1, 0.4, 11.05), (49.1, 0.5, 12.83), (58.6, 0.2, 11.46), (58.6, 0.4, 13.95), (58.6, 0.5, 16.15),
            (87.1, 0.2, 17.33), (87.1, 0.4, 20.98), (87.1, 0.5, 25.32)]
    a1, a2 = [], []
    for t, al, k in rows:
        T = t + 273.15
        d = 0.77 * x1 + 0.033 * x1 * al * T
        a1.append(un[t] * math.exp(d) / k)
        a2.append(un[t] * math.exp(0.033 * x1 * al * T) / k)
    print(f"with b1 term: mean ratio {np.mean(a1):.3f}; without b1 term: mean ratio {np.mean(a2):.3f}; note ln(delta) at alpha=0 is b1 x1 = {0.77*x1:.4f}")

    print("\n=== viscosity against Hartono Table 7, 30 mass% ===")
    # water viscosity mPa s, Hartono Table A2 not water; use Vogel general form
    mu_w = lambda T: 2.414e-2 * 10 ** (247.8 / (T - 140.0))
    t7 = {20: [2.874, 3.112, 3.309, 3.597, 3.899, 4.251], 40: [1.628, 1.765, 1.875, 2.101, 2.246, 2.470],
          60: [1.055, 1.148, 1.235, 1.369, 1.492, 1.630], 80: [0.742, 0.818, 0.879, 0.987, 1.075, 1.143]}
    al = [0, 0.11, 0.19, 0.29, 0.39, 0.50]
    x3s = [0, 0.0130, 0.0215, 0.0328, 0.0431, 0.0544]
    w1, w2 = [], []
    for t, row in t7.items():
        T = t + 273.15
        for a, x3, e in zip(al, x3s, row):
            om = 30.0
            eta = mu_w(T) * math.exp((21.186 * om + 2373) * (a * (0.01015 * om + 0.0093 * T - 2.2589) + 1) * om / T**2)
            w1.append(eta / e)
            lnstar = (6.98 * x1 + 10.48 * a * x1) / (0.049 + x1)
            eta2 = math.exp(x3 * (lnstar - math.log(1e3)) + (1 - x3) * math.log(row[0] * 1e-3)) * 1e3
            w2.append(eta2 / e)
    w1, w2 = np.array(w1), np.array(w2)
    print(f"Weiland (Dugas Eq 5.16, Plaza Eq 3.18) AARD {100*np.mean(abs(w1-1)):.1f}% [{w1.min():.3f},{w1.max():.3f}]")
    print(f"Hartono Eq 8-9 with measured unloaded eta AARD {100*np.mean(abs(w2-1)):.1f}% [{w2.min():.3f},{w2.max():.3f}]")

    print("\n=== density against Hartono Table 3, 30 mass% ===")
    t3 = {20: [1012.68, 1030.99, 1046.92, 1068.90, 1086.89, 1108.20], 40: [1003.45, 1021.88, 1037.84, 1059.81, 1077.75, 1098.66],
          60: [992.23, 1010.90, 1027.00, 1049.12, 1067.11, 1087.88], 80: [979.27, 998.31, 1014.66, 1037.04, 1055.16, 1075.85]}
    rhow = {20: 998.2, 40: 992.2, 60: 983.2, 80: 971.8}
    for vco2, lab in ((0.04747, "V_CO2 = 0.04747 mL/mol as printed"), (47.47, "V_CO2 = 47.47 mL/mol")):
        e = []
        for t, row in t3.items():
            T = t + 273.15
            vmea = 61.08 / (-5.35162e-7 * T * T - 4.51417e-4 * T + 1.19451)
            vw = 18.015 / (rhow[t] / 1000)
            xm0 = x1
            for a, r in zip(al, row):
                nm, nw, nc = xm0, 1 - xm0, xm0 * a
                nt = nm + nw + nc
                xm, xw, xc = nm / nt, nw / nt, nc / nt
                V = xm * vmea + xw * vw + xc * vco2 + xm * xw * (-1.8218)
                M = xm * 61.08 + xw * 18.015 + xc * 44.01
                e.append((M / V * 1000) / r)
        e = np.array(e)
        print(f"{lab}: AARD {100*np.mean(abs(e-1)):.2f}% [{e.min():.3f},{e.max():.3f}]")
    # Hartono Eq 3-5 applied to measured unloaded density
    e = []
    for t, row in t3.items():
        for a, r in zip(al, row):
            if a == 0:
                continue
            m1, m2, m3 = 61.08, 18.015, 44.01
            w = a * x1 * m3 / (x1 * m1 + (1 - x1) * m2 + a * x1 * m3)
            phi = (0.29 * x1 * a + 0.18 * x1) / (0.66 + x1)
            rho = row[0] / (1 - w * (1 - phi**3))
            e.append(rho / r)
    e = np.array(e)
    print(f"Hartono Eq 3-5 with x1 CO2-free and Eq 4 form: AARD {100*np.mean(abs(e-1)):.2f}% [{e.min():.3f},{e.max():.3f}]")


def wong():
    print("\n=== Wong 2016 printed-equation audit ===")
    T = 313.15
    k1_print = -38.846 - 17.3 / T + 0.05764 * math.log(T)
    k1_alt = -38.846 - 17.3 / T + 0.05764 * T
    print(f"K1 as printed (c ln T): ln K {k1_print:.2f} pKa {-k1_print/2.3026:.2f}; with c*T: ln K {k1_alt:.2f} pKa {-k1_alt/2.3026:.2f}")
    print("K4 as printed", round(140.932 - 13445.9 / 298.15 - 22.4773 * math.log(298.15), 2), " K5 as printed", round(220.067 - 12431.7 / 298.15 - 35.4819 * math.log(298.15), 2),
          " eqn 15 needs carbonate (ln K about -23.8), eqn 16 needs water (about -32.2)")
    for T in (313.15,):
        e22 = 94.4914 + 6789.04 / T - 11.4519 * math.log(T) + 0.01045 * T
        alt = 94.4914 - 6789.04 / T - 11.4519 * math.log(T) - 0.01045 * T
        v17 = 2.82e6 * math.exp(-2044 / T) * 0.9922  # kPa kg/mol from Pa m3/mol and rho kg/L
        print(f"Eq 22 at {T}: as printed value {e22:.2f}; with minus signs ln H {alt:.3f} -> {math.exp(alt):.1f}; Versteeg Eq 17 equivalent {v17:.0f} kPa kg/mol = {v17/101.325:.1f} atm kg/mol")

    print("\n=== high pressure test: Wong Table 4 (10 mass%, pure CO2) against prototype ===")
    W = {303.15: [(1.0, 0.754), (2.3, 0.832), (3.0, 0.888), (6.1, 1.012), (10.0, 1.161), (19.8, 1.219), (30.0, 1.424), (40.1, 1.517), (50.3, 1.621)],
         313.15: [(1.0, 0.693), (2.2, 0.794), (3.2, 0.859), (6.0, 0.914), (9.3, 1.019), (20.0, 1.157), (30.0, 1.339), (40.0, 1.435), (50.0, 1.539)],
         323.15: [(1.0, 0.639), (2.0, 0.742), (3.2, 0.794), (6.0, 0.900), (10.1, 0.941), (20.1, 1.025), (30.0, 1.085), (40.0, 1.155), (50.0, 1.255)]}
    res = []
    for T, rows in W.items():
        for Pb, aexp in rows:
            P = Pb * 1e5
            pw = 0.95 * psat_w(T)
            y = max(1 - pw / P, 0.5)
            phi = v.srk_phi_co2(T, P, y)
            f = phi * y * P
            poy = math.exp(VINF * (P - pw) / (R * T))
            target_kPa = f / poy / 1e3
            try:
                a = brentq(lambda z: math.log(v.pco2(T, 0.10, z)["pco2_kPa"] / target_kPa), 0.3, 2.2, xtol=1e-3)
            except Exception:
                a = float("nan")
            res.append((T, Pb, aexp, a))
            print(f"T {T} P {Pb:5.1f} bar alpha Raman {aexp:.3f} prototype {a:.3f} diff {a-aexp:+.3f}")
    d = np.array([r[3] - r[2] for r in res if r[1] <= 10.2 and not math.isnan(r[3])])
    e = np.array([r[3] - r[2] for r in res if r[1] > 10.2 and not math.isnan(r[3])])
    print(f"P<=10 bar: mean diff {d.mean():+.3f}, MAD {abs(d).mean():.3f}, n={len(d)};  P>10 bar: mean diff {e.mean():+.3f} MAD {abs(e).mean():.3f} n={len(e)}")


def crosschecks():
    print("\n=== cross checks ===")
    for T in (298.15, 313.15, 333.15, 353.15, 393.15):
        t1 = 0.1559 + 110.80 / T   # tau(H2O,MEA)
        t2 = 1.5201 - 910.30 / T   # tau(MEA,H2O)
        lnr = t1 + t2 * math.exp(-0.2 * t2)
        print(f"T {T}: ln gamma_inf MEA, Zhang NRTL Table 8 {lnr:.3f}; Aronu UNIQUAC Tables 7-9 {v.lg_inf(T):.3f}")
    T = 298.15
    aron = eu.lnK("mea", T)
    cow = -12.4751 - 5780.9 / T + 0.0145966 * T
    print(f"ln K_MEA at 298.15: Aronu Table 2 {aron:.3f} (pKa basis); + ln gamma_inf = {aron + v.lg_inf(T):.3f}; Cownden Table 1 {cow:.3f}")
    for T in (313.15, 353.15, 393.15):
        cow = -12.4751 - 5780.9 / T + 0.0145966 * T
        print(f"  T {T}: Aronu+lngamma {eu.lnK('mea', T) + v.lg_inf(T):.3f}  Cownden {cow:.3f}; KMEACOO Aronu {eu.lnK('meacoo', T):.3f}  Cownden {-10.1504 - 270.194 / T + 0.0074803 * T:.3f}")
    print("Bates-Pinching check: 2677.91*2.302585 =", round(2677.91 * 2.302585, 2), " 0.3869*2.302585 =", round(0.3869 * 2.302585, 4), "; note: BP constants from my memory, match to C2 and C1+ln55.51 =", round(-4.9074 + math.log(55.51), 4))
    print("dG sensitivity: 1 kJ/mol at 313 K changes K by factor", round(math.exp(1000 / (8.314 * 313.15)), 3), "; MEACOO- dfG shift Table2->Table10", (4.9299e8 - 4.8994e8) / 1e6, "kJ/mol factor", round(math.exp(3050 / (8.314 * 313.15)), 2))
    print("Rate constants, k2 m3/mol/s")
    for T in (293.15, 298.15, 303.15, 313.15, 333.15):
        print(f"  T {T}: Versteeg1996 4.4e8 exp(-5400/T) {4.4e8*math.exp(-5400/T):.2f}; Bronsted Eq 20 with pKa from Aronu K {math.exp((-(eu.lnK('mea',T)+math.log(55.51))/2.302585)+16.26-7188/T):.2f}")
    print("Eq 35 check, k_OH at 298 K: with 2985:", round(10 ** (13.635 - 2985 / 298.0)), " with 2895:", round(10 ** (13.635 - 2895 / 298.0)), " text says 8322")
    print("Eq 5.26 at 40 C alpha 0.25 =", round(v.xu(313.15, 0.25), 2), "Pa")


def plant():
    print("\n=== plant conditions, 30 mass% ===")
    P = 21e5
    for t in (35, 40, 50, 60, 70):
        T = t + 273.15
        y = 0.18
        # liquid fugacity basis: p* corresponds to low P. Correct target for Poynting at 21 bar, ideal gas phase.
        poy = math.exp(VINF * (P - psat_w(T)) / (R * T))
        a0 = v.alpha_at(T, 0.30, 378.0)
        a1 = v.alpha_at(T, 0.30, 378.0 / poy)
        print(f"T {t}: alpha_eq(378 kPa) {a0:.3f}; with 21 bar Poynting {poy:.3f}: {a1:.3f}; free MEA fraction of total at alpha_eq {v.pco2(T,0.30,a0)['n'][eu.iM]:.4f}")
    print("Figure read-offs at 313 K, 30 wt%: Jou via Zhang Fig 9 alpha(100 kPa) about 0.64, alpha(378 kPa) about 0.74; Wong Fig 3 alpha(1 bar) about 0.54, alpha(3.4 bar) about 0.62")
    print("prototype alpha(100 kPa, 40 C) =", round(v.alpha_at(313.15, 0.30, 100.0), 3))
    print("\nH2 co-absorption, order of magnitude (external Henry value, not from a local source)")
    Hw = 7.0e9  # Pa, H2 in water at 25 C mole-fraction Henry, general knowledge
    for salt in (1.0, 1.7):
        x = 0.757 * 21e5 / (Hw * salt)
        L = 1450e3  # kg/h midpoint of amine_spec starting guess
        n = L * 1000 / (0.3 * 61.08 + 0.7 * 0  ) if False else None
        ntot = (1450e3 * 0.30 / 61.08 + 1450e3 * 0.70 / 18.015)
        h2 = x * ntot
        print(f"  salting factor {salt}: x_H2 {x:.2e}, dissolved {h2:.1f} kmol/h, {100*h2/7477:.2f}% of H2 feed, {100*h2/(h2+1599):.2f} mol% in CO2 if no rich flash")
    k = {25.2: (4.67, 7.94), 39.5: (6.34, 10.90), 87.1: (13.47, 25.32)}
    print("  N2O kH ratio alpha 0.5 / alpha 0 at 30 wt%:", {t: round(b / a, 2) for t, (a, b) in k.items()})
    print("  N2O kH 30 wt% unloaded / water Eq 18:", {t: round(a * 1e3 / (8.55e6 * math.exp(-2284 / (t + 273.15))), 2) for t, (a, b) in k.items()})


def aw():
    print("\n=== water activity, (Ptot - pCO2 model) vs prototype a_w Psat, 30 mass% ===")
    r = []
    for T, rows in eu.T4_MODEL.items():
        for Pt, p, a in rows:
            s = v.pco2(T, 0.30, a)
            aw_ = math.exp(s["lng"][eu.iW]) * s["x"][eu.iW]
            r.append(((Pt - p), aw_ * psat_w(T) / 1e3, T, a))
    e = np.array([b / a for a, b, _, _ in r])
    print("n", len(e), "ratio median", round(float(np.median(e)), 3), "AARD", round(float(100 * np.mean(abs(e - 1))), 1), "range", round(float(e.min()), 2), round(float(e.max()), 2))


def lean():
    print("\n=== lean end, prototype: alpha at which p* reaches treated-gas pCO2 45 kPa with driving-force margins ===")
    for t in (40, 50, 60):
        print(t, "C alpha(p*=45 kPa)", round(v.alpha_at(t + 273.15, 0.30, 45.0), 3), "alpha(p*=22 kPa)", round(v.alpha_at(t + 273.15, 0.30, 22.0), 3), "alpha(p*=9 kPa)", round(v.alpha_at(t + 273.15, 0.30, 9.0), 3))


if __name__ == "__main__":
    code_verif()
    aw()
    n2o()
    hartono_checks()
    wong()
    crosschecks()
    plant()
    lean()
