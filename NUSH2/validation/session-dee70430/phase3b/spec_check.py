"""Lead-engineer independent check of the IEAGHG 2017-02 Base Case numbers used in spec.md.

Run with: python -I spec_check.py
Every value printed here is either published (copied from the report tables, with the page in the
spec) or calculated from published values with the equation shown in the label.
Constants that are assumed are listed at the top.
"""
import json

# ---- assumed constants (repository engine/props/src/units.rs where stated) ----
MW_CO2 = 44.0095        # kg/kmol, repo units.rs
LHV = {"H2": 241.8, "CO": 283.0, "CH4": 802.3}   # kJ/mol, repo units.rs
KCAL = 4.1868           # kJ/kcal, assumed
NM3_KMOL = 22.414       # Nm3/kmol, inferred from stream 12 (4461.5 x 22.414 = 100,000)
MMBTU_GJ = 1.05505585   # repo
NG_HHV_BTU_SCF = 1044.0 # repo parameters.toml NG_HHV
SCF_LBMOL = 379.49      # repo
LB = 0.45359237         # repo
AM = {"C": 12.011, "H": 1.008, "O": 15.999, "N": 14.007}

# ---- published: stream table PDF p44, p45 ----
sp = ["CO2", "CO", "H2", "N2", "O2", "CH4", "C2H6", "C3H8", "nC4H10", "nC5H12", "H2O"]
C_ATOMS = {"CO2": 1, "CO": 1, "CH4": 1, "C2H6": 2, "C3H8": 3, "nC4H10": 4, "nC5H12": 5}
ng = {"CO2": .02, "N2": .0089, "CH4": .89, "C2H6": .07, "C3H8": .01, "nC4H10": .001, "nC5H12": .0001}
S = {
    1: dict(n=1696.6, m=30563, y=ng),
    2: dict(n=1455.8, m=26231, y=ng),
    3: dict(n=240.4, m=4332, y=ng),
    7: dict(n=6596.9, m=69711, y={"CO2": .1627, "CO": .0464, "H2": .7563, "N2": .0020, "CH4": .0302, "H2O": .0024}),
    8: dict(n=2106.3, m=60658, y={"CO2": .5095, "CO": .1454, "H2": .2369, "N2": .0062, "CH4": .0945, "H2O": .0076}),
    9: dict(n=8659.4, m=257698, y={"CO2": .2123, "N2": .6083, "O2": .0102, "H2O": .1692}),
    12: dict(n=4461.5, m=8994, y={"H2": 1.0}),
    13: dict(n=29.1, m=59, y={"H2": 1.0}),
}
out = {}

def carbon(sid):
    s = S[sid]
    return sum(s["n"] * y * C_ATOMS.get(k, 0) for k, y in s["y"].items())

# 1. Table 2 consistency
A = 30.563 * 46.50 / 3.6
B = 8.994 * 119.96 / 3.6
out["energy_in_MW"] = A
out["energy_H2_MW"] = B
out["net_power_MWe"] = 11.500 - 1.216 - 0.366
out["eta_H2_LHV"] = B / A
out["H2_nm3_h_from_stream12"] = 4461.5 * NM3_KMOL

# 2. stack CO2 and intensity with three CO2 molar masses
for m in (44.009, 44.0095, 44.01):
    co2 = S[9]["n"] * S[9]["y"]["CO2"] * m / 1000.0   # t/h
    out[f"stack_CO2_t_h_M{m}"] = co2
    out[f"stack_CO2_kg_nm3_M{m}"] = co2 * 1000 / 100000
    out[f"stack_CO2_kg_kg_M{m}"] = co2 * 1000 / 8994
out["published_kg_kg"] = 0.8091 * 100000 / 8994
co2_repo = S[9]["n"] * S[9]["y"]["CO2"] * MW_CO2 / 1000.0
out["stack_vs_published_rel"] = (co2_repo * 1000 / 100000) / 0.8091 - 1

# 3. carbon balances
c1, c2, c3, c7, c8 = carbon(1), carbon(2), carbon(3), carbon(7), carbon(8)
c9 = S[9]["n"] * S[9]["y"]["CO2"]
out["C_kmol_h"] = dict(s1=c1, s2=c2, s3=c3, s7=c7, s8=c8, s9=c9)
out["C_res_NG_to_stack_rel"] = c9 / c1 - 1
out["C_res_tail_plus_fuel_to_stack_rel"] = c9 / (c8 + c3) - 1
out["C_res_feed_to_tail_rel"] = c8 / c2 - 1
out["C_per_kmol_NG"] = c1 / S[1]["n"]
out["M_NG_from_comp"] = (ng["CO2"] * (AM["C"] + 2 * AM["O"]) + ng["N2"] * 2 * AM["N"] + ng["CH4"] * (AM["C"] + 4 * AM["H"])
                         + ng["C2H6"] * (2 * AM["C"] + 6 * AM["H"]) + ng["C3H8"] * (3 * AM["C"] + 8 * AM["H"])
                         + ng["nC4H10"] * (4 * AM["C"] + 10 * AM["H"]) + ng["nC5H12"] * (5 * AM["C"] + 12 * AM["H"]))

# 4. PSA component balance (impurities all rejected, H2 split by recovery)
f7, f8 = S[7], S[8]
rec = (S[12]["n"] + S[13]["n"]) / (f7["n"] * f7["y"]["H2"])
rec_net = S[12]["n"] / (f7["n"] * f7["y"]["H2"])
psa = {}
for k in ("CO2", "CO", "H2", "N2", "CH4", "H2O"):
    feed = f7["n"] * f7["y"][k]
    tail_pub = f8["n"] * f8["y"][k]
    tail_model = feed * (1 - rec) if k == "H2" else feed
    psa[k] = dict(feed=feed, tail_published=tail_pub, tail_model=tail_model, res=tail_model - tail_pub)
out["PSA"] = dict(recovery_incl_recycle=rec, recovery_net=rec_net, species=psa,
                  tail_total_model=sum(v["tail_model"] for v in psa.values()), tail_total_published=f8["n"])
out["PSA_mass_closure_kg_h"] = f7["m"] - f8["m"] - S[12]["m"] - S[13]["m"]

# 5. tail gas LHV, burner duty
q_tail = {k: f8["n"] * f8["y"][k] * LHV[k] / 3600.0 for k in LHV}   # kmol/h x kJ/mol = MJ/h... /3600 = MW
q_tail_tot = sum(q_tail.values())
q_ng = 4.332 * 46.50 / 3.6
q_feed_ng = 26.231 * 46.50 / 3.6
q_fired = q_tail_tot + q_ng
out["Q_tail_MW"] = dict(**q_tail, total=q_tail_tot)
out["tail_LHV"] = dict(MJ_kmol=q_tail_tot * 3600 / f8["n"], MJ_kg=q_tail_tot * 3600 / f8["m"],
                       MJ_nm3=q_tail_tot * 3600 / (f8["n"] * NM3_KMOL))
out["Q_NG_fuel_MW"] = q_ng
out["Q_NG_feed_MW"] = q_feed_ng
out["Q_fired_MW"] = q_fired
out["tail_share_of_fired"] = q_tail_tot / q_fired
out["fired_share_of_plant_input"] = q_fired / A
out["radiant_MW"] = 82.63e6 * KCAL / 3.6e6
out["radiant_over_fired"] = out["radiant_MW"] / q_fired
out["utility_fuel_MW"] = 48.2e6 * KCAL / 3.6e6

# 6. CO2 split A1 A2 A3 B1 B2
def tco2(kmol):
    return kmol * MW_CO2 / 1000.0
a1, a2, a3 = tco2(f8["n"] * f8["y"]["CO2"]), tco2(f8["n"] * f8["y"]["CO"]), tco2(f8["n"] * f8["y"]["CH4"])
b1 = tco2(S[3]["n"] * ng["CO2"])
b2 = tco2(c3 - S[3]["n"] * ng["CO2"])
A_t, B_t = a1 + a2 + a3, b1 + b2
sumAB = A_t + B_t
resc = co2_repo / sumAB
out["split_t_h"] = dict(A1=a1, A2=a2, A3=a3, A=A_t, B1=b1, B2=b2, B=B_t, sum=sumAB, stack=co2_repo, rescale=resc)
out["split_kg_nm3_rescaled"] = {k: v * resc * 1000 / 100000 for k, v in dict(A1=a1, A2=a2, A3=a3, A=A_t, B1=b1, B2=b2, B=B_t).items()}
out["split_share"] = dict(A=A_t / sumAB, B=B_t / sumAB)

# 7. gas energy and repo conversion
hhv, lhv_ng = 51.473, 46.502
E_hhv_gj_h = 30.563 * hhv
gj_per_mmscf = NG_HHV_BTU_SCF * MMBTU_GJ
n_energy = E_hhv_gj_h * 24 / gj_per_mmscf
kmol_per_mmscfd = 1e6 / SCF_LBMOL / 24 * LB
n_volume = 1696.6 / kmol_per_mmscfd
mj_scf = hhv * (18.019 * LB) / SCF_LBMOL   # MJ per scf of IEAGHG NG, HHV
out["gas"] = dict(HHV_over_LHV=hhv / lhv_ng, E_HHV_GJ_h=E_hhv_gj_h, E_HHV_MW=E_hhv_gj_h / 3.6,
                  mmscfd_by_energy_x1=n_energy, mmscfd_by_volume_x1=n_volume,
                  HHV_btu_per_scf_IEAGHG=mj_scf * 1e6 / 1055.05585)

# 8. what-if at Case B (repo values quoted from results tables, labelled)
H2_B_KG_H = 100000 * 0.08988
x_ref = H2_B_KG_H / 8994
hours = 0.85 * 8760
days = 0.85 * 365
k_gas = gj_per_mmscf * days * 15 / 1e6                # S$M per MMSCFD-y at S$15/GJ
n_ref_engine = 35.092267
n_ref_new = n_energy * x_ref
dgas = (n_ref_engine - n_ref_new) * k_gas              # S$M/y, GAS line becomes less negative
a_net = 557767.415516
power_new = 9.918 * x_ref * hours * 0.2 * 1e3 / 1e6    # BASE_POWER new, positive = cost to NUS-H2
power_old = -6.3 * 0.683281 * hours * 0.2 * 1e3 / 1e6
steam_old = 15.2339
out["whatif_CaseB"] = dict(x_ref=x_ref, hours=hours, k_gas_SM_per_MMSCFD_y=k_gas, n_ref_new=n_ref_new,
                           dGAS_SM_y=dgas, dGAS_S_t=dgas * 1e6 / a_net,
                           BASE_POWER_new_SM_y=power_new, BASE_POWER_old_SM_y=power_old,
                           LOST_STEAM_new_SM_y=0.0, LOST_STEAM_old_SM_y=steam_old,
                           d_sys_lines_SM_y=(power_new + 0.0) - (power_old + steam_old),
                           d_sys_lines_S_t=((power_new + 0.0) - (power_old + steam_old)) * 1e6 / a_net,
                           d_L5_total_S_t=(dgas + (power_new - power_old - steam_old)) * 1e6 / a_net,
                           old_L5=203.180)
e_new = co2_repo * 1000 / 8994
out["intensity_new_kg_kg"] = e_new
out["intensity_old_kg_kg"] = 0.8091 / 0.08994
h2_t_y = H2_B_KG_H * hours / 1000
out["baseline_t_y"] = dict(old=0.8091 / 0.08994 * h2_t_y, new=e_new * h2_t_y, rel=e_new / (0.8091 / 0.08994) - 1)
grid_t_y = 9.918 * x_ref * hours * 0.402
out["K18_grid_t_y_reference_export"] = grid_t_y
out["K18_grid_t_y_old_purchase"] = 6.3 * 0.683281 * hours * 0.402

# 9. steam and power
out["steam_power"] = dict(total_HP_t_h=46.053 + 95.301, export_share=46.053 / (46.053 + 95.301),
                          kwh_per_t_stream10=11500 / 46.053, kwh_per_t_utility=11500 / 45.9,
                          internal_MWe=1.216 + 0.366, internal_share_gross=(1.216 + 0.366) / 11.5,
                          net_kWh_per_1000nm3=9918 / 100.0, net_kWh_per_kg=9918 / 8994.0 * 1.0 * 1.0,
                          export_revenue_EUR_y=9.918 * 8322 * 80, export_revenue_S_y_at_0p186=9.918 * 8322 * 80 * 2.322)

# 10. island numbers
tcr, tpc, pi_tpc = 222.886722, 170.952, 20.124
crf25 = 0.08 * 1.08 ** 25 / (1.08 ** 25 - 1)
pi_tcr_prorata = pi_tpc / tpc * tcr
out["island"] = dict(TCR_SM=tcr * 2.322, power_island_TCR_prorata_MEUR=pi_tcr_prorata,
                     power_island_TCR_prorata_SM=pi_tcr_prorata * 2.322,
                     LAC_effect_S_t_if_NUSH2_excludes=pi_tcr_prorata * 2.322 * crf25 * 1e6 / a_net, crf25=crf25)
print(json.dumps(out, indent=1, default=float))
