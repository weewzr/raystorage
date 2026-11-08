# Batch A: Zhang, Isa, Halim, Salvinder, Chinchen, Kraussler, Xu-Froment,
# Versteeg, Khan Table 1. Every literal was read from the rendered page.
from fx import Fx

# ---------------------------------------------------------------- Zhang
z = Fx("zhang_que_chen_2011.csv", "Zhang Que Chen 2011 Fluid Phase Equilib. 311 67-75")
L = "Table 8 PDF p5"
for name, i, j, v, sd in [("a_ij", "H2O", "MEA", "0.1559", "0.4085"),
                          ("a_ij", "MEA", "H2O", "1.5201", "0.3234"),
                          ("b_ij", "H2O", "MEA", "110.80", "89.04"),
                          ("b_ij", "MEA", "H2O", "-910.30", "73.64")]:
    unit = "-" if name == "a_ij" else "K"
    z.add(L, "table", "constant", f"nrtl_{name}_{i}_{j}", name, v, unit, {"i": i, "j": j, "alpha": "0.2"})
    z.add(L, "table", "constant", f"nrtl_{name}_{i}_{j}", name + "_sd", sd, unit, {"i": i, "j": j, "alpha": "0.2"})

L = "Table 10 PDF p7"
for q, unit, i, j, v, sd in [
    ("dfG298_aq", "J/kmol", "MEAH+", "", "-1.8962e8", "3.9474e5"),
    ("dfH298_aq", "J/kmol", "MEAH+", "", "-3.3164e8", "2.7557e10"),
    ("Cp_aq_inf", "J/kmol/K", "MEAH+", "", "2.1022e5", "1.0326e4"),
    ("dfG298_aq", "J/kmol", "MEACOO-", "", "-4.9299e8", "2.7020e5"),
    ("dfH298_aq", "J/kmol", "MEACOO-", "", "-7.0747e8", "2.7557e10"),
    ("Cp_aq_inf", "J/kmol/K", "MEACOO-", "", "-4.9781e-2", "5.2297e-1"),
    ("a_ij", "-", "H2O", "MEAH+/MEACOO-", "6.7320", "0.4891"),
    ("a_ij", "-", "MEAH+/MEACOO-", "H2O", "-3.1628", "0.3606"),
    ("a_ij", "-", "H2O", "MEAH+/HCO3-", "8.5721", "0.2403"),
    ("a_ij", "-", "MEAH+/HCO3-", "H2O", "-4.0092", "0.0815"),
    ("b_ij", "K", "H2O", "MEAH+/MEACOO-", "-9.2407", "110.73"),
    ("b_ij", "K", "MEAH+/MEACOO-", "H2O", "39.498", "80.84"),
]:
    cond = {"i": i, "alpha": "0.2"}
    if j:
        cond["j"] = j
    s = f"t10_{q}_{i}_{j}".replace("+", "p").replace("-", "m").replace("/", "_")
    z.add(L, "table", "constant", s, q, v, unit, cond)
    z.add(L, "table", "constant", s, q + "_sd", sd, unit, cond)

# Table 7: binary regression inventory. Points are not tabulated in the paper.
L = "Table 7 PDF p4"
for dt, T, P, x, n, ard, ref in [
    ("vle_isothermal_TPxy", "363", "4.0-70.1", "0-1", "11", "3.3", "Tochigi[30]"),
    ("vle_isothermal_TPX", "298-308", "0.065-5.6", "0-1", "30", "4.2", "Touhara[38]"),
    ("vle_isothermal_TPX", "333-364", "1.3-75.8", "0-1", "41", "4.6", "NathBender[39]"),
    ("vle_isobaric_TPxy", "361-443", "66.7/101.3", "0-1", "29", "5.2", "Cai[40]"),
    ("vle_isobaric_TPxy", "373-443", "101.3", "0-1", "18", "3.3", "ParkLee[41]"),
    ("vle_isothermal_TPxy", "313-373", "6.1-99.6", "0-0.57", "87", "2.3", "Kim[42]"),
    ("excess_enthalpy", "298", "", "0.006-0.98", "52", "3.3", "Touhara[38]"),
    ("excess_enthalpy", "298", "", "0.10-0.95", "10", "5.3", "Buslaeva[43]"),
    ("excess_enthalpy", "298", "", "0.0016-0.0098", "5", "7.1", "Dohnal[44]"),
    ("excess_enthalpy", "298/342", "", "0.15-0.67", "6", "2.1", "Posey[6]"),
    ("heat_capacity", "303-353", "", "0.2-0.8", "44", "2.2", "Chiu[34]"),
    ("heat_capacity", "283-313", "", "0-1.0", "54", "2.3", "Page[45]"),
    ("heat_capacity", "298", "", "0.10-0.40", "4", "3.6", "Weiland[46]"),
]:
    c = {"data_type": dt, "T_K": T, "x_MEA": x, "ref": ref}
    if P:
        c["P_kPa"] = P
    z.add(L, "table", "inventory", f"t7_{ref}_{dt}", "aard", ard, "percent", c)
    z.add(L, "table", "inventory", f"t7_{ref}_{dt}", "n_points", n, "count", c)

# Table 9: ternary regression inventory.
L = "Table 9 PDF p6"
for dt, T, P, x, a, n, ard, ref in [
    ("vle_TPxy", "313-333", "0.005-50", "0.06-0.16", "0.11-0.59", "55", "35.5", "Hilliard[10]"),
    ("vle_TPx_pCO2", "273-423", "0.001-20000", "0.11", "0.002-1.33", "124", "33.5", "Jou[58]"),
    ("vle_TPx_pCO2", "393", "7-192", "0.11", "0.16-0.42", "19", "13.5", "Mamun[65]"),
    ("vle_TPxy", "373-443", "12-1626", "0.11", "0.30-0.52", "63", "28.0", "XuRochelle[66]"),
    ("enthalpy_of_absorption", "313-393", "", "0.11", "0.04-0.72", "84", "10.9", "KimSvendsen[67]"),
    ("heat_capacity_isobaric", "313-393", "", "0.06-0.11", "0.10-0.58", "102", "2.4", "Hilliard[10]"),
    ("heat_capacity_isobaric", "298", "", "0.03-0.16", "0.05-0.50", "20", "2.0", "Weiland[46]"),
    ("species_concentration", "293-313", "", "0.05-0.11", "0.09-0.96", "24", "35.7", "Jakobsen[68]"),
]:
    c = {"data_type": dt, "T_K": T, "x_MEA": x, "alpha_mol_mol": a, "ref": ref}
    if P:
        c["P_kPa"] = P
    z.add(L, "table", "inventory", f"t9_{ref}_{dt}", "aard", ard, "percent", c)
    z.add(L, "table", "inventory", f"t9_{ref}_{dt}", "n_points", n, "count", c)

# Table 11: out-of-regression comparison, model error against data not tabulated.
L = "Table 11 PDF p8"
for ref, n, T, P, x, a, err in [
    ("MuhlbauerMonaghan[47]", "18", "298-373", "0.1-140", "0.05", "0.38-0.74", "30.4"),
    ("Jones[48]", "48", "313-413", "0.08-930", "0.05", "0.13-0.73", "32.2"),
    ("Lee[49]", "45", "315-373", "1-6600", "0.05", "0.14-1.19", "44.4"),
    ("LawsonGarst[50]", "24", "313-413", "3-2800", "0.05-0.11", "0.11-1.0", "30.7"),
    ("Lee[51]", "256", "298-393", "0.1-10000", "0.02-0.11", "0.11-2.19", "50.5"),
    ("Isaacs[52]", "19", "353-373", "0.007-1.8", "0.05", "0.035-0.32", "112"),
    ("Maddox[53]", "60", "298-353", "4-6800", "0.05", "0.41-1.33", "28.6"),
    ("Treckmann[54]", "7", "298", "5-37", "0.02", "0.60-0.78", "54.1"),
    ("Austgen[55]", "8", "313-353", "0.1-230", "0.05", "0.27-0.69", "42.6"),
    ("ShenLi[56]", "13", "313", "16-2600", "0.05", "0.56-1.05", "35.5"),
    ("DawoduMeisen[57]", "5", "373", "460-3900", "0.09", "0.54-0.72", "43.2"),
    ("Song[59]", "10", "313", "3-2400", "0.11", "0.49-1.06", "74.4"),
    ("JaneLi[60]", "7", "353", "4-120", "0.05", "0.36-0.58", "37.7"),
    ("Park[61]", "7", "313", "3-2100", "0.05", "0.51-1.05", "28.6"),
    ("Mathonat[62]", "9", "313-393", "2000-10000", "0.11", "0.55-1.67", "36.0"),
    ("Park[63]", "13", "313", "3-2200", "0.03-0.05", "0.48-1.07", "49.8"),
    ("Bonenfant[64]", "2", "296", "101", "0.02", "0.81", "80.6"),
]:
    c = {"T_K": T, "P_kPa": P, "x_MEA": x, "alpha_mol_mol": a, "ref": ref}
    z.add(L, "table", "inventory", f"t11_{ref}", "mean_abs_dP_over_P", err, "percent", c)
    z.add(L, "table", "inventory", f"t11_{ref}", "n_points", n, "count", c)
z.write()

# ---------------------------------------------------------------- Isa
s = Fx("isa_2017.csv", "Isa et al. 2017 Int. J. Greenh. Gas Control 56 173-186")
GEOM = {"D_m": "0.145", "Z_m": "1.505", "packing": "Flexipac_1Y", "a_p_m2_m3": "420", "w_MEA_pct": "20", "alpha_lean": "0"}
L = "Table 3 PDF p4"
rows = {"20": ("24.57", "24.57", "1.03", "33.8", "32.1"),
        "30": ("24.89", "24.89", "1.0", "36", "30"),
        "40": ("24.87", "24.87", "1.03", "35", "35.9"),
        "50": ("24.92", "24.92", "1.0", "34", "36.7"),
        "60": ("24.9", "24.9", "1.0", "33.8", "33")}
for P, (co2, ch4, sol, Ts, Tg) in rows.items():
    c = dict(GEOM, P_bar=P)
    for q, v, u in [("CO2_flow", co2, "slpm"), ("CH4_flow", ch4, "slpm"), ("solvent_flow", sol, "kg/min"),
                    ("solvent_T", Ts, "C"), ("gas_T", Tg, "C")]:
        s.add(L, "table", "measured", f"inlet_P{P}", q, v, u, c)
L = "Table 7 PDF p9"
T7 = {
 "Onda": [("Tsai", "669000.00", "137000.00", "69000.00", "1937.73"), ("Rocha", "1600000.00", "328000.00", "165000.00", "4631.66"),
          ("Wang", "1860.00", "1710.00", "57.75", "802.82"), ("Gualito", "3910.00", "827.79", "415.64", "113.96"),
          ("Onda", "629000.00", "129000.00", "64900.00", "1821.80"), ("Billet", "5220000.00", "1070000.00", "539000.00", "15130.66"),
          ("deBrito", "1090000.00", "224000.00", "112000.00", "3157.24")],
 "BilletSchultes": [("Tsai", "1.23", "0.82", "0.73", "39.0"), ("Rocha", "2.93", "1.96", "1.74", "47.0"),
          ("Wang", "0.02", "0.01", "0.01", "99.0"), ("Gualito", "0.01", "0.00", "0.00", "100.0"),
          ("Onda", "1.15", "0.77", "0.68", "42.00"), ("Billet", "9.58", "6.41", "5.67", "380.0"),
          ("deBrito", "3.93", "1.53", "1.20", "61.0")],
 "Bravo": [("Tsai", "0.01", "0.00", "0.00", "100"), ("Rocha", "0.02", "0.01", "0.01", "99"),
          ("Wang", "0.00", "0.00", "0.00", "100"), ("Gualito", "0.00", "0.00", "0.00", "100"),
          ("Onda", "0.01", "0.00", "0.00", "100"), ("Billet", "0.08", "0.03", "0.02", "97"),
          ("deBrito", "0.02", "0.01", "0.00", "99")],
 "Rocha": [(a, "0.00", "0.00", "0.00", "100") for a in ("Tsai", "Rocha", "Wang", "Gualito", "Onda", "Billet", "deBrito")],
}
for mt, block in T7.items():
    for area, h20, h40, h60, err in block:
        for P, h in (("20", h20), ("40", h40), ("60", h60)):
            s.add(L, "table", "reported_model", f"t7_{mt}_{area}", "predicted_height", h, "m",
                  dict(GEOM, P_bar=P, mass_transfer=mt, area=area))
        s.add(L, "table", "reported_model", f"t7_{mt}_{area}", "height_error", err, "percent",
              dict(GEOM, mass_transfer=mt, area=area))
L = "Table 8 PDF p9"
for area, r, ae in [("Tsai", "0.9398", "0.023369"), ("Rocha", "0.3934", "0.009782"), ("Wang", "0.8041", "0.019995"),
                    ("Gualito", "0.3934", "0.009782"), ("Onda", "1.00", "0.024866"), ("Billet", "0.1204", "0.002994"),
                    ("DeBrito", "0.5770", "0.014348")]:
    s.add(L, "table", "reported_model", f"t8_{area}", "ae_over_ap", r, "-", dict(GEOM, area=area))
    s.add(L, "table", "reported_model", f"t8_{area}", "effective_area", ae, "m2", dict(GEOM, area=area))
L = "Table 9 PDF p9"
for P, a in [("20", "0.056298735"), ("30", "0.084910728"), ("40", "0.117369331"), ("50", "0.1588894"), ("60", "0.19547522")]:
    s.add(L, "table", "measured", f"t9_P{P}", "alpha_2", a, "mol/mol", dict(GEOM, P_bar=P))
    s.add(L, "table", "measured", f"t9_P{P}", "alpha_1", "0", "mol/mol", dict(GEOM, P_bar=P))
L = "Table 10 PDF p10"
AREAS = ("Tsai", "Rocha", "Wang", "Gualito", "Onda", "Billet", "deBrito")
for P, vals in [("20", ("9.92", "23.71", "11.60", "23.71", "9.33", "77.47", "16.16")),
                ("30", ("5.88", "14.04", "6.87", "14.04", "5.52", "45.88", "9.57")),
                ("40", ("4.24", "10.13", "4.96", "10.13", "3.99", "33.11", "6.91")),
                ("50", ("3.53", "8.44", "4.13", "8.44", "3.32", "27.57", "5.75")),
                ("60", ("2.92", "6.98", "3.42", "6.98", "2.75", "22.82", "4.76"))]:
    for a, v in zip(AREAS, vals):
        s.add(L, "table", "constant", f"t10_P{P}_{a}", "index_f", v, "-", dict(GEOM, P_bar=P, area=a))
L = "Table 12 PDF p11"
for P, vals in [("20", ("35.85", "73.16", "45.14", "73.16", "31.79", "91.79", "60.62")),
                ("30", ("17.05", "65.26", "29", "65.26", "11.64", "89.37", "0.93")),
                ("40", ("0.40", "57.98", "14.18", "57.98", "6.68", "87.14", "38.40")),
                ("50", ("7.87", "54.88", "7.81", "54.88", "14.68", "86.19", "33.78")),
                ("60", ("29.12", "45.99", "10.25", "45.99", "37.10", "83.48", "20.79"))]:
    for a, v in zip(AREAS, vals):
        s.add(L, "table", "reported_model", f"t12_ModelF_P{P}_{a}", "height_error", v, "percent",
              dict(GEOM, P_bar=P, model="F", f_area=a))
L = "Conclusions PDF p13"
s.add(L, "text", "measured", "ytop_conclusion", "Y_CO2_top", "0.05", "mol CO2/mol CH4", dict(GEOM, P_bar="up to 50"))
s.add(L, "text", "reported_model", "ytop_conclusion", "Y_CO2_top", "0.055", "mol CO2/mol CH4", dict(GEOM, P_bar="up to 50", model="index_f"))
s.write()

# ---------------------------------------------------------------- Abdul Halim
h = Fx("abdul_halim_2015.csv", "Abdul Halim et al. 2015 Ind. Eng. Chem. Res. 54 1675-1680")
# Lean loading is printed nowhere in this paper, so it is not a condition here.
GEOM = {"D_m": "0.046", "Z_m": "2.040", "packing": "Sulzer_metal_gauze", "a_p_m2_m3": "500", "P_MPa": "5.0", "y_CO2_in_pct": "20"}
L = "Table 1 PDF p3"
for q, lo, hi, u in [("gas_flow_min", "18.89", "", "kmol/m2/h"), ("gas_flow_max", "35.08", "", "kmol/m2/h"),
                     ("liquid_flow_min", "3.0", "", "L/h"), ("liquid_flow_max", "7.5", "", "L/h"),
                     ("liquid_load_min", "1.81", "", "m3/m2/h"), ("liquid_load_max", "4.51", "", "m3/m2/h"),
                     ("MEA_conc_min", "1.0", "", "kmol/m3"), ("MEA_conc_max", "4.0", "", "kmol/m3"),
                     ("liquid_T_min", "27", "", "C"), ("liquid_T_max", "45", "", "C"),
                     ("CO2_conc", "20", "", "percent"), ("operating_pressure", "5.0", "", "MPa")]:
    h.add(L, "table", "measured", "range", q, lo, u)
L = "Table 3 PDF p5"
cols = [("18.89", "2.0", "1.81", "27"), ("35.08", "2.0", "1.81", "27"), ("19.72", "1.0", "1.81", "27"), ("19.72", "1.0", "4.51", "27"),
        ("28.18", "1.0", "1.81", "27"), ("28.18", "4.0", "1.81", "27"), ("27.50", "2.0", "2.71", "40"), ("27.50", "2.0", "2.71", "45")]
prof = {"0.34": ("32.5", "32.5", "30.5", "32.3", "30.5", "34.7", "35.1", "37.3"),
        "0.68": ("34.1", "34.7", "30.7", "32.6", "30.7", "36.6", "37.2", "38.2"),
        "1.02": ("35.0", "35.9", "31.0", "33.3", "31.0", "37.4", "39.1", "40.4"),
        "1.36": ("38.4", "40.5", "31.5", "35.0", "32.0", "39.3", "40.5", "45.5"),
        "1.70": ("36.5", "38.1", "31.3", "33.2", "31.4", "40.0", "46.2", "45.0"),
        "2.04": ("32.4", "33.0", "30.5", "31.5", "30.5", "33.2", "36.5", "43.1")}
for z_m, temps in prof.items():
    for i, (g, m, l, t) in enumerate(cols, 1):
        h.add(L, "table", "measured", f"t3_col{i}", "column_T", temps[i - 1], "C",
              dict(GEOM, gas_flow_kmol_m2_h=g, MEA_kmol_m3=m, liquid_m3_m2_h=l, liquid_T_C=t, height_m=z_m))
h.add("Results text PDF p4", "text", "measured", "fig3_text", "KGav_low", "1.27", "mol/m2/h/kPa as printed",
      dict(GEOM, MEA_kmol_m3="2.0", liquid_m3_m2_h="1.81", liquid_T_C="27", gas_flow_kmol_m2_h="18.89"))
h.add("Results text PDF p4", "text", "measured", "fig3_text", "KGav_high", "1.32", "mol/m2/h/kPa as printed",
      dict(GEOM, MEA_kmol_m3="2.0", liquid_m3_m2_h="1.81", liquid_T_C="27", gas_flow_kmol_m2_h="35.08"))
h.add("Results text PDF p4", "text", "measured", "fig4_text", "CO2_removal_increase", "52.59", "percent",
      dict(GEOM, MEA_kmol_m3="1.0", gas_flow_kmol_m2_h="19.72", liquid_T_C="27", liquid_from="1.81", liquid_to="4.51"))
h.add("Results text PDF p5", "text", "measured", "fig6_text", "KGav_reduction", "17", "percent",
      dict(GEOM, MEA_kmol_m3="2.0", gas_flow_kmol_m2_h="27.50", liquid_m3_m2_h="2.71", T_from_C="40", T_to_C="45"))
h.write()

# ---------------------------------------------------------------- Salvinder
v = Fx("salvinder_2018.csv", "Salvinder et al. 2018 Int. J. Greenh. Gas Control 70 164-177")
L = "Table 1 PDF p4"
for q, val, u in [("sour_gas_CO2", "0.5", "mol as printed"), ("sour_gas_CH4", "0.5", "mol as printed"),
                  ("lean_H2O", "0.8", "mol as printed"), ("lean_MEA", "0.2", "mol as printed"),
                  ("sour_gas_flow", "50", "L/min"), ("lean_flow", "1.04", "L/min"),
                  ("sour_gas_T", "26.0", "C"), ("lean_T", "26.0", "C"),
                  ("sour_gas_P", "10.5", "bar"), ("lean_P", "10.6", "bar"),
                  ("column_D", "0.145", "m"), ("column_H", "1.5", "m"), ("initial_stages", "5", "count")]:
    v.add(L, "table", "measured", "nominal", q, val, u, {"packing": "Flexipac_1Y"})
v.add("Section 2.1 PDF p2", "text", "measured", "nominal", "solvent_MEA", "20", "wt percent", {"packing": "Flexipac_1Y"})
L = "Table 2 PDF p5"
v.add(L, "table", "measured", "t2", "CO2_removal", "85.0", "percent", {"basis": "pilot"})
v.add(L, "table", "reported_model", "t2", "CO2_removal", "55.0", "percent", {"basis": "aspen_equilibrium"})
v.add(L, "table", "reported_model", "t2", "CO2_removal", "84.3", "percent", {"basis": "aspen_rate_based"})
v.write()

# ---------------------------------------------------------------- Chinchen
c = Fx("wgs_chinchen_1984.csv", "Chinchen et al. 1984 Appl. Catal. 12 69-88")
L = "Table 1 PDF p6"
SEMI = {"feed_dry": "H2_80/CO_10/CO2_10", "steam_ratio": "1.0", "catalyst_vol_ml": "350", "dry_gas_m3_h_20C_1bar": "1.75"}
cat = [("16", "15-5"), ("16", "15-4"), ("21", "15-5"), ("21", "15-4"), ("31", "15-5"), ("31", "15-4")]
T1 = {"310": ("0.019", "0.018", "0.018", "0.018", "0.017", "0.017"),
      "330": ("", "0.041", "0.046", "0.044", "0.043", "0.043"),
      "350": ("0.103", "0.090", "0.114", "0.091", "0.092", "0.097"),
      "370": ("", "0.156", "0.216", "0.139", "0.190", "0.154"),
      "400": ("0.461", "0.334", "", "", "0.404", "0.256"),
      "425": ("", "", "", "", "0.586", "0.358"),
      "450": ("1.200", "0.829", "", "", "0.918", "0.565")}
for T, vals in T1.items():
    for (P, ca), val in zip(cat, vals):
        if val:
            c.add(L, "table", "measured", f"t1_{P}bar_{ca}_{T}C", "k_empty_bed", val, "1/s",
                  dict(SEMI, P_bar=P, catalyst=ca, T_C=T))
L = "Table 2 PDF p7"
MR = {"P_bar": "31.6", "GHSV_STP_1_h": "5000", "dry_gas_steam_ratio": "1", "catalyst": "15-5",
      "feed_dry": "H2_80/CO_10/CO2_10", "feed_source": "PDF_p5"}
T2 = [("11.2", "2.00", "", ("50.9", "", "80.3", "49.4")),
      ("2.54", "0.453", "", ("34.3", "9.2", "18.5", "33.0")),
      ("5.08", "0.905", "", ("43.1", "18.6", "30.2", "46.7")),
      ("10.2", "1.81", "", ("55.8", "18.1", "33.0", "51.0")),
      ("15.2", "2.72", "", ("54.8", "11.8", "30.8", "50.5")),
      ("2.54", "0.45", "a", ("49.8", "15.4", "30.3", "46.9")),
      ("15.2", "0.45", "ac", ("54.9", "19.0", "37.2", "")),
      ("2.54", "0.45", "b", ("51.6", "16.9", "28.4", "45.5")),
      ("15.2", "0.45", "bc", ("54.2", "13.5", "27.6", "50.5"))]
for i, (zc, vol, note, vals) in enumerate(T2, 1):
    for seq, (T, val) in enumerate(zip(("350", "310", "330", "350"), vals), 1):
        if not val:
            continue
        Tp = "400" if (i == 1 and seq == 3) else T
        c.add(L, "table", "measured", f"t2_row{i}_seq{seq}", "CO_conversion", val, "percent",
              dict(MR, bed_cm=zc, vol_ml=vol, T_C=Tp, table_note=note or "none"))
L = "Table 3 PDF p9"
T3 = [("310", "5.6", "2.0", "0.02", "0.02"), ("350", "5.7", "2.1", "0.09", "0.10"), ("370", "5.9", "2.1", "0.19", "0.15"),
      ("400", "6.0", "2.1", "0.40", "0.26"), ("425", "6.1", "2.2", "0.59", "0.36")]
for T, ki5, ki4, k5, k4 in T3:
    for ca, a, b in (("15-5", ki5, k5), ("15-4", ki4, k4)):
        cond = dict(SEMI, P_bar="31", catalyst=ca, T_C=T)
        c.add(L, "table", "reported_model", f"t3_{ca}_{T}C", "k_inf", a, "1/s", cond)
        c.add(L, "table", "measured", f"t3_{ca}_{T}C", "k", b, "1/s", cond)
L = "Table 5 PDF p13"
T5 = [("310", "15-5", ("0.99", "0.013", "0.014"), ("0.99", "0.013", "0.015")),
      ("310", "15-4", ("0.97", "0.013", "0.015"), ("0.98", "0.013", "0.015")),
      ("330", "15-5", ("0.98", "0.034", "0.035"), ("0.98", "0.034", "")),
      ("330", "15-4", ("0.95", "0.033", "0.037"), ("0.96", "0.034", "0.034")),
      ("350", "15-5", ("0.97", "0.078", "0.088"), ("0.98", "0.078", "0.079")),
      ("350", "15-4", ("0.93", "0.074", "0.076"), ("0.94", "0.075", "0.075")),
      ("370", "15-5", ("0.93", "0.151", "0.166"), ("0.94", "0.152", "")),
      ("370", "15-4", ("0.75", "0.125", "0.116"), ("0.80", "0.133", "0.130")),
      ("400", "15-5", ("0.80", "0.355", ""), ("0.85", "0.377", "0.355")),
      ("400", "15-4", ("0.51", "0.234", ""), ("0.56", "0.257", "0.278")),
      ("425", "15-5", ("0.60", "0.531", ""), ("0.66", "0.584", "")),
      ("425", "15-4", ("0.38", "0.346", ""), ("0.42", "0.382", "")),
      ("450", "15-5", ("0.52", "0.967", ""), ("0.57", "1.060", "0.923")),
      ("450", "15-4", ("0.29", "0.537", ""), ("0.34", "0.629", "0.691"))]
for T, ca, p21, p16 in T5:
    for P, (E, k, obs) in (("21", p21), ("16", p16)):
        cond = dict(P_bar=P, catalyst=ca, T_C=T)
        c.add(L, "table", "reported_model", f"t5_{P}bar_{ca}_{T}C", "effectiveness", E, "-", cond)
        c.add(L, "table", "reported_model", f"t5_{P}bar_{ca}_{T}C", "k_calculated", k, "cm3/g/s", cond)
        if obs:
            c.add(L, "table", "measured", f"t5_{P}bar_{ca}_{T}C", "k_observed", obs, "cm3/g/s", cond)
c.add("Effect of pressure text PDF p13", "text", "reported_model", "activation", "true_activation_energy", "121.8", "kJ/mol")
c.write()

# ---------------------------------------------------------------- Kraussler
k = Fx("wgs_kraussler_2017.csv", "Kraussler et al. 2017 Biomass Conv. Bioref. 7 153-165")
for loc, E, ser in (("Table 1 PDF p2", "101.9", "former"), ("Table 3 PDF p8", "126.6", "improved")):
    for q, val, u in [("k0", "117.8", "mol/g/Pa^(a+b+c+d)/s"), ("Ea", E, "kJ/mol"), ("a_CO", "1.77", "-"),
                      ("b_H2O", "0.23", "-"), ("c_CO2", "-0.17", "-"), ("d_H2", "-0.12", "-")]:
        k.add(loc, "table", "constant", f"kinetics_{ser}", q, val, u)
L = "Table 2 PDF p7"
for op, g, st, vd in [("1", "495", "1.6", "1.24"), ("2", "445", "1.9", "1.11"), ("3", "326", "1.6", "0.82"), ("4", "414", "1.2", "1.04")]:
    k.add(L, "table", "measured", f"OP{op}", "GHSV", g, "1/h", {"pressure": "ambient"})
    k.add(L, "table", "measured", f"OP{op}", "STDGR", st, "-", {"pressure": "ambient"})
    k.add(L, "table", "measured", f"OP{op}", "V_dry", vd, "m3/h", {"pressure": "ambient"})
L = "Table 4 PDF p10"
T4 = {"1": (("38.9", "25.4", "20.7", "10.5", "4.5"), ("48.9", "3.4", "35.0", "9.0", "3.7"), ("49.9", "2.9", "34.9", "8.6", "3.7"), ("1.9", "14.1", "0.3", "3.7", "0.3")),
      "2": (("37.3", "23.0", "24.1", "10.4", "5.2"), ("46.7", "4.4", "36.0", "9.0", "3.9"), ("46.4", "5.1", "35.2", "8.9", "4.4"), ("0.6", "17.2", "2.4", "1.0", "12.7")),
      "3": (("39.8", "25.3", "21.1", "9.5", "4.3"), ("49.8", "5.1", "33.3", "8.4", "3.4"), ("50.3", "3.6", "34.8", "7.9", "3.6"), ("0.8", "30.0", "4.5", "6.6", "4.7")),
      "4": (("38.2", "23.3", "22.7", "10.0", "5.8"), ("48.4", "3.4", "35.2", "8.7", "4.5"), ("48.7", "2.5", "35.8", "8.3", "4.8"), ("0.6", "27.5", "1.7", "3.9", "8.1"))}
SP = ("H2", "CO", "CO2", "CH4", "N2")
for op, (inl, meas, calc, err) in T4.items():
    for sp, a, b, cc, e in zip(SP, inl, meas, calc, err):
        k.add(L, "table", "measured", f"OP{op}", f"inlet_{sp}", a, "vol percent")
        k.add(L, "table", "measured", f"OP{op}", f"outlet_{sp}", b, "vol percent", {"H2_by_balance": "1" if sp == "H2" else "0"})
        k.add(L, "table", "reported_model", f"OP{op}", f"outlet_{sp}", cc, "vol percent", {"model": "improved"})
        k.add(L, "table", "reported_model", f"OP{op}", f"rel_error_{sp}", e, "percent", {"model": "improved"})
L = "Table 5 PDF p10"
for sp, val in zip(SP, ("50.5", "1.7", "35.7", "8.5", "3.6")):
    k.add(L, "table", "reported_model", "OP1", f"outlet_{sp}", val, "vol percent", {"model": "former"})
k.write()

# ---------------------------------------------------------------- Xu-Froment
x = Fx("xu_froment_1989.csv", "Xu and Froment 1989 AIChE J. 35 88-96 and 97-103")
L = "Table 1 PDF p2"
for P, s_m in [("5.0", "5.0"), ("5.0", "3.0"), ("10.0", "3.0"), ("15.0", "5.0"), ("3.0", "5.0"), ("3.0", "3.0")]:
    x.add(L, "table", "measured", f"range_sr_P{P}_S{s_m}", "H2O_to_CH4", s_m, "mol/mol", {"P_bar": P, "H2_to_CH4": "1.25", "T_K": "773/798/823/848"})
for i, P in enumerate(("8.0", "10.0", "3.0", "10.0"), 1):
    x.add(L, "table", "measured", f"range_rwgs_r{i}_P{P}", "H2_to_CO2_max", "1.0", "mol/mol", {"P_bar": P, "H2_to_CO2": "1.0/0.5", "T_K": "573/598/623/648/673", "table_row": i})
L = "Table 5 PDF p7"
T5 = [("k1", "1.842e-4", "17.94", "2.05e-4", "1.64e-4", "kmol bar^0.5/kgcat/h", "648"),
      ("k2", "7.558", "23.53", "8.200", "6.915", "kmol/kgcat/h/bar", "648"),
      ("k3", "2.193e-5", "10.69", "2.60e-5", "1.78e-5", "kmol bar^0.5/kgcat/h", "648"),
      ("KCO", "40.91", "21.87", "44.65", "37.17", "1/bar", "648"),
      ("KH2", "0.02960", "2.501", "0.0533", "0.0059", "1/bar", "648"),
      ("KCH4", "0.1791", "8.535", "0.2211", "0.1371", "1/bar", "823"),
      ("KH2O", "0.4152", "9.445", "0.5032", "0.0317", "-", "823")]
for n, val, t, ul, ll, u, Tr in T5:
    cond = {"T_ref_K": Tr, "activity": "reference"}
    x.add(L, "table", "constant", f"t5_{n}", n, val, u, cond)
    x.add(L, "table", "constant", f"t5_{n}", n + "_t_value", t, "-", cond)
    x.add(L, "table", "constant", f"t5_{n}", n + "_UL", ul, u, cond)
    x.add(L, "table", "constant", f"t5_{n}", n + "_LL_printed", ll, u, cond)
TE = [("E1", "240.1", "108.0", "244.5", "235.7"), ("E2", "67.13", "27.21", "72.06", "62.19"), ("E3", "243.9", "43.83", "255.0", "232.7"),
      ("dH_CO", "-70.65", "-23.47", "-64.61", "-76.66"), ("dH_H2", "-82.90", "-5.473", "-52.61", "-113.2"),
      ("dH_CH4", "-38.28", "-4.403", "-20.89", "-55.66"), ("dH_H2O", "88.68", "23.97", "96.06", "81.26")]
for n, val, t, ul, ll in TE:
    x.add(L, "table", "constant", f"t5_{n}", n, val, "kJ/mol")
    x.add(L, "table", "constant", f"t5_{n}", n + "_t_value", t, "-")
    x.add(L, "table", "constant", f"t5_{n}", n + "_UL", ul, "kJ/mol")
    x.add(L, "table", "constant", f"t5_{n}", n + "_LL", ll, "kJ/mol")
L = "Table 6 PDF p7"
for n, val, u in [("A_k1", "4.225e15", "kmol bar^0.5/kgcat/h"), ("A_k2", "1.955e6", "kmol/kgcat/h/bar"), ("A_k3", "1.020e15", "kmol bar^0.5/kgcat/h"),
                  ("A_KCO", "8.23e-5", "1/bar"), ("A_KH2", "6.12e-9", "1/bar"), ("A_KCH4", "6.65e-4", "1/bar"), ("A_KH2O", "1.77e5", "-")]:
    x.add(L, "table", "constant", "t6", n, val, u, {"activity": "reference"})
x.add("Thermodynamic text PDF p7", "text", "constant", "activity", "fresh_catalyst_factor", "2.246", "-")
x.add("Text PDF p6", "text", "constant", "activity", "rwgs_activity_factor", "1.225", "-")
L = "Table 7 PDF p8"
for n, val in [("dH1_948K", "224.0"), ("dH2_948K", "-37.3"), ("dH3_948K", "187.5"), ("dH1_298K", "206.1"), ("dH2_298K", "-41.15"), ("dH3_298K", "164.9")]:
    x.add(L, "table", "constant", "t7", n, val, "kJ/mol")
x.write()

# ---------------------------------------------------------------- Versteeg
w = Fx("versteeg_1988.csv", "Versteeg and van Swaaij 1988 Chem. Eng. Sci. 43(3) 573-585")
L = "Table 10 PDF p12"
for T, ref, k2, pk in [("293", "PennyRitter1983", "3.70", "9.55"), ("293", "Hikita1977a", "4.42", "9.66"), ("293", "AlvarezFuster1980", "4.30", "9.66"),
                       ("298", "PennyRitter1983", "4.89", "9.51"), ("298", "Hikita1977a", "5.87", "9.51"), ("298", "LaddhaDanckwerts1981", "5.72", "9.51"),
                       ("303", "PennyRitter1983", "6.48", "9.36"), ("303", "Hikita1977a", "7.72", "9.36"), ("303", "Sada1985", "7.74", "9.36")]:
    c2 = {"amine": "MEA", "T_K": T, "ref": ref}
    w.add(L, "table", "measured", f"t10_MEA_{T}_{ref}", "k2", k2, "m3/mol/s", c2)
    w.add(L, "table", "measured", f"t10_MEA_{T}_{ref}", "pKa", pk, "-", c2)
w.add("Eq 20 PDF p12", "text", "constant", "bronsted", "intercept", "16.26", "-", {"form": "ln_k2=pKa+c-Ta/T"})
w.add("Eq 20 PDF p12", "text", "constant", "bronsted", "T_a", "7188", "K", {"form": "ln_k2=pKa+c-Ta/T"})
w.write()

# ---------------------------------------------------------------- Khan Table 1
q = Fx("khan_2011.csv", "Khan Krishnamoorthi Mahmud 2011 Chem. Eng. Res. Des. 89 1600-1608")
L = "Table 1 PDF p5"
RUNS = {"T22": ("14.8", "19.1", "9.5", "3.00", "19.0", "100.0", "103.15"),
        "T13": ("14.8", "15.3", "13.5", "2.00", "19.0", "100.0", "103.15"),
        "T15": ("14.8", "19.5", "13.5", "2.03", "19.0", "100.0", "103.15"),
        "T19": ("14.8", "11.5", "13.5", "2.00", "19.0", "100.0", "103.15"),
        "Aroonwilas2001": ("10.7", "15.0", "15.9", "5.20", "19.7", "", "101.32"),
        "I06": ("89.32", "1.53", "13.27", "4.10", "38.7", "99.92", "724.47")}
GEO = {"T22": ("0.1", "6.6", "Berl_saddle_12.7mm"), "T13": ("0.1", "6.6", "Berl_saddle_12.7mm"), "T15": ("0.1", "6.6", "Berl_saddle_12.7mm"),
       "T19": ("0.1", "6.6", "Berl_saddle_12.7mm"), "Aroonwilas2001": ("0.1", "2.21", "Gempak_4A"), "I06": ("1.9", "14.1", "Pall_ring_50mm")}
NM = [("gas_flow", "mol/m2/s"), ("inlet_CO2", "mol percent"), ("liquid_flow", "m3/m2/h"), ("inlet_MEA", "kmol/m3"),
      ("liquid_T", "C"), ("CO2_removal", "percent"), ("column_P", "kPa")]
for run, vals in RUNS.items():
    d, zp, pk = GEO[run]
    for (n, u), val in zip(NM, vals):
        if val:
            q.add(L, "table", "measured", run, n, val, u, {"D_m": d, "Z_m": zp, "packing": pk})
q.write()
