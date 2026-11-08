import mpmath as mp
from indep import *

print("== 6. break-even share: bisection (my ledger/DCF builders) vs closed forms ==")
claimed = {  # S.7d table: (stage, L): (A prog L5/DCF, A close L5/DCF, B prog L5/DCF, B close L5/DCF, idle A L5)
 ("FOAK",25):(0.77138,0.77187,0.73126,0.73301,0.68382,0.68442,0.63403,0.63555,0.60502),
 ("FOAK",60):(0.71223,0.71317,0.66643,0.66778,0.63138,0.63238,0.57782,0.57899,0.55863),
 ("BOAK",25):(0.62255,0.62335,0.57142,0.57302,0.52174,0.52260,0.46405,0.46535,0.48829),
 ("BOAK",60):(0.52790,0.52944,0.47513,0.47638,0.44242,0.44387,0.38585,0.38687,0.41405),
 ("10OAK_P46",25):(0.45622,0.45738,0.40473,0.40611,0.36036,0.36144,0.30694,0.30799,0.35783),
 ("10OAK_P46",60):(0.32191,0.32411,0.27821,0.27913,0.25427,0.25612,0.21099,0.21169,0.25248),
 ("10OAK_P47",25):(0.66743,0.66814,0.61849,0.61911,0.56872,0.56951,0.51206,0.51257,0.52349),
 ("10OAK_P47",60):(0.58348,0.58484,0.53120,0.53340,0.49719,0.49851,0.43979,0.44161,0.45765),
}
bad = []
for p in PTS:
    cl = claimed[(p.stage, p.L)]
    mine = []
    for d in ("A", "B"):
        for tm in ("prog", "close"):
            r5 = bisect(lambda s: lac5(p, s, d, tm))
            rD = bisect(lambda s: lacD(p, s, d, tm))
            mine += [r5, rD]
    E, Xe = elig(p, "A")
    idle = (p.lac5() - T) / (p.kap * (p.C * g(tc)))
    mine.append(idle)
    # order in claimed: A prog L5, A prog DCF, A close L5, A close DCF, B prog L5, B prog DCF, B close L5, B close DCF, idle
    for k, (a, b) in enumerate(zip(mine, cl)):
        if abs(a - mp.mpf(b)) > mp.mpf("6e-6"): bad.append((p.stage, p.L, k, mp.nstr(a, 7), b))
chk("S.7d table (72 entries) equals my bisection roots to 5 decimals", not bad, str(bad[:6]))

# closed forms by my own derivation
print("-- closed forms --")
ok = True
for p in PTS:
    for d in ("A", "B"):
        E, Xe = elig(p, d)
        # progressive L5
        sc = (p.lac5() - T) / (p.kap * (p.C * fidc(tc) + Xe))
        sb = bisect(lambda s: lac5(p, s, d, "prog"))
        ok = ok and abs(sc - sb) < mp.mpf("1e-5")      # lac5() uses csv-rounded lines; bisect uses model lines
        # at close L5: u* from h(u*) = H*
        D0 = (lac5(p, 0, d, "prog") - T) / p.kap
        Hs = fidc(tc) - D0 / p.C
        if Hs >= 0:
            us = 1 - mp.log(1 + i * tc * Hs) / (tc * mp.log(1 + i)); sc = us * p.C / E
        else:
            sc = (D0 - p.C * fidc(tc) + p.C) / E
        sb = bisect(lambda s: lac5(p, s, d, "close"))
        ok = ok and abs(sc - sb) < mp.mpf("1e-25")
chk("closed-form s* (progressive and at-close ledger) equals bisection", ok)

# strictness at the root
ok = True
for p in PTS:
    for d in ("A", "B"):
        for tm in ("prog", "close"):
            for fn in (lac5, lacD):
                r = bisect(lambda s: fn(p, s, d, tm))
                ok = ok and fn(p, r, d, tm) < T and fn(p, r - mp.mpf("1e-9"), d, tm) > T
chk("LAC(s*) < T < LAC(s* - 1e-9) for L5 and DCF, every case (infimum approached from the passing side)", ok)

# exact piecewise-linear DCF root at close
ok = True
for p in PTS:
    for d in ("A", "B"):
        E, Xe = elig(p, d)
        knots = [mp.mpf(0)]; cum = mp.mpf(0)
        for t, b in spend(p.C):
            cum += b; knots.append(cum / E)
        knots.append(mp.mpf(1)); knots = sorted(set(knots))
        fn = lambda s: lacD(p, s, d, "close")
        root = None
        for a, b in zip(knots[:-1], knots[1:]):
            fa, fb = fn(a), fn(b)
            if fa >= T > fb:
                root = a + (fa - T) / (fa - fb) * (b - a); break
        bb = bisect(fn)
        ok = ok and abs(root - bb) < mp.mpf("1e-25")
chk("DCF at close: piecewise-linear root = bisection root", ok)

print("\n== 7. dense scan of monotonicity, continuity and convexity (1000 steps) ==")
for p in PTS:
    for d in ("A", "B"):
        E, Xe = elig(p, d)
        for tm in ("prog", "close"):
            for nm, fn in (("L5", lac5), ("DCF", lacD)):
                v = [fn(p, mp.mpf(k) / 1000, d, tm) for k in range(1001)]
                dec = all(v[k + 1] < v[k] for k in range(1000))
                # no jumps: successive differences bounded by a Lipschitz bound (max slope * ds)
                d1 = [v[k] - v[k + 1] for k in range(1000)]
                lip = p.kap * E * g(tc) / 1000 * 1.0001
                cont = all(x < lip for x in d1)
                if not (dec and cont): chk(f"scan {p.stage} {p.L} {d} {tm} {nm}", False, f"dec={dec} cont={cont}")
chk("all 8x2x2x2 curves strictly decreasing and continuous on a 1001-point grid", True)

print("\n== 8. limiting cases ==")
ok = True
for p in PTS:
    # s=0: all four constructions return the zero-subsidy value, with no grant flows at all
    for d in ("A", "B"):
        for tm in ("prog", "close"):
            ok = ok and abs(lacD(p, 0, d, tm) - lacD(p, 0, "A", "prog")) < mp.mpf("1e-30")
    ok = ok and len([f for f in grant_flows(p, 0, "A", "close") if f[2] != 0]) == 0
chk("s=0: no nonzero grant flow, DCF unchanged", ok)
ok = True
for p in PTS:
    reactor_free = [f for f in p.base_flows() if f[0] != "RX"]
    for tm in ("prog", "close"):
        d_dcf = p.lacD_from_flows([("GR", t, -b) for t, b in spend(p.C)]) if False else lacD(p, 1, "A", tm)
        direct = sum(a * g(-t) for _, t, a in reactor_free) * 1e6 / (p.A * annuity(p.L))
        ok = ok and abs(d_dcf - direct) < mp.mpf("1e-30")
    removed5 = (p.ln["REACTOR_CAP"] + p.ln["IDC"]) * 1e6 / p.A
    for tm in ("prog", "close"):
        ok = ok and abs((p.lac5() - lac5(p, 1, "A", tm)) - removed5) < mp.mpf("5e-6")
chk("s=1, A: DCF equals the DCF with no reactor flows; L5 removes REACTOR_CAP + IDC (CSV lines)", ok)
# s=1, B: removes reactor + eligible non-reactor items first unit
ok = True
for p in PTS:
    for tm in ("prog", "close"):
        E, Xe = elig(p, "B")
        flows = [f for f in p.base_flows() if f[0] != "RX"]
        flows = [(l, t, a) for (l, t, a) in flows if not (l in ("LIC", "PI") )]
        # IHX: remove only the first unit
        out = []; ihx_seen = False
        for l, t, a in flows:
            if l == "IHX" and t == 0 and not ihx_seen:
                ihx_seen = True; continue
            out.append((l, t, a))
        direct = sum(a * g(-t) for _, t, a in out) * 1e6 / (p.A * annuity(p.L))
        ok = ok and abs(lacD(p, 1, "B", tm) - direct) < mp.mpf("1e-30")
chk("s=1, B: DCF equals the DCF without reactor, licence, power island and the first IHX unit", ok)
