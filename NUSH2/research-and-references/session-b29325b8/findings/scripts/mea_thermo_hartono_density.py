"""mea_thermo agent: test readings of Hartono Eq 3 and Eq 5 against Hartono Tables 2 and 3."""
import numpy as np

M1, M2, M3 = 61.08, 18.015, 44.01
AL30 = [0.0, 0.11, 0.19, 0.29, 0.39, 0.50]
T30 = {20: [1012.68, 1030.99, 1046.92, 1068.90, 1086.89, 1108.20], 30: [1008.31, 1026.65, 1042.58, 1064.53, 1082.44, 1103.51],
       40: [1003.45, 1021.88, 1037.84, 1059.81, 1077.75, 1098.66], 50: [998.07, 1016.60, 1032.62, 1054.64, 1072.60, 1093.45],
       60: [992.23, 1010.90, 1027.00, 1049.12, 1067.11, 1087.88], 70: [985.96, 1004.80, 1021.01, 1043.23, 1061.28, 1082.00],
       80: [979.27, 998.31, 1014.66, 1037.04, 1055.16, 1075.85]}
AL6 = [0.0, 0.10, 0.20, 0.36, 0.47]
T6 = {20: [1000.88, 1004.95, 1009.77, 1017.47, 1022.17], 40: [994.42, 998.47, 1003.09, 1010.79, 1015.13],
      60: [985.12, 989.10, 993.75, 1001.33, 1005.60], 80: [973.57, 977.56, 981.76, 989.89, 993.98]}


def x_from_w(w):
    return (w / M1) / (w / M1 + (1 - w) / M2)


def run(w, table, al, cube, use_mass):
    x1 = x_from_w(w)
    z = w if use_mass else x1
    r = []
    for t, row in table.items():
        for a, rho in zip(al, row):
            if a == 0:
                continue
            wc = a * x1 * M3 / (x1 * M1 + (1 - x1) * M2 + a * x1 * M3)
            phi = (0.29 * z * a + 0.18 * z) / (0.66 + z)
            f = phi**3 if cube else phi
            r.append((row[0] / (1 - wc * (1 - f))) / rho)
    r = np.array(r)
    return 100 * np.mean(abs(r - 1)), r.min(), r.max()


for lab, cube, mass in (("as printed: phi^3, x1 = mole fraction", True, False), ("phi^1, mole fraction", False, False),
                        ("phi^3, mass fraction", True, True), ("phi^1, mass fraction", False, True)):
    a30 = run(0.30, T30, AL30, cube, mass)
    a6 = run(0.062, T6, AL6, cube, mass)
    print(f"{lab:42s} 30 mass% AARD {a30[0]:.2f}% [{a30[1]:.3f},{a30[2]:.3f}]   6.2 mass% AARD {a6[0]:.2f}% [{a6[1]:.3f},{a6[2]:.3f}]")
x = x_from_w(0.30)
print("phi at 30 mass% alpha 0.5, mole fraction basis:", (0.29 * x * 0.5 + 0.18 * x) / (0.66 + x), " mass basis:", (0.29 * 0.3 * 0.5 + 0.18 * 0.3) / (0.66 + 0.3))
