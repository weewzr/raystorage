use nuclear_assisted_smr::e3_siting::{cooling_csv,infrastructure_csv};use std::fs;
fn main(){let d="../results/e3_siting";fs::create_dir_all(d).unwrap();fs::write(format!("{d}/e3_cooling_screen.csv"),cooling_csv()).unwrap();fs::write(format!("{d}/e3_infrastructure_demands.csv"),infrastructure_csv()).unwrap();}
