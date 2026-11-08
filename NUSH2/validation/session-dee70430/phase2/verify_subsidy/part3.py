import mpmath as mp
import indep as X
from indep import *

print("== 3. at-close algebra versus an explicit loan account (independent of S.3a/S.3b formulas) ==")

def loan_account(C, G, tcv, rate):
    """Event simulation. Dates are year ends. Grant cash G idle at 0%, spent first. Loan grows at `rate`."""
    bk = spend(C, tcv)
    cash, loan, prev = G, mp.mpf(0), None
    for t, b in bk:
        if prev is not None:
            loan *= (1 + rate) ** (t - prev)
        use = min(cash, b); cash -= use
        loan += b - use
        prev = t
    return loan

# (a) integer construction time: the ledger smooth formula is exact for the DCF buckets when the grant clears whole buckets
ok = True
for tcv in (mp.mpf(4), mp.mpf(5), mp.mpf(6)):
    C = mp.mpf(742.767)
    for m in range(0, int(tcv) + 1):
        G = C * m / tcv
        loan = loan_account(C, G, tcv, i)
        u = G / C
        hh = (g(tcv * (1 - u)) - 1) / (i * tcv)
        ok = ok and abs(loan - C * hh) < mp.mpf("1e-30")
chk("ledger h(u) equals the explicit loan account when t_c is an integer and the grant clears whole buckets", ok)

# (b) the actual t_c = 5.5: my loan account vs my PV-of-private-payments vs the draft's closed form P_D (re-derived by me)
def pd_formula(C, G):
    gam = G * tc / C
    if gam >= tc: return mp.mpf(0)
    if gam <= q: rem, e = q - gam, w
    else:
        j = int(mp.floor(gam - q)); r = gam - q - j; rem, e = 1 - r, w - 1 - j
    return C / tc * (rem * g(e) + (g(e) - 1) / i)

ok = True; worst = mp.mpf(0)
C = boak.C
for k in range(0, 1001):
    G = C * k / 1000
    a = loan_account(C, G, tc, i)
    pot, pv = G, mp.mpf(0)
    for t, b in spend(C):
        use = min(pot, b); pot -= use; pv += (b - use) * g(-t)
    c = pd_formula(C, G)
    worst = max(worst, abs(a - pv), abs(a - c))
chk("loan account = PV of private payments = S.3b closed form on 1001 grant sizes", worst < mp.mpf("1e-30"), mp.nstr(worst, 3))

# (c) DCF-vs-ledger gap at close: bounded between 0 and the zero-subsidy gap
ok, mx = True, mp.mpf(0)
for p in PTS:
    gap0 = lacD(p, 0, "A", "prog") - lac5(p, 0, "A", "prog")
    for d in ("A", "B"):
        for k in range(0, 101):
            s = mp.mpf(k) / 100
            gp = lacD(p, s, d, "close") - lac5(p, s, d, "close")
            mx = max(mx, abs(gp))
            ok = ok and (-mp.mpf("3e-6") <= gp <= gap0 + mp.mpf("3e-6"))
chk("at close, 0 <= LAC_D - LAC_5 <= zero-subsidy gap on 101 shares x 8 points x 2 definitions", ok, "max |gap| " + mp.nstr(mx, 4))

# (d) progressive gap law
ok = True
for p in PTS:
    for s in (mp.mpf(x) for x in ("0", "0.25", "0.5", "0.75", "1")):
        gap = lacD(p, s, "A", "prog") - lac5(p, s, "A", "prog")
        reactor0 = p.C * crf(p.L) * (X.fidc(tc) * 0 + (( q * g(w) + (g(w) - 1) / i) / tc) - fidc(tc)) * 1e6 / p.A
        gap0 = lacD(p, 0, "A", "prog") - lac5(p, 0, "A", "prog")
        ok = ok and abs(gap - (gap0 - s * reactor0)) < mp.mpf("1e-5")
chk("progressive: gap(s) = gap(0) - s * reactor IDC term (to the 6-decimal CSV rounding)", ok)

print("\n== 4. interest during construction: no double count ==")
p = boak; s = mp.mpf("0.5"); G = s * p.C
fD = (q * g(w) + (g(w) - 1) / i) / tc
f = fidc(tc)
kap = p.kap
# correct progressive reduction in L5 and DCF
corr5 = lac5(p, 0, "A", "prog") - lac5(p, s, "A", "prog")
corrD = lacD(p, 0, "A", "prog") - lacD(p, s, "A", "prog")
print("correct progressive reduction: L5", mp.nstr(corr5, 8), " DCF", mp.nstr(corrD, 8))
chk("correct progressive L5 reduction = kappa s C f", near(corr5, kap * s * p.C * f, mp.mpf("1e-5")))
# W-A: grant base is a present value that already holds interest: G' = s C f
# W-A-prog: take credit s*C*f as a nominal grant AND let the owner borrow (1-s) of the spend (so IDC falls by s*C*(f-1) as well)
wa = kap * (s * p.C * f + s * p.C * (f - 1))
print("W2 (PV-based credit and a pro-rata interest cut): reduction", mp.nstr(wa, 8), " excess over correct", mp.nstr(wa - corr5, 8))
chk("W2 excess equals kappa s C (f-1) = 11.896", near(wa - corr5, kap * s * p.C * (f - 1), mp.mpf("1e-5")) and near(wa - corr5, mp.mpf("11.896"), mp.mpf("5e-4")),
    mp.nstr(wa - corr5, 8))
# W-B: share applied to a present value at year 0 AND the resulting figure dated at close and rolled forward (dated in the past)
wb_grant = s * p.C * f           # share of a PV figure (already includes interest)
wb_rolled = wb_grant * g(tc)      # dated at financial close, compounded to year 0
wb_red = kap * wb_rolled
print("W4 (share of the PV figure, dated at close, compounded again): reduction", mp.nstr(wb_red, 8), " excess over correct", mp.nstr(wb_red - corr5, 8))
print("   as multiple of the correct reduction", mp.nstr(wb_red / corr5, 8), " ; equals (1+i)^t_c =", mp.nstr(g(tc), 8))
chk("W4 interest effect counted twice: reduction is (1+i)^t_c times the correct one", near(wb_red / corr5, g(tc), mp.mpf("1e-25")))
# and the pure nominal grant dated at close (idle cash earns i): reduction kappa * G * (1+i)^tc
wc_red = kap * s * p.C * g(tc)
print("S.3d bound (nominal G dated at close, idle cash earns i): reduction", mp.nstr(wc_red, 8))
# which of the two wrongly-high numbers is compatible with s=1 reactor-only?
chk("S.3d at s=1 removes more than the reactor value C f", p.C * g(tc) > p.C * f)

# explicit flow check: with the dated-at-close idle-cash flow, the grant PV is bigger than the private reactor PV for s>f/(1+i)^tc
s_cross = f / g(tc)
chk("dated idle cash: private value crosses zero at s = f/(1+i)^tc = 0.7843", near(s_cross, mp.mpf("0.7843"), mp.mpf("5e-5")), mp.nstr(s_cross, 6))

print("\n== 5. IDC bases ==")
idc1 = p.C * (f - 1)
idc2p = (1 - s) * p.C * (f - 1)
u = G / p.C
idc2c = p.C * h(u) - p.C * (1 - u)
print("basis1", mp.nstr(idc1, 8), "prog b2", mp.nstr(idc2p, 8), "close b2", mp.nstr(idc2c, 8), "close b3", mp.nstr(idc1 - idc2c, 8))
# IDC DCF: balance at year 0 less cash paid in
bal_gross = loan_account(p.C, 0, tc, i); bal_prog = (1 - s) * bal_gross; bal_close = loan_account(p.C, G, tc, i)
print("DCF IDC gross", mp.nstr(bal_gross - p.C, 8), "prog", mp.nstr(bal_prog - (1 - s) * p.C, 8), "close", mp.nstr(bal_close - (p.C - G), 8))
