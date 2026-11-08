"""Own calculations for the old_repo report. Inputs are quoted with their source in comments."""
import math

ST = 0.90718474  # t per short ton
print("== Elemental carbon check at plant boundary (own calculation) ==")
# TEV-953 PDF p.39 natural gas composition, mol %
comp = {"CH4": (93.571, 1, 16.043), "C2H6": (3.749, 2, 30.070), "C3H8": (0.920, 3, 44.097), "C4H10": (0.260, 4, 58.123),
        "C5H12": (0.040, 5, 72.150), "C6H14": (0.010, 6, 86.177), "N2": (1.190, 0, 28.013), "O2": (0.010, 0, 31.999), "CO2": (0.250, 1, 44.010)}
tot = sum(v[0] for v in comp.values())
c_per_mol = sum(v[0] * v[1] for v in comp.values()) / tot
mw = sum(v[0] * v[2] for v in comp.values()) / tot
print("mol% sum", tot, "C/mol", round(c_per_mol, 5), "MW", round(mw, 3))
ton_per_mmscf = 1189.0 / 52.5  # TEV-953 PDF p.39: 1189 ton/d at 52.5 MMSCFD
for name, mmscfd, cap, em in [("Conv no CC", 52.5, 0, 3205), ("Conv CC", 49.2, 2143, 855), ("725C ROT CC", 43.5, 2123, 525), ("775", 40.8, 2076, 403),
                              ("825", 37.7, 2024, 267), ("875", 34.5, 1970, 126), ("925C ROT Case 6", 34.0, 1927, 142)]:
    tpd = ton_per_mmscf * mmscfd
    lbmol = tpd * 2000 / mw
    co2_eq = lbmol * c_per_mol * 44.01 / 2000
    out = cap + em
    print(f"{name:18s} NG {mmscfd} MMSCFD -> feed C as CO2 {co2_eq:8.1f} ton/d; reported {out:5d}; residual {co2_eq-out:7.1f} ton/d = {100*(co2_eq-out)/co2_eq:5.2f}%; captured/feedC {100*cap/co2_eq:5.1f}%")

print("\n== TEV-961 Table 2 Case 6 power rows ==")
rows = dict(NGref=1.2, Syngas=2.1, Steam=0.3, CO2comp=6.4, Purge=5.7, CT=0.1, Water=1.5)
print(sum(rows.values()), rows)
print("17.3-5.7 =", round(17.3 - 5.7, 1), "; 11.6/0.397 =", round(11.6 / 0.397, 2), "; 176.8+29.2 =", round(176.8 + 11.6 / 0.397, 2))
print("With purge recovery kept: 17.3/0.397 =", round(17.3 / 0.397, 2), "-> total", round(176.8 + 17.3 / 0.397, 1), "MWth")
print("Old Nishihara source split 88 MWe / 230 MWth =", round(88 / 230, 4))

print("\n== Capacity basis ==")
h = 8760 * 0.85
inl = 29000 * 0.45359237 * h / 1000
soxal = 66927
print("hours", h, "INL H2 t/y", round(inl, 1), "deck H2", soxal, "ratio", round(soxal / inl, 4), "inverse", round(inl / soxal, 4))
print("176.8 at deck H2 pro rata:", round(176.8 * soxal / inl, 1), "MWth; 34 MMSCFD pro rata", round(34 * soxal / inl, 2), "; 52.5 pro rata", round(52.5 * soxal / inl, 2))
print("130 MMSCFD in Nm3/h, old E2 using IEAGHG 0.08994 kg/Nm3:", round(29000 * 0.45359237 / 0.08994, 0))

print("\n== Deck arithmetic ==")
b = 9.35 * soxal
print("baseline", round(b), "furnace 35%", round(.35 * b), "process 65%", round(.65 * b), "residual 10%", round(.065 * b), "net", round(b - .065 * b), "pct", round(100 * (1 - .065), 2))
print("80% capture", round(b - .2 * .65 * b), "70% avail", round((b - .065 * b) * .7 / .85))
print("no-CCS avoided", round(.35 * b))
print("CCS cost", 366224 * 80 / 1e6, "USD M; SGD@1.30", round(366224 * 80 * 1.3 / 1e6, 2))
dq = 9e6 * 1044 * 1.05505585262e-6
print("dQ GJ/d", round(dq, 2), "annual S$M full 130MMSCFD basis", round(dq * 310.25 * 15 / 1e6, 2), "scaled to deck H2", round(dq * 310.25 * 15 / 1e6 * soxal / inl, 2))
print("If 925C NG (34.0) used, scaled:", round((52.5 - 34.0) * 1e6 * 1044 * 1.05505585262e-6 * 310.25 * 15 / 1e6 * soxal / inl, 2), "S$M/y")
crf = lambda i, n: i * (1 + i) ** n / ((1 + i) ** n - 1)
for n in (25, 40, 60):
    print("CRF", n, round(crf(.08, n), 4), "annual S$M at US$420M x1.30:", round(420 * 1.3 * crf(.08, n), 1))
ann = 420 * 1.3 * crf(.08, 25)
remain = 58.51 - 38.09 + 31.5
print("Budget remaining after CCS and gas credit", round(remain, 2), "minus CAPEX", round(remain - ann, 2))
print("Implied deck AC with CAPEX only: ", round((38.09 + ann - 31.5) / 0.585075, 1), "S$/t; on 0.544 Mt:", round((38.09 + ann - 31.5) / 0.544, 1))
print("FOAK US$672M annual S$M", round(672 * 1.3 * crf(.08, 25), 1), "AC CAPEX only", round((38.09 + 672 * 1.3 * crf(.08, 25) - 31.5) / 0.585075, 1))
print("Units to bridge 1.6x at 10%:", round(2 ** (math.log(1 / 1.6) / math.log(0.9)), 1))
print("US$9200/kWe x 80 MWe =", 9200 * 80000 / 1e6, "US$M; $5250 x 80 MWe =", 5250 * 80000 / 1e6)
print("Abatement share of 45-50 Mt:", round(.585 / 50 * 100, 2), round(.585 / 45 * 100, 2))

print("\n== Old repo per-train lifecycle ledger reconstructed ==")
gj = lambda m: m * 1e6 * 1044 * 1.05505585262e-6
a = .85
base = 3205 * ST * 365 * a
em = 142 * ST * 365 * a
cap = 1927 * ST * 365 * a
up = (gj(52.5) - gj(34.0)) * 365 * a * 11.5 / 1000
nuc = 176.8 * 1000 * h * (5.5 * .504) / 1e6
aux = (17.3 - 6.3) * 1000 * h * 5.5 / 1e6
ts = .025 * cap
net = base - em + up - nuc - aux - ts
print("base", round(base), "emit", round(em), "cap", round(cap), "upstream", round(up), "nuc", round(nuc), "aux", round(aux), "ts", round(ts), "net", round(net))
print("indirect sum nuc+aux+ts", round(nuc + aux + ts))
print("direct intensity kg/kg:", round(base / inl, 3), "candidate direct:", round(em / inl, 3), "deck residual:", round(40692 / soxal, 3))

print("\n== Illustrative cooling flows with the old E3 formula (own calculation) ==")
vol = lambda q, dt: q * 1000 / (3.99 * dt) / 1025
for label, q in [("Old 246.4 MWth envelope", 246.4), ("Old 600 MWth envelope", 600), ("210 MWth full reactor envelope", 210), ("210-176.8 residual", 33.2), ("29.2 power cycle input minus 11.6 MWe = 17.6 reject", 17.6)]:
    print(label, q, "MW: 8K flow m3/s", round(vol(q, 8), 3), "5K", round(vol(q, 5), 3))

print("\n== Old E2B with 600 MWth vs 210 MWth utilisation ==")
print("176.8/210", round(176.8 / 210, 3), "206/210", round(206 / 210, 3), "176.8/600", round(176.8 / 600, 3), "353.6/600", round(353.6 / 600, 3))
print("HTR-Modul 170 vs 176.8 short by", round(176.8 - 170, 1), "MWth; XE-100 200 MWth vs 206 short by", 6.0, "MWth")
