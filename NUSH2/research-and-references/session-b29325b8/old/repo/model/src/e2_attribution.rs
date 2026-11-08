//! E2 matched-service comparator and incremental nuclear-benefit attribution.
use crate::{IEAGHG_ENERGY_BASE,IEAGHG_ENERGY_CASE_1A,IEAGHG_KG_H2_PER_NM3};
use crate::deployment::{ccs_capex_sgd,crf,JAEA_AVAIL};
use crate::final_design::{final_design, H2_LB_H, CAPTURED_SHORT_T_D, EMITTED_SHORT_T_D, NG_FINAL_MMSCFD, NG_HHV_BTU_SCF, PROCESS_ELECTRIC_MWE};
use crate::e1_economics::{e1_case,NuclearCase};

pub const UPSTREAM_NG_KG_CO2E_GJ:f64=11.5;
pub const TS_LCA_T_PER_T:f64=0.025;
pub const NUCLEAR_G_KWH_E:f64=5.5;
pub const NUCLEAR_THERMAL_ALLOC:f64=0.504;
pub const COMMON_ELEC_MWE:f64=6.3;

fn short_t_to_t(x:f64)->f64{x*0.90718474}
fn ng_gj_day(mmscfd:f64)->f64{mmscfd*1e6*NG_HHV_BTU_SCF*1.05505585262e-6}
pub fn common_h2_kg_h()->f64{H2_LB_H*0.45359237}
pub fn common_h2_nm3_h_ieaghg_equivalent()->f64{common_h2_kg_h()/IEAGHG_KG_H2_PER_NM3}
pub fn annual_h2_t()->f64{common_h2_kg_h()*JAEA_AVAIL*8760.0/1000.0}
fn ieaghg_direct_t_y(kg_nm3:f64)->f64{kg_nm3*common_h2_nm3_h_ieaghg_equivalent()*JAEA_AVAIL*8760.0/1000.0}
fn ieaghg_ng_upstream_t_y(mw:f64)->f64{mw*JAEA_AVAIL*8760.0*3.6*UPSTREAM_NG_KG_CO2E_GJ/1000.0}
fn ieaghg_captured_t_y(kg_nm3:f64)->f64{kg_nm3*common_h2_nm3_h_ieaghg_equivalent()*JAEA_AVAIL*8760.0/1000.0}

#[derive(Clone,Copy,Debug)]
pub struct ComparatorCase{
 pub direct_t:f64,pub upstream_t:f64,pub nuclear_t:f64,pub aux_t:f64,pub ts_lca_t:f64,
 pub lifecycle_t:f64,pub represented_cost_sgd_y:f64,pub ng_source_mw:f64,pub captured_t:f64
}
pub fn case_a()->ComparatorCase{
 let direct=ieaghg_direct_t_y(IEAGHG_ENERGY_BASE.co2_kg_per_nm3_h2);
 let upstream=ieaghg_ng_upstream_t_y(IEAGHG_ENERGY_BASE.ng_input_mw_lhv);
 ComparatorCase{direct_t:direct,upstream_t:upstream,nuclear_t:0.0,aux_t:0.0,ts_lca_t:0.0,
 lifecycle_t:direct+upstream,represented_cost_sgd_y:e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0).baseline_sgd_y,
 ng_source_mw:IEAGHG_ENERGY_BASE.ng_input_mw_lhv,captured_t:0.0}
}
pub fn case_b()->ComparatorCase{
 let direct=ieaghg_direct_t_y(IEAGHG_ENERGY_CASE_1A.co2_kg_per_nm3_h2);
 let upstream=ieaghg_ng_upstream_t_y(IEAGHG_ENERGY_CASE_1A.ng_input_mw_lhv);
 let captured=ieaghg_captured_t_y(0.4660);
 let ts=TS_LCA_T_PER_T*captured;
 // Strongest current cost treatment: IEAGHG Case-1A non-T&S screening CAC
 // normalized in economics/singapore_case1a_screen_v1.md (S$77.7/t direct avoided),
 // plus the E1 S$15/t captured T&S assumption. This avoids charging Case B the
 // nuclear-assisted lower NG flow.
 let a=case_a();
 let direct_avoided=a.direct_t-direct;
 let incremental_cost=77.7*direct_avoided+15.0*captured;
 ComparatorCase{direct_t:direct,upstream_t:upstream,nuclear_t:0.0,aux_t:0.0,ts_lca_t:ts,
 lifecycle_t:direct+upstream+ts,represented_cost_sgd_y:a.represented_cost_sgd_y+incremental_cost,
 ng_source_mw:IEAGHG_ENERGY_CASE_1A.ng_input_mw_lhv,captured_t:captured}
}
pub fn case_c()->ComparatorCase{
 let hours=JAEA_AVAIL*8760.0;
 let direct=short_t_to_t(EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let upstream=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*UPSTREAM_NG_KG_CO2E_GJ/1000.0;
 let nuclear=176.8*1000.0*hours*(NUCLEAR_G_KWH_E*NUCLEAR_THERMAL_ALLOC)/1e6;
 let aux=(PROCESS_ELECTRIC_MWE-COMMON_ELEC_MWE).max(0.0)*1000.0*hours*NUCLEAR_G_KWH_E/1e6;
 let captured=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let ts=TS_LCA_T_PER_T*captured;
 let econ=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);
 ComparatorCase{direct_t:direct,upstream_t:upstream,nuclear_t:nuclear,aux_t:aux,ts_lca_t:ts,
 lifecycle_t:direct+upstream+nuclear+aux+ts,represented_cost_sgd_y:econ.candidate_sgd_y,
 ng_source_mw:ng_gj_day(NG_FINAL_MMSCFD)/86400.0*1000.0,captured_t:captured}
}
#[derive(Clone,Copy,Debug)]
pub struct Attribution{
 pub a_to_b_abatement:f64,pub b_to_c_abatement:f64,pub a_to_c_abatement:f64,
 pub ccs_share:f64,pub nuclear_share:f64,pub a_to_b_cost:f64,pub b_to_c_cost:f64,
 pub ccs_step_cost:f64,pub nuclear_step_cost:f64
}
pub fn attribution()->Attribution{
 let a=case_a();let b=case_b();let c=case_c();
 let ab=a.lifecycle_t-b.lifecycle_t;let bc=b.lifecycle_t-c.lifecycle_t;let ac=a.lifecycle_t-c.lifecycle_t;
 Attribution{a_to_b_abatement:ab,b_to_c_abatement:bc,a_to_c_abatement:ac,
 ccs_share:ab/ac,nuclear_share:bc/ac,a_to_b_cost:b.represented_cost_sgd_y-a.represented_cost_sgd_y,
 b_to_c_cost:c.represented_cost_sgd_y-b.represented_cost_sgd_y,
 ccs_step_cost:(b.represented_cost_sgd_y-a.represented_cost_sgd_y)/ab,
 nuclear_step_cost:(c.represented_cost_sgd_y-b.represented_cost_sgd_y)/bc}
}
pub fn comparator_csv()->String{
 let a=case_a();let b=case_b();let c=case_c();let x=attribution();
 format!("metric,unabated_smr,smr_ccs,nuclear_smr_ccs,a_to_b,b_to_c\nH2_t_y,{0:.3},{0:.3},{0:.3},0,0\nNG_source_energy_MW_mixed_basis,{1:.3},{2:.3},{3:.3},{4:.3},{5:.3}\nDirect_CO2_t_y,{6:.3},{7:.3},{8:.3},{9:.3},{10:.3}\nLifecycle_CO2e_t_y,{11:.3},{12:.3},{13:.3},{14:.3},{15:.3}\nRepresented_cost_SGD_y,{16:.3},{17:.3},{18:.3},{19:.3},{20:.3}\nIncremental_step_SGD_t,,{21:.6},{22:.6},,\n",
 annual_h2_t(),a.ng_source_mw,b.ng_source_mw,c.ng_source_mw,b.ng_source_mw-a.ng_source_mw,c.ng_source_mw-b.ng_source_mw,
 a.direct_t,b.direct_t,c.direct_t,a.direct_t-b.direct_t,b.direct_t-c.direct_t,
 a.lifecycle_t,b.lifecycle_t,c.lifecycle_t,x.a_to_b_abatement,x.b_to_c_abatement,
 a.represented_cost_sgd_y,b.represented_cost_sgd_y,c.represented_cost_sgd_y,x.a_to_b_cost,x.b_to_c_cost,
 x.ccs_step_cost,x.nuclear_step_cost)
}
pub fn provenance_csv()->String{String::from("case,input,value,unit,provenance,normalization,limitation\nA,IEAGHG H2 source scale,100000,Nm3/h,IEAGHG 2017-02,normalize by 8.994 t/h per 100000 Nm3/h to project 29000 lb/h,IEAGHG and INL source families differ\nA,NG input,394.77,MW LHV,IEAGHG base case,same H2 normalization,source conventional process\nA,direct CO2,0.8091,kg/Nm3 H2,IEAGHG base case,same H2 normalization,plant gate\nB,NG input,407.68,MW LHV,IEAGHG Case 1A,same H2 normalization,includes CCS energy penalty\nB,direct CO2,0.3704,kg/Nm3 H2,IEAGHG Case 1A,same H2 normalization,plant gate\nB,captured CO2,0.4660,kg/Nm3 H2,IEAGHG Case 1A,same H2 normalization,55.7 percent overall capture topology differs from Case C\nB,non-TS avoidance cost,77.7,SGD/t direct avoided,economics/singapore_case1a_screen_v1.md,CEPCI/FX screening harmonization,not lifecycle-denominator cost\nC,NG,34.0,MMSCFD,INL Case 6,current project source service,cross-source matching limitation\nC,direct/captured CO2,142/1927,short ton/day,INL Case 6,current project annualization,cross-source matching limitation\nC,economics,E1 central,SGD/y,results/E1_ENGINEERING_ECONOMIC_MODEL.md,no change,E1 one-module MHR-T case\n")}
#[cfg(test)]mod tests{use super::*;
 #[test]fn common_h2_service_matches_project(){assert!((annual_h2_t()-final_design(0.0,false).annual_h2_t).abs()<1e-8);}
 #[test]fn attribution_closes(){let x=attribution();assert!((x.a_to_b_abatement+x.b_to_c_abatement-x.a_to_c_abatement).abs()<1e-8);}
 #[test]fn ccs_alone_clears_abatement_threshold(){assert!(attribution().a_to_b_abatement>250_000.0);}
 #[test]fn nuclear_adds_abatement_but_is_not_majority(){let x=attribution();assert!(x.b_to_c_abatement>0.0);assert!(x.ccs_share>x.nuclear_share);}
 #[test]fn e1_candidate_cost_is_unchanged(){let c=case_c();let e=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);assert!((c.represented_cost_sgd_y-e.candidate_sgd_y).abs()<1e-8);}
 #[test]fn no_capture_double_count(){let b=case_b();assert!((b.lifecycle_t-b.direct_t-b.upstream_t-b.ts_lca_t).abs()<1e-8);}
}
