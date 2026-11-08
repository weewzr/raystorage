use nuclear_assisted_smr::final_design::final_lifecycle_ledger;
fn main(){let x=final_lifecycle_ledger();println!("item,direction,change_tco2e_y,meaning");
println!("Direct plant emissions saved,SAVED,{:.3},baseline direct minus candidate direct",x.direct_saved);
println!("Upstream natural gas emissions saved,SAVED,{:.3},lower natural gas throughput",x.upstream_saved);
println!("Nuclear heat lifecycle burden,ADDED,{:.3},declared nuclear heat proxy",x.nuclear_added);
println!("Incremental auxiliary electricity,ADDED,{:.3},increment above baseline process electricity",x.auxiliary_added);
println!("CO2 transport and storage lifecycle burden,ADDED,{:.3},screening transport-storage proxy",x.transport_storage_added);
println!("NET LIFECYCLE EMISSIONS SAVED,NET,{:.3},saved minus added",x.net_lifecycle_saved);}