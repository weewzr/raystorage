"""Own calculation: back-check XF Table 6 from Table 5, confidence-interval symmetry,
and XF Part 2 Table 2 feed bookkeeping. All inputs are printed values from the papers."""
import math

R = 8.314e-3  # kJ/mol/K
t5 = {
    "k1": (1.842e-4, 240.1, 648, 4.225e15),
    "k2": (7.558, 67.13, 648, 1.955e6),
    "k3": (2.193e-5, 243.9, 648, 1.020e15),
}
for k, (v, E, Tr, A) in t5.items():
    calc = v * math.exp(E / (R * Tr))
    print(f"{k}: A from Table5 = {calc:.4g}; Table 6 = {A:.4g}; ratio = {calc/A:.3f}")

K5 = {
    "KCO": (40.91, -70.65, 648, 8.23e-5),
    "KH2": (0.02960, -82.90, 648, 6.12e-9),
    "KCH4": (0.1791, -38.28, 823, 6.65e-4),
    "KH2O": (0.4152, 88.68, 823, 1.77e5),
}
for k, (v, H, Tr, A) in K5.items():
    calc = v * math.exp(H / (R * Tr))
    print(f"{k}: A from Table5 = {calc:.4g}; Table 6 = {A:.4g}; ratio = {calc/A:.3f}")

print("\nCI symmetry check (value, UL, LL, mid of UL/LL)")
ci = {
    "k1": (1.842e-4, 2.05e-4, 1.64e-4),
    "k2": (7.558, 8.200, 6.915),
    "k3": (2.193e-5, 2.60e-5, 1.78e-5),
    "KCO": (40.91, 44.65, 37.17),
    "KH2": (0.02960, 0.0533, 0.0059),
    "KCH4": (0.1791, 0.2211, 0.1371),
    "KH2O": (0.4152, 0.5032, 0.0317),
    "E1": (240.1, 244.5, 235.7),
    "E2": (67.13, 72.06, 62.19),
    "E3": (243.9, 255.0, 232.7),
    "dHCO": (-70.65, -64.61, -76.66),
    "dHH2": (-82.90, -52.61, -113.2),
    "dHCH4": (-38.28, -20.89, -55.66),
    "dHH2O": (88.68, 96.06, 81.26),
}
for k, (v, ul, ll) in ci.items():
    print(f"{k}: value {v:.5g} mid {(ul+ll)/2:.5g} symmetric LL would be {2*v-ul:.5g} printed LL {ll:.5g}")

print("\nln A and entropy back-check")
for k, A, printed_ln, dS in (("CO", 8.23e-5, -9.40, 77.12), ("H2", 6.12e-9, -18.90, 157.0), ("CH4", 6.65e-4, -7.32, 60.81)):
    print(k, "lnA", round(math.log(A), 3), "printed", printed_ln, "R ln A J/mol/K", round(8.314 * math.log(A), 2), "printed |dS|", dS)

print("\nPart 2 Table 2 feed bookkeeping")
Nm3 = 22.414
ng = 135.00 / Nm3
st = 399.17 / Nm3
comp = {"CH4": (0.815, 1), "N2": (0.141, 0), "C2H6": (0.028, 2), "CO2": (0.010, 0), "C3H8": (0.004, 3), "C4H10": (0.001, 4), "C5H12": (0.002, 5)}
print("sum comp", sum(v[0] for v in comp.values()))
carbon = sum(ng * y * n for (y, n) in comp.values())
print(f"NG kmol/h {ng:.3f}, steam kmol/h {st:.3f}, HC carbon kmol/h {carbon:.3f}")
eq = 5.168
print("CO2 equiv kmol/h", 0.056 * eq, "H2 equiv", 0.122 * eq, "N2 equiv", 0.164 * eq, "N2 from comp", ng * 0.141)
print("H2O equiv kmol/h", 3.358 * eq, "steam consumed", st - 3.358 * eq)
print("carbon: CH4eq + CO2eq", eq + 0.056 * eq, "vs C in", carbon + ng * 0.010)

print("\nPart 2: active volume-equivalent slab factor check")
dpe, dpi, H, t = 0.0173, 0.0084, 0.010, 0.002
print("ring H/dpe etc", H / dpe)

print("\nFig 3 approximate readouts -> conversion")
for label, y in (("x_CH4/(1+zeta) ~0.57", 0.57), ("x_CO2/(1+zeta) ~0.33", 0.33)):
    print(label, "x =", y * 1.056)

print("\nDuty implied by XF Part 2 Table 2 as a per-tube check (own calc, rough)")
# heat of reaction at ~948 K Table 7: 224.0 (I), -37.3 (II), 187.5 (III)
x_ch4 = 0.60
x_co2 = 0.35
F = 5.168
rxI = F * (x_ch4 - x_co2)
rxIII = F * x_co2
print("kmol/h I,III (treating CH4->CO by I and CH4->CO2 by III)", rxI, rxIII)
Q = (rxI * 224.0 + rxIII * 187.5) * 1000 / 3600
print("reaction duty kW per tube", Q)
