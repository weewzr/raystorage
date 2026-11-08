//! Rows of the unit benchmarks in `nush2_units::bench` (M4, M5 and M6), carried
//! into the results schema, and the LTS sensitivity on the NIST Gibbs constant.

use crate::report::{Row, Status};
use crate::rowkit::{Meta, INDEPENDENT, PSI_PA};
use nush2_numerics::Result;
use nush2_props::equilibrium::KSource;
use nush2_units::bench::{self, BenchRow};
use nush2_units::case5::{self, TrainOpts};
use nush2_units::stream::Sp;
use nush2_units::wgs::{self, LtsSpec};

/// One unit benchmark row in the results schema. Status follows
/// `BenchRow::status`, so a row with no limit stays `reported`.
pub fn to_row(b: BenchRow) -> Row {
    let status = match b.status() {
        "pass" => Status::Pass,
        "fail" => Status::Fail,
        "not_run" => Status::NotRun,
        _ => Status::Reported,
    };
    let abs = b.abs_error().filter(|a| a.is_finite());
    let rel = match (abs, b.literature) {
        (Some(a), Some(l)) if l != 0.0 => Some(a / l.abs()),
        _ => None,
    };
    let explanation = if status == Status::NotRun && !b.explanation.starts_with("not_run") {
        format!("not_run: {}", b.explanation)
    } else {
        b.explanation
    };
    Row {
        test_id: b.test_id,
        level: b.level,
        subject: b.subject,
        source: b.source,
        locator: b.locator,
        conditions: b.conditions,
        literature: b.literature,
        simulated: b.simulated,
        unit: b.unit,
        abs_error: abs,
        rel_error: rel,
        tolerance: b.tolerance,
        status,
        data_use: b.data_use,
        explanation,
    }
}

/// M4 and M5 rows: gas units, the reformer tube and the JAERI runs.
pub fn gas_rows() -> Result<Vec<Row>> {
    Ok(bench::rows()?.into_iter().map(to_row).collect())
}

/// M6 rows: absorber and stripper. These take minutes in a debug build.
pub fn column_rows() -> Result<Vec<Row>> {
    Ok(bench::column_rows()?.into_iter().map(to_row).collect())
}

/// The LTS of the Case 5 chain rerun on the NIST Gibbs K2, the HTS left on Zhang.
/// A sensitivity: the default stays Zhang because T961-6 passes with it.
pub fn lts_nist_sensitivity() -> Result<Row> {
    let t = case5::train(
        &case5::raw_syngas()?,
        &TrainOpts::sized_on_ieaghg(KSource::Zhang)?,
    )?;
    let nist = wgs::lts(
        &t.ex2.outlet,
        &LtsSpec {
            tag: "WGS-LTS".into(),
            approach_k: wgs::LTS_APPROACH_K,
            p_out_pa: 396.0 * PSI_PA,
            k_source: KSource::NistGibbs,
            outlet_tag: "S09".into(),
        },
    )?;
    let co = 100.0 * nist.outlet.dry_fraction(Sp::Co)?;
    let zhang = 100.0 * t.lts.outlet.dry_fraction(Sp::Co)?;
    let lit = case5::LTS_OUT_CO_DRY_PCT;
    Ok(Meta::new(
        "T961-6-CO-NIST",
        2,
        "WGS-LTS outlet CO, dry, NIST Gibbs K2",
        "INL TEV-961 Rev. 0 2010",
        "A-53 PDF p77",
    )
    .unit("mol%")
    .use_(INDEPENDENT)
    .cond(format!(
        "LTS inlet 375 F, NIST Gibbs K2 at the LTS only, HTS on Zhang, outlet {:.1} C",
        nist.outlet.t_k - 273.15
    ))
    .reported(
        Some(lit),
        co,
        format!(
            "Sensitivity row. On the default Zhang K2, T961-6-CO reads {zhang:.3} ({}). On the \
             NIST constant the gap is {:.3} points against the 0.1 point limit of T961-6-CO",
            if (zhang - lit).abs() <= 0.1 {
                "inside its limit"
            } else {
                "outside its limit"
            },
            (co - lit).abs()
        ),
    ))
}
