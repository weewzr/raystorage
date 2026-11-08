"""Own calculations for the HTGR/IHX evidence report. Inputs are published values;
every number produced here is an OWN CALCULATION and is labelled so in the report."""
import math

CP = 5195.0          # J/kg/K, Petersen (1970) Riso-224 eq. 4-4
CP_INL = 5190.0      # implied by TEV-961 (computed in section A)
R_He = 2077.0        # J/kg/K
GAMMA = 5.0 / 3.0


def lmtd(dt1, dt2):
    if abs(dt1 - dt2) < 1e-9:
        return dt1
    return (dt1 - dt2) / math.log(dt1 / dt2)


def mu(T):            # Pa s, Petersen eq. 6-1
    return 3.674e-7 * T ** 0.7


def k(T, Pbar):       # W/m/K, Petersen eq. 7-1
    return 2.682e-3 * (1 + 1.123e-3 * Pbar) * T ** (0.71 * (1 - 2e-4 * Pbar))


def rho(T, Pbar):     # kg/m3, Petersen eq. 3-19
    return 48.14 * Pbar / T / (1 + 0.4446 * Pbar / T ** 1.2)


print("=== A. Helium energy balance, 78.49 kg/s, 900->466 C")
m, dT = 78.49, 434.0
for lab, c in [("5193.2 (5R/2M, own)", 2.5 * 8.314462 / 4.002602e-3), ("5195 Petersen", 5195.0), ("5190 implied", 5190.0), ("5.19 kJ/kgK (final_v1_7)", 5190.0), ("5200 repo", 5200.0)]:
    print("  cp=%-26s Q=%.2f MW" % (lab, m * c * dT / 1e6))
print("  implied cp from 176.8: %.1f J/kg/K" % (176.8e6 / (m * dT)))
lo = 176.75e6 / (78.495 * 434.5)
hi = 176.85e6 / (78.485 * 433.5)
print("  rounding band for implied cp: %.0f to %.0f" % (lo, hi))
print("  178.8 MWth check: m*cp*(900-461) = %.2f MW (cp 5190)" % (m * 5190 * 439 / 1e6))
print("  2.0 MWth excess check: m*cp*(466-461) = %.2f MW" % (m * 5190 * 5 / 1e6))
print("  With text's 3 K instead of 5 K: %.2f MW" % (m * 5190 * 3 / 1e6))

print("\n=== B. TEV-961 Table 2 reconstruction (cp=5190)")
cases = [("725", 83.62, 700, 431, 303, 55.6, 172.5), ("775", 79.78, 750, 431, 343, 36.5, 168.7),
         ("825", 79.21, 800, 437, 382, 22.6, 171.7), ("875", 79.42, 850, 446, 421, 10.3, 176.9),
         ("925", 78.49, 900, 466, 461, 2.0, 178.8)]
for n, md, ts, tr, ti, ex, rs in cases:
    q = md * 5190 * (ts - tr) / 1e6
    e = md * 5190 * (tr - ti) / 1e6
    r = md * 5190 * (ts - ti) / 1e6
    print("  ROT %s: process %.1f; excess %.1f (pub %.1f); reactor %.1f (pub %.1f, err %+.2f%%); return-ideal diff %d K" % (n, q, e, ex, r, rs, (r - rs) / rs * 100, tr - ti))

print("\n=== C. Published reactor energy-balance closure m*cp*dT vs stated power (cp=5195)")
rows = [("HTTR 30 MW measured (T3-21) +0.81 VCS", 12.3, 392, 844, 30.0, 0.81), ("HTTR rated analysis (T3-1)", 12.4, 395, 850, 30.0, 0.0),
        ("HTR-10", 4.3, 250, 700, 10.0, 0), ("HTR-Modul el", 85.5, 250, 700, 200.0, 0), ("HTR-Modul PH", 50.3, 300, 950, 170.0, 0),
        ("GT-MHR", 320, 491, 850, 600.0, 0), ("H2-MHR", 320, 590, 950, 600.0, 0), ("GTHTR300C", 322, 594, 950, 600.0, 0),
        ("MHR-100 SMR", 81.7, 450, 950, 215.0, 0), ("SC-MHR", 157.1, 322, 750, 350.0, 0), ("SC-HTGR", 281.9, 325, 750, 625.0, 0),
        ("HTR-PM (web)", 96, 250, 750, 250.0, 0)]
for lab, md, ti, to, p, extra in rows:
    q = md * CP * (to - ti) / 1e6 + extra
    print("  %-40s calc %.1f  stated %.1f  err %+.1f%%" % (lab, q, p, (q - p) / p * 100))
print("  Xe-100 web: 71.1 kg/s is stated with 9.6%% bypass; 71.1/0.904=%.1f kg/s gives %.1f MW" % (71.1 / 0.904, 71.1 / 0.904 * CP * 490 / 1e6))
print("  GTHTR300C IHX: 322 kg/s * cp * (950-850) = %.1f MW; if 950->556 = %.1f MW (> reactor 600 MW)" % (322 * CP * 100 / 1e6, 322 * CP * 394 / 1e6))
print("  168 MW with secondary 500->900: m = %.1f kg/s" % (168e6 / (CP * 400)))

print("\n=== D. HTTR and concept IHX performance")
A_httr = 96 * math.pi * 0.0318 * 22.2
print("  HTTR tube area: 96*pi*0.0318*22.2 = %.1f m2 (21.5-22.9 m range %.0f-%.0f)" % (A_httr, 96 * math.pi * .0318 * 21.5, 96 * math.pi * .0318 * 22.9))
sets = [
    ("HTTR design Table 2 Tochio", 3.333, 950, 390, 3.333, 300, 860, A_httr),
    ("HTTR LP test 30 MW T3-23 meas", 4.1, 828, 383, 3.6, 248, 751, A_httr),
    ("HTTR LP test 15 MW T3-23 meas", 4.1, 468, 235, 3.6, 158, 420, A_httr),
    ("HTTR LP analysis 30 MW T3-12", 4.1, 850, 395, 3.6, 241, 783, A_httr),
    ("HTTR HP-11 Fig9 readoff", None, 815, 270, None, 115, 733, A_httr),
    ("HTTR 2004 NP-T-4.3 hot end", None, 941, None, None, None, 859, A_httr),
    ("HTTR-SR 950/905 Sato Fig12", 3.4, 950, None, 2.519, 149, 905, A_httr),
    ("HTR-10 IHX T5-5 He/N2", 3.21, 900, 600, None, 483, 850, 153.0),
    ("H2-MHR S-I T19", 320, 950, 590, 320, 565, 925, None),
    ("GTHTR300C (hot 950/900)", 322, 950, 850, 80.8, 500, 900, None),
]
for s in sets:
    lab, mp, tpi, tpo, ms, tsi, tso, A = s
    hot = tpi - tso
    if tpo is None and mp is not None and ms is not None:
        q = ms * CP * (tso - tsi)
        tpo = tpi - q / (mp * CP)
    if tpo is None or tsi is None:
        print("  %-34s hot-end approach %d K (no cold-end data)" % (lab, hot))
        continue
    cold = tpo - tsi
    L = lmtd(hot, cold)
    qp = mp * CP * (tpi - tpo) if mp else None
    qs = ms * CP * (tso - tsi) if ms else None
    q = qs if qs else qp
    if lab.startswith("HTTR HP-11"):
        q = 85e3 * L
    ua = q / L
    if ms is None and mp is None:
        c_min = None
    elif ms is None:
        c_min = None
    else:
        c_min = min(mp, ms) * CP
    eps = (max(tpi - tpo, tso - tsi)) / (tpi - tsi)
    txt = "  %-34s hot %.0f cold %.0f LMTD %.1f Q %.2f MW UA %.1f kW/K eps %.3f" % (lab, hot, cold, L, q / 1e6, ua / 1e3, eps)
    if qp and qs:
        txt += " balance %+.1f%%" % ((qp - qs) / qs * 100)
    if c_min:
        txt += " NTU %.2f" % (ua / c_min)
    if A:
        txt += " U %.0f W/m2K" % (ua / A)
    print(txt)
print("  Tochio Fig 8/10 UA 85 kW/K -> U = %.0f W/m2K on OD area" % (85e3 / A_httr))
print("  Tochio HP-11 hot 82 cold 155: LMTD %.1f; Q at UA 85 = %.2f MW" % (lmtd(82, 155), 85e3 * lmtd(82, 155) / 1e6))
print("  HTTR 950/905 UA requirement vs measured 85 kW/K: see line above")

print("\n=== E. NUS-H2 IHX requirement")
Qlist = [176.8, 206.0, 210.0]
for Q in Qlist:
    for appr in [25, 45, 50, 82]:
        ua = Q * 1e6 / appr
        print("  Q=%.1f balanced approach %d K: UA %.2f MW/K = %.0f x HTTR(85 kW/K); NTU %.1f; A@U400 %.0f m2 (%.0f HTTR tubes); A@U760 %.0f m2" % (
            Q, appr, ua / 1e6, ua / 85e3, ua / (78.49 * CP), ua / 400, ua / 400 / (A_httr / 96), ua / 760))
print("  Effectiveness 900-466 over 925-466 = %.3f; HTTR 751-248 over 828-248 = %.3f" % (434 / 459, 503 / 580))
print("  ... unbalanced primary, Q=176.8, hot end 25 K, sec 466->900")
for Tpo in [491, 500, 525, 550, 600]:
    mp = 176.8e6 / (CP * (925 - Tpo))
    cold = Tpo - 466
    L = lmtd(25, cold)
    print("    primary out %d: mp %.1f kg/s (ratio %.2f), cold-end %d K, LMTD %.1f, UA %.2f MW/K (%.0f x HTTR), A@U400 %.0f, A@U760 %.0f" % (
        Tpo, mp, mp / 78.49, cold, L, 176.8e6 / L / 1e6, 176.8e6 / L / 85e3, 176.8e6 / L / 400, 176.8e6 / L / 760))
print("  Duty ratios: 176.8/10 = %.1f; 210/10 = %.1f; 176.8/168 = %.3f" % (17.68, 21.0, 176.8 / 168))
print("  GTHTR300C draft A=1448 m2: LMTD(50,350)=%.1f, U=%.0f W/m2K, UA=%.2f MW/K, NTU=%.2f" % (lmtd(50, 350), 170e6 / lmtd(50, 350) / 1448, 170e6 / lmtd(50, 350) / 1e6, 170e6 / lmtd(50, 350) / (81 * CP)))
print("  NUS-H2 UA / GTHTR300C UA = %.1f" % ((176.8e6 / 25) / (170e6 / lmtd(50, 350))))

print("\n=== F. Reactor inlet implied")
for Trin in [461, 486, 491]:
    print("  reactor inlet %d: flow at 210 MWth %.1f kg/s; at 178.8 MWth %.1f kg/s" % (Trin, 210e6 / (CP * (925 - Trin)), 178.8e6 / (CP * (925 - Trin))))

print("\n=== G. Temperature cascade to reformer (871 C outlet target)")
for rot in [925, 950]:
    for dih in [25, 45, 50, 82]:
        for dd in [0, 5, 25]:
            t = rot - dih - dd
            print("  ROT %d IHX %d duct %d -> He at reformer %d, pinch to 871 = %d K" % (rot, dih, dd, t, t - 871))
print("  ROT for 900 at reformer: 25+0=925; 25+5=930; 25+25=950; 50+0=950; 45+25=970; 82+25=1007")
print("  Precedent reformer hot-end pinch: HTTR-SR 880-830=50; HTR-10 890-840=50; HTR-Modul 950-810=140; MHR-100 950-870=80; NUS-H2 900-871=29")

print("\n=== H. Hot-duct heat loss")
qd = 2.519 * CP * 25
print("  HTTR-SR 905->880 at 2.519 kg/s: %.0f kW = %.1f%% of 10 MW" % (qd / 1e3, qd / 1e5))
print("  Sato loop closure: 2.519*cp*(905-149) = %.2f MW" % (2.519 * CP * 756 / 1e6))
for lab, ql in [("same dT 25 K", 78.49 * CP * 25), ("area scaled (flow^0.5, same length)", qd * math.sqrt(78.49 / 2.519))]:
    print("  NUS-H2 duct loss %s: %.2f MW, dT %.1f K" % (lab, ql / 1e6, ql / (78.49 * CP)))

print("\n=== I. Vessel loss")
print("  HTTR VCS measured 0.81/30 = %.2f%%; design 0.6/30 = %.1f%%" % (0.81 / 30 * 100, 2.0))
print("  Scaled proportional to 210: %.2f MW; surface scaled (P^2/3): %.2f MW" % (0.0270 * 210, 0.81 * (210 / 30) ** (2 / 3)))

print("\n=== J. Circulator work")
print("  HTTR-SR secondary circulator rise 133->150 C at 2.519 kg/s: %.0f kW = %.2f%% of 10 MW" % (2.519 * CP * 17 / 1e3, 2.519 * CP * 17 / 1e5))
ex = (GAMMA - 1) / GAMMA
print("  loop: secondary 78.49 kg/s, T1 739 K, 5.15 MPa; primary 92.2 kg/s, T1 764 K, 5.0 MPa")


def wshaft(md, T1, P_kPa, dP_kPa, eta):
    pr = (P_kPa + dP_kPa) / P_kPa
    return md * CP * T1 * (pr ** ex - 1) / eta, pr


for dP in [100, 150, 200, 300, 500]:
    for eta in [0.65, 0.80]:
        ws, pr = wshaft(78.49, 739, 5150, dP, eta)
        wp, pr2 = wshaft(92.2, 764, 5000, dP, eta)
        print("  dP %d kPa eta %.2f: PR %.3f, secondary %.2f MW, primary %.2f MW, total %.2f MW, per Q176.8 %.1f%%" % (dP, eta, pr, ws / 1e6, wp / 1e6, (ws + wp) / 1e6, (ws + wp) / 176.8e4))
print("  HTTR circulator PR up to 1.065 at 4 MPa = dP %.0f kPa" % (0.065 * 4000))
print("  Sato-scaled 2.2%% of 176.8 = %.1f MW; with T1 739/406 = %.1f MW" % (0.0222 * 176.8, 0.0222 * 176.8 * 739 / 406))
print("  JAERI reformer He dP: 0.36 kgf/cm2 = %.1f kPa; calc 0.26 = %.1f; smooth 0.104 = %.1f" % (0.36 * 98.0665, 0.26 * 98.0665, 0.104 * 98.0665))
print("  HTTR core dP 0.12 kgf/cm2 = %.1f kPa" % (0.12 * 98.0665))

print("\n=== K. Reactor size sensitivity")
ETA_C, ETA_M = 0.397, 0.90


def rsize(Eaux, W_sh, qves, qduct):
    Pcyc = (Eaux + W_sh / ETA_M) / ETA_C
    return 176.8 + Pcyc + qves + qduct - W_sh


print("  base                     %.1f" % rsize(11.6, 0, 0, 0))
print("  +vessel 2.96             %.1f" % rsize(11.6, 0, 2.96, 0))
print("  +vessel 5.67             %.1f" % rsize(11.6, 0, 5.67, 0))
print("  +vessel 2.96 duct 1.8    %.1f" % rsize(11.6, 0, 2.96, 1.82))
for W in [3, 5, 7, 10, 15]:
    print("  W_shaft %2d, vessel 2.96, duct 1.82 -> %.1f MWth; electric %.1f MWe" % (W, rsize(11.6, W, 2.96, 1.82), 11.6 + W / ETA_M))
print("  with full 17.3 MWe: %.1f" % (176.8 + 17.3 / ETA_C))
print("  margin 210-206.02 = %.2f MWth, %.1f%%" % (210 - 206.02, (210 - 206.02) / 206.02 * 100))
print("  W_shaft that exhausts margin with no losses: %.2f MW" % (3.98 / (1 / (ETA_M * ETA_C) - 1)))

print("\n=== L. Life")
h = 20 * 8760 * 0.6
print("  HTTR 20 y at 60%% = %.0f h; at 7446 h/y = %.1f y" % (h, h / 7446))

print("\n=== M. Report/final power split")
print("  25.1*0.47=%.2f; 25.1*0.397=%.2f; 184.9-176.8=%.1f (%.1f%%); 11/0.47=%.1f; 11.6/0.397=%.2f" % (25.1 * .47, 25.1 * .397, 8.1, 8.1 / 176.8 * 100, 11 / .47, 11.6 / .397))
print("  230 basis: 178.8+16.506/0.397=%.2f" % (178.8 + 16.506 / 0.397))

print("\n=== N. Helium properties at IHX conditions (Petersen)")
for T, P in [(1173, 51.5), (1198, 51.5), (955, 51.5), (739, 51.5), (1173, 91), (873, 40)]:
    print("  T %d K P %.1f bar: rho %.3f mu %.3e k %.3f Pr %.3f" % (T, P, rho(T, P), mu(T), k(T, P), CP * mu(T) / k(T, P)))

print("\n=== O. Tube-side screening at HTTR geometry")
di, do, L = 0.0248, 0.0318, 22.2
for Pbar, ms_tube, lab in [(40, 3.333 / 96, "HTTR"), (51.5, 3.333 / 96, "NUS-H2 same tube flow")]:
    Tm = 873
    G = ms_tube / (math.pi / 4 * di ** 2)
    Re = G * di / mu(Tm)
    Pr = CP * mu(Tm) / k(Tm, Pbar)
    Nu = 0.023 * Re ** 0.8 * Pr ** 0.4
    h = Nu * k(Tm, Pbar) / di
    f = 0.184 * Re ** -0.2
    dp = f * L / di * G ** 2 / (2 * rho(Tm, Pbar))
    print("  %s: G %.1f Re %.0f Nu %.0f h_i %.0f W/m2K, straight-tube dP %.1f kPa" % (lab, G, Re, Nu, h, dp / 1e3))
print("  Tubes at 78.49 kg/s and HTTR tube flow: %.0f" % (78.49 / (3.333 / 96)))
print("  NTU per HTTR tube at measured U: %.2f; required 17.0 => length factor %.1f" % (85e3 / (3.333 * CP), (176.8e6 / 25 / (78.49 * CP)) / (85e3 / (3.333 * CP))))

print("\n=== P. Reformer run sheet implied cp and loss (JAERI A-2-a)")
print("  127.6/(0.091*270) = %.3f; 114.4/(0.091*242) = %.3f kJ/kg/K" % (127.6 / (0.091 * 270), 114.4 / (0.091 * 242)))
print("  Heat loss 41.5/(163.0+5.1) = %.1f%%; 24.6/(140.3+5.1) = %.1f%%" % (41.5 / 168.1 * 100, 24.6 / 145.4 * 100))
print("\n=== Q. Emission factors")
print("  15.3 tC/TJ*44/12 = %.1f kgCO2/GJ (LHV, EU ETS table); 0.0507 t/GJ HHV; 50.7/0.0507" % (15.3 * 44 / 12))
print("  14.47 kgC/MMBtu = %.2f kgC/GJ HHV -> %.1f kgCO2/GJ" % (14.47 / 1.05506, 14.47 / 1.05506 * 44 / 12))
