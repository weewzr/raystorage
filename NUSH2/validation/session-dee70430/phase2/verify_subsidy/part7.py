import mpmath as mp
from indep import *
p = boak
idc2 = lambda u: p.C * (h(u) - (1 - u))
# scan u
us = [mp.mpf(k) / 20000 for k in range(0, 20001)]
vals = [(idc2(u), u) for u in us]
mn = min(vals)
print("min IDC2 (S$M)", mp.nstr(mn[0], 6), "at u =", mp.nstr(mn[1], 6), "as share of C_R", mp.nstr(mn[0] / p.C, 5))
print("zero crossing u = 1 - 1/tc =", mp.nstr(1 - 1 / tc, 6), " IDC2 there", mp.nstr(idc2(1 - 1 / tc), 3))
# stationary point: d/du [h(u) - (1-u)] = 0  ->  (1+i)^{tc(1-u)} = i/ln(1+i)
ustar = 1 - mp.log(i / mp.log(1 + i)) / (tc * mp.log(1 + i))
print("analytic minimiser u =", mp.nstr(ustar, 8), " min IDC2/C =", mp.nstr(h(ustar) - (1 - ustar), 8))
# L5 effect of this dip, in S$/t: kappa * |IDC2| 
print("LAC effect S$/t", mp.nstr(p.kap * mn[0], 5), "(IDC2 enters the cost as +kappa*IDC2)")
# which shares (def A and B) put u in the negative region
for d in ("A", "B"):
    E, X = elig(p, d)
    print(d, "shares with negative IDC2: s from", mp.nstr(p.C * (1 - 1 / tc) / E, 5), "to", mp.nstr(p.C / E, 5), "(u from 0.818 to 1)")
# DCF form can't go negative
print("DCF: balance >= cash paid for all G:", all(loan(p.C, p.C * k / 400, tc, i) >= p.C - p.C * k / 400 - mp.mpf('1e-30') for k in range(401)))
# lac5 with IDC floored at 0 vs smooth: size at s=0.9 A
s = mp.mpf("0.9")
print("L5 at s=0.9 A close smooth", mp.nstr(lac5(p, s, 'A', 'close'), 8))
G = s * p.C; u = G / p.C
flo = lac5(p, s, 'A', 'close') + p.kap * max(mp.mpf(0), -idc2(u))
print("                 floored  ", mp.nstr(flo, 8), "difference", mp.nstr(flo - lac5(p, s, 'A', 'close'), 4))
