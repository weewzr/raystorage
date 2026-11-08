exec(open("recompute.py").read().split("rows = list(csv.DictReader")[0])
rows = list(csv.DictReader(open(f"{REPO}/results/Subsidy_Scenarios.csv", newline="")))
def cell(stage, L, d, timing, kind, method, share=None):
    for r in rows:
        if (r["stage"], int(r["life_y"]), r["definition"], r["timing"], r["row_kind"], r["method"]) == (stage, L, d, timing, kind, method):
            if share is None or abs(float(r["share"]) - share) < 1e-9: return r
print("BOAK 25 y, definition A: (mine | CSV)")
K, Y, L5s, A = items("BOAK", 25)
for timing in ["PROGRESSIVE", "AT_CLOSE"]:
    for s in [0.25, 0.5, 0.75]:
        m5 = lac_5(K, L5s, 25, A, "A", s, timing); md = lac_d(K, Y, 25, A, "A", s, timing)
        c5 = float(cell("BOAK", 25, "A", timing, "grid", "L5", s)["cost_net_s_per_t"]); cd = float(cell("BOAK", 25, "A", timing, "grid", "DCF", s)["cost_net_s_per_t"])
        print(f"  {timing:11s} s={s:4.2f}  L5 {m5:10.6f} | {c5:10.6f}   DCF {md:10.6f} | {cd:10.6f}")
print("zero subsidy BOAK 25: L5", round(lac_5(K, L5s, 25, A, 'A', 0.0, 'PROGRESSIVE'), 6), "DCF", round(lac_d(K, Y, 25, A, 'A', 0.0, 'PROGRESSIVE'), 6))
print("BOAK 60 y, definition B at close, s=0.5:", round(lac_5(*[items('BOAK',60)[i] for i in (0,2)][:1], items('BOAK',60)[2], 60, items('BOAK',60)[3], 'B', 0.5, 'AT_CLOSE'), 6) if False else "")
# break-even BOAK 25 A progressive and the other three
for d in "AB":
    for timing in ["PROGRESSIVE", "AT_CLOSE"]:
        b5 = infimum(lambda s: lac_5(K, L5s, 25, A, d, s, timing)); bd = infimum(lambda s: lac_d(K, Y, 25, A, d, s, timing))
        p5 = float(cell("BOAK", 25, d, timing, "breakeven", "L5")["share"]); pd_ = float(cell("BOAK", 25, d, timing, "breakeven", "DCF")["share"]); pb = float(cell("BOAK", 25, d, timing, "breakeven", "BOTH")["share"])
        print(f"  BE {d} {timing:11s} L5 inf {b5:.8f} printed {p5:.6f} | DCF inf {bd:.8f} printed {pd_:.6f} | BOTH printed {pb:.6f} (max inf {max(b5,bd):.8f})")
# CSV column checks: private capital, interest columns, state PV, outlay
worst = collections.defaultdict(float)
def u(n, a, b): worst[n] = max(worst[n], abs(a-b))
for r in rows:
    if r["row_kind"] != "grid" and r["method"] == "BOTH": continue
    stage, L, d, timing = r["stage"], int(r["life_y"]), r["definition"], r["timing"]; s = float(r["share"]); meth = r["method"]
    K, Y, L5s, A = items(stage, L)
    E, G, fl = grant_flows(K, d, s, timing)
    Ktot = K["R"] + K["CAP"] + K["IHX"] + K["LIC"] + K["PI"] + K["REC"]
    u("total_capital", Ktot, float(r["total_capital_sgdm"])); u("private_capital", Ktot - G, float(r["private_capital_sgdm"]))
    CR = K["R"]; f = ((1+i)**tc-1)/(i*tc)
    bk = buckets(CR)
    if meth == "L5":
        gross = CR*(f-1)
        if timing == "PROGRESSIVE": priv = (1-s)*gross
        else:
            uu = min(G, CR)/CR; tp = tc*(1-uu); priv = CR*(1-uu)*(((1+i)**tp-1)/(i*tp)-1) if tp > 0 else 0.0
        state = G + gross - priv if timing == "PROGRESSIVE" else G*(1+i)**tc
        u("L5_int_gross", gross, float(r["interest_gross_sgdm"])); u("L5_int_priv", priv, float(r["interest_private_sgdm"])); u("L5_int_saved", gross-priv, float(r["interest_saved_sgdm"]))
        u("L5_state_pv", state, float(r["state_pv_sgdm"]))
        u("L5_annual_net", (L5s - crf(L)*(G+gross-priv)), float(r["annual_cost_net_sgdm_y"]))
        u("L5_outlay", state*crf(L)*1e6/A, float(r["public_outlay_s_per_t"]))
    elif meth == "DCF":
        gross = pv(bk) - CR
        # private reactor payments = bucket minus grant applied to reactor buckets
        gr = [(y, -v) for y, v in fl if True]
        react_g = {}
        for y, v in fl: react_g[y] = react_g.get(y, 0.0) + (-v)
        # reactor part of the grant: dated flows at bucket dates, but year-0 flow may mix reactor bucket and excess; split using reactor spend caps
        tot_reactor_grant = min(G, CR) if timing == "AT_CLOSE" else s*CR
        if timing == "PROGRESSIVE":
            privflows = [(y, (1-s)*b) for y, b in bk]
        else:
            pot = min(G, CR); privflows = []
            for y, b in bk:
                use = min(b, pot); pot -= use; privflows.append((y, b-use))
        priv = pv(privflows) - sum(v for _, v in privflows)
        state = pv([(y, -v) for y, v in fl]) if timing == "PROGRESSIVE" else G*(1+i)**tc
        u("DCF_int_gross", gross, float(r["interest_gross_sgdm"])); u("DCF_int_priv", priv, float(r["interest_private_sgdm"])); u("DCF_int_saved", gross-priv, float(r["interest_saved_sgdm"]))
        u("DCF_state_pv", state, float(r["state_pv_sgdm"]))
        u("DCF_outlay", state*crf(L)*1e6/A, float(r["public_outlay_s_per_t"]))
        lacd = lac_d(K, Y, L, A, d, s, timing)
        u("DCF_annual_net", lacd*A*1e-6, float(r["annual_cost_net_sgdm_y"]))
        l5 = lac_5(K, L5s, L, A, d, s, timing)
        u("DCF_gap_to_l5", lacd - l5, float(r["gap_to_l5_s_per_t"])); u("DCF_gap_rel", (lacd-l5)/l5, float(r["gap_share_of_l5"]))
for k, v in sorted(worst.items()): print(f"  col {k:18s} max abs diff {v:.2e}")
