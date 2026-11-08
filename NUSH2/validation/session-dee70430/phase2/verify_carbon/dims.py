"""Unit-carrying evaluation (sympy.physics.units) of the draft's annual formulas; the numbers must come out in the stated unit."""
import sympy as sp
from sympy.physics.units import kilogram, hour, convert_to, mole, watt, joule, Quantity
kilowatt = Quantity("kilowatt"); kilowatt.set_global_relative_scale_factor(1000, watt)
gigajoule = Quantity("gigajoule"); gigajoule.set_global_relative_scale_factor(10**9, joule)
kmol = Quantity("kmol"); kmol.set_global_relative_scale_factor(1000, mole)
tonne = Quantity("tonne"); tonne.set_global_relative_scale_factor(1000, kilogram)
yr = Quantity("yr"); yr.set_global_relative_scale_factor(8760, hour)
MWh = Quantity("MWh"); MWh.set_global_relative_scale_factor(3.6, gigajoule)
res = []
def show(name, expr, unit, expect):
    v = convert_to(expr, unit)
    num = float(v / unit)
    ok = abs(num - expect) / abs(expect) < 2e-4
    res.append(ok)
    print(("PASS " if ok else "FAIL ") + f"{name}: {num:.4f} {unit}  (expected {expect})")
th = 7446 * hour / yr                     # operating hours per year as the draft writes it (a ratio of two times)
# N.8.2: 1 kmol C/h as CO2
show("N.8.2 E = nC M th", (1 * kmol / hour) * (44.0095 * kilogram / kmol) * th, tonne / yr, 327.695)
# N.7.4: 10 GJ/h less steam
show("N.7.4 E_lost = dQ th e_NG/eta_b", (10 * gigajoule / hour) * th * (0.0561 * tonne / gigajoule) / 0.9, tonne / yr, 10 * 7446 * 0.0561 / 0.9)
show("N.7.4 E_pow = W th EF", (4.3047 * 1000 * kilowatt) * th * (0.402 * tonne / MWh), tonne / yr, 4.3047 * 7446 * 0.402)
show("N.9.5 DeltaG e_up", (4319752 * gigajoule / yr) * (0.0115 * tonne / gigajoule), tonne / yr, 4319752 * 0.0115)
w = 0.1 * MWh / tonne; EF = 0.4 * tonne / MWh
t = convert_to(w * EF, [kilogram, hour, joule])
res.append(t == 0.04 or abs(float(t) - 0.04) < 1e-12)
print("PASS " if res[-1] else "FAIL ", "N.9.4 w_TS x EF_el is a pure number:", t)
print("all pass:", all(res))
