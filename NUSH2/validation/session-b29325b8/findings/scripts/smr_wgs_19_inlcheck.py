"""Own calculations on INL Case 5 printed values. Screening checks only."""
import math, sys
sys.stdout.reconfigure(encoding="utf-8")

K = lambda T: 1.767e-2 * math.exp(4400.0 / T)
wet = dict(H2=48.8, CO=8.7, CO2=4.8, N2=0.7, H2O=33.4, CH4=3.6)
print("sum", sum(wet.values()), "S/DG in", wet["H2O"] / (100 - wet["H2O"]))


def ext(codry):
    # (CO - e)/(dry0 + e) = codry
    d0 = 100 - wet["H2O"]
    return (wet["CO"] - codry * d0) / (1 + codry)


for nm, cd, TF in (("HTS outlet", 0.0301, 764), ("LTS outlet", 0.0033, 415)):
    e = ext(cd)
    co, h2o, co2, h2 = wet["CO"] - e, wet["H2O"] - e, wet["CO2"] + e, wet["H2"] + e
    Q = co2 * h2 / (co * h2o)
    T = (TF - 32) / 1.8 + 273.15
    Teq = 4400 / math.log(Q / 1.767e-2)
    print(nm, f"e={e:.3f} CO={co:.3f} H2O={h2o:.2f} CO2={co2:.2f} H2={h2:.2f} S/DG={h2o / (100 - h2o):.3f} Q={Q:.1f} K(T={T - 273.15:.1f}C)={K(T):.1f} Q/K={Q / K(T):.2f} Teq={Teq - 273.15:.0f}C dry: H2={h2 / (100 - h2o) * 100:.1f} CO2={co2 / (100 - h2o) * 100:.1f} CH4={wet['CH4'] / (100 - h2o) * 100:.1f}")

print("\nFraction of carbon as CO at raw syngas", wet["CO"] / (wet["CO"] + wet["CO2"] + wet["CH4"]))
print("HTS-only: CO left dry 3.01%, extra CO2 from LTS per 100 wet mol", ext(0.0033) - ext(0.0301), "relative to CO2 after HTS", (ext(0.0033) - ext(0.0301)) / (wet["CO2"] + ext(0.0301)))
print("Selexol inlet CO2 printed 17.7% vs my dry LTS", (wet["CO2"] + ext(0.0033)) / (100 - wet["H2O"] + ext(0.0033)) * 100)

# Once-through H2 yield if PSA tail gas is not recycled (screening)
for x in (0.781, 0.763, 0.64):
    f = 0.96
    y = 3 * x + x * f
    print(f"x={x}: H2 per mol CH4 reformer+shift={y:.2f}; at PSA 88%={y * 0.88:.2f}; NG for 130 MMSCFD={130 / (y * 0.88):.1f} MMSCFD (CH4 only, pure)")
print("INL scf H2/scf NG: Case 6 3.82, conventional with CC 2.64; 130/3.82=", 130 / 3.82)

# NG mass basis
ton = 781 / 34.5  # short ton/d per MMSCFD
ng = 34.0 * ton * 0.90718 / 24
print("INL Case 6 NG t/h", ng, "kg/kg H2", ng / 13.154)
print("Ahn h-SMR kg/kg", 24816 / 9000, "gray", 30563 / 9000)
print("LHV MJ/kg H2 Ahn h-SMR", 24816 * 46.651 / 9000, "INL HHV MJ/kg H2", 34.0e6 * 1044 * 1.05506e-3 / 24 / 13154)
print("NG reduction INL", 1 - 34.0 / 49.2, "v 52.5", 1 - 34.0 / 52.5, "deck 43.5 v 52.5", 1 - 43.5 / 52.5)
print("NG energy S$/y at S$15/GJ, 34.0 vs 43.5 saved vs 52.5 (310.25 d)")
for g in (34.0, 43.5):
    d = (52.5 - g) * 1e6 * 1044 * 1.05506e-6 * 310.25  # GJ/y
    print(g, round(d / 1e6, 3), "PJ/y", round(d * 15 / 1e6, 1), "S$M/y")

# heat split
print("Case 5 split: reform-1", 404.2 / 568.4, "preheat", 164.2 / 568.4, "stage2 fired MW", 74 * 0.293071)
print("176.8 x split", 176.8 * 404.2 / 568.4, 176.8 * 164.2 / 568.4)
# Duties after HTS in Case 5 vs MEA reboiler with pilot SRD
co2 = 164385 * 0.45359237 / 1000
for srd in (3.4, 3.8):
    print("SRD", srd, "reboiler MW for Case 5 CO2", co2 * srd * 1000 / 3600)
# Temperatures for low-grade heat
print("EX3 213>149 C MW", 65.7 * 0.293071, "EX4 149>70 C MW", 117.1 * 0.293071)
# Water balance
print("Water out KO lb/h per lb H2", 126553 / 29000, "reformer feed water REF-WAT 12945, steam 2x248376 (A-54)", 248376 * 2, "COND-BD 113607")
print("GPM check 227.0 gpm to lb/h", 227.0 * 8.34 * 60)
print("H2O entering KO in SG-7 by raw composition unknown after shift")
# CH4 slip implied in the PSA feed by hot-end approach (dry, after LTS, S/C 3, 30 bar), wet-basis constant CH4 moles
