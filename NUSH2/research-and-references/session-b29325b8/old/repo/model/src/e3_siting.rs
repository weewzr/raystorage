//! E3 Jurong candidate-context infrastructure and cooling screens.
//! These are siting-screen quantities, not a site approval or cooling-system design.
use crate::e2b_maturation::{REACTOR_MWTH,TRAIN_HEAT_MWTH,TRAIN_H2_T_Y,TRAIN_AVOIDED_T_Y,AVAIL};
use crate::final_design::{NG_FINAL_MMSCFD,CAPTURED_SHORT_T_D,EMITTED_SHORT_T_D};

pub const TRAINS:f64=2.0;
pub const USEFUL_PROCESS_HEAT_MWTH:f64=TRAIN_HEAT_MWTH*TRAINS;
pub const FULL_POWER_RESIDUAL_ENVELOPE_MWTH:f64=REACTOR_MWTH-USEFUL_PROCESS_HEAT_MWTH;
pub const SEAWATER_CP_KJ_KG_K:f64=3.99;
pub const SEAWATER_DENSITY_KG_M3:f64=1025.0;
pub const EMA_REFERENCE_SEAWATER_C:f64=29.2;
pub const EMA_REFERENCE_DELTA_T_K:f64=8.0;

fn short_t_to_t(x:f64)->f64{x*0.90718474}
pub fn ng_mmscfd()->f64{NG_FINAL_MMSCFD*TRAINS}
pub fn ng_bscf_y()->f64{ng_mmscfd()*365.0*AVAIL/1000.0}
pub fn h2_mmscfd()->f64{130.0*TRAINS}
pub fn h2_t_y()->f64{TRAIN_H2_T_Y*TRAINS}
pub fn captured_co2_t_d_operating()->f64{short_t_to_t(CAPTURED_SHORT_T_D)*TRAINS}
pub fn captured_co2_t_y()->f64{captured_co2_t_d_operating()*365.0*AVAIL}
pub fn emitted_co2_t_y()->f64{short_t_to_t(EMITTED_SHORT_T_D)*TRAINS*365.0*AVAIL}
pub fn lifecycle_avoided_t_y()->f64{TRAIN_AVOIDED_T_Y*TRAINS}
pub fn cooling_mass_flow_kg_s(q_mw:f64,delta_t_k:f64)->f64{q_mw*1000.0/(SEAWATER_CP_KJ_KG_K*delta_t_k)}
pub fn cooling_volume_flow_m3_s(q_mw:f64,delta_t_k:f64)->f64{cooling_mass_flow_kg_s(q_mw,delta_t_k)/SEAWATER_DENSITY_KG_M3}
pub fn cooling_csv()->String{
 let mut s=String::from("boundary,Q_MWth,deltaT_K,mass_flow_kg_s,volume_flow_m3_s,inlet_C,outlet_C,status\n");
 for (name,q) in [("full_power_residual_disposition_envelope",FULL_POWER_RESIDUAL_ENVELOPE_MWTH),("full_600MWth_ultimate_sink_envelope",REACTOR_MWTH)]{
  for dt in [5.0,8.0,10.0]{
   s.push_str(&format!("{name},{q:.3},{dt:.3},{:.3},{:.6},{:.3},{:.3},ILLUSTRATIVE_NOT_PROJECT_COOLING_DUTY\n",cooling_mass_flow_kg_s(q,dt),cooling_volume_flow_m3_s(q,dt),EMA_REFERENCE_SEAWATER_C,EMA_REFERENCE_SEAWATER_C+dt));
  }
 }
 s
}
pub fn infrastructure_csv()->String{
 format!("item,value,unit,classification\nreactor_installed_thermal,{:.3},MWth,E2B architecture\ntwo_train_useful_process_heat,{:.3},MWth,E2B architecture\nfull_power_residual_capacity,{:.3},MWth,NOT cooling duty\nnatural_gas,{:.3},MMSCFD,source-scaled operating flow\nnatural_gas_annual,{:.3},Bscf/y,derived at 85% availability\nhydrogen,{:.3},MMSCFD,source-scaled operating flow\nhydrogen_annual,{:.3},t/y,derived at 85% availability\ncaptured_CO2_operating,{:.3},t/day,source-scaled\ncaptured_CO2_annual,{:.3},t/y,derived at 85% availability\nemitted_CO2_annual,{:.3},t/y,derived at 85% availability\nlifecycle_abatement,{:.3},tCO2e/y,E2B scaling basis\n",
 REACTOR_MWTH,USEFUL_PROCESS_HEAT_MWTH,FULL_POWER_RESIDUAL_ENVELOPE_MWTH,ng_mmscfd(),ng_bscf_y(),h2_mmscfd(),h2_t_y(),captured_co2_t_d_operating(),captured_co2_t_y(),emitted_co2_t_y(),lifecycle_avoided_t_y())
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn two_train_heat_closes(){assert!((USEFUL_PROCESS_HEAT_MWTH-353.6).abs()<1e-9);}
 #[test]fn residual_is_capacity_not_duty(){assert!((FULL_POWER_RESIDUAL_ENVELOPE_MWTH-246.4).abs()<1e-9);}
 #[test]fn two_train_flows_scale_exactly(){assert!((ng_mmscfd()-68.0).abs()<1e-9);assert!((h2_mmscfd()-260.0).abs()<1e-9);}
 #[test]fn captured_co2_scale(){assert!((captured_co2_t_y()-1_084_724.0).abs()<2.0);}
 #[test]fn ema_reference_flow_is_reproducible(){assert!((cooling_volume_flow_m3_s(246.4,8.0)-7.5310).abs()<0.001);}
 #[test]fn lower_delta_t_requires_more_water(){assert!(cooling_volume_flow_m3_s(246.4,5.0)>cooling_volume_flow_m3_s(246.4,10.0));}
 #[test]fn e2b_abatement_preserved(){assert!((lifecycle_avoided_t_y()-1_834_277.8).abs()<1.0);}
}
