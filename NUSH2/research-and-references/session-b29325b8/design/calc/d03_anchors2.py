"""Second anchor set for design/03_derivations_outline.md. Standard Shomate form with coefficients as held in qproc_kcheck.py."""
import importlib.util, sys, io, contextlib, math
from pathlib import Path
here = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("q", here / "qproc_kcheck.py")
q = importlib.util.module_from_spec(spec)
with contextlib.redirect_stdout(io.StringIO()):
    spec.loader.exec_module(q)
for T in (700.0, 948.0, 1144.15):
    d1 = q.H("CO",T)+3*q.H("H2",T)-q.H("CH4",T)-q.H("H2O",T)
    d2 = q.H("CO2",T)+q.H("H2",T)-q.H("CO",T)-q.H("H2O",T)
    print(T, round(d1,2), round(d2,2), round(d1+d2,2))
print("vH slope", 26830*8.314462/1000, 4400*8.314462/1000)
A = 578051.6
for f in (0.65,0.75,0.85,0.90): print(f, round(A/0.85*f))
print("CI", 38309/66925, "reduction", A/616360.6)
print("carbon intensity bound feed", 1219.23*44.0095/8988)
print("NG B MW HHV", 34*0.68328*1044*1.05505585/86.4, "LHV approx", 0.9*34*0.68328*1044*1.05505585/86.4)
print("H2 LHV MW", 8988/3600*119.96)
print("eta_th", 8988/3600*119.96/(0.9*34*0.68328*1044*1.05505585/86.4+179.04))
print("Gas saved Case B MMSCFD", (52.5-34)*0.68328, "S$M/y", (52.5-34)*0.68328*1044*1.05505585*310.25*15/1e6)
print("AC parts L3 n1 R210", (-64.796+69.582+9.252+2.667+21.587+25.323+8.361+6.12+1.501), (-64.796+69.582+9.252+2.667+21.587+25.323+8.361+6.12+1.501)/0.5780516)
print("IHX area", 120.8e6/(400*25), 120.8e6/(760*25))
print("Solvent Case B t/h", 1097.9/(0.25)*61.08/0.30/1000)
