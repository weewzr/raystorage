use nuclear_assisted_smr::e2b_maturation::{scenarios_csv,thresholds_csv};
use std::fs;
fn main(){let d="../results/e2b_maturation";fs::create_dir_all(d).unwrap();fs::write(format!("{d}/e2b_scenarios.csv"),scenarios_csv()).unwrap();fs::write(format!("{d}/e2b_thresholds.csv"),thresholds_csv()).unwrap();}
