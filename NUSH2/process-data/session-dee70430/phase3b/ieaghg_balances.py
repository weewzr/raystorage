"""IEAGHG 2017-02 (Merchant H2 plant with CCS), Base Case (unabated SMR).

Source-traced reconstruction and component-wise balances.
Run:  python -I ieaghg_balances.py
Writes ieaghg_streams.md next to this script and prints a short summary.

Data labels used throughout
  published : read from the report (PDF page, sheet or table given in SRC)
  calculated: computed here from published values (equation given in the md)
  assumed   : constant or convention not stated in the report (listed in ASSUMED)

Residual convention: residual = out - in, relative residual = (out - in) / in.
All arithmetic is done here; nothing is typed in by hand except published data.
"""
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT_MD = os.path.join(HERE, "ieaghg_streams.md")

# ---------------------------------------------------------------- assumed constants
AM = {"C": 12.011, "H": 1.008, "O": 15.999, "N": 14.007}      # kg/kmol, IUPAC conventional (assumed)
V_N = 22.414                                                  # Nm3/kmol, 0 C and 101.325 kPa (assumed, see check)
KCAL_KJ = 4.1868                                              # kJ/kcal, International Table (assumed)
AIR_O2, AIR_N2 = 0.21, 0.79                                   # dry air, mole fractions (assumed)
# lower heating values at 25 C, water as vapour, MJ/kmol (assumed, standard thermochemical data)
LHV = {"H2": 241.8, "CO": 283.0, "CH4": 802.3, "C2H6": 1428.6, "C3H8": 2043.1,
       "nC4H10": 2657.5, "nC5H12": 3272.6}

FORM = {"CO2": {"C": 1, "O": 2}, "CO": {"C": 1, "O": 1}, "H2": {"H": 2}, "N2": {"N": 2},
        "O2": {"O": 2}, "CH4": {"C": 1, "H": 4}, "C2H6": {"C": 2, "H": 6},
        "C3H8": {"C": 3, "H": 8}, "nC4H10": {"C": 4, "H": 10}, "nC5H12": {"C": 5, "H": 12},
        "H2O": {"H": 2, "O": 1}}
SPEC = list(FORM)
MW = {s: sum(AM[e] * k for e, k in f.items()) for s, f in FORM.items()}
ELEM = ["C", "H", "O", "N"]


def comp(**kw):
    return {s: kw.get(s, 0.0) for s in SPEC}


# ---------------------------------------------------------------- published data
# Heat and material balance, Base case: PDF p44 (streams 1-12) and p45 (streams 13-15),
# unnumbered A3 inserts after Sheet 19 of 112 (section 2.6).  Rev 1, June 2015, FWI 1BD0840A.
NGX = comp(CO2=.0200, N2=.0089, CH4=.8900, C2H6=.0700, C3H8=.0100, nC4H10=.0010, nC5H12=.0001)
S = {
    1: dict(name="Natural gas from B.L.", T=9, P=7.00, n=1696.6, m=30563, x=NGX),
    2: dict(name="Natural gas feedstock to H2 plant", T=128, P=3.71, n=1455.8, m=26231, x=NGX),
    3: dict(name="Natural gas fuel to burners", T=121, P=0.50, n=240.4, m=4332, x=NGX),
    4: dict(name="Purified feedstock to pre-reformer", T=500, P=3.39, n=5514.0, m=98874,
            x=comp(CO2=.0053, H2=.0053, N2=.0023, CH4=.2350, C2H6=.0185, C3H8=.0026,
                   nC4H10=.0003, H2O=.7307)),
    5: dict(name="HTS reactor inlet (reformer effluent)", T=320, P=2.80, n=8370.3, m=101667,
            x=comp(CO2=.0492, CO=.1156, H2=.5171, N2=.0015, CH4=.0238, H2O=.2927)),
    6: dict(name="HTS reactor outlet (shift outlet)", T=412, P=2.77, n=8370.3, m=101667,
            x=comp(CO2=.1283, CO=.0366, H2=.5961, N2=.0015, CH4=.0238, H2O=.2137)),
    7: dict(name="PSA inlet (PSA feed)", T=35, P=2.58, n=6596.9, m=69711,
            x=comp(CO2=.1627, CO=.0464, H2=.7563, N2=.0020, CH4=.0302, H2O=.0024)),
    8: dict(name="PSA tail gas", T=28, P=0.13, n=2106.3, m=60658,
            x=comp(CO2=.5095, CO=.1454, H2=.2369, N2=.0062, CH4=.0945, H2O=.0076)),
    # CO in the flue gas is given only as '30 mg/Nm3 max'; taken as 0 here (bounded below)
    9: dict(name="Flue gas to atmosphere", T=136, P=0.02, n=8659.4, m=257698,
            x=comp(CO2=.2123, N2=.6083, O2=.0102, H2O=.1692)),
    10: dict(name="HP steam export", T=395, P=4.23, n=2556.0, m=46053, x=comp(H2O=1.0)),
    11: dict(name="Demi water make-up and condensate", T=15, P=0.60, n=5095.7, m=91800, x=comp(H2O=1.0)),
    # H2 purity is '0.9999+'; taken as 1.0 for balances
    12: dict(name="Hydrogen to B.L.", T=40, P=2.50, n=4461.5, m=8994, x=comp(H2=1.0)),
    13: dict(name="H2 recycle", T=40, P=2.51, n=29.1, m=59, x=comp(H2=1.0)),
    14: dict(name="HP steam to process", T=400, P=4.29, n=5290.1, m=95301, x=comp(H2O=1.0)),
    15: dict(name="LP steam to deaerator", T=177, P=0.44, n=30.0, m=540, x=comp(H2O=1.0)),
}
PUB = dict(  # Table 2 (PDF p16) and section 2.7 (PDF p46, Sheet 20 of 112), Base Case column
    ng_feed_th=26.231, ng_fuel_th=4.332, ng_total_th=30.563, ng_lhv=46.50, e_in_mw=394.77,
    h2_th=8.994, h2_nm3h=100000, h2_lhv=119.96, e_out_mw=299.70, gross_mwe=11.500,
    h2plant_mwe=-1.216, cogen_mwe=-0.366, export_mwe=9.918, sc_feed=12.197, sc_fuel=2.014,
    sc_total=14.212, co2_spec=0.8091,
    ng_lhv_annex=46.502, h2_lhv_annex=119.96,           # Annex Reference Document Tables 1, 2
    radiant_mmkcal=82.63, fuel_util_mmkcal=48.2, fuel_util_total_mmkcal=48.7,
    bfd=dict(ng=30563, raw_water=59700, sea_water_m3h=3435, h2=8994, flue=257698, power_kw=9918, waste_kg=15500),
)
DATA_ROUNDING = 0.00005   # half unit of the 4th decimal on published mole fractions

# ---------------------------------------------------------------- helpers


def nflow(i):
    s = S[i]
    return {k: s["n"] * v for k, v in s["x"].items()}


def elem(flows):
    e = {k: 0.0 for k in ELEM}
    for sp, n in flows.items():
        for el, k in FORM[sp].items():
            e[el] += n * k
    return e


def mass(flows):
    return sum(n * MW[sp] for sp, n in flows.items())


def lhv_mj_h(flows):
    return sum(n * LHV.get(sp, 0.0) for sp, n in flows.items())


def add(*fl):
    out = {s: 0.0 for s in SPEC}
    for f in fl:
        for k, v in f.items():
            out[k] += v
    return out


def res(inn, out):
    a = out - inn
    r = a / inn if inn else float("nan")
    return a, r


def pct(x, d=3):
    return f"{100 * x:.{d}f} %"


R = {}      # numerical results for the md and for the return summary
T = {}      # tables for the md

# ---------------------------------------------------------------- B1 stream integrity
rows = []
for i in sorted(S):
    s = S[i]
    sx = sum(s["x"].values())
    m_pub = s["m"] / s["n"]
    m_cmp = sum(s["x"][k] * MW[k] for k in SPEC) / sx
    rows.append((i, s["name"], sx, m_pub, m_cmp, (m_cmp - m_pub) / m_pub))
T["integrity"] = rows

# ---------------------------------------------------------------- B2 natural gas split and NG properties
c_ng = sum(NGX[k] * FORM[k].get("C", 0) for k in SPEC)          # kmol C / kmol NG
M_ng_cmp = sum(NGX[k] * MW[k] for k in SPEC)
lhv_ng_mol = sum(NGX[k] * LHV.get(k, 0) for k in SPEC)          # MJ/kmol
R["ng"] = dict(
    c_per_kmol=c_ng, M=M_ng_cmp, M_pub=S[1]["m"] / S[1]["n"],
    lhv_cmp_mj_kg=lhv_ng_mol / M_ng_cmp, lhv_pub_annex=PUB["ng_lhv_annex"], lhv_pub_t2=PUB["ng_lhv"],
    split_n=res(S[1]["n"], S[2]["n"] + S[3]["n"]), split_m=res(S[1]["m"], S[2]["m"] + S[3]["m"]),
    frac_fuel_m=S[3]["m"] / S[1]["m"],
    h2_lhv_cmp=LHV["H2"] / MW["H2"], h2_lhv_pub=PUB["h2_lhv"],
)
R["ng"]["lhv_dev"] = (R["ng"]["lhv_cmp_mj_kg"] - PUB["ng_lhv_annex"]) / PUB["ng_lhv_annex"]
R["ng"]["h2_dev"] = (R["ng"]["h2_lhv_cmp"] - PUB["h2_lhv"]) / PUB["h2_lhv"]

# Nm3 convention check: stream 12 kmol/h x 22.414 versus 100,000 Nm3/h
R["nm3"] = dict(n12=S[12]["n"], v=S[12]["n"] * V_N, rel=(S[12]["n"] * V_N - PUB["h2_nm3h"]) / PUB["h2_nm3h"],
                kg_per_nm3=S[12]["m"] / PUB["h2_nm3h"])

# ---------------------------------------------------------------- B3 reformer + pre-reformer + shift (stream 4 + water2 -> stream 5 -> stream 6)
e4, e5, e6 = elem(nflow(4)), elem(nflow(5)), elem(nflow(6))
w2_H = (e5["H"] - e4["H"]) / 2.0          # kmol/h of extra H2O needed to close H
w2_O = (e5["O"] - e4["O"]) / 1.0          # kmol/h of extra H2O needed to close O
w1 = nflow(4)["H2O"]
w2_14 = S[14]["n"] - w1                   # if stream 14 were all of the process steam
c_hc = sum(nflow(4)[k] * FORM[k].get("C", 0) for k in SPEC) - nflow(4)["CO2"]   # hydrocarbon carbon (excl CO2)
c_all4 = e4["C"]
R["ref"] = dict(
    e4=e4, e5=e5, e6=e6, w1=w1, w2_H=w2_H, w2_O=w2_O, w2_14=w2_14,
    dw2=w2_H - w2_O,
    res_C=res(e4["C"], e5["C"]), res_N=res(e4["N"], e5["N"]),
    res_C56=res(e5["C"], e6["C"]), res_H56=res(e5["H"], e6["H"]), res_O56=res(e5["O"], e6["O"]),
    res_N56=res(e5["N"], e6["N"]),
    sc_H=(w1 + w2_H) / c_all4, sc_hc_H=(w1 + w2_H) / c_hc,
    sc_14=(w1 + w2_14) / c_all4, sc_hc_14=(w1 + w2_14) / c_hc,
    steam_kg_H=(w1 + w2_H) * MW["H2O"], steam_kg_14=S[14]["m"],
    n56=res(S[5]["n"], S[6]["n"]),
    co_dry6=S[6]["x"]["CO"] / (1 - S[6]["x"]["H2O"]), ch4_dry5=S[5]["x"]["CH4"] / (1 - S[5]["x"]["H2O"]),
    co_dry7=S[7]["x"]["CO"] / (1 - S[7]["x"]["H2O"]),
    mass4=S[4]["m"], mass4_chk=S[2]["m"] + S[13]["m"] + w1 * MW["H2O"],
)
R["ref"]["mass4_res"] = res(S[4]["m"], R["ref"]["mass4_chk"])
# shift extent from stream 5 -> 6
f5, f6 = nflow(5), nflow(6)
R["ref"]["shift"] = {k: f6[k] - f5[k] for k in ("CO", "H2O", "CO2", "H2")}
# chemical heat of reforming (25 C, water vapour basis): LHV(stream 5) - LHV(stream 4)
R["ref"]["dLHV_MW"] = (lhv_mj_h(f5) - lhv_mj_h(nflow(4))) / 3600.0

# ---------------------------------------------------------------- B4 condensate knock-out and PSA
f6, f7, f8, f12, f13 = nflow(6), nflow(7), nflow(8), nflow(12), nflow(13)
prod = add(f12, f13)
cond_w = f6["H2O"] - f7["H2O"]               # condensate (derived, assumed pure water)
rows = []
for k in SPEC:
    inn = f7[k]
    out = f8[k] + prod[k]
    a, r = res(inn, out)
    rows.append((k, f7[k], f8[k], prod[k], a, r, DATA_ROUNDING * (S[7]["n"] + S[8]["n"])))
T["psa"] = rows
nonw6 = {k: f6[k] for k in SPEC if k != "H2O"}
nonw7 = {k: f7[k] for k in SPEC if k != "H2O"}
rows = []
for k in SPEC:
    if k == "H2O":
        continue
    a, r = res(f6[k], f7[k])
    rows.append((k, f6[k], f7[k], a, r))
T["ko"] = rows
h2_feed = f7["H2"]
R["psa"] = dict(
    h2_feed=h2_feed, h2_prod=prod["H2"], h2_tail=f8["H2"],
    recovery=prod["H2"] / h2_feed, recovery_12only=f12["H2"] / h2_feed,
    mass=res(S[7]["m"], S[8]["m"] + S[12]["m"] + S[13]["m"]),
    mol=res(S[7]["n"], S[8]["n"] + S[12]["n"] + S[13]["n"]),
    cond_w=cond_w, cond_kg=cond_w * MW["H2O"],
    tail_mw=S[8]["m"] / S[8]["n"], tail_dry_co2=S[8]["x"]["CO2"] / (1 - S[8]["x"]["H2O"]),
    feed_dry_co2=S[7]["x"]["CO2"] / (1 - S[7]["x"]["H2O"]),
    feed_nm3h=S[7]["n"] * V_N, tail_nm3h=S[8]["n"] * V_N,
)
f_tail = f8
tail_lhv_mj_h = lhv_mj_h(f_tail)
R["tail"] = dict(
    lhv_mj_kmol=tail_lhv_mj_h / S[8]["n"], lhv_mj_kg=tail_lhv_mj_h / S[8]["m"],
    lhv_mj_nm3=tail_lhv_mj_h / S[8]["n"] / V_N, mw_th=tail_lhv_mj_h / 3600.0,
    parts={k: f_tail[k] * LHV[k] / 3600.0 for k in LHV if f_tail[k] > 0},
)

# ---------------------------------------------------------------- B5 furnace: fuel = streams 8 + 3, product = stream 9, air derived
fuel = add(nflow(8), nflow(3))
ef, e9 = elem(fuel), elem(nflow(9))
f9 = nflow(9)
n_air_dry = (f9["N2"] - fuel["N2"]) / AIR_N2          # from N2 balance
o2_air = n_air_dry * AIR_O2
w_air = (e9["H"] - ef["H"]) / 2.0                     # from H balance (air humidity)
o_in = ef["O"] + 2 * o2_air + w_air
a_O, r_O = res(o_in, e9["O"])
a_C, r_C = res(ef["C"], e9["C"])
m_air_dry = n_air_dry * (AIR_O2 * MW["O2"] + AIR_N2 * MW["N2"])
m_air_w = w_air * MW["H2O"]
m_in_f = S[8]["m"] + S[3]["m"] + m_air_dry + m_air_w
a_m, r_m = res(m_in_f, S[9]["m"])
m_air_by_diff = S[9]["m"] - S[8]["m"] - S[3]["m"]
# stoichiometric O2 for the fuel (complete combustion of C to CO2, H to H2O), fuel O credited
o2_stoich = (ef["C"] + ef["H"] / 4.0 - ef["O"] / 2.0)
o2_excess = f9["O2"]
# humidity check at the reference condition (Annex Sheet 8 of 24: 9 C, 80 % RH, 101.3 kPa)
import math
t_ref = 9.0
psat_kpa = 0.61121 * math.exp(17.502 * t_ref / (240.97 + t_ref))     # Magnus form (assumed)
y_w_ref = 0.80 * psat_kpa / 101.3
hum_ref = y_w_ref / (1 - y_w_ref)                                   # kmol H2O / kmol dry air
# CO bound in flue gas: 30 mg/Nm3 at 3 % O2 dry, converted to actual O2 and dry flow
n_dry9 = S[9]["n"] * (1 - S[9]["x"]["H2O"])
o2_dry9 = S[9]["x"]["O2"] / (1 - S[9]["x"]["H2O"])
co_mg = 30.0 * (21.0 - 100 * o2_dry9) / (21.0 - 3.0)
co_kmol_h = co_mg * 1e-6 * n_dry9 * V_N / MW["CO"]
R["furn"] = dict(
    fuel_n=sum(fuel.values()), fuel_m=mass(fuel), ef=ef, e9=e9,
    n_air_dry=n_air_dry, m_air_dry=m_air_dry, w_air=w_air, hum_derived=w_air / n_air_dry, hum_ref=hum_ref,
    res_C=(a_C, r_C), res_O=(a_O, r_O), res_m=(a_m, r_m),
    m_air_by_diff=m_air_by_diff, m_air_total=m_air_dry + m_air_w,
    o2_stoich=o2_stoich, o2_air=o2_air, o2_excess=o2_excess, excess_air=o2_excess / (o2_air - o2_excess),
    o2_dry9=o2_dry9, co_mg=co_mg, co_kmol_h=co_kmol_h, co_c_frac=co_kmol_h / e9["C"],
    hnorm=n_air_dry * V_N,
)
# duty
q_tail = tail_lhv_mj_h / 3600.0                                           # MW (calculated, assumed LHVs)
q_ng_pub = S[3]["m"] * PUB["ng_lhv"] / 3600.0                              # MW (published LHV)
q_ng_cmp = S[3]["m"] * (lhv_ng_mol / M_ng_cmp) / 3600.0                    # MW (composition LHV)
q_rad = PUB["radiant_mmkcal"] * 1e6 * KCAL_KJ / 1e3 / 3600.0               # MJ/h -> MW
q_util_fuel = PUB["fuel_util_mmkcal"] * 1e6 * KCAL_KJ / 1e3 / 3600.0
q_util_fuel_tot = PUB["fuel_util_total_mmkcal"] * 1e6 * KCAL_KJ / 1e3 / 3600.0
R["duty"] = dict(
    q_tail=q_tail, q_ng_pub=q_ng_pub, q_ng_cmp=q_ng_cmp, q_fuel=q_tail + q_ng_pub,
    f_tail=q_tail / (q_tail + q_ng_pub), f_ng=q_ng_pub / (q_tail + q_ng_pub),
    q_rad=q_rad, rad_frac=q_rad / (q_tail + q_ng_pub), q_util=q_util_fuel, q_util_tot=q_util_fuel_tot,
    util_vs_ng=(q_util_fuel - q_ng_pub) / q_ng_pub,
    e_feed_mw=S[2]["m"] * PUB["ng_lhv"] / 3600.0,
)
R["duty"]["fuel_vs_input"] = (q_tail + q_ng_pub) / PUB["e_in_mw"]

# ---------------------------------------------------------------- B6 carbon chain NG -> stack and specific CO2
f1, f2, f3 = nflow(1), nflow(2), nflow(3)
cC = lambda fl: elem(fl)["C"]
C1, C2, C3, C8, C9 = cC(f1), cC(f2), cC(f3), cC(f8), cC(f9)
C7, C12, C13 = cC(f7), cC(f12), cC(f13)
M_CO2 = MW["CO2"]
t_co2_stack = f9["CO2"] * M_CO2 / 1000.0                        # t/h
t_co2_from_C_in = C1 * M_CO2 / 1000.0
h2_kg_h = S[12]["m"]
spec = dict(
    stack_t_h=t_co2_stack,
    per_nm3=t_co2_stack * 1000.0 / PUB["h2_nm3h"], per_kg_h2=t_co2_stack * 1000.0 / h2_kg_h,
    per_nm3_from_Cin=t_co2_from_C_in * 1000.0 / PUB["h2_nm3h"], per_kg_h2_from_Cin=t_co2_from_C_in * 1000.0 / h2_kg_h,
    pub_nm3=PUB["co2_spec"], pub_kg_h2=PUB["co2_spec"] * PUB["h2_nm3h"] / h2_kg_h,
    pub_t_h=PUB["co2_spec"] * PUB["h2_nm3h"] / 1000.0,
)
spec["res_nm3"] = res(spec["pub_nm3"], spec["per_nm3"])
spec["res_nm3_Cin"] = res(spec["pub_nm3"], spec["per_nm3_from_Cin"])
spec["res_kg"] = res(spec["pub_kg_h2"], spec["per_kg_h2"])
spec["rounding_stack"] = DATA_ROUNDING * S[9]["n"] * M_CO2 / 1000.0 / (t_co2_stack)
R["spec"] = spec
R["carbon"] = dict(
    C1=C1, C2=C2, C3=C3, C8=C8, C9=C9, C7=C7, C12=C12, C13=C13,
    res_split=res(C1, C2 + C3), res_ref_to_psa=res(C2, C8 + 0.0), res_psa=res(C7, C8 + C12 + C13),
    res_stack=res(C1, C9), res_stack_withCO=res(C1, C9 + co_kmol_h),
    res_fuel_tail=res(C8 + C3, C9),
    frac_tail=C8 / C9, frac_ng=C3 / C9,
)
# CO2 split of the stack (kmol/h of CO2 at the stack and mass)
tail_parts_kmol = dict(CO2_asis=f8["CO2"], CO_burnt=f8["CO"], CH4_burnt=f8["CH4"])
ng_parts_kmol = dict(CO2_asis=f3["CO2"], HC_burnt=C3 - f3["CO2"])
def tn(k): return k * M_CO2 / 1000.0
split = dict(
    tail=dict(total=tn(C8), parts={k: tn(v) for k, v in tail_parts_kmol.items()}, kmol=C8),
    ng=dict(total=tn(C3), parts={k: tn(v) for k, v in ng_parts_kmol.items()}, kmol=C3),
    stack=t_co2_stack,
)
split["sum"] = split["tail"]["total"] + split["ng"]["total"]
split["res"] = res(split["stack"], split["sum"])
for g in ("tail", "ng"):
    split[g]["per_nm3"] = split[g]["total"] * 1000.0 / PUB["h2_nm3h"]
    split[g]["per_kg"] = split[g]["total"] * 1000.0 / h2_kg_h
    split[g]["frac"] = split[g]["total"] / split["sum"]
    split[g]["parts_nm3"] = {k: v * 1000.0 / PUB["h2_nm3h"] for k, v in split[g]["parts"].items()}
    split[g]["parts_kg"] = {k: v * 1000.0 / h2_kg_h for k, v in split[g]["parts"].items()}
split["feed_C_in_NG_feedstock"] = dict(
    kmol=C2, t_co2eq=tn(C2), co2_in_feed_kmol=f2["CO2"], co2_in_feed_t=tn(f2["CO2"]))
R["split"] = split

# ---------------------------------------------------------------- B7 energy and efficiency (published inputs)
en = dict(
    e_in_calc=PUB["ng_total_th"] * PUB["ng_lhv"] / 3.6, e_out_calc=PUB["h2_th"] * PUB["h2_lhv"] / 3.6,
    feed_mw=PUB["ng_feed_th"] * PUB["ng_lhv"] / 3.6, fuel_mw=PUB["ng_fuel_th"] * PUB["ng_lhv"] / 3.6,
    elec_calc=PUB["gross_mwe"] + PUB["h2plant_mwe"] + PUB["cogen_mwe"],
    sc_feed_calc=PUB["ng_feed_th"] * PUB["ng_lhv"] * 1000.0 / PUB["h2_nm3h"],
    sc_fuel_calc=PUB["ng_fuel_th"] * PUB["ng_lhv"] * 1000.0 / PUB["h2_nm3h"],
    sum_ng=PUB["ng_feed_th"] + PUB["ng_fuel_th"],
    eta_h2=PUB["e_out_mw"] / PUB["e_in_mw"],
    eta_h2_el=(PUB["e_out_mw"] + PUB["export_mwe"]) / PUB["e_in_mw"],
)
en["res_in"] = res(PUB["e_in_mw"], en["e_in_calc"])
en["res_out"] = res(PUB["e_out_mw"], en["e_out_calc"])
en["res_el"] = res(PUB["export_mwe"], en["elec_calc"])
en["res_scf"] = res(PUB["sc_feed"], en["sc_feed_calc"])
en["res_scu"] = res(PUB["sc_fuel"], en["sc_fuel_calc"])
en["res_ng"] = res(PUB["ng_total_th"], en["sum_ng"])
R["en"] = en
bf = PUB["bfd"]
R["bfd"] = dict(ng=res(bf["ng"], S[1]["m"]), h2=res(bf["h2"], S[12]["m"]), flue=res(bf["flue"], S[9]["m"]),
                power=res(bf["power_kw"] / 1000.0, PUB["export_mwe"]))

# ---------------------------------------------------------------- extra checks
# worst-case rounding bands (published mole fractions have 4 decimals, flows 0.1 kmol/h)
def band_species(i, k):
    return DATA_ROUNDING * S[i]["n"] + 0.05 * S[i]["x"][k]


def band_elem(i, el):
    return sum(FORM[k].get(el, 0) * band_species(i, k) for k in SPEC if S[i]["x"][k] > 0 and FORM[k].get(el, 0))


R["band"] = dict(C1=band_elem(1, "C"), C2=band_elem(2, "C"), C3=band_elem(3, "C"), C8=band_elem(8, "C"),
                 C9=band_elem(9, "C"), C7=band_elem(7, "C"))
R["band"]["stack_chain"] = R["band"]["C1"] + R["band"]["C9"]
R["band"]["split_chain"] = R["band"]["C2"] + R["band"]["C3"] + R["band"]["C9"]

# dry-air O2 fraction that would close the furnace O balance exactly (N balance fixed, H balance gives air water)
N2_air = f9["N2"] - fuel["N2"]
target = e9["O"] - ef["O"] - w_air
yk = target / (2 * N2_air)                  # y/(1-y)
R["furn"]["y_close"] = yk / (1 + yk)
R["furn"]["N2_air"] = N2_air

# net water consumed by the reactions, from the derived steam, versus the utilities sheet (losses 44.2 t/h)
w2 = w2_H
net_w = (w1 + w2) - cond_w - f7["H2O"]
R["water"] = dict(w2_used=w2, steam_total=w1 + w2, steam_total_kg=(w1 + w2) * MW["H2O"], cond=cond_w,
                  net_kmol=net_w, net_th=net_w * MW["H2O"] / 1000.0, util_losses_th=44.2,
                  net_if14=((w1 + w2_14) - cond_w - f7["H2O"]) * MW["H2O"] / 1000.0)

# overall element balance: reformer + shift + PSA + furnace (control volume), H2 recycle internal
air_fl = {"N2": n_air_dry * AIR_N2, "O2": n_air_dry * AIR_O2, "H2O": w_air}
steam_fl = {"H2O": w1 + w2}
inn = add(f2, f3, steam_fl, air_fl)
outp = add(f9, f12, {"H2O": cond_w})
e_in, e_out = elem(inn), elem(outp)
R["overall"] = {el: (e_in[el], e_out[el]) + res(e_in[el], e_out[el]) for el in ELEM}
R["overall"]["mass"] = (mass(inn), mass(outp)) + res(mass(inn), mass(outp))

# carbon bound in H2 product from the spec (CO+CO2 <= 10 ppmv, Annex Table 2) and in condensate (knock-out residual)
R["c_leaks"] = dict(h2_spec_bound=10e-6 * S[12]["n"], ko_resid=abs(f7["CO2"] - f6["CO2"]) + abs(f7["CO"] - f6["CO"]) + abs(f7["CH4"] - f6["CH4"]),
                    co_flue=co_kmol_h)

# Table 2 capture-rate definitions (Case 1A and 2A): emitted, captured as published
cap = {}
for case, em, ca, pr_cap, pr_av in (("1A", .3704, .4660, .557, .542), ("2A", .3870, .4556, .541, .522)):
    cap[case] = dict(em=em, ca=ca, rate=ca / (ca + em), rate_pub=pr_cap, avoided=1 - em / PUB["co2_spec"], avoided_pub=pr_av)
R["cap"] = cap
R["case2A_check"] = dict(tail_co2_per_nm3=R["split"]["tail"]["parts_nm3"]["CO2_asis"], captured=.4556,
                         ratio=.4556 / R["split"]["tail"]["parts_nm3"]["CO2_asis"])
# Case 2A published tail-gas data: compressor 47,204 Nm3/h, MW 28.8, 28 C (PDF p152); 51 mol % CO2 in, 3.5 mol % out (PDF p136)
y_in, y_out = S[8]["x"]["CO2"], 0.035
inert = 1.0 - y_in
removed_frac = 1.0 - (y_out / (1.0 - y_out)) * (inert / y_in)
R["case2A_check"].update(comp_nm3h=47204.0, comp_rel=(47204.0 - R["psa"]["tail_nm3h"]) / R["psa"]["tail_nm3h"],
                         comp_mw=28.8, mw_rel=(28.8 - R["psa"]["tail_mw"]) / R["psa"]["tail_mw"], removed_frac=removed_frac)

# scaled split so that the parts sum to the published 0.8091 kg/Nm3
sc = PUB["co2_spec"] / (split["sum"] * 1000.0 / PUB["h2_nm3h"])
R["scaled"] = dict(factor=sc,
                   tail_co2=split["tail"]["parts_nm3"]["CO2_asis"] * sc, tail_co=split["tail"]["parts_nm3"]["CO_burnt"] * sc,
                   tail_ch4=split["tail"]["parts_nm3"]["CH4_burnt"] * sc, ng_co2=split["ng"]["parts_nm3"]["CO2_asis"] * sc,
                   ng_hc=split["ng"]["parts_nm3"]["HC_burnt"] * sc, tail=split["tail"]["per_nm3"] * sc, ng=split["ng"]["per_nm3"] * sc)

# ---------------------------------------------------------------- markdown writer
L = []
w = L.append


def table(head, rows, align=None):
    w("| " + " | ".join(head) + " |")
    w("|" + "|".join(["---"] * len(head)) + "|")
    for r in rows:
        w("| " + " | ".join(str(c) for c in r) + " |")
    w("")


def f(x, d=3):
    return f"{x:,.{d}f}"


def sg(x, d=3):
    return f"{x:+,.{d}f}"


def rp(x, d=4):
    return "n/a" if x != x else f"{100 * x:+.{d}f} %"


PDF16 = "PDF p16"
w("# IEAGHG 2017-02 Base Case (unabated SMR): source-traced reference and balances")
w("")
w("Report: IEAGHG 2017-02, Techno-Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS (final, December 2016, 286 PDF pages). "
  "Everything below refers to the Base Case (no CO2 capture). Produced by `ieaghg_balances.py` (run with `python -I`); every calculated number is regenerated by that script.")
w("")
w("Labels: **published** = read from the report; **calculated** = computed from published values with the equation shown; **assumed** = constant or convention that the report does not state. "
  "Residual = out minus in; relative residual = residual / in. Page references give the PDF page and the printed sheet or table number. "
  "The executive summary pages carry no printed page number, so Table 2 is cited as PDF p16 plus its table number.")
w("")
w("## 0. Verdict in four lines")
w("")
w(f"1. All twelve owner target values match the report to the printed digits (section 1). The specific CO2 of {PUB['co2_spec']} kg/Nm3 H2 is the stack CO2 of the Base Case (flue gas stream 9 gives {R['spec']['per_nm3']:.5f} kg/Nm3 H2, calculated).")
w(f"2. The tail-gas flow is published (stream 8: {S[8]['n']} kmol/h, {S[8]['m']} kg/h, CO2 {S[8]['x']['CO2']}, CO {S[8]['x']['CO']}, H2 {S[8]['x']['H2']}, CH4 {S[8]['x']['CH4']}). Its LHV, the furnace fuel split by energy and the combustion air flow are not published and are derived here.")
w(f"3. Carbon closes from natural gas to stack: {f(C1,1)} kmol C/h in stream 1, {f(C9,1)} kmol C/h as CO2 in stream 9 ({rp(R['carbon']['res_stack'][1],3)}, inside the worst-case rounding band of {f(R['band']['stack_chain'],1)} kmol/h).")
w(f"4. Of the stack CO2, {pct(split['tail']['frac'],1)} comes from carbon that entered as feedstock and left in the PSA tail gas, and {pct(split['ng']['frac'],1)} from the supplementary natural gas. The published 0.8091 already contains all tail-gas combustion and the pass-through process CO2. Adding a tail-gas combustion term on top of 0.8091 double counts (section 8).")
w("")

# ------------------------------------------------------------ section 1: owner targets
w("## 1. Owner verification targets against the report")
w("")
rows = [
    ("Hydrogen product, Nm3/h", "100,000", "100,000", "published", "Table 2 (PDF p16); Sheet 20 of 112 (PDF p46); stream 12 gives 4461.5 kmol/h x 22.414 = " + f(R['nm3']['v'], 1) + " Nm3/h (calculated)", "match"),
    ("Hydrogen product, t/h", "8.994", "8.994", "published", "Table 2; Sheet 20; stream 12 = 8994 kg/h (PDF p44); BFD battery limits 8994 kg/h (PDF p34)", "match"),
    ("NG feedstock, t/h", "26.231", "26.231", "published", "Table 2 row 'NG to Feedstock'; Sheet 20; stream 2 = 26231 kg/h", "match"),
    ("NG to furnace as supplementary fuel, t/h", "4.332", "4.332", "published", "Table 2 row 'NG to Fuel'; Sheet 20; stream 3 = 4332 kg/h. This is the supplementary natural gas only; the PSA tail gas (stream 8, 60,658 kg/h) is the primary furnace fuel and is not in this figure", "match, label with care"),
    ("NG total, t/h", "30.563", "30.563", "published", "Sheet 20 'Natural Gas (Total Consumption)'; stream 1 = 30563 kg/h; BFD (PDF p34). Calculated 26.231 + 4.332 = " + f(R['en']['sum_ng'], 3), "match"),
    ("NG LHV, MJ/kg", "46.50", "46.50", "published", "Table 2; Sheet 20; Annex Table 1 (PDF p255, Sheet 9 of 24) gives 46.502", "match"),
    ("Total energy input, MW", "394.77", "394.77", "published", "Table 2; Sheet 20. Calculated 30.563 t/h x 46.50 MJ/kg / 3.6 = " + f(R['en']['e_in_calc'], 3) + " MW", "match"),
    ("Hydrogen energy output, MW", "299.70", "299.70", "published", "Table 2; Sheet 20. Calculated 8.994 x 119.96 / 3.6 = " + f(R['en']['e_out_calc'], 3) + " MW", "match"),
    ("Gross electricity, MWe", "11.500", "11.500", "published", "Table 2 'Gross Power Output from COGEN Plant'; Sheet 20; utilities sheet (PDF p48) -11,500 kW", "match"),
    ("Net exported electricity, MWe", "9.918", "9.918", "published", "Table 2 'Export Power to the Grid'; Sheet 20; BFD 9918 kW. Calculated 11.500 - 1.216 - 0.366 = " + f(R['en']['elec_calc'], 3), "match"),
    ("CO2 intensity, kg/Nm3 H2", "0.8091", "0.8091", "published", "Table 2 'Specific CO2 Emissions' 0.8091 kg/Nm3 H2; Sheet 20 prints 0.8091 t/1000 Nm3 H2", "match"),
    ("CO2 intensity, kg/kg H2", "8.996", f(R['spec']['pub_kg_h2'], 4), "calculated", "0.8091 kg/Nm3 x 100,000 Nm3/h / 8994 kg/h = " + f(R['spec']['pub_kg_h2'], 4) + " (the report prints no per-kg figure). From stream 9: " + f(R['spec']['per_kg_h2'], 4), "match"),
]
table(["Target", "Owner value", "Report value", "Label", "Evidence", "Verdict"], rows)

# ------------------------------------------------------------ section 2: source map
w("## 2. Source map (Base Case)")
w("")
rows = [
    ("Table 2 Plant Performance Summary (all cases)", "PDF p16", "Executive summary, no printed page number; Base Case column"),
    ("Section 2.3 Overall block flow diagram, battery-limit table", "PDF p34", "A3 insert after Sheet 11 of 112; NG 30563 kg/h, raw water 59700 kg/h, sea water 3435 m3/h, H2 8994 kg/h, flue gas 257698 kg/h, power 9918 kW, waste water 15500 kg/h"),
    ("Section 2.4.1 Hydrogen plant process description", "PDF p35 to p40", "Sheets 12 to 17 of 112; S/C 2.7 to 2.8 (Sheet 12), reformer and furnace (Sheets 13 and 14), PSA and tail-gas routing (Sheet 14)"),
    ("Section 2.5 Process flow diagram, hydrogen plant", "PDF p42", "A3 insert after Sheet 18 of 112; 'UNIT: Hydrogen Plant, Case: Base case, Sheet 01 of 01', rev 1, 15/06/2015; diamonds 1 to 15 are the stream numbers of the H&M sheet; combustion air is drawn but not numbered"),
    ("Section 2.6 Heat and material balance, streams 1 to 12", "PDF p44", "A3 insert after Sheet 19 of 112; Foster Wheeler contract 1BD0840A, rev 1 June 2015"),
    ("Section 2.6 Heat and material balance, streams 13 to 15", "PDF p45", "Same sheet series"),
    ("Section 2.7 Plant performance data, Base Case", "PDF p46", "Sheet 20 of 112"),
    ("Section 2.8 Estimated utility consumptions", "PDF p48", "Unnumbered insert after Sheet 21 of 112; fuel 48.2 MMkcal/h hydrogen plant, 0.5 utilities, 48.7 total"),
    ("Section 2.9 Preliminary equipment list, steam reformer furnace", "PDF p54", "Equipment list sheet 5 of 8; 82.63 MMkcal/h marked '(**) Radiant Duty'"),
    ("Executive summary basis of design, CO2 origin and tail-gas routing", "PDF p6", "'all of the CO2 is emitted from the flue gas of the steam reformer'; tail gas 'fed into the burners of the SMR as its primary fuel'; PSA recovery 85 to 90 %"),
    ("Executive summary footnote 2", "PDF p5", "16 %v CO2 at 2.5 MPa shifted syngas; 51 %v CO2 at 0.2 MPa PSA tail gas (nominal; stream 8 gives 0.5095 at 0.13 MPa)"),
    ("Annex Reference Document: climate (9 C, 80 % RH, 101.3 kPa reference)", "PDF p254", "Sheet 8 of 24"),
    ("Annex: NG specification (Table 1)", "PDF p255", "Sheet 9 of 24"),
    ("Annex: hydrogen specification (Table 2)", "PDF p256", "Sheet 10 of 24; H2 99.5 %v min, CO + CO2 10 ppm max, LHV 119.96 MJ/kg"),
    ("Annex: capacity and emission limits", "PDF p258 and p259", "Sheets 12 and 13 of 24; CO 30 mg/Nm3, NOx 120 mg/Nm3 at 3 % O2 dry; 100,000 Nm3/h H2"),
]
table(["Item", "Where", "Note"], rows)
w("Process route of the Base Case (Sheets 12 to 17, PDF p35 to p40; PFD PDF p42). Natural gas arrives at 9 C and 7.0 MPa, is preheated to 135 C and let down to 3.70 MPa, and splits into feedstock (stream 2) and burner fuel (stream 3). "
  "The feedstock takes a small H2 recycle, is desulphurised, mixed with HP steam (overall steam to carbon 2.7 to 2.8), pre-reformed adiabatically, given a second steam addition and reformed in the terrace-wall tubes (3.3 to 4 % CH4 dry at the outlet). "
  "The effluent is cooled to 320 C in the reformer waste heat boiler, shifted in one HT shift reactor, cooled, stripped of condensate and sent to the PSA. The PSA gives 40 C, 2.5 MPa hydrogen; its tail gas is the primary burner fuel and is topped up with natural gas. "
  "Combustion air is preheated by the flue gas, which leaves at 136 C. Excess HP steam (4.23 MPa, 395 C, stream 10) drives a condensing turbine for 11.5 MWe gross.")
w("")

# ------------------------------------------------------------ section 3: stream tables
w("## 3. Stream tables (published, PDF p44 and p45)")
w("")
w("Pressure unit is MPa as printed; the sheet does not say gauge or absolute. Compositions are mol/mol, wet basis, as printed.")
w("")
ids_a = [1, 2, 3, 4, 5, 6, 7, 8]
ids_b = [9, 10, 11, 12, 13, 14, 15]


def stream_tab(ids):
    head = ["Property"] + [f"{i} {S[i]['name']}" for i in ids]
    rows = [["T, C"] + [S[i]["T"] for i in ids], ["P, MPa"] + [f"{S[i]['P']:.2f}" for i in ids],
            ["Molar flow, kmol/h"] + [f"{S[i]['n']:.1f}" for i in ids], ["Mass flow, kg/h"] + [f"{S[i]['m']:,}" for i in ids]]
    for k in SPEC:
        row = [k]
        for i in ids:
            v = S[i]["x"][k]
            if i in (12, 13) and k == "H2":
                row.append("0.9999+")
            elif i == 9 and k == "CO":
                row.append("30 mg/Nm3 max")
            else:
                row.append(f"{v:.4f}")
        if any(c not in ("0.0000",) for c in row[1:]):
            rows.append(row)
    table(head, rows)


stream_tab(ids_a)
stream_tab(ids_b)
w("Notes printed on the sheet: stream 1 carries 5 ppmv H2S for design purposes (removed in the sulphur adsorber); the flue gas carries NOx 120 mg/Nm3 max and CO 30 mg/Nm3 max. "
  "Streams that carry carbon: 1, 2, 3, 4, 5, 6, 7, 8 and 9 (H2 product 12 and 13 hold at most 10 ppmv CO + CO2 by specification). "
  "Mapping of the requested streams to H&M numbers: natural gas feed 1 (split into 2 and 3); desulphurised feed 4 (already mixed with the first steam addition and the H2 recycle); process steam 14 (see section 7.2 for a discrepancy); "
  "reformer outlet 5 (HTS inlet, cooled to 320 C); shift outlet 6; PSA feed 7; PSA tail gas 8 (also the main fuel gas); NG fuel 3; flue gas 9; hydrogen product 12. Combustion air has no number and no published flow.")
w("")
w("Stream-table integrity (calculated): sum of mole fractions, and published M = m/n against M from composition and the atomic masses.")
w("")
table(["Stream", "Sum x", "M published, kg/kmol", "M from composition", "Relative difference"],
      [(r[0], f"{r[2]:.4f}", f"{r[3]:.3f}", f"{r[4]:.3f}", rp(r[5], 3)) for r in T["integrity"]])
w("Stream 13 and 15 differences come from integer or one-decimal rounding of tiny flows (59 kg/h, 30.0 kmol/h). Everything else agrees to 0.03 % or better.")
w("")

# ------------------------------------------------------------ section 4: PSA
w("## 4. PSA unit")
w("")
w("Published: feed stream 7 (35 C, 2.58 MPa, 6596.9 kmol/h, 69,711 kg/h); tail gas stream 8 (28 C, 0.13 MPa, 2106.3 kmol/h, 60,658 kg/h); product stream 12 (40 C, 2.50 MPa, 4461.5 kmol/h, 8994 kg/h, H2 0.9999+) and recycle stream 13 (40 C, 2.51 MPa, 29.1 kmol/h, 59 kg/h). "
  "Text (Sheet 14, PDF p37; executive summary PDF p6): 'typically 85 to 90 %' of the PSA feed H2 is recovered, purity 99.9+ %, the tail gas goes to the SMR burners as the main fuel. "
  "The report gives no single stated recovery value for the Base Case.")
w("")
w("Calculated from the streams:")
w("")
p = R["psa"]
rows = [
    ("H2 in PSA feed", f"n7 x y_H2 = {S[7]['n']} x {S[7]['x']['H2']}", f"{f(p['h2_feed'],1)} kmol/h"),
    ("H2 in product + recycle", f"n12 + n13 = {S[12]['n']} + {S[13]['n']}", f"{f(p['h2_prod'],1)} kmol/h"),
    ("H2 in tail gas", f"n8 x y_H2 = {S[8]['n']} x {S[8]['x']['H2']}", f"{f(p['h2_tail'],1)} kmol/h"),
    ("H2 recovery (product + recycle)", "(n12 + n13) / (n7 y_H2)", pct(p["recovery"], 2)),
    ("H2 recovery (net product only)", "n12 / (n7 y_H2)", pct(p["recovery_12only"], 2)),
    ("Product purity", "published", ">= 99.99 mol % (0.9999+); Annex spec 99.5 %v min"),
    ("Tail-gas molar mass", "m8 / n8", f"{p['tail_mw']:.3f} kg/kmol"),
    ("Tail-gas CO2, dry basis", "y_CO2 / (1 - y_H2O)", pct(p["tail_dry_co2"], 2)),
    ("Process condensate removed upstream of PSA", "n6(H2O) - n7(H2O)", f"{f(p['cond_w'],1)} kmol/h = {f(p['cond_kg']/1000,2)} t/h water (derived, unpublished)"),
    ("Tail-gas LHV (derived, assumed LHV constants)", "sum n_i LHV_i / n8", f"{R['tail']['lhv_mj_kmol']:.2f} MJ/kmol = {R['tail']['lhv_mj_kg']:.3f} MJ/kg = {R['tail']['lhv_mj_nm3']:.3f} MJ/Nm3 (0 C basis)"),
    ("Tail-gas heat flow (LHV)", "n8 x LHV", f"{R['tail']['mw_th']:.2f} MW (H2 {R['tail']['parts']['H2']:.2f}, CO {R['tail']['parts']['CO']:.2f}, CH4 {R['tail']['parts']['CH4']:.2f} MW)"),
]
table(["Quantity", "Equation and substitution", "Value (calculated)"], rows)
w("Not published for the Base Case: PSA hydrogen recovery as a single number, tail-gas LHV, tail-gas compression or heating before the burners (the sheet gives 28 C and 0.13 MPa at the PSA outlet and 0.50 MPa for the NG fuel header only), "
  "and the split of H2 between the sales stream and the recycle other than the two stream flows. The tail-gas LHV above uses standard thermochemical LHVs (assumed) that reproduce the published NG LHV within "
  f"{abs(R['ng']['lhv_dev'])*100:.3f} % and the published H2 LHV within {abs(R['ng']['h2_dev'])*100:.3f} % (section 6).")
w("")
w("PSA species balance, n_i,feed = n_i,H2 product + n_i,tail gas (kmol/h; H2 product taken as pure H2 and as the sum of streams 12 and 13):")
w("")
rows = []
for k, a, b, c, d_, rr, band in T["psa"]:
    if a == 0 and b == 0 and c == 0:
        continue
    rows.append((k, f"{a:.3f}", f"{b:.3f}", f"{c:.3f}", sg(d_, 3), rp(rr, 3) if rr == rr else "n/a", f"+/-{band:.2f}"))
rows.append(("Total moles", f"{S[7]['n']:.1f}", f"{S[8]['n']:.1f}", f"{S[12]['n'] + S[13]['n']:.1f}", sg(p["mol"][0], 3), rp(p["mol"][1], 4), "-"))
rows.append(("Total mass, kg/h", f"{S[7]['m']}", f"{S[8]['m']}", f"{S[12]['m'] + S[13]['m']}", sg(p["mass"][0], 1), rp(p["mass"][1], 4), "-"))
table(["Species", "Feed (stream 7)", "Tail gas (stream 8)", "Product (12 + 13)", "Residual", "Relative", "Rounding band"], rows)
w("Residuals are at or below the rounding of the 4-decimal mole fractions (band = 0.00005 x (n7 + n8)); N2 and H2O show larger relative values only because their flows are tiny. "
  "Total mass closes exactly (69,711 = 60,658 + 8994 + 59).")
w("")

# ------------------------------------------------------------ section 5: furnace
w("## 5. Furnace (steam reformer, radiant plus convective)")
w("")
d_ = R["duty"]
fu = R["furn"]
rows = [
    ("Radiant duty, published", "Equipment list sheet 5 of 8 (PDF p54), 82.63 MMkcal/h", f"{d_['q_rad']:.2f} MW (calculated, 82.63 x 1e6 x 4.1868 kJ/kcal / 3.6e6)"),
    ("Primary fuel: PSA tail gas", "stream 8, LHV derived", f"{d_['q_tail']:.2f} MW ({pct(d_['f_tail'],1)} of burner LHV)"),
    ("Supplementary fuel: natural gas", "stream 3, 4.332 t/h x 46.50 MJ/kg / 3.6 (published inputs)", f"{d_['q_ng_pub']:.3f} MW ({pct(d_['f_ng'],1)})"),
    ("Total burner LHV input", "sum", f"{d_['q_fuel']:.2f} MW = {pct(d_['fuel_vs_input'],2)} of the 394.77 MW plant energy input"),
    ("Radiant duty as fraction of burner LHV", "96.10 / " + f"{d_['q_fuel']:.2f}", f"{pct(d_['rad_frac'],1)} (an indicator only; convective duties are not published)"),
    ("Utilities sheet 'FUEL'", "PDF p48: 48.2 MMkcal/h hydrogen plant, 48.7 total", f"{d_['q_util']:.2f} MW vs stream 3 at {d_['q_ng_pub']:.2f} MW ({rp(d_['util_vs_ng'],2)}); the utilities fuel is the supplementary NG only"),
    ("Flue gas flow", "stream 9 (published)", f"{S[9]['n']} kmol/h, {S[9]['m']:,} kg/h at 136 C and 0.02 MPa; BFD battery limits agree (PDF p34)"),
    ("Flue gas composition (wet)", "stream 9 (published)", "CO2 0.2123, N2 0.6083, O2 0.0102, H2O 0.1692; CO 30 mg/Nm3 max; NOx 120 mg/Nm3 max"),
    ("Stack CO2", "n9 x y_CO2 x 44.009 / 1000", f"{f(f9['CO2'],1)} kmol/h = {f(spec['stack_t_h'],3)} t/h (calculated)"),
    ("Dry-basis flue O2", "0.0102 / (1 - 0.1692)", pct(fu["o2_dry9"], 2)),
]
table(["Quantity", "Source or equation", "Value"], rows)
w("**Efficiency figures.** The report states no furnace efficiency and no overall hydrogen production efficiency for the Base Case (the word appears only in qualitative remarks). "
  "Calculated indicators from published Table 2 values: hydrogen LHV efficiency = B / A = 299.70 / 394.77 = " + pct(R['en']['eta_h2'], 2) +
  "; the same with the 9.918 MWe export added at face value = (299.70 + 9.918) / 394.77 = " + pct(R['en']['eta_h2_el'], 2) +
  ". These are plant-level figures; a furnace thermal efficiency cannot be derived from the published data (no flue-gas or process enthalpies are given).")
w("")
w("**Combustion air** is not a numbered stream. Derived here (assumed dry air 21 % O2 and 79 % N2): from the N2 balance, n_air = (n9 y_N2 - N2 in fuel) / 0.79 = "
  f"({f(f9['N2'],1)} - {f(fuel['N2'],1)}) / 0.79 = {f(fu['n_air_dry'],1)} kmol/h dry air = {f(fu['m_air_dry'],0)} kg/h; humidity from the H balance = {f(fu['w_air'],1)} kmol/h H2O "
  f"(humidity ratio {fu['hum_derived']:.5f}, against {fu['hum_ref']:.5f} for the Annex reference condition 9 C, 80 % RH, 101.3 kPa, Magnus vapour pressure, assumed). "
  f"Total air {f(fu['m_air_total'],0)} kg/h by balance and {f(fu['m_air_by_diff'],0)} kg/h by mass difference (257,698 - 60,658 - 4332). "
  f"Excess air from the flue O2: {f(fu['o2_excess'],1)} / ({f(fu['o2_air'],1)} - {f(fu['o2_excess'],1)}) = {pct(fu['excess_air'],2)}.")
w("")

# ------------------------------------------------------------ section 6: NG
w("## 6. Natural gas composition and LHV")
w("")
w("Published, Annex Table 1 (PDF p255, Sheet 9 of 24), vol %: CH4 89.0, C2H6 7.0, C3H8 1.0, butane 0.1, pentane 0.01, CO2 2.0, N2 0.89, H2S 5 ppmv (design value). "
  "HHV 51.473 MJ/kg and LHV 46.502 MJ/kg; delivered at 7.0 MPa. The H&M sheet (streams 1 to 3) repeats the same composition. Table 2 uses 46.50 MJ/kg. "
  "The report lists the species as butane and pentane; the H&M sheet names them n-butane and n-pentane.")
w("")
ng = R["ng"]
rows = [
    ("Carbon per kmol NG", "sum y_i c_i", f"{ng['c_per_kmol']:.4f} kmol C/kmol"),
    ("Molar mass of NG from composition", "sum y_i M_i", f"{ng['M']:.3f} kg/kmol (published m/n: {ng['M_pub']:.3f})"),
    ("NG LHV from composition (assumed constants)", "sum y_i LHV_i / M", f"{ng['lhv_cmp_mj_kg']:.3f} MJ/kg vs published 46.502 ({rp(ng['lhv_dev'],3)})"),
    ("H2 LHV from constant", f"{LHV['H2']} / {MW['H2']:.3f}", f"{ng['h2_lhv_cmp']:.3f} MJ/kg vs published 119.96 ({rp(ng['h2_dev'],3)})"),
    ("Split of stream 1 into 2 + 3, molar", "n2 + n3 vs n1", f"{S[2]['n'] + S[3]['n']:.1f} vs {S[1]['n']} kmol/h ({sg(ng['split_n'][0],2)}, {rp(ng['split_n'][1],3)})"),
    ("Split of stream 1 into 2 + 3, mass", "m2 + m3 vs m1", f"{S[2]['m'] + S[3]['m']} vs {S[1]['m']} kg/h (exact)"),
    ("Fuel share of NG, mass", "m3 / m1", pct(ng["frac_fuel_m"], 2)),
    ("Nm3 convention", "n12 x 22.414 vs 100,000", f"{f(R['nm3']['v'],1)} Nm3/h ({rp(R['nm3']['rel'],5)}): the report's Nm3 is 0 C and 101.325 kPa (inferred, not stated in the pages read)"),
]
table(["Quantity", "Equation", "Value (calculated)"], rows)

# ------------------------------------------------------------ section 7: balances
w("## 7. Component balances from published stream data")
w("")
w("Assumed constants: atomic masses C 12.011, H 1.008, O 15.999, N 14.007; LHV (MJ/kmol, 25 C, water vapour) H2 241.8, CO 283.0, CH4 802.3, C2H6 1428.6, C3H8 2043.1, n-C4H10 2657.5, n-C5H12 3272.6; dry air 21 % O2 and 79 % N2; 1 kcal = 4.1868 kJ; 22.414 Nm3/kmol. "
  "H2 product taken as pure H2. Flue-gas CO taken as 0 (bounded below). Rounding band = worst-case effect of the 4-decimal fractions and 0.1 kmol/h flows.")
w("")
rf = R["ref"]
w("### 7.1 Pre-reformer, reformer and shift (stream 4 plus second steam addition to stream 5, then stream 6)")
w("")
w("Equations: C_in = sum_i n_i c_i over stream 4, C_out over stream 5; the H and O balances each give the unpublished second steam addition w2 (kmol/h H2O): "
  "w2 = (H5 - H4) / 2 and w2 = O5 - O4.")
w("")
rows = [
    ("C", f(rf["e4"]["C"], 2), f(rf["e5"]["C"], 2), sg(rf["res_C"][0], 3), rp(rf["res_C"][1], 4)),
    ("N", f(rf["e4"]["N"], 2), f(rf["e5"]["N"], 2), sg(rf["res_N"][0], 3), rp(rf["res_N"][1], 2) + " (N2 0.0023 vs 0.0015, rounding)"),
    ("H (with w2 from H)", f(rf["e4"]["H"], 1) + f" + 2 x {rf['w2_H']:.2f}", f(rf["e5"]["H"], 1), "0 by construction", "-"),
    ("O (with w2 from H)", f(rf["e4"]["O"], 1) + f" + {rf['w2_H']:.2f}", f(rf["e5"]["O"], 1), sg(rf["w2_O"] - rf["w2_H"], 2), rp((rf["w2_O"] - rf["w2_H"]) / rf["e5"]["O"], 3)),
]
table(["Element", "In, kmol/h (stream 4 + water)", "Out, kmol/h (stream 5)", "Residual", "Relative"], rows)
w(f"The two estimates of the second steam addition agree: w2 from H = {rf['w2_H']:.2f} kmol/h, from O = {rf['w2_O']:.2f} kmol/h (difference {rf['dw2']:.2f} kmol/h, within rounding). "
  f"Total process steam to the reformer = {f(rf['w1'],1)} (in stream 4) + {rf['w2_H']:.1f} = {f(rf['w1']+rf['w2_H'],1)} kmol/h = {f(rf['steam_kg_H']/1000,2)} t/h. "
  f"Steam to carbon: {rf['sc_H']:.3f} on all carbon, {rf['sc_hc_H']:.3f} on hydrocarbon carbon (the report states 2.7 to 2.8, Sheet 12). "
  f"Stream 4 mass check: NG 26,231 + H2 recycle 59 + steam {f(rf['w1']*MW['H2O'],0)} = {f(rf['mass4_chk'],0)} vs 98,874 kg/h ({rp(rf['mass4_res'][1],4)}).")
w("")
rows = [
    ("Moles 5 to 6 (shift)", f"{S[5]['n']}", f"{S[6]['n']}", sg(rf["n56"][0], 2), "0"),
    ("C 5 to 6", f(rf["e5"]["C"], 2), f(rf["e6"]["C"], 2), sg(rf["res_C56"][0], 3), rp(rf["res_C56"][1], 4)),
    ("H 5 to 6", f(rf["e5"]["H"], 1), f(rf["e6"]["H"], 1), sg(rf["res_H56"][0], 3), "0"),
    ("O 5 to 6", f(rf["e5"]["O"], 1), f(rf["e6"]["O"], 1), sg(rf["res_O56"][0], 3), rp(rf["res_O56"][1], 4)),
]
table(["Shift balance", "In (stream 5)", "Out (stream 6)", "Residual", "Relative"], rows)
sh = rf["shift"]
w(f"Shift extent: CO {sg(sh['CO'],1)}, H2O {sg(sh['H2O'],1)}, CO2 {sg(sh['CO2'],1)}, H2 {sg(sh['H2'],1)} kmol/h (equimolar within 1 kmol/h). "
  f"Dry-basis CO at the shift outlet is {pct(rf['co_dry6'],2)} and dry CH4 at the reformer outlet is {pct(rf['ch4_dry5'],2)}. "
  "The text (Sheet 14; executive summary) quotes residual CO of 2.5 to 3.5 % dry as typical for an HTS, while the stream table gives 4.65 % dry; the stream table is used here. "
  f"Chemical heat of reforming, 25 C basis, LHV(stream 5) - LHV(stream 4) = {rf['dLHV_MW']:.1f} MW (calculated, informational; the radiant duty is {d_['q_rad']:.1f} MW).")
w("")
w("### 7.2 Water cross-check and a discrepancy in stream 14")
w("")
wt = R["water"]
w(f"Net water consumed by the reactions = steam fed - condensate removed - water in PSA feed = ({f(rf['w1'],1)} + {wt['w2_used']:.1f}) - {f(wt['cond'],1)} - {f(f7['H2O'],1)} = {f(wt['net_kmol'],1)} kmol/h = {wt['net_th']:.1f} t/h (calculated). "
  f"The utilities sheet (PDF p48) lists losses of 44.2 t/h for the hydrogen plant 'including water consumed in the reaction and deaerator vent'. "
  f"The difference of {44.2 - wt['net_th']:.1f} t/h is of the size of a deaerator vent (not itemised). "
  f"Stream 14 'HP steam to process' is {S[14]['m']:,} kg/h ({S[14]['n']} kmol/h). If it were all reformer steam, the second addition would be {rf['w2_14']:.0f} kmol/h instead of {rf['w2_H']:.0f}, "
  f"S/C would be {rf['sc_14']:.2f}, and net water consumption would be {wt['net_if14']:.1f} t/h, which is inconsistent with the 44.2 t/h. "
  f"Conclusion: the H and O balances imply about {f(wt['steam_total_kg']/1000,1)} t/h of reformer steam, and with that flow the net water consumption matches the utilities sheet; stream 14 is about {f((S[14]['m']-wt['steam_total_kg'])/1000,1)} t/h above it and the report does not say where the difference goes (unresolved). No carbon balance depends on stream 14.")
w("")
w("### 7.3 Condensate knock-out (stream 6 to stream 7)")
w("")
rows = [(k, f"{a:.3f}", f"{b:.3f}", sg(c, 3), rp(d2, 3) if d2 == d2 else "n/a") for k, a, b, c, d2 in T["ko"] if a or b]
table(["Species (non-water)", "Stream 6", "Stream 7", "Residual", "Relative"], rows)
w(f"Non-water species pass to the PSA feed within rounding (the 5 % for N2 is 0.64 kmol/h, inside the band set by N2 fractions of 0.0015 and 0.0020). The summed absolute residual of the carbon species is {R['c_leaks']['ko_resid']:.2f} kmol/h, "
  "an upper bound on carbon leaving as dissolved gas in process condensate or the deaerator vent (not published).")
w("")
w("### 7.4 PSA")
w("")
w("See section 4 for the species balance table. H2 recovery (n12 + n13) / (n7 y_H2) = " + pct(p['recovery'], 2) + ".")
w("")
w("### 7.5 Furnace")
w("")
rows = [
    ("C", f(fu["ef"]["C"], 2), f(fu["e9"]["C"], 2), sg(fu["res_C"][0], 3), rp(fu["res_C"][1], 4), "none"),
    ("N (fixes air)", f"{f(fu['ef']['N'],2)} + 2 x {f(fu['n_air_dry']*AIR_N2,1)}", f(fu["e9"]["N"], 1), "0 by construction", "-", "air"),
    ("H (fixes humidity)", f"{f(fu['ef']['H'],1)} + 2 x {fu['w_air']:.2f}", f(fu["e9"]["H"], 1), "0 by construction", "-", "air water"),
    ("O", f"{f(fu['ef']['O'],1)} + 2 x {f(fu['o2_air'],1)} + {fu['w_air']:.2f}", f(fu["e9"]["O"], 1), sg(fu["res_O"][0], 2), rp(fu["res_O"][1], 3), "air composition"),
    ("Mass, kg/h", f"{S[8]['m']} + {S[3]['m']} + {f(fu['m_air_dry'],0)} + {f(fu['w_air']*MW['H2O'],0)}", f"{S[9]['m']}", sg(fu["res_m"][0], 0), rp(fu["res_m"][1], 3), "air composition"),
]
table(["Element", "In, kmol/h (fuel = streams 8 + 3, plus air)", "Out, kmol/h (stream 9)", "Residual", "Relative", "Unknown it uses"], rows)
w(f"Carbon closes to {rp(fu['res_C'][1],3)}. The O residual of {sg(fu['res_O'][0],1)} kmol/h disappears if dry air has {fu['y_close']:.4f} mole fraction O2 instead of 0.21 (calculated); the report does not publish the air composition, so this residual is the air-composition uncertainty. "
  f"Bound on flue CO (30 mg/Nm3 at 3 % O2 dry converted to {fu['co_mg']:.1f} mg/Nm3 at the actual {pct(fu['o2_dry9'],2)} dry O2): {fu['co_kmol_h']:.2f} kmol/h, {pct(fu['co_c_frac'],3)} of the stack carbon.")
w("")
w("Overall control volume (reformer + shift + PSA + furnace, H2 recycle internal; inputs streams 2 and 3, derived steam, derived air; outputs streams 9 and 12, derived condensate):")
w("")
ov = R["overall"]
rows = [(el, f(ov[el][0], 2), f(ov[el][1], 2), sg(ov[el][2], 2), rp(ov[el][3], 4)) for el in ELEM]
rows.append(("Mass, kg/h", f(ov["mass"][0], 0), f(ov["mass"][1], 0), sg(ov["mass"][2], 0), rp(ov["mass"][3], 4)))
table(["Element", "In, kmol/h", "Out, kmol/h", "Residual", "Relative"], rows)
w("C is the only independent element in this table (no derived unknown enters it). H and N close largely by construction because the steam, air and condensate flows are derived from those elements; O is the check on them.")
w("")
w("### 7.6 Furnace duty from LHV")
w("")
w(f"Q_tail = n8 x sum y_i LHV_i = {S[8]['n']} x {R['tail']['lhv_mj_kmol']:.2f} MJ/kmol / 3600 = {d_['q_tail']:.2f} MW; "
  f"Q_NG = 4.332 t/h x 46.50 MJ/kg / 3.6 = {d_['q_ng_pub']:.3f} MW (composition LHV gives {d_['q_ng_cmp']:.3f} MW); total {d_['q_fuel']:.2f} MW. "
  f"Feedstock energy {d_['e_feed_mw']:.2f} MW + fuel NG {d_['q_ng_pub']:.2f} MW = 394.77 MW. The radiant duty of {d_['q_rad']:.2f} MW is {pct(d_['rad_frac'],1)} of the burner LHV.")
w("")
w("### 7.7 Carbon from natural gas to stack, and specific CO2")
w("")
cb = R["carbon"]
bd = R["band"]
rows = [
    ("NG split", "C(stream 1) vs C(2) + C(3)", f(cb["C1"], 2), f(cb["C2"] + cb["C3"], 2), sg(cb["res_split"][0], 2), rp(cb["res_split"][1], 3), f"+/-{bd['C1']+bd['C2']+bd['C3']:.1f}"),
    ("Feedstock carbon to tail gas", "C(stream 2) vs C(stream 8)", f(cb["C2"], 2), f(cb["C8"], 2), sg(cb["C8"] - cb["C2"], 2), rp((cb["C8"] - cb["C2"]) / cb["C2"], 3), f"+/-{bd['C2']+bd['C8']:.1f}"),
    ("PSA carbon", "C(7) vs C(8) + C(12) + C(13)", f(cb["C7"], 2), f(cb["C8"] + cb["C12"] + cb["C13"], 2), sg(cb["res_psa"][0], 2), rp(cb["res_psa"][1], 3), f"+/-{bd['C7']+bd['C8']:.1f}"),
    ("Tail gas + NG fuel to stack", "C(8) + C(3) vs C(9)", f(cb["C8"] + cb["C3"], 2), f(cb["C9"], 2), sg(cb["res_fuel_tail"][0], 2), rp(cb["res_fuel_tail"][1], 3), f"+/-{bd['C8']+bd['C3']+bd['C9']:.1f}"),
    ("Natural gas to stack CO2", "C(stream 1) vs C(stream 9)", f(cb["C1"], 2), f(cb["C9"], 2), sg(cb["res_stack"][0], 2), rp(cb["res_stack"][1], 3), f"+/-{bd['stack_chain']:.1f}"),
]
table(["Balance", "Equation", "In, kmol C/h", "Out, kmol C/h", "Residual", "Relative", "Rounding band"], rows)
w(f"The full natural-gas-to-stack carbon balance closes to {rp(cb['res_stack'][1],3)}, inside the worst-case rounding band. Carbon leaving elsewhere is bounded: H2 product at most {R['c_leaks']['h2_spec_bound']:.3f} kmol/h (10 ppmv CO + CO2, Annex Table 2), "
  f"condensate and vent at most {R['c_leaks']['ko_resid']:.2f} kmol/h, flue CO at most {R['c_leaks']['co_flue']:.2f} kmol/h. So there is no carbon outlet other than stream 9 at the level of 0.1 %.")
w("")
sp_ = R["spec"]
rows = [
    ("Stack CO2, t/h", f"n9 y_CO2 M_CO2 / 1000 = {S[9]['n']} x {S[9]['x']['CO2']} x {MW['CO2']:.3f} / 1000", f"{sp_['stack_t_h']:.3f}", f"published 0.8091 x 100,000 / 1000 = {sp_['pub_t_h']:.2f}"),
    ("Specific CO2, kg/Nm3 H2", f"{sp_['stack_t_h']:.3f} x 1000 / 100,000", f"{sp_['per_nm3']:.5f}", f"published 0.8091; residual {sg(sp_['res_nm3'][0],5)} ({rp(sp_['res_nm3'][1],3)}); rounding of y_CO2 alone is +/-{pct(sp_['rounding_stack'],3)}"),
    ("Specific CO2, kg/kg H2", f"{sp_['stack_t_h']:.3f} x 1000 / 8994", f"{sp_['per_kg_h2']:.4f}", f"from published 0.8091: {sp_['pub_kg_h2']:.4f}; owner 8.996"),
    ("Specific CO2 from carbon in, kg/Nm3", f"C(stream 1) x 44.009 / 1000 x 1000 / 100,000", f"{sp_['per_nm3_from_Cin']:.5f}", f"vs published {rp(sp_['res_nm3_Cin'][1],3)}; per kg H2 {sp_['per_kg_h2_from_Cin']:.4f}"),
]
table(["Quantity", "Equation and substitution", "Value (calculated)", "Comparison"], rows)
w("### 7.8 Energy and Table 2 internal consistency (published inputs)")
w("")
en = R["en"]
rows = [
    ("Total energy input A", "30.563 x 46.50 / 3.6", f"{en['e_in_calc']:.3f} MW", f"published 394.77 ({sg(en['res_in'][0],3)} MW)"),
    ("Energy in product B", "8.994 x 119.96 / 3.6", f"{en['e_out_calc']:.3f} MW", f"published 299.70 ({sg(en['res_out'][0],4)} MW)"),
    ("Net electricity", "11.500 - 1.216 - 0.366", f"{en['elec_calc']:.3f} MWe", f"published 9.918 ({sg(en['res_el'][0],3)})"),
    ("Specific NG feedstock", "26.231 x 46.50 x 1000 / 100,000", f"{en['sc_feed_calc']:.3f} MJ/Nm3", "published 12.197"),
    ("Specific NG fuel", "4.332 x 46.50 x 1000 / 100,000", f"{en['sc_fuel_calc']:.3f} MJ/Nm3", "published 2.014"),
]
table(["Quantity", "Equation", "Calculated", "Published"], rows)
cp = R["cap"]
w(f"Capture-rate definitions in Table 2 are consistent with 0.8091 being the unabated stack total: Case 1A captured / (captured + emitted) = 0.4660 / (0.4660 + 0.3704) = {pct(cp['1A']['rate'],1)} (published 55.7 %), "
  f"avoided = 1 - 0.3704 / 0.8091 = {pct(cp['1A']['avoided'],1)} (published 54.2 %); Case 2A: {pct(cp['2A']['rate'],1)} (published 54.1 %), {pct(cp['2A']['avoided'],1)} (published 52.2 %).")
w("")

# ------------------------------------------------------------ section 8: Q6
w("## 8. Does the published 0.8091 kg/Nm3 H2 include the tail-gas combustion and the process CO2?")
w("")
w("**Yes, all of it.** The Base Case has no capture and one carbon outlet (flue gas, stream 9). Evidence in order of strength:")
w("")
w(f"- Stream 9 carries {f(f9['CO2'],1)} kmol/h CO2 = {sp_['stack_t_h']:.3f} t/h = {sp_['per_nm3']:.5f} kg/Nm3 H2, which reproduces the Table 2 value ({rp(sp_['res_nm3'][1],3)}). The flue gas is what is left after burning the tail gas and the supplementary NG, so the figure includes both.")
w(f"- The carbon chain closes from NG to stack ({rp(cb['res_stack'][1],3)}), with the tail gas (stream 8) accounting for {pct(split['tail']['frac'],1)} and the supplementary NG (stream 3) for {pct(split['ng']['frac'],1)} of the carbon reaching the stack (calculated). No carbon is credited anywhere else.")
w("- The report states (executive summary, PDF p6) that 'all of the CO2 is emitted from the flue gas of the steam reformer' and that it is produced during reforming and shift and during the combustion of the residual CO in the PSA tail gas and of the supplementary natural gas. Tail gas is 'the primary fuel' of the burners (same page; Sheet 14, PDF p37).")
w(f"- Table 2 treats 'Specific CO2 Emissions' as the stack emission (the Case 1A and 2A capture rates reproduce from it, section 7.8). The Base Case value equals the stack CO2 within {abs(sp_['res_nm3'][1])*100:.4f} %, so no credit for the 9.918 MWe exported and no indirect term is included.")
c2 = R["case2A_check"]
w(f"- Corroboration from Case 2A, which captures CO2 from the same PSA tail gas: the Case 2A equipment list (PDF p152, sheet 4 of 5) gives a tail-gas compressor flow of 47,204 Nm3/h, MW 28.8 and 28 C, against {f(R['psa']['tail_nm3h'],0)} Nm3/h ({rp(c2['comp_rel'],3)}) and {R['psa']['tail_mw']:.2f} kg/kmol from Base Case stream 8 (calculated), so the tail gas is the same and the 22.414 Nm3/kmol convention holds. "
  f"The Case 2A text (PDF p136, Sheet 50 of 112) says the tail gas holds about 51 mol % CO2 and the washed gas 3.5 mol %, which removes about {pct(c2['removed_frac'],1)} of its CO2 (calculated, wet basis, indicative). Table 2 gives 0.4556 kg/Nm3 captured in Case 2A, which is {pct(c2['ratio'],1)} of the pass-through CO2 of Base Case stream 8 ({c2['tail_co2_per_nm3']:.4f} kg/Nm3, calculated). The tail-gas capture plant therefore removes CO2 that the Base Case counts inside the 0.8091.")
w("")
w("Split of the stack CO2 (calculated from streams 8, 3 and 9). Parts are computed from carbon in the streams; the last column rescales them by "
  f"{R['scaled']['factor']:.5f} so that they add to exactly the published 0.8091 kg/Nm3.")
w("")
sl = R["split"]
sc_ = R["scaled"]
rows = [
    ("A1. Process CO2 already in the tail gas (leaves unburnt through the stack)", f"n8 y_CO2 = {f(f8['CO2'],1)} kmol/h", f"{sl['tail']['parts']['CO2_asis']:.3f}", f"{sl['tail']['parts_nm3']['CO2_asis']:.4f}", f"{sl['tail']['parts_kg']['CO2_asis']:.3f}", f"{sc_['tail_co2']:.4f}"),
    ("A2. CO in the tail gas, burnt to CO2", f"n8 y_CO = {f(f8['CO'],1)} kmol/h", f"{sl['tail']['parts']['CO_burnt']:.3f}", f"{sl['tail']['parts_nm3']['CO_burnt']:.4f}", f"{sl['tail']['parts_kg']['CO_burnt']:.3f}", f"{sc_['tail_co']:.4f}"),
    ("A3. CH4 in the tail gas, burnt to CO2", f"n8 y_CH4 = {f(f8['CH4'],1)} kmol/h", f"{sl['tail']['parts']['CH4_burnt']:.3f}", f"{sl['tail']['parts_nm3']['CH4_burnt']:.4f}", f"{sl['tail']['parts_kg']['CH4_burnt']:.3f}", f"{sc_['tail_ch4']:.4f}"),
    ("**A. Feedstock carbon via the tail gas (A1 + A2 + A3)**", f"{f(cb['C8'],1)} kmol C/h", f"**{sl['tail']['total']:.3f}**", f"**{sl['tail']['per_nm3']:.4f}**", f"**{sl['tail']['per_kg']:.3f}**", f"**{sc_['tail']:.4f}**"),
    ("B1. CO2 already in the supplementary NG (2 vol %)", f"n3 y_CO2 = {f(f3['CO2'],1)} kmol/h", f"{sl['ng']['parts']['CO2_asis']:.3f}", f"{sl['ng']['parts_nm3']['CO2_asis']:.4f}", f"{sl['ng']['parts_kg']['CO2_asis']:.3f}", f"{sc_['ng_co2']:.4f}"),
    ("B2. Hydrocarbon carbon of the supplementary NG, burnt", f"{f(cb['C3']-f3['CO2'],1)} kmol C/h", f"{sl['ng']['parts']['HC_burnt']:.3f}", f"{sl['ng']['parts_nm3']['HC_burnt']:.4f}", f"{sl['ng']['parts_kg']['HC_burnt']:.3f}", f"{sc_['ng_hc']:.4f}"),
    ("**B. Supplementary NG fuel (B1 + B2)**", f"{f(cb['C3'],1)} kmol C/h", f"**{sl['ng']['total']:.3f}**", f"**{sl['ng']['per_nm3']:.4f}**", f"**{sl['ng']['per_kg']:.3f}**", f"**{sc_['ng']:.4f}**"),
    ("**Sum A + B**", "", f"**{sl['sum']:.3f}**", f"**{sl['sum']*1000/PUB['h2_nm3h']:.4f}**", f"**{sl['sum']*1000/S[12]['m']:.3f}**", f"**{PUB['co2_spec']:.4f}**"),
    ("Stack CO2 from stream 9 (check)", f"{f(f9['CO2'],1)} kmol/h", f"{sl['stack']:.3f}", f"{sp_['per_nm3']:.4f}", f"{sp_['per_kg_h2']:.3f}", f"A + B exceeds the stack value by {sl['res'][1]*100:.3f} %"),
]
table(["Component of the stack CO2", "Basis", "t CO2/h", "kg CO2/Nm3 H2", "kg CO2/kg H2", "Rescaled to 0.8091 (kg/Nm3)"], rows)
fc = sl["feed_C_in_NG_feedstock"]
w(f"Feedstock carbon in stream 2 is {f(fc['kmol'],1)} kmol C/h ({fc['t_co2eq']:.2f} t CO2 equivalent), which equals the carbon in stream 8 ({f(cb['C8'],1)}) within {rp((cb['C8']-cb['C2'])/cb['C2'],3)}; "
  f"{f(fc['co2_in_feed_kmol'],1)} kmol/h of it is CO2 already present in the NG feed (2 vol %).")
w("")
w(f"**Rule for later model work, so nothing is counted twice.** The stack CO2 of the Base Case is A + B = 0.8091 kg/Nm3 H2 (8.996 kg/kg H2). Row A ({sc_['tail']:.3f} kg/Nm3, rescaled) is the tail-gas term and already contains its own combustion. "
  "If a model burns the tail gas explicitly, it must take its carbon from the tail-gas composition (A1 + A2 + A3) and add only the supplementary NG (B) on top; it must not add a tail-gas combustion term to the published 0.8091. "
  "If a model starts from 0.8091, only changes relative to the Base Case (a different tail-gas composition, a different supplementary NG flow) may be added. "
  f"A capture or abatement model that acts on the tail gas can remove at most A ({sc_['tail']:.3f} kg/Nm3, {pct(sl['tail']['frac'],1)} of the stack CO2); one that acts on the flue gas sees all of A + B.")
w("")

# ------------------------------------------------------------ section 9
w("## 9. Not published or not resolved")
w("")
w("- Tail-gas LHV, combustion-air flow and composition, furnace thermal efficiency, convective-section duties, and a single stated PSA recovery for the Base Case: unavailable. The first two are derived above with stated assumptions; the other three cannot be derived from published data.")
w("- The sheet does not say whether the pressures are gauge or absolute, and the executive summary footnote quotes 0.2 MPa for the tail gas while stream 8 gives 0.13 MPa.")
w(f"- Stream 14 (HP steam to process, 95,301 kg/h) is about {f((S[14]['m']-wt['steam_total_kg'])/1000,1)} t/h larger than the reformer steam implied by the H and O balances (section 7.2).")
w("- The text quotes residual CO of 2.5 to 3.5 % dry at the HTS outlet; the stream table gives 4.65 % dry (section 7.1).")
w("- The owner's 'furnace fuel 4.332 t/h' is the supplementary natural gas only. Total furnace fuel is 4332 kg/h NG plus 60,658 kg/h tail gas.")
w("- Nm3 is not defined in the pages read; 22.414 Nm3/kmol (0 C, 101.325 kPa) reproduces 100,000 Nm3/h from stream 12 to 6e-7 relative, so it is used.")
w("")

with open(OUT_MD, "w", encoding="utf-8", newline="\n") as fh:
    fh.write("\n".join(L))

if __name__ == "__main__":
    print("wrote", OUT_MD, len(L), "lines")
    print("stack CO2 t/h %.4f  kg/Nm3 %.5f  kg/kg %.4f" % (spec["stack_t_h"], spec["per_nm3"], spec["per_kg_h2"]))
    print("C chain: in %.2f out %.2f res %.2f (%.4f %%), band %.2f" % (C1, C9, C9 - C1, 100 * (C9 - C1) / C1, R["band"]["stack_chain"]))
    print("split t/h: tail %.3f  NG %.3f  sum %.3f  stack %.3f" % (split["tail"]["total"], split["ng"]["total"], split["sum"], split["stack"]))
    print("tail frac %.4f  ng frac %.4f" % (split["tail"]["frac"], split["ng"]["frac"]))
    print("recovery %.4f  tail LHV MJ/kg %.3f MJ/Nm3 %.3f  Q_tail %.2f MW" % (R["psa"]["recovery"], R["tail"]["lhv_mj_kg"], R["tail"]["lhv_mj_nm3"], q_tail))
    print("water net t/h %.2f (util 44.2); w2 H %.2f O %.2f; y_close %.4f" % (R["water"]["net_th"], w2_H, w2_O, R["furn"]["y_close"]))
    print("overall:", {k: tuple(round(x, 3) for x in v) for k, v in R["overall"].items()})
