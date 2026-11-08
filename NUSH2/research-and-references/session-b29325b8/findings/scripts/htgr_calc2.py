"""Final reactor-size cases. All outputs are OWN CALCULATIONS."""
CP = 5195.0
ex = 0.4
ETA_C, ETA_M = 0.397, 0.90


def circ(md, T1, PMPa, dPkPa, eta):
    return md * CP * T1 * ((1 + dPkPa / 1e3 / PMPa) ** ex - 1) / eta / 1e6


def rsize(Eaux, W, qv, qd):
    return 176.8 + (Eaux + W / ETA_M) / ETA_C + qv + qd - W


cases = [
    ("Low", 7.0, 150, 100, 0.80, 2.96, 0.45),
    ("Central", 7.0, 200, 150, 0.75, 2.96, 0.45),
    ("Central, 5.15 MPa", 5.15, 200, 150, 0.75, 2.96, 0.45),
    ("Central, 500 m duct", 7.0, 200, 150, 0.75, 2.96, 3.12),
    ("High", 5.15, 300, 200, 0.65, 5.67, 3.12),
]
print("case | P MPa | dPs | dPp | eta | Wsec | Wprim | Wtot | Eaux MWe | Pcycle MWth | Reactor MWth | short of 210")
for lab, P, dps, dpp, eta, qv, qd in cases:
    ws = circ(78.49, 739, P, dps, eta)
    wp = circ(92.2, 764, P, dpp, eta)
    w = ws + wp
    e = 11.6 + w / ETA_M
    r = rsize(11.6, w, qv, qd)
    print("%s | %.2f | %d | %d | %.2f | %.1f | %.1f | %.1f | %.1f | %.1f | %.1f | %+.1f" % (lab, P, dps, dpp, eta, ws, wp, w, e, e / ETA_C, r, r - 210))
print("INL implied primary 12.35 MW only:", round(rsize(11.6, 12.35, 2.96, 0.45), 1))
print("no circ, losses only:", round(rsize(11.6, 0, 2.96, 0.45), 1))
print("Electric load uplift central: %.0f%%" % (100 * (circ(78.49, 739, 7.0, 200, .75) + circ(92.2, 764, 7.0, 150, .75)) / 0.9 / 11.6))
print("HTR-Modul mean core T (300+950)/2 =", (300 + 950) / 2, "NUS-H2 (491+925)/2 =", (491 + 925) / 2, "(516+925)/2 =", (516 + 925) / 2)
print("11.6/0.475 =", round(11.6 / 0.475, 1), "reactor", round(176.8 + 11.6 / 0.475, 1))
