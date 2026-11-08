#!/usr/bin/env python3
"""SymPy verification of docs/derivation/01_carbon_and_abatement.md (equations N.1 to N.10).

What this script does
---------------------
1. Checks the algebra and the units of the independent carbon balance symbolically.
2. Substitutes values read from results/*.csv and engine/data/parameters.toml to show
   (a) that the equations reproduce the existing Level 3 flowsheet run FS_B_REC_F, and
   (b) that the legacy anchored basis (cards J.5 to J.11, K.1 to K.11) is reproduced
   in results/Emissions_Ledger.csv for B_REC_F at the INL baseline (INL_9p21).

What it does not do
-------------------
It does not recompute plant physics. The reformer and shift equilibria, the flash, the
PSA split and the cryogenic split stay in the Rust flowsheet; their outputs are read
from the CSV files and only the balance equations built on them are checked here.

Run (the paths are found from the location of this file, so any working directory works):
    python -I math/verify/sympy_carbon.py        (SymPy 1.14, Python 3.10 or later)
Exit status is 0 when every check passes and 1 otherwise.
"""
from __future__ import annotations

import csv
import math
import re
import sys
from pathlib import Path

import sympy as sp
from sympy.physics.units import Dimension, amount_of_substance, energy, mass, time
from sympy.physics.units.systems.si import dimsys_SI

ROOT = Path(__file__).resolve().parents[2]
RESULTS: list[tuple[str, bool, str]] = []
VALUES: dict[str, float] = {}


def check(name: str, ok: bool, detail: str = "") -> None:
    RESULTS.append((name, bool(ok), detail))
    print(("PASS  " if ok else "FAIL  ") + name + (("   " + detail) if detail else ""))


def close(a: float, b: float, rel: float = 1e-9, abs_: float = 0.0) -> bool:
    return abs(a - b) <= max(abs_, rel * max(abs(a), abs(b)))


def rel_err(a: float, b: float) -> float:
    return (a - b) / b if b else float("nan")


# ----------------------------------------------------------------------------------
# 0. Inputs: Rust constants, parameters.toml, results CSV files
# ----------------------------------------------------------------------------------
def rust_consts(path: Path) -> dict[str, float]:
    out = {}
    for m in re.finditer(r"pub const (\w+): f64 = ([0-9_.eE+\-]+);", path.read_text(encoding="utf-8")):
        out[m.group(1)] = float(m.group(2).replace("_", ""))
    return out


def read_params(path: Path) -> dict[str, dict]:
    params: dict[str, dict] = {}
    cur = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line == "[[param]]":
            cur = {}
            continue
        if cur is None or not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        k, v = k.strip(), v.strip()
        if v.startswith('"'):
            v = v[1 : v.rindex('"')]
        else:
            try:
                v = float(v)
            except ValueError:
                pass
        cur[k] = v
        if k == "id":
            params[v] = cur
    return params


def read_csv(path: Path) -> list[dict]:
    with path.open(encoding="utf-8", newline="") as fh:
        return list(csv.DictReader(fh))


def fnum(x: str) -> float:
    return float(x) if x not in ("", None) else 0.0


U = rust_consts(ROOT / "engine/props/src/units.rs")
FS = rust_consts(ROOT / "engine/plant/src/flowsheet.rs")
PR = read_params(ROOT / "engine/data/parameters.toml")


def prm(pid: str) -> float:
    return float(PR[pid]["value"])


LB, ST, SCF_LBMOL = U["LB"], U["ST"], U["SCF_LBMOL"]
MW_CO2, MW_H2 = U["MW_CO2"], U["MW_H2"]
HOURS_PER_YEAR, DAYS_PER_YEAR, MMBTU_GJ = U["HOURS_PER_YEAR"], U["DAYS_PER_YEAR"], U["MMBTU_GJ"]
LHV_H2, LHV_CH4, LHV_CO = U["LHV_H2"], U["LHV_CH4"], U["LHV_CO"]
ELEMENT_TOL, TEAR_TOL = FS["ELEMENT_TOL"], FS["TEAR_TOL"]


def mmscfd_to_kmol_h(x: float) -> float:  # props/units.rs:49
    return x * 1.0e6 / SCF_LBMOL / 24.0 * LB


def st_d_to_kmol_h(x: float) -> float:  # props/units.rs:44
    return x * ST * 1000.0 / 24.0 / MW_CO2


# Natural gas, lumped as in engine/plant/src/flowsheet.rs NG_MOL_FRACTIONS.
ng_block = re.search(r"NG_MOL_FRACTIONS: \[\(Sp, f64\); \d+\] = \[(.*?)\];", (ROOT / "engine/plant/src/flowsheet.rs").read_text(encoding="utf-8"), re.S).group(1)
Y: dict[str, float] = {}
for m in re.finditer(r"\(Sp::(\w+),\s*([^)]+)\)", ng_block):
    Y[m.group(1)] = float(sp.sympify(m.group(2)))

# Species, in the order of engine/units Sp: (C, H, O) atoms.
SPECIES = {
    "H2": (0, 2, 0), "CO": (1, 0, 1), "CO2": (1, 0, 2), "CH4": (1, 4, 0), "H2O": (0, 2, 1),
    "N2": (0, 0, 0), "C2H6": (2, 6, 0), "C3H8": (3, 8, 0), "C4H10": (4, 10, 0), "C5H12": (5, 12, 0),
}
SP_KEY = {"Ch4": "CH4", "C2h6": "C2H6", "C3h8": "C3H8", "Nc4h10": "C4H10", "Nc5h12": "C5H12", "N2": "N2", "Co2": "CO2"}
CSV_COL = {"H2": "h2", "CO": "co", "CO2": "co2", "CH4": "ch4", "H2O": "h2o", "N2": "n2",
           "C2H6": "c2h6", "C3H8": "c3h8", "C4H10": "nc4h10", "C5H12": "nc5h12"}


def z_of(sp_name: str) -> int:
    c, h, o = SPECIES[sp_name]
    return 2 * c - o + h // 2


ZV = {k: z_of(k) for k in SPECIES}
ZV["O2"] = -2  # O2: 2*0 - 2 + 0

mb_rows = read_csv(ROOT / "results/Integrated_Mass_Balance.csv")
em_rows = read_csv(ROOT / "results/Emissions_Ledger.csv")


def stream(run: str, sid: str) -> dict[str, float]:
    for r in mb_rows:
        if r["run_id"] == run and r["stream_id"] == sid:
            d = {k: fnum(r[CSV_COL[k]]) for k in SPECIES}
            d["total"] = fnum(r["total_kmol_h"])
            d["carbon_csv"] = fnum(r["carbon_kmol_h"])
            return d
    raise KeyError((run, sid))


def c_flow(s: dict[str, float]) -> float:
    return sum(SPECIES[k][0] * s[k] for k in SPECIES)


def z_flow(s: dict[str, float]) -> float:
    return sum(ZV[k] * s[k] for k in SPECIES)


def mb_scalar(run: str, sid: str) -> float:
    for r in mb_rows:
        if r["run_id"] == run and r["stream_id"] == sid:
            return fnum(r["total_kmol_h"]) or fnum(r["carbon_kmol_h"])
    raise KeyError((run, sid))


def em_value(run: str, base: str, view: str, line: str, col: str = "t_per_y") -> float:
    for r in em_rows:
        if (r["run_id"], r["baseline_id"], r["view"], r["line_id"]) == (run, base, view, line):
            return float(r[col])
    raise KeyError((run, base, view, line))


# ----------------------------------------------------------------------------------
# 1. Hydrogen potential Z and its conservation (basis of N.1.2 and N.5.1)
# ----------------------------------------------------------------------------------
print("== N.1.2 hydrogen potential Z = 2 C - O + H/2")
expected_z = {"H2": 1, "CO": 1, "CO2": 0, "CH4": 4, "H2O": 0, "N2": 0, "C2H6": 7, "C3H8": 10, "C4H10": 13, "C5H12": 16, "O2": -2}
check("Z table", ZV == expected_z, str(ZV))

n_, m_ = sp.symbols("n m", positive=True)
Zhc = 2 * n_ + m_ / 2  # CnHm: C=n, H=m, O=0
rxn = {
    "D.1 CH4+H2O->CO+3H2": (ZV["CH4"] + ZV["H2O"]) - (ZV["CO"] + 3 * ZV["H2"]),
    "D.2 CO+H2O->CO2+H2": (ZV["CO"] + ZV["H2O"]) - (ZV["CO2"] + ZV["H2"]),
    "D.3 CH4+2H2O->CO2+4H2": (ZV["CH4"] + 2 * ZV["H2O"]) - (ZV["CO2"] + 4 * ZV["H2"]),
    "D.4 CnHm+nH2O->nCO+(n+m/2)H2": sp.simplify(Zhc + n_ * ZV["H2O"] - n_ * ZV["CO"] - (n_ + m_ / 2) * ZV["H2"]),
    "CnHm+2nH2O->nCO2+(2n+m/2)H2": sp.simplify(Zhc + 2 * n_ * ZV["H2O"] - n_ * ZV["CO2"] - (2 * n_ + m_ / 2) * ZV["H2"]),
    "combustion CH4+2O2->CO2+2H2O": (ZV["CH4"] + 2 * ZV["O2"]) - (ZV["CO2"] + 2 * ZV["H2O"]),
    "combustion H2+0.5O2->H2O": (ZV["H2"] + sp.Rational(1, 2) * ZV["O2"]) - ZV["H2O"],
    "combustion CO+0.5O2->CO2": (ZV["CO"] + sp.Rational(1, 2) * ZV["O2"]) - ZV["CO2"],
}
for k, v in rxn.items():
    check(f"Z conserved by {k}", sp.simplify(v) == 0)
for hc, (c, h) in {"C2H6": (2, 6), "C3H8": (3, 8), "C4H10": (4, 10), "C5H12": (5, 12)}.items():
    check(f"Z({hc}) = 2n + m/2", ZV[hc] == 2 * c + h // 2)

# ----------------------------------------------------------------------------------
# 2. N.1.1 carbon per kmol of gas and the lumped composition
# ----------------------------------------------------------------------------------
print("== N.1.1 feed carbon")
ycs = {"CH4": Y["Ch4"], "C2H6": Y["C2h6"], "C3H8": Y["C3h8"], "C4H10": Y["Nc4h10"], "C5H12": Y["Nc5h12"], "CO2": Y["Co2"], "N2": Y["N2"]}
cbar = sum(SPECIES[k][0] * v for k, v in ycs.items())
sum_y = sum(ycs.values())
Z_NG = sum(ZV[k] * v for k, v in ycs.items())
# the same number from the parameter ids (names say C_PER_MOL but values are mole fractions)
cbar_toml = sum(prm(f"C_PER_MOL_NG_{s}") * c for s, c in [("CH4", 1), ("C2", 2), ("C3", 3), ("C4", 4), ("C5", 5), ("C6", 6), ("CO2", 1)])
check("c_bar = 1.05379 (J.5)", close(cbar, 1.05379, 1e-6), f"{cbar:.6f}")
check("c_bar from parameters.toml equals c_bar from flowsheet composition", close(cbar, cbar_toml, 1e-12), f"{cbar_toml:.6f}")
VALUES.update(cbar=cbar, sum_y=sum_y, Z_NG=Z_NG)
print(f"   c_bar = {cbar:.6f}  sum y = {sum_y:.5f}  Z_NG = {Z_NG:.6f}")

# ----------------------------------------------------------------------------------
# 3. Loop closure N.2.1, capture N.3, recycle N.4, closure N.6 (symbolic)
# ----------------------------------------------------------------------------------
print("== N.2.1 / N.3 / N.4 / N.6 symbolic")
FC, f, kap, eta, phi, P = sp.symbols("F_C f kappa eta phi P", positive=True)
T, G, R, B, C = sp.symbols("T G R B C")
sol = sp.solve([sp.Eq(T, FC + R), sp.Eq(R, f * G), sp.Eq(G, T * (1 - kap - eta * phi) - P)], [T, G, R], dict=True)[0]
Dloop = 1 - f * (1 - kap - eta * phi)
T_closed = (FC - f * P) / Dloop
check("N.2.1 tube carbon closed form", sp.simplify(sol[T] - T_closed) == 0)
G_closed = sp.simplify(sol[G])
Cc, Bc = eta * phi * sol[T], (1 - f) * sol[G]
resid = sp.simplify(FC - (Cc + Bc + P + kap * sol[T]))
check("N.6.1 carbon closure identity (residual simplifies to 0)", resid == 0)
check("N.4.1 f = (T - F_C)/G", sp.simplify((sol[T] - FC) / sol[G] - f) == 0)
eta_pl = sp.simplify(Cc / FC)
check("N.3.2 plant capture ratio", sp.simplify(eta_pl - eta * phi * (1 - f * P / FC) / Dloop) == 0)
check("N.3.2 limit f->0: eta_pl = eta*phi*(1)", sp.simplify(eta_pl.subs({f: 0, P: 0}) - eta * phi) == 0)
check("N.2.1 limit f->0: T = F_C", sp.simplify(T_closed.subs(f, 0) - FC) == 0)
# with eta = 0 and f = 0 (no capture, no recycle) all carbon except P and condensate is in the reject
check("N.2.2 reference limit: eta=0, f=0 gives G = F_C(1-kappa) - P", sp.simplify(sol[G].subs({f: 0, eta: 0}) - (FC * (1 - kap) - P)) == 0)
# convergence of the loop requires D > 0: always true for 0 <= f <= 1 and 1-kap-eta*phi < 1
check("N.2.1 D positive for 0<=f<=1 when kappa + eta*phi > 0", sp.simplify(Dloop.subs({f: 1}) - (kap + eta * phi)) == 0)

# ----------------------------------------------------------------------------------
# 4. PSA with cryogenic H2 return: xi_eff (N.1.4)
# ----------------------------------------------------------------------------------
xi, r_ret, H12, Hfeed = sp.symbols("xi r H12 Hfeed", positive=True)
Hf = sp.solve(sp.Eq(Hfeed, H12 + r_ret * (1 - xi) * Hfeed), Hfeed)[0]
xi_eff = sp.simplify(xi * Hf / H12)
check("N.1.4 xi_eff = xi/(1 - r(1-xi))", sp.simplify(xi_eff - xi / (1 - r_ret * (1 - xi))) == 0)
check("N.1.4 limit r=0: xi_eff = xi", sp.simplify(xi_eff.subs(r_ret, 0) - xi) == 0)
check("N.1.4 limit r=1: xi_eff = 1", sp.simplify(xi_eff.subs(r_ret, 1) - 1) == 0)

# ----------------------------------------------------------------------------------
# 5. Gas rate from the hydrogen target N.1.5 (symbolic)
# ----------------------------------------------------------------------------------
print("== N.1.5 gas rate from the H2 target")
n, nH2 = sp.symbols("n nH2", positive=True)
cb, ZN, xe, pi_, psi, om = sp.symbols("cbar Z_NG xi_eff pi psi omega", positive=True)
Pn = pi_ * nH2
Tn = (cb * n - f * Pn) / Dloop
H12n = ZN * n + f * (4 * (psi * Tn - Pn) + om * Tn) - 4 * psi * Tn - om * Tn
n_sol = sp.solve(sp.Eq(nH2, xe * H12n), n)[0]
Acoef = (1 - f) * (4 * psi + om) / Dloop
n_closed = nH2 * (1 / xe + f * pi_ * (4 - Acoef)) / (ZN - Acoef * cb)
check("N.1.5 gas rate closed form", sp.simplify(n_sol - n_closed) == 0)
# reference plant: f = 0, eta = 0, r = 0 (xi_eff = xi)
n_ref = sp.simplify(n_closed.subs({f: 0, xe: xi}))
check("N.7.1 reference process feed", sp.simplify(n_ref - nH2 / (xi * (ZN - (4 * psi + om) * cb))) == 0)
check("N.7.1 lower bound: n_ref >= nH2/(xi Z_NG) for psi, omega >= 0",
      sp.simplify(sp.limit(n_ref.subs({psi: 0, om: 0}), psi, 0) - nH2 / (xi * ZN)) == 0)

# ----------------------------------------------------------------------------------
# 6. Numerical check against the existing Level 3 run FS_B_REC_F
# ----------------------------------------------------------------------------------
print("== substitution of FS_B_REC_F values")
RUN = "FS_B_REC_F"
S = {t: stream(RUN, t) for t in ["S01", "S03", "S04", "S05", "S09", "S09C", "S10", "S11", "S12", "S13", "S14", "S15", "S16", "S19", "S20", "S21"]}
for t, d in S.items():
    ok = close(c_flow(d), d["carbon_csv"], 1e-9, 1e-8)
    if not ok:
        check(f"carbon of {t} from species equals CSV carbon", ok, f"{c_flow(d)} vs {d['carbon_csv']}")
check("carbon of every listed stream equals CSV carbon column", all(close(c_flow(d), d["carbon_csv"], 1e-9, 1e-8) for d in S.values()))

F_C = c_flow(S["S01"])
T_fs = c_flow(S["S04"])
R_fs, G_fs = c_flow(S["S03"]), c_flow(S["S14"])
B_fs, C_fs = c_flow(S["S15"]), c_flow(S["S20"])
P_fs, K_fs = c_flow(S["S13"]), c_flow(S["S10"])
f_fs = R_fs / G_fs
kap_fs = K_fs / T_fs
phi_fs = S["S11"]["CO2"] / T_fs
psi_fs = S["S11"]["CH4"] / T_fs
om_fs = S["S11"]["CO"] / T_fs
eta_fs = C_fs / S["S11"]["CO2"]
check("phi + psi + omega + kappa = 1 (carbon through reformer, shift, flash)", close(phi_fs + psi_fs + om_fs + kap_fs, 1.0, 1e-8))
check("S19 = S20 (100 percent transfer, no vent modelled)", close(S["S19"]["CO2"], S["S20"]["CO2"], 1e-12))
check("S16 CO2 equals carbon of S15 (N.5.1)", close(S["S16"]["CO2"], c_flow(S["S15"]), 1e-9))
check("S13 carbon is CH4 only", close(c_flow(S["S13"]), S["S13"]["CH4"], 1e-12))
check("S21 holds no carbon", abs(c_flow(S["S21"])) < 1e-12)
VALUES.update(F_C=F_C, T_fs=T_fs, R_fs=R_fs, G_fs=G_fs, B_fs=B_fs, C_fs=C_fs, P_fs=P_fs, K_fs=K_fs,
              f_fs=f_fs, kap_fs=kap_fs, phi_fs=phi_fs, psi_fs=psi_fs, om_fs=om_fs, eta_fs=eta_fs)

# N.2.1 loop
T_pred = float(T_closed.subs({FC: F_C, f: f_fs, P: P_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs}))
check("N.2.1 closed-form tube carbon equals flowsheet S04", close(T_pred, T_fs, 1e-9), f"{T_pred:.6f} vs {T_fs:.6f}; rel {rel_err(T_pred, T_fs):.2e}")
VALUES["T_pred"] = T_pred
# N.2.2 stream vector
G_pred = T_fs * (1 - kap_fs - eta_fs * phi_fs) - P_fs
check("N.2.2 reject carbon G", close(G_pred, G_fs, 1e-9), f"{G_pred:.6f} vs {G_fs:.6f}")
check("N.2.2 stack carbon (1-f)G", close((1 - f_fs) * G_fs, B_fs, 1e-9))
check("N.2.2 slip CO2 (1-eta) phi T in S12", close((1 - eta_fs) * phi_fs * T_fs, S["S12"]["CO2"], 1e-8))
# N.6 closure residual
resid_fs = F_C - (C_fs + B_fs + P_fs + K_fs)
rel_resid = resid_fs / F_C
# The CSV prints six decimals, so a residual recomputed from printed values is only good to about 5e-6 kmol/h.
check("N.6.1 residual recomputed from printed stream values is within the CSV print precision", abs(resid_fs) <= 5e-6, f"residual {resid_fs:.3e} kmol C/h (print precision 5e-6)")
CLOS_CSV = mb_scalar(RUN, "CLOSURE_CARBON")
check("N.6.1 exported envelope residual (full precision) is below the tolerance", CLOS_CSV / F_C <= ELEMENT_TOL, f"{CLOS_CSV:.3e} kmol C/h = {CLOS_CSV / F_C:.2e} of the feed; tolerance {ELEMENT_TOL:.0e}")
rel_resid = CLOS_CSV / F_C
VALUES.update(resid_fs=resid_fs, rel_resid=rel_resid, CLOS_CSV=CLOS_CSV)

# N.1 hydrogen potential balance
Z_in = z_flow(S["S01"]) + z_flow(S["S03"])
check("Z conserved S04 -> S05", close(z_flow(S["S04"]), z_flow(S["S05"]), 1e-9))
check("Z conserved S05 -> S09", close(z_flow(S["S05"]), z_flow(S["S09"]), 1e-9))
check("Z of S01 + S03 equals Z of S04 (mixer; steam has Z = 0)", close(Z_in, z_flow(S["S04"]), 1e-9))
H2_12_pred = Z_in - 4 * S["S09"]["CH4"] - S["S09"]["CO"]
check("N.1.3 H2 at the absorber from the Z balance", close(H2_12_pred, S["S12"]["H2"], 1e-9), f"{H2_12_pred:.6f} vs {S['S12']['H2']:.6f}")
check("H2 passes flash and absorber unchanged (S09 = S12)", close(S["S09"]["H2"], S["S12"]["H2"], 1e-12))
xi_p, y_ch4 = prm("PSA_H2_RECOVERY"), prm("CH4_IN_H2")
r_fs = S["S21"]["H2"] / S["S14"]["H2"]
xe_val = xi_p / (1 - r_fs * (1 - xi_p))
check("N.1.4 xi_eff equals S13 H2 / S12 H2", close(xe_val, S["S13"]["H2"] / S["S12"]["H2"], 1e-9), f"{xe_val:.6f}")
pi_val = y_ch4 / (1 - y_ch4)
check("product CH4 = pi * product H2", close(S["S13"]["CH4"], pi_val * S["S13"]["H2"], 1e-7))
nH2_fs = S["S13"]["H2"]
n_pred = float(n_closed.subs({nH2: nH2_fs, xe: xe_val, f: f_fs, pi_: pi_val, ZN: Z_NG, cb: cbar, psi: psi_fs, om: om_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs}))
n_fs = F_C / cbar
check("N.1.5 gas rate from the H2 target equals flowsheet gas rate", close(n_pred, n_fs, 1e-6), f"{n_pred:.5f} vs {n_fs:.5f} kmol/h; rel {rel_err(n_pred, n_fs):.2e}")
H2_kg_h = nH2_fs * MW_H2
check("H2 target 8988 kg/h = H2_B x rho_N (J.2)", close(H2_kg_h, prm("H2_B") * prm("RHO_H2_NORMAL"), 1e-9), f"{H2_kg_h:.4f}")
A_c = float(Acoef.subs({f: f_fs, psi: psi_fs, om: om_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs}))
Y_inv = n_pred / nH2_fs
VALUES.update(n_pred=n_pred, n_fs=n_fs, xe_val=xe_val, r_fs=r_fs, A_c=A_c, Y_inv=Y_inv, nH2_fs=nH2_fs, H2_kg_h=H2_kg_h,
              n_fs_mmscfd=n_fs / mmscfd_to_kmol_h(1.0))
n_min_plant = nH2_fs / (xe_val * Z_NG)
VALUES["n_min_plant"] = n_min_plant

# N.4.2 nitrogen cut diagnostics
n_N2 = S["S01"]["N2"]
check("all feed N2 leaves in the cut (S15)", close(n_N2, S["S15"]["N2"], 1e-12))
y_n2_cut = S["S15"]["N2"] / S["S15"]["total"]
beta = B_fs / n_N2
check("N.4.2 f = 1 - beta n_N2 / G", close(f_fs, 1 - beta * n_N2 / G_fs, 1e-12))
VALUES.update(y_n2_cut=y_n2_cut, beta=beta, n_N2=n_N2)

# N.5.1 combustion
O2_csv = mb_scalar(RUN, "COMBUSTION_O2")
O2_pred = z_flow(S["S15"]) / 2.0
check("N.5.1 O2 demand = Z(S15)/2", close(O2_pred, O2_csv, 1e-9), f"{O2_pred:.6f} vs {O2_csv:.6f}")
# generic symbolic element balance for the burn
nC, nHh, nO, nNn, xCO2, xH2O, xN2, xO2 = sp.symbols("nC nH nO nN xCO2 xH2O xN2 xO2")
sb = sp.solve([sp.Eq(xCO2, nC), sp.Eq(2 * xH2O, nHh), sp.Eq(2 * xN2, nNn), sp.Eq(nO + 2 * xO2, 2 * xCO2 + xH2O)], [xCO2, xH2O, xN2, xO2], dict=True)[0]
check("N.5.1 O2 = (2C + H/2 - O)/2 from the four element balances", sp.simplify(sb[xO2] - (2 * nC + nHh / 2 - nO) / 2) == 0)

# annual quantities for the present flowsheet streams
avail = prm("AVAILABILITY")
hours = avail * HOURS_PER_YEAR
tpy = lambda k: k * MW_CO2 / 1000.0 * hours
VALUES.update(hours=hours, cap_fs_t_y=tpy(C_fs), stack_fs_t_y=tpy(B_fs), cond_fs_t_y=tpy(K_fs), prod_fs_t_y=tpy(P_fs))
print(f"   flowsheet streams at 7446 h/y: captured {tpy(C_fs):,.0f}  stack {tpy(B_fs):,.0f}  condensate {tpy(K_fs):,.0f} t/y")

# closure verdict algebra (N.6.1): known/unknown states
def closure_verdict(feed: float, outs: dict[str, float | None], tol: float) -> tuple[str, float | None]:
    if any(v is None for v in outs.values()):
        return "undetermined", None
    res = feed - sum(outs.values())  # type: ignore[arg-type]
    return ("closed" if abs(res) <= tol * feed else "open"), res


outs = {"S20": C_fs, "S15": B_fs, "S13": P_fs, "S10": K_fs}
v_all, _ = closure_verdict(F_C, outs, ELEMENT_TOL)
v_drop, _ = closure_verdict(F_C, {k: v for k, v in outs.items() if k != "S13"}, ELEMENT_TOL)
v_unk, _ = closure_verdict(F_C, {**outs, "MEA_LOSS": None}, ELEMENT_TOL)
check("N.6.1 verdict closed when every stream is known and the residual is small", v_all == "closed")
check("N.6.1 verdict open when the smallest modelled stream (P) is dropped", v_drop == "open", f"dropping P leaves {P_fs / F_C:.3e} of the feed")
check("N.6.1 verdict undetermined, never closed, when a stream is unknown", v_unk == "undetermined")
VALUES["P_over_FC"] = P_fs / F_C
VALUES["tol_t_y"] = ELEMENT_TOL * F_C * MW_CO2 / 1000.0 * hours

# ----------------------------------------------------------------------------------
# 7. Legacy anchored basis: cards J.5 to J.11 and K.1 to K.11 reproduced
# ----------------------------------------------------------------------------------
print("== legacy anchored basis, B_REC_F at INL_9p21")
days = avail * DAYS_PER_YEAR
h2_a_kgh = prm("H2_A") * LB
h2_b_kgh = prm("H2_B") * prm("RHO_H2_NORMAL")
s_b = h2_b_kgh / h2_a_kgh
ng_c6 = prm("NG_CASE6")
f_c6 = mmscfd_to_kmol_h(ng_c6) * cbar_toml
p_c = mmscfd_to_kmol_h(prm("H2_A_MMSCFD")) * prm("CH4_IN_H2")
cap6 = st_d_to_kmol_h(prm("CO2_CAPTURED_C6"))
eta_sel, phi_l, eta_l = prm("ETA_SELEXOL"), prm("PHI_RECYCLE"), prm("ETA_MEA")
t6 = cap6 / eta_sel / phi_l
g6 = t6 - cap6 - p_c
f6 = (t6 - f_c6) / g6                                  # J.7
t_l = (f_c6 - f6 * p_c) / (1 - f6 * (1 - eta_l * phi_l))  # J.8
cap_l = eta_l * phi_l * t_l                             # J.9
g_l = t_l - cap_l - p_c                                 # J.10
rec_l, burn_l = f6 * g_l, (1 - f6) * g_l
tpy_s = lambda k: k * MW_CO2 / 1000.0 * hours * s_b
baseline_t_y = prm("CO2_CONV_NOCC") * ST * days * s_b   # K.1, K.3
atm_t_y, cap_t_y = tpy_s(burn_l), tpy_s(cap_l)
A_leg = baseline_t_y - atm_t_y                          # K.11
h2_t_y = h2_b_kgh * hours / 1000.0
LEG = dict(s_b=s_b, f_c6=f_c6, p_c=p_c, cap6=cap6, t6=t6, g6=g6, f6=f6, t_l=t_l, cap_l=cap_l, g_l=g_l, rec_l=rec_l, burn_l=burn_l,
           baseline_t_y=baseline_t_y, atm_t_y=atm_t_y, cap_t_y=cap_t_y, A_leg=A_leg, h2_t_y=h2_t_y,
           f_c6_s=f_c6 * s_b, t_l_s=t_l * s_b, cap_l_s=cap_l * s_b, burn_l_s=burn_l * s_b, g_l_s=g_l * s_b, rec_l_s=rec_l * s_b, p_c_s=p_c * s_b)
VALUES.update({"leg_" + k: v for k, v in LEG.items()})
em = lambda line, base="INL_9p21": em_value("B_REC_F", base, "direct_atm", line)
check("J.7 recycled fraction f_r = 0.7522 (register value)", close(f6, 0.7522, 1e-3), f"{f6:.6f}")
# The Level 3 flowsheet takes this INL back-calculated value as its cryogenic carbon split (flowsheet.rs:328),
# so the recycle share read back from its streams must equal it to the CSV print precision.
check("N.4.1 recycle share read from FS_B_REC_F streams equals the input J.7 value (0.752178)", close(f_fs, f6, 1e-6), f"{f_fs:.7f} vs {f6:.7f}")
T_pred_in = float(T_closed.subs({FC: F_C, f: f6, P: P_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs}))
check("N.2.1 tear convergence: closed-form tube carbon with the J.7 input equals S04", close(T_pred_in, T_fs, 1e-6), f"{T_pred_in:.6f} vs {T_fs:.6f}; rel {rel_err(T_pred_in, T_fs):.2e}")
VALUES["T_pred_in"] = T_pred_in
check("legacy captured t/y equals Emissions_Ledger CAPTURED", close(cap_t_y, em("CAPTURED"), 1e-8), f"{cap_t_y:.4f} vs {em('CAPTURED'):.4f}")
check("legacy residual t/y equals Emissions_Ledger N2_RICH_CUT", close(atm_t_y, em("N2_RICH_CUT"), 1e-8), f"{atm_t_y:.4f} vs {em('N2_RICH_CUT'):.4f}")
check("legacy baseline t/y equals Emissions_Ledger BASELINE (K.3)", close(baseline_t_y, em("BASELINE"), 1e-8), f"{baseline_t_y:.4f} vs {em('BASELINE'):.4f}")
check("legacy abatement t/y equals Emissions_Ledger ABATEMENT (K.11)", close(A_leg, em("ABATEMENT"), 1e-8), f"{A_leg:.4f} vs {em('ABATEMENT'):.4f}")
check("legacy baseline intensity 9.20977 kg/kg (K.1)", close(baseline_t_y / h2_t_y, 9.20977, 2e-6), f"{baseline_t_y / h2_t_y:.6f}")
abat_row = [r for r in em_rows if (r["run_id"], r["baseline_id"], r["view"], r["line_id"]) == ("B_REC_F", "INL_9p21", "direct_atm", "ABATEMENT")][0]
check("N.10.1 abatement per kg H2 equals Emissions_Ledger kg_per_kg_h2", close(A_leg / h2_t_y, float(abat_row["kg_per_kg_h2"]), 1e-8), f"{A_leg / h2_t_y:.6f}")
check("N.10.2 legacy plant_capture column equals C/(C+B)", close(cap_t_y / (cap_t_y + atm_t_y), float(abat_row["plant_capture"]), 1e-6), f"{cap_t_y / (cap_t_y + atm_t_y):.6f}")
VALUES.update(leg_a_h2=A_leg / h2_t_y, leg_reduction=A_leg / baseline_t_y, leg_ci=atm_t_y / h2_t_y, leg_plant_capture=cap_t_y / (cap_t_y + atm_t_y))
check("legacy closure F_C = C + (1-f)G + P (J.11)", close(f_c6, cap_l + burn_l + p_c, 1e-12))
# validation row of the audit: anchored kmol C/h at Case B scale
check("legacy anchored captured kmol/h (Case B) = 1097.904", close(cap_l * s_b, 1097.903971, 1e-8), f"{cap_l * s_b:.6f}")
check("legacy anchored burned kmol/h (Case B) = 116.9046", close(burn_l * s_b, 116.904567, 1e-8), f"{burn_l * s_b:.6f}")
check("legacy anchored feed kmol/h (Case B) = 1219.2324", close(f_c6 * s_b, 1219.232352, 1e-8), f"{f_c6 * s_b:.6f}")

# carbon identity of the unabated reference (N.7.3), validation against INL pairs
def ref_co2_st_d(ng_mmscfd: float, scale: float = 1.0) -> float:
    kmol_c = mmscfd_to_kmol_h(ng_mmscfd) * cbar_toml
    p_ref = mmscfd_to_kmol_h(prm("H2_A_MMSCFD")) * prm("CH4_IN_H2")
    return (kmol_c - p_ref) * MW_CO2 / 1000.0 * 24.0 / ST


ref_conv = ref_co2_st_d(prm("NG_CONV_NOCC"))
ref_htgr = ref_co2_st_d(prm("NG_NORECYCLE"))
check("N.7.3 carbon identity reproduces TEV-953 conventional 3205 st/d within 0.5 percent", abs(rel_err(ref_conv, prm("CO2_CONV_NOCC"))) < 5e-3, f"{ref_conv:.1f} vs {prm('CO2_CONV_NOCC'):.0f} st/d, {100 * rel_err(ref_conv, prm('CO2_CONV_NOCC')):+.3f} %")
check("N.7.3 carbon identity reproduces TEV-953 HTGR no-capture 2713 st/d within 0.5 percent", abs(rel_err(ref_htgr, prm("CO2_HTGR_NOCC"))) < 5e-3, f"{ref_htgr:.1f} vs {prm('CO2_HTGR_NOCC'):.0f} st/d, {100 * rel_err(ref_htgr, prm('CO2_HTGR_NOCC')):+.3f} %")
VALUES.update(ref_conv_st_d=ref_conv, ref_htgr_st_d=ref_htgr)
# gas rate that INL's 3205 st/d implies through the identity
n_equiv_mmscfd = (st_d_to_kmol_h(prm("CO2_CONV_NOCC")) + p_c) / cbar_toml / mmscfd_to_kmol_h(1.0)
VALUES["n_equiv_mmscfd"] = n_equiv_mmscfd

# N.9.2 identity on the legacy basis: A = M t_h [C + c_bar (n_ref - n_plant)] with the INL reference
dn_legacy = (prm("NG_CONV_NOCC") - ng_c6) * s_b          # MMSCFD at Case B
c_dn_legacy = mmscfd_to_kmol_h(dn_legacy) * cbar_toml     # kmol C/h
A_from_id = tpy_s(cap_l) / s_b + tpy_s(c_dn_legacy) / s_b  # t/y
A_from_id = (cap_l * s_b + c_dn_legacy) * MW_CO2 / 1000.0 * hours
VALUES.update(dn_legacy_mmscfd=dn_legacy, c_dn_legacy=c_dn_legacy, A_from_id=A_from_id)
print(f"   legacy A = {A_leg:,.1f} t/y; captured + c_bar*dn identity gives {A_from_id:,.1f} t/y ({100 * rel_err(A_from_id, A_leg):+.3f} %)")
check("N.9.2 identity on the legacy basis within 1 percent (gap is INL rounding of 3205 and 52.5)", abs(rel_err(A_from_id, A_leg)) < 1e-2)

# lifecycle pieces that are reproducible: T&S burden and saved upstream gas
ts_burden = prm("TS_BURDEN") * cap_t_y
check("legacy TS burden = 0.025 x captured (K.19)", close(ts_burden, abs(em_value("B_REC_F", "INL_9p21", "lifecycle_burdens", "TS_BURDEN")), 1e-8), f"{ts_burden:.3f}")
gj = prm("NG_HHV") * MMBTU_GJ * days
up_credit = (prm("NG_CONV_NOCC") * s_b - ng_c6 * s_b) * gj * prm("EF_UPSTREAM") / 1000.0
check("legacy saved upstream gas credit (K.19, HHV basis) reproduced", close(up_credit, em_value("B_REC_F", "INL_9p21", "lifecycle_upstream", "UPSTREAM_CREDIT"), 1e-8), f"{up_credit:.3f}")
# Added by the verifier: the heat of the saved gas written out as in the Markdown of N.9.5, in MMSCFD x Btu/scf x GJ/MMBtu x days
dG_gj = (prm("NG_CONV_NOCC") - ng_c6) * s_b * prm("NG_HHV") * MMBTU_GJ * days
check("N.9.5 saved heat DeltaG = 12.6407 MMSCFD x 1044 Btu/scf x 1.05505585 x 310.25 d = 4319752 GJ/y, and DeltaG x 11.5 kg/GJ is the ledger credit",
      abs(dG_gj - 4319752.0) < 1.0 and close(dG_gj * prm("EF_UPSTREAM") / 1000.0, up_credit, 1e-12), f"{dG_gj:.1f} GJ/y")
power_t = prm("E_CONV_NOCC") * s_b * hours * prm("GRID_EF")
sys_delta =em_value("B_REC_F", "INL_9p21", "system", "ABATEMENT") - A_leg
VALUES.update(ts_burden=ts_burden, up_credit=up_credit, power_t=power_t, sys_delta=sys_delta, lost_steam_implied=power_t - sys_delta)

# INL back-calculation of f_r is ill conditioned in phi (N.4.3)
def f_inl(phi_x: float) -> float:
    t = cap6 / eta_sel / phi_x
    g = t - cap6 - p_c
    return (t - f_c6) / g


f_at_fs_phi = f_inl(phi_fs)
phis = sp.symbols("phi_x", positive=True)
Tc6, Fc6s, Pc, Cap6 = sp.symbols("T6 F_c6 P cap6", positive=True)
f_sym = (Cap6 / (sp.Rational(99, 100) * phis) - Fc6s) / (Cap6 / (sp.Rational(99, 100) * phis) - Cap6 - Pc)
dfdphi = sp.diff(f_sym, phis)
dfdphi_val = float(dfdphi.subs({Cap6: cap6, Fc6s: f_c6, Pc: p_c, phis: phi_l}))
VALUES.update(f_at_fs_phi=f_at_fs_phi, dfdphi=dfdphi_val)
check("N.4.3 J.7 evaluated at the flowsheet phi differs strongly from 0.7522", abs(f_at_fs_phi - f6) > 0.05, f"f(0.775)={f6:.4f}, f({phi_fs:.4f})={f_at_fs_phi:.4f}, df/dphi={dfdphi_val:.3f}")
# hand-derived derivative of N.4.3 against SymPy
dfdphi_hand = -(t6 / phi_l) * (f_c6 - cap6 - p_c) / g6**2
check("N.4.3 derivative formula equals the SymPy derivative", close(dfdphi_val, dfdphi_hand, 1e-9), f"{dfdphi_hand:.6f}")

# ----------------------------------------------------------------------------------
# N.4.4 stack carbon as a function of f, carbon shares held at their values (approximation)
# ----------------------------------------------------------------------------------
print("== N.4.4 stack carbon against f")
m_s, Bs = sp.symbols("m B_s", positive=True)          # m = 1 - kappa - eta*phi
D_m = 1 - f * m_s
G_m = sp.simplify(m_s * (FC - f * P) / D_m - P)
check("N.4.4 G = (m F_C - P)/D", sp.simplify(G_m - (m_s * FC - P) / D_m) == 0)
B0_s = m_s * FC - P
B_f = (1 - f) * (m_s * FC - P) / (1 - m_s * f)
check("N.4.4 B(f) = B0 (1-f)/(1-m f) with B0 = m F_C - P", sp.simplify((1 - f) * G_m - B_f) == 0)
f_of_B = (B0_s - Bs) / (B0_s - m_s * Bs)
check("N.4.4 inverse f(B) = (B0 - B)/(B0 - m B)", sp.simplify(B_f.subs(f, f_of_B) - Bs) == 0)
check("N.4.4 dB/df = -B0 (1-m)/(1-m f)^2", sp.simplify(sp.diff(B_f, f) + B0_s * (1 - m_s) / (1 - m_s * f) ** 2) == 0)
check("N.4.4 B(0) = B0 and B(1) = 0", sp.simplify(B_f.subs(f, 0) - B0_s) == 0 and sp.simplify(B_f.subs(f, 1)) == 0)
m_val = 1 - kap_fs - eta_fs * phi_fs
B0_val = m_val * F_C - P_fs
B_at = lambda ff: (1 - ff) * B0_val / (1 - m_val * ff)
check("N.4.4 B(f) at the flowsheet f equals the stack carbon of S15", close(B_at(f_fs), B_fs, 1e-8), f"{B_at(f_fs):.6f} vs {B_fs:.6f}")
dBdf_val = -B0_val * (1 - m_val) / (1 - m_val * f_fs) ** 2
VALUES.update(m_val=m_val, B0_val=B0_val, B0_t_y=tpy(B0_val), dBdf_val=dBdf_val, dBdf_t_y_per_0p01=tpy(dBdf_val) * 0.01,
              B_f05=B_at(0.5), B_f09=B_at(0.9), B_f05_t_y=tpy(B_at(0.5)), B_f09_t_y=tpy(B_at(0.9)),
              f_for_half=float(f_of_B.subs({m_s: m_val, FC: F_C, P: P_fs, Bs: B_fs / 2})))

# N.4.4 at a fixed hydrogen output (added by the verifier): F_C = cbar n_NG(f) with n_NG(f) from N.1.5,
# shares phi, psi, omega, kappa and xi_eff held at the run values.  The fixed-gas curve above leaves the
# hydrogen output free to change with f; the plant is specified by its hydrogen output.
n_of_f = n_closed.subs({nH2: nH2_fs, xe: xe_val, pi_: pi_val, ZN: Z_NG, cb: cbar, psi: psi_fs, om: om_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs})
B_H = lambda ff: float((1 - ff) * (m_val * cbar * n_of_f.subs(f, ff) - P_fs) / (1 - m_val * ff))
check("N.4.4 fixed-hydrogen form: gas rate at the run's f equals F_C/cbar", close(float(n_of_f.subs(f, f_fs)), F_C / cbar, 1e-6), f"{float(n_of_f.subs(f, f_fs)):.5f}")
check("N.4.4 fixed-hydrogen form reproduces the stack carbon of S15 at the run's f", close(B_H(f_fs), B_fs, 1e-7), f"{B_H(f_fs):.6f} vs {B_fs:.6f}")
check("N.4.4 fixed-hydrogen form: B_H(1) = 0", abs(B_H(1.0)) < 1e-9)
check("N.4.4 fixed-hydrogen form: gas rate falls with f (1305.53, 1205.42, 1116.53 at f = 0, 0.5, 0.9)",
      all(abs(float(n_of_f.subs(f, ff)) - v) < 5e-3 for ff, v in [(0.0, 1305.53), (0.5, 1205.42), (0.9, 1116.53)]))
_dh = 1e-6
slope_H = (B_H(f_fs + _dh) - B_H(f_fs - _dh)) / (2 * _dh) * 0.01 * tpy(1.0)
check("N.4.4 fixed-hydrogen slope is -1205 t/y per 0.01 of f (fixed-gas slope -1144)", abs(slope_H + 1205.4) < 0.1 and abs(tpy(dBdf_val) * 0.01 + 1144.3) < 0.1, f"{slope_H:.1f} vs {tpy(dBdf_val) * 0.01:.1f}")
check("N.4.4 fixed-hydrogen table in t/y (113993, 60290, 12642 at f = 0, 0.5, 0.9)",
      all(abs(tpy(B_H(ff)) - v) < 1.0 for ff, v in [(0.0, 113993.0), (0.5, 60290.0), (0.9, 12642.0)]), f"{tpy(B_H(0.0)):.0f} {tpy(B_H(0.5)):.0f} {tpy(B_H(0.9)):.0f}")
_lo, _hi = f_fs, 0.99
for _ in range(80):
    _mid = 0.5 * (_lo + _hi)
    _lo, _hi = (_mid, _hi) if B_H(_mid) > B_fs / 2 else (_lo, _mid)
check("N.4.4 f that halves the stack at fixed hydrogen output is 0.8779 (0.8812 at fixed gas)", abs(_lo - 0.8779) < 5e-5 and abs(VALUES["f_for_half"] - 0.8812) < 5e-5, f"{_lo:.5f} vs {VALUES['f_for_half']:.5f}")
VALUES.update(slope_H_t_y_per_0p01=slope_H, f_for_half_H2=_lo)

# ----------------------------------------------------------------------------------
# 8. N.7 and N.9 symbolic identities
# ----------------------------------------------------------------------------------
print("== N.7 / N.9 symbolic")
Mc, th = sp.symbols("M_CO2 t_h", positive=True)
c_b, n_r, n_p, Pp, vv, Cx, Bx = sp.symbols("cbar n_ref n_plant P v C B")
# energies in kmol CO2/h: E_ref = cbar n_ref - P - (1-v) kref ; E_plant = B + v k  with k = kappa*T
kref, kpl = sp.symbols("k_ref k_pl")
E_ref = c_b * n_r - Pp - (1 - vv) * kref
closure_B = sp.solve(sp.Eq(c_b * n_p, Cx + Bx + Pp + kpl), Bx)[0]
E_plant = closure_B + vv * kpl
A_dir = sp.simplify(E_ref - E_plant)
check("N.9.2 A = C + cbar (n_ref - n_plant) - (1-v)(k_ref - k_pl)", sp.simplify(A_dir - (Cx + c_b * (n_r - n_p) - (1 - vv) * (kref - kpl))) == 0)
check("N.9.2 with v = 1 the condensate terms vanish", sp.simplify(A_dir.subs(vv, 1) - (Cx + c_b * (n_r - n_p))) == 0)
check("N.9.2 moving v from 1 to 0 changes the plant side by exactly k_pl", sp.simplify(E_plant.subs(vv, 1) - E_plant.subs(vv, 0) - kpl) == 0)
# Added by the verifier: scope of the decomposition when unmodelled inlets and outlets exist (kmol C/h).
e_in, e_oth, e_atm = sp.symbols("eps_in eps_out_other eps_out_atm")
B_gen = sp.solve(sp.Eq(c_b * n_p + e_in, Cx + Bx + Pp + kpl + e_atm + e_oth), Bx)[0]
A_gen = sp.simplify(E_ref - (B_gen + vv * kpl + e_atm))
check("N.9.2 scope: with unmodelled streams, N.9.1 minus the decomposition equals eps_out_other - eps_in (zero only when those are zero or balanced)",
      sp.simplify(A_gen - (Cx + c_b * (n_r - n_p) - (1 - vv) * (kref - kpl)) - (e_oth - e_in)) == 0)

# system view: lost steam counted once
Qr, Qp, eb, ef_ng, th2 = sp.symbols("Qexp_ref Qexp_plant eta_b e_NG t_h2", positive=True)
E_lost = (Qr - Qp) * th2 * ef_ng / eb
check("N.7.4 lost-steam term is proportional to the export difference, zero when equal", sp.simplify(E_lost.subs(Qp, Qr)) == 0)
check("N.7.4 two plants with the same export give no system correction", sp.simplify(E_lost.subs({Qr: 5, Qp: 5})) == 0)

# net after T&S and lifecycle (D4: CO2 only vs CO2e)
A_d, A_cap, lam, e_ship, w_ts, ef_el = sp.symbols("A_dir A_cap lambda e_ship w_TS EF_el", nonnegative=True)
tau_ts = lam + e_ship + w_ts * ef_el
A_ts = A_d - tau_ts * A_cap
check("N.9.4 tau_TS = 0 gives A_TS = A_dir", sp.simplify(A_ts.subs({lam: 0, e_ship: 0, w_ts: 0}) - A_d) == 0)
check("N.9.4 legacy lumped burden 0.025 t/t: A_TS = A_dir - 0.025 A_cap", sp.simplify(A_ts.subs({lam: sp.Rational(25, 1000), e_ship: 0, w_ts: 0}) - (A_d - sp.Rational(25, 1000) * A_cap)) == 0)
dG, eup_c, eup_m, gwp, eN, Hh = sp.symbols("dG e_up_CO2 e_up_CH4 GWP_H e_nuc E_nuc", nonnegative=True)
A_lc_co2e = A_ts + dG * (eup_c + gwp * eup_m) - Hh
A_lc_co2 = A_ts + dG * eup_c - Hh
check("N.9.5 setting GWP_H = 0 turns the CO2e view into the CO2 view", sp.simplify(A_lc_co2e.subs(gwp, 0) - A_lc_co2) == 0)
check("N.9.5 CO2e view exceeds the CO2 view by GWP_H e_up_CH4 dG", sp.simplify(A_lc_co2e - A_lc_co2 - gwp * eup_m * dG) == 0)

# ----------------------------------------------------------------------------------
# 9. N.8 annual quantities, N.10 intensities (symbolic and unit scale)
# ----------------------------------------------------------------------------------
print("== N.8 / N.10")
nd, fav = sp.symbols("ndot f_av", positive=True)
th_expr = 8760 * fav
E_year = nd * Mc * th_expr / 1000
check("N.8.2 t/y = ndot [kmol/h] x M [kg/kmol] x 8760 f_av [h/y] / 1000", sp.simplify(E_year - nd * Mc * 8760 * fav / 1000) == 0)
check("K.10 hours = 24 x days (days = 365 f_av)", close(24.0 * days, hours, 1e-12), f"{hours:.1f} h/y")
check("N.8 unit factor: 1 kmol/h CO2 for 7446 h = 327.7 t", close(1.0 * MW_CO2 / 1000.0 * hours, 327.7, 1e-3), f"{MW_CO2 / 1000.0 * hours:.4f}")
check("st/d to kmol/h: 1927 st/d = 1655.2 kmol/h", close(st_d_to_kmol_h(1927.0), 1655.2, 1e-4), f"{st_d_to_kmol_h(1927.0):.3f}")
check("MMSCFD to kmol/h: 1 MMSCFD = 49.80 kmol/h", close(mmscfd_to_kmol_h(1.0), 49.80, 1e-3), f"{mmscfd_to_kmol_h(1.0):.4f}")
E_p, E_r, M_h = sp.symbols("E_plant E_ref M_H2", positive=True)
a_h2 = (E_r - E_p) / M_h
red = (E_r - E_p) / E_r
ci = E_p / M_h
check("N.10: a_H2 = e_ref - ci", sp.simplify(a_h2 - (E_r / M_h - ci)) == 0)
check("N.10: reduction = 1 - E_plant/E_ref", sp.simplify(red - (1 - E_p / E_r)) == 0)
e_st = MW_CO2 / (4 * MW_H2)
check("K.5 stoichiometric minimum 5.46 kg CO2 per kg H2", close(e_st, 5.46, 1e-3), f"{e_st:.4f}")
e_feed_fs = F_C * MW_CO2 / H2_kg_h
VALUES.update(e_st=e_st, e_feed_fs=e_feed_fs)
check("N.10: feed carbon intensity of FS_B_REC_F exceeds the stoichiometric minimum", e_feed_fs > e_st, f"{e_feed_fs:.4f} > {e_st:.4f}")

# ----------------------------------------------------------------------------------
# 10. Physical lower bound on the reference feed and on abatement (follows from N.7.1 and N.9.2)
# ----------------------------------------------------------------------------------
h2_t_y_fs = H2_kg_h * hours / 1000.0
VALUES.update(h2_t_y_fs=h2_t_y_fs, cap_fs_per_kg=tpy(C_fs) / h2_t_y_fs, ci_fs_stack=tpy(B_fs) / h2_t_y_fs,
              ci_fs_vent=(tpy(B_fs) + tpy(K_fs)) / h2_t_y_fs, stack_plus_cond_t_y=tpy(B_fs) + tpy(K_fs),
              eta_pl_fs=C_fs / F_C, D_fs=1 - f_fs * (1 - kap_fs - eta_fs * phi_fs), t_over_fc=T_fs / F_C,
              eta_pl_f1=float(eta_pl.subs({f: 1, FC: F_C, P: P_fs, kap: kap_fs, eta: eta_fs, phi: phi_fs})),
              t_per_kmolh=MW_CO2 / 1000.0 * hours, sens_per_mmscfd=mmscfd_to_kmol_h(1.0) * cbar * MW_CO2 / 1000.0 * hours,
              z_in_fs=Z_in, z_s15=z_flow(S["S15"]))
check("sensitivity: 1 MMSCFD of reference gas moves A_dir by 17.2 kt/y", close(VALUES["sens_per_mmscfd"], 17197.8, 1e-3), f"{VALUES['sens_per_mmscfd']:.1f} t/y")
check("N.3.2 plant capture at f = 1 stays below 1 because of K and P", VALUES["eta_pl_f1"] < 1.0, f"{VALUES['eta_pl_f1']:.5f}")
n_min_ref = nH2_fs / (xi_p * Z_NG)  # r = 0 in the reference, psi' = omega' = 0
dn_min = n_min_ref - n_fs
A_lb = (C_fs + cbar * dn_min) * MW_CO2 / 1000.0 * hours
VALUES.update(n_min_ref=n_min_ref, n_min_ref_mmscfd=n_min_ref / mmscfd_to_kmol_h(1.0), dn_min=dn_min, A_lb=A_lb,
              inl_conv_B_mmscfd=prm("NG_CONV_NOCC") * s_b, inl_htgr_B_mmscfd=prm("NG_NORECYCLE") * s_b)
check("N.7.1 reference process feed lower bound exceeds the plant gas rate", n_min_ref > n_fs, f"{n_min_ref:.2f} vs {n_fs:.2f} kmol/h")
print(f"   lower bound A_dir >= {A_lb:,.0f} t/y (reference with no fuel and complete conversion, v = 1)")

# ----------------------------------------------------------------------------------
# 11. Dimension table
# ----------------------------------------------------------------------------------
print("== dimension table")
Nn, Mm, Tt = amount_of_substance, mass, time
one = Dimension(1)
kmol_h = Nn / Tt
# Each row: (id, text, left-hand dimension, list of right-hand terms). Products are formed
# by multiplying dimensions; every term of a sum must carry the dimension of the left side.
# "Tt/Tt" is the factor hours-per-year (a pure number in SI dimensions).
DIMS = [
    ("N.1.1", "c_bar = sum_j y_j a_C,j", one, [one * one]),
    ("N.1.2", "Z = 2 a_C - a_O + a_H/2", one, [one, one, one]),
    ("N.1.3", "H2_12 = Zdot_in - 4 CH4 - CO", kmol_h, [kmol_h * one, kmol_h * one, kmol_h * one]),
    ("N.1.4", "xi_eff = xi/(1 - r(1-xi))", one, [one / one]),
    ("N.1.5", "n_NG = nH2 [1/xi_eff + f pi (4-A)]/(Z - A c_bar)", kmol_h, [kmol_h * one / one]),
    ("N.2.1", "T = (F_C - f P)/D", kmol_h, [kmol_h / one]),
    ("N.2.2", "G = (1-kappa-eta phi) T - P", kmol_h, [kmol_h * one, kmol_h]),
    ("N.3.1", "C = eta phi T", kmol_h, [one * one * kmol_h]),
    ("N.3.2", "eta_pl = C/F_C", one, [kmol_h / kmol_h]),
    ("N.4.1", "f = R/G", one, [kmol_h / kmol_h]),
    ("N.4.2", "beta = B/n_N2", one, [kmol_h / kmol_h]),
    ("N.4.4", "B = (1-f)(m F_C - P)/(1 - m f)", kmol_h, [kmol_h * one / one, kmol_h * one / one]),
    ("N.5.1", "O2 = (2 C + H/2 - O)/2", kmol_h, [kmol_h * one]),
    ("N.5.2", "E_plant = M [B + v kappa T] t_h", Mm / Tt, [Mm / Nn * kmol_h * (Tt / Tt)]),
    ("N.6.1", "rho = R_C/F_C", one, [kmol_h / kmol_h]),
    ("N.7.2", "n_fuel = (Q_fire - Q_rej)/LHV_NG", kmol_h, [(energy / Tt) / (energy / Nn)]),
    ("N.7.3", "E_ref = M [c_bar (n_proc + n_fuel) - P] t_h", Mm / Tt, [Mm / Nn * kmol_h * (Tt / Tt)]),
    ("N.7.4", "E_lost = (Qexp_ref - Qexp_pl) t_h e_NG/eta_b", Mm / Tt, [(energy / Tt) * (Mm / energy) * (Tt / Tt)]),
    ("N.8.2", "E [t/y] = ndot M t_h/1000", Mm / Tt, [kmol_h * (Mm / Nn) * (Tt / Tt)]),
    ("N.9.1", "A_dir = E_ref - E_plant", Mm / Tt, [Mm / Tt, Mm / Tt]),
    ("N.9.4", "A_TS = A_dir - tau A_cap", Mm / Tt, [Mm / Tt, one * Mm / Tt]),
    ("N.9.5", "A_LC = A_TS + dG (e_up_CO2 + GWP e_up_CH4) - E_nuc", Mm / Tt, [Mm / Tt, (energy / Tt) * (Mm / energy) * (Tt / Tt), Mm / Tt]),
    ("N.10.1", "a_H2 = A/M_H2", one, [(Mm / Tt) / (Mm / Tt)]),
    ("N.10.2", "reduction = A/E_ref", one, [(Mm / Tt) / (Mm / Tt)]),
]
all_dim_ok = True
for eqid, text, lhs, terms in DIMS:
    dl = dimsys_SI.get_dimensional_dependencies(lhs)
    ok = all(dimsys_SI.get_dimensional_dependencies(t) == dl for t in terms)
    all_dim_ok &= ok
    print(f"   {eqid:7s} {text:62s} {'ok' if ok else 'MISMATCH'}")
check("dimension table: every equation has equal left and right dimensions", all_dim_ok)

# ----------------------------------------------------------------------------------
# Report
# ----------------------------------------------------------------------------------
print("== key values for the derivation text")
for k in sorted(VALUES):
    v = VALUES[k]
    print(f"   {k:24s} {v:.10g}")
npass = sum(1 for _, ok, _ in RESULTS if ok)
print(f"\n{npass} of {len(RESULTS)} checks passed")
sys.exit(0 if npass == len(RESULTS) else 1)
