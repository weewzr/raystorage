//! M2 property benchmarks as validation rows. Fixtures live in `data/props` with
//! their citations in comment lines; the M2 tests in `engine/props/tests` read the
//! same files. Limits are those of the M2 tests, unchanged.
//!
//! The Shomate comparison against JANAF misses its Cp limit on one value of 162:
//! CO at 400 K, 29.3027 against 29.342 J/mol/K, or 0.134%. The printed NIST
//! polynomial itself gives 29.3027, so the code is correct and the row stays
//! failed against the unchanged 0.1% limit.

use crate::report::Row;
use crate::rowkit::{Meta, INDEPENDENT, SAME};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::{self, KSource};
use nush2_props::helium;
use nush2_props::mixture::{self, Flows};
use nush2_props::species::{self, Sp, NSP};
use nush2_props::steam;
use nush2_props::units::BAR_PA;

const JANAF: &str = include_str!("../data/props/janaf_spot.csv");
const IF97_VECTORS: &str = include_str!("../data/props/if97_vectors.csv");
const IF97_SAT: &str = include_str!("../data/props/if97_saturation.csv");
const PETERSEN: &str = include_str!("../data/props/petersen_tables.csv");
const EQUILIBRIUM: &str = include_str!("../data/props/equilibrium_benchmarks.csv");

const JANAF_SRC: &str = "NIST-JANAF Thermochemical Tables, Chase 1998";
const IF97_SRC: &str = "IAPWS-IF97 Revised Release, Lucerne 2007";
const PETERSEN_SRC: &str = "Petersen 1970, The Properties of Helium, Riso Report No. 224";
const ZHANG_SRC: &str = "Zhang et al. 2021, Table 1 (PDF p5), against the NIST Gibbs route";

/// Shomate limits, set before the first run of the M2 test: Cp relative 1e-3, S
/// relative 1e-4, H - H298 absolute 0.01 kJ/mol.
pub const SHOMATE_CP_REL: f64 = 1.0e-3;
pub const SHOMATE_S_REL: f64 = 1.0e-4;
pub const SHOMATE_H_ABS_KJ: f64 = 0.01;

/// Data rows of a fixture, split on commas, with comments and the header gone.
fn rows(text: &str) -> Vec<Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .skip(1)
        .map(|l| l.split(',').map(str::trim).collect())
        .collect()
}

fn num(s: &str) -> Result<f64> {
    s.parse()
        .map_err(|_| EngineError::Parse(format!("bad number {s}")))
}

fn species_named(name: &str) -> Result<Sp> {
    Ok(match name {
        "CH4" => Sp::Ch4,
        "CO" => Sp::Co,
        "CO2" => Sp::Co2,
        "H2" => Sp::H2,
        "H2O" => Sp::H2o,
        "N2" => Sp::N2,
        other => return Err(EngineError::Parse(format!("unknown species {other}"))),
    })
}

/// Worst point of one Shomate quantity: (deviation, label, literature, model).
#[derive(Default)]
struct Worst {
    dev: f64,
    at: String,
    lit: f64,
    sim: f64,
    over: Vec<String>,
}

impl Worst {
    fn see(&mut self, dev: f64, at: String, lit: f64, sim: f64, over: bool) {
        if over {
            self.over.push(format!("{at}: {sim:.4} against {lit}"));
        }
        if dev > self.dev || self.at.is_empty() {
            *self = Worst {
                dev,
                at,
                lit,
                sim,
                over: std::mem::take(&mut self.over),
            };
        }
    }
}

/// Shomate Cp, S and H - H298 against JANAF, one row per quantity at its worst
/// point. Pass needs every point inside the limit.
pub fn shomate(out: &mut Vec<Row>) -> Result<()> {
    let (mut cp, mut s, mut h) = (Worst::default(), Worst::default(), Worst::default());
    let mut tables: Vec<String> = Vec::new();
    let data = rows(JANAF);
    for r in &data {
        let sp = species_named(r[0])?;
        if !tables.iter().any(|t| t == r[1]) {
            tables.push(r[1].to_string());
        }
        let t = num(r[2])?;
        let (lit_cp, lit_s, lit_h) = (num(r[3])?, num(r[4])?, num(r[5])?);
        let got_cp = species::cp_j_mol_k(sp, t)?;
        let got_s = species::s_j_mol_k(sp, t)?;
        let got_h = (species::h_j_mol(sp, t)? - species::h_j_mol(sp, 298.15)?) / 1000.0;
        let at = format!("{} {t} K", r[0]);
        let rel_cp = ((got_cp - lit_cp) / lit_cp).abs();
        let rel_s = ((got_s - lit_s) / lit_s).abs();
        let abs_h = (got_h - lit_h).abs();
        cp.see(rel_cp, at.clone(), lit_cp, got_cp, rel_cp >= SHOMATE_CP_REL);
        s.see(rel_s, at.clone(), lit_s, got_s, rel_s >= SHOMATE_S_REL);
        h.see(abs_h, at, lit_h, got_h, abs_h >= SHOMATE_H_ABS_KJ);
    }
    let locator = format!("Tables {}", tables.join(", "));
    for (id, subject, unit, w, tol, kind) in [
        (
            "PROPS-SHOMATE-CP",
            "Heat capacity, Shomate against JANAF",
            "J/mol/K",
            cp,
            SHOMATE_CP_REL,
            "relative",
        ),
        (
            "PROPS-SHOMATE-S",
            "Entropy, Shomate against JANAF",
            "J/mol/K",
            s,
            SHOMATE_S_REL,
            "relative",
        ),
        (
            "PROPS-SHOMATE-H",
            "H - H298, Shomate against JANAF",
            "kJ/mol",
            h,
            SHOMATE_H_ABS_KJ,
            "absolute",
        ),
    ] {
        let note = if w.over.is_empty() {
            format!("Every point inside the {kind} limit; worst point shown")
        } else {
            format!(
                "{} of {} points at or over the {kind} limit: {}. The printed NIST polynomial \
                 itself gives the model value, so the miss is the fit's own swing about JANAF",
                w.over.len(),
                data.len(),
                w.over.join("; ")
            )
        };
        out.push(
            Meta::new(id, 1, subject, JANAF_SRC, &locator)
                .unit(unit)
                .use_(INDEPENDENT)
                .cond(format!(
                    "{} rows, CH4 CO CO2 H2 H2O N2, worst at {}, {kind} limit",
                    data.len(),
                    w.at
                ))
                .flagged(Some(w.lit), w.sim, tol, w.over.is_empty(), note),
        );
    }
    Ok(())
}

/// The Gibbs route against JANAF log Kf, the Zhang constants against the Gibbs
/// route, and the recorded K2 departure at the low-temperature shift.
pub fn constants(out: &mut Vec<Row>) -> Result<()> {
    let table = rows(JANAF);
    let lk = |name: &str, t: f64| -> Result<f64> {
        let r = table
            .iter()
            .find(|r| r[0] == name && num(r[2]).is_ok_and(|v| v == t))
            .ok_or_else(|| EngineError::Parse(format!("no JANAF row {name} {t}")))?;
        num(r[6])
    };
    let mut worst = (0.0f64, 0.0f64);
    for t in [400.0, 500.0, 600.0, 700.0, 800.0, 1000.0, 1200.0, 1300.0] {
        let k1 = 10f64.powf(lk("CO", t)? - lk("CH4", t)? - lk("H2O", t)?);
        let k2 = 10f64.powf(lk("CO2", t)? - lk("CO", t)? - lk("H2O", t)?);
        let r1 = (equilibrium::k1_nist_bar2(t)? / k1 - 1.0).abs();
        let r2 = (equilibrium::k2_nist(t)? / k2 - 1.0).abs();
        if r1.max(r2) > worst.0 {
            worst = (r1.max(r2), t);
        }
    }
    out.push(
        Meta::new(
            "PROPS-K-NIST-JANAF",
            1,
            "K1 and K2 of the Gibbs route against JANAF log Kf",
            JANAF_SRC,
            "log Kf column",
        )
        .unit("relative")
        .use_(INDEPENDENT)
        .cond(format!(
            "400 to 1300 K, eight temperatures, worst at {} K",
            worst.1
        ))
        .flagged(
            Some(0.0),
            worst.0,
            0.01,
            worst.0 < 0.01,
            format!(
                "Worst relative deviation {:.3e}. JANAF prints log Kf to 3 decimals, so three \
                     terms carry up to 0.35%. Limit 1%",
                worst.0
            ),
        ),
    );

    let mut grid: Vec<f64> = (0..=92).map(|i| 680.0 + 5.0 * f64::from(i)).collect();
    grid.extend([800.0, 950.0, 1073.15, 1098.15, 1144.15]);
    let mut w = (1.0f64, 0.0f64, "");
    for &t in &grid {
        let r1 = equilibrium::k1_zhang_bar2(t)? / equilibrium::k1_nist_bar2(t)?;
        let r2 = equilibrium::k2_zhang(t)? / equilibrium::k2_nist(t)?;
        for (r, k) in [(r1, "K1"), (r2, "K2")] {
            if (r - 1.0).abs() > (w.0 - 1.0).abs() {
                w = (r, t, k);
            }
        }
    }
    out.push(
        Meta::new(
            "PROPS-K-ZHANG-NIST",
            1,
            "Zhang constants against the Gibbs route",
            ZHANG_SRC,
            "Table 1 PDF p5",
        )
        .unit("ratio")
        .use_(SAME)
        .cond(format!(
            "680 to 1144 K, {} temperatures, worst {} at {} K",
            grid.len(),
            w.2,
            w.1
        ))
        .judged(
            1.0,
            w.0,
            0.06,
            "Limit 6% across the reformer and the high-temperature shift",
        ),
    );
    let t_lts = 486.15;
    let r = equilibrium::k2_zhang(t_lts)? / equilibrium::k2_nist(t_lts)?;
    out.push(
        Meta::new(
            "PROPS-K2-ZHANG-LTS",
            1,
            "Zhang K2 against the Gibbs route at the LTS",
            ZHANG_SRC,
            "Table 1 PDF p5",
        )
        .unit("ratio")
        .use_(SAME)
        .cond(format!("{t_lts} K, outside the 6% domain"))
        .reported(
            Some(1.0),
            r,
            "Recorded departure. The LTS keeps Zhang K2 as its default because T961-6 passes \
                 with it, and T961-6-CO-NIST runs the NIST route as a sensitivity",
        ),
    );

    let vapour = species::h_j_mol(Sp::H2o, 298.15)?;
    let liquid = species::h_liquid_water_j_mol(298.15, 1.0e5)?;
    out.push(
        Meta::new(
            "PROPS-LATENT-298",
            1,
            "Vapour less liquid water enthalpy",
            "CODATA key values",
            "H2O(g) and H2O(l) formation enthalpies",
        )
        .unit("kJ/mol")
        .use_(INDEPENDENT)
        .cond("298.15 K, 1 bar")
        .judged(
            species::CODATA_LATENT_298_J_MOL / 1000.0,
            (vapour - liquid) / 1000.0,
            0.057,
            "Limit 0.040 kJ/mol on each formation enthalpy, added in quadrature",
        ),
    );
    Ok(())
}

/// Helium against the Petersen tables and IAPWS-IF97 against its own vectors.
pub fn tables(out: &mut Vec<Row>) -> Result<()> {
    let mut worst = (0.0f64, String::new());
    let mut n = 0;
    for r in rows(PETERSEN) {
        let (t, p, want) = (num(r[1])?, num(r[2])? * BAR_PA, num(r[3])?);
        let got = match r[0] {
            "rho" => helium::density_kg_m3(t, p)?,
            "mu" => helium::viscosity_pa_s(t)? * 1.0e5,
            "k" => helium::conductivity_w_m_k(t, p)?,
            other => return Err(EngineError::Parse(format!("unknown property {other}"))),
        };
        let rel = (got - want).abs() / want;
        if rel > worst.0 {
            worst = (rel, format!("{} at {t} K", r[0]));
        }
        n += 1;
    }
    out.push(
        Meta::new(
            "PROPS-HE-PETERSEN",
            1,
            "Helium density, viscosity and conductivity",
            PETERSEN_SRC,
            "Printed pp 39 to 41 (PDF pp 43 to 45)",
        )
        .unit("relative")
        .use_(SAME)
        .cond(format!("{n} table rows, worst {}", worst.1))
        .flagged(
            Some(0.0),
            worst.0,
            7.0e-4,
            worst.0 < 7.0e-4,
            format!(
                "Worst relative deviation {:.3e}. Correlations of the same report against \
                     its own tables. Limit one unit in the last printed digit of the coarsest \
                     table",
                worst.0
            ),
        ),
    );

    let mut worst = (0.0f64, String::new());
    let mut n = 0;
    let mut see = |got: f64, want: f64, what: String| {
        let rel = ((got - want) / want).abs();
        if rel > worst.0 {
            worst = (rel, what);
        }
        n += 1;
    };
    for r in rows(IF97_VECTORS) {
        let (t, p) = (num(r[2])?, num(r[3])? * 1.0e6);
        let s = steam::state(t, p)?;
        let tag = format!("Table {} at {t} K", r[0]);
        see(s.v_m3_kg, num(r[4])?, format!("v, {tag}"));
        see(s.h_j_kg, num(r[5])? * 1.0e3, format!("h, {tag}"));
        see(s.u_j_kg, num(r[6])? * 1.0e3, format!("u, {tag}"));
        see(s.s_j_kg_k, num(r[7])? * 1.0e3, format!("s, {tag}"));
        see(s.cp_j_kg_k, num(r[8])? * 1.0e3, format!("cp, {tag}"));
        see(s.w_m_s, num(r[9])?, format!("w, {tag}"));
    }
    for r in rows(IF97_SAT) {
        let (t, p) = (num(r[1])?, num(r[2])? * 1.0e6);
        match r[0] {
            "35" => see(steam::psat_pa(t)?, p, format!("ps at {t} K")),
            "36" => see(steam::tsat_k(p)?, t, format!("Ts at {p} Pa")),
            other => return Err(EngineError::Parse(format!("unknown table {other}"))),
        }
    }
    out.push(
        Meta::new(
            "PROPS-IF97-VECTORS",
            1,
            "Steam properties against the release test vectors",
            IF97_SRC,
            "Tables 5, 15 (PDF p9, p17), 35, 36 (PDF p34, p36)",
        )
        .unit("relative")
        .use_(SAME)
        .cond(format!("{n} values, worst {}", worst.1))
        .flagged(
            Some(0.0),
            worst.0,
            1.0e-8,
            worst.0 < 1.0e-8,
            format!(
                "Worst relative deviation {:.3e}. Code verification against vectors printed \
                     to 9 significant digits",
                worst.0
            ),
        ),
    );
    Ok(())
}

fn sp(pairs: &[(Sp, f64)]) -> Flows {
    let mut n = [0.0; NSP];
    for (s, v) in pairs {
        n[s.index()] = *v;
    }
    n
}

fn wet(r: &equilibrium::ReformEq) -> Result<[f64; 6]> {
    let y = mixture::mole_fractions(&r.n)?;
    Ok([Sp::Ch4, Sp::H2, Sp::Co, Sp::Co2, Sp::H2o, Sp::N2].map(|s| 100.0 * y[s.index()]))
}

/// The two equilibrium routes of the M2 tests: TECDOC-1236 on pure methane at the
/// steam to carbon its product gas implies, and TEV-953 A-40 on pure methane at
/// S/C 3. The matrix rows IA-1 and T953-1 run the printed feeds.
pub fn equilibrium_routes(out: &mut Vec<Row>) -> Result<()> {
    const NAMES: [&str; 6] = ["CH4", "H2", "CO", "CO2", "H2O", "N2"];
    let accepted = [(800.0, 41.9), (860.0, 41.9), (800.0, 20.0)];
    for r in rows(EQUILIBRIUM) {
        let (t_c, p_bar, x) = (num(r[1])?, num(r[2])?, num(r[3])?);
        let mut y = [0.0; 6];
        for (i, v) in y.iter_mut().enumerate() {
            *v = num(r[4 + i])?;
        }
        match r[0] {
            "TE1236" => {
                let [ch4, _, co, co2, h2o, _] = y;
                let sc = (h2o + co + 2.0 * co2) / (ch4 + co + co2);
                let feed = sp(&[(Sp::Ch4, 1.0), (Sp::H2o, sc)]);
                let z = equilibrium::reform(&feed, t_c + 273.15, p_bar * BAR_PA, KSource::Zhang)?;
                let meta = Meta::new(
                    format!("PROPS-TE1236-{t_c:.0}C-{p_bar}BAR"),
                    1,
                    "CH4 conversion, product-gas route",
                    "IAEA-TECDOC-1236, 2001",
                    "Tables 2.1 and 2.2 (PDF p13)",
                )
                .unit("%")
                .cond(format!(
                    "Pure CH4 at S/C {sc:.3} from the printed product gas, Zhang K"
                ));
                let note = "Route of the M2 test te1236_equilibrium_three_points";
                out.push(if accepted.contains(&(t_c, p_bar)) {
                    meta.judged(x, 100.0 * z.conversion, 0.5, note)
                } else {
                    meta.reported(Some(x), 100.0 * z.conversion, note)
                });
            }
            "INL" => {
                let pure = sp(&[(Sp::Ch4, 1.0), (Sp::H2o, 3.0)]);
                let z = equilibrium::reform(&pure, t_c + 273.15, p_bar * BAR_PA, KSource::Zhang)?;
                let got = wet(&z)?;
                for k in 0..5 {
                    out.push(
                        Meta::new(
                            format!("PROPS-INL-A40-PURE-{}", NAMES[k]),
                            1,
                            "Raw syngas, wet, pure methane route",
                            "INL TEV-953 Rev. 0 2010",
                            "A-40 PDF p74",
                        )
                        .unit("mol%")
                        .cond(format!("Pure CH4, S/C 3, {t_c:.1} C, {p_bar:.2} bar, Zhang K"))
                        .judged(
                            y[k],
                            got[k],
                            0.5,
                            "Route of the M2 test inl_a40_raw_syngas; N2 stays out of the comparison",
                        ),
                    );
                }
            }
            other => return Err(EngineError::Parse(format!("unknown source {other}"))),
        }
    }
    Ok(())
}

/// Every M2 row.
pub fn rows_m2(out: &mut Vec<Row>) -> Result<()> {
    shomate(out)?;
    constants(out)?;
    tables(out)?;
    equilibrium_routes(out)
}
