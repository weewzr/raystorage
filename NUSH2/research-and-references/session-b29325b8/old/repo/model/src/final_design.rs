use crate::*;
use crate::deployment::{crf,ccs_capex_sgd,jaea_plant_capex_sgd,jaea_heat_sgd_gj,jaea_electric_sgd_mwh,JAEA_AVAIL};

pub const REFORMER_C:f64=871.0;
pub const PROCESS_HEAT_C:f64=900.0;
pub const INL_ROT_C:f64=925.0;
pub const PRESSURE_BAR:f64=31.7;
pub const STEAM_CARBON:f64=3.0;
pub const METHANE_CONVERSION:f64=0.781;
pub const PSA_RECOVERY:f64=0.88;
pub const H2_LB_H:f64=29_000.0;
pub const H2_MMSCFD:f64=130.0;
pub const NG_BASE_MMSCFD:f64=52.5;
pub const NG_FINAL_MMSCFD:f64=34.0;
pub const HEAT_MWTH:f64=176.8;
pub const PROCESS_ELECTRIC_MWE:f64=17.3;
pub const CAPTURED_SHORT_T_D:f64=1927.0;
pub const EMITTED_SHORT_T_D:f64=142.0;
pub const BASE_EMITTED_SHORT_T_D:f64=3205.0;

// Coherent Nishihara et al. (2007) GTHTR300C economic architecture.
pub const REACTOR_MWTH:f64=600.0;
pub const NISHIHARA_IHX_MWTH:f64=370.0;
pub const NISHIHARA_POWER_BRANCH_MWTH:f64=REACTOR_MWTH-NISHIHARA_IHX_MWTH;
pub const NISHIHARA_GROSS_MWE:f64=88.0;

// INL TEV-961 Case-6 process-side helium state.
pub const HE_SUPPLY_C:f64=900.0;
pub const HE_RETURN_C:f64=466.0;
pub const CP_HE_KJ_KG_K:f64=5.2; // screening average; reproduces source flow closely.

pub const NG_HHV_BTU_SCF:f64=1044.0;
pub const GAS_PRICE_SGD_GJ:f64=15.0;
pub const ELEC_VALUE_SGD_MWH:f64=150.0;
pub const T_AND_S_SGD_T:f64=15.0;

#[derive(Clone,Copy,Debug)]
pub struct FinalDesign {
 pub annual_h2_t:f64,pub captured_t_y:f64,pub emitted_t_y:f64,
 pub annual_avoided_t:f64,pub lifecycle_avoided_t:f64,pub lifecycle_ci_kgkg:f64,
 pub helium_flow_kg_s:f64,pub reactor_utilisation:f64,
 pub residual_thermal_capacity_mw:f64,pub source_gross_electric_mwe:f64,
 pub reactor_capex_sgd:f64,pub ccs_capex_sgd:f64,pub annual_incremental_sgd:f64,
 pub abatement_cost_sgd_t:f64,pub pass_abatement:bool,pub pass_cost:bool,pub joint_pass:bool
}
fn short_t_to_t(x:f64)->f64{x*0.90718474}
fn ng_gj_day(mmscfd:f64)->f64{mmscfd*1e6*NG_HHV_BTU_SCF*1.05505585262e-6}
pub fn final_design(electricity_value:f64,double_cost_sensitivity:bool)->FinalDesign{
 let hours=JAEA_AVAIL*8760.0;
 let annual_h2_t=H2_LB_H*0.45359237*hours/1000.0;
 let captured_t_y=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let emitted_t_y=short_t_to_t(EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let base_emitted=short_t_to_t(BASE_EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let direct_avoided=base_emitted-emitted_t_y;
 let upstream_base=ng_gj_day(NG_BASE_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let upstream_final=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let nuclear_lca=HEAT_MWTH*1000.0*hours*(5.5*0.504)/1e6;
 let aux_lca=(PROCESS_ELECTRIC_MWE-6.3).max(0.0)*1000.0*hours*5.5/1e6;
 let transport=0.025*captured_t_y;
 let lifecycle_avoided=direct_avoided+(upstream_base-upstream_final)-nuclear_lca-aux_lca-transport;
 // tCO2e/tH2 is numerically identical to kgCO2e/kgH2.
 let lifecycle_ci=(base_emitted+upstream_base-lifecycle_avoided)/annual_h2_t;

 // INL Case-6 helium state; constant-Cp screen is checked against the source 78.49 kg/s.
 let helium_flow=HEAT_MWTH*1000.0/(CP_HE_KJ_KG_K*(HE_SUPPLY_C-HE_RETURN_C));
 let util=HEAT_MWTH/REACTOR_MWTH;

 // Thermal capacity screen: the INL duty is below the source 370 MWth heat branch.
 // The remaining 423.2 MWth is capacity only; no off-design electric output is claimed.
 let residual_thermal_capacity=(REACTOR_MWTH-HEAT_MWTH).max(0.0);

 // Recover the full published Nishihara source-product economic burden.
 // This deliberately prices the source 370 MWth heat product plus source 88 MWe
 // generation and assigns ZERO project export/revenue because no source-backed
 // off-design power-cycle/internal-load model is available for the 176.8 MWth draw.
 // double_cost_sensitivity remains an adverse COST sensitivity only.
 let heat_burden=NISHIHARA_IHX_MWTH*hours*3.6*jaea_heat_sgd_gj(double_cost_sensitivity);
 let power_burden=NISHIHARA_GROSS_MWE*hours*jaea_electric_sgd_mwh(double_cost_sensitivity);
 let reactor_burden=heat_burden+power_burden;
 let power_value=0.0*electricity_value;
 let ccs_cap=ccs_capex_sgd(captured_t_y);
 let annual_ccs=ccs_cap*crf(0.08,25);
 let integration=0.10*(jaea_plant_capex_sgd(double_cost_sensitivity)+ccs_cap)*crf(0.03,40);
 let ts=captured_t_y*T_AND_S_SGD_T;
 let base_ng=ng_gj_day(NG_BASE_MMSCFD)*365.0*JAEA_AVAIL*GAS_PRICE_SGD_GJ;
 let final_ng=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*GAS_PRICE_SGD_GJ;
 let incremental=final_ng+reactor_burden+annual_ccs+integration+ts-power_value-base_ng;
 let cost=incremental/lifecycle_avoided;
 FinalDesign{annual_h2_t,captured_t_y,emitted_t_y,annual_avoided_t:direct_avoided,lifecycle_avoided_t:lifecycle_avoided,
 lifecycle_ci_kgkg:lifecycle_ci,helium_flow_kg_s:helium_flow,reactor_utilisation:util,
 residual_thermal_capacity_mw:residual_thermal_capacity,source_gross_electric_mwe:NISHIHARA_GROSS_MWE,
 reactor_capex_sgd:jaea_plant_capex_sgd(double_cost_sensitivity),
 ccs_capex_sgd:ccs_cap,annual_incremental_sgd:incremental,abatement_cost_sgd_t:cost,
 pass_abatement:lifecycle_avoided>250_000.0,pass_cost:cost<100.0,joint_pass:lifecycle_avoided>250_000.0&&cost<100.0}
}
#[derive(Clone,Copy,Debug)]
pub struct AvailabilitySensitivity {
 pub availability:f64,pub annual_h2_t:f64,pub lifecycle_avoided_t:f64,
 pub pass_abatement:bool
}
// Emissions/throughput-only screening sensitivity. The source operating point is
// held fixed while annual operating quantities scale linearly with effective
// annual process availability. Installed/fixed plant economics are deliberately
// NOT scaled with downtime because the repository has no defensible fixed-vs-
// variable decomposition for the already-built reactor/CCS/integration system.
// This is not a reliability/PRA model and exposes no economic pass/fail result.
pub fn availability_sensitivity(a:f64)->AvailabilitySensitivity{
 assert!(a>0.0&&a<=1.0);
 let hours=a*8760.0;
 let annual_h2_t=H2_LB_H*0.45359237*hours/1000.0;
 let captured=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*a;
 let candidate_direct=short_t_to_t(EMITTED_SHORT_T_D)*365.0*a;
 let baseline_direct=short_t_to_t(BASE_EMITTED_SHORT_T_D)*365.0*a;
 let direct_saved=baseline_direct-candidate_direct;
 let upstream_base=ng_gj_day(NG_BASE_MMSCFD)*365.0*a*11.5/1000.0;
 let upstream_final=ng_gj_day(NG_FINAL_MMSCFD)*365.0*a*11.5/1000.0;
 let nuclear_lca=HEAT_MWTH*1000.0*hours*(5.5*0.504)/1e6;
 let aux_lca=(PROCESS_ELECTRIC_MWE-6.3).max(0.0)*1000.0*hours*5.5/1e6;
 let transport=0.025*captured;
 let avoided=direct_saved+(upstream_base-upstream_final)-nuclear_lca-aux_lca-transport;
 AvailabilitySensitivity{availability:a,annual_h2_t,lifecycle_avoided_t:avoided,
  pass_abatement:avoided>250_000.0}
}
pub fn availability_sensitivity_csv()->String{
 let mut s=String::from("availability,h2ty,lifecycleavoidedty,apass\n");
 for a in [0.50,0.60,0.70,0.80,0.85,0.90,0.95]{
  let x=availability_sensitivity(a);
  s.push_str(&format!("{:.3},{:.3},{:.3},{}\n",a,x.annual_h2_t,x.lifecycle_avoided_t,x.pass_abatement));
 }
 s
}

#[derive(Clone,Copy,Debug)]
pub struct CcsCaptureSensitivity {
 pub delivered_fraction:f64,pub delivered_stored_t_y:f64,pub candidate_direct_t_y:f64,
 pub lifecycle_avoided_t:f64,pub pass_abatement:bool
}
// Emissions-only algebraic screen on the fraction of the canonical captured
// stream that ultimately receives credited capture/storage treatment.
// The canonical source process operating point is held fixed. A non-delivered
// fraction is returned to direct atmospheric emissions; the T&S lifecycle
// burden scales with tonnes actually delivered/stored. No CAPEX, turndown,
// solvent/compression, outage or bankable economic claim is made here.
pub fn ccs_capture_sensitivity(f:f64)->CcsCaptureSensitivity{
 assert!((0.0..=1.0).contains(&f));
 let hours=JAEA_AVAIL*8760.0;
 let source_captured=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let delivered=source_captured*f;
 let canonical_direct=short_t_to_t(EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let candidate_direct=canonical_direct+source_captured*(1.0-f);
 let baseline_direct=short_t_to_t(BASE_EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let direct_saved=baseline_direct-candidate_direct;
 let upstream_base=ng_gj_day(NG_BASE_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let upstream_final=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let nuclear_lca=HEAT_MWTH*1000.0*hours*(5.5*0.504)/1e6;
 let aux_lca=(PROCESS_ELECTRIC_MWE-6.3).max(0.0)*1000.0*hours*5.5/1e6;
 let transport=0.025*delivered;
 let avoided=direct_saved+(upstream_base-upstream_final)-nuclear_lca-aux_lca-transport;
 CcsCaptureSensitivity{delivered_fraction:f,delivered_stored_t_y:delivered,
  candidate_direct_t_y:candidate_direct,lifecycle_avoided_t:avoided,
  pass_abatement:avoided>250_000.0}
}
pub fn ccs_capture_sensitivity_csv()->String{
 let mut s=String::from("deliveredfraction,deliveredstoredty,candidatedirectty,lifecycleavoidedty,apass\n");
 for f in [0.0,0.25,0.50,0.557,0.75,0.90,1.0]{
  let x=ccs_capture_sensitivity(f);
  s.push_str(&format!("{:.3},{:.3},{:.3},{:.3},{}\n",f,x.delivered_stored_t_y,x.candidate_direct_t_y,x.lifecycle_avoided_t,x.pass_abatement));
 }
 s
}

pub const BACKUP_HEATER_EFFICIENCY:f64=0.856; // Rossi et al. 2025 conventional SMR furnace case
pub const NG_COMBUSTION_KG_CO2_MMBTU:f64=53.02; // US EPA stationary-combustion default, HHV
pub const MMBTU_GJ:f64=1.05505585262;

#[derive(Clone,Copy,Debug)]
pub struct GasBackupSensitivity {
 pub nuclear_availability:f64,pub backup_hours:f64,pub backup_fuel_gj:f64,
 pub backup_direct_co2_t:f64,pub backup_upstream_co2e_t:f64,
 pub lifecycle_avoided_t:f64,pub backup_variable_cost_sgd:f64,
 pub annual_incremental_sgd:f64,pub abatement_cost_sgd_t:f64,
 pub pass_abatement:bool,pub pass_cost:bool,pub joint_pass:bool
}
// Lower-bound operating sensitivity: gas-fired backup maintains the base 85%
// process-service hours when nuclear availability is lower. Reactor installed
// economic burden is retained at the base-case value; backup CAPEX, staffing,
// maintenance, start-up and integration costs are deliberately NOT invented.
pub fn gas_backup_sensitivity(nuclear_availability:f64)->GasBackupSensitivity{
 assert!(nuclear_availability>0.0&&nuclear_availability<=JAEA_AVAIL);
 let base=final_design(0.0,false);
 let backup_hours=(JAEA_AVAIL-nuclear_availability)*8760.0;
 let backup_heat_gj=HEAT_MWTH*backup_hours*3.6;
 let backup_fuel_gj=backup_heat_gj/BACKUP_HEATER_EFFICIENCY;
 let combustion_kg_gj=NG_COMBUSTION_KG_CO2_MMBTU/MMBTU_GJ;
 let backup_direct=backup_fuel_gj*combustion_kg_gj/1000.0;
 let backup_upstream=backup_fuel_gj*11.5/1000.0;
 // Nuclear lifecycle burden is avoided during the hours in which backup replaces nuclear heat.
 let nuclear_lca_reduction=HEAT_MWTH*1000.0*backup_hours*(5.5*0.504)/1e6;
 let avoided=base.lifecycle_avoided_t+nuclear_lca_reduction-backup_direct-backup_upstream;
 let backup_cost=backup_fuel_gj*GAS_PRICE_SGD_GJ;
 let incremental=base.annual_incremental_sgd+backup_cost;
 let cost=incremental/avoided;
 GasBackupSensitivity{nuclear_availability,backup_hours,backup_fuel_gj,
  backup_direct_co2_t:backup_direct,backup_upstream_co2e_t:backup_upstream,
  lifecycle_avoided_t:avoided,backup_variable_cost_sgd:backup_cost,
  annual_incremental_sgd:incremental,abatement_cost_sgd_t:cost,
  pass_abatement:avoided>250_000.0,pass_cost:cost<100.0,
  joint_pass:avoided>250_000.0&&cost<100.0}
}
pub fn gas_backup_sensitivity_csv()->String{
 let mut s=String::from("nuclearavailability,backuphours,backupfuelgj,backupdirectco2t,backupupstreamco2et,lifecycleavoidedty,backupvariablecostsgd,incrementalsgd,costsgdt,apass,cpass,joint\n");
 for a in [0.50,0.60,0.70,0.80,0.85]{
  let x=gas_backup_sensitivity(a);
  s.push_str(&format!("{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{}\n",a,x.backup_hours,x.backup_fuel_gj,x.backup_direct_co2_t,x.backup_upstream_co2e_t,x.lifecycle_avoided_t,x.backup_variable_cost_sgd,x.annual_incremental_sgd,x.abatement_cost_sgd_t,x.pass_abatement,x.pass_cost,x.joint_pass));
 }
 s
}

pub fn modern_final_cost(occ_usd_kwth:f64,om_usd_mwh:f64,wacc:f64,electricity_value:f64)->FinalDesign{
 let mut x=final_design(electricity_value,false);
 let hours=JAEA_AVAIL*8760.0;
 let cap=occ_usd_kwth*crate::deployment::US_ESC_2024_2025*crate::deployment::USD_SGD_2026_09_29*1000.0*REACTOR_MWTH;
 let reactor=cap*crf(wacc,60)+om_usd_mwh*crate::deployment::US_ESC_2024_2025*crate::deployment::USD_SGD_2026_09_29*REACTOR_MWTH*hours;
 let source_reactor=NISHIHARA_IHX_MWTH*hours*3.6*jaea_heat_sgd_gj(false)+NISHIHARA_GROSS_MWE*hours*jaea_electric_sgd_mwh(false);
 x.annual_incremental_sgd += reactor-source_reactor;
 x.abatement_cost_sgd_t=x.annual_incremental_sgd/x.lifecycle_avoided_t;
 x.pass_cost=x.abatement_cost_sgd_t<100.0;x.joint_pass=x.pass_abatement&&x.pass_cost;x
}

#[derive(Clone,Copy,Debug)]
pub struct FinalLifecycleLedger {
 pub baseline_direct:f64,pub candidate_direct:f64,pub direct_saved:f64,
 pub upstream_saved:f64,pub nuclear_added:f64,pub auxiliary_added:f64,
 pub transport_storage_added:f64,pub net_lifecycle_saved:f64
}
pub fn final_lifecycle_ledger()->FinalLifecycleLedger{
 let hours=JAEA_AVAIL*8760.0;
 let candidate_direct=short_t_to_t(EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let baseline_direct=short_t_to_t(BASE_EMITTED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let direct_saved=baseline_direct-candidate_direct;
 let upstream_base=ng_gj_day(NG_BASE_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let upstream_final=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*11.5/1000.0;
 let upstream_saved=upstream_base-upstream_final;
 let nuclear_added=HEAT_MWTH*1000.0*hours*(5.5*0.504)/1e6;
 let auxiliary_added=(PROCESS_ELECTRIC_MWE-6.3).max(0.0)*1000.0*hours*5.5/1e6;
 let captured=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let transport_storage_added=0.025*captured;
 let net_lifecycle_saved=direct_saved+upstream_saved-nuclear_added-auxiliary_added-transport_storage_added;
 FinalLifecycleLedger{baseline_direct,candidate_direct,direct_saved,upstream_saved,nuclear_added,auxiliary_added,transport_storage_added,net_lifecycle_saved}
}
#[derive(Clone,Copy,Debug)]
pub struct FinalCostLedger {
 pub baseline_ng:f64,pub candidate_ng:f64,pub ng_saved:f64,pub reactor_added:f64,
 pub ccs_annual_added:f64,pub integration_added:f64,pub transport_storage_added:f64,
 pub electricity_revenue:f64,pub net_incremental:f64
}
pub fn final_cost_ledger()->FinalCostLedger{
 let hours=JAEA_AVAIL*8760.0;
 let baseline_ng=ng_gj_day(NG_BASE_MMSCFD)*365.0*JAEA_AVAIL*GAS_PRICE_SGD_GJ;
 let candidate_ng=ng_gj_day(NG_FINAL_MMSCFD)*365.0*JAEA_AVAIL*GAS_PRICE_SGD_GJ;
 let ng_saved=baseline_ng-candidate_ng;
 let reactor_added=NISHIHARA_IHX_MWTH*hours*3.6*jaea_heat_sgd_gj(false)+NISHIHARA_GROSS_MWE*hours*jaea_electric_sgd_mwh(false);
 let captured=short_t_to_t(CAPTURED_SHORT_T_D)*365.0*JAEA_AVAIL;
 let ccs_cap=ccs_capex_sgd(captured);
 let ccs_annual_added=ccs_cap*crf(0.08,25);
 let integration_added=0.10*(jaea_plant_capex_sgd(false)+ccs_cap)*crf(0.03,40);
 let transport_storage_added=captured*T_AND_S_SGD_T;
 let electricity_revenue=0.0;
 let net_incremental=candidate_ng+reactor_added+ccs_annual_added+integration_added+transport_storage_added-electricity_revenue-baseline_ng;
 FinalCostLedger{baseline_ng,candidate_ng,ng_saved,reactor_added,ccs_annual_added,integration_added,transport_storage_added,electricity_revenue,net_incremental}
}
pub fn results_csv()->String{
 let e=final_design(0.0,false);
 format!("scenario,h2ty,heatmw,heliumkgs,residualthermalmwth,sourcegrossmwe,capturedty,avoidedty,lifecycleci,incrementalsgd,costsgdt,apass,cpass,joint\nZero-value electricity,{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.6},{:.3},{:.3},{},{},{}\n",
 e.annual_h2_t,HEAT_MWTH,e.helium_flow_kg_s,e.residual_thermal_capacity_mw,e.source_gross_electric_mwe,e.captured_t_y,e.lifecycle_avoided_t,e.lifecycle_ci_kgkg,e.annual_incremental_sgd,e.abatement_cost_sgd_t,e.pass_abatement,e.pass_cost,e.joint_pass)
}
pub fn source_balance_csv()->String{format!("parameter,value,unit,class\nReformer outlet,{REFORMER_C},C,SOURCE-BACKED INL\nProcess heat delivery,{PROCESS_HEAT_C},C,SOURCE-BACKED INL\nReactor outlet INL,{INL_ROT_C},C,SOURCE-BACKED INL\nPressure,{PRESSURE_BAR},bar,SOURCE-BACKED INL\nSteam-carbon,{STEAM_CARBON},mol/mol,SOURCE-BACKED INL\nMethane conversion,{METHANE_CONVERSION},fraction,SOURCE-BACKED INL\nPSA recovery,{PSA_RECOVERY},fraction,SOURCE-BACKED INL\nHydrogen,{H2_MMSCFD},MMSCFD,SOURCE-BACKED INL\nNatural gas final,{NG_FINAL_MMSCFD},MMSCFD,SOURCE-BACKED INL\nProcess heat,{HEAT_MWTH},MWth,SOURCE-BACKED INL\nProcess electricity,{PROCESS_ELECTRIC_MWE},MWe,SOURCE-BACKED INL\nCaptured CO2,{CAPTURED_SHORT_T_D},short ton/day,SOURCE-BACKED INL\nEmitted CO2,{EMITTED_SHORT_T_D},short ton/day,SOURCE-BACKED INL\nJAEA reactor,{REACTOR_MWTH},MWth,SOURCE-BACKED NISHIHARA\nJAEA source IHX,{NISHIHARA_IHX_MWTH},MWth,SOURCE-BACKED NISHIHARA\nJAEA source gross electricity,{NISHIHARA_GROSS_MWE},MWe,SOURCE-BACKED NISHIHARA\nProject residual thermal capacity,{:.3},MWth,PROJECT-DERIVED CAPACITY ONLY\nProject electricity export,0,MWe,NOT CLAIMED - NO OFF-DESIGN MODEL\n",REACTOR_MWTH-HEAT_MWTH)
}
pub fn temperature_sensitivity_csv()->String{
 String::from("case,reactoroutc,heatdeliveryc,reformerc,architecture\nINL lower,875,850,871,nuclear plus fired trim\nFINAL INL process,925,900,871,all nuclear reforming heat\nJAEA hardware source,950,900,871,separate GTHTR300C hardware/economic architecture\n")
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn source_temperature_ladder(){assert!(INL_ROT_C>PROCESS_HEAT_C&&PROCESS_HEAT_C>REFORMER_C);assert_eq!(PROCESS_HEAT_C-REFORMER_C,29.0);}
 #[test]fn source_balance_values(){assert_eq!(METHANE_CONVERSION,0.781);assert_eq!(STEAM_CARBON,3.0);assert_eq!(PSA_RECOVERY,0.88);}
 #[test]fn nishihara_source_variant_identity(){assert_eq!(NISHIHARA_IHX_MWTH,370.0);assert_eq!(NISHIHARA_POWER_BRANCH_MWTH,230.0);assert_eq!(NISHIHARA_GROSS_MWE,88.0);assert!(HEAT_MWTH<NISHIHARA_IHX_MWTH);}
 #[test]fn reactor_thermal_capacity_closes(){let x=final_design(0.0,false);assert!((HEAT_MWTH+x.residual_thermal_capacity_mw-REACTOR_MWTH).abs()<1e-12);}
 #[test]fn no_unsupported_project_power_claim(){let x=final_design(0.0,false);assert_eq!(x.source_gross_electric_mwe,88.0);}
 #[test]fn inl_helium_state_reproduces_source_flow(){let x=final_design(150.0,false);assert!((x.helium_flow_kg_s-78.49).abs()/78.49<0.005);}
 #[test]fn lifecycle_ci_units_are_correct(){let x=final_design(150.0,false);assert!(x.lifecycle_ci_kgkg>1.9&&x.lifecycle_ci_kgkg<2.1);}
 #[test]fn final_mass_scale_positive(){let x=final_design(150.0,false);assert!(x.annual_h2_t>90_000.0&&x.captured_t_y>500_000.0);}
 #[test]fn final_lifecycle_positive_and_threshold(){let x=final_design(150.0,false);assert!(x.lifecycle_avoided_t>250_000.0);}
 #[test]fn zero_value_case_recovers_full_reactor_burden(){let x=final_design(0.0,false);assert!(x.annual_incremental_sgd.is_finite());assert!(x.abatement_cost_sgd_t<100.0);}
 #[test]fn electricity_value_does_not_create_unsupported_credit(){let a=final_design(0.0,false);let b=final_design(200.0,false);assert!((a.annual_incremental_sgd-b.annual_incremental_sgd).abs()<1e-9);}
 #[test]fn doubled_cost_is_adverse_not_capacity(){let a=final_design(0.0,false);let b=final_design(0.0,true);assert!(b.abatement_cost_sgd_t>a.abatement_cost_sgd_t);assert_eq!(a.source_gross_electric_mwe,b.source_gross_electric_mwe);}
 #[test]fn modern_cost_sensitivity_is_more_expensive(){assert!(modern_final_cost(3250.0,16.0,0.10,150.0).abatement_cost_sgd_t>modern_final_cost(2500.0,12.0,0.075,150.0).abatement_cost_sgd_t);}
 #[test]fn lifecycle_ledger_reconciles(){let l=final_lifecycle_ledger();let x=final_design(0.0,false);assert!((l.net_lifecycle_saved-x.lifecycle_avoided_t).abs()<1e-8);}
 #[test]fn cost_ledger_reconciles(){let l=final_cost_ledger();let x=final_design(0.0,false);assert!((l.net_incremental-x.annual_incremental_sgd).abs()<1e-8);assert_eq!(l.electricity_revenue,0.0);}
 #[test]fn availability_sensitivity_is_monotonic(){let a=availability_sensitivity(0.50);let b=availability_sensitivity(0.85);assert!(b.annual_h2_t>a.annual_h2_t);assert!(b.lifecycle_avoided_t>a.lifecycle_avoided_t);assert!(a.lifecycle_avoided_t>250_000.0);}
 #[test]fn availability_base_reconciles(){let a=availability_sensitivity(JAEA_AVAIL);let b=final_design(0.0,false);assert!((a.annual_h2_t-b.annual_h2_t).abs()<1e-8);assert!((a.lifecycle_avoided_t-b.lifecycle_avoided_t).abs()<1e-8);}
 #[test]fn ccs_capture_base_reconciles(){let x=ccs_capture_sensitivity(1.0);let b=final_design(0.0,false);assert!((x.lifecycle_avoided_t-b.lifecycle_avoided_t).abs()<1e-8);}
 #[test]fn ccs_capture_abatement_is_monotonic(){let a=ccs_capture_sensitivity(0.25);let b=ccs_capture_sensitivity(0.90);assert!(b.delivered_stored_t_y>a.delivered_stored_t_y);assert!(b.lifecycle_avoided_t>a.lifecycle_avoided_t);}
 #[test]fn availability_threshold_reproduces(){let b=final_design(0.0,false);let crit=JAEA_AVAIL*250_000.0/b.lifecycle_avoided_t;assert!((crit-0.231697).abs()<1e-5);let lo=availability_sensitivity(crit-1e-6);let hi=availability_sensitivity(crit+1e-6);assert!(!lo.pass_abatement&&hi.pass_abatement);}
 #[test]fn ccs_zero_delivered_endpoint(){let x=ccs_capture_sensitivity(0.0);assert!((x.lifecycle_avoided_t-388_335.9).abs()<2.0);assert!(x.candidate_direct_t_y>580_000.0);assert!(x.pass_abatement);}
 #[test]fn gas_backup_base_reconciles(){let x=gas_backup_sensitivity(JAEA_AVAIL);let b=final_design(0.0,false);assert!(x.backup_hours.abs()<1e-12);assert!((x.lifecycle_avoided_t-b.lifecycle_avoided_t).abs()<1e-8);assert!((x.abatement_cost_sgd_t-b.abatement_cost_sgd_t).abs()<1e-8);}
 #[test]fn gas_backup_penalty_is_monotonic(){let low=gas_backup_sensitivity(0.50);let high=gas_backup_sensitivity(0.80);assert!(low.backup_fuel_gj>high.backup_fuel_gj);assert!(low.lifecycle_avoided_t<high.lifecycle_avoided_t);assert!(low.abatement_cost_sgd_t>high.abatement_cost_sgd_t);assert!(low.joint_pass);}
 #[test]fn historical_gate5_still_reproduces(){let x=r3_uncertainty_summary();assert_eq!(x.n,64);assert_eq!(x.both_pass,0);}
}
