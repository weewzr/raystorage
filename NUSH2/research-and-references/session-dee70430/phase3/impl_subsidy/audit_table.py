"""Recompute every grid row of results/Subsidy_Scenarios.csv from S.6a and S.6b at 40 digits,
using only the zero-share cost, the eligible capital of the rows and the registry rate and
construction time. No engine call."""
import csv, re
from mpmath import mp, mpf, floor
mp.dps = 40
i = mpf("0.08"); tc = mpf("5.5")
rows = list(csv.DictReader(open("results/Subsidy_Scenarios.csv", encoding="utf-8")))
def crf(n): return i / (1 - (1 + i) ** (-n))
w = int(floor(tc)); q = tc - w
def f_idc(): return ((1 + i) ** tc - 1) / (i * tc)
def f_dcf(): return (q * (1 + i) ** w + ((1 + i) ** w - 1) / i) / tc
def h(u): return ((1 + i) ** (tc * (1 - u)) - 1) / (i * tc)
def pd(cr, g):
    buckets = ([(-w, cr * q / tc)] if q > 0 else []) + [(k - w + 1, cr / tc) for k in range(w)]
    spent = mpf(0); bal = mpf(0); prev = None
    for d, b in buckets:
        if prev is not None: bal *= (1 + i) ** (d - prev)
        gg = min(b, max(mpf(0), g - spent)); bal += b - gg; spent += b; prev = d
    return bal * (1 + i) ** (-prev)
key = lambda r: (r["stage"], r["life_y"], r["definition"], r["timing"])
cr = {}; lac0 = {}
for r in rows:
    if r["definition"] == "A" and r["row_kind"] == "grid" and r["share"] == "0.000000":
        cr[(r["stage"], r["life_y"])] = mpf(r["eligible_capital_sgdm"])
        lac0[(r["stage"], r["life_y"], r["method"])] = mpf(r["cost_gross_s_per_t"])
worst = mpf(0); n = 0
for r in rows:
    if r["row_kind"] != "grid": continue
    st, life = r["stage"], r["life_y"]
    C = cr[(st, life)]; E = mpf(r["eligible_capital_sgdm"]); X = E - C
    s = mpf(r["share"]); a = mpf(r["abatement_t_y"])
    kappa = crf(int(life)) * mpf(10) ** 6 / a
    L0 = lac0[(st, life, r["method"])]
    G = s * E
    if r["method"] == "L5":
        f = f_idc()
        red = s * (C * f + X) if r["timing"] == "PROGRESSIVE" else C * (f - h(min(G / C, 1))) + max(mpf(0), G - C)
    else:
        f = f_dcf()
        red = s * (C * f + X) if r["timing"] == "PROGRESSIVE" else C * f - pd(C, min(G, C)) + max(mpf(0), G - C)
    want = L0 - kappa * red
    got = mpf(r["cost_net_s_per_t"])
    worst = max(worst, abs(want - got)); n += 1
    assert (got < 100) == (r["target_met"] == "true"), r
print(n, "grid rows; largest difference from the 40-digit closed form:", mp.nstr(worst, 5), "S$/t (CSV prints six decimals)")
