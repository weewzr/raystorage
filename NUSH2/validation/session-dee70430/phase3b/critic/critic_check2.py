"""Second critic check, run with python -I."""
fx = 2.322
tpc, pi = 170.952, 20.124
share = pi / tpc
fom_share_eur = (2_564_280 + 1_709_520) * share      # maintenance + insurance and local taxes, Table 4 (PDF p18)
print("power island share of TPC", share)
print("PI share of maintenance+insurance EUR/y", fom_share_eur)
print("in S$M/y", fom_share_eur * fx / 1e6, " S$/t at A_net 557767.4:", fom_share_eur * fx / 557767.4)
# S/C check on stream 4 plus derived second steam
n4 = 5514.0
hc_c = n4 * (0.2350 + 2*0.0185 + 3*0.0026 + 4*0.0003)
w4 = n4 * 0.7307
print("S/C stream 4 only", w4 / hc_c, " with 155.17 kmol/h extra", (w4 + 155.17) / hc_c)
# rounding band of stack CO2 y
print("rounding band of y9,CO2 (+/-0.00005)", 0.00005 / 0.2123)
# carbon rounding band worst case NG to stack (kmol C/h): stream 1 y rounding on C-bearing species and n9 y9
band_in = 1696.6 * (0.00005 * (1 + 1 + 2 + 3 + 4 + 5))   # crude: sum of absolute rounding times carbon atoms
band_out = 8659.4 * 0.00005
print("band in/out kmol C/h", band_in, band_out, " sum", band_in + band_out, " rel to 1839.96", (band_in + band_out) / 1839.96)
# blocking tolerance 0.2 % in kmol C/h
print("0.2% of 1839.96", 0.002 * 1839.96)
# Case 2A extra fuel CO2
extra_c = (5.597 - 4.332) * 1000 / 18.019 * 1.0845
print("Case 2A extra fuel CO2 kg/Nm3", extra_c * 44.0095 / 1e5, " emitted+captured-0.8091", 0.3870 + 0.4556 - 0.8091)
# 2A captured over A1 and A (rescaled)
print("0.4556/0.47209", 0.4556 / 0.47209, "0.4556/0.69438", 0.4556 / 0.69438)
# H2 mass check
print("stream 12 mass from kmol", 4461.5 * 2.01588)
# gap limit weakening: example at BOAK 60 y f=1 old reference
print("gap over net", 0.0687 / 2.914, "gap over gross", 0.0687 / 178.5)
