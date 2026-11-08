import random, mpmath as mp
mp.mp.dps = 40
random.seed(20261106)

def buckets(C, tcv):
    wh = int(mp.floor(tcv)); pt = tcv - wh
    out = []
    if pt > 0: out.append((-wh, C * pt / tcv))
    for k in range(wh): out.append((k - wh + 1, C / tcv))
    return out

def loan(C, G, tcv, i):
    cash, bal, prev = G, mp.mpf(0), None
    for t, b in buckets(C, tcv):
        if prev is not None: bal *= (1 + i) ** (t - prev)
        u = min(cash, b); cash -= u; bal += b - u; prev = t
    return bal

def pd_closed(C, G, tcv, i):
    wh = int(mp.floor(tcv)); qq = tcv - wh
    gam = G * tcv / C
    if gam >= tcv: return mp.mpf(0)
    if gam <= qq: rem, e = qq - gam, wh
    else:
        j = int(mp.floor(gam - qq)); r = gam - qq - j; rem, e = 1 - r, wh - 1 - j
    return C / tcv * (rem * (1 + i) ** e + ((1 + i) ** e - 1) / i)

worst = {"pd": 0, "df": 0, "h": 0, "s*": 0}
n = 0
for _ in range(400):
    i = mp.mpf(random.uniform(0.03, 0.12)); tcv = mp.mpf(random.uniform(1.2, 9.5)); C = mp.mpf(random.uniform(100, 2000))
    wh = int(mp.floor(tcv)); qq = tcv - wh
    # S.0.2
    fD = (qq * (1 + i) ** wh + ((1 + i) ** wh - 1) / i) / tcv
    fI = ((1 + i) ** tcv - 1) / (i * tcv)
    claim = (1 + i) ** wh * (1 + qq * i - (1 + i) ** qq) / (tcv * i)
    worst["df"] = max(worst["df"], abs(fD - fI - claim))
    assert fD >= fI - mp.mpf("1e-30")
    # S.3b
    for k in range(0, 21):
        G = C * k / 20
        worst["pd"] = max(worst["pd"], abs(loan(C, G, tcv, i) - pd_closed(C, G, tcv, i)))
    # S.3c exact at integer tc
    ti = mp.mpf(random.randint(2, 9))
    for m in range(0, int(ti) + 1):
        G = C * m / ti
        h = ((1 + i) ** (ti - m) - 1) / (i * ti)
        worst["h"] = max(worst["h"], abs(loan(C, G, ti, i) - C * h))
    # S.7c closed form against bisection on the smooth ledger (reactor-only def A)
    D0 = C * (fI - mp.mpf(random.uniform(0.05, 0.95)) * fI)  # a requested year-0 reduction in (0.05 fI, 0.95 fI) of C
    Hs = fI - D0 / C
    us = 1 - mp.log(1 + i * tcv * Hs) / (tcv * mp.log(1 + i))
    red = lambda u: C * (fI - ((1 + i) ** (tcv * (1 - u)) - 1) / (i * tcv))
    worst["s*"] = max(worst["s*"], abs(red(us) - D0))
    n += 1
print(n, "random parameter sets;", {k: mp.nstr(v, 3) for k, v in worst.items()})
