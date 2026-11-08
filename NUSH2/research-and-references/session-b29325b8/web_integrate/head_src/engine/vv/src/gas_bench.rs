//! Gas-side benchmarks of `docs/VALIDATION_MATRIX.md` that the unit crates do not
//! already run: TEV-953 (section 4), TEV-961 Table 2 (section 5), Xu and Froment
//! (section 6), Chinchen (section 7) and IAEA-TECDOC-1236 (section 17). Every
//! literature value and every printed input is read from the fixtures; the
//! tolerance of each row is the frozen value of the matrix.

use crate::fixture::{Fix, Fixture};
use crate::report::Row;
use crate::rowkit::{aard_pct, f_to_k, worst, Meta, CALIBRATION, PSI_PA, SAME};
use nush2_numerics::{EngineError, Result};
use nush2_props::equilibrium::{self, KSource};
use nush2_props::helium;
use nush2_props::mixture::{self, Flows};
use nush2_props::species::{self, Sp, NSP};
use nush2_props::units::{BAR_PA, LB, ST};
use nush2_units::case5;
use nush2_units::jaeri::{self, A2, A2_FIT_ACTIVITY};
use nush2_units::smr_eq::{self, SmrEqSpec};
use nush2_units::smr_tube::{self, HeatModel, Section, TubeSpec};
use nush2_units::stream::Stream;
use nush2_units::wgs::{k_chinchen, Pellet};
use nush2_units::xf::{self, Kinetics};

fn flows(pairs: &[(Sp, f64)]) -> Flows {
    let mut n = [0.0; NSP];
    for (s, v) in pairs {
        n[s.index()] += *v;
    }
    n
}

/// Wet mol% of one species.
fn wet_pct(n: &Flows, s: Sp) -> Result<f64> {
    Ok(100.0 * mixture::mole_fractions(n)?[s.index()])
}

fn sp_named(name: &str) -> Result<Sp> {
    Ok(match name {
        "H2" => Sp::H2,
        "CO" => Sp::Co,
        "CO2" => Sp::Co2,
        "CH4" => Sp::Ch4,
        "H2O" => Sp::H2o,
        "N2" => Sp::N2,
        other => return Err(EngineError::Parse(format!("unknown species {other}"))),
    })
}

/// TEV-953 A-40 natural gas per mol, lumped as plan section 3 states: C6 enters as
/// 1.2 mol of C5 and O2 joins N2.
fn a40_gas(f: &Fixture) -> Result<Flows> {
    let s = "htgr_cc_summary";
    let v = |q: &str| -> Result<f64> { Ok(f.v(s, q)? / 100.0) };
    Ok(flows(&[
        (Sp::Ch4, v("NG_CH4")?),
        (Sp::C2h6, v("NG_C2H6")?),
        (Sp::C3h8, v("NG_C3H8")?),
        (Sp::Nc4h10, v("NG_C4H10")?),
        (Sp::Nc5h12, v("NG_C5H12")? + 1.2 * v("NG_C6H14")?),
        (Sp::N2, v("NG_N2")? + v("NG_O2")?),
        (Sp::Co2, v("NG_CO2")?),
    ]))
}

/// T953-1 to T953-4.
pub fn tev953(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("tev953.csv")?;
    let ng = a40_gas(&f)?;
    let sc = f.v("htgr_cc_reformer", "steam_to_carbon_stage1")?;
    let t_out = f_to_k(f.v("htgr_cc_REFORM2_out", "T")?);
    let p_out = f.v("htgr_cc_REFORM2_out", "P")? * PSI_PA;
    let carbon = mixture::elements(&ng)?[0];
    let mut feed = ng;
    feed[Sp::H2o.index()] = sc * carbon;
    let r = equilibrium::reform(&feed, t_out, p_out, KSource::Zhang)?;
    let cond = format!(
        "A-40 natural gas, S/C {sc:.2} on all carbon fed (A-42 PDF p76), {:.0} C, {:.0} psia, \
         Zhang K",
        t_out - 273.15,
        p_out / PSI_PA
    );
    for name in ["H2", "CO", "CO2", "N2", "H2O", "CH4"] {
        let fix = f.get("htgr_cc_summary", &format!("raw_{name}"))?;
        out.push(
            Meta::at(fix, format!("T953-1-{name}"), 1, "Raw syngas, wet", "mol%")
                .cond(cond.clone())
                .judged(
                    fix.value,
                    wet_pct(&r.n, sp_named(name)?)?,
                    0.5,
                    "SMR-EQ on the printed feed. C6 is lumped as 1.2 mol C5 and O2 joins N2 \
                     (plan section 3)",
                ),
        );
    }

    let t = f.get("reformer_basis", "reformer_T")?;
    let s_c = f.v("reformer_basis", "steam_to_carbon")?;
    let x = f.get("reformer_basis", "CH4_conversion_gibbs")?;
    let pure = flows(&[(Sp::Ch4, 1.0), (Sp::H2o, s_c)]);
    let eq = equilibrium::reform(&pure, t.value + 273.15, p_out, KSource::Zhang)?;
    out.push(
        Meta::at(x, "T953-2", 1, "CH4 conversion, conventional reformer", "%")
            .cond(format!(
                "Pure CH4, S/C {s_c:.1}, {:.0} C, {:.0} psia carried over from A-45",
                t.value,
                p_out / PSI_PA
            ))
            .judged(
                x.value,
                100.0 * eq.conversion,
                1.0,
                "Pure methane stands in for the conventional feed, whose sheets have no text \
                 layer and were not read",
            ),
    );

    let he = f.v("htgr_cc_HE-1", "mass_flow")? * LB / 3600.0;
    let t_hot = f_to_k(f.v("htgr_cc_HE-1", "T")?);
    let t_cold = f_to_k(f.v("htgr_cc_HE-10", "T")?);
    let q = f.get("HTGR_CC", "process_heat_demand")?;
    out.push(
        Meta::at(q, "T953-3", 1, "Process heat from the helium duty", "MWt")
            .use_(SAME)
            .cond(format!(
                "HE-1 {:.0} lb/h, {:.0} F to HE-10 {:.0} F (A-45 PDF p79)",
                he * 3600.0 / LB,
                f.v("htgr_cc_HE-1", "T")?,
                f.v("htgr_cc_HE-10", "T")?
            ))
            .judged(
                q.value,
                helium::duty_w(he, t_hot, t_cold)? / 1.0e6,
                0.01 * q.value,
                "Limit 1% of the printed value, which carries three figures",
            ),
    );

    let co2 = f.get("Conv_noCC", "CO2_emitted")?;
    let h2 = f.v("htgr_cc_summary", "H2_mass")?;
    let lit = co2.value * ST * 1000.0 / 24.0 / (h2 * LB);
    let m = nush2_plant::anchored::Model::embedded()?;
    let sim = m.co2_conv * ST * 1000.0 / 24.0 / m.h2_a_kgh;
    out.push(
        Meta::at(
            co2,
            "T953-4",
            5,
            "CO2 intensity of the unabated SMR",
            "kg/kg",
        )
        .use_(SAME)
        .cond(format!(
            "Conventional without capture, {:.0} short ton/day over {h2:.0} lb/h H2 \
                 (A-40 PDF p74)",
            co2.value
        ))
        .judged(
            lit,
            sim,
            1.0e-3 * lit,
            "Literature value calculated from the two printed numbers; simulated value is \
                 the emissions baseline of the plant model. Limit 0.1%",
        ),
    );
    Ok(())
}

/// T961-1 and T961-2 on Table 2 and its continuation, and the check that every
/// Case 5 number typed in `nush2_units::case5` equals the fixture.
pub fn tev961(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("tev961.csv")?;
    for case in ["ROT725C", "ROT775C", "ROT825C", "ROT875C", "ROT925C"] {
        let m = f.v(case, "He_flow")?;
        let t_s = f.v(case, "He_supply_T")? + 273.15;
        let t_r = f.v(case, "He_return_T")? + 273.15;
        let t_i = f.v(case, "He_ideal_return_T")? + 273.15;
        let q = f.get(case, "process_heat_demand")?;
        let rel = if case == "ROT925C" { 0.002 } else { 0.0015 };
        out.push(
            Meta::at(q, format!("T961-1-{case}"), 1, "Process heat demand", "MWt")
                .use_(SAME)
                .cond(format!(
                    "{m} kg/s helium, {:.0} C to {:.0} C (Table 2 continued PDF p18)",
                    t_s - 273.15,
                    t_r - 273.15
                ))
                .judged(
                    q.value,
                    helium::duty_w(m, t_s, t_r)? / 1.0e6,
                    rel * q.value,
                    format!("Limit {:.2}% of the printed value", 100.0 * rel),
                ),
        );
        let x = f.get(case, "excess_heat")?;
        out.push(
            Meta::at(x, format!("T961-2-{case}"), 1, "Excess heat", "MWt")
                .use_(SAME)
                .cond(format!(
                    "{m} kg/s helium from the return {:.0} C to the ideal return {:.0} C",
                    t_r - 273.15,
                    t_i - 273.15
                ))
                .judged(
                    x.value,
                    helium::duty_w(m, t_r, t_i)? / 1.0e6,
                    0.5,
                    "Two temperatures printed to 1 K carry 0.4 MWt",
                ),
        );
    }

    let raw = "case5_summary";
    let shift = "case5_shift_selexol_psa";
    let mut pairs: Vec<(String, f64, f64)> = Vec::new();
    let mut push =
        |what: &str, typed: f64, fixed: f64| pairs.push((what.to_string(), typed, fixed));
    for (sp, typed) in case5::RAW_MOL_PCT {
        let name = match sp {
            nush2_units::stream::Sp::H2 => "H2",
            nush2_units::stream::Sp::Co => "CO",
            nush2_units::stream::Sp::Co2 => "CO2",
            nush2_units::stream::Sp::N2 => "N2",
            nush2_units::stream::Sp::H2o => "H2O",
            nush2_units::stream::Sp::Ch4 => "CH4",
            _ => return Err(EngineError::Parse("unexpected raw syngas species".into())),
        };
        push(name, typed, f.v(raw, &format!("raw_{name}"))?);
    }
    push(
        "SG-2 mass",
        case5::SYNGAS_LB_H,
        f.v("case5_SG-2", "mass_flow")?,
    );
    push(
        "reformer exit T",
        case5::REFORMER_OUT_T_F,
        f.v("case5_REFORM2_out", "T")?,
    );
    push(
        "reformer exit P",
        case5::REFORMER_OUT_P_PSI,
        f.v("case5_REFORM2_out", "P")?,
    );
    push("NG-10 T", case5::REFORMER_IN_T_F, f.v("case5_NG-10", "T")?);
    push(
        "NG-10 P",
        case5::REFORMER_IN_P_PSI,
        f.v("case5_NG-10", "P")?,
    );
    for (block, typed) in [
        ("REFORM-1", case5::DUTY_REFORM_1),
        ("REFORM-2", case5::DUTY_REFORM_2),
        ("HPS-GEN1", case5::DUTY_HPS_GEN1),
        ("WGS-EX1", case5::DUTY_WGS_EX1),
        ("WGS-EX2", case5::DUTY_WGS_EX2),
        ("WGS-EX3", case5::DUTY_WGS_EX3),
        ("WGS-EX4", case5::DUTY_WGS_EX4),
        ("KO-DRM-1", case5::DUTY_KO_DRM),
    ] {
        let fixed = f.v(&format!("case5_{block}"), "duty")?.abs();
        push(block, typed, fixed);
    }
    push(
        "HTS exit T",
        case5::HTS_OUT_T_F,
        f.v(shift, "HTS_outlet_T")?,
    );
    push(
        "HTS exit CO",
        case5::HTS_OUT_CO_DRY_PCT,
        f.v(shift, "HTS_outlet_CO_dry")?,
    );
    push(
        "LTS exit T",
        case5::LTS_OUT_T_F,
        f.v(shift, "LTS_outlet_T")?,
    );
    push(
        "LTS exit CO",
        case5::LTS_OUT_CO_DRY_PCT,
        f.v(shift, "LTS_outlet_CO_dry")?,
    );
    push(
        "Selexol inlet CO2",
        case5::SELEXOL_IN_CO2_PCT,
        f.v(shift, "selexol_inlet_CO2")?,
    );
    push(
        "KO-LIQ-1 mass",
        case5::KO_LIQ_LB_H,
        f.v("case5_KO-LIQ-1", "mass_flow")?,
    );
    push(
        "SG-8 mass",
        case5::KO_GAS_LB_H,
        f.v("case5_SG-8", "mass_flow")?,
    );
    let differ: Vec<String> = pairs
        .iter()
        .filter(|(_, a, b)| a != b)
        .map(|(w, a, b)| format!("{w} typed {a} against {b}"))
        .collect();
    let first = f.get(raw, "raw_H2")?;
    out.push(
        Meta::new(
            "T961-FIXTURE",
            2,
            "Case 5 values typed in nush2_units::case5 against the fixture",
            &first.source,
            "A-50 PDF p74, A-53 PDF p77, A-55 PDF p79, A-56 PDF p80",
        )
        .use_(SAME)
        .unit("values differing")
        .cond(format!("{} values compared", pairs.len()))
        .flagged(
            Some(0.0),
            differ.len() as f64,
            0.0,
            differ.is_empty(),
            if differ.is_empty() {
                "Transcription check: the T961-3 to T961-8 rows compare against these typed \
                 values"
                    .to_string()
            } else {
                differ.join("; ")
            },
        ),
    );
    Ok(())
}

fn tube_limit_spec() -> TubeSpec {
    TubeSpec {
        tag: "LIMIT".into(),
        length_m: 40.0,
        cat_start_m: 0.0,
        bed_area_m2: 0.01,
        void: 0.45,
        dp_m: 0.005,
        bulk_density_kg_m3: 1300.0,
        sections: vec![Section {
            end_m: 40.0,
            up_he_w_m_k: 2.0e3,
            up_ret_w_m_k: 0.0,
            q_he_w_m: 0.0,
        }],
        heat: HeatModel::Sections,
        kinetics: Kinetics::default(),
        k_source: KSource::Zhang,
        bayonet: false,
        ergun: false,
        he_m_kg_s: 50.0,
        he_t_in_k: 1100.0,
        he_p_pa: 40.0e5,
        he_out_guess_k: None,
        ret_out_guess_k: None,
    }
}

fn eq_at(feed: &Stream, t_k: f64, p_pa: f64) -> Result<f64> {
    Ok(smr_eq::solve(
        feed,
        &SmrEqSpec {
            tag: "EQ".into(),
            t_out_k: t_k,
            p_out_pa: p_pa,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        },
    )?
    .conversion)
}

/// XF-2 to XF-4. XF-1 comes from `nush2_units::bench`.
pub fn xu_froment(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("xu_froment_1989.csv")?;
    let kh2o = f.get("t6", "A_KH2O")?;
    out.push(
        Meta::at(
            kh2o,
            "XF-2-KH2O",
            2,
            "KH2O pre-exponential in the engine",
            "1",
        )
        .use_(SAME)
        .cond("Reference activity")
        .flagged(
            Some(kh2o.value),
            xf::TABLE6.a_ads[3],
            0.0,
            xf::TABLE6.a_ads[3] == kh2o.value,
            "Exact. The engine refuses the Hoseinzade and Adams set (test xf_guards)",
        ),
    );
    let fresh = f.get("activity", "fresh_catalyst_factor")?;
    out.push(
        Meta::at(
            fresh,
            "XF-2-FRESH",
            2,
            "Fresh catalyst factor against the activity search limit",
            "1",
        )
        .use_(SAME)
        .cond("JAERI A-2 activity fit")
        .flagged(
            Some(fresh.value),
            jaeri::FIT_HIGH,
            0.0,
            jaeri::FIT_HIGH == fresh.value,
            "Exact. The factor is the upper limit of the A-2 activity search; the fitted \
                 activity sits at it and is the one disclosed tuned value",
        ),
    );

    let mut pairs: Vec<(String, f64, f64)> = Vec::new();
    for row in xf::TABLE5 {
        pairs.push((
            row.name.into(),
            row.value,
            f.v(&format!("t5_{}", row.name), row.name)?,
        ));
    }
    for (i, k) in ["k1", "k2", "k3"].iter().enumerate() {
        pairs.push((
            format!("A_{k}"),
            xf::TABLE6.a_k[i],
            f.v("t6", &format!("A_{k}"))?,
        ));
        let e = format!("E{}", i + 1);
        pairs.push((e.clone(), xf::TABLE6.e_k[i], f.v(&format!("t5_{e}"), &e)?));
    }
    for (i, k) in ["CO", "H2", "CH4", "H2O"].iter().enumerate() {
        pairs.push((
            format!("A_K{k}"),
            xf::TABLE6.a_ads[i],
            f.v("t6", &format!("A_K{k}"))?,
        ));
        let d = format!("dH_{k}");
        pairs.push((
            d.clone(),
            xf::TABLE6.dh_ads[i],
            f.v(&format!("t5_{d}"), &d)?,
        ));
    }
    let differ: Vec<String> = pairs
        .iter()
        .filter(|(_, a, b)| a != b)
        .map(|(w, a, b)| format!("{w} typed {a} against {b}"))
        .collect();
    out.push(
        Meta::new(
            "XF-2-TABLES",
            2,
            "Tables 5 and 6 typed in nush2_units::xf against the fixture",
            &kh2o.source,
            "Tables 5 and 6 PDF p7",
        )
        .unit("values differing")
        .use_(SAME)
        .cond(format!("{} values compared", pairs.len()))
        .flagged(
            Some(0.0),
            differ.len() as f64,
            0.0,
            differ.is_empty(),
            if differ.is_empty() {
                "Transcription check behind XF-1".to_string()
            } else {
                differ.join("; ")
            },
        ),
    );

    let reactions: [(&str, [(Sp, f64); 4]); 3] = [
        (
            "I",
            [
                (Sp::Co, 1.0),
                (Sp::H2, 3.0),
                (Sp::Ch4, -1.0),
                (Sp::H2o, -1.0),
            ],
        ),
        (
            "II",
            [
                (Sp::Co2, 1.0),
                (Sp::H2, 1.0),
                (Sp::Co, -1.0),
                (Sp::H2o, -1.0),
            ],
        ),
        (
            "III",
            [
                (Sp::Co2, 1.0),
                (Sp::H2, 4.0),
                (Sp::Ch4, -1.0),
                (Sp::H2o, -2.0),
            ],
        ),
    ];
    for (i, (name, nu)) in reactions.iter().enumerate() {
        for (t_label, t_k, tol) in [("298K", 298.15, 1.0), ("948K", 948.0, 2.0)] {
            let fix = f.get("t7", &format!("dH{}_{t_label}", i + 1))?;
            let mut dh = 0.0;
            for (s, v) in nu {
                dh += v * species::h_j_mol(*s, t_k)?;
            }
            out.push(
                Meta::at(
                    fix,
                    format!("XF-3-{name}-{t_label}"),
                    1,
                    "Reaction enthalpy",
                    "kJ/mol",
                )
                .cond(format!(
                    "Reaction {name} at {t_k} K, ideal gas, water as vapour"
                ))
                .judged(
                    fix.value,
                    dh / 1000.0,
                    tol,
                    if t_label == "298K" {
                        "Shomate and IF97 layer of nush2_props. Table 7 prints 298 K, run at \
                             298.15 K. Limit 1.0 kJ/mol"
                    } else {
                        "Shomate layer of nush2_props. Limit 2.0 kJ/mol"
                    },
                ),
            );
        }
    }

    let mut n = [0.0; NSP];
    n[Sp::Ch4.index()] = 0.75;
    n[Sp::H2o.index()] = 2.6;
    let feed = Stream::gas("F", 800.0, 30.0e5, n);
    let tube = smr_tube::solve(&tube_limit_spec(), &feed)?;
    let eq = eq_at(&feed, tube.outlet.t_k, tube.outlet.p_pa)?;
    out.push(
        Meta::new(
            "XF-4",
            2,
            "Exit conversion of a long tube against SMR-EQ",
            "Xu and Froment 1989 Part I",
            "No literature value: SMR-EQ at the tube outlet",
        )
        .unit("points")
        .cond(format!(
            "40 m isothermal tube, CH4 0.75 and H2O 2.6 mol/s, outlet {:.1} K",
            tube.outlet.t_k
        ))
        .judged(
            100.0 * eq,
            100.0 * tube.conversion,
            0.5,
            "Code check of the rate model at its equilibrium limit",
        ),
    );
    Ok(())
}

/// CH-1 and CH-2 on Chinchen Table 1.
pub fn chinchen(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("wgs_chinchen_1984.csv")?;
    let pellet = |fix: &Fix| -> Result<Pellet> {
        match fix.cond_text("catalyst")? {
            "15-5" => Ok(Pellet::S),
            "15-4" => Ok(Pellet::L),
            other => Err(EngineError::Parse(format!("unknown pellet {other}"))),
        }
    };
    // CH-2 stops at 350 C, so the points above it at 16 and 21 bar are not read.
    let used = |r: &Fix| {
        r.quantity == "k_empty_bed"
            && (r.has("P_bar", "31") || r.cond("T_C").is_ok_and(|t| t <= 350.0))
    };
    type Set = (Vec<(f64, f64)>, Option<Fix>);
    let mut sets: [Set; 2] = [(Vec::new(), None), (Vec::new(), None)];
    for fix in f.select(used) {
        let t = fix.cond("T_C")?;
        let i = usize::from(!fix.has("P_bar", "31"));
        let model = k_chinchen(t + 273.15, pellet(fix)?)?;
        sets[i].0.push((model, fix.value));
        sets[i].1.get_or_insert_with(|| fix.clone());
    }
    for (i, (id, subject, limit, data_use, cond, note)) in [
        (
            "CH-1",
            "Empty-bed rate constant, 31 bar fit",
            15.0,
            CALIBRATION,
            "31 bar, 310 to 450 C, pellets 15-5 and 15-4",
            "The engine interpolates these points in ln k against 1/T, so agreement holds by \
             construction",
        ),
        (
            "CH-2",
            "Empty-bed rate constant at 16 and 21 bar against the 31 bar fit",
            25.0,
            SAME,
            "16 and 21 bar, 310 to 350 C, where pore diffusion is absent",
            "Tests the pressure independence the HTS model assumes",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let pairs = &sets[i].0;
        let fix = sets[i]
            .1
            .clone()
            .ok_or_else(|| EngineError::Parse(format!("{id}: no points")))?;
        let a = aard_pct(pairs);
        let w = worst(pairs).unwrap_or((f64::NAN, f64::NAN));
        out.push(
            Meta::at(&fix, id, 2, subject, "% AARD")
                .use_(data_use)
                .cond(format!("{cond}, {} points", pairs.len()))
                .flagged(
                    None,
                    a,
                    limit,
                    a <= limit,
                    format!(
                        "{note}. Worst point: model {:.4} against {:.4} 1/s",
                        w.0, w.1
                    ),
                ),
        );
    }
    Ok(())
}

/// IA-1 to IA-5 and the HT-2 anomaly of TECDOC-1236.
pub fn tecdoc1236(out: &mut Vec<Row>) -> Result<()> {
    let f = Fixture::open("iaea_tecdoc_1236.csv")?;
    let b = "analysis_basis";
    let feed = flows(&[
        (Sp::Ch4, f.v(b, "feed_CH4")?),
        (Sp::C2h6, f.v(b, "feed_C2H6")?),
        (Sp::Co, f.v(b, "feed_CO")?),
        (Sp::H2o, f.v(b, "feed_H2O")?),
    ]);
    let feed_note = "Printed feed CH4 19.77, C2H6 0.17, CO 0.02, H2O 80.05 mol% (PDF p12), at the \
                     printed equilibrium temperature, which lies about 10 K below the exit";
    let mut run =
        |series: &str, id_tag: &str, species_ids: bool, x_id: Option<&str>| -> Result<()> {
            let xfix = f.get(series, "conversion_CH4_C2H6")?;
            let t_c = xfix.cond("T_eq_C")?;
            let p_bar = xfix.cond("P_exit_bar")?;
            let r = equilibrium::reform(&feed, t_c + 273.15, p_bar * BAR_PA, KSource::Zhang)?;
            let cond = format!("{t_c:.0} C, {p_bar} bar, Zhang K");
            if let Some(id) = x_id {
                out.push(
                    Meta::at(xfix, id, 1, "CH4 and C2H6 conversion", "%")
                        .cond(cond.clone())
                        .judged(xfix.value, 100.0 * r.conversion, 0.5, feed_note),
                );
            }
            if species_ids {
                for name in ["CH4", "H2", "CO", "CO2", "H2O"] {
                    let fix = f.get(series, &format!("product_{name}"))?;
                    out.push(
                        Meta::at(
                            fix,
                            format!("{id_tag}-{name}"),
                            1,
                            "Product gas, wet",
                            "mol%",
                        )
                        .cond(cond.clone())
                        .judged(
                            fix.value,
                            wet_pct(&r.n, sp_named(name)?)?,
                            0.5,
                            feed_note,
                        ),
                    );
                }
            }
            Ok(())
        };
    run(
        "t21_P41.9",
        "IA-1-800C-41.9BAR",
        false,
        Some("IA-1-800C-41.9BAR"),
    )?;
    run("t21_P20", "IA-3", true, Some("IA-1-800C-20BAR"))?;
    run("t22_T860", "IA-2-860C", true, Some("IA-1-860C-41.9BAR"))?;
    run("t22_T800", "IA-2-800C", true, None)?;
    run("t22_T825", "IA-2-825C", true, Some("IA-2-825C-X"))?;
    run("t22_T850", "IA-2-850C", true, Some("IA-2-850C-X"))?;

    let h = "httr_sr";
    let x = f.get(h, "conversion")?;
    let t = f.v(h, "process_T_max")?;
    let p = f.v(h, "process_P")?;
    let sc = f.v(h, "steam_to_carbon")?;
    let pure = flows(&[(Sp::Ch4, 1.0), (Sp::H2o, sc)]);
    let r = equilibrium::reform(&pure, t + 273.15, p * 1.0e6, KSource::Zhang)?;
    out.push(
        Meta::at(x, "IA-4", 1, "HTTR steam reformer conversion", "%")
            .cond(format!(
                "{t:.0} C, {p} MPa, S/C {sc}, pure CH4 at equilibrium"
            ))
            .judged(
                x.value,
                100.0 * r.conversion,
                5.0,
                "The report prints about 68%, so the limit is 5 points; methane stands in for \
                 the natural gas",
            ),
    );

    let band = f.get("eva", "conversion_below_eq_max")?;
    let low = f.v("eva", "conversion_below_eq_min")?;
    let t = jaeri::run(&A2, A2_FIT_ACTIVITY)?;
    let end = t
        .nodes
        .last()
        .ok_or_else(|| EngineError::Parse("JAERI A-2 run has no nodes".into()))?;
    let gap = 100.0 * (eq_at(&A2.feed(), end.t_k, end.p_pa)? - end.conversion);
    out.push(
        Meta::at(
            band,
            "IA-5",
            2,
            "Exit conversion below equilibrium",
            "points",
        )
        .cond(format!(
            "JAERI A-2 tube at activity {A2_FIT_ACTIVITY}, tip {:.1} C",
            end.t_k - 273.15
        ))
        .flagged(
            Some(band.value),
            gap,
            band.value,
            gap <= band.value,
            format!(
                "Plausibility: EVA ran {low} to {} points below equilibrium, and the row \
                     passes at or below the upper end",
                band.value
            ),
        ),
    );

    let d = "htr10_reformer";
    let q = f.get(d, "reformer_power")?;
    let m = f.v(d, "He_flow")?;
    let t_in = f.v(d, "He_in_T")?;
    let t_out = f.v(d, "He_out_T")?;
    out.push(
        Meta::at(
            q,
            "HT-2",
            1,
            "HTR-10 reformer duty from the helium side",
            "MW",
        )
        .use_(SAME)
        .cond(format!("{m} kg/s helium, {t_in:.0} C to {t_out:.0} C"))
        .reported(
            Some(q.value),
            helium::duty_w(m, t_in + 273.15, t_out + 273.15)? / 1.0e6,
            "Source anomaly: the printed table does not close on its own helium side, so no \
                 gate uses this row",
        ),
    );
    Ok(())
}
