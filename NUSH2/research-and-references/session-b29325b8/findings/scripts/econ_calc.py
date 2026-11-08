# Cross-check calculations for the econ_lit report. Own calculations only.
# Inputs are taken from: IEAGHG 2017-02 Table 2, 3, 4, 5 (PDF pp16-18), INL TEV-961 Table 2 (PDF pp17-18),
# INL TEV-953 Table 2 (PDF p30), Ahn and Lee 2026 Tables 5, 6, Fig 6, and team drafts.

ST = 0.90718474  # t per short ton
H2_INL_kgh = 29000 * 0.45359237
H2_INL_td = H2_INL_kgh * 24 / 1000
print("INL H2 kg/h %.0f, t/d %.1f, Nm3/h at 0.08988 %.0f" % (H2_INL_kgh, H2_INL_td, H2_INL_kgh / 0.08988))
print("INL annual t/y at 85pc %.0f" % (H2_INL_kgh * 8760 * 0.85 / 1000))
print("scale 100000 Nm3/h vs INL %.4f, inverse %.4f" % (8988 / H2_INL_kgh, H2_INL_kgh / 8988))

print("\n-- IEAGHG 2017 per kg H2 (0.08994 kg/Nm3)")
rho = 8994 / 100000
cases = {
    "Base": (0.8091, 0.0, 170.95, 222.89, 11.4, None, 72.427258, 14.212),
    "1A": (0.3704, 0.4660, 201.80, 263.91, 13.5, 47.1, 85.450483, 14.676),
    "1B": (0.2918, 0.5899, 228.48, 298.68, 14.6, 62.0, 91.387119, 15.500),
    "2A": (0.3870, 0.4556, 226.07, 295.21, 14.2, 66.3, 88.337527, 14.800),
    "2B": (0.3772, 0.4289, 241.44, 313.87, 14.0, 59.5, 84.528373, 14.180),
    "3": (0.0888, 0.8004, 305.33, 398.48, 16.5, 69.8, 96.367002, 15.614),
}
for k, (e, c, tpc, tcr, lc, cac, om, ng) in cases.items():
    gen = e + c
    rate = c / gen if gen else 0
    av = (0.8091 - e) / 0.8091
    print(
        "%-5s emit %.3f capt %.3f gen %.3f kg/kgH2 | CCR %.1f%% avoided %.1f%% | LCOH EUR %.3f/kg | TCR/TPC %.3f | NG %.1f GJ/tH2"
        % (k, e / rho, c / rho, gen / rho, rate * 100, av * 100, lc / 100 / rho, tcr / tpc, ng / rho)
    )
print("annual H2 at 8322 h t/y %.0f" % (8.994 * 8322))
print("Case 1A capture+compression TPC (M EUR) %.3f" % (22.068 + 17.004))
print("Case 3 capture+compression TPC (M EUR) %.3f" % (95.162 + 28.036))
print("Base case hydrogen plant TPC 97.212; total 170.952")
print("Case 1A delta TCR %.2f, annual t captured %.0f" % (263.91 - 222.89, 0.4660 * 100000 * 8322 / 1000))
print("Fuel NG share of carbon-bearing NG energy, Base %.1f%%" % (2.014 / 14.212 * 100))
print("Case 1A flue share %.1f%%, syngas share %.1f%%" % (0.3704 / 0.8364 * 100, 0.4660 / 0.8364 * 100))
print("Base: 0.4660/0.8091 = %.1f%%" % (0.4660 / 0.8091 * 100))

print("\n-- INL CO2 per kg H2")
inl = {
    "Conv noCC (953)": (0, 3205, 52.5),
    "Conv CC": (2143, 855, 49.2),
    "HTGR noCC (953)": (0, 2713, 44.5),
    "725C CC": (2123, 525, 43.5),
    "775C": (2076, 403, 40.8),
    "825C": (2024, 267, 37.7),
    "875C": (1970, 126, 34.5),
    "925C": (1927, 142, 34.0),
}
for k, (c, e, ng) in inl.items():
    tot = c + e
    print(
        "%-16s total %.2f capt %.2f emit %.2f kg/kgH2 | capture frac %.1f%% | short ton CO2 per MMSCF NG %.1f"
        % (k, tot * ST / H2_INL_td, c * ST / H2_INL_td, e * ST / H2_INL_td, c / tot * 100 if tot else 0, tot / ng)
    )

print("\n-- Deck ledger test (100,000 Nm3/h, 66,927 t/y, 9.35)")
H = 66927
print("baseline %.0f; proc 65pc %.0f; comb 35pc %.0f" % (H * 9.35, H * 9.35 * 0.65, H * 9.35 * 0.35))
print("deck split comb 218851/625767 = %.2f%%" % (218851 / 625767 * 100))
print("deck process %.3f kg/kg, residual %.3f, captured %.3f" % (406916 / H, 40692 / H, 366224 / H))
s = 8988 / H2_INL_kgh
print("scaled 925C INL at this H2: total %.0f t/y, captured %.0f, emitted %.0f" % (5.95 * H, 5.54 * H, 0.41 * H))
print("scaled heat 176.8 -> %.1f MWth; 11.6 MWe -> %.1f MWe; 206.0 -> %.1f MWth" % (176.8 * s, 11.6 * s, 206.0 * s))
print("abatement at 130 MMSCFD on deck intensities: %.0f t/y" % (585075 / s))
dq = 9e6 * 1044 * 1.05506e-6
print("deck dQ GJ/d %.2f; annual S$M at INL scale %.2f; scaled %.2f" % (dq, dq * 310.25 * 15 / 1e6, dq * 310.25 * 15 / 1e6 * s))
dq2 = 18.5e6 * 1044 * 1.05506e-6
print("52.5-34.0 dQ GJ/d %.1f; S$M/y at INL scale %.2f; scaled %.2f" % (dq2, dq2 * 310.25 * 15 / 1e6, dq2 * 310.25 * 15 / 1e6 * s))
print("9.35 under 56/44 split: proc %.2f comb %.2f" % (9.35 * 0.56, 9.35 * 0.44))
print("9.35 vs others: IEAGHG17 8.996 (%.1f%%), INL 9.21 (%.1f%%), Ahn 9.10 (%.1f%%)" % ((9.35 / 8.996 - 1) * 100, (9.35 / 9.21 - 1) * 100, (9.35 / 9.10 - 1) * 100))

print("\n-- Ahn and Lee")
print("gray 30563 kg/h NG: kgCO2/kgNG implied IEAGHG %.3f" % (80.91 / 30.563))
for k, ng in (("gray", 30563), ("blue", 31762), ("hSMR", 24816), ("eSMR", 23439)):
    print(" %-5s NG/H2 %.3f kg/kg; C-equivalent CO2 at IEAGHG ratio %.2f kg/kgH2" % (k, ng / 9000, ng * 80.91 / 30563 / 9000))
print("h-SMR heat per t/h H2: %.1f MWth/(t/h) vs INL %.1f" % (143.4 / 9.0, 176.8 / (H2_INL_kgh / 1000)))
print("Ahn scaled to 13.15 t/h: %.1f MWth" % (143.4 / 9.0 * H2_INL_kgh / 1000))
print("direct CO2 h-SMR 4pc of 3.5 = %.2f kg/kg" % (0.04 * 3.5))
print("Implied abatement h-SMR vs gray US$/t: %.1f (LCA basis)" % ((2.37 - 1.78) / (12.91 - 3.50) * 1000))
print("Implied abatement blue vs gray US$/t: %.1f (LCA basis)" % ((1.93 - 1.78) / (12.91 - 8.23) * 1000))
print("HTGR share 27pc of 2.37 = US$%.2f/kg" % (0.27 * 2.37))

print("\n-- final_v1_7 stress tests (S$M/y). Base BOAK dC 88.7, A 0.899329")
A = 0.899329
base = 88.7
print("each S$1/t T&S on 524,096 t = S$%.3fM = S$%.3f/tCO2e" % (0.524096, 0.524096 / A))
for ts in (23.2, 25.5, 30, 60, 100, 130):
    print(" T&S S$%5.1f -> BOAK %.1f, 10-OAK %.1f, FOAK %.1f" % (ts, base / A + (ts - 23.2) * 0.524096 / A, 57.5 / A + (ts - 23.2) * 0.524096 / A, 152.2 / A + (ts - 23.2) * 0.524096 / A))
gj = 20378 * 310.25 / 1e6
gj = 18.5e6 * 1044 * 1.05506e-6 * 310.25 / 1e6
print("gas saving GJ/y %.3f M; S$1/GJ = S$%.2fM/y = S$%.2f/t" % (gj, gj, gj / A))
for usd, fx in ((12.09, 1.3065), (12.09, 1.28), (17.19, 1.28), (26.637, 1.28)):
    p = usd * fx / 1.05506
    print(" LNG US$%.2f at %.4f = S$%.2f/GJ -> BOAK gas-only effect %.1f" % (usd, fx, p, base / A - (p - 15) * gj / A))
# MEA heat
for gjt in (3.46, 3.9, 4.05):
    mw = 524096 / 7446 * gjt / 3.6
    print(" MEA reboiler at %.2f GJ/t: %.1f MWth at 70.4 t/h" % (gjt, mw))
print(" INL 925C steam export 83.7 MMBtu/h = %.1f MWth" % (83.7 * 1.05506 / 3.6))
d = 524096 * 3.9 * 15 / 1e6
print(" full MEA heat valued at S$15/GJ: S$%.1fM/y = S$%.1f/t" % (d, d / A))
d2 = (524096 * 3.9 - 83.7 * 1.05506 * 7446) * 15 / 1e6
print(" net of export steam: S$%.1fM/y = S$%.1f/t" % (d2, d2 / A))
# IDC
for y in (3, 4, 5):
    f = ((1.08 ** y - 1) / (0.08 * y))
    print(" IDC factor uniform spend %d y, year-end convention %.3f" % (y, f))
cap_b = 198.8 * 0.533
print(" BOAK reactor capital charge at 210 MWth %.1f S$M/y" % cap_b)
for f in (1.08, 1.127):
    print("  IDC x%.3f adds S$%.1fM/y = S$%.1f/t" % (f, cap_b * (f - 1), cap_b * (f - 1) / A))
print(" IEAGHG TCR/TPC 1.30 applied to BOAK reactor capital adds S$%.1f/t" % (cap_b * 0.30 / A))
# Capture scale basis
print(" capture t/h: Case 1A 46.6; project %.1f" % (524096 / 7446))
print(" linear hourly capex EUR M %.1f vs draft 55.4; delta S$M/y %.2f" % (41.02 * (524096 / 7446) / 46.6, (41.02 * (524096 / 7446) / 46.6 - 55.4) * 2.322 * 0.09368))
print(" 0.6 rule EUR M %.1f" % (41.02 * ((524096 / 7446) / 46.6) ** 0.6))
# Purge recovery
print(" purge 5.7 MWe at S$0.20/kWh: S$%.1fM/y = S$%.1f/t" % (5.7 * 7446 * 0.2 / 1000, 5.7 * 7446 * 0.2 / 1000 / A))
print(" 5.7 MWe at 39.7pc = %.1f MWth; 206.0 + = %.1f" % (5.7 / 0.397, 206.0 + 5.7 / 0.397))
print(" 11.6/0.397 = %.1f; 176.8+ = %.1f; 17.3/0.397 = %.1f; 176.8+ = %.1f" % (11.6 / 0.397, 176.8 + 11.6 / 0.397, 17.3 / 0.397, 176.8 + 17.3 / 0.397))
# Steam export and baseline power
ds = (189.2 - 83.7) * 1.05506 * 7446 / 1e6
print(" steam export delta (189.2-83.7 MMBtu/h) = %.3f M GJ/y; at S$15/GJ S$%.1fM/y = S$%.1f/t" % (ds, ds * 15, ds * 15 / A))
print(" baseline 6.3 MWe at S$0.20/kWh S$%.1fM/y = S$%.1f/t" % (6.3 * 7446 * .2 / 1000, 6.3 * 7446 * .2 / 1000 / A))
# Xe-100 numbers
print("\n-- Xe-100")
print(" 1.1-1.2bn/320 MWe = %.0f-%.0f $/kW" % (1100 / 0.32, 1200 / 0.32))
print(" 4.75-5.75bn/320 = %.0f-%.0f $/kW (not an overnight reactor cost)" % (4750 / .32, 5750 / .32))
print(" 9200 x 80 MWe = US$%.0fM" % (9200 * 80 / 1000))
print(" 3600 at 40pc = $/kWt %.0f" % (3600 * 0.4))
print(" draft BOAK at 210 MWth, 2019 USD: %.0f M = %.0f $/kWt" % (1310.4 * 0.533, 1310.4 * 0.533 / 210 * 1000))
print(" single unit penalty in a 4-pack at 0.825: x%.3f" % (4 ** 0.175))
print(" INL multi-unit vs size scaling: (210/600)^0.6 = %.3f; specific cost ratio %.3f" % ((210 / 600) ** 0.6, (210 / 600) ** (-0.4)))
print(" exponent n 0.5 -> %.3f" % ((210 / 600) ** 0.5))
print(" deck CRF checks 25y %.5f 40y %.5f 60y %.5f" % tuple(0.08 / (1 - 1.08 ** -n) for n in (25, 40, 60)))
print("\n-- NDC share: %.2f%% to %.2f%% for 0.585; %.2f%% to %.2f%% for 0.899" % (0.585 / 50 * 100, 0.585 / 45 * 100, 0.899 / 50 * 100, 0.899 / 45 * 100))
print("McKinsey implied T&S (30pc of 60-120, 150): %.0f-%.0f, %.0f" % (18, 36, 45))
print("BCG non-capture residual if capture+shipping 60-80pc: US$ %.1f-%.1f" % (141 * 0.2, 174 * 0.4))
print("BCG total at 1.3065: S$ %.0f-%.0f" % (141 * 1.3065, 174 * 1.3065))
print("EUR54.6/t 2020 x approx: at 0.8757 EUR/USD = US$%.1f" % (54.6 / 0.8757))
print("Carbon tax S$45 x 843,827 = S$%.1fM" % (45 * 0.843827))

print("\n-- Corrected Ahn carbon and gas comparison")
r = 80910 / 30563
print("kgCO2/kgNG IEAGHG ratio %.3f" % r)
for k, ng in (("gray", 30563), ("blue", 31762), ("hSMR", 24816), ("eSMR", 23439)):
    print(" %-5s NG/H2 %.3f kg/kg; total CO2 from NG carbon %.2f kg/kgH2; GJ LHV/tH2 %.1f" % (k, ng / 9000, ng * r / 9000, ng * 46.651 / 9000))
hhv_gj_t = 34.0e6 * 1044 * 1.05506e-6 / 315.7
print("INL 925C GJ HHV/tH2 %.1f; 52.5 case %.1f" % (hhv_gj_t, 52.5e6 * 1044 * 1.05506e-6 / 315.7))
print("INL saving vs noCC %.1f%%, vs CC %.1f%%; Ahn vs gray %.1f%%" % ((1 - 34 / 52.5) * 100, (1 - 34 / 49.2) * 100, (1 - 24816 / 30563) * 100))
d = 52.5 * (1 - 24816 / 30563)
s = d * 1e6 * 1044 * 1.05506e-6 * 310.25 * 15 / 1e6
print("Apply Ahn saving to 52.5: dNG %.2f MMSCFD, S$%.1fM/y vs 94.8; delta S$%.1fM/y = S$%.1f/t" % (d, s, 94.8 - s, (94.8 - s) / 0.899329))
print("BOAK then %.1f; 10-OAK %.1f; FOAK %.1f" % ((88.7 + 94.8 - s) / 0.899329, (57.5 + 94.8 - s) / 0.899329, (152.2 + 94.8 - s) / 0.899329))
print("DICE: captured t/y approx %.0f" % (683000 * 0.85 * (221 / 0.1 * 0.9) * 0.45359237 / 1000))
print("DICE TASC delta 573-474 = 99; per t/y US$%.0f; dLCOE per t captured US$%.1f" % (99e6 / (683000 * 0.85 * 2210 * 0.9 * 0.45359237 / 1000), (223.9 - 148.6) / (2210 * 0.9 * 0.45359237 / 1000)))
c3 = 398.48 - 222.89
t3 = 0.8004 * 100000 * 8322 / 1000
print("IEAGHG Case 3 delta TCR %.2f M EUR, t/y %.0f, EUR %.1f per t/y, S$ %.0f per t/y; 1A EUR %.1f" % (c3, t3, c3 * 1e6 / t3, c3 * 1e6 / t3 * 2.322, 41.02e6 / 387805))
print("Case 3 basis at 524,096 t/y: EUR %.0fM, S$%.0fM, annual S$%.1fM vs draft 12.1, delta S$%.1f/t" % (c3 / t3 * 524096, c3 / t3 * 524096 * 2.322, c3 / t3 * 524096 * 2.322 * 0.09368, (c3 / t3 * 524096 * 2.322 * 0.09368 - 12.06) / 0.899329))
