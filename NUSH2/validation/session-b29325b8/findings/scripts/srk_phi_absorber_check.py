# Independent SRK check: phi of CO2 in a binary CO2 + "other" vapour, kij = 0,
# largest real root, standard SRK (Soave 1972) with the critical constants the
# engine states. Compares "other" = H2O (engine's binary) against "other" = H2.
import numpy as np
R = 8.314462618
def phi_co2(T, P, y, other):
    crit = {'CO2': (304.12, 7.374e6, 0.225), 'H2O': (647.1, 22.064e6, 0.344), 'H2': (33.19, 1.313e6, -0.216)}
    comps = ['CO2', other]
    yv = np.array([y, 1 - y])
    a = []; b = []
    for c in comps:
        Tc, Pc, w = crit[c]
        m = 0.480 + 1.574*w - 0.176*w*w
        s = 1 + m*(1 - np.sqrt(T/Tc))
        a.append(0.42748*R*R*Tc*Tc/Pc*s*s); b.append(0.08664*R*Tc/Pc)
    a = np.array(a); b = np.array(b)
    amix = sum(yv[i]*yv[j]*np.sqrt(a[i]*a[j]) for i in range(2) for j in range(2))
    bmix = yv @ b
    A = amix*P/(R*T)**2; B = bmix*P/(R*T)
    roots = np.roots([1, -1, A - B - B*B, -A*B])
    real = sorted(r.real for r in roots if abs(r.imag) < 1e-12)
    Z = real[-1]
    a0 = yv[0]*a[0] + yv[1]*np.sqrt(a[0]*a[1])
    lnphi = b[0]/bmix*(Z-1) - np.log(Z-B) - A/B*(2*a0/amix - b[0]/bmix)*np.log(1+B/Z)
    return np.exp(lnphi), Z, real
for T in (313.15, 323.15, 333.15):
    for P in (1.0e5, 2.68e6):
        for y in (1e-4, 0.01, 0.15):
            pw, zw, rw = phi_co2(T, P, y, 'H2O')
            ph, zh, rh = phi_co2(T, P, y, 'H2')
            print(f"T={T} P={P:.3g} y={y:g}  binary-with-H2O phi={pw:.4g} Z={zw:.4g} nroots={len(rw)} | binary-with-H2 phi={ph:.4g} Z={zh:.4g}")
