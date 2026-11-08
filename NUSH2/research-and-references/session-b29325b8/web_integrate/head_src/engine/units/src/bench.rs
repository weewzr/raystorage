//! Benchmark rows of the gas units and the reformer tube, for the V&V harness to
//! carry into `results/Validation_Results.csv`. Every row comes from a comparison
//! that ran in this call, and a row with `pass` of `None` is reported with no limit.
//!
//! Test ids follow `docs/VALIDATION_MATRIX.md`. The JAERI ids are new, since that
//! file leaves the source to M5. The M6 column benchmarks live in `columns`.

mod columns;

pub use columns::{
    absorber_case_b_spec, absorber_khan_table1, absorber_salvinder_85pct,
    absorber_sonderby_profiles, absorber_sonderby_runs, absorber_tan_trend, column_gate,
    column_rows, column_terms, stripper_dice, stripper_kwak, stripper_morgan, stripper_vinjarapu,
    unread_fixture_rows, COLUMN_FIXTURES,
};

use crate::case5::{self, TrainOpts};
use crate::jaeri::{self, Run, A2, A2_FIT_ACTIVITY, A3, A4, A5};
use crate::stream::Sp;
use crate::tube_case_b::{self, CasePoint};
use crate::xf;
use nush2_numerics::Result;
use nush2_props::equilibrium::KSource;
use nush2_props::mixture;
use nush2_props::units::{mmbtu_h_to_mw, LB};

const INDEPENDENT: &str = "independent";
const CALIBRATION: &str = "calibration";
const SAME: &str = "same_dataset_disclosed";
const TEV961: &str = "INL TEV-961 Rev. 0 2010";
const XF1: &str = "Xu and Froment 1989 Part I";

/// One comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchRow {
    pub test_id: String,
    pub level: u8,
    pub subject: String,
    pub source: String,
    pub locator: String,
    pub conditions: String,
    pub literature: Option<f64>,
    pub simulated: f64,
    pub unit: String,
    pub tolerance: Option<f64>,
    pub pass: Option<bool>,
    pub data_use: &'static str,
    pub explanation: String,
}

impl BenchRow {
    /// Absolute gap, when a literature value exists.
    pub fn abs_error(&self) -> Option<f64> {
        self.literature.map(|l| (self.simulated - l).abs())
    }

    /// Status in the terms of `results/Validation_Results.csv`: `pass` or `fail`
    /// for a row judged against a limit, `not_run` for a row whose simulated value
    /// is missing, and `reported` for a row printed with no limit.
    pub fn status(&self) -> &'static str {
        if !self.simulated.is_finite() {
            return "not_run";
        }
        match self.pass {
            Some(true) => "pass",
            Some(false) => "fail",
            None => "reported",
        }
    }
}

/// Identity of a row.
struct Id<'a> {
    test_id: &'a str,
    subject: &'a str,
    source: &'a str,
    locator: &'a str,
    conditions: &'a str,
    unit: &'a str,
    data_use: &'static str,
}

fn row(id: &Id<'_>, lit: f64, sim: f64, tol: Option<f64>, note: String) -> BenchRow {
    BenchRow {
        test_id: id.test_id.to_string(),
        level: 2,
        subject: id.subject.to_string(),
        source: id.source.to_string(),
        locator: id.locator.to_string(),
        conditions: id.conditions.to_string(),
        literature: Some(lit),
        simulated: sim,
        unit: id.unit.to_string(),
        tolerance: tol,
        pass: tol.map(|t| (sim - lit).abs() <= t),
        data_use: id.data_use,
        explanation: note,
    }
}

fn c_of_f(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8
}

fn train_rows(out: &mut Vec<BenchRow>, suffix: &str, t: &case5::Train, note: &str) -> Result<()> {
    let hts_t = t.hts.outlet.t_k - 273.15;
    let hts_co = 100.0 * t.hts.outlet.dry_fraction(Sp::Co)?;
    let base = Id {
        test_id: "",
        subject: "",
        source: TEV961,
        locator: "A-53 PDF p77",
        conditions: "Case 5 raw syngas, HTS inlet 635 F",
        unit: "",
        data_use: INDEPENDENT,
    };
    let id_t = format!("T961-5{suffix}-T");
    let id_co = format!("T961-5{suffix}-CO");
    out.push(row(
        &Id {
            test_id: &id_t,
            subject: "WGS-HTS outlet temperature",
            unit: "C",
            ..base
        },
        c_of_f(case5::HTS_OUT_T_F),
        hts_t,
        Some(10.0),
        note.to_string(),
    ));
    out.push(row(
        &Id {
            test_id: &id_co,
            subject: "WGS-HTS outlet CO, dry",
            unit: "mol%",
            ..base
        },
        case5::HTS_OUT_CO_DRY_PCT,
        hts_co,
        Some(0.3),
        note.to_string(),
    ));
    Ok(())
}

fn m4_rows(out: &mut Vec<BenchRow>) -> Result<()> {
    let rf = case5::reformer(KSource::Zhang)?;
    let y = mixture::mole_fractions(&rf.outlet.n)?;
    for (sp, pct) in case5::RAW_MOL_PCT {
        let id = format!("T961-3-{sp:?}");
        out.push(row(
            &Id {
                test_id: &id,
                subject: "Raw syngas, wet",
                source: TEV961,
                locator: "A-50 PDF p74",
                conditions: "Case 5 NG-10 feed, 1600 F, 439 psia, recycle composition calculated",
                unit: "mol%",
                data_use: INDEPENDENT,
            },
            pct,
            100.0 * y[sp as usize],
            Some(0.5),
            "SMR-EQ on the reconstructed NG-10 feed".to_string(),
        ));
    }
    let want = mmbtu_h_to_mw(case5::DUTY_REFORM_1 + case5::DUTY_REFORM_2);
    out.push(row(
        &Id {
            test_id: "T961-4",
            subject: "Reformer enthalpy rise",
            source: TEV961,
            locator: "A-55 PDF p79",
            conditions: "1000 F to 1600 F, 349724 lb/h",
            unit: "MW",
            data_use: INDEPENDENT,
        },
        want,
        rf.duty_w / 1.0e6,
        Some(0.03 * want),
        "REFORM-1 plus burner duty".to_string(),
    ));

    let raw = case5::raw_syngas()?;
    let sized = case5::train(&raw, &TrainOpts::sized_on_ieaghg(KSource::Zhang)?)?;
    train_rows(
        out,
        "",
        &sized,
        "Bed sized to the Q/K of the printed IEAGHG HTS outlet, an independent flowsheet",
    )?;
    let first = case5::train(&raw, &TrainOpts::chinchen_space_velocity(KSource::Zhang))?;
    train_rows(
        out,
        "-GHSV",
        &first,
        "First basis tried: Chinchen's 5000 per hour test space velocity, kept as run",
    )?;

    let t = &sized;
    let lts_id =
        |test_id: &'static str, subject: &'static str, unit: &'static str| (test_id, subject, unit);
    for ((test_id, subject, unit), lit, sim, tol) in [
        (
            lts_id("T961-6-T", "WGS-LTS outlet temperature", "C"),
            c_of_f(case5::LTS_OUT_T_F),
            t.lts.outlet.t_k - 273.15,
            10.0,
        ),
        (
            lts_id("T961-6-CO", "WGS-LTS outlet CO, dry", "mol%"),
            case5::LTS_OUT_CO_DRY_PCT,
            100.0 * t.lts.outlet.dry_fraction(Sp::Co)?,
            0.1,
        ),
    ] {
        out.push(row(
            &Id {
                test_id,
                subject,
                source: TEV961,
                locator: "A-53 PDF p77",
                conditions: "LTS inlet 375 F, Zhang K2",
                unit,
                data_use: INDEPENDENT,
            },
            lit,
            sim,
            Some(tol),
            "10 K approach, no Cu/Zn kinetics in the library".to_string(),
        ));
    }

    for (name, unit, printed) in [
        ("HPS-GEN1", &t.hps_gen1, case5::DUTY_HPS_GEN1),
        ("WGS-EX1", &t.ex1, case5::DUTY_WGS_EX1),
        ("WGS-EX2", &t.ex2, case5::DUTY_WGS_EX2),
        ("WGS-EX3", &t.ex3, case5::DUTY_WGS_EX3),
        ("WGS-EX4", &t.ex4, case5::DUTY_WGS_EX4),
        ("KO-DRM-1", &t.ko, case5::DUTY_KO_DRM),
    ] {
        let want = mmbtu_h_to_mw(printed);
        let id = format!("T961-7-{name}");
        out.push(row(
            &Id {
                test_id: &id,
                subject: "Cooler duty",
                source: TEV961,
                locator: "A-55 PDF p79, A-56 PDF p80",
                conditions: "Case 5 outlet temperatures as printed, ideal vapour",
                unit: "MW",
                data_use: INDEPENDENT,
            },
            want,
            unit.duty_w / 1.0e6,
            Some(0.10 * want),
            "Gate of term T05".to_string(),
        ));
    }

    let got = t.drum.liquid.as_ref().map_or(Ok(0.0), |l| l.mass_kg_s())? * 3.6;
    let want = case5::KO_LIQ_LB_H * LB / 1000.0;
    let base = Id {
        test_id: "T961-8-LIQ",
        subject: "Condensate",
        source: TEV961,
        locator: "A-56 PDF p80",
        conditions: "KO drum at 104 F, 389 psia",
        unit: "t/h",
        data_use: INDEPENDENT,
    };
    out.push(row(&base, want, got, Some(0.03 * want), String::new()));
    out.push(row(
        &Id {
            test_id: "T961-8-CO2",
            subject: "Dry CO2 leaving the drum",
            locator: "A-53 PDF p77",
            unit: "mol%",
            ..base
        },
        case5::SELEXOL_IN_CO2_PCT,
        100.0 * t.drum.vapour.dry_fraction(Sp::Co2)?,
        Some(0.3),
        String::new(),
    ));
    Ok(())
}

fn m5_table_rows(out: &mut Vec<BenchRow>) {
    for (row5, ratio) in xf::TABLE5.iter().zip(xf::table5_over_table6()) {
        let id = format!("XF-1-{}", row5.name);
        out.push(BenchRow {
            test_id: id,
            level: 2,
            subject: "Table 5 carried to the Table 6 form".to_string(),
            source: XF1.to_string(),
            locator: "Tables 5 and 6 PDF p7".to_string(),
            conditions: format!("Reference {} K", row5.t_ref_k),
            literature: Some(1.0),
            simulated: ratio,
            unit: "ratio".to_string(),
            tolerance: None,
            pass: Some((0.986..=1.004).contains(&((ratio * 1000.0).round() / 1000.0))),
            data_use: SAME,
            explanation: format!(
                "Plan limits 0.986 to 1.004, printed to 3 decimals, so the ratio is judged \
                 rounded to 3 decimals: {:.3}. Unrounded ratio {ratio:.6}{}",
                (ratio * 1000.0).round() / 1000.0,
                if (0.986..=1.004).contains(&ratio) {
                    ""
                } else {
                    ", outside the limits before rounding"
                }
            ),
        });
    }
}

fn jaeri_rows(out: &mut Vec<BenchRow>, r: &Run, data_use: &'static str) -> Result<()> {
    let t = jaeri::run(r, A2_FIT_ACTIVITY)?;
    let conditions = format!(
        "Activity {A2_FIT_ACTIVITY}, calculated U, loss from the A-2 balance{}",
        if r.inlet_assumed {
            ", inlet set from Yin, helium flow and heater assumed from A-2"
        } else {
            ""
        }
    );
    let tuned = if data_use == CALIBRATION {
        "tuned = true: this run set the activity"
    } else {
        "held back from the fit"
    };
    let id = format!("JA-{}-X", r.id);
    let base = Id {
        test_id: &id,
        subject: "CH4 conversion",
        source: r.source,
        locator: "appendix sheet",
        conditions: &conditions,
        unit: "-",
        data_use,
    };
    out.push(row(
        &base,
        r.conversion(),
        t.conversion,
        Some(0.03),
        tuned.to_string(),
    ));
    for (i, name) in ["S1", "S2", "S3", "S4", "S5"].iter().enumerate() {
        let n = t.at(jaeri::station_z_m(i));
        for (quantity, lit, sim) in [
            ("process", r.process_c[i], n.t_k - 273.15),
            ("helium", r.he_c[i], n.t_he_k - 273.15),
            ("return", r.return_c[i], n.t_ret_k - 273.15),
        ] {
            if lit.is_nan() {
                continue;
            }
            let id = format!("JA-{}-{name}-{quantity}", r.id);
            out.push(row(
                &Id {
                    test_id: &id,
                    subject: "Station temperature",
                    unit: "C",
                    ..base
                },
                lit,
                sim,
                None,
                format!("{quantity} gas at {:.2} m, reported", jaeri::station_z_m(i)),
            ));
        }
    }
    if !r.q_return_kw.is_nan() {
        let id = format!("JA-{}-Qret", r.id);
        out.push(row(
            &Id {
                test_id: &id,
                subject: "Heat handed back by the return tube",
                unit: "kW",
                ..base
            },
            r.q_return_kw,
            t.return_heat_w / 1.0e3,
            None,
            "JAERI's calculated K12 falls short of the measured return duty".to_string(),
        ));
        let id = format!("JA-{}-Qproc", r.id);
        out.push(row(
            &Id {
                test_id: &id,
                subject: "Net heat into the process gas",
                unit: "kW",
                ..base
            },
            r.q_sensible_kw + r.q_reaction_kw - r.q_return_kw,
            t.process_heat_w / 1.0e3,
            None,
            "Printed sensible plus reaction heat, less the return duty".to_string(),
        ));
    }
    if r.id == A2.id {
        let end = t.nodes.last().map_or(r.p_in_pa, |n| n.p_pa);
        let id = format!("JA-{}-dP", r.id);
        out.push(row(
            &Id {
                test_id: &id,
                subject: "Process pressure drop, feed to tip",
                unit: "kPa",
                ..base
            },
            jaeri::A2_DP_GAUGES_PA / 1.0e3,
            (r.p_in_pa - end) / 1.0e3,
            None,
            format!(
                "Ergun on the bed only, return tube left out. Gauges PT19G less PT2C; the \
                 differential gauge DT20G reads {:.0} kPa",
                jaeri::A2_DP_DIFFERENTIAL_PA / 1.0e3
            ),
        ));
    }
    Ok(())
}

/// Case B points, printed for the report. All tube geometry is assumed.
pub fn case_b_points(activity: f64) -> Result<Vec<CasePoint>> {
    let mut out = Vec::new();
    for n in [tube_case_b::HA_TUBES, 300.0, 450.0, 700.0, 1000.0, 2500.0] {
        out.push(tube_case_b::run(n, activity)?.0);
    }
    Ok(out)
}

/// Every row of M4 and M5.
pub fn rows() -> Result<Vec<BenchRow>> {
    let mut out = Vec::new();
    m4_rows(&mut out)?;
    m5_table_rows(&mut out);
    jaeri_rows(&mut out, &A2, CALIBRATION)?;
    jaeri_rows(&mut out, &A3, INDEPENDENT)?;
    jaeri_rows(&mut out, &A4, INDEPENDENT)?;
    jaeri_rows(&mut out, &A5, INDEPENDENT)?;
    Ok(out)
}

/// Modelled values of the screening terms M4 fills. T03 stays anchored by plan, so
/// its field carries the Case 5 check value and no replacement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasTerms {
    /// T01, CO2 / (CO2 + CH4 + CO) after the shift, full chain from SMR-EQ.
    pub t01_phi: f64,
    /// T03 check, Case 5 reformer enthalpy rise, MW.
    pub t03_case5_reformer_mw: f64,
    /// T05 on Case 5, heat from 149 C to 130 C, MW, before the plant's CO2 scale.
    pub t05_case5_mw: f64,
}

/// Run the chain from SMR-EQ and return the term values.
pub fn gas_terms() -> Result<GasTerms> {
    let rf = case5::reformer(KSource::Zhang)?;
    let t = case5::train(&rf.outlet, &TrainOpts::sized_on_ieaghg(KSource::Zhang)?)?;
    Ok(GasTerms {
        t01_phi: case5::phi_after_shift(&t.lts.outlet),
        t03_case5_reformer_mw: rf.duty_w / 1.0e6,
        t05_case5_mw: case5::syngas_above_130c_w(&t)? / 1.0e6,
    })
}
