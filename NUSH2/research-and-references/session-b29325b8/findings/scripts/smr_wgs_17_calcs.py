"""Own calculations for the smr_wgs evidence report. All inputs come from printed values in
the named sources. Results are screening checks, not design values."""
import math, sys
sys.stdout.reconfigure(encoding="utf-8")
from scipy.optimize import brentq  # noqa


def K1_bar2(T):
    # Zhang 2021 Table 1: 1.198e11 exp(-26830/T) MPa2 = 1.198e13 bar2
    return 1.198e13 * math.exp(-26830.0 / T)


def K2(T, c=1.767e-2):
    return c * math.exp(4400.0 / T)


def equil(T, Pbar, sc, x0=None, ninert=0.0):
    """CH4 + H2O = CO + 3H2 ; CO + H2O = CO2 + H2. Feed 1 CH4, sc H2O, ninert N2. T in K."""
    K1 = K1_bar2(T)
    K2v = K2(T)

    def f2(x, y):
        # given reform extent x and shift extent y
        ch4 = 1 - x
        h2o = sc - x - y
        co = x - y
        h2 = 3 * x + y
        co2 = y
        return ch4, h2o, co, h2, co2

    def solve_y(x):
        lo, hi = x * 1e-12, x * (1 - 1e-12)
        def g(y):
            ch4, h2o, co, h2, co2 = f2(x, y)
            return co2 * h2 - K2v * co * h2o
        return brentq(g, lo, hi, xtol=1e-14)

    def h(x):
        y = solve_y(x)
        ch4, h2o, co, h2, co2 = f2(x, y)
        n = ch4 + h2o + co + h2 + co2 + ninert
        return (co * h2 ** 3) * (Pbar / n) ** 2 - K1 * ch4 * h2o

    x = brentq(h, 1e-6, 0.99999)
    y = solve_y(x)
    ch4, h2o, co, h2, co2 = f2(x, y)
    n = ch4 + h2o + co + h2 + co2 + ninert
    return x, y, dict(CH4=ch4 / n, H2O=h2o / n, CO=co / n, H2=h2 / n, CO2=co2 / n)


print("== 1. Constant conversion Hoseinzade vs XF")
f = 2.246
for nm, A, conv, printed in (("k1", 4.225e15, 10, 9.49e16), ("k2", 1.955e6, 1 / 100, 4.39e4), ("k3", 1.020e15, 10, 2.29e16)):
    print(nm, "XF to kPa basis", A * conv, "x2.246", A * conv * f, "Hoseinzade", printed, "ratio", printed / (A * conv))
for nm, E in (("E1", 240.1), ("E2", 67.13), ("E3", 243.9), ("dHCH4", 38.28), ("dHH2O", 88.68), ("dHH2", 82.90), ("dHCO", 70.65)):
    print(nm, "E/R with R=8.314", E * 1000 / 8.314)
print("ln(1.767e-2)", math.log(1.767e-2), "ln(1.176e-2)", math.log(1.176e-2), "e^-4.063", math.exp(-4.063), "e^-4.036", math.exp(-4.036), "e^30.11", math.exp(30.11), "e^30.114", math.exp(30.114))
print("101.325^2", 101.325 ** 2)
print("van't Hoff from K forms: dH1", 26830 * 8.314 / 1000, "dH2", 4400 * 8.314 / 1000, "XF Table 7 948 K: 224.0, -37.3")

print("\n== 2. KH2O trap at a sample point")
T = 1000.0
P = 30.0
y = dict(CH4=0.15, H2O=0.45, H2=0.25, CO=0.03, CO2=0.12)
R = 8.314e-3
KCO = 8.23e-5 * math.exp(70.65 / R / T)
KH2 = 6.12e-9 * math.exp(82.90 / R / T)
KCH4 = 6.65e-4 * math.exp(38.28 / R / T)
for A in (1.77e5, 1.77e3):
    KW = A * math.exp(-88.68 / R / T)
    p = {k: v * P for k, v in y.items()}
    DEN = 1 + KCO * p["CO"] + KH2 * p["H2"] + KCH4 * p["CH4"] + KW * p["H2O"] / p["H2"]
    print(f"A(KH2O)={A:.3g} KH2O={KW:.3g} term={KW * p['H2O'] / p['H2']:.3g} DEN={DEN:.3g} DEN2={DEN ** 2:.4g}")

print("\n== 3. JAERI mock-up tests: S/C, conversion, equilibrium comparison")
tests = {
    "2002-01-31 (A-5)": dict(H2=65.5, N2=0.99, CO=6.89, CO2=11.46, CH4=15.16, mc=11.96, ms=None, tip=None, h=119.64),
    "2002-11-28 (A-2)": dict(H2=66.6, N2=0.58, CO=6.74, CO2=11.48, CH4=14.6, mc=12.08, ms=48.0, tip=755, h=122.7),
    "2003-07-01 (A-3)": dict(H2=66.6, N2=0.51, CO=7.31, CO2=11.34, CH4=14.24, mc=11.91, ms=45.9, tip=760, h=122.8),
    "2004-11-15 (A-4)": dict(H2=65.8, N2=0.3, CO=6.1, CO2=11.7, CH4=16.1, mc=11.86, ms=46.9, tip=740, h=114.8),
}
for nm, t in tests.items():
    X = (t["CO"] + t["CO2"]) / (t["CO"] + t["CO2"] + t["CH4"])
    XCO = t["CO2"] / (t["CO"] + t["CO2"])
    H2CH4 = None
    line = f"{nm}: X={X:.3f} X_CO2frac={XCO:.3f} sumdry={t['H2'] + t['N2'] + t['CO'] + t['CO2'] + t['CH4']:.2f}"
    if t["ms"]:
        sc = (t["ms"] / 18.015) / (t["mc"] / 16.04)
        xe, ye, c = equil(t["tip"] + 273.15, 41.0, sc)
        # approach temperature: T at which equilibrium conversion equals measured X
        Ta = brentq(lambda TT: equil(TT, 41.0, sc)[0] - X, 700, 1400)
        # H2 from balance
        F = t["mc"] / 16.04  # mol/s
        H2mol = F * X * (t["H2"] / (t["CO"] + t["CO2"]))  # dry ratio H2 to converted C
        Nm3h = H2mol * 22.414e-3 * 3600
        # WGS approach at tip
        wet = None
        line += f" S/C={sc:.2f} Xeq(tip,41bar)={xe:.3f} Tapproach={Ta - 273.15:.0f}C (tip {t['tip']}) dT={t['tip'] - (Ta - 273.15):.0f} H2 from GC={Nm3h:.1f} printed {t['h']}"
    print(line)

print("sensitivity of Xeq to pressure for 2002-11-28: ")
sc = (48 / 18.015) / (12.08 / 16.04)
for Pb in (40.5, 41.2, 42):
    print(Pb, round(equil(755 + 273.15, Pb, sc)[0], 3))

print("\n== 4. Literature equilibrium checks")
for nm, TC, Pb, sc, printed in (
    ("INL conventional 1600F, S/C 3 (pressure 30.3 bar Case 5)", 871.1, 30.3, 3.0, 78.1),
    ("INL Case 6 per v1.7 Table A1 871C 31.7 bar", 871.1, 31.7, 3.0, None),
    ("Ahn 875C 26 bar S/C 2.7", 875, 26, 2.7, None),
    ("Yin HTR-10 design 840C 3.2 MPa S/C 4", 840, 32, 4.0, 78),
    ("Hoseinzade PNP 853C(1126K) 5.6 MPa S/C 4", 852.85, 56, 4.0, 73),
    ("XF commercial about 1050K? 26 bar S/C 3.358", 1050 - 273.15, 26, 3.358, 60),
    ("TECDOC T2-2 800C 41.9 bar S/C 4.05", 800, 41.9, 80.05 / (19.77 + 2 * 0.17), 66.1),
    ("TECDOC T2-2 860C 41.9 bar S/C 4.05", 860, 41.9, 80.05 / (19.77 + 2 * 0.17), 79.8),
    ("TECDOC T2-1 800C 20 bar S/C 4.05", 800, 20, 80.05 / (19.77 + 2 * 0.17), 81.3),
):
    x, ys, c = equil(TC + 273.15, Pb, sc)
    print(f"{nm}: Xeq={x * 100:.1f}% printed {printed}; wet y: " + " ".join(f"{k}={v * 100:.1f}" for k, v in c.items()))

print("\n== 5. Effect of the hot-end approach for the project basis (S/C 3, 30.3 bar, pure CH4 equilibrium)")
for TC in (871, 850, 825, 800, 775, 760):
    x, ys, c = equil(TC + 273.15, 30.3, 3.0)
    print(TC, "He-T", 900 - TC, f"Xeq={x * 100:.1f}%", f"CH4 dry={c['CH4'] / (1 - c['H2O']) * 100:.1f}%")

print("\n== 6. WGS approach to equilibrium, IEAGHG stream 6 and INL")
Q = 0.1283 * 0.5961 / (0.0366 * 0.2137)
print("IEAGHG Q", Q, "K(412C) Zhang", K2(412 + 273.15), "K with 4.063", math.exp(4400 / 685.15 - 4.063), "ratio Q/K", Q / K2(685.15), "T where K=Q", 4400 / math.log(Q / 1.767e-2) - 273.15)
Qi = 0.0492 * 0.5171 / (0.1156 * 0.2927)
print("IEAGHG inlet Q", Qi, "CO conversion", 1 - 0.0366 / 0.1156)
print("Equilibrium inlet stream 5 at given T: ")
y0 = dict(CO=0.1156, H2O=0.2927, CO2=0.0492, H2=0.5171)
for TC in (412, 425, 215, 200):
    K = K2(TC + 273.15)
    fn = lambda e: (y0["CO2"] + e) * (y0["H2"] + e) - K * (y0["CO"] - e) * (y0["H2O"] - e)
    e = brentq(fn, 0, y0["CO"] - 1e-9)
    dry = 1 - (y0["H2O"] - e)
    print(TC, "K", round(K, 2), "CO wet", round(y0["CO"] - e, 4), "CO dry %", round((y0["CO"] - e) / dry * 100, 2))

print("\n== 7. Case 5 cooling train in MW (1 MMBtu/h = 0.293071 MW)")
c = 0.293071
items = {"HPS-GEN1 871>335C": 236.3, "WGS-EX1 407>249C": 66.7, "WGS-EX2 249>191C": 24.2, "WGS-EX3 213>149C": 65.7, "WGS-EX4 149>70C": 117.1, "KO-DRM-1 70>40C": 19.7}
tot = 0
for k, v in items.items():
    print(k, v, round(v * c, 1), "MW")
    tot += v
print("total", tot, tot * c)
print("reactor heat to REFORM-1", 404.2 * c, "preheats", (61.3 + 36.3 + 36.4 + 14.3 + 5.0 + 10.9), (61.3 + 36.3 + 36.4 + 14.3 + 5.0 + 10.9) * c, "sum", 404.2 + 164.2, "fired stage 2", 74 * c)
print("temps F>C", [round((f - 32) / 1.8, 1) for f in (635, 764, 480, 375, 415, 300, 158, 104, 1600, 1517, 1562, 835, 1045, 331, 332, 769, 418)])
print("pressures psi>bar", [round(p * 0.0689476, 1) for p in (439, 442, 445, 460, 436, 419, 413, 396, 390, 389, 1015, 945)])
print("mass lb/h>t/h", [round(m * 0.45359237 / 1000, 2) for m in (349724, 223172, 126553, 113607, 12945, 29000, 164385, 630342)])
print("condensate per t H2", 126553 / 29000)
print("Dew 331F", (331 - 32) / 1.8)

print("\n== 8. Helium balance checks")
cp = 127.6 / (0.091 * (880 - 610))
print("cp He from JAERI Table 3.3 case B:", cp, "kJ/kg/K; case A:", 108.7 / (0.091 * 230))
for nm, m, a, b in (("Case 6", 78.49, 900, 466), ("Case 5", 79.42, 850, 446)):
    print(nm, m * 5.193 * (a - b) / 1000, "MW with 5.193")
print("206.0 check", 176.8 + 11.6 / 0.397)

print("\n== 9. Scale ratios")
H2_t_h_inl = 29000 * 0.45359237 / 1000
print("INL H2 t/h", H2_t_h_inl, "Nm3/h (130 MMSCFD)", 130e6 * 0.0283168 * (273.15 / 288.71) * 101325 / 101325 / 24)
print("heat per t/h: INL Case 6", 176.8 / H2_t_h_inl, "Ahn", 143.4 / 9.0)
print("Ahn scaled to 13.15 t/h", 143.4 * H2_t_h_inl / 9, "INL scaled to 8.988 t/h", 176.8 * 8.988 / H2_t_h_inl)
print("mock-up per-tube: H2 122.7 Nm3/h per tube, tubes for 146,000 Nm3/h", 130e6 * 0.0283168 * (273.15 / 288.71) / 24 / 122.7)
print("JAERI heat flux on outer area", 94.9 / 6.98, "kW/m2 (reactor tube) ; He-given 114.4/6.98", 114.4 / 6.98)
print("A required at 40-60 kW/m2 for 176.8 MW", 176.8e3 / 60, 176.8e3 / 40, "for Case-5-like 70.9%", 0.711 * 176.8e3 / 60, 0.711 * 176.8e3 / 40)
print("tubes of 6.98 m2", 0.711 * 176.8e3 / 50 / 6.98)
print("PNP scaling 72.9 MW for 199 tubes -> tubes per MW", 199 / 72.9, "for 125.7 MW", 0.711 * 176.8 * 199 / 72.9, "for 176.8", 176.8 * 199 / 72.9)

print("\n== 10. Ahn vs deck")
print("NG reduction", 1 - 24816 / 30563, "dLCOH", 2.37 - 1.78, "dGWP", 12.91 - 3.50, "US$/t", (2.37 - 1.78) / (12.91 - 3.50) * 1000, "S$ at 1.30", (2.37 - 1.78) / (12.91 - 3.50) * 1000 * 1.30)
print("direct-only dGWP (9.10 - 4% of 3.50)", 9.10 - 0.04 * 3.5, "US$/t", 0.59 / (9.10 - 0.14) * 1000)
print("HTGR cost per year US$M", 0.64 * 9000 * 8322 / 1e6, "per MWth", 0.64 * 9000 * 8322 / 1e6 / 143.4)
print("Availability", 8322 / 8760, "deck 7446 h")
print("Deck residual kg/kg", 40692 / 66927, "baseline t/y", 9.35 * 66927, "direct abatement kg/kg", 585075 / 66927)
print("Ahn GWP abatement on deck H2", (12.91 - 3.50) * 66927 / 1e6, "Mt; direct only", 8.96 * 66927 / 1e6, "Mt")
print("Ahn upstream NG on deck H2", 3.09 * 66927 / 1e6, "Mt/y")
print("upstream factor kgCO2e per kg NG implied gray", 3.80 * 9000 / 30563, "h-SMR", 3.09 * 9000 / 24816)
print("v1.7 upstream 11.5 kgCO2e/GJ on Case6 34.0 MMSCFD", 34.0e6 * 1044 * 1055.06e-9 * 365 * 0.85 * 11.5 / 1000, "t/y", "per kg H2 (97946 t/y)", 34.0e6 * 1044 * 1055.06e-9 * 365 * 0.85 * 11.5 / 1000 / 97946)
print("NG energy GJ/kg H2 Case 6", 34.0e6 * 1044 * 1055.06e-9 / (13.154 * 24000))

print("\n== 11. Water saturation of knock-out gas (IEAGHG stream 7)")
print("p H2O kPa", 0.0024 * 2580, "amine_spec 5.63 kPa at 35 C -> y", 5.63 / 2580, "ratio", 0.0024 * 2580 / 5.63)
print("IEAGHG condensate kg/h by mass", 101667 - 69711, "by H2O", (8370.3 * 0.2137 - 6596.9 * 0.0024) * 18.015, "kmol change", 8370.3 - 6596.9, 8370.3 * 0.2137 - 6596.9 * 0.0024)
print("per t H2", (101667 - 69711) / 8994, "H2O in vs out %", 1 - 6596.9 * 0.0024 / (8370.3 * 0.2137))
print("carbon in PSA feed as CO, CH4 fraction", 0.0464 / (0.0464 + 0.0302 + 0.1627), 0.0302 / (0.0464 + 0.0302 + 0.1627), (0.0464 + 0.0302) / (0.0464 + 0.0302 + 0.1627))
# INL case 5 dry after LTS: CO 0.33
print("Case 5 raw syngas S/DG", 33.4 / (100 - 33.4))
print("Case 5 CO to CO2 total (C basis), HTS exit dry 3.01 > LTS 0.33: extra H2+CO2 per 100 dry mol", 3.01 - 0.33)
