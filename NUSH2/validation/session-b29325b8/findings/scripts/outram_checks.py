"""Independent checks. Nothing from the clones is executed.

Part A evaluates the published helium Helmholtz EOS. The coefficients were read
from crates/outram-park-fork-coolprop/src/fluids/helium.rs (SPDX MIT, generated
from CoolProp Helium.json). The evaluation code below is written from the
standard Helmholtz relations and is mine.

Part B solves SMR plus WGS equilibrium three ways at the DOVER base case:
  a) constant volumetric flow with constant-dH, dS K values (DOVER's assumptions)
  b) isobaric ideal gas with the same K values
  c) isobaric ideal gas with K1, K2 from Hoseinzade and Adams (2017) eq. 51-53
"""
import math
import sys

sys.stdout.reconfigure(encoding="utf-8")

# --------------------------------------------------------------------- Part A
R_MOL = 8.3144598
M = 0.004002602
RS = R_MOL / M
T_R = 5.1953
RHO_R = 17383.7

PN = [0.015559018, 3.0638932, -4.2420844, 0.054418088, -0.18971904, 0.087856262,
      2.2833566, -0.53331595, -0.53296502, 0.99444915, -0.30078896, -1.6432563]
PT = [1.0, 0.425, 0.63, 0.69, 1.83, 0.575, 0.925, 1.585, 1.69, 1.51, 2.9, 0.8]
PD = [4.0, 1.0, 1.0, 2.0, 2.0, 3.0, 1.0, 1.0, 3.0, 2.0, 2.0, 1.0]
PL = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 1.0, 2.0, 1.0]
GN = [0.8029102, 0.026838669, 0.04687678, -0.14832766, 0.03016211, -0.019986041,
      0.14283514, 0.007418269, -0.22989793, 0.79224829, -0.049386338]
GT = [1.26, 3.51, 2.785, 1.0, 4.22, 0.83, 1.575, 3.447, 0.73, 1.634, 6.13]
GD = [2.0, 1.0, 2.0, 1.0, 1.0, 3.0, 2.0, 2.0, 3.0, 2.0, 2.0]
GE = [1.5497, 9.245, 4.76323, 6.3826, 8.7023, 0.255, 0.3523, 0.1492, 0.05, 0.1668, 42.2358]
GEP = [0.596, 0.3423, 0.761, 0.9747, 0.5868, 0.5627, 2.5346, 3.6763, 4.5245, 5.039, 0.959]
GB = [0.2471, 0.0983, 0.1556, 2.6782, 2.7077, 0.6621, 0.1775, 0.4821, 0.3069, 0.1758, 1357.6577]
GG = [3.15, 2.54505, 1.2513, 1.9416, 0.5984, 2.2282, 1.606, 3.815, 1.61958, 0.6407, 1.076]


def ar(d, t):
    s = 0.0
    for n, ti, di, l in zip(PN, PT, PD, PL):
        e = math.exp(-(d ** l)) if l > 0 else 1.0
        s += n * d ** di * t ** ti * e
    for n, ti, di, eta, eps, beta, gam in zip(GN, GT, GD, GE, GEP, GB, GG):
        s += n * d ** di * t ** ti * math.exp(-eta * (d - eps) ** 2 - beta * (t - gam) ** 2)
    return s


def derivs(d, t):
    h_d = 1e-4 * max(d, 1e-3)
    h_t = 1e-4 * t
    f = ar
    a_d = (f(d + h_d, t) - f(d - h_d, t)) / (2 * h_d)
    a_t = (f(d, t + h_t) - f(d, t - h_t)) / (2 * h_t)
    a_dd = (f(d + h_d, t) - 2 * f(d, t) + f(d - h_d, t)) / h_d ** 2
    a_tt = (f(d, t + h_t) - 2 * f(d, t) + f(d, t - h_t)) / h_t ** 2
    a_dt = (f(d + h_d, t + h_t) - f(d + h_d, t - h_t) - f(d - h_d, t + h_t) + f(d - h_d, t - h_t)) / (4 * h_d * h_t)
    return a_d, a_t, a_dd, a_tt, a_dt


def state(T, p):
    t = T_R / T
    rho = p / (R_MOL * T)
    for _ in range(60):
        d = rho / RHO_R
        a_d, _, a_dd, _, _ = derivs(d, t)
        fx = rho * R_MOL * T * (1 + d * a_d) - p
        dfx = R_MOL * T * (1 + 2 * d * a_d + d * d * a_dd)
        step = fx / dfx
        rho -= step
        if abs(step) < 1e-9 * rho:
            break
    d = rho / RHO_R
    a_d, a_t, a_dd, a_tt, a_dt = derivs(d, t)
    Z = 1 + d * a_d
    # Ideal part: d2(a0)/dtau2 = -1.5/tau^2 for monatomic helium (LogTau a = 1.5).
    cv_r = 1.5 - t * t * a_tt
    cp_r = cv_r + (1 + d * a_d - d * t * a_dt) ** 2 / (1 + 2 * d * a_d + d * d * a_dd)
    # Residual enthalpy on top of the ideal 2.5 R T.
    h_res = R_MOL * T * (t * a_t + d * a_d)
    return dict(rho_mass=rho * M, Z=Z, cp=cp_r * RS, h_res_J_kg=h_res / M)


cp0 = 2.5 * RS
print("# Part A: helium, ideal gas against the Ortiz-Vega form EOS coefficients")
print(f"R_specific = {RS:.2f} J/kg/K, ideal cp0 = 2.5 R = {cp0:.1f} J/kg/K")
print("| T C | p MPa | Z | cp J/kg/K | cp/cp0-1 % | rho kg/m3 | ideal-gas rho error % |")
print("|---|---|---|---|---|---|---|")
for Tc in (466.0, 900.0, 925.0):
    for pm in (3.0, 5.0, 7.0, 9.0):
        T = Tc + 273.15
        s = state(T, pm * 1e6)
        rho_ig = pm * 1e6 / (RS * T)
        print(f"| {Tc:.0f} | {pm:.0f} | {s['Z']:.5f} | {s['cp']:.1f} | {100*(s['cp']/cp0-1):+.3f} | "
              f"{s['rho_mass']:.4f} | {100*(rho_ig/s['rho_mass']-1):+.3f} |")

print()
print("Secondary helium mass flow for the user duty 176.8 MWth, 900 C to 466 C:")
Q = 176.8e6
dT = 900.0 - 466.0
print(f"  ideal gas: m = {Q/(cp0*dT):.3f} kg/s")
for pm in (3.0, 5.0, 7.0, 9.0):
    a = state(900 + 273.15, pm * 1e6)
    b = state(466 + 273.15, pm * 1e6)
    dh = cp0 * dT + a["h_res_J_kg"] - b["h_res_J_kg"]
    print(f"  EOS at {pm:.0f} MPa: dh = {dh/1e3:.1f} kJ/kg, m = {Q/dh:.3f} kg/s, "
          f"ideal-gas error {100*((Q/(cp0*dT))/(Q/dh)-1):+.3f} %")

# --------------------------------------------------------------------- Part B
print()
print("# Part B: SMR + WGS equilibrium, DOVER base case 1123.15 K, 20 bar, S/C 3")
R = 8.31446261815324
T = 1123.15
P_BAR = 20.0
P_PA = 20.0e5
P0 = 1.0e5
SC = 3.0

Hf = dict(CH4=-74600.0, H2O=-241800.0, CO=-110500.0, CO2=-393500.0, H2=0.0)
S = dict(CH4=186.3, H2O=188.8, CO=197.7, CO2=213.8, H2=130.7)
dH1 = Hf["CO"] + 3 * Hf["H2"] - Hf["CH4"] - Hf["H2O"]
dS1 = S["CO"] + 3 * S["H2"] - S["CH4"] - S["H2O"]
dH2 = Hf["CO2"] + Hf["H2"] - Hf["CO"] - Hf["H2O"]
dS2 = S["CO2"] + S["H2"] - S["CO"] - S["H2O"]


def k_const(dH, dS, T):
    return math.exp(-(dH - T * dS) / (R * T))


def k1_hoseinzade(T):
    # eq. 51, kPa^2 -> bar^2 by dividing by 1e4
    return 10266.76 * math.exp(-26830.0 / T + 30.11) / 1.0e4


def k2_hoseinzade(T):
    return math.exp(4400.0 / T - 4.063)  # eq. 52


def flows(x1, x2):
    return dict(CH4=1 - x1, H2O=SC - x1 - x2, CO=x1 - x2, CO2=x2, H2=3 * x1 + x2)


def solve(q1_fun, q2_target):
    """Nested bisection. WGS extent solved for each reforming extent."""
    def x2_of(x1):
        lo, hi = 0.0, x1 * (1 - 1e-12)
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            f = flows(x1, mid)
            q = f["CO2"] * f["H2"] / (f["CO"] * f["H2O"])
            if q > q2_target:
                hi = mid
            else:
                lo = mid
        return 0.5 * (lo + hi)

    lo, hi = 1e-9, 1 - 1e-9
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        x2 = x2_of(mid)
        if q1_fun(mid, x2) > 0:
            hi = mid
        else:
            lo = mid
    x1 = 0.5 * (lo + hi)
    return x1, x2_of(x1)


K1c, K2c = k_const(dH1, dS1, T), k_const(dH2, dS2, T)
K1h, K2h = k1_hoseinzade(T), k2_hoseinzade(T)
print(f"dH1 = {dH1/1e3:.1f} kJ/mol, dS1 = {dS1:.1f} J/mol/K; dH2 = {dH2/1e3:.1f}, dS2 = {dS2:.1f}")
print(f"K1 constant dH,dS = {K1c:.2f} bar2, K2 = {K2c:.4f}")
print(f"K1 Hoseinzade eq.51 = {K1h:.1f} bar2, K2 eq.52 = {K2h:.4f}")
print(f"ratio K1 Hoseinzade / constant = {K1h/K1c:.2f}; K2 ratio = {K2h/K2c:.3f}")

Q_in = (1 + SC) * R * T / P_PA
Kc1 = K1c * (P0 / (R * T)) ** 2


def res_fixed_q(x1, x2):
    f = flows(x1, x2)
    c = {k: v / Q_in for k, v in f.items()}
    return c["CO"] * c["H2"] ** 3 / (c["CH4"] * c["H2O"]) - Kc1


def res_isobaric(K1):
    def fn(x1, x2):
        f = flows(x1, x2)
        tot = sum(f.values())
        p = {k: v / tot * P_BAR for k, v in f.items()}
        return p["CO"] * p["H2"] ** 3 / (p["CH4"] * p["H2O"]) - K1
    return fn


cases = [
    ("a fixed Q, constant dH,dS (DOVER model)", res_fixed_q, K2c),
    ("b isobaric, constant dH,dS", res_isobaric(K1c), K2c),
    ("c isobaric, Hoseinzade K1,K2", res_isobaric(K1h), K2h),
]
print("| case | X_CH4 | WGS extent | H2 per CH4 | y_H2 wet |")
print("|---|---|---|---|---|")
for name, fn, k2 in cases:
    x1, x2 = solve(fn, k2)
    f = flows(x1, x2)
    print(f"| {name} | {x1:.6f} | {x2:.6f} | {f['H2']:.4f} | {f['H2']/sum(f.values()):.4f} |")
print("DOVER test note states 0.529272 at V = 2e5 m3, 20 bar, S/C = 3 (tests/smr_cstr.rs line 279).")

# Pressure sensitivity for the corrected case.
print()
print("Case c at other temperatures, 20 bar, S/C 3:")
for Tc in (750.0, 800.0, 850.0, 900.0):
    Tk = Tc + 273.15
    fn = res_isobaric(k1_hoseinzade(Tk))
    x1, x2 = solve(fn, k2_hoseinzade(Tk))
    kc_ = k_const(dH1, dS1, Tk)
    print(f"  {Tc:.0f} C: K1 = {k1_hoseinzade(Tk):.1f} bar2 vs constant {kc_:.1f} (x{k1_hoseinzade(Tk)/kc_:.1f}), X_CH4 = {x1:.4f}")
