use nuclear_assisted_smr::final_design::ccs_capture_sensitivity;
fn main(){
 println!("# CCS Delivered-Storage Robustness Screen\n");
 println!("Emissions-only algebraic screen on the fraction of the canonical captured stream that ultimately receives credited capture/storage treatment. The source process operating point is fixed. No CAPEX, turndown, solvent/compression, outage or bankable economic claim is made.\n");
 println!("| Delivered/stored fraction | Delivered/stored (t/y) | Candidate direct CO2 (t/y) | Lifecycle avoided (tCO2e/y) | >0.25 Mt/y |");
 println!("|---:|---:|---:|---:|:---:|");
 for f in [0.0,0.25,0.50,0.557,0.75,0.90,1.0]{
  let x=ccs_capture_sensitivity(f);
  println!("| {:.1}% | {:.0} | {:.0} | {:.0} | {} |",100.0*f,x.delivered_stored_t_y,x.candidate_direct_t_y,x.lifecycle_avoided_t,if x.pass_abatement{"PASS"}else{"FAIL"});
 }
 let z=ccs_capture_sensitivity(0.0);
 println!("\nAt zero delivered/stored canonical captured stream the narrow screen gives **{:.3} MtCO2e/y avoided** but candidate direct emissions rise to **{:.3} MtCO2/y**.",z.lifecycle_avoided_t/1e6,z.candidate_direct_t_y/1e6);
 println!("This does not establish that CCS is unnecessary, acceptable direct emissions, outage operability, regulatory acceptability, redesigned capture-plant economics or commercial viability.");
}