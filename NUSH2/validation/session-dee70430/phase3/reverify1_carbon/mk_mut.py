import json
M = []
def add(label, desc, file, old, new, **kw):
    d = dict(label=label, desc=desc, file=file, old=old, new=new); d.update(kw); M.append(d)

AB = "engine/plant/src/abatement.rs"
CA = "engine/plant/src/carbon.rs"
FS = "engine/plant/src/flowsheet.rs"
EM = "engine/plant/src/emissions.rs"
EX = "engine/plant/src/export.rs"

add("M1a_drop_condensate_stream", "drop the vented condensate CO2 from the atmospheric carbon of the plant", AB,
    "            c.burn = a.burned * k + vented;\n            c.exported = 0.0;",
    "            c.burn = a.burned * k;\n            c.exported = 0.0;")
add("M1b_drop_product_from_account", "drop the PRODUCT carbon stream from the closure account", CA,
    '            known(\n                "PRODUCT",\n                "methane in the hydrogen product, S13",\n                false,\n                self.product,\n                false,\n            ),\n', "")
add("M2_unknown_set_to_zero", "enter the unknown CO2 vent U2 as a known zero", CA,
    '            unknown("U2", "CO2 vented between S19 and S20", false, true),',
    '            known("U2", "CO2 vented between S19 and S20", false, 0.0, true),')
add("M3_swap_baseline", "central IEAGHG baseline reads the Cho intensity", AB,
    "            BaselineKind::Ieaghg => self.base_ieaghg,",
    "            BaselineKind::Ieaghg => self.base_cho,")
add("M4a_capture_sign_flowsheet", "negative capture efficiency in the MEA island", FS,
    "        let cap = self.eta * feed.n[CO2];\n        let mut gas = feed.n;\n        gas[CO2] -= cap;",
    "        let cap = -self.eta * feed.n[CO2];\n        let mut gas = feed.n;\n        gas[CO2] -= cap;")
add("M4b_capture_sign_closed_form", "sign of the capture term in the loop denominator D", AB,
    "        1.0 - f * (1.0 - kappa - eta * phi)",
    "        1.0 - f * (1.0 - kappa + eta * phi)")
add("M5_double_count_condensate", "condensate CO2 counted twice in the atmospheric carbon", AB,
    "            c.burn = a.burned * k + vented;\n            c.exported = 0.0;",
    "            c.burn = a.burned * k + vented + a.condensate * k;\n            c.exported = 0.0;")
add("M6_baseline_sign", "direct abatement adds the plant emission to the baseline", EM,
    "        let a_direct = baseline_central - atm;",
    "        let a_direct = baseline_central + atm;")
add("M7_verdict_closed_with_unknowns", "overall closure verdict ignores unknown streams", CA,
    "    let overall = if unknown.is_empty() {",
    "    let overall = if true {")
add("M8_drop_co2e_label", "upstream credit row loses its CO2e label", EX,
    '"Saved upstream gas, CO2e (GWP100 = {}, IEA 2026, global gas supply mix)",',
    '"Saved upstream gas (GWP100 = {}, IEA 2026, global gas supply mix)",')
add("M9_gas_back_to_inl", "reference gas reads INL conventional-plant gas again (regression of finding 1)", AB,
    "        if c.src != CarbonSrc::Plant {\n            let h_s = s * c.h_scale;\n            return Ok(self.ng_conv * h_s);\n        }",
    "        if true {\n            let h_s = s * c.h_scale;\n            return Ok(self.ng_conv * h_s);\n        }")
add("M10_ts_terminal_dropped", "terminal share dropped from the T&S sum", AB,
    "            terminal: t.terminal,", "            terminal: 0.0,")
add("M11_kappa_zero", "condensate carbon share silently zero", AB,
    "            kappa: l.condensate / l.tube,", "            kappa: 0.0,")
add("M12_ts_ship_units", "ship term unit error (multiply instead of divide by 1000)", AB,
    "ship: t.ship_g_t_km * t.dist_km / 1000.0 * kg_co2_per_kg_fuel / 1000.0,",
    "ship: t.ship_g_t_km * t.dist_km / 1000.0 * kg_co2_per_kg_fuel * 1000.0,")
json.dump(M, open("mutations.json", "w", encoding="utf-8"), indent=1)
print(len(M))
