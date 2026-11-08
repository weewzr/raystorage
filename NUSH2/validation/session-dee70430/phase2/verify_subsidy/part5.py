import mpmath as mp
from indep import *
p = boak; s = mp.mpf("0.5")
N = lambda x, n=8: mp.nstr(x, n)
print("capitals", [N(x, 7) for x in (p.C, p.Kcap, p.Kihx, p.Klic, p.Kpi, p.Krec, p.D)], "Y", N(p.Y, 8))
fD = (q * g(w) + (g(w) - 1) / i) / tc; fI = fidc(tc)
print("fD", N(fD, 11), "fI", N(fI, 11), "df", N(fD - fI, 9), "series", N(g(w) * q * (1 - q) * i / (2 * tc), 6), "ratio", N(g(w) * q * (1 - q) * i / (2 * tc) / (fD - fI), 6))
print("(1+i)^5, S, q(1+i)^5, bracket", N(g(5), 11), N((g(5) - 1) / i, 11), N(q * g(5), 11), N(q * g(5) + (g(5) - 1) / i, 11))
print("g(q)", N(1 + q * i - g(q), 9), "(1+i)^q", N(g(q), 10))
def rho(L):
    n = -(-L // tu); r = sum(g(-m * tu) for m in range(n)); un = n * tu - L
    return r - (mp.mpf(un) / tu) * g(-L) if un > 0 else r
print("rho60", N(rho(60), 9), "diff", N(rho(60) * crf(60) - crf(25), 6), "v25 v50 v60", N(g(-25), 7), N(g(-50), 7), N(g(-60), 7), "crf25, crf60", N(crf(25), 8), N(crf(60), 8), "alpha25 alpha60", N(annuity(25), 8), N(annuity(60), 8))
k = 1e6 / p.A
print("1e6/A", N(k, 6), "gap terms 60y BOAK cap/ihx/rec", [N(K * (rho(60) * crf(60) - crf(25)) * k, 5) for K in (p.Kcap, p.Kihx, p.Krec)])
p60 = [x for x in PTS if x.stage == "BOAK" and x.L == 60][0]
print("60y BOAK terms", [N(K * (rho(60) * crf(60) - crf(25)) * 1e6 / p60.A, 5) for K in (p60.Kcap, p60.Kihx, p60.Krec)], "sum", N(sum(K * (rho(60) * crf(60) - crf(25)) * 1e6 / p60.A for K in (p60.Kcap, p60.Kihx, p60.Krec)), 6), "reactor", N(p60.C * crf(60) * (fD - fI) * 1e6 / p60.A, 6))
print("S.0.5 pieces: c_f K_rec", N(fom * p.Krec, 7), "reactor w/ interest", N(p.C * crf(25) * fI, 8), "lic+pi", N((p.Klic + p.Kpi) * crf(25), 7),
      "cap+ihx+rec", N((p.Kcap + p.Kihx + p.Krec) * crf(25), 8), "decom", N(p.D * sff(25), 7), "sum", N(p.Y + fom * p.Krec + p.C * crf(25) * fI + (p.Klic + p.Kpi) * crf(25) + (p.Kcap + p.Kihx + p.Krec) * crf(25) + p.D * sff(25), 9))
flows = p.base_flows()
npv = sum(a * g(-t) for _, t, a in flows)
print("NPV", N(npv, 10), "LAC_D", N(npv * 1e6 / (p.A * annuity(25)), 8), "kappa", N(p.kap, 9), "kappa*C", N(p.kap * p.C, 8))
for d in ("A", "B"):
    E, Xe = elig(p, d)
    print(d, "E", N(E, 8), "X", N(Xe, 8), "G(.5)", N(s * E, 8), "Kpriv", N(sum([p.C, p.Kcap, p.Kihx, p.Klic, p.Kpi, p.Krec]) - s * E, 8))
ktot = sum([p.C, p.Kcap, p.Kihx, p.Klic, p.Kpi, p.Krec]); print("Ktot", N(ktot, 8), "with interest", N(ktot + p.C * (fI - 1), 8))
bk = spend(p.C); print("buckets", [N(b, 6) for t, b in bk])
def traj(G, mode="close"):
    bal, prev, cash, out = mp.mpf(0), None, G, []
    for t, b in bk:
        if prev is not None: bal *= g(t - prev)
        if mode == "dated":
            bal += b
        else:
            u = min(cash, b); cash -= u; bal += b - u
        prev = t
        out.append(bal)
    if mode == "dated":
        out = [x - G * g(t + tc) for x, (t, b) in zip(out, bk)]
    return out
G = s * p.C
print("gross", [N(x, 6) for x in traj(0)])
print("prog ", [N(x * (1 - s), 6) for x in traj(0)])
print("close", [N(x, 6) for x in traj(G)])
print("dated", [N(x, 6) for x in traj(G, "dated")])
print("S.3b 399.0417:", N(C := p.C / tc * (mp.mpf("0.75") * g(2) + (g(2) - 1) / i), 8), " S.3c: h(.5)", N(h(mp.mpf("0.5")), 9), "C h", N(p.C * h(mp.mpf("0.5")), 8))
print("s0 dated", N((fI - 1) / (g(tc) - 1), 6), "(1+i)^tc", N(g(tc), 6), "IDC dated s=.5", N(p.C * (fI - 1) - G * (g(tc) - 1), 7), "PV priv s=1", N(p.C * (fI - g(tc)), 7))
print("W3", N(p.kap * p.C * ((fI - h(mp.mpf("0.5"))) - s * fI), 6), "W1 as % of L5(0)", N(p.kap * s * p.C * (fI - 1) / p.lac5() * 100, 4))
print("M5, MD", N(p.kap * p.C * fI, 9), N(p.kap * p.C * fD, 9), "kappa X_E", N(p.kap * elig(p, 'B')[1], 9))
print("2nd diff close A L5", N(lac5(p, mp.mpf('0.75'), 'A', 'close') - 2 * lac5(p, mp.mpf('0.5'), 'A', 'close') + lac5(p, mp.mpf('0.25'), 'A', 'close'), 6))
print("S.7c: D0", N((p.lac5() - T) / p.kap, 7), "H*", N(fI - ((p.lac5() - T) / p.kap) / p.C, 7), "u*", N(1 - mp.log(1 + i * tc * (fI - ((p.lac5() - T) / p.kap) / p.C)) / (tc * mp.log(1 + i)), 7))
print("net annual cost", {(d, t): N(lac5(p, s, d, t) * p.A / 1e6, 8) for d in "AB" for t in ("prog", "close")})
print("state PV close", N(G * g(tc), 8), "private benefit", N(p.C * fD - p.C / tc * (mp.mpf('0.75') * g(2) + (g(2) - 1) / i), 8))
print("slope ratio", N(mp.log(1 + i) / i, 8), "kink share", N(p.C / elig(p, 'B')[0], 6))
print("rho_R(25)", rho(25))
