import json
M = []
def add(label, desc, file, old, new, **kw):
    d = dict(label=label, desc=desc, file=file, old=old, new=new); d.update(kw); M.append(d)
AB = "engine/plant/src/abatement.rs"
CA = "engine/plant/src/carbon.rs"
FS = "engine/plant/src/flowsheet.rs"
EM = "engine/plant/src/emissions.rs"
add("M4a_capture_sign_flowsheet", "negative capture efficiency in the MEA island", FS,
    "        let cap = self.eta * feed.n[CO2];\n        let mut gas = feed.n;\n        gas[CO2] -= cap;\n        let treated = Stream::gas(\"S12\", ABSORBER_OUT.t_k(), ABSORBER_OUT.p_pa(), gas);\n        // The absorber sets circulation",
    "        let cap = -self.eta * feed.n[CO2];\n        let mut gas = feed.n;\n        gas[CO2] -= cap;\n        let treated = Stream::gas(\"S12\", ABSORBER_OUT.t_k(), ABSORBER_OUT.p_pa(), gas);\n        // The absorber sets circulation")
add("M13_ts_sign", "transport and storage emission added instead of subtracted", EM,
    "            a_net: a_direct - ts_emission,", "            a_net: a_direct + ts_emission,")
add("M15_reference_carbon_drops_product", "reference carbon omits the product methane P", AB,
    "        Ok(e.baseline_central / k + c.p + (1.0 - v) * k_ref)",
    "        Ok(e.baseline_central / k + (1.0 - v) * k_ref)")
add("M18_c2_atoms", "ethane counted with one carbon atom in the ledger", CA,
    "pub const CARBON_ATOMS: [f64; NSP] = [0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 3.0, 4.0, 5.0];",
    "pub const CARBON_ATOMS: [f64; NSP] = [0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 3.0, 4.0, 5.0];")
add("M19_hours_in_k", "annual conversion uses 8760 h instead of availability hours", AB,
    "    pub fn tonnes_per_kmol_h(hours: f64) -> f64 {", "    pub fn tonnes_per_kmol_h(_hours: f64) -> f64 {\n        let hours = 8760.0;")
json.dump(M, open("mutations2.json", "w", encoding="utf-8"), indent=1)
print(len(M))
