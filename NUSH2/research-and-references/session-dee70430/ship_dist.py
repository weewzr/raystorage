from math import radians, sin, cos, asin, sqrt
def gc(a, b):
    la1, lo1, la2, lo2 = map(radians, (*a, *b))
    h = sin((la2-la1)/2)**2 + cos(la1)*cos(la2)*sin((lo2-lo1)/2)**2
    return 2*6371.0*asin(sqrt(h))
sg = (1.27, 103.70)            # Jurong Island, approximate
pts = {
 "Bintulu (Sarawak) coast":      (3.17, 113.04),
 "Kuantan/Kerteh (Pen. Malaysia east coast)": (4.52, 103.45),
 "Natuna Sea (approx. centre 4N 108E)": (4.0, 108.0),
 "Tanjung Priok (Jakarta)":      (-6.10, 106.88),
 "Palembang/Musi mouth (S. Sumatra)": (-2.30, 104.95),
}
RF = 1.15   # sea-route over great-circle factor, assumed
ctau = 363.3  # kt/y captured (carbon plan)
Adir = 569.5  # kt/y (carbon plan)
for n, p in pts.items():
    d = gc(sg, p)
    for lab, D in (("gc", d), ("route", d*RF)):
        tau = 6.77e-3*D*3.102e-3 + 0.0246*0.4080 + 0.00138
        print(f"{n:45s} {lab:5s} {D:7.0f} km  tau_TS={tau:.4f}  A_net={Adir-ctau*tau:6.1f} kt/y")
