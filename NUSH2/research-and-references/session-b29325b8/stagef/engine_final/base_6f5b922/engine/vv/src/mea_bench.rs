//! Solvent benchmarks of `docs/VALIDATION_MATRIX.md` sections 9 to 11: Aronu VLE,
//! Hartono physical properties and Versteeg rate constants. AR-1 runs through
//! `nush2_mea::validation`, whose 104 targets are typed separately in
//! `engine/mea/tests/data`; the row AR-1-FIXTURE checks them against this fixture.

use crate::fixture::{Fix, Fixture};
use crate::report::{Row, Status};
use crate::rowkit::{aard_pct, worst, Meta, INDEPENDENT, SAME};
use nush2_mea::aronu::{henry_water_co2_pa_m3_mol, henry_water_n2o_pa_m3_mol};
use nush2_mea::density::{c_mea_kmol_m3, density_kg_m3};
use nush2_mea::kinetics::{henry_khan_pa_m3_mol, k2_hikita};
use nush2_mea::pstar::{pstar_kpa, water_partial_pressure_pa};
use nush2_mea::validation::{
    rows_csv, run_model_points, split_csv, summarise, AARD_LIMIT_PCT, POINT_TOLERANCE,
};
use nush2_mea::viscosity::viscosity_pa_s;
use nush2_numerics::{EngineError, Result};

/// State of a fixture row: temperature K, CO2-free MEA mass fraction, loading.
fn state(fix: &Fix) -> Result<(f64, f64, f64)> {
    Ok((
        fix.cond("T_C")? + 273.15,
        fix.cond("w_MEA_mass_pct")? / 100.0,
        fix.cond("alpha_mol_mol")?,
    ))
}

/// AARD of `model` over `points`, skipping a point whose model call fails and
/// naming how many ran.
fn aard_over(
    points: &[&Fix],
    model: impl Fn(&Fix) -> Result<f64>,
    data: impl Fn(&Fix) -> f64,
) -> (Vec<(f64, f64)>, usize) {
    let mut pairs = Vec::new();
    let mut failed = 0;
    for p in points {
        match model(p) {
            Ok(m) if m.is_finite() => pairs.push((m, data(p))),
            _ => failed += 1,
        }
    }
    (pairs, failed)
}

#[allow(clippy::too_many_arguments)]
fn aard_row(
    first: &Fix,
    id: &str,
    subject: &str,
    cond: &str,
    pairs: &[(f64, f64)],
    failed: usize,
    limit: f64,
    data_use: &'static str,
    note: &str,
) -> Row {
    let a = aard_pct(pairs);
    let w = worst(pairs).unwrap_or((f64::NAN, f64::NAN));
    let ran = format!("{} of {} points ran", pairs.len(), pairs.len() + failed);
    Meta::at(first, id, 1, subject, "% AARD")
        .use_(data_use)
        .cond(format!("{cond}, {ran}"))
        .flagged(
            None,
            a,
            limit,
            a <= limit,
            format!(
                "{note}. Limit {limit}% AARD. Worst point: model {:.5} against {:.5} {}",
                w.0, w.1, first.unit
            ),
        )
}

fn first<'a>(v: &[&'a Fix], id: &str) -> Result<&'a Fix> {
    v.first()
        .copied()
        .ok_or_else(|| EngineError::Parse(format!("{id}: no fixture points")))
}

/// The 104 AR-1 point rows as `nush2_mea::validation::rows_csv` writes them.
fn aronu_point_rows() -> Result<(Vec<Row>, Vec<nush2_mea::validation::PointResult>)> {
    let results = run_model_points()?;
    let text = rows_csv(&results);
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let c = split_csv(line);
        if c.len() != 16 {
            return Err(EngineError::Parse(format!(
                "Aronu row with {} cells",
                c.len()
            )));
        }
        let f = |i: usize| -> Result<f64> {
            c[i].parse()
                .map_err(|_| EngineError::Parse(format!("Aronu row cell {i}: {}", c[i])))
        };
        out.push(Row {
            test_id: c[0].clone(),
            level: 1,
            subject: c[2].clone(),
            source: c[3].clone(),
            locator: c[4].clone(),
            conditions: c[5].clone(),
            literature: Some(f(6)?),
            simulated: f(7)?,
            unit: c[8].clone(),
            abs_error: Some(f(9)?),
            rel_error: Some(f(10)?),
            tolerance: Some(f(11)?),
            status: match c[12].as_str() {
                "pass" => Status::Pass,
                "fail" => Status::Fail,
                other => return Err(EngineError::Parse(format!("Aronu status {other}"))),
            },
            data_use: SAME,
            explanation: format!(
                "{}. Limit {:.1}% relative to the model value",
                c[14].trim_end_matches('.'),
                100.0 * POINT_TOLERANCE
            ),
        });
    }
    Ok((out, results))
}

/// AR-1 to AR-4.
pub fn aronu(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("aronu_2011.csv")?;
    let (points, results) = aronu_point_rows()?;
    out.extend(points);
    let s = summarise(&results)?;
    let model_rows = f.select(|r| r.quantity == "pCO2_model");
    let ptot_rows = f.select(|r| r.quantity == "Ptot");
    let fix0 = first(&model_rows, "AR-1")?;
    out.push(
        Meta::at(
            fix0,
            "AR-1",
            1,
            "Published pCO2 model column, extended UNIQUAC port",
            "% AARD",
        )
        .use_(SAME)
        .locator("Tables 3, 5 (corrigendum PDF p16, p17), Tables 4, 6 (PDF p5, p6)")
        .cond(format!(
            "{} points, 15 to 60 mass%, 60 to 120 C, SRK and Poynting at the printed Ptot",
            s.n
        ))
        .flagged(
            None,
            s.aard_pct,
            AARD_LIMIT_PCT,
            s.aard_pct <= AARD_LIMIT_PCT && s.failed_points == 0,
            format!(
                "Limit AARD {AARD_LIMIT_PCT}% and every point within {:.1}% (rows \
                     ARONU_104_001 to 104). Worst point {:.3}%, {} points over. AARD before the \
                     gas corrections {:.3}%",
                100.0 * POINT_TOLERANCE,
                s.max_abs_dev_pct,
                s.failed_points,
                s.aard_ideal_pct
            ),
        ),
    );

    let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(1.0);
    let mut unmatched = Vec::new();
    for r in &results {
        let p = &r.point;
        let hit = model_rows.iter().find(|m| {
            state(m)
                .is_ok_and(|(t, w, a)| close(t, p.t_k) && close(w, p.w_mea) && close(a, p.alpha))
                && close(m.value, p.pco2_model_kpa)
                && !m.has("dup_printed", "1")
                && ptot_rows
                    .iter()
                    .any(|q| q.series == m.series && close(q.value, p.ptot_kpa))
        });
        if hit.is_none() {
            unmatched.push(format!("{} {:.2} K alpha {}", p.table, p.t_k, p.alpha));
        }
    }
    let dup: Vec<String> = model_rows
        .iter()
        .filter(|m| m.has("dup_printed", "1"))
        .map(|m| {
            let ptot = ptot_rows
                .iter()
                .find(|q| q.series == m.series)
                .map_or(f64::NAN, |q| q.value);
            format!("{} at Ptot {ptot} kPa, pCO2 model {}", m.series, m.value)
        })
        .collect();
    out.push(
        Meta::at(
            fix0,
            "AR-1-FIXTURE",
            1,
            "AR-1 targets in engine/mea against this fixture",
            "points unmatched",
        )
        .use_(SAME)
        .locator("Tables 3 to 6")
        .cond(format!("{} targets", results.len()))
        .flagged(
            Some(0.0),
            unmatched.len() as f64,
            0.0,
            unmatched.is_empty(),
            format!(
                "Each target needs a fixture row with the same state, model value and Ptot. \
                     Excluded as the printed repeat: {}. {}",
                dup.join("; "),
                if unmatched.is_empty() {
                    String::new()
                } else {
                    format!("Unmatched: {}", unmatched.join("; "))
                }
            )
            .trim_end()
            .to_string(),
        ),
    );

    let pco2 = |fix: &Fix| -> Result<f64> {
        let (t, w, a) = state(fix)?;
        pstar_kpa(t, w, a)
    };
    let measured = |loc: Option<&str>| {
        f.select(move |r| {
            r.quantity == "pCO2"
                && r.data_kind == "measured"
                && loc.is_none_or(|l| r.locator.starts_with(l))
        })
    };
    let t4 = measured(Some("Table 4"));
    let (pairs, failed) = aard_over(&t4, pco2, |p| p.value);
    out.push(aard_row(
        first(&t4, "AR-2")?,
        "AR-2",
        "Measured pCO2 against p*",
        "30 mass%, 40 to 80 C",
        &pairs,
        failed,
        25.0,
        SAME,
        "Ideal p* of the extended UNIQUAC port, the quantity the absorber uses",
    ));
    let all = measured(None);
    let (pairs, failed) = aard_over(&all, pco2, |p| p.value);
    out.push(aard_row(
        first(&all, "AR-3")?,
        "AR-3",
        "Measured pCO2 against p*, all strengths",
        "15 to 60 mass%, 40 to 80 C, Tables 3 to 6",
        &pairs,
        failed,
        25.0,
        SAME,
        "Aronu reports 24.3% for the model (Abstract PDF p1)",
    ));
    let ptot: Vec<&Fix> = ptot_rows
        .iter()
        .copied()
        .filter(|r| r.data_kind == "measured" && !r.has("dup_printed", "1"))
        .collect();
    let (pairs, failed) = aard_over(
        &ptot,
        |fix| {
            let (t, w, a) = state(fix)?;
            Ok(pstar_kpa(t, w, a)? + water_partial_pressure_pa(t, w, a)? / 1000.0)
        },
        |p| p.value,
    );
    out.push(aard_row(
        first(&ptot, "AR-4")?,
        "AR-4",
        "Measured total pressure against p* plus water",
        "15 to 60 mass%, 60 to 120 C, Tables 3 to 6, the printed repeat excluded",
        &pairs,
        failed,
        12.0,
        SAME,
        "MEA vapour pressure left out. Aronu reports 11.7% (Abstract PDF p1)",
    ));
    Ok(())
}

/// HA-1 to HA-3.
pub fn hartono(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("hartono_2014.csv")?;
    let at30 = |q: &'static str, loc: &'static str| {
        f.select(move |r| {
            r.quantity == q && r.locator.starts_with(loc) && r.has("w_MEA_mass_pct", "30")
        })
    };
    let rho = at30("density", "Table 3");
    let (pairs, failed) = aard_over(
        &rho,
        |fix| {
            let (t, w, a) = state(fix)?;
            density_kg_m3(t, w, a)
        },
        |p| p.value,
    );
    out.push(aard_row(
        first(&rho, "HA-1")?,
        "HA-1",
        "Density, Weiland correlation",
        "30 mass%, 20 to 80 C, loading 0 to 0.50",
        &pairs,
        failed,
        1.0,
        INDEPENDENT,
        "Out of sample: the engine uses Weiland",
    ));
    let mu = at30("viscosity", "Table 7");
    let (pairs, failed) = aard_over(
        &mu,
        |fix| {
            let (t, w, a) = state(fix)?;
            Ok(viscosity_pa_s(t, w, a)? * 1.0e3)
        },
        |p| p.value,
    );
    out.push(aard_row(
        first(&mu, "HA-2")?,
        "HA-2",
        "Viscosity, Weiland correlation",
        "30 mass%, 20 to 80 C, loading 0 to 0.50, two repeats",
        &pairs,
        failed,
        3.0,
        INDEPENDENT,
        "Out of sample: the engine uses Weiland",
    ));
    let kh = at30("kH_N2O", "Table 11");
    let (pairs, failed) = aard_over(
        &kh,
        |fix| {
            let (t, w, a) = state(fix)?;
            henry_khan_pa_m3_mol(t, c_mea_kmol_m3(t, w, a)?)
        },
        |p| {
            let t = p.cond("T_C").unwrap_or(f64::NAN) + 273.15;
            p.value * 1.0e3 * henry_water_co2_pa_m3_mol(t) / henry_water_n2o_pa_m3_mol(t)
        },
    );
    out.push(aard_row(
        first(&kh, "HA-3")?,
        "HA-3",
        "CO2 Henry constant, Khan Eq 8 against N2O data by the N2O analogy",
        "30 mass%, 25.2 to 87.1 C, loading 0 to 0.50",
        &pairs,
        failed,
        30.0,
        INDEPENDENT,
        "Values in Pa m3/mol after the analogy of Aronu Eq 19 (PDF p4). Khan Eq 8 has no \
         loading term",
    ));
    Ok(())
}

/// VER-1 to VER-3.
pub fn versteeg(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("versteeg_1988.csv")?;
    let c = f.v("bronsted", "intercept")?;
    let ta = f.v("bronsted", "T_a")?;
    for fix in f.select(|r| r.quantity == "k2") {
        let t = fix.cond("T_K")?;
        let lab = fix.cond_text("ref")?;
        let model = k2_hikita(t)? / 1000.0;
        if lab == "Hikita1977a" {
            out.push(
                Meta::at(
                    fix,
                    format!("VER-1-{t:.0}K"),
                    1,
                    "k2, Hikita rows",
                    "m3/mol/s",
                )
                .use_(SAME)
                .cond(format!("MEA, {t} K"))
                .judged(
                    fix.value,
                    model,
                    0.005 * fix.value,
                    "Limit 0.5%. Khan Eq 2 is Hikita's correlation, so these are its own data",
                ),
            );
            let pka = f.v(&fix.series, "pKa")?;
            let eq20 = (pka + c - ta / t).exp();
            out.push(
                Meta::at(
                    fix,
                    format!("VER-3-{t:.0}K"),
                    1,
                    "k2 against the Bronsted relation",
                    "m3/mol/s",
                )
                .locator("Eq 20 PDF p12")
                .cond(format!("MEA, {t} K, pKa {pka} from the Hikita row"))
                .judged(
                    eq20,
                    model,
                    0.2 * eq20,
                    format!("Literature value from Eq 20, ln k2 = pKa + {c} - {ta}/T. Limit 20%"),
                ),
            );
        } else {
            out.push(
                Meta::at(
                    fix,
                    format!("VER-2-{t:.0}K-{lab}"),
                    1,
                    "k2, other laboratories",
                    "m3/mol/s",
                )
                .cond(format!("MEA, {t} K, {lab}"))
                .judged(
                    fix.value,
                    model,
                    0.2 * fix.value,
                    "Limit 20%, about two standard deviations of the printed values",
                ),
            );
        }
    }
    Ok(())
}
