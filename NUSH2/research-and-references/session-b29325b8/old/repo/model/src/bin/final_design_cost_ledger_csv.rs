use nuclear_assisted_smr::final_design::final_cost_ledger;
fn main(){let x=final_cost_ledger();println!("item,direction,annual_sgd,meaning");
println!("Natural gas expenditure saved,SAVED,{:.3},baseline natural gas minus candidate natural gas",x.ng_saved);
println!("Full selected reactor economic burden,ADDED,{:.3},source heat plus source electricity product burden",x.reactor_added);
println!("CCS capital annualisation,ADDED,{:.3},scaled capture capital times CRF",x.ccs_annual_added);
println!("Integration and site allowance,ADDED,{:.3},screening integration allowance annualised",x.integration_added);
println!("CO2 transport and storage,ADDED,{:.3},captured tonnes times screening transport-storage tariff",x.transport_storage_added);
println!("Project electricity revenue,NO CHANGE,{:.3},zero credit by controlling boundary",x.electricity_revenue);
println!("NET ANNUAL COST CHANGE,NET,{:.3},new costs minus saved natural gas",x.net_incremental);}