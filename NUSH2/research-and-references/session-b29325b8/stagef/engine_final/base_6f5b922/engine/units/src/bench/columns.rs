//! M6 benchmarks of the absorber and the stripper. Every literature value and
//! every input printed by a source is read from `engine/vv/data/benchmarks/`; the
//! assumptions a source leaves open are stated in each row's explanation.
//!
//! A row that cannot run carries `simulated = NaN`, `pass = None` and an
//! explanation that starts with "not_run:". A row with `tolerance = None` is
//! reported with no limit. Tolerances are the frozen values of
//! `docs/VALIDATION_MATRIX.md` (KH, SA, VI) or the plan's section 12 targets.

use super::BenchRow;
use crate::absorber::{self, AbsorberResult, AbsorberSpec, GasFilm};
use crate::columns;
use crate::packing::{Structured, FLEXIPAC_1Y, RSP_250};
use crate::regen;
use crate::solvent::{Solvent, MW_CO2};
use crate::stream::{Evidence, SolventStream, Sp, Stream, NSP};
use crate::stripper::{self, StripMode, StripperResult, StripperSpec};
use nush2_mea::aronu::{MW_MEA, MW_WATER};
use nush2_mea::validation::split_csv;
use nush2_numerics::{EngineError, Result};
use nush2_props::steam::psat_pa;
use nush2_props::units::{mmbtu_h_to_mw, LB};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

const KHAN: &str = include_str!("../../../vv/data/benchmarks/khan_2011.csv");
const SALVINDER: &str = include_str!("../../../vv/data/benchmarks/salvinder_2018.csv");
const TAN: &str = include_str!("../../../vv/data/benchmarks/tan_2015.csv");
const SONDERBY: &str = include_str!("../../../vv/data/benchmarks/sonderby_2013.csv");
const DICE: &str = include_str!("../../../vv/data/benchmarks/dice_2020.csv");
const KWAK: &str = include_str!("../../../vv/data/benchmarks/kwak_2012.csv");
const MORGAN: &str = include_str!("../../../vv/data/benchmarks/morgan_2018.csv");
const VINJ: &str = include_str!("../../../vv/data/benchmarks/vinjarapu_2024_parametric.csv");

const INDEPENDENT: &str = "independent";
const SAME: &str = "same_dataset_disclosed";
/// Standard atmosphere, Pa, for gauge pressures and standard volumes.
const ATM_PA: f64 = 101_325.0;
/// 0 C, the standard temperature assumed for "slpm" and Sm3.
const T_STD_K: f64 = 273.15;
/// One kgf/cm2 in Pa (standard gravity).
const KGF_CM2_PA: f64 = 98_066.5;
const R: f64 = nush2_mea::pstar::R_GAS;

/// The fixtures this module reads, by file name. Every lookup marks the row it
/// returns, so a test can show that each stored value is read by a benchmark
/// (lead decision S7).
pub const COLUMN_FIXTURES: [(&str, &str); 8] = [
    ("khan_2011.csv", KHAN),
    ("salvinder_2018.csv", SALVINDER),
    ("tan_2015.csv", TAN),
    ("sonderby_2013.csv", SONDERBY),
    ("dice_2020.csv", DICE),
    ("kwak_2012.csv", KWAK),
    ("morgan_2018.csv", MORGAN),
    ("vinjarapu_2024_parametric.csv", VINJ),
];

static READS: Mutex<BTreeMap<&'static str, BTreeSet<usize>>> = Mutex::new(BTreeMap::new());

/// Data rows of a column fixture that no lookup has returned in this process,
/// by zero-based data-row index, as (series, quantity) pairs.
pub fn unread_fixture_rows(name: &str) -> Result<Vec<(usize, String, String)>> {
    let (key, text) = COLUMN_FIXTURES
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| EngineError::Parse(format!("no column fixture named {name}")))?;
    let f = Fixture::parse(text)?;
    let read = READS
        .lock()
        .map_err(|_| EngineError::Parse("fixture read log is poisoned".into()))?
        .get(key)
        .cloned()
        .unwrap_or_default();
    Ok(f.rows
        .iter()
        .enumerate()
        .filter(|(i, _)| !read.contains(i))
        .map(|(i, r)| (i, r.series.clone(), r.quantity.clone()))
        .collect())
}

/// One fixture row.
#[derive(Debug, Clone)]
struct Fix {
    index: usize,
    source: String,
    locator: String,
    series: String,
    quantity: String,
    value: f64,
}

struct Fixture {
    name: &'static str,
    rows: Vec<Fix>,
}

impl Fixture {
    fn parse(text: &str) -> Result<Self> {
        let name = COLUMN_FIXTURES
            .iter()
            .find(|(_, t)| *t == text)
            .map(|(n, _)| *n)
            .unwrap_or("unnamed");
        let mut rows = Vec::new();
        for (i, line) in text.lines().enumerate().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let c = split_csv(line);
            if c.len() < 10 {
                return Err(EngineError::Parse(format!(
                    "fixture line {} is short",
                    i + 1
                )));
            }
            let value = c[6]
                .parse::<f64>()
                .map_err(|_| EngineError::Parse(format!("fixture line {}: {}", i + 1, c[6])))?;
            rows.push(Fix {
                index: rows.len(),
                source: c[0].clone(),
                locator: c[1].clone(),
                series: c[4].clone(),
                quantity: c[5].clone(),
                value,
            });
        }
        Ok(Self { name, rows })
    }

    fn get(&self, series: &str, quantity: &str) -> Result<&Fix> {
        let r = self
            .rows
            .iter()
            .find(|r| r.series == series && r.quantity == quantity)
            .ok_or_else(|| EngineError::Parse(format!("fixture has no {series} {quantity}")))?;
        if let Ok(mut log) = READS.lock() {
            log.entry(self.name).or_default().insert(r.index);
        }
        Ok(r)
    }

    fn v(&self, series: &str, quantity: &str) -> Result<f64> {
        Ok(self.get(series, quantity)?.value)
    }

    fn series(&self) -> Vec<String> {
        let mut s: Vec<String> = Vec::new();
        for r in &self.rows {
            if !s.contains(&r.series) {
                s.push(r.series.clone());
            }
        }
        s
    }
}

#[allow(clippy::too_many_arguments)]
fn bench_row(
    test_id: String,
    subject: &str,
    fix: &Fix,
    conditions: String,
    lit: f64,
    sim: f64,
    unit: &str,
    tol: Option<f64>,
    data_use: &'static str,
    note: String,
) -> BenchRow {
    BenchRow {
        test_id,
        level: 2,
        subject: subject.to_string(),
        source: fix.source.clone(),
        locator: fix.locator.clone(),
        conditions,
        literature: Some(lit),
        simulated: sim,
        unit: unit.to_string(),
        tolerance: tol,
        pass: tol.map(|t| sim.is_finite() && (sim - lit).abs() <= t),
        data_use,
        explanation: note,
    }
}

fn not_run(mut r: BenchRow, reason: &str) -> BenchRow {
    r.simulated = f64::NAN;
    r.pass = None;
    r.explanation = format!("not_run: {reason}");
    r
}

/// Solvent of `n_mea` mol/s at CO2-free mass fraction `w` and loading `alpha`.
fn solvent_of(tag: &str, n_mea: f64, w: f64, alpha: f64, t_k: f64, p_pa: f64) -> SolventStream {
    let m_free = n_mea * MW_MEA / w;
    SolventStream {
        tag: tag.into(),
        t_k,
        p_pa,
        n_mea,
        n_h2o: m_free * (1.0 - w) / MW_WATER,
        n_co2: alpha * n_mea,
    }
}

/// MEA flow, mol/s, in a solution mass flow `m_kg_s` at CO2-free fraction `w` and
/// loading `alpha`.
fn n_mea_of_mass(m_kg_s: f64, w: f64, alpha: f64) -> f64 {
    let m_free = m_kg_s / (1.0 + alpha * w * MW_CO2 / MW_MEA);
    m_free * w / MW_MEA
}

fn std_mol_s(volume_l_min: f64) -> f64 {
    volume_l_min * 1.0e-3 / 60.0 * ATM_PA / (R * T_STD_K)
}

fn pct_err(sim: f64, lit: f64) -> f64 {
    100.0 * (sim - lit).abs() / lit
}

fn median(mut v: Vec<f64>) -> f64 {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

fn pilot_absorber(
    tag: &str,
    packing: Structured,
    d: f64,
    z: f64,
    p: f64,
    w: f64,
    t_phi: f64,
) -> AbsorberSpec {
    AbsorberSpec {
        tag: tag.into(),
        packing,
        diameter_m: d,
        height_m: z,
        slices: columns::SLICES,
        p_pa: p,
        gas_film: GasFilm::Omitted,
        t_phi_k: t_phi,
        w_mea: w,
        d_mea: absorber::DUGAS_7M_LEAN,
    }
}

/// KH-1 and KH-2. Khan Table 1 prints no specific area or critical surface tension
/// for its random packings, and no local source holds them, so Onda's Eqs 5a to 5c
/// cannot be set up. Each row is reported as not run, with its frozen limit.
pub fn absorber_khan_table1() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(KHAN)?;
    let mut out = Vec::new();
    for (id, run, cond) in [
        (
            "KH-1-T22",
            "T22",
            "103.15 kPa, 12.7 mm Berl saddles, 6.6 m, 0.1 m",
        ),
        (
            "KH-1-T13",
            "T13",
            "103.15 kPa, 12.7 mm Berl saddles, 6.6 m, 0.1 m",
        ),
        (
            "KH-1-T15",
            "T15",
            "103.15 kPa, 12.7 mm Berl saddles, 6.6 m, 0.1 m",
        ),
        (
            "KH-1-T19",
            "T19",
            "103.15 kPa, 12.7 mm Berl saddles, 6.6 m, 0.1 m",
        ),
        (
            "KH-2-I06",
            "I06",
            "724.47 kPa, 50 mm Pall rings, 14.1 m, 1.9 m",
        ),
    ] {
        let fix = f.get(run, "CO2_removal")?;
        let r = bench_row(
            id.into(),
            "CO2 removal",
            fix,
            cond.into(),
            fix.value,
            f64::NAN,
            "%",
            Some(2.0),
            SAME,
            String::new(),
        );
        out.push(not_run(
            r,
            "Khan prints no specific area or critical surface tension for these random \
             packings (Table 1 and Section 3, PDF p5) and no local source holds them, so \
             Onda's Eqs 5a to 5c cannot run (derivation note M17)",
        ));
    }
    Ok(out)
}

/// SA-1: Salvinder pilot at 10.5 bar, CO2 removal 85.0% within 5 points.
pub fn absorber_salvinder_85pct() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(SALVINDER)?;
    let s = "nominal";
    let p = f.v(s, "sour_gas_P")? * 1.0e5;
    let t_g = f.v(s, "sour_gas_T")? + 273.15;
    let t_l = f.v(s, "lean_T")? + 273.15;
    let w = f.v(s, "solvent_MEA")? / 100.0;
    let g = std_mol_s(f.v(s, "sour_gas_flow")?);
    let mut n = [0.0; NSP];
    n[Sp::Co2 as usize] = g * f.v(s, "sour_gas_CO2")?;
    n[Sp::Ch4 as usize] = g * f.v(s, "sour_gas_CH4")?;
    let gas = Stream::gas("SA_GAS", t_g, p, n);
    let solvent = Solvent::new(w, 313.15)?;
    let m_l = f.v(s, "lean_flow")? * 1.0e-3 / 60.0 * solvent.density(t_l, 0.0)?;
    let lean = solvent_of("SA_LEAN", n_mea_of_mass(m_l, w, 0.0), w, 0.0, t_l, p);
    let spec = pilot_absorber(
        "SA",
        FLEXIPAC_1Y,
        f.v(s, "column_D")?,
        f.v(s, "column_H")?,
        p,
        w,
        313.15,
    );
    let fix = f.get("t2", "CO2_removal")?;
    let cond = "10.5 bar, 26 C, 50 L/min CO2:CH4 1:1, 1.04 L/min fresh 20 wt% MEA, \
                1.5 m Flexipac 1Y"
        .to_string();
    let base = bench_row(
        "SA-1".into(),
        "CO2 removal",
        fix,
        cond,
        fix.value,
        f64::NAN,
        "%",
        Some(5.0),
        INDEPENDENT,
        String::new(),
    );
    let note = "Gas flow read at 0 C and 1 atm (actual flow at 10.5 bar cannot close the \
                CO2 balance); lean solvent taken as fresh, since the pilot runs once \
                through; gas film omitted, no local CH4 diffusivity; Flexipac 1Y geometry \
                per `packing`";
    Ok(vec![match absorber::solve(&spec, &gas, &lean, None) {
        Ok(r) => BenchRow {
            simulated: 100.0 * r.capture,
            pass: Some((100.0 * r.capture - fix.value).abs() <= 5.0),
            explanation: format!("{note}; rich loading {:.3}", r.rich.n_co2 / r.rich.n_mea),
            ..base
        },
        Err(e) => not_run(base, &format!("absorber did not converge: {e}")),
    }])
}

/// Tan at 0.1, 1 and 5 MPa: removal printed beside 76, 95 and 100%, and one trend
/// row that passes when removal rises from 0.1 to 1 MPa and does not fall to 5 MPa.
pub fn absorber_tan_trend() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(TAN)?;
    let s = "setup";
    let w = f.v(s, "MEA")? / 100.0;
    let t = f.v(s, "liquid_T")? + 273.15;
    let g = std_mol_s(f.v(s, "gas_flow")?);
    let y = f.v(s, "CO2_in")? / 100.0;
    let solvent = Solvent::new(w, 313.15)?;
    let m_l = f.v(s, "liquid_flow")? * 1.0e-3 / 60.0 * solvent.density(t, 0.0)?;
    let note = "100 slpm read at 0 C and 1 atm; the 50% balance taken as CH4; gas at the \
                35 C solvent temperature (unprinted); fresh solvent; gas film omitted, no \
                local CH4 diffusivity";
    let mut out = Vec::new();
    let mut sims = Vec::new();
    let mut lits = Vec::new();
    for (series, mpa) in [("P0.1", 0.1), ("P1", 1.0), ("P5", 5.0)] {
        let p = mpa * 1.0e6;
        let mut n = [0.0; NSP];
        n[Sp::Co2 as usize] = g * y;
        n[Sp::Ch4 as usize] = g * (1.0 - y);
        let gas = Stream::gas("TAN_GAS", t, p, n);
        let lean = solvent_of("TAN_LEAN", n_mea_of_mass(m_l, w, 0.0), w, 0.0, t, p);
        let spec = pilot_absorber(
            "TAN",
            FLEXIPAC_1Y,
            f.v(s, "column_ID")?,
            f.v(s, "packing_height")?,
            p,
            w,
            313.15,
        );
        let fix = f.get(series, "CO2_removal")?;
        lits.push(fix.value);
        let base = bench_row(
            format!("TAN-{series}"),
            "CO2 removal",
            fix,
            format!("{mpa} MPa, 50% CO2 in natural gas, 20 wt% MEA, 1.0 m Flexipac 1Y"),
            fix.value,
            f64::NAN,
            "%",
            None,
            INDEPENDENT,
            String::new(),
        );
        match absorber::solve(&spec, &gas, &lean, None) {
            Ok(r) => {
                sims.push(100.0 * r.capture);
                let flag = if r.notes.pco2_above_reliable > 0 {
                    "; CO2 pressure above 10 bar, past the Aronu reliability (G.32)"
                } else {
                    ""
                };
                out.push(BenchRow {
                    simulated: 100.0 * r.capture,
                    explanation: format!(
                        "{note}; rich loading {:.3}{flag}",
                        r.rich.n_co2 / r.rich.n_mea
                    ),
                    ..base
                });
            }
            Err(e) => {
                sims.push(f64::NAN);
                out.push(not_run(base, &format!("absorber did not converge: {e}")));
            }
        }
    }
    let fix = f.get("P1", "CO2_removal")?;
    let rising =
        sims.iter().all(|v| v.is_finite()) && sims[0] < sims[1] && sims[1] <= sims[2] + 1e-9;
    out.push(BenchRow {
        test_id: "TAN-TREND".into(),
        subject: "Removal rises with pressure, 0.1 to 1 to 5 MPa".into(),
        literature: Some(1.0),
        simulated: if sims.iter().all(|v| v.is_finite()) {
            f64::from(u8::from(rising))
        } else {
            f64::NAN
        },
        unit: "1 = rising".into(),
        tolerance: Some(0.0),
        pass: sims.iter().all(|v| v.is_finite()).then_some(rising),
        explanation: {
            let miss: Vec<f64> = sims.iter().zip(&lits).map(|(s, l)| (l - s).abs()).collect();
            let lo = miss.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = miss.iter().cloned().fold(0.0, f64::max);
            format!(
                "Plan section 12 trend on 76, 95, 100%; simulated {:.1}, {:.1}, {:.1}%. The \
                 frozen criterion judges the sign of the trend only. The magnitudes miss \
                 the printed removals by {lo:.0} to {hi:.0} points (TAN-P0.1, TAN-P1, \
                 TAN-P5), so this row is not counted as validation evidence",
                sims[0], sims[1], sims[2]
            )
        },
        conditions: "Section 3.2 removals at the gas outlet".into(),
        ..bench_row(
            String::new(),
            "",
            fix,
            String::new(),
            1.0,
            f64::NAN,
            "",
            None,
            INDEPENDENT,
            String::new(),
        )
    });
    Ok(out)
}

fn liquid_t_at(r: &AbsorberResult, z: f64) -> f64 {
    let s = &r.slices;
    if z <= s[0].z_m {
        return s[0].t_liq_k;
    }
    for k in 1..s.len() {
        if z <= s[k].z_m {
            let u = (z - s[k - 1].z_m) / (s[k].z_m - s[k - 1].z_m);
            return s[k - 1].t_liq_k + u * (s[k].t_liq_k - s[k - 1].t_liq_k);
        }
    }
    s[s.len() - 1].t_liq_k
}

fn alpha_at(r: &AbsorberResult, z: f64) -> f64 {
    let s = &r.slices;
    if z <= s[0].z_m {
        return s[0].alpha;
    }
    for k in 1..s.len() {
        if z <= s[k].z_m {
            let u = (z - s[k - 1].z_m) / (s[k].z_m - s[k - 1].z_m);
            return s[k - 1].alpha + u * (s[k].alpha - s[k - 1].alpha);
        }
    }
    s[s.len() - 1].alpha
}

/// Sonderby runs R1 to R5 (8.2 m): removal, liquid temperature at TI1 to TI10 and
/// loading where Table 6 prints it, all reported with no limit (plan section 12).
pub fn absorber_sonderby_profiles() -> Result<Vec<BenchRow>> {
    absorber_sonderby_runs(&["R1", "R2", "R3", "R4", "R5"])
}

/// The Sonderby rows of the named runs.
pub fn absorber_sonderby_runs(runs: &[&str]) -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(SONDERBY)?;
    let w = f.v("setup", "MEA")? / 100.0;
    let d = f.v("setup", "packing_diameter")?;
    let solvent = Solvent::new(w, 313.15)?;
    let note = "Mellapak 250Y geometry is not local, RSP-250 stands in; gas film omitted, \
                no local N2 diffusivity; gas flow from the printed L/G; inlet gas at TI31, \
                saturated with water; lean solvent at TI43; probe TIk read at 0.82(k-1) m";
    let mut out = Vec::new();
    for &run in runs {
        let p = 100.0 * (f.v(run, "P_external")? + f.v(run, "PI1_bottom")?);
        let alpha0 = f.v(run, "loading_V0")?;
        let t_l = f.v(run, "TI43")? + 273.15;
        let t_g = f.v(run, "TI31")? + 273.15;
        let m_l = f.v(run, "FI1_liquid")? * 1.0e-3 / 60.0 * solvent.density(t_l, alpha0)?;
        let lean = solvent_of("SO_LEAN", n_mea_of_mass(m_l, w, alpha0), w, alpha0, t_l, p);
        let l_mol = lean.n_mea + lean.n_h2o + lean.n_co2;
        let g = l_mol / f.v(run, "L_over_G")?;
        let y_c = f.v(run, "Ci1_CO2_in")? / 100.0;
        let y_w = psat_pa(t_g)? / p;
        let mut n = [0.0; NSP];
        n[Sp::Co2 as usize] = g * y_c;
        n[Sp::H2o as usize] = g * y_w;
        n[Sp::N2 as usize] = g * (1.0 - y_c - y_w);
        let gas = Stream::gas("SO_GAS", t_g, p, n);
        let spec = pilot_absorber("SO", RSP_250, d, f.v(run, "column_H")?, p, w, 313.15);
        let ci1 = f.get(run, "Ci1_CO2_in")?;
        let c1 = ci1.value;
        let c2 = f.v(run, "Ci2_CO2_out")?;
        let lit_cap = 100.0 * (1.0 - (c2 / (100.0 - c2)) / (c1 / (100.0 - c1)));
        let cond = format!("{run}, 8.2 m, L/G {}", f.v(run, "L_over_G")?);
        let res = absorber::solve(&spec, &gas, &lean, None);
        let cap_row = bench_row(
            format!("SO-{run}-CAP"),
            "CO2 removal from Ci1 and Ci2 on a dry, inert basis",
            ci1,
            cond.clone(),
            lit_cap,
            f64::NAN,
            "%",
            None,
            INDEPENDENT,
            note.into(),
        );
        let r = match res {
            Ok(r) => r,
            Err(e) => {
                out.push(not_run(cap_row, &format!("absorber did not converge: {e}")));
                continue;
            }
        };
        out.push(BenchRow {
            simulated: 100.0 * r.capture,
            explanation: format!("{note}; solved by {:?} in {} steps", r.path, r.iterations),
            ..cap_row
        });
        let mut se = Vec::new();
        for k in 1..=10 {
            let q = format!("TI{k}");
            let fix = f.get(run, &q)?;
            let z = 0.82 * (k as f64 - 1.0);
            let sim = liquid_t_at(&r, z) - 273.15;
            se.push((sim - fix.value).powi(2));
            out.push(bench_row(
                format!("SO-{run}-{q}"),
                "Liquid temperature profile",
                fix,
                format!("{cond}, z = {z:.2} m"),
                fix.value,
                sim,
                "C",
                None,
                INDEPENDENT,
                "reported".into(),
            ));
        }
        let rms = (se.iter().sum::<f64>() / se.len() as f64).sqrt();
        let fix = f.get(run, "TI1")?;
        out.push(bench_row(
            format!("SO-{run}-T-RMS"),
            "RMS gap over TI1 to TI10",
            fix,
            cond.clone(),
            0.0,
            rms,
            "K",
            None,
            INDEPENDENT,
            "reported; the matrix freezes no limit for these profiles".into(),
        ));
        for v in ["V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "V10"] {
            let q = format!("loading_{v}");
            if let Ok(fix) = f.get(run, &q) {
                let k: f64 = v[1..].parse().unwrap_or(0.0);
                let z = 8.2 - 0.82 * (k - 1.0);
                out.push(bench_row(
                    format!("SO-{run}-{v}"),
                    "Liquid loading profile",
                    fix,
                    format!("{cond}, z = {z:.2} m"),
                    fix.value,
                    alpha_at(&r, z),
                    "mol/mol",
                    None,
                    INDEPENDENT,
                    "reported".into(),
                ));
            }
        }
    }
    Ok(out)
}

/// The CO2 fugacity coefficient `Solvent::pstar_gas_pa` applies at one state,
/// backed out of its own output, and a note giving the CO2 and water SRK binary of
/// card G.21 at the same state for comparison.
fn applied_phi_note(s: &Solvent, t_k: f64, alpha: f64, p_pa: f64) -> Result<(f64, String)> {
    let ideal = s.pstar_pa(t_k, alpha)?;
    let gas = s.pstar_gas_pa(t_k, alpha, p_pa)?;
    let t = t_k.max(298.15);
    let phi = ideal * nush2_mea::pstar::poynting(t, p_pa)? / gas;
    let bin = nush2_mea::pstar::srk_co2_h2o(t, p_pa, (ideal / p_pa).min(0.98))?;
    let root = if bin.z < 0.3 {
        "a liquid-like root"
    } else {
        "a vapour root"
    };
    Ok((
        phi,
        format!(
            "ideal vapour of card G.19; the CO2 and water SRK binary of card G.21 gives \
             Z {:.4} and phi {:.1} here, {root}, and the absorber does not use it because \
             its vapour is a carrier gas",
            bin.z, bin.phi_co2
        ),
    ))
}

/// Case B design check: 90% per pass within 40 m at rich loading 0.50, with the
/// height, diameter and liquid load reported.
pub fn absorber_case_b_spec() -> Result<Vec<BenchRow>> {
    let feed = columns::absorber_feed()?;
    let lean = columns::lean_solvent(&feed)?;
    let mk = |id: &str,
              subject: &str,
              lit: Option<f64>,
              sim: f64,
              unit: &str,
              tol: Option<f64>,
              note: String| BenchRow {
        test_id: id.into(),
        level: 2,
        subject: subject.into(),
        source: "Plan section 7 limits; USER_ANSWERS absorber 40 C, 26.8 bar, rich cap 0.50".into(),
        locator: "design check, no literature value".into(),
        conditions: "Case B feed from the M4 chain, 30 wt% MEA, lean 0.25, RSP-250 stand-in".into(),
        literature: lit,
        simulated: sim,
        unit: unit.into(),
        tolerance: tol,
        pass: match (lit, tol) {
            (Some(l), Some(t)) => Some(sim.is_finite() && (sim - l).abs() <= t),
            _ => None,
        },
        data_use: SAME,
        explanation: note,
    };
    let mut out = Vec::new();
    match columns::absorber_case(&feed, &lean, columns::LIQUID_LOAD_M3_M2_H) {
        Ok(a) => {
            let rich = a.result.rich.n_co2 / a.result.rich.n_mea;
            let meets = a.height_m.is_some_and(|z| z <= columns::HEIGHT_LIMIT_M)
                && rich <= columns::ALPHA_RICH_CAP + 1e-6;
            out.push(mk(
                "ABS-CASEB-SPEC",
                "90% per pass within 40 m at rich loading 0.50",
                Some(1.0),
                f64::from(u8::from(meets)),
                "1 = meets",
                Some(0.0),
                format!(
                    "capture {:.4} at {} m, rich loading {rich:.4}",
                    a.result.capture,
                    a.height_m.map_or("over 40".into(), |z| format!("{z:.2}"))
                ),
            ));
            out.push(mk(
                "ABS-CASEB-HEIGHT",
                "Packed height for 90%",
                None,
                a.height_m.unwrap_or(f64::NAN),
                "m",
                None,
                "reported; one enthalpy path temperature (110 C) for the whole island. Both \
                 high-pressure absorber benchmarks miss: SA-1 fails, and the Tan removals \
                 fall far below the printed values, so this height has no high-pressure \
                 validation behind it"
                    .into(),
            ));
            out.push(mk(
                "ABS-CASEB-DIAMETER",
                "Diameter at the assumed 40 m3/m2/h liquid load",
                None,
                a.diameter_m,
                "m",
                None,
                "reported; H.29".into(),
            ));
            out.push(mk(
                "ABS-CASEB-LOAD",
                "Liquid load",
                None,
                a.load_m3_m2_h,
                "m3/m2/h",
                None,
                "assumed, the low end of H.29".into(),
            ));
            out.push(mk(
                "ABS-CASEB-RICH",
                "Rich loading at 90%",
                None,
                rich,
                "mol/mol",
                None,
                "reported".into(),
            ));
            let solvent = Solvent::new(columns::W_MEA, columns::T_PHI_ISLAND_K)?;
            let lean_alpha = lean.n_co2 / lean.n_mea;
            for (id, end, t_k, alpha) in [
                ("ABS-CASEB-PHI-TOP", "top", lean.t_k, lean_alpha),
                ("ABS-CASEB-PHI-BOTTOM", "bottom", a.result.rich.t_k, rich),
            ] {
                let (phi, note) = applied_phi_note(&solvent, t_k, alpha, columns::P_ABS_PA)?;
                out.push(mk(
                    id,
                    &format!("CO2 fugacity coefficient applied at the absorber {end}"),
                    None,
                    phi,
                    "1",
                    None,
                    format!(
                        "reported; {:.2} C, loading {alpha:.4}, 26.8 bar; {note}",
                        t_k - 273.15
                    ),
                ));
            }
        }
        Err(e) => out.push(not_run(
            mk(
                "ABS-CASEB-SPEC",
                "90% per pass within 40 m at rich loading 0.50",
                Some(1.0),
                f64::NAN,
                "1 = meets",
                Some(0.0),
                String::new(),
            ),
            &format!("absorber did not converge after the fallback ladder: {e}"),
        )),
    }
    Ok(out)
}

fn strip_spec(
    tag: &str,
    w: f64,
    p_top: f64,
    p_bot: f64,
    t_cond: f64,
    mode: StripMode,
) -> StripperSpec {
    StripperSpec {
        tag: tag.into(),
        stages: columns::STAGES,
        murphree: columns::MURPHREE,
        p_top_pa: p_top,
        p_bottom_pa: p_bot,
        t_cond_k: t_cond,
        t_phi_k: columns::T_PHI_STRIP_K,
        w_mea: w,
        mode,
    }
}

fn f_to_k(t_f: f64) -> f64 {
    (t_f - 32.0) / 1.8 + 273.15
}

const STRIP_NOTE: &str = "12 equilibrium stages, Murphree 1, no heat loss, solvent cp 3.6 kJ/kg/K";

/// DICE design: SRD 3.64 GJ/t within 10%, from the printed rich stream, the
/// rich/lean exchanger duty and the lean loading. Reboiler and top temperatures,
/// overhead steam ratio and condenser duty are reported.
pub fn stripper_dice() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(DICE)?;
    // lbmol/h to mol/s: LB kg per lb, so LB kmol per lbmol.
    let lbmol = |x: f64| x * LB * 1000.0 / 3600.0;
    let mea = lbmol(f.v("stream21", "MEA")?);
    let water = lbmol(f.v("stream21", "water")?);
    let co2 = lbmol(f.v("stream21", "CO2_liquid")?);
    let w = mea * MW_MEA / (mea * MW_MEA + water * MW_WATER);
    let alpha_lean = f.v("stream22", "CO2_liquid")? / f.v("stream22", "MEA")?;
    let m_rich = mea * MW_MEA + water * MW_WATER + co2 * MW_CO2;
    let m_lean = m_rich - (co2 - alpha_lean * mea) * MW_CO2;
    let t_rich = f_to_k(f.v("stream21", "T")?);
    let x = regen::rich_lean_at_duty(
        m_rich,
        t_rich,
        m_lean,
        f_to_k(f.v("stream22", "T")?),
        mmbtu_h_to_mw(f.v("equipment", "E101_duty")?) * 1.0e6,
    )?;
    let rich = SolventStream {
        tag: "DICE_RICH".into(),
        t_k: x.t_rich_out_k,
        p_pa: 0.0,
        n_mea: mea,
        n_h2o: water,
        n_co2: co2,
    };
    let psia = 6894.757;
    let spec = strip_spec(
        "DICE",
        w,
        f.v("stream23", "P")? * psia,
        f.v("stream22", "P")? * psia,
        f_to_k(f.v("stream14", "T")?),
        StripMode::LeanLoading(alpha_lean),
    );
    let reb = f.get("equipment", "E104_duty")?;
    let m_co2_lit = (co2 - alpha_lean * mea) * MW_CO2;
    let srd_lit = mmbtu_h_to_mw(reb.value) * 1.0e6 / m_co2_lit / 1.0e6;
    let cond = format!(
        "Rich stream 21, heated by E-101 to {:.1} C, lean loading {alpha_lean:.4}",
        x.t_rich_out_k - 273.15
    );
    let base = bench_row(
        "DICE-SRD".into(),
        "Specific reboiler duty",
        reb,
        cond.clone(),
        srd_lit,
        f64::NAN,
        "GJ/t",
        Some(0.10 * srd_lit),
        INDEPENDENT,
        format!("E-104 duty over CO2 of streams 21 and 22; {STRIP_NOTE}"),
    );
    let s = match stripper::stages(&spec, &rich) {
        Ok(s) => s,
        Err(e) => {
            return Ok(vec![not_run(
                base,
                &format!("stripper did not converge: {e}"),
            )])
        }
    };
    let mut out = vec![BenchRow {
        simulated: s.srd_j_kg / 1.0e6,
        pass: Some((s.srd_j_kg / 1.0e6 - srd_lit).abs() <= 0.10 * srd_lit),
        ..base
    }];
    let reported = |id: &str, subj: &str, fix: &Fix, lit: f64, sim: f64, unit: &str| {
        bench_row(
            id.into(),
            subj,
            fix,
            cond.clone(),
            lit,
            sim,
            unit,
            None,
            INDEPENDENT,
            "reported".into(),
        )
    };
    let t22 = f.get("stream22", "T")?;
    out.push(reported(
        "DICE-TREB",
        "Reboiler temperature",
        t22,
        f_to_k(t22.value) - 273.15,
        s.t_reb_k - 273.15,
        "C",
    ));
    let t23 = f.get("stream23", "T")?;
    out.push(reported(
        "DICE-TTOP",
        "Stripper overhead temperature",
        t23,
        f_to_k(t23.value) - 273.15,
        s.t_top_k - 273.15,
        "C",
    ));
    let h2o = f.get("stream23", "H2O_vapour")?;
    out.push(reported(
        "DICE-RATIO",
        "Overhead H2O over CO2",
        h2o,
        h2o.value / f.v("stream23", "CO2_vapour")?,
        s.steam_ratio_top,
        "mol/mol",
    ));
    let e103 = f.get("equipment", "E103_duty")?;
    out.push(reported(
        "DICE-QCOND",
        "Condenser duty",
        e103,
        mmbtu_h_to_mw(e103.value),
        s.q_cond_w / 1.0e6,
        "MW",
    ));
    Ok(out)
}

/// Kwak base point: SRD 3.92 GJ/t within 10% at lean 0.15 and rich 0.50. The rich
/// feed temperature is printed as 90 to 95 C, so the gated row runs at the middle
/// and both ends are reported.
pub fn stripper_kwak() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(KWAK)?;
    let s = "base";
    let w = f.v(s, "MEA")? / 100.0;
    let a_l = f.v(s, "alpha_lean")?;
    let a_r = f.v(s, "alpha_rich")?;
    let m_lean = f.v(s, "L_over_G")? * f.v(s, "flue_gas_flow")? / 3600.0;
    let mea = n_mea_of_mass(m_lean, w, a_l);
    let p = f.v(s, "stripper_P")? * KGF_CM2_PA + ATM_PA;
    let srd = f.get(s, "SRD")?;
    let lo = f.v(s, "rich_to_stripper_T_low")?;
    let hi = f.v(s, "rich_to_stripper_T_high")?;
    let mut out = Vec::new();
    for (id, t_c, tol) in [
        ("KW-SRD", 0.5 * (lo + hi), Some(0.10 * srd.value)),
        ("KW-SRD-90C", lo, None),
        ("KW-SRD-95C", hi, None),
    ] {
        let rich = solvent_of("KW_RICH", mea, w, a_r, t_c + 273.15, p);
        let spec = strip_spec(
            "KW",
            w,
            p,
            p,
            columns::T_COND_K,
            StripMode::LeanLoading(a_l),
        );
        let base = bench_row(
            id.into(),
            "Specific reboiler duty",
            srd,
            format!("Rich 0.50 to lean 0.15 at {p:.0} Pa, rich feed {t_c:.1} C"),
            srd.value,
            f64::NAN,
            "GJ/t",
            tol,
            INDEPENDENT,
            format!(
                "Solvent from L/G 3.7 kg/Sm3 at 350 Sm3/h as lean solution; gauge \
                 pressure on 1 atm; condenser at 40 C assumed; {STRIP_NOTE}; the pilot \
                 duty includes its heat loss"
            ),
        );
        match stripper::stages(&spec, &rich) {
            Ok(st) => {
                let sim = st.srd_j_kg / 1.0e6;
                out.push(BenchRow {
                    simulated: sim,
                    pass: tol.map(|t| (sim - srd.value).abs() <= t),
                    ..base
                });
                if id == "KW-SRD" {
                    let tr = f.get(s, "reboiler_T")?;
                    out.push(bench_row(
                        "KW-TREB".into(),
                        "Reboiler temperature",
                        tr,
                        format!("{p:.0} Pa, lean 0.15"),
                        tr.value,
                        st.t_reb_k - 273.15,
                        "C",
                        None,
                        INDEPENDENT,
                        "reported".into(),
                    ));
                }
            }
            Err(e) => out.push(not_run(base, &format!("stripper did not converge: {e}"))),
        }
    }
    Ok(out)
}

fn rich_from_mass(tag: &str, m_kg_s: f64, w: f64, alpha: f64, t_k: f64) -> SolventStream {
    solvent_of(tag, n_mea_of_mass(m_kg_s, w, alpha), w, alpha, t_k, 0.0)
}

/// Morgan NCCC K14 to K16 at their printed reboiler duty: outlet solvent
/// temperature within the 1.4 C band (plan section 12, Morgan PDF p13) and lean
/// loading reported.
pub fn stripper_morgan() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(MORGAN)?;
    let mut out = Vec::new();
    for case in ["K14", "K15", "K16"] {
        let w = f.v(case, "w_MEA_rich")?;
        let rich = rich_from_mass(
            "MO_RICH",
            f.v(case, "rich_flow")? / 3600.0,
            w,
            f.v(case, "alpha_rich")?,
            f.v(case, "inlet_T")? + 273.15,
        );
        let p = 1000.0 * f.v(case, "P")?;
        let q = 1000.0 * f.v(case, "reboiler_duty")?;
        let spec = strip_spec("MO", w, p, p, columns::T_COND_K, StripMode::ReboilerDuty(q));
        let t_out = f.get(case, "outlet_T")?;
        let lean = f.get(case, "alpha_lean")?;
        let cond = format!("{case}, {:.0} kW, {:.2} kPa", q / 1000.0, p / 1000.0);
        let note = format!(
            "Rich MEA weight fraction read on the nominal CO2-free basis of Morgan Eq 1 \
             (PDF p5); condenser at 40 C assumed; {STRIP_NOTE}"
        );
        let t_row = bench_row(
            format!("MO-{case}-TOUT"),
            "Stripper outlet solvent temperature",
            t_out,
            cond.clone(),
            t_out.value,
            f64::NAN,
            "C",
            Some(1.4),
            INDEPENDENT,
            note.clone(),
        );
        let l_row = bench_row(
            format!("MO-{case}-LEAN"),
            "Lean loading",
            lean,
            cond,
            lean.value,
            f64::NAN,
            "mol/mol",
            None,
            INDEPENDENT,
            format!("reported; {note}"),
        );
        match stripper::stages(&spec, &rich) {
            Ok(s) => {
                let sim_t = s.t_reb_k - 273.15;
                out.push(BenchRow {
                    simulated: sim_t,
                    pass: Some((sim_t - t_out.value).abs() <= 1.4),
                    ..t_row
                });
                out.push(BenchRow {
                    simulated: s.lean.n_co2 / s.lean.n_mea,
                    ..l_row
                });
            }
            Err(e) => {
                out.push(not_run(t_row, &format!("stripper did not converge: {e}")));
                out.push(not_run(l_row, &format!("stripper did not converge: {e}")));
            }
        }
    }
    Ok(out)
}

struct VinjState {
    id: String,
    rich: SolventStream,
    p: f64,
    w: f64,
    alpha_lean: f64,
    q: f64,
    capture: f64,
}

fn vinj_state(f: &Fixture, s: &str) -> Result<VinjState> {
    let w = f.v(s, "wt_MEA_rich")? / 100.0;
    let p = f.v(s, "PTS1_stripper_bottom_P")? * 1.0e5 + ATM_PA;
    Ok(VinjState {
        id: s.into(),
        rich: rich_from_mass(
            "VI_RICH",
            f.v(s, "FTA1_rich_flow")? / 3600.0,
            w,
            f.v(s, "alpha_rich")?,
            f.v(s, "TTA4_rich_to_stripper_T")? + 273.15,
        ),
        p,
        w,
        alpha_lean: f.v(s, "alpha_lean")?,
        q: 1000.0 * f.v(s, "RD_reboiler_duty")?,
        capture: f.v(s, "capture")?,
    })
}

fn vinj_srd(st: &VinjState) -> Result<StripperResult> {
    let spec = strip_spec(
        "VI",
        st.w,
        st.p,
        st.p,
        columns::T_COND_K,
        StripMode::LeanLoading(st.alpha_lean),
    );
    stripper::stages(&spec, &st.rich)
}

const VINJ_NOTE: &str = "Inputs from Tables D.12, D.17 and D.19; rich MEA wt% read on the \
                         CO2-free basis; gauge pressure on 1 atm; condenser at 40 C \
                         assumed; the D.19 SRD is net of pilot heat loss and the model has \
                         none";

/// Vinjarapu Amager Bakke: VI-1 median SRD error over the 16 states at 88 to 92%
/// capture (10%), VI-2 over all 85 states (15%), VI-3 lean loading of the Table 5
/// states SS1 to SS3 (0.03), which are D-table states SS4, SS50 and SS5 by their
/// printed lean flow, SRD and lean loading.
pub fn stripper_vinjarapu() -> Result<Vec<BenchRow>> {
    let f = Fixture::parse(VINJ)?;
    let mut out = Vec::new();
    let mut err_all = Vec::new();
    let mut err_window = Vec::new();
    let states: Vec<String> = f
        .series()
        .into_iter()
        .filter(|s| s.starts_with("SS") && f.get(s, "SRD").is_ok())
        .collect();
    for s in &states {
        let st = vinj_state(&f, s)?;
        let srd = f.get(s, "SRD")?;
        let in_window = (88.0..=92.0).contains(&st.capture);
        let base = bench_row(
            format!("VI-SRD-{}", st.id),
            "Specific reboiler duty",
            srd,
            format!(
                "{}, capture {:.1}%, lean {:.3}, {:.3} bar",
                st.id,
                st.capture,
                st.alpha_lean,
                st.p / 1.0e5
            ),
            srd.value,
            f64::NAN,
            "GJ/t",
            None,
            INDEPENDENT,
            VINJ_NOTE.into(),
        );
        match vinj_srd(&st) {
            Ok(r) => {
                let sim = r.srd_j_kg / 1.0e6;
                let e = pct_err(sim, srd.value);
                err_all.push(e);
                if in_window {
                    err_window.push(e);
                }
                out.push(BenchRow {
                    simulated: sim,
                    ..base
                });
            }
            Err(e) => {
                err_all.push(f64::NAN);
                out.push(not_run(base, &format!("stripper did not converge: {e}")));
            }
        }
    }
    let srd_fix = f.get(&states[0], "SRD")?;
    for (id, subject, errs, n_expected, tol) in [
        (
            "VI-1",
            "Median absolute SRD error, capture 88 to 92%",
            &err_window,
            16usize,
            10.0,
        ),
        (
            "VI-2",
            "Median absolute SRD error, all states",
            &err_all,
            states.len(),
            15.0,
        ),
    ] {
        let ran = errs.iter().filter(|v| v.is_finite()).count();
        let m = median(errs.clone());
        out.push(BenchRow {
            test_id: id.into(),
            subject: subject.into(),
            conditions: format!("{ran} of {n_expected} states ran"),
            literature: Some(0.0),
            simulated: m,
            unit: "%".into(),
            tolerance: Some(tol),
            pass: m.is_finite().then_some(m <= tol),
            ..bench_row(
                String::new(),
                "",
                srd_fix,
                String::new(),
                0.0,
                m,
                "%",
                None,
                INDEPENDENT,
                VINJ_NOTE.into(),
            )
        });
    }
    for (t5, d) in [("t5_SS1", "SS4"), ("t5_SS2", "SS50"), ("t5_SS3", "SS5")] {
        let lit = f.get(t5, "alpha_lean")?;
        let st = vinj_state(&f, d)?;
        let spec = strip_spec(
            "VI",
            st.w,
            st.p,
            st.p,
            columns::T_COND_K,
            StripMode::ReboilerDuty(st.q),
        );
        let base = bench_row(
            format!("VI-3-{}", &t5[3..]),
            "Lean loading at the printed reboiler duty",
            lit,
            format!(
                "Table 5 {} as D-table {d}, {:.1} kW",
                &t5[3..],
                st.q / 1000.0
            ),
            lit.value,
            f64::NAN,
            "mol/mol",
            Some(0.03),
            INDEPENDENT,
            VINJ_NOTE.into(),
        );
        match stripper::stages(&spec, &st.rich) {
            Ok(r) => {
                let sim = r.lean.n_co2 / r.lean.n_mea;
                out.push(BenchRow {
                    simulated: sim,
                    pass: Some((sim - lit.value).abs() <= 0.03),
                    ..base
                });
            }
            Err(e) => out.push(not_run(base, &format!("stripper did not converge: {e}"))),
        }
    }
    Ok(out)
}

/// Every M6 benchmark row.
pub fn column_rows() -> Result<Vec<BenchRow>> {
    let mut out = Vec::new();
    out.extend(absorber_khan_table1()?);
    out.extend(absorber_salvinder_85pct()?);
    out.extend(absorber_tan_trend()?);
    out.extend(absorber_sonderby_profiles()?);
    out.extend(absorber_case_b_spec()?);
    out.extend(stripper_dice()?);
    out.extend(stripper_kwak()?);
    out.extend(stripper_morgan()?);
    out.extend(stripper_vinjarapu()?);
    Ok(out)
}

/// Gate rows of terms T07 and T08: every M6 row with a limit. A row that missed
/// or failed to converge fails the gate; a row that cannot run for want of
/// source data (the Khan rows) is left out of it.
pub fn column_gate(rows: &[BenchRow]) -> Evidence {
    let gated = rows.iter().filter(|r| r.tolerance.is_some());
    let mut any = false;
    for r in gated {
        let lacks_data = r.explanation.starts_with("not_run: Khan prints");
        if lacks_data {
            continue;
        }
        any = true;
        if r.pass != Some(true) {
            return Evidence::GateFailed;
        }
    }
    if any {
        Evidence::Validated
    } else {
        Evidence::NotRun
    }
}

/// Modelled values of the M6 terms on Case B.
pub fn column_terms() -> Result<columns::ColumnTerms> {
    columns::column_terms()
}
