exec(open("recompute.py").read().split("rows = list(csv.DictReader")[0])
K, Y, L5s, A = items("BOAK", 25)
for label, cap in (("B as built (capture eligible)", 1.0), ("B without capture plant", 0.0)):
    flags["B"]["CAPTURE"] = cap
    for timing in ("PROGRESSIVE", "AT_CLOSE"):
        be = max(infimum(lambda s: lac_5(K, L5s, 25, A, "B", s, timing)), infimum(lambda s: lac_d(K, Y, 25, A, "B", s, timing)))
        c50 = lac_5(K, L5s, 25, A, "B", 0.5, timing)
        print(f"{label:32s} {timing:11s} break-even(both) {be:.6f}  L5 at s=0.5 {c50:.4f}")
kappa = crf(25)*1e6/A
print("capture K", round(K["CAP"],3), "kappa*K_cap*0.5 =", round(kappa*K["CAP"]*0.5, 3), "S$/t")
