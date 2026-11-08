//! Post-E8 bounded architecture-sizing re-optimization.
//! Evidence-supported Case-6 train calculations are kept separate from topology sensitivities.
use crate::e2b_maturation::{scenario,TRAIN_HEAT_MWTH,TRAIN_H2_T_Y,TRAIN_AVOIDED_T_Y,SOURCE_MAX_PROCESS_HEAT_MWTH,REACTOR_MWTH};
use crate::e1_economics::ihx_loop_annual_sgd_unscaled;

#[derive(Clone,Copy,Debug)]
pub struct Topology {
 pub name:&'static str,pub trains:f64,pub heat_per_train:f64,pub total_heat:f64,
 pub branch_util:f64,pub reactor_util:f64,pub h2_t_y:f64,pub avoided_t_y:f64,
 pub evidence:&'static str,pub economics:&'static str
}
pub fn topology(name:&'static str,trains:f64,heat_per_train:f64,evidence:&'static str,economics:&'static str)->Topology{
 let total=trains*heat_per_train;
 Topology{name,trains,heat_per_train,total_heat:total,branch_util:total/SOURCE_MAX_PROCESS_HEAT_MWTH,
 reactor_util:total/REACTOR_MWTH,h2_t_y:TRAIN_H2_T_Y*total/TRAIN_HEAT_MWTH,
 avoided_t_y:TRAIN_AVOIDED_T_Y*total/TRAIN_HEAT_MWTH,evidence,economics}
}
pub fn candidates()->Vec<Topology>{
 vec![
  topology("benchmark_1x_case6",1.0,176.8,"SOURCE-ANCHORED","E2B stage economics calculable"),
  topology("preferred_2x_case6",2.0,176.8,"SOURCE-ANCHORED + PROJECT REPLICATION","E2B stage economics calculable"),
  topology("sensitivity_3x_smaller",3.0,353.6/3.0,"BOUNDED TOPOLOGY SENSITIVITY","equipment/cost/reliability scaling unresolved"),
  topology("sensitivity_1x_custom_large",1.0,353.6,"BOUNDED TOPOLOGY SENSITIVITY","scale-up/equipment economics unresolved"),
 ]
}
pub fn integration_cost_sensitivity_sgd_t(loop_count:f64)->f64{
 let mature=scenario(2,"10-OAK");
 let delta=(loop_count-2.0)*ihx_loop_annual_sgd_unscaled();
 mature.cost_sgd_t+delta/mature.avoided_t_y
}
pub fn csv()->String{
 let mut s=String::from("candidate,trains,heat_per_train_MWth,total_heat_MWth,branch_utilization,reactor_utilization,H2_t_y,avoided_tCO2e_y,evidence,economics\n");
 for x in candidates(){s.push_str(&format!("{},{:.3},{:.6},{:.6},{:.6},{:.6},{:.3},{:.3},{},{}\n",x.name,x.trains,x.heat_per_train,x.total_heat,x.branch_util,x.reactor_util,x.h2_t_y,x.avoided_t_y,x.evidence,x.economics));} s
}
pub fn economics_sensitivity_csv()->String{
 let base=scenario(2,"10-OAK");
 format!("case,represented_IHX_secondary_loop_count,mature_cost_SGD_t,status\nvalidated_E2B_2train,2,{:.6},CANONICAL\ncustom_large_if_one_loop,1,{:.6},BOUNDED_SENSITIVITY_NOT_VALIDATED\nthree_small_if_three_loops,3,{:.6},BOUNDED_SENSITIVITY_NOT_VALIDATED\n",base.cost_sgd_t,integration_cost_sensitivity_sgd_t(1.0),integration_cost_sensitivity_sgd_t(3.0))
}
#[cfg(test)] mod tests{use super::*;
 #[test]fn benchmark_matches_e2b(){let x=&candidates()[1];assert!((x.total_heat-353.6).abs()<1e-9);assert!((x.branch_util-353.6/370.0).abs()<1e-12);}
 #[test]fn integer_rule_is_feasibility_not_global_optimum(){assert_eq!((SOURCE_MAX_PROCESS_HEAT_MWTH/TRAIN_HEAT_MWTH).floor() as u32,2);assert_eq!(candidates().len(),4);}
 #[test]fn sensitivities_preserve_same_total_service(){for x in &candidates()[1..]{assert!((x.total_heat-353.6).abs()<1e-8);}}
 #[test]fn topology_cost_credit_is_not_canonical(){assert!(integration_cost_sensitivity_sgd_t(1.0)<scenario(2,"10-OAK").cost_sgd_t);assert!(integration_cost_sensitivity_sgd_t(3.0)>scenario(2,"10-OAK").cost_sgd_t);}
}
