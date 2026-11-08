use nuclear_assisted_smr::final_design::{final_design,HEAT_MWTH,REACTOR_MWTH,NISHIHARA_IHX_MWTH,PROCESS_HEAT_C,REFORMER_C,INL_ROT_C};
fn main(){
 let x=final_design(0.0,false);
 println!("metric,value,unit,class");
 println!("reactor_rating,{:.3},MWth,source-backed",REACTOR_MWTH);
 println!("source_heat_branch,{:.3},MWth,source-backed",NISHIHARA_IHX_MWTH);
 println!("process_heat,{:.3},MWth,source-backed",HEAT_MWTH);
 println!("remaining_thermal_capacity,{:.3},MWth,project-derived-capacity-only",REACTOR_MWTH-HEAT_MWTH);
 println!("reactor_outlet,{:.3},C,source-backed",INL_ROT_C);
 println!("process_heat_supply,{:.3},C,source-backed",PROCESS_HEAT_C);
 println!("reformer_outlet,{:.3},C,source-backed",REFORMER_C);
 println!("lifecycle_avoided,{:.3},tCO2e/y,verified-model",x.lifecycle_avoided_t);
 println!("abatement_threshold,250000.000,tCO2e/y,assignment");
 println!("abatement_multiple,{:.6},ratio,project-derived",x.lifecycle_avoided_t/250000.0);
 println!("abatement_cost,{:.6},SGD/tCO2e,verified-model",x.abatement_cost_sgd_t);
 println!("cost_ceiling,100.000,SGD/tCO2e,assignment");
 println!("cost_headroom,{:.6},SGD/tCO2e,project-derived",100.0-x.abatement_cost_sgd_t);
}