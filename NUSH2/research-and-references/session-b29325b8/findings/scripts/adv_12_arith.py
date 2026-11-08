"""Independent re-computation by the adversarial reviewer. No import of any reader script.
Every input is typed from a primary page checked in this review, or is flagged [reader]/[assumed]."""
import math, sys
sys.stdout.reconfigure(encoding="utf-8")
LB = 0.45359237
BTU_GJ = 1.05505585e-6
H = 8760 * 0.85
print("=== 1. 130 MMSCFD conversions ===")
print("hours at 85%", H, "days", 365 * 0.85)
# Standard cubic foot at 60 F, 14.696 psia, ideal gas
R = 10.7316  # psia ft3/lbmol/R
Vm_scf = R * (60 + 459.67) / 14.696
print("scf/lbmol at 60F", round(Vm_scf, 2))
n_lbmol_d = 130e6 / Vm_scf
n_lbmol_h = n_lbmol_d / 24
kmol_h = n_lbmol_h * LB
print("kmol/h", round(kmol_h, 1))
Nm3h = kmol_h * 22.414
print("Nm3/h (0 C, 1 atm, ideal)", round(Nm3h, 0), "ratio to 100,000:", round(Nm3h / 1e5, 4), "inverse", round(1e5 / Nm3h, 4))
M_H2 = 2.01588
M_CH4 = 16.0425
pure_all = kmol_h * M_H2
print("if pure H2, kg/h", round(pure_all, 1), "lb/h", round(pure_all / LB, 0))
prod = kmol_h * (0.999 * M_H2 + 0.001 * M_CH4)
print("99.9/0.1 product, kg/h", round(prod, 1), "lb/h", round(prod / LB, 0))
inl = 29000 * LB
print("INL 29,000 lb/h = kg/h", round(inl, 1), "t/y", round(inl * H / 1000, 0))
purefrac = 0.999 * M_H2 / (0.999 * M_H2 + 0.001 * M_CH4)
print("pure H2 mass fraction", round(purefrac, 4), "pure H2 in 29,000 lb/h, t/y", round(inl * purefrac * H / 1000, 0))
print("via density 0.08988, t/y", round(Nm3h * 0.08988 * H / 1000, 0))
print("econ_lit 29,000 lb/h / 0.08988 =", round(inl / 0.08988, 0), "Nm3/h (mixes product mass with pure H2 density)")
print("mass scale s = 66,925/97,946 =", round(1e5 * 0.08988 * H / 1000 / (inl * H / 1000), 4), " volume scale =", round(1e5 / Nm3h, 4))
print("100,000 Nm3/h t/y", round(1e5 * 0.08988 * H / 1000, 1))

print("\n=== 2. Deck emissions ===")
Hd = 66927
print("H2 t/y from 8.988 t/h", round(8.988 * H, 0), " 215.71 t/d x 310.25 =", round(215.71 * 310.25, 0))
base = 9.35 * Hd
comb = 3.27 * Hd
proc = 6.08 * Hd
print("baseline", round(base), "comb", round(comb), "proc", round(proc), "35% exact", round(0.35 * base))
for cap in (0.8, 0.85, 0.9, 0.95):
    res = proc * (1 - cap)
    print(" capture", cap, "captured", round(proc * cap), "residual", round(res), "net", round(base - res))
print("no CCS net", round(comb))
print("70% avail", round((base - proc * .1) * 70 / 85), "90% avail", round((base - proc * .1) * 90 / 85))
print("reduction", round((base - proc * .1) / base * 100, 2))

print("\n=== 3. Reactor sizing ===")
Q6, E6, Epurge, eta = 176.8, 17.3, 5.7, 0.397
print("E adj", round(E6 - Epurge, 1))
print("slide 6:", round(Q6 + (E6 - Epurge) / eta, 2), " th for power", round((E6 - Epurge) / eta, 2))
print("complete Case 6:", round(Q6 + E6 / eta, 2))
print("Case 5:", round(166.6 + 15.3 / 0.39, 2))
print("Case 6 at 47.5%:", round(Q6 + E6 / 0.475, 2))
print("210: power available", round((210 - Q6) * eta, 2), "short of 17.3 by", round(E6 - (210 - Q6) * eta, 2))
print("230 unallocated", round(230 - (Q6 + E6 / eta), 1), round((230 - (Q6 + E6 / eta)) / (Q6 + E6 / eta) * 100, 1), "%")
print("v1_7: 184.9 + 25.1 =", 184.9 + 25.1, " 11.8/25.1 =", round(11.8 / 25.1, 3))
print("Case 6 row sum", round(1.2 + 2.1 + .3 + 6.4 + 5.7 + .1 + 1.5, 1))
print("He: 78.49*5.193*434/1000 =", round(78.49 * 5.193 * 434 / 1000, 1), "implied cp", round(176.8e3 / (78.49 * 434), 4))
print("He excess: 78.49*5.193*5/1000 =", round(78.49 * 5.193 * 5 / 1000, 2))
print("INL 0.30 x 600 =", 0.3 * 600, " 178.8/600 =", round(178.8 / 600, 3))
print("Case 5 He 630342 lb/h = kg/s", round(630342 * LB / 3600, 2), " duty", round(630342 * LB / 3600 * 5.193 * (1562 - 835) / 1.8 / 1000, 1))

print("\n=== 4. Natural gas ===")
hhv = 1044
def gj(mmscfd):
    return mmscfd * 1e6 * hhv * BTU_GJ
print("9.0 MMSCFD GJ/d", round(gj(9.0), 2), " 18.5:", round(gj(18.5), 1), " 10.5:", round(gj(10.5), 1))
d = 310.25
for x in (9.0, 18.5, 10.5):
    print(f" {x} MMSCFD S$M/y", round(gj(x) * 15 * d / 1e6, 2), " x0.6833:", round(gj(x) * 15 * d / 1e6 * 66927 / 97946, 2))
print("Savings: 34/52.5", round((1 - 34 / 52.5) * 100, 1), " 34/49.2", round((1 - 34 / 49.2) * 100, 1), " 43.5/49.2", round((1 - 43.5 / 49.2) * 100, 1))
print("Purge 5.7 MWe x 7446 h x S$0.20/kWh = S$M/y", round(5.7e3 * H * 0.2 / 1e6, 2))
print("H2/NG: Case 6", round(130 / 34, 2), "  no recycle 130/44.5", round(130 / 44.5, 2), "  Case 2", round(130 / 43.5, 2))
# Independent once-through yield check
C_per = 1.05379
conv = 0.78
for psa in (0.88,):
    # H2 made per mol NG tube feed, mostly CH4; 3 per CH4 reformed + shift of ~96% of CO formed, plus heavier HC
    h2 = (conv * C_per * 3.0 + conv * C_per * 0.96 + (C_per - 1) * 1.0)
    print("Once-through H2 scf/scf gas (conv 0.78, shift .96, PSA .88):", round(h2 * psa, 2), " implies NG MMSCFD", round(130 / (h2 * psa), 1))
print("Stoich ceiling with full reforming, H2/NG:", round(4 * C_per + 0.0, 2), "Case 6 uses", round(3.82 / (4 * C_per) * 100, 1), "% of ceiling")
# linear fit Cases 2-5 NG vs RECY-2
import statistics as st
recy2 = [2174, None, None, 22790]
ng = [43.5, 40.8, 37.7, 34.5]
recy1 = [2244, 8012, 14743, 23358]
xb = sum(recy1) / 4
yb = sum(ng) / 4
b = sum((x - xb) * (y - yb) for x, y in zip(recy1, ng)) / sum((x - xb) ** 2 for x in recy1)
a = yb - b * xb
print("fit NG vs RECY-1: intercept", round(a, 2), "slope per 1000 lb/h", round(b * 1000, 3), "; NG=34.0 needs RECY-1", round((34.0 - a) / b, 0))

print("\n=== 5. Recycle compression in Case 6 ===")
kw = [398.3, 1409.0, 2542.0, 3943.5]
for f, w in zip(recy1, kw):
    print(f" RECY-1 {f} lb/h, {w} kW, spec {w / f:.4f} kW/(lb/h)")
for lt in (29786, 31000, 32000, 33770):
    print(f" Case 6 all reject {lt} lb/h at 0.165-0.169: {lt * .165 / 1e3:.2f}-{lt * .169 / 1e3:.2f} MW")
print(" lb/h that absorbs all 5.7 MW at 0.1688:", round(5700 / 0.1688))
print(" Case 5 row:", round(3943.5 + 609.7 + 188.2 + 2.7 + .7, 1), "kW; Case 6 NG-COMP scaled", round(609.7 * 34 / 34.5, 1))

print("\n=== 6. MEA heat ===")
for cap_ty, lab in ((524096, "v1_7"), (526544, "REC_F"), (366224, "deck")):
    tph = cap_ty / H
    print(lab, "t/h", round(tph, 2), [round(tph * s / 3.6, 1) for s in (3.4, 3.8, 3.9, 4.05, 4.2)])
sel = 65.0 * 1e6 * BTU_GJ * 1e3 / 3.6 / 1e0
print("Selexol 65.0 MMBtu/h MW", round(65.0 * 0.29307, 2), " x 1927/1970", round(65.0 * 0.29307 * 1927 / 1970, 2))
print("Selexol GJ/t", round(65.0 * 1.05506 / (164385 * LB / 1000), 3))
print("Export 83.7 MMBtu/h MW", round(83.7 * .29307, 2), " ZLD 56.7:", round(56.7 * .29307, 2))
stack = 18.6 + 24.5 + 11.2
tph = 526544 / H
for s in (3.4, 3.8, 4.2):
    r = tph * s / 3.6
    print(f" SRD {s}: reb {r:.1f}; gap with EX4 {r - stack:.1f}; gap no EX4 {r - 18.6 - 24.5:.1f}; gap with ZLD freed {r - stack - 16.3:.1f}; gap with no export steam {r - 18.6 - 11.2:.1f}")

print("\n=== 7. Screening walk ===")
E = 28.95
gap = 20.2
r = 0.169
w = min(E, r * gap)
br = gap + w + (E - w) / eta
print("branch", round(br, 2))
tot = 176.8 + 1.3 + 1.4 + 0.45 + 3.4 + br - 9.8
print("reactor", round(tot, 1))
print("Circulator shaft check: (5.10+5.81)*0.9 =", round((5.10 + 5.81) * .9, 2))
print("If no reboiler/circulator double: 261.5 - 0.6833 scale:", round(261.5 * .6833, 1), " with 0.6891:", round(261.5 * .6891, 1))

print("\n=== 8. Secondary circulator on INL's own 60 psi ===")
m = 78.49
cp = 5193
gam = 5 / 3
T1 = 466 + 273.15
P1, P2 = 955.0, 1015.0
ws = m * cp * T1 * ((P2 / P1) ** ((gam - 1) / gam) - 1) / 1e6
print("isentropic MW", round(ws, 2), " at eta .75 shaft", round(ws / .75, 2), " MWe at motor .9", round(ws / .75 / .9, 2))
print("Screening central dP 200 kPa = psi", round(200 / 6.89476, 1), " INL 60 psi = kPa", round(60 * 6.89476, 0))
dE = ws / .75 / .9 - 5.10
print("Extra electricity vs central", round(dE, 2), "MWe; net reactor effect", round(dE / eta - (ws / .75 - 5.10 * .9), 1), "MWth")

print("\n=== 9. Deck cost ledger ===")
abat = 0.585075
ceil = abat * 100
ccs = 366224 * 80 * 1.3 / 1e6
print("ceiling", round(ceil, 2), "CCS", round(ccs, 2), " US$M", round(366224 * 80 / 1e6, 2))
crf = lambda n: .08 / (1 - 1.08 ** -n)
print("CRF", [round(crf(n), 4) for n in (25, 40, 60)])
cap = 420 * 1.3
print("Reactor S$M", cap, "annual", [round(cap * crf(n), 2) for n in (25, 40, 60)])
av = gj(9.0) * 15 * d / 1e6 * 66927 / 97946
print("gas avoided", round(av, 2), "budget", round(ceil - ccs + av, 2), "after reactor", round(ceil - ccs + av - cap * crf(25), 2))
print("S$/t 25y:", round((ccs + cap * crf(25) - av) / abat, 1))
print("candidate (38.09+45.8-31.5)/0.544 =", round((38.09 + 45.8 - 31.5) / .544, 2))
print("same at 0.585:", round((38.09 + 45.8 - 31.5) / .585, 2), "; 57.79/0.60 =", round(57.79 / .6, 2))
ccs80 = 366224 * 8 / 9 * 80 * 1.3 / 1e6
print("consistent 80% capture: CCS", round(ccs80, 2), " 25y", round((ccs80 + 51.2 - 31.5) / .544, 1), " 40y", round((ccs80 + 45.8 - 31.5) / .544, 1))
print("100 - 3.725 =", 100 - 3.725)
av34 = gj(18.5) * 15 * d / 1e6 * 66927 / 97946
print("gas at 34.0 scaled", round(av34, 1), " S$/t", round((ccs + 51.2 - av34) / abat, 1), " +O&M24.1+IHX11.4:", round((ccs + 51.2 + 24.1 + 11.4 - av34) / abat, 1), " at 40y:", round((ccs + 45.8 + 24.1 + 11.4 - av34) / abat, 1))
print("Learning units to reach BOAK from 1.6:", round(2 ** (math.log(1 / 1.6) / math.log(.9)), 1))
print("5250 x 80000 =", 5250 * 80000 / 1e6, " x1.6 =", 5250 * 80000 / 1e6 * 1.6, " per kWt at 200 MWth", 420e6 / 200e3)
print("X-energy 3600 x 320 MW = US$M", 3600 * 320e3 / 1e6, " per 80 MWe", 3600 * 80e3 / 1e6, " 9200 x 80 MWe", 9200 * 80e3 / 1e6, " 9200x 200 MWth at .4", 9200 * .4 * 200e3 / 1e6)

print("\n=== 10. INL cost recipe ===")
occ = 6000 * .91 * .4
print("OCC central kWt", occ, " low", 4000 * .91 * .4, " high", 7000 * .91 * .4)
print("FOAK", occ * 1.6, " 0.9^log2(10) =", round(.9 ** math.log2(10), 4))
print("10-OAK via BOAK (p46 route)", round(occ * .9 ** math.log2(10), 0), " via FOAK (p47 route)", round(occ * 1.6 * .9 ** math.log2(10), 0))
F = 1.6195
P = 261.5
print("Reactor capital n=.6, S$M", round(occ * F * 600e3 * (P / 600) ** .6 / 1e6, 1), "annual", round(occ * F * 600e3 * (P / 600) ** .6 / 1e6 * crf(25), 1))
print("Reactor capital n=1, S$M", round(occ * F * P * 1e3 / 1e6, 1), "annual", round(occ * F * P * 1e3 / 1e6 * crf(25), 1))
print("OPEX n=1 S$M/y", round(10 * F * P * H / 1e6, 1), " n=.6 staff scaled", round(10 * F * P * H / 1e6 * (.4438 + .5562 * (P / 600) ** -.4), 1))
A = 0.846
L = dict(gas=-94.8, rc=120.8, cc=13.5, fom=3.9, ts=31.6, om=38.4, ihx=12.2, lic=6.1, dec=1.9)
L0 = L['gas'] + L['rc'] + L['cc'] + L['fom'] + L['ts']
print("L0", round(L0, 1), round(L0 / A, 1), " L1", round((L0 + L['om']) / A, 1), " L2", round((L0 + L['om'] + L['ihx']) / A, 1), " L3", round((L0 + L['om'] + L['ihx'] + L['lic'] + L['dec']) / A, 1))
rc1 = occ * F * P * 1e3 / 1e6 * crf(25)
om1 = 10 * F * P * H / 1e6
for nm, lifecrf in (("25y", crf(25)), ("40y", crf(40)), ("60y", crf(60))):
    rc = occ * F * P * 1e3 / 1e6 * lifecrf
    x0 = L['gas'] + rc + L['cc'] + L['fom'] + L['ts']
    print(f"n=1 {nm}: L0 {x0 / A:.1f}, L1 {(x0 + om1) / A:.1f}, L1+IHX {(x0 + om1 + L['ihx']) / A:.1f}, L3 {(x0 + om1 + L['ihx'] + L['lic'] + L['dec']) / A:.1f}")
print("share of S$/t from unverified IHX + licensing at Case A:", round((L['ihx'] + L['lic']) / A, 1))

print("\n=== 11. Ahn anchor at 100,000 Nm3/h ===")
print("gas saving", round((1 - 24816 / 30563) * 100, 2), "%")
print("MW per t/h H2: Ahn", round(143.4 / 9.0, 2), " INL", round(176.8 / 13.154, 2), " ratio", round((143.4 / 9.0) / (176.8 / 13.154), 3))
print("kg NG/kg H2: Ahn", round(24816 / 9000, 3), " INL Case 6", round(34.0 * 65046 / 34.5 * LB / 13154, 3))
print("Case B process heat scaled INL", round(176.8 * .6833, 1), " Ahn 143.4, delta", round(143.4 - 176.8 * .6833, 1))
print("Case B central 179.0 + delta =", round(179.0 + 143.4 - 176.8 * .6833, 1), " surplus vs 210:", round(210 - (179.0 + 143.4 - 176.8 * .6833), 1))
print("Deck method at Ahn heat: 143.4 + 11.6*.6833/.397 =", round(143.4 + 11.6 * .6833 / .397, 1), "; + 17.3*.6833/.397 =", round(143.4 + 17.3 * .6833 / .397, 1))

print("\n=== 12. Carbon ===")
ngC = 34e6 / Vm_scf * 1.05379 * 44.0095 / 2000
print("feed C as st/d CO2", round(ngC, 1), " captured+emitted", 1927 + 142, " closure", round(2069 / ngC * 100, 2))
print("t/t H2:", round(ngC * .90718474 / (inl * 24 / 1000), 3), " deck 6.08 over by", round((6.08 / (ngC * .90718474 / (inl * 24 / 1000)) - 1) * 100, 2), "%")
print("1927/.99 =", round(1927 / .99, 1), " 2069*.9=", round(2069 * .9, 1), " .9*1946.5 =", round(.9 * 1946.5, 1))
print("3205 st/d t/y", round(3205 * .90718474 * d, 0), " kg/kg", round(3205 * .90718474 / (inl * 24 / 1000), 3))
print("35.4%:", round((3205 - 2069) / 3205 * 100, 1))
print("Abat: baseline-emission REC_F", 902060 - 56066)
print("0.9 per pass, recycle limit: if slip recycles fully and only cryo fraction lost")
