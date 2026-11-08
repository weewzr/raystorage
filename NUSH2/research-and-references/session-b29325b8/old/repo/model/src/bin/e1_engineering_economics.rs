use nuclear_assisted_smr::e1_economics::{cases_csv, provenance_csv, break_even_ng_price, break_even_ts_price, ihx_loop_annual_sgd_unscaled, ihx_loop_annual_sgd_linear_duty_sensitivity, NuclearCase};
use std::fs;
fn main(){
 let dir="../results/e1_economics";
 fs::create_dir_all(dir).unwrap();
 fs::write(format!("{dir}/e1_cases.csv"),cases_csv()).unwrap();
 fs::write(format!("{dir}/e1_input_provenance.csv"),provenance_csv()).unwrap();
 let s=format!("metric,value,unit\nbreak_even_ng_price_central,{:.6},SGD/GJ\nbreak_even_ts_price_central,{:.6},SGD/tCO2\niaea1682_ihx_loop_annual_unscaled,{:.3},SGD/y\niaea1682_ihx_loop_annual_linear_duty_sensitivity,{:.3},SGD/y\n",
  break_even_ng_price(NuclearCase::MhrtOneModuleCentral,15.0),
  break_even_ts_price(NuclearCase::MhrtOneModuleCentral,15.0),
  ihx_loop_annual_sgd_unscaled(),ihx_loop_annual_sgd_linear_duty_sensitivity());
 fs::write(format!("{dir}/e1_thresholds.csv"),s).unwrap();
}
