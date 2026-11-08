use crate::*;

pub const REACTOR_MWTH:f64=600.0;
pub const PRIMARY_OUT_C:f64=950.0;
pub const IHX_SECONDARY_OUT_C:f64=900.0; // GTHTR300C source IHX
pub const REFORMER_HE_IN_C:f64=880.0;    // HTTR SMR source condition after hot-duct loss
pub const REFORMER_HE_OUT_C:f64=585.0;   // HTTR design condition
pub const PROCESS_GAS_OUT_C:f64=600.0;   // JAEA HTTR/mock-up source process-gas outlet
pub const MIN_IHX_APPROACH_K:f64=50.0;
pub const MIN_PROCESS_APPROACH_K:f64=40.0;
pub const JAEA_AVAIL:f64=0.85;
pub const JAEA_PLANT_BJPY:f64=59.7;
pub const JAEA_IHX_LOOP_BJPY:f64=11.2;
pub const JAEA_HEAT_JPY_MJ:f64=0.52;
pub const JAEA_ELEC_JPY_KWH:f64=4.9;
pub const JAEA_DOUBLE_PLANT_BJPY:f64=70.9;
pub const JAEA_DOUBLE_HEAT_JPY_MJ:f64=0.57;
pub const JAEA_DOUBLE_ELEC_JPY_KWH:f64=5.5;
pub const JPY_SGD_2026_09_29:f64=0.008117;
pub const USD_SGD_2026_09_29:f64=1.2776;
pub const EUR_SGD_2026_09_29:f64=1.452;
pub const JP_DEFLATOR_2007:f64=99.59;
pub const JP_DEFLATOR_2025:f64=112.27;
pub const DE_DEFLATOR_2014:f64=90.46;
pub const DE_DEFLATOR_2025:f64=123.84;
pub const US_ESC_2024_2025:f64=1.0280;
pub const CCS_TCR_EUR2014:f64=41.02e6;
pub const CCS_REF_CAPTURE_T_Y:f64=0.4660*100_000.0*8322.0/1000.0;

#[derive(Clone,Copy,Debug)]
pub struct DeploymentPhysical {
 pub annual_scale:f64,pub throughput_scale:f64,pub annual_h2_t:f64,
 pub process_heat_mw:f64,pub helium_flow_kg_s:f64,pub co2_stored_t_y:f64,
 pub annual_avoided_t:f64,pub specific_abatement_t_t:f64,
 pub reactor_utilisation:f64,pub gross_electric_mwe:f64,pub net_electric_mwe:f64,
 pub annual_electric_mwh:f64,pub ihx_approach_k:f64,pub process_approach_k:f64,
}
pub fn deployment_physical(annual_scale:f64)->DeploymentPhysical {
 assert!(annual_scale>0.0);
 assert!(PRIMARY_OUT_C>IHX_SECONDARY_OUT_C);
 assert!(REFORMER_HE_IN_C>PROCESS_GAS_OUT_C);
 let hours=JAEA_AVAIL*8760.0;
 let throughput_scale=annual_scale*8322.0/hours;
 let fref=r3_canonical_recycle_case().fresh_fraction;
 let s=r3_solve_at_fixed_fresh(fref,PROCESS_GAS_OUT_C+273.15,20.0,685.15,27.7,0.95,0.90,1e-6,50000);
 assert!(s.converged);
 let c=r3_ccs_ledger(s);
 let hin=wet6_enthalpy_mw(s.reformer_in,650.0+273.15);
 let hout=wet6_enthalpy_mw(s.reformer_out,PROCESS_GAS_OUT_C+273.15);
 let qref=(hout-hin).max(0.0);
 let qwhb=wet6_enthalpy_mw(s.reformer_out,PROCESS_GAS_OUT_C+273.15)
     -wet6_enthalpy_mw(s.reformer_out,320.0+273.15);
 let rec=c.mdea_heat_hi_mw.min(qwhb.max(0.0));
 let q=(qref+(c.mdea_heat_hi_mw-rec))*throughput_scale;
 let heflow=helium_mass_flow_kg_s(q,5.2,REFORMER_HE_IN_C,REFORMER_HE_OUT_C);
 let h2kg_h=s.product_h2_kmol_h*2.01588;
 let h2=h2kg_h*hours*throughput_scale/1000.0;
 let purge=c.purge_oxidation_co2_kmol_h;
 let stored=(s.captured_co2_kmol_h+0.95*purge)*44.0095*hours*throughput_scale/1000.0;
 let residual=((s.shifted.co2-s.captured_co2_kmol_h)+0.05*purge)*44.0095/h2kg_h;
 let upstream=(ieaghg_feed_lhv_mw()*s.fresh_fraction*3600.0*11.5/1000.0)/h2kg_h;
 let nuclear=(q/throughput_scale)*1000.0*(5.5*0.504)/1000.0/h2kg_h;
 let circ=helium_circulator_power_mw(heflow,58.0*4.0,5.15,REFORMER_HE_OUT_C,0.70);
 let aux=(c.co2_compression_mwe+c.tail_compression_mwe)*throughput_scale+circ;
 let aux_ci=aux*5.5*1000.0/(h2kg_h*throughput_scale);
 let transport=(stored/h2)*0.025;
 let ci=residual+upstream+nuclear+aux_ci+transport;
 let base=ieaghg_unabated_lifecycle_screen(11.5).total();
 let specific=base-ci;
 let avoided=specific*h2;
 // GTHTR300C source: 202 MWe at 170 MW heat; 276 MWe at zero heat.
 let gross=202.0+(170.0-q).clamp(0.0,170.0)*(74.0/170.0);
 let net=gross; // source electricity is plant output; no second auxiliary deduction.
 DeploymentPhysical{annual_scale,throughput_scale,annual_h2_t:h2,process_heat_mw:q,
  helium_flow_kg_s:heflow,co2_stored_t_y:stored,annual_avoided_t:avoided,
  specific_abatement_t_t:specific,reactor_utilisation:q/REACTOR_MWTH,
  gross_electric_mwe:gross,net_electric_mwe:net,annual_electric_mwh:net*hours,
  ihx_approach_k:PRIMARY_OUT_C-IHX_SECONDARY_OUT_C,
  process_approach_k:REFORMER_HE_IN_C-PROCESS_GAS_OUT_C}
}
pub fn minimum_annual_scale()->f64 {let x=deployment_physical(1.0);if x.annual_avoided_t>0.0{250_000.0/x.annual_avoided_t}else{f64::INFINITY}}
pub fn selected_annual_scale()->f64 {let m=minimum_annual_scale();if m.is_finite(){(m*1.10).max(1.40)}else{1.50}}

pub fn jp_escalation()->f64{JP_DEFLATOR_2025/JP_DEFLATOR_2007}
pub fn eu_escalation()->f64{DE_DEFLATOR_2025/DE_DEFLATOR_2014}
pub fn jaea_heat_sgd_gj(double_ihx:bool)->f64{let x=if double_ihx{JAEA_DOUBLE_HEAT_JPY_MJ}else{JAEA_HEAT_JPY_MJ};x*jp_escalation()*JPY_SGD_2026_09_29*1000.0}
pub fn jaea_electric_sgd_mwh(double_ihx:bool)->f64{let x=if double_ihx{JAEA_DOUBLE_ELEC_JPY_KWH}else{JAEA_ELEC_JPY_KWH};x*jp_escalation()*JPY_SGD_2026_09_29*1000.0}
pub fn jaea_plant_capex_sgd(double_ihx:bool)->f64{let x=if double_ihx{JAEA_DOUBLE_PLANT_BJPY}else{JAEA_PLANT_BJPY};x*1e9*jp_escalation()*JPY_SGD_2026_09_29}
pub fn ccs_capex_sgd(stored:f64)->f64{CCS_TCR_EUR2014*eu_escalation()*EUR_SGD_2026_09_29*(stored/CCS_REF_CAPTURE_T_Y)}
pub fn crf(i:f64,n:u32)->f64{i*(1.0+i).powi(n as i32)/((1.0+i).powi(n as i32)-1.0)}

#[derive(Clone,Copy,Debug)]
pub struct EconomicResult {
 pub annual_scale:f64,pub double_ihx:bool,pub electricity_value_sgd_mwh:f64,
 pub reactor_burden_sgd_y:f64,pub electricity_value_sgd_y:f64,
 pub ccs_capex_sgd:f64,pub annual_incremental_sgd:f64,pub cost_sgd_t:f64,
 pub pass_abatement:bool,pub pass_cost:bool,pub joint_pass:bool,
}
pub fn mature_economic(annual_scale:f64,double_ihx:bool,electricity_value:f64)->EconomicResult {
 let x=deployment_physical(annual_scale);let hours=JAEA_AVAIL*8760.0;
 // Full source economic burden: source-priced heat + source-priced electricity.
 let heat_burden=x.process_heat_mw*hours*3.6*jaea_heat_sgd_gj(double_ihx);
 let power_burden=x.annual_electric_mwh*jaea_electric_sgd_mwh(double_ihx);
 let reactor_burden=heat_burden+power_burden;
 let power_value=x.annual_electric_mwh*electricity_value;
 let ccs=ccs_capex_sgd(x.co2_stored_t_y);
 let annual_ccs=ccs*crf(0.08,25);
 let plant=jaea_plant_capex_sgd(double_ihx);
 let integration=0.10*(plant+ccs)*crf(0.03,40);
 let ts=x.co2_stored_t_y*15.0;
 let fref=r3_canonical_recycle_case().fresh_fraction;
 let s=r3_solve_at_fixed_fresh(fref,PROCESS_GAS_OUT_C+273.15,20.0,685.15,27.7,0.95,0.90,1e-6,50000);
 assert!(s.converged);
 let base=annual_thermal_energy_cost_sgd(ieaghg_total_ng_lhv_mw()*x.throughput_scale,hours,15.0);
 let fresh=annual_thermal_energy_cost_sgd(ieaghg_feed_lhv_mw()*s.fresh_fraction*x.throughput_scale,hours,15.0);
 let c=r3_ccs_ledger(s);
 let aux=(c.co2_compression_mwe+c.tail_compression_mwe)*x.throughput_scale*hours*150.0;
 let incremental=fresh+aux+reactor_burden+annual_ccs+integration+ts-power_value-base;
 let cost=if x.annual_avoided_t>0.0{incremental/x.annual_avoided_t}else{f64::INFINITY};
 EconomicResult{annual_scale,double_ihx,electricity_value_sgd_mwh:electricity_value,
  reactor_burden_sgd_y:reactor_burden,electricity_value_sgd_y:power_value,
  ccs_capex_sgd:ccs,annual_incremental_sgd:incremental,cost_sgd_t:cost,
  pass_abatement:x.annual_avoided_t>250_000.0,pass_cost:cost<100.0,
  joint_pass:x.annual_avoided_t>250_000.0&&cost<100.0}
}
pub fn break_even_electricity_sgd_mwh(annual_scale:f64,double_ihx:bool)->f64 {
 let z=mature_economic(annual_scale,double_ihx,0.0);let x=deployment_physical(annual_scale);
 if x.annual_avoided_t<=0.0{f64::INFINITY}else{(z.annual_incremental_sgd-100.0*x.annual_avoided_t)/x.annual_electric_mwh}
}

#[derive(Clone,Copy,Debug)]
pub enum CostClass{ModernCentral,FoakAdverse}
pub fn modern_cost(annual_scale:f64,k:CostClass)->EconomicResult {
 let x=deployment_physical(annual_scale);let hours=match k{CostClass::ModernCentral=>0.93*8760.0,CostClass::FoakAdverse=>0.80*8760.0};
 let (occ,om,wacc,ts,integ)=match k{CostClass::ModernCentral=>(2500.0*US_ESC_2024_2025*USD_SGD_2026_09_29,12.0*US_ESC_2024_2025*USD_SGD_2026_09_29,0.075,30.0,0.20),CostClass::FoakAdverse=>(3250.0*US_ESC_2024_2025*USD_SGD_2026_09_29,16.0*US_ESC_2024_2025*USD_SGD_2026_09_29,0.10,45.0,0.30)};
 let cap=occ*1000.0*REACTOR_MWTH;let reactor=cap*crf(wacc,60)+om*REACTOR_MWTH*hours;
 let ccs=ccs_capex_sgd(x.co2_stored_t_y);let annual_ccs=ccs*crf(0.08,25);
 let integration=integ*(cap+ccs)*crf(wacc,60);
 let fref=r3_canonical_recycle_case().fresh_fraction;
 let s=r3_solve_at_fixed_fresh(fref,PROCESS_GAS_OUT_C+273.15,20.0,685.15,27.7,0.95,0.90,1e-6,50000);
 assert!(s.converged);
 let base=annual_thermal_energy_cost_sgd(ieaghg_total_ng_lhv_mw()*x.throughput_scale,hours,17.5);
 let fresh=annual_thermal_energy_cost_sgd(ieaghg_feed_lhv_mw()*s.fresh_fraction*x.throughput_scale,hours,17.5);
 let incremental=fresh+reactor+annual_ccs+integration+x.co2_stored_t_y*ts-base;
 let cost=if x.annual_avoided_t>0.0{incremental/x.annual_avoided_t}else{f64::INFINITY};
 EconomicResult{annual_scale,double_ihx:false,electricity_value_sgd_mwh:0.0,reactor_burden_sgd_y:reactor,electricity_value_sgd_y:0.0,ccs_capex_sgd:ccs,annual_incremental_sgd:incremental,cost_sgd_t:cost,pass_abatement:x.annual_avoided_t>250_000.0,pass_cost:cost<100.0,joint_pass:x.annual_avoided_t>250_000.0&&cost<100.0}
}

pub fn review5_results_csv()->String{
 let a=selected_annual_scale();let mut s=String::from("scenario,scale,h2ty,avoidedty,processheatmw,heflowkgs,electricmwe,electricmwh,electricvaluesgdmwh,costsgdt,jointpass\n");
 let p=deployment_physical(a);
 for (n,r) in [("JAEA zero surplus",mature_economic(a,false,0.0)),("JAEA break-even",mature_economic(a,false,break_even_electricity_sgd_mwh(a,false))),("JAEA SG 100",mature_economic(a,false,100.0)),("JAEA SG 150",mature_economic(a,false,150.0)),("JAEA SG 200",mature_economic(a,false,200.0)),("JAEA doubled IHX SG150",mature_economic(a,true,150.0)),("Modern central",modern_cost(a,CostClass::ModernCentral)),("FOAK adverse",modern_cost(a,CostClass::FoakAdverse))]{s.push_str(&format!("{},{:.4},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{}\n",n,a,p.annual_h2_t,p.annual_avoided_t,p.process_heat_mw,p.helium_flow_kg_s,p.net_electric_mwe,p.annual_electric_mwh,r.electricity_value_sgd_mwh,r.cost_sgd_t,r.joint_pass));}s
}
pub fn review5_margin_csv()->String{
 let mut s=String::from("scale,h2ty,avoidedty,marginpct,processheatmw,utilisation,costsgdt\n");
 for a in [1.0,1.30,1.35,1.40,1.50]{let p=deployment_physical(a);let e=mature_economic(a,false,150.0);s.push_str(&format!("{:.6},{:.3},{:.3},{:.3},{:.3},{:.4},{:.3}\n",a,p.annual_h2_t,p.annual_avoided_t,100.0*(p.annual_avoided_t/250000.0-1.0),p.process_heat_mw,p.reactor_utilisation,e.cost_sgd_t));}s
}
pub fn review5_cogeneration_csv()->String{
 let a=selected_annual_scale();let p=deployment_physical(a);let z=mature_economic(a,false,0.0);
 format!("quantity,value,unit\nReactor thermal capacity,{:.3},MWth\nHydrogen process heat,{:.3},MWth\nRemaining thermal service,{:.3},MWth\nGross electric output,{:.3},MWe\nNet electric output,{:.3},MWe\nAnnual electricity,{:.3},MWh/y\nFull reactor economic burden,{:.3},SGD/y\nZero-value incremental cost,{:.3},SGD/y\nCN4252 break-even electricity value,inf,SGD/MWh\nEMA context low,100,SGD/MWh\nEMA context central,150,SGD/MWh\nEMA context high,200,SGD/MWh\n",REACTOR_MWTH,p.process_heat_mw,REACTOR_MWTH-p.process_heat_mw,p.gross_electric_mwe,p.net_electric_mwe,p.annual_electric_mwh,z.reactor_burden_sgd_y,z.annual_incremental_sgd)
}
pub fn review5_cost_ledger_csv()->String{format!("item,sourceyear,sourcecurrency,sourcevalue,indexmethod,indexsource,indexfactor,fxdate,fxsgd,finalbasis,convertedsgd\nGTHTR300C plant,2007,JPY,{:.3} billion,Japan GDP deflator,World Bank 2007=99.59 2025=112.27,{:.6},2026-09-29,{:.6},2025-price SGD,{:.3}\nGTHTR300C IHX-loop,2007,JPY,{:.3} billion,Japan GDP deflator,World Bank,{:.6},2026-09-29,{:.6},2025-price SGD,{:.3}\nIEAGHG CCS increment,2014,EUR,{:.3} million,Germany GDP deflator proxy,World Bank/IMF 2014=90.46 2025=123.84,{:.6},2026-09-29,{:.3},2025-price SGD,{:.3}\n",JAEA_PLANT_BJPY,jp_escalation(),JPY_SGD_2026_09_29,jaea_plant_capex_sgd(false),JAEA_IHX_LOOP_BJPY,jp_escalation(),JPY_SGD_2026_09_29,JAEA_IHX_LOOP_BJPY*1e9*jp_escalation()*JPY_SGD_2026_09_29,CCS_TCR_EUR2014/1e6,eu_escalation(),EUR_SGD_2026_09_29,CCS_TCR_EUR2014*eu_escalation()*EUR_SGD_2026_09_29)}
#[cfg(test)]mod tests{use super::*;
 #[test]fn temperatures_are_source_defined_and_positive(){let p=deployment_physical(1.5);assert!(p.ihx_approach_k>=MIN_IHX_APPROACH_K);assert!(p.process_approach_k>=MIN_PROCESS_APPROACH_K);assert_eq!(REFORMER_HE_IN_C,880.0);assert_eq!(PROCESS_GAS_OUT_C,600.0);}
 #[test]fn duty_fits_ihx_and_module(){let p=deployment_physical(1.5);assert!(p.process_heat_mw<170.0);assert!(p.reactor_utilisation<1.0);}
 #[test]fn cost_conversions_reproduce(){assert!((jp_escalation()-112.27/99.59).abs()<1e-12);assert!((jaea_plant_capex_sgd(false)-59.7e9*(112.27/99.59)*0.008117).abs()<1.0);}
 #[test]fn source_temperature_resolve_is_adverse(){let p=deployment_physical(1.0);assert!(p.annual_avoided_t<=0.0);assert!(minimum_annual_scale().is_infinite());}
 #[test]fn no_finite_abatement_cost_when_abatement_nonpositive(){for v in [0.0,100.0,150.0,200.0]{let e=mature_economic(1.5,false,v);assert!(e.cost_sgd_t.is_infinite()&&!e.joint_pass);}}
 #[test]fn full_reactor_burden_is_closed(){let e=mature_economic(1.5,false,150.0);assert!(e.reactor_burden_sgd_y>0.0&&e.electricity_value_sgd_y>0.0);}
 #[test]fn doubled_ihx_raises_full_reactor_burden(){assert!(mature_economic(1.5,true,150.0).reactor_burden_sgd_y>mature_economic(1.5,false,150.0).reactor_burden_sgd_y);}
 #[test]fn break_even_is_undefined_without_positive_abatement(){assert!(break_even_electricity_sgd_mwh(1.5,false).is_infinite());}
 #[test]fn gate5_unchanged(){let x=r3_uncertainty_summary();assert_eq!(x.n,64);assert_eq!(x.both_pass,0);}
}
