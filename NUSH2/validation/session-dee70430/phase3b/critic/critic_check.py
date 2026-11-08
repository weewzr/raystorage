"""Independent re-computation by the completeness critic. Run with python -I.
Inputs are typed from the PDF text or page images read by the critic (page cited), not copied from spec_check.py.
"""
import math

out = {}

# ---- V1 stack CO2, stream 9 (PDF p44): 8659.4 kmol/h, y(CO2)=0.2123, 257,698 kg/h
n9, y9, m9 = 8659.4, 0.2123, 257698.0
MW = 44.0095
co2_t_h = n9 * y9 * MW / 1000
out["V1_stack_CO2_t_h"] = co2_t_h
out["V1_kg_per_Nm3"] = co2_t_h * 1000 / 100000
out["V1_kg_per_kg"] = co2_t_h * 1000 / 8994
out["V1_rel_to_0.8091"] = co2_t_h * 1000 / 100000 / 0.8091 - 1
# molar mass check of stream 9 from composition
M = 0.2123*44.0095 + 0.6083*28.0134 + 0.0102*31.998 + 0.1692*18.0153
out["V1_M_from_comp_vs_m_over_n"] = (M, m9 / n9, M / (m9 / n9) - 1)

# ---- V2 PSA recovery (p44, p45): n7 6596.9, yH2 .7563, n12 4461.5, n13 29.1
rec = (4461.5 + 29.1) / (6596.9 * 0.7563)
out["V2_recovery"] = rec
out["V2_range_high_0.90_exceeded_by"] = rec - 0.90
# PSA mass closure
out["V2_mass_closure"] = 69711 - 60658 - 8994 - 59

# ---- V3 tail gas LHV with engine constants
n8 = 2106.3
y8 = dict(H2=.2369, CO=.1454, CH4=.0945)
lhv = dict(H2=241.8, CO=283.0, CH4=802.3)
q_tail = sum(n8 * y8[k] * lhv[k] for k in y8) / 3600
out["V3_Q_tail_MW"] = q_tail
out["V3_Q_fired_MW"] = q_tail + 4.332 * 46.50 / 3.6

# ---- V4 Table 2 energy and power
out["V4_energy_in_MW"] = 30.563 * 46.50 / 3.6
out["V4_energy_H2_MW"] = 8.994 * 119.96 / 3.6
out["V4_net_power_utility_sheet_kW"] = 11500 - 1216 - 19 - 347
out["V4_revenue_EUR"] = 9.918 * 8322 * 80
out["V4_hours_fraction"] = 8322 / 8760

# ---- V5 TPC and island
tpc = 97.212 + 20.124 + 53.616
out["V5_tpc_sum"] = tpc
tcr_pub = 222.887
fx = 2.322
out["V5_K_I_from_222.887"] = tcr_pub * fx
out["V5_K_I_from_registry_222.89"] = 222.89 * fx
crf = 0.08 * 1.08**25 / (1.08**25 - 1)
a_net = 557767.4
out["V5_LAC_island_f1_registry"] = 222.89 * fx * crf * 1e6 / a_net
out["V5_LAC_island_f0.5_registry"] = 0.5 * 222.89 * fx * crf * 1e6 / a_net
pi_tcr = 20.124 / tpc * tcr_pub
out["V5_pi_TCR_MEUR"] = pi_tcr
out["V5_pi_LAC_S_t"] = pi_tcr * fx * crf * 1e6 / a_net

# ---- V6 gas energy conversion and Case B what-if
hhv = 51.473
E_hhv = 30.563 * hhv
gj_mmscf = 1044 * 1.05505585
n_x1 = E_hhv * 24 / gj_mmscf
x = 100000 * 0.08988 / 8994
out["V6_x"] = x
out["V6_n_ref_x1"] = n_x1
out["V6_n_ref_B"] = n_x1 * x
hours = 0.85 * 8760
days = 0.85 * 365
k_gas = gj_mmscf * days * 15 / 1e6
out["V6_k_gas"] = k_gas
n_plant = 35.0923 - 61.4798 / k_gas
out["V6_n_plant_implied"] = n_plant
out["V6_GAS_after"] = -(n_x1 * x - n_plant) * k_gas
base_power = 9.918 * x * hours * 0.2 / 1e3
out["V6_BASE_POWER_after"] = base_power
price_mmbtu = 1.05505585 * hours * 15 / 1e6
out["V6_steam_price_SM_per_MMBtu_h"] = price_mmbtu
out["V6_old_steam_plant_implied"] = 189.2 * 0.683281 - 15.2339 / price_mmbtu
# dA_net from baseline change: baseline moves -24.9 t/y
a_new = a_net - 24.91
d_l5 = ((-(n_x1 * x - n_plant) * k_gas) - (-61.4798) + (base_power - (-6.4105)) + (0 - 15.2339))
out["V6_dL5_S_t_with_old_Anet"] = d_l5 * 1e6 / a_net
out["V6_L5_after_old_Anet"] = 203.180 + d_l5 * 1e6 / a_net
out["V6_L5_after_new_Anet"] = (203.180 * a_net / 1e6 + d_l5) * 1e6 / a_new
out["V6_power_at_0.18576"] = 9.918 * x * hours * 0.18576 / 1e3
out["V6_power_at_0.186"] = 9.918 * x * hours * 0.186 / 1e3

# ---- V7 INL gas HHV per scf (flowsheet.rs NG_MOL_FRACTIONS) and engine MMSCFD currency
hhv_kj_mol = dict(CH4=890.3, C2H6=1560.7, C3H8=2219.2, nC4=2877.5, nC5=3535.6, N2=0, CO2=0)
inl = dict(CH4=.93571, C2H6=.03749, C3H8=.00920, nC4=.00260, nC5=.00040 + 1.2 * .00010, N2=.0120, CO2=.00250)
hh = sum(inl[k] * hhv_kj_mol[k] for k in inl)
mol_per_scf = 453.59237 / 379.49
out["V7_INL_HHV_btu_scf"] = hh * mol_per_scf / 1.05505585
iea = dict(CH4=.89, C2H6=.07, C3H8=.01, nC4=.001, nC5=.0001, N2=.0089, CO2=.02)
hi = sum(iea[k] * hhv_kj_mol[k] for k in iea)
out["V7_IEAGHG_HHV_from_comp_btu_scf"] = hi * mol_per_scf / 1.05505585
out["V7_IEAGHG_HHV_from_comp_MJ_kmol"] = hi
out["V7_IEAGHG_HHV_published_MJ_kmol"] = 51.473 * 18.019

# ---- V8 LCOH bridge
lcoh_nm3 = 0.1144
per_kg = lcoh_nm3 / 0.08994
sk = per_kg * fx
out["V8_report_LCOH_S_kg"] = sk
fuel_c = 8.527
fuel_s_kg = fuel_c / 100 / 0.08994 * fx
out["V8_fuel_S_kg"] = fuel_s_kg
uplift = 15 / (6 * fx / (51.473 / 46.502)) - 1
out["V8_gas_uplift_frac"] = uplift
out["V8_gas_term"] = fuel_s_kg * uplift
out["V8_avail_term_on_cap_plus_fom"] = (2.716 + 0.928) / 100 / 0.08994 * fx * (95 / 85 - 1)
elec_c = 0.793
elec_s = elec_c / 100 / 0.08994 * fx
out["V8_elec_credit_at_report_price"] = elec_s
out["V8_elec_credit_rebase_0.20"] = elec_s * (0.2 / (80 * fx / 1000)) - elec_s
out["V8_bridge_sum"] = sk + fuel_s_kg * uplift + (2.716 + 0.928) / 100 / 0.08994 * fx * (95 / 85 - 1) + (elec_s * (0.2 / (80 * fx / 1000)) - elec_s) * -1
out["V8_3.618_vs_bridge_rel"] = 3.618 / 3.472 - 1

# ---- V9 Case 2A capture vs A1
a1_nm3 = n8 * .5095 * MW / 1000 * 1000 / 100000
out["V9_A1_kg_Nm3_raw"] = a1_nm3
out["V9_2A_captured_over_A1_raw"] = 0.4556 / a1_nm3
out["V9_A_raw"] = (n8 * (.5095 + .1454 + .0945) * MW / 1000) * 1000 / 100000
out["V9_2A_captured_over_A_raw"] = 0.4556 / out["V9_A_raw"]
out["V9_A1_share_of_stack"] = a1_nm3 / 0.8091

# ---- V10 gap shares and kink share
out["V10_gap_over_net"] = 0.0687 / 2.914
out["V10_gap_over_gross"] = 0.0687 / 178.5
out["V10_kink"] = 742.767 / 1063.188
out["V10_row_count_A_B"] = 4 * 2 * 2 * 2 * (4 * 2 + 3)
out["V10_row_count_C"] = 4 * 2 * 1 * 2 * (21 * 2 + 3)

# ---- V11 baseline t/y
h2_t_y = 8.988 * hours
out["V11_baseline_old"] = 0.8091 / 0.08994 * h2_t_y
out["V11_baseline_new"] = co2_t_h * 1000 / 8994 * h2_t_y
out["V11_delta"] = out["V11_baseline_new"] - out["V11_baseline_old"]

# ---- V12 repo S14/S15 LHV from committed mass balance (FS_B_REC_F), kmol/h
S14 = dict(H2=607.990735, CO=21.316376, CH4=234.506353)
S15 = dict(H2=47.648378, CO=5.282666, CH4=58.115818)
S03 = dict(H2=0.0, CO=16.033711, CH4=176.390535)
S21 = dict(H2=560.342358, CO=0.0, CH4=0.0)
q = lambda d: sum(d[k] * lhv[k] for k in d) / 3600
out["V12_LHV_S14_MW"] = q(S14)
out["V12_LHV_S15_MW"] = q(S15)
out["V12_S03_S21_S15_minus_S14_MW"] = q(S03) + q(S21) + q(S15) - q(S14)

for k, v in out.items():
    print(k, v)
