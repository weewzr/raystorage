"""Second pass checks on E to H cards: liquid Reynolds number, F.11 screening curve."""
import math
import re

SP = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/"
src = open(SP + "findings/scripts/screening.py", encoding="utf-8").read()
ns = {"math": math}
for name in ("psat_bar", "ex4_heat_above"):
    m = re.search(r"def " + name + r"\(.*?(?=\n\n\n|\ndef )", src, re.S)
    exec("import math\n" + m.group(0), ns)
print("F.11 ex4_heat_above(130) MW", ns["ex4_heat_above"](130.0))
print("F.11 ex4_heat_above(130, 27) source default p_bar 27")

# Water viscosity at 40 C, IAPWS 2008 value 653.2 uPa s (0.6527 mPa s)
mu_w = 0.6527e-3
a_p = 420.0
g = 9.80665
for label, rho, ratio in (("lean 0.25", 1056.5, 3.129), ("rich 0.50", 1110.4, 3.901)):
    mu = mu_w * ratio
    for load in (40.0, 100.0):
        u = load / 3600.0
        re_l = u * rho / (a_p * mu)
        fr = u * u * a_p / g
        print(f"{label} load {load:5.1f} mu {mu*1e3:.3f} mPa s Re_L {re_l:6.2f} Fr_L {fr:.4f}")

# Case B L/G with Case 5 dry gas after the drum, scaled by 0.668
gas_tph = 223172 * 0.45359237 / 1000 * 0.6683667
print("Case B dry gas t/h", gas_tph, "L/G on 942 t/h", 942.4 / gas_tph, "on 894 t/h", 894.1 / gas_tph)
