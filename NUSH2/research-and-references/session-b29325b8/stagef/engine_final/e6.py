import sys
sys.path.insert(0, r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/engine_final")
from e1lib import rep
F = "engine/plant/src/export.rs"
rep(F, """    level4_block(rec, &mut rows);
""", """    level4_block(rec, &mut rows);
    tube_block(&mut rows)?;
""")
rep(F, """    if id.starts_with("L4_") {""", """    if id.starts_with("TUBE_") {
        return TUBE_STATUS.to_string();
    }
    if id.starts_with("L4_") {""")
rep(F, """        "model_m6" => "engine M6 column models",""", """        "model_m6" => "engine M6 column models",
        TUBE_STATUS => "engine M5 Case B tube model, HA Table 6 tube with the JAERI finned annulus",""")
rep(F, """fn level4_block(rec: &TermRecord, rows: &mut Vec<ResultRow>) {""", """/// Status of the rows of the M5 Case B tube model.
const TUBE_STATUS: &str = "model_m5";

/// The reformer exit the assumed tube reaches with helium at its 900 C supply,
/// at the HA tube count and at the upper limit of the tube-count search, and
/// the shortfall below INL's 871 C at each. The tube cannot reach 871 C within
/// the search, so these rows carry the finding (decision D29).
fn tube_block(rows: &mut Vec<ResultRow>) -> Result<()> {
    use nush2_units::tube_case_b as tube;
    let activity = nush2_units::jaeri::A2_FIT_ACTIVITY;
    let he_c = tube::HE_T_IN_K - 273.15;
    for (n, what) in [
        (tube::HA_TUBES, "the HA Table 6 tube count"),
        (
            tube::N_TUBES_SEARCH_MAX,
            "the upper limit of the tube-count search",
        ),
    ] {
        let (point, _) = tube::run(n, activity)?;
        let tag = format!("N{n:.0}");
        let mut add = |id: String, parameter: String, v: f64, unit: &'static str| {
            rows.push(ResultRow {
                id,
                group: "heat",
                parameter,
                b: Some(v),
                a_inl: None,
                a_mea: None,
                unit,
                equation: "D.25",
                decision: "D29",
            });
        };
        add(
            format!("TUBE_EXIT_C_{tag}"),
            format!(
                "Reformer process-gas exit on the assumed tube at {n:.0} tubes, {what}, helium \\
                 supply {he_c:.0} C, Case B feed; tube model run tube_case_b at JAERI A-2 \\
                 activity {activity:.3}"
            ),
            point.exit_c,
            "C",
        );
        add(
            format!("TUBE_SHORTFALL_{:.0}C_K_{tag}", REFORMER_INL_C),
            format!(
                "Shortfall of that exit below INL's {REFORMER_INL_C:.0} C reformer exit (TEV-961 \\
                 PDF p14), {n:.0} tubes"
            ),
            REFORMER_INL_C - point.exit_c,
            "K",
        );
    }
    Ok(())
}

/// INL's equilibrium reformer exit, C, the headline basis (decision D29).
const REFORMER_INL_C: f64 = flowsheet::T_REFORMER_OUT_C;

fn level4_block(rec: &TermRecord, rows: &mut Vec<ResultRow>) {""")
# units: search limit constant
rep("engine/units/src/tube_case_b.rs", """/// Case 6 helium supply and flow, TEV-961 PDF p18.""", """/// Upper limit of the tube-count search for a target exit temperature.
pub const N_TUBES_SEARCH_MAX: f64 = 5000.0;
/// Case 6 helium supply and flow, TEV-961 PDF p18.""")
print("ok")
