"""Scoping check, not a model. Two questions for the engine plan:
1. How far does Zhang Table 1 K(T) sit from K(T) built from the NIST Shomate data the energy balance will use?
2. If process heat is computed from enthalpies on the INL anchors, how close does it land to INL's published heat?
Shomate coefficients are those already in the old repo (lib.rs), which cites NIST SRD 69. Heavier hydrocarbons are lumped as
CH4 carbon-equivalents for this scoping check only. Stream data: inl.md s7.3, s3.3; smr_wgs.md s5.3."""
import math

SH = {
 "CH4": [(298, 1300, -0.703029, 108.4773, -42.52157, 5.862788, 0.678565, -76.84376, 158.7163, -74.87310)],
 "CO":  [(298, 1300, 25.56759, 6.096130, 4.054656, -2.671301, 0.131021, -118.0089, 227.3665, -110.5271)],
 "CO2": [(298, 1200, 24.99735, 55.18696, -33.69137, 7.948387, -0.136638, -403.6075, 228.2431, -393.5224),
         (1200, 6000, 58.16639, 2.720074, -0.492289, 0.038844, -6.447293, -425.9186, 263.6125, -393.5224)],
 "H2":  [(298, 1000, 33.066178, -11.363417, 11.432816, -2.772874, -0.158558, -9.980797, 172.707974, 0.0),
         (1000, 2500, 18.563083, 12.257357, -2.859786, 0.268238, 1.977990, -1.147438, 156.288133, 0.0)],
 "H2O": [(500, 1700, 30.09200, 6.832514, 6.793435, -2.534480, 0.082139, -250.8810, 223.3967, -241.8264)],
 "N2":  [(500, 2000, 19.50583, 19.88705, -8.598535, 1.369784, 0.527601, -4.935202, 212.3900, 0.0)],
}
HF = {"CH4": -74.873, "CO": -110.527, "CO2": -393.522, "H2": 0.0, "H2O": -241.826, "N2": 0.0}

def row(sp, T):
    for r in SH[sp]:
        if r[0] <= T <= r[1]:
            return r
    raise ValueError((sp, T))

def H(sp, T):
    """Total molar enthalpy, kJ/mol, formation basis."""
    _, _, a, b, c, d, e, f, g, h = row(sp, T)
    t = T / 1000
    return HF[sp] + a*t + b*t*t/2 + c*t**3/3 + d*t**4/4 - e/t + f - h

def Sm(sp, T):
    _, _, a, b, c, d, e, f, g, h = row(sp, T)
    t = T / 1000
    return a*math.log(t) + b*t + c*t*t/2 + d*t**3/3 - e/(2*t*t) + g

def G(sp, T):
    return H(sp, T) - T * Sm(sp, T) / 1000

def K_nist(T):
    R = 8.314462e-3
    d1 = G("CO", T) + 3*G("H2", T) - G("CH4", T) - G("H2O", T)
    d2 = G("CO2", T) + G("H2", T) - G("CO", T) - G("H2O", T)
    return math.exp(-d1/(R*T)), math.exp(-d2/(R*T))

def K_zhang(T):
    return 1.198e13 * math.exp(-26830/T), 1.767e-2 * math.exp(4400/T)

print("T K, K1 Zhang (bar2), K1 NIST (1 bar std), ratio, K2 Zhang, K2 NIST, ratio")
for T in (680.0, 800.0, 950.0, 1073.15, 1098.15, 1144.15):
    z1, z2 = K_zhang(T); n1, n2 = K_nist(T)
    print(f"{T:8.2f} {z1:12.4g} {n1:12.4g} {z1/n1:6.3f}   {z2:8.4f} {n2:8.4f} {z2/n2:6.3f}")
for T in (500.0, 680.0):
    z1, z2 = K_zhang(T)
print("note: NIST H2O Shomate starts at 500 K; K2 below that needs the low-T water enthalpy route")

# Case 5 anchors, kmol/h. Outlet from raw syngas composition and SG-2 mass; inlet from NG, recycle, water.
LB = 0.45359237
MW = {"H2": 2.016, "CO": 28.01, "CO2": 44.01, "N2": 28.013, "H2O": 18.015, "CH4": 16.043}
y5 = {"H2": .488, "CO": .087, "CO2": .048, "N2": .007, "H2O": .334, "CH4": .036}
Mbar = sum(y5[k]*MW[k] for k in y5)
n5 = 349724*LB/Mbar
out5 = {k: y5[k]*n5 for k in y5}
print(f"\nCase 5 outlet total {n5:.0f} kmol/h, Mbar {Mbar:.3f}, carbon {out5['CH4']+out5['CO']+out5['CO2']:.0f} kmol/h")

# Inlet by element balance on the outlet: all carbon as CH4 eq. plus CO2, H from H2O + CH4 + recycled H2.
# Steam+water from flowsheet; recycled H2 3,077 lb/h; recycled CO small.
water = (248376 + 13513)*LB/18.015
h2_rec = 3077*LB/2.016
co2_in = 0.0025*34.5e6/379.49/24*LB + 0.012*1155   # NG CO2 plus recycle CO2, kmol/h
co_in = 0.022*1155
ch4_in = out5["CH4"] + out5["CO"] + out5["CO2"] - co2_in - co_in
in5 = {"CH4": ch4_in, "CO": co_in, "CO2": co2_in, "H2": h2_rec, "H2O": water, "N2": out5["N2"]}
el = lambda d: (d["CH4"]+d["CO"]+d["CO2"], 4*d["CH4"]+2*d["H2"]+2*d["H2O"], d["CO"]+2*d["CO2"]+d["H2O"])
print("element in  C H O", [round(v) for v in el(in5)])
print("element out C H O", [round(v) for v in el(out5)])

Tout = (1600-32)/1.8 + 273.15
for Tin_f, lab in ((718, "NG-9, before preheat"), (1000, "NG-10, reformer inlet")):
    Tin = (Tin_f-32)/1.8 + 273.15
    q = (sum(out5[k]*H(k, Tout) for k in out5) - sum(in5[k]*H(k, Tin) for k in in5)) / 3.6e3   # kmol/h * kJ/mol = MJ/h ; /3600 = MW
    print(f"Case 5 enthalpy rise, {lab} {Tin_f} F to 1600 F: {q:.1f} MW")
print("Published Case 5: helium process heat 166.6 MW (568.4 MMBtu/h), plus fired Stage 2. REFORM-1 404.2 MMBtu/h = 118.5 MW; preheat 164.2 MMBtu/h = 48.1 MW.")
print("Burner Q 74.0 MMBtu/h = 21.7 MW.")

# Case 6 scoped by the same recipe: carbon to tubes from REC_F/INL_SEL, S/C 3, same conversion ratio as Case 5 outlet.
for lab, tube_c, h2_rec6 in (("Case 6 INL_SEL", 2157.2, None), ("Case 6 REC_F", 2303.7, None)):
    f = tube_c / (out5["CH4"]+out5["CO"]+out5["CO2"])
    o = {k: v*f for k, v in out5.items()}
    i = {k: v*f for k, v in in5.items()}
    Tin = (718-32)/1.8 + 273.15
    q = (sum(o[k]*H(k, Tout) for k in o) - sum(i[k]*H(k, Tin) for k in i)) / 3.6e3
    print(f"{lab}: tube carbon {tube_c}, scoped Q from 718 F {q:.1f} MW, published 176.8 MW, ratio {q/176.8:.3f}")
