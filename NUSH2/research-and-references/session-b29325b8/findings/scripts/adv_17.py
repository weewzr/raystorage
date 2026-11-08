import sys, re, pickle
sys.stdout.reconfigure(encoding="utf-8")
SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
idx = pickle.load(open(SP + "/findings/adv_index.pkl", "rb"))
for (rel, p), t in idx.items():
    if "P1772" in rel and p == 23:
        tt = re.sub(r"\s+", " ", t)
        i = tt.find("3.4. AVAILABILITY")
        print(tt[i:i+1100])
# amine_spec carbon vs fresh feed and vs Case 5 published chain
f6 = 34e6 / 379.48 * 1.05379 * 0.45359237 / 24
f5 = 34.5e6 / 379.48 * 1.05379 * 0.45359237 / 24
print("fresh C Case 6 kmol/h", round(f6, 1), " Case 5", round(f5, 1))
spec = 1777 + 395 + 197
print("spec C", spec, " ratio to Case 6 fresh", round(spec / f6, 3))
c5 = (3754 + 1020 + 70) * 0.45359237
print("Case 5 reconstruction at absorber (inl_calcs 3754+1020+70 lbmol/h)", round(c5, 0), " ratio to Case 5 fresh", round(c5 / f5, 3))
print("REC_F tube C 2303.7 ratio", round(2303.7 / f6, 3), " spec vs REC_F", round(spec / 2303.7, 3), " spec vs no-recycle 2335.4", round(spec / 2335.4, 3))
print("REC_F CO2 at absorber st/d", round(1785.3 * 44.0095 * 24 / 907.18474, 0), " spec CO2 1777 vs 1785.3:", round(1777 / 1785.3, 3))
print("17.3 x 0.6833 =", round(17.3 * .6833, 2), "; 176.8 x 0.6833 =", round(176.8 * .6833, 1), "; 206.0 x .6833 =", round(206.02 * .6833, 1))
