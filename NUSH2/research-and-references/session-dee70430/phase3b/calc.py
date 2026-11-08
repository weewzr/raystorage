"""IEAGHG 2017-02 Base Case verification arithmetic. Run with python -I."""
# ---- Table 2 (PDF p16) targets
ng_feed, ng_fuel, lhv_ng = 26.231, 4.332, 46.50      # t/h, t/h, MJ/kg
h2_th, lhv_h2 = 8.994, 119.96                         # t/h, MJ/kg
nm3h = 100000.0
print("== Table 2 checks")
ng_tot = ng_feed + ng_fuel
print("NG total t/h", round(ng_tot, 3))
print("Energy in MW = NG*LHV/3.6", round(ng_tot * lhv_ng / 3.6, 3))
print("  feed MW", round(ng_feed * lhv_ng / 3.6, 3), " fuel MW", round(ng_fuel * lhv_ng / 3.6, 3))
print("H2 energy MW", round(h2_th * lhv_h2 / 3.6, 3))
print("H2 density kg/Nm3", h2_th * 1000 / nm3h)
gross, h2p, cog = 11.500, 1.216, 0.366
print("Net export MWe = 11.500-1.216-0.366 =", round(gross - h2p - cog, 3))
print("Utility sheet kW: 1216+19+347-11500 =", 1216 + 19 + 347 - 11500)
print("COGEN aux + BoP = 19+347 kW =", 19 + 347, "kW (Table 2: 0.366)")
print("Total internal consumption MWe", round(h2p + cog, 3), " share of gross %", round((h2p + cog) / gross * 100, 2))
co2 = 0.8091
print("CO2 kg/kg H2 =", round(co2 * nm3h / (h2_th * 1000), 5), " unrounded repo 8.995997")
print("CO2 t/h =", round(co2 * nm3h / 1000, 3))
# flue gas CO2 from H&M stream 9: 8659.4 kmol/h * 0.2123 * 44.01
print("Flue gas CO2 kg/h (H&M stream 9; MW CO2 44.01 external) =", round(8659.4 * 0.2123 * 44.01, 0))
print("Specific NG GJ/1000Nm3 feed", round(ng_feed * lhv_ng / (nm3h / 1000), 3), "fuel", round(ng_fuel * lhv_ng / (nm3h / 1000), 4),
      "total", round(ng_tot * lhv_ng / (nm3h / 1000), 4))
print("LHV efficiency H2/NG %", round(299.70 / 394.77 * 100, 2), " (H2+net power)/NG %", round((299.70 + 9.918) / 394.77 * 100, 2))
print("Energy not in H2 MW", round(394.77 - 299.70, 2))
# specific electricity
for name, mw in (("H2 plant", 1.216), ("COGEN aux+BoP", 0.366), ("total internal", 1.582), ("gross", 11.5), ("net export", 9.918)):
    print(f"  {name}: {mw*1000/nm3h*1000:.2f} kWh/1000Nm3  {mw*1000/(h2_th*1000):.4f} kWh/kg H2")
print("net export MJ/Nm3 H2", round(9.918 * 3600 / nm3h, 4))
# ---- hours
print("== hours")
print("0.95*8760 =", 0.95 * 8760, " 0.70*8760 =", 0.70 * 8760)
hrs = 8322
print("H2 Nm3/y", nm3h * hrs, " t/y", round(h2_th * hrs, 1))
print("NG t/y", round(ng_tot * hrs, 1), " feed", round(ng_feed * hrs, 1), " fuel", round(ng_fuel * hrs, 1))
print("NG cost EUR/y at 279.0 EUR/t (=6*46.5)", round(ng_tot * hrs * 279.0), " 6*46.5=", 6 * 46.5)
print("  feed", round(ng_feed * hrs * 279.0), " fuel", round(ng_fuel * hrs * 279.0), " Annex IV feed 60,906,750 fuel 10,058,630")
print("  Table 4 sum 70,965,387; diff vs 279.0*30.563*8322 =", 70965387 - round(ng_tot * hrs * 279.0))
print("Electricity MWh/y", round(9.918 * hrs, 1), " EUR/y at 80", round(9.918 * hrs * 80))
print("Water m3/y", 59.7 * hrs, " EUR/y at 0.2", round(59.7 * hrs * 0.2, 1), "(Table 4: 99,365)")
print("Year-1 scale 70/95 =", round(70 / 95, 4), " 60906.75*70/95 =", round(60906.75 * 70 / 95, 2), "(Annex IV 44,878.66)")
# ---- Capex checks
print("== capex checks (EUR)")
inst = 142.460e6
print("contingency 20% of installed", 0.2 * inst)
tpc = 170.952e6
print("Maintenance 1.5% TPC", 0.015 * tpc, " ins+tax 1%", 0.01 * tpc)
lab = 2.28e6
mlab = 0.4 * 0.015 * tpc
print("Overheads 30% (labour + maint labour)", 0.3 * (lab + mlab))
print("Spare 0.5%", 0.005 * tpc, " owner 7%", 0.07 * tpc, " startup 2%", 0.02 * tpc)
print("Addl fuel 25% of one month", 0.25 * 70965387 / 12)
print("Startup O&M 3 months (labour+overhead+maint labour)", (lab + 991714 + mlab) / 4)
print("Startup cat/chem 1 month (420k + maint materials)", (420000 + 0.6 * 0.015 * tpc) / 12)
print("TCR sum", 170952000 + 854760 + 3419040 + 1478446 + 1074356 + 163214 + 11966640 + 32941928 + 36337)
print("Sub-total (TCR - TPC)", 222886722 - 170952000)
print("Power island share of TPC %", round(20.124 / 170.952 * 100, 2), " H2 plant", round(97.212 / 170.952 * 100, 2), " BoP", round(53.616 / 170.952 * 100, 2))
print("TPC per Nm3/h EUR", 170.952e6 / nm3h, " TCR per Nm3/h", 222.886722e6 / nm3h)
print("TPC per t/h H2 (MEUR)", round(170.952 / h2_th, 3), " TCR per t/h", round(222.886722 / h2_th, 3))
# IDC reproduction (spend at year end, interest on prior-year closing balance, 8%)
def idc(r=0.08, spend_share=(0.20, 0.45, 0.35)):
    spares = 854760.0
    owner = 11966640.0
    bal = 0.0
    tot_int = 0.0
    for i, s in enumerate(spend_share):
        interest = bal * r
        spend = s * tpc + s * spares + (owner if i == 0 else 0.0)
        bal = bal + interest + spend
        tot_int += interest
    return tot_int
print("IDC reproduction (owner yr1, spares phased) =", round(idc()), " report 32,941,928")
# ---- Opex table
print("== Opex (Table 4, PDF p18)")
fixed = 2280000 + 991714 + 1709520 + 2564280
var = 70965387 + 99365 + 420000
print("fixed", fixed, " variable", var, " total", fixed + var, " less elec 6,603,008 =", fixed + var - 6603008)
print("Annual O&M per Nm3 (EUR/Nm3)", (fixed + var - 6603008) / (nm3h * hrs))
# ---- LCOH reproduction using published Annex II rows (EUR M), mapping periods consecutively
r = 0.08
periods = [-3, -2, -1] + list(range(1, 26)) + [26]
idx = {p: i for i, p in enumerate(periods)}
capex = {-3: 46.33, -2: 81.34, -1: 77.36, 1: 17.82}
wc = {1: 0.04, 2: -0.01, 26: -0.04}
fixed_c = 7.54551
var_c = {1: 52.78350, **{k: 71.48475 for k in range(2, 26)}}
elec = {1: 4.86537, **{k: 6.60301 for k in range(2, 26)}}
vol = {1: nm3h * hrs * 70 / 95 / 1e6, **{k: nm3h * hrs / 1e6 for k in range(2, 26)}}  # million Nm3
def pv(d):
    return sum(v / (1 + r) ** idx[p] for p, v in d.items())
pv_capex = pv(capex); pv_wc = pv(wc)
pv_fixed = pv({k: fixed_c for k in range(1, 26)})
pv_var = pv(var_c); pv_el = pv(elec); pv_vol = pv(vol)
lcoh = (pv_capex + pv_wc + pv_fixed + pv_var - pv_el) / pv_vol
print("== LCOH reproduction from published rows")
print("PV capex", round(pv_capex, 2), "wc", round(pv_wc, 3), "fixed", round(pv_fixed, 2), "var", round(pv_var, 2), "elec", round(pv_el, 2), "vol Mm3", round(pv_vol, 1))
print("LCOH EUR/Nm3 reproduced", round(lcoh, 5), " published 0.1144")
for nm, v in (("capital", pv_capex + pv_wc), ("fixed O&M", pv_fixed), ("variable O&M (chem, cat, water)", None), ("EE", -pv_el)):
    pass
# split variable into fuel and non-fuel
fuel_c = {1: (44.87866 + 7.41163), **{k: (60.90675 + 10.05863) for k in range(2, 26)}}
oth_c = {k: var_c[k] - fuel_c[k] for k in var_c}
print("components c/Nm3: capital", round((pv_capex + pv_wc) / pv_vol * 100, 3), " fixed", round(pv_fixed / pv_vol * 100, 3),
      " var non-fuel", round(pv(oth_c) / pv_vol * 100, 3), " fuel", round(pv(fuel_c) / pv_vol * 100, 3), " EE", round(-pv_el / pv_vol * 100, 3),
      " total", round(lcoh * 100, 3))
print("Sales check: 0.1144*832.2 +6.603 =", round(0.1144 * nm3h * hrs / 1e6 + 6.603, 2), " published 101.82; yr1", round(0.1144 * nm3h * hrs * 70 / 95 / 1e6 + 4.865, 2), "pub 75.02")
print("capex sum", 46.33 + 81.34 + 77.36 + 17.82)
# EUR per kg / per t
lc = 0.1144
print("LCOH EUR/kg", round(lc / (h2_th * 1000 / nm3h), 4), " EUR/t", round(lc / (h2_th * 1000 / nm3h) * 1000, 1))
lc_t5 = 0.114
print("Table 5 rounded 11.4 c/Nm3 -> EUR/kg", round(lc_t5 / 0.08994, 4))
# EE credit in LCOH
print("EE credit c/Nm3 simple", round(6.60301e6 / (nm3h * hrs) * 100, 3))
# ---- FX (repository)
F = 2.322
print("== repository conversions (S$/EUR2014 = 2.322, repo parameters.toml)")
for nm, v in (("TPC M", 170.952), ("TCR M", 222.886722), ("Power island TPC M", 20.124), ("H2 plant TPC M", 97.212), ("BoP TPC M", 53.616),
              ("Fixed O&M M/y", 7.545514), ("Elec revenue M/y", 6.603008), ("Annual O&M net M/y", 72.427258)):
    print(f"  {nm}: EUR {v} -> S$ {v*F:.2f}")
print("  NG 6 EUR/GJ LHV -> S$/GJ LHV", 6 * F)
print("  Electricity 80 EUR/MWh -> S$/kWh", 80 * F / 1000, " repo 0.20 ->", round(0.20 / (80 * F / 1000) - 1, 4))
print("  T&S 10 EUR/t -> S$/t", 10 * F, " (repo low 23.2)")
print("  LCOH 0.1144 EUR/Nm3/0.08994 -> S$/kg", round(0.1144 / 0.08994 * F, 4), " repo unabated SMR 3.618")
print("  implied EUR/USD from 2.322:", round(2.322 / ((128.885 / 96.417) * 1.3065), 4))
# ---- steam & power island
print("== steam / cogen")
hp_exp = 46.053; hp_proc = 95.301; lp = 0.540
tot = hp_exp + hp_proc
print("HP steam total t/h (stream 10 + 14)", round(tot, 3), " export share %", round(hp_exp / tot * 100, 2), " process %", round(hp_proc / tot * 100, 2))
print("75% reformer WHB t/h", round(0.75 * tot, 2), " remaining 25%", round(0.25 * tot, 2), "(shift WHB + convective coil)")
print("Min export if all non-reformer-WHB steam were lost: ", round(0.75 * tot - hp_proc, 2), "t/h (inference)")
print("Utility-sheet export 45.9 vs H&M 46.053 diff %", round((46.053 - 45.9) / 46.053 * 100, 2))
print("Turbine kWh per t steam", round(11500 / 45.9, 1), " (utility sheet 45.9 t/h) ; with 46.053:", round(11500 / 46.053, 1))
MW_per_Mkcal = 4.1868e6 / 3.6e6  # MW per MMkcal/h, IT calorie 4.1868 J
print("MW per MMkcal/h (4.1868)", round(MW_per_Mkcal, 4), " thermochem 4.184:", round(4.184 / 3.6, 4))
print("Condenser 24 MMkcal/h =", round(24 * MW_per_Mkcal, 2), "MW")
print("Sea water to condenser 3420 m3/h*7K*4.18 =", round(3420 * 1000 * 4.18 * 7 / 3.6e6, 2), "MW (cp=4.18 assumed)")
print("Turbine heat input approx = gross + condenser =", round(11.5 + 24 * MW_per_Mkcal, 2), "MW; share of 394.77 MW %", round((11.5 + 24 * MW_per_Mkcal) / 394.77 * 100, 2))
print("Radiant duty 82.63 MMkcal/h =", round(82.63 * MW_per_Mkcal, 2), "MW")
print("Utility sheet fuel 48.2 MMkcal/h =", round(48.2 * MW_per_Mkcal, 2), "MW vs NG fuel 4.332 t/h =", round(ng_fuel * lhv_ng / 3.6, 2), "MW (ratio", round(48.2 * MW_per_Mkcal / (ng_fuel * lhv_ng / 3.6), 4), ")")
print("Utility total fuel 48.7 MMkcal/h =", round(48.7 * MW_per_Mkcal, 2), "MW")
print("Water: makeup 45.9 vs losses 44.2 + effluent 1.71 =", 44.2 + 1.71)
print("Water balance: 91.8 DMW+cond; raw water 59.7; effluent 15.5 total; (1.71 + 13.8)=", 1.71 + 13.8)
print("Steam condensing: export 45.9 t/h vs makeup 45.9: ok; sea water total 3420+15 =", 3420 + 15)
print("PSA tail gas flow kg/h 60658; fuel NG 4332")
print("Radiant duty share of total NG-LHV input %", round(82.63 * MW_per_Mkcal / 394.77 * 100, 2))
