//! E2B forward architecture and FOAK-to-NOAK maturation model.
//! Scenario choices are evidence-led; threshold comparison is performed after construction.
use crate::deployment::{crf, USD_SGD_2026_09_29};
use crate::e1_economics::{e1_case, NuclearCase, ihx_loop_annual_sgd_unscaled};
use crate::final_design::final_design;

pub const TRAIN_HEAT_MWTH:f64=176.8;
pub const REACTOR_MWTH:f64=600.0;
pub const SOURCE_MAX_PROCESS_HEAT_MWTH:f64=370.0;
pub const TRAIN_H2_T_Y:f64=97_946.036;
pub const TRAIN_BASELINE_SGD_Y:f64=269_115_246.004;
pub const TRAIN_CANDIDATE_NG_SGD_Y:f64=174_284_159.317;
pub const TRAIN_CCS_SGD_Y:f64=10_682_759.887;
pub const TRAIN_TS_SGD_Y:f64=8_135_429.766;
pub const TRAIN_AVOIDED_T_Y:f64=917_138.9;
pub const AVAIL:f64=0.85;
pub const HOURS_Y:f64=7446.0;

// INL/RPT-23-72972 (2019 USD), medium BOAK values and its non-electric HTGR method:
// OCC=6000 $/kWe; OPEX=25 $/MWh; remove 9% energy-conversion contribution;
// convert to thermal basis at 40% HTGR efficiency; medium FOAK premium=1.6;
// medium learning rate=10%; OPEX held constant through learning in the example.
pub const INL_BOAK_MEDIUM_OCC_USD_KWE:f64=6000.0;
pub const INL_BOAK_MEDIUM_OPEX_USD_MWHE:f64=25.0;
pub const INL_ENERGY_CONVERSION_SHARE:f64=0.09;
pub const INL_HTGR_EFFICIENCY:f64=0.40;
pub const INL_FOAK_PREMIUM_MEDIUM:f64=1.6;
pub const INL_LEARNING_RATE_MEDIUM:f64=0.10;
pub const US_GDP_DEFLATOR_2019_QAVG:f64=(103.328+103.862+104.192+104.516)/4.0;
pub const US_GDP_DEFLATOR_2025:f64=128.893;

pub fn usd2019_to_sgd2025(x:f64)->f64{x*(US_GDP_DEFLATOR_2025/US_GDP_DEFLATOR_2019_QAVG)*USD_SGD_2026_09_29}
pub fn heat_only_boak_occ_usd_kwt()->f64{INL_BOAK_MEDIUM_OCC_USD_KWE*(1.0-INL_ENERGY_CONVERSION_SHARE)*INL_HTGR_EFFICIENCY}
pub fn heat_only_opex_usd_mwth()->f64{INL_BOAK_MEDIUM_OPEX_USD_MWHE*INL_HTGR_EFFICIENCY}
pub fn learning_factor(n:f64)->f64{(1.0-INL_LEARNING_RATE_MEDIUM).powf(n.log2())}
pub fn heat_only_occ_usd_kwt(stage:&str)->f64{
 match stage{
  "FOAK"=>heat_only_boak_occ_usd_kwt()*INL_FOAK_PREMIUM_MEDIUM,
  "BOAK"=>heat_only_boak_occ_usd_kwt(),
  "10-OAK"=>heat_only_boak_occ_usd_kwt()*learning_factor(10.0),
  _=>panic!("unsupported stage")
 }
}
pub fn nuclear_annual_sgd(stage:&str)->f64{
 let cap=usd2019_to_sgd2025(heat_only_occ_usd_kwt(stage)*REACTOR_MWTH*1000.0);
 let om=usd2019_to_sgd2025(heat_only_opex_usd_mwth()*REACTOR_MWTH*HOURS_Y);
 cap*crf(0.08,25)+om
}
#[derive(Clone,Copy,Debug)]
pub struct Scenario{pub trains:u32,pub installed_mwth:f64,pub useful_heat_mwth:f64,pub utilization:f64,
 pub h2_t_y:f64,pub avoided_t_y:f64,pub nuclear_sgd_y:f64,pub integration_sgd_y:f64,
 pub baseline_sgd_y:f64,pub candidate_sgd_y:f64,pub net_sgd_y:f64,pub cost_sgd_t:f64,
 pub margin_sgd_y:f64,pub abatement_pass:bool,pub cost_pass:bool}
pub fn scenario(trains:u32,stage:&str)->Scenario{
 let n=trains as f64;let heat=TRAIN_HEAT_MWTH*n;
 assert!(heat<=SOURCE_MAX_PROCESS_HEAT_MWTH+1e-9,"architecture exceeds source process-heat branch");
 let nuclear=nuclear_annual_sgd(stage);
 // One TECDOC-1682 IHX+secondary-loop source anchor per process train is a
 // conservative represented integration treatment. Other project-specific BOP remains unresolved.
 let integration=ihx_loop_annual_sgd_unscaled()*n;
 let baseline=TRAIN_BASELINE_SGD_Y*n;
 let candidate=(TRAIN_CANDIDATE_NG_SGD_Y+TRAIN_CCS_SGD_Y+TRAIN_TS_SGD_Y)*n+nuclear+integration;
 let net=candidate-baseline;let avoided=TRAIN_AVOIDED_T_Y*n;let cost=net/avoided;
 Scenario{trains,installed_mwth:REACTOR_MWTH,useful_heat_mwth:heat,utilization:heat/REACTOR_MWTH,
 h2_t_y:TRAIN_H2_T_Y*n,avoided_t_y:avoided,nuclear_sgd_y:nuclear,integration_sgd_y:integration,
 baseline_sgd_y:baseline,candidate_sgd_y:candidate,net_sgd_y:net,cost_sgd_t:cost,
 margin_sgd_y:100.0*avoided-net,abatement_pass:avoided>250_000.0,cost_pass:cost<100.0}
}
pub fn e1_gap_sgd_y()->f64{
 let x=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);
 x.net_incremental_sgd_y-100.0*x.avoided_t_y
}
pub fn e1_max_nuclear_annual_sgd()->f64{
 let x=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);
 x.nuclear_sgd_y-e1_gap_sgd_y()
}
pub fn e1_break_even_capex_sgd()->f64{
 // Hold E1 one-module O&M fixed and solve annual burden = threshold.
 let def=US_GDP_DEFLATOR_2025/110.159;
 let om=324e6*def*USD_SGD_2026_09_29/4.0;
 (e1_max_nuclear_annual_sgd()-om)/crf(0.08,25)
}
pub fn e1_break_even_om_sgd_y()->f64{
 let def=US_GDP_DEFLATOR_2025/110.159;
 let cap=2748e6*def*USD_SGD_2026_09_29/4.0;
 e1_max_nuclear_annual_sgd()-cap*crf(0.08,25)
}
pub fn required_real_coproduct_cost_bearing_sgd_y()->f64{e1_gap_sgd_y().max(0.0)}
pub fn scenarios_csv()->String{
 let mut s=String::from("scenario,stage,trains,installed_MWth,useful_heat_MWth,thermal_utilization,H2_t_y,avoided_t_y,nuclear_SGD_y,integration_SGD_y,baseline_SGD_y,candidate_SGD_y,net_SGD_y,cost_SGD_t,margin_to_100_SGD_y,abatement_pass,cost_pass,evidence_maturity\n");
 for (name,stage,n) in [("FOAK two-train","FOAK",2),("early-commercial BOAK two-train","BOAK",2),("preferred mature 10-OAK two-train","10-OAK",2),("FOAK one-train","FOAK",1),("10-OAK one-train","10-OAK",1)]{
  let x=scenario(n,stage);
  s.push_str(&format!("{name},{stage},{},{:.3},{:.3},{:.6},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.6},{:.3},{},{},{}\n",
   x.trains,x.installed_mwth,x.useful_heat_mwth,x.utilization,x.h2_t_y,x.avoided_t_y,x.nuclear_sgd_y,x.integration_sgd_y,x.baseline_sgd_y,x.candidate_sgd_y,x.net_sgd_y,x.cost_sgd_t,x.margin_sgd_y,x.abatement_pass,x.cost_pass,
   if stage=="FOAK"{"MODELLED"}else if stage=="BOAK"{"MODELLED"}else{"PROJECTED"}));
 }
 s
}
pub fn thresholds_csv()->String{
 let e1=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);
 format!("metric,value,unit\nE1_net_incremental,{:.3},SGD/y\nE1_max_at_100,{:.3},SGD/y\nE1_gap,{:.3},SGD/y\nE1_required_reduction_fraction_of_nuclear,{:.6},fraction\nE1_break_even_nuclear_annual,{:.3},SGD/y\nE1_break_even_CAPEX_at_fixed_OM,{:.3},SGD\nE1_break_even_OM_at_fixed_CAPEX,{:.3},SGD/y\nrequired_real_coproduct_cost_bearing,{:.3},SGD/y\n",
 e1.net_incremental_sgd_y,100.0*e1.avoided_t_y,e1_gap_sgd_y(),e1_gap_sgd_y()/e1.nuclear_sgd_y,e1_max_nuclear_annual_sgd(),e1_break_even_capex_sgd(),e1_break_even_om_sgd_y(),required_real_coproduct_cost_bearing_sgd_y())
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn e1_gap_is_preserved(){assert!((e1_gap_sgd_y()-49_563_924.0).abs()<2_000.0);}
 #[test]fn three_trains_are_rejected_by_source_heat_branch(){assert!(TRAIN_HEAT_MWTH*3.0>SOURCE_MAX_PROCESS_HEAT_MWTH);}
 #[test]fn two_trains_close_heat_branch(){assert!(TRAIN_HEAT_MWTH*2.0<=SOURCE_MAX_PROCESS_HEAT_MWTH);}
 #[test]fn preferred_is_defined_by_source_method_and_learning(){assert!((heat_only_occ_usd_kwt("10-OAK")-1539.04).abs()<0.1);}
 #[test]fn foak_two_train_fails_but_boak_and_tenoak_pass(){assert!(!scenario(2,"FOAK").cost_pass);assert!(scenario(2,"BOAK").cost_pass);assert!(scenario(2,"10-OAK").cost_pass);}
 #[test]fn preferred_passes_both_thresholds(){let x=scenario(2,"10-OAK");assert!(x.abatement_pass&&x.cost_pass);}
 #[test]fn one_train_does_not_gain_unsupported_credit(){assert!(!scenario(1,"10-OAK").cost_pass);}
 #[test]fn no_coproduct_credit_is_embedded(){let x=scenario(2,"10-OAK");assert!(x.candidate_sgd_y>0.0);assert!(required_real_coproduct_cost_bearing_sgd_y()>0.0);}
 #[test]fn emissions_scale_only_with_real_h2_trains(){assert!((scenario(2,"10-OAK").avoided_t_y-2.0*final_design(0.0,false).lifecycle_avoided_t).abs()<1.0);}
}
