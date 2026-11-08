import sys
from mutate import run
MUT2 = {
 "S1_solve_at0_le": ("engine/plant/src/subsidy.rs", "if at0 < target {", "if at0 <= target {"),
 "S2_solve_atmax_gt": ("engine/plant/src/subsidy.rs", "if at_max >= target {", "if at_max > target {"),
 "S3_bisect_le": ("engine/plant/src/subsidy.rs", "if lac(mid)? < target {", "if lac(mid)? <= target {"),
 "R1_private_capital": ("engine/plant/src/subsidy.rs", "private_capital: total_capital - grant,", "private_capital: total_capital,"),
 "R2_ledger_state_pv_close": ("engine/plant/src/subsidy.rs", "(c_r * (h - (1.0 - u)), grant * (1.0 + rate).powf(tc))", "(c_r * (h - (1.0 - u)), grant)"),
 "R3_export_swap_interest_cols": ("engine/plant/src/export.rs", "by(&|x| x.interest_private_sm),\n        by(&|x| x.interest_saved_sm),", "by(&|x| x.interest_saved_sm),\n        by(&|x| x.interest_private_sm),"),
 "R4_outlay_wrong_factor": ("engine/plant/src/subsidy.rs", "public_outlay_s_per_t: plan.ledger_state_pv * outlay,", "public_outlay_s_per_t: plan.ledger_state_pv * per_t,"),
 "R5_export_private_col": ("engine/plant/src/export.rs", "num(p.private_capital),", "num(p.grant),"),
 "R7_dcf_interest_private_sign": ("engine/plant/src/subsidy.rs", "let dcf_interest_private = dcf_private_reactor_pv - (c_r - grant_reactor);", "let dcf_interest_private = dcf_private_reactor_pv + (c_r - grant_reactor);"),
 "R8_progressive_year0_dropped": ("engine/plant/src/subsidy.rs", "Timing::Progressive => (s * c_r, s * other_eligible),", "Timing::Progressive => (s * c_r, 0.0),"),
 "R9_licence_uses_power_island": ("engine/plant/src/subsidy.rs", "(el.licence, k.licence_capital),", "(el.licence, k.power_island_capital),"),
}
for n in sys.argv[1:]:
    run(n, *MUT2[n])
