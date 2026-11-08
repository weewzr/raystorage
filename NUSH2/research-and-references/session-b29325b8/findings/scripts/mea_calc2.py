"""Second batch of own calculations for the mea_columns report (screening only)."""
import math

MW_CO2, MW_MEA = 44.01, 61.08


def hdr(t):
    print("\n" + "=" * 8 + " " + t + " " + "=" * 8)


hdr("Reactor size with extraction steam for the regeneration heat")
Qsmr, eta, E = 176.8, 0.397, 11.6
sel, export = 19.05, 24.53
for E in (11.6, 17.3):
    for r in (0.169, 0.30):
        for s in (3.4, 3.8, 4.2):
            q = 70.4 * s / 3.6
            need = max(0.0, q - sel - export)
            w = min(E, r * need)
            reactor = Qsmr + need + w + (E - w) / eta
            print(f"E={E} MWe r={r} SRD={s}: Qreb {q:.1f} need {need:.1f} W_ext {w:.1f} reactor {reactor:.1f} MWth")
print("no export credit, SRD 3.8, E 11.6, r 0.169:")
q = 70.4 * 3.8 / 3.6
need = q - sel
w = min(11.6, 0.169 * need)
print(f"need {need:.1f} W {w:.1f} reactor {176.8 + need + w + (11.6 - w) / 0.397:.1f}")

hdr("Interheating arithmetic (Vinjarapu 2025 Table 4)")
for nm, si, mr, co2, srd in [("base", 189, 275, 14.0, 3.71), ("SI_IH 7%", 190, 272, 14.2, 3.38), ("SI_IH 28%", 187, 264, 13.1, 2.71)]:
    tot = si * mr / 3600
    print(f"{nm}: total stripping heat {tot:.2f} kW; total specific {tot * 3.6 / co2:.2f} GJ/t; reported reboiler-only SRD {srd}")
print("28%% of 74.3 MW = %.1f MW" % (0.28 * 74.3))

hdr("Absorber adiabatic rise and rich equilibrium margin")
dHa = 1.93  # GJ/t equal to 85 kJ/mol CO2; DICE back-calculation gives 1.87
for F, cp in [(1300, 3.6), (1600, 3.6)]:
    Q = dHa * 70.4 / 3.6 * 1000  # kW
    print(f"solvent {F} t/h: absorption heat {Q / 1000:.1f} MW, rise if all to liquid {Q / (F * 1000 * cp / 3600):.1f} K")

hdr("Hatta and instantaneous enhancement screening at 40 C")
T = 313.15
k2 = 10 ** (10.99 - 2152 / T)  # Khan Eq 2, L/mol/s
Hk = 10 ** (5.3 - 0.035 * 5 - 1140 / T)  # Khan Eq 8; units not stated in Khan
D = 2.35e-6 * math.exp(-2119 / T)  # Khan Eq 6b, m2/s if the printed 2.35e-6 is m2/s
print("k2 %.0f L/mol/s; H(Eq 8) %.1f (units not stated); DCO2,w %.2e m2/s" % (k2, Hk, D))
for pc, nm in [(0.131, "flue 12.5 vol%"), (4.74, "syngas bottom"), (0.58, "syngas top 90%")]:
    Ci = pc / 1.01325 / Hk * 1000  # mol/m3 if H is atm L/mol
    Einf = 1 + 5000 * 0.6 / (2 * Ci)
    print(f"{nm}: pCO2 {pc} bar, C_i {Ci:.1f} mol/m3, E_inf {Einf:.1f}")
kl = 1e-4
Ha = math.sqrt(D * 0.6 * k2 * 5) / kl
print("Hatta at kL=1e-4 m/s, C_MEA=5 kmol/m3, D=0.6 D_w: %.0f" % Ha)

hdr("Small unit checks")
print("20 wt%% MEA mole fraction %.3f; 0.2 mole fraction = %.1f wt%%" % (
    (0.2 / 61.08) / (0.2 / 61.08 + 0.8 / 18.015), 100 * 0.2 * 61.08 / (0.2 * 61.08 + 0.8 * 18.015)))
print("Kwak: CO2 feed %.1f kg/h, 90%% = %.1f kg/h; MEA %.2f kmol/h; delta alpha needed %.3f" % (
    350 * 0.152 / 22.414 * 44.01, 0.9 * 350 * 0.152 / 22.414 * 44.01, 1300 * 0.3 / 61.08, 0.9 * 350 * 0.152 / 22.414 / (1300 * 0.3 / 61.08)))
print("Kwak solvent per t CO2 %.1f" % (1300 / (0.9 * 350 * 0.152 / 22.414 * 44.01)))
print("Vinjarapu 3.46 point: L/G %.2f kg/kg" % (306 / 113))
print("Halim KGa ratio low: 0.18 kmol vs 1.0 mol: %.0f; high: 0.60 vs 3.65: %.0f" % (180 / 1.0, 600 / 3.65))
print("INL Selexol H2 in CO2: 0.7 mol%% x 1706 kmol/h = %.1f kmol/h; of H2 feed %.0f: %.3f%%" % (
    0.007 * 1706, 12872 * 0.5725, 100 * 0.007 * 1706 / (12872 * 0.5725)))
print("MEA H2 co-absorption screening (not verified): 7.8e-4 mol/L/atm x pH2 x 1400 m3/h")
for ph2 in (15.9, 20.0):
    n = 7.8e-4 * ph2 / 1.01325 * 1.33e6 / 1000
    print(f"  pH2 {ph2} bar: {n:.1f} kmol/h in water-solubility limit; {100 * n / 7477:.2f}% of 7477 kmol/h")
print("deck residual on plant carbon: 6.08 t/t vs INL %.2f tCO2/tH2" % (2069 * 0.90718474 / (29000 * 0.45359237 * 24 / 1000)))
