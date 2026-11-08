use nuclear_assisted_smr::e2_attribution::{comparator_csv,provenance_csv,attribution,case_a,case_b,case_c};
use std::fs;
fn main(){
 let dir="../results/e2_attribution";fs::create_dir_all(dir).unwrap();
 fs::write(format!("{dir}/e2_comparator.csv"),comparator_csv()).unwrap();
 fs::write(format!("{dir}/e2_input_provenance.csv"),provenance_csv()).unwrap();
 let x=attribution();let a=case_a();let b=case_b();let c=case_c();
 let s=format!("metric,value,unit,interpretation\nA_to_B_lifecycle_abatement,{:.3},tCO2e/y,CCS comparator step on IEAGHG matched source family\nB_to_C_bundled_lifecycle_difference,{:.3},tCO2e/y,cross-source difference NOT pure nuclear attribution\nA_to_C_cross_source_total,{:.3},tCO2e/y,cross-source E2 matched-service total\nA_to_B_share_of_cross_source_total,{:.6},fraction,descriptive attribution\nB_to_C_bundled_share_of_cross_source_total,{:.6},fraction,NOT pure nuclear share\nA_to_B_incremental_cost,{:.3},SGD/y,screening matched comparator\nB_to_C_bundled_cost_difference,{:.3},SGD/y,NOT pure nuclear premium\nA_to_B_step_cost,{:.6},SGD/tCO2e,CCS step\nB_to_C_bundled_step_cost,{:.6},SGD/tCO2e,NOT pure nuclear step metric\nA_lifecycle,{:.3},tCO2e/y,\nB_lifecycle,{:.3},tCO2e/y,\nC_lifecycle,{:.3},tCO2e/y,\n",
 x.a_to_b_abatement,x.b_to_c_abatement,x.a_to_c_abatement,x.ccs_share,x.nuclear_share,
 x.a_to_b_cost,x.b_to_c_cost,x.ccs_step_cost,x.nuclear_step_cost,a.lifecycle_t,b.lifecycle_t,c.lifecycle_t);
 fs::write(format!("{dir}/e2_attribution.csv"),s).unwrap();
}
