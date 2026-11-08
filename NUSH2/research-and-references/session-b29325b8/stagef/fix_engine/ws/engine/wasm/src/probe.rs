use serde_json::{json, Value};
#[test]
fn probe_band() {
    for rating in [178.4, 179.5, 181.0, 182.0] {
        let r: Value = serde_json::from_str(&crate::dispatch(2, json!({"inputs": {"rating_mwth": rating}}).to_string().as_bytes())).unwrap();
        let e = |id: &str| r["balances"]["energy"].as_array().unwrap().iter().find(|x| x["id"] == id).unwrap()["value"].clone();
        println!("PROBE rating {rating}: feasible {} kpi unmet_power_mwe {} energy unmet_electrical {} generation {}", r["feasibility"]["feasible"], r["kpi"]["unmet_power_mwe"], e("unmet_electrical"), e("electricity_generation"));
    }
}
