use nuclear_assisted_smr::final_design::{availability_sensitivity,gas_backup_sensitivity};
fn main(){
 println!("# Final-Design Reliability Sensitivity\n");
 println!("The no-backup case is an emissions/throughput-only linear screen, not a plant-downtime economic or reliability model.\n");
 println!("## No-backup effective process-availability screen\n");
 println!("| Effective process availability | H2 (t/y) | Lifecycle avoided (tCO2e/y) | >0.25 Mt/y |");
 println!("|---:|---:|---:|:---:|");
 for a in [0.30,0.40,0.50,0.60,0.70,0.80,0.85,0.90,0.95]{
  let x=availability_sensitivity(a);
  println!("| {:.0}% | {:.0} | {:.0} | {} |",100.0*a,x.annual_h2_t,x.lifecycle_avoided_t,if x.pass_abatement{"PASS"}else{"FAIL"});
 }
 let acrit=0.85*250_000.0/availability_sensitivity(0.85).lifecycle_avoided_t;
 println!("\nThe linear emissions screen crosses 0.25 MtCO2e/y at approximately **{:.1}% effective process availability**.",100.0*acrit);
 println!("This does NOT imply commercially acceptable availability, reliable H2 supply, realistic industrial operation, or acceptable economics. No no-backup availability economics are exposed because installed/fixed cost behaviour under downtime is not defensibly decomposed.\n");
 println!("## Gas-backup lower-bound operating sensitivity\n");
 println!("Here the x-variable is **nuclear-source availability**, while gas backup maintains the defined 85% process-service hours. Backup CAPEX/staffing/maintenance/start-up/integration are excluded.\n");
 println!("| Nuclear-source availability | Backup h/y | Backup fuel (GJ/y) | Backup direct CO2 (t/y) | Lifecycle avoided (tCO2e/y) | Backup fuel S$/y | S$/tCO2e | Joint pass |");
 println!("|---:|---:|---:|---:|---:|---:|---:|:---:|");
 for a in [0.50,0.60,0.70,0.80,0.85]{
  let x=gas_backup_sensitivity(a);
  println!("| {:.0}% | {:.0} | {:.0} | {:.0} | {:.0} | {:.0} | {:.2} | {} |",100.0*a,x.backup_hours,x.backup_fuel_gj,x.backup_direct_co2_t,x.lifecycle_avoided_t,x.backup_variable_cost_sgd,x.abatement_cost_sgd_t,if x.joint_pass{"PASS"}else{"FAIL"});
 }
 println!("\nInterpretation: gas-backup economics are lower-bound operating economics only; installed backup capital and fixed O&M are not represented.");
}