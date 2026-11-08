//! Reaction, tube and constant guards drawn from the OUTRAM PARK concept review
//! (concepts after OUTRAM PARK, AGPL-3.0, reference only; no OUTRAM code). Each
//! test names the derivation card it checks and prints what it computed.

use nush2_props::equilibrium::{self, KSource};
use nush2_props::mixture;
use nush2_props::species::{Sp as PSp, NSP as PNSP};
use nush2_units::case5::{self, TrainOpts};
use nush2_units::jaeri::{self, UBasis, A2, A2_FIT_ACTIVITY};
use nush2_units::smr_eq::{self, SmrEqSpec};
use nush2_units::smr_tube::{self, HeatModel, Section, TubeResult, TubeSpec};
use nush2_units::stream::{Sp, Stream, NSP};
use nush2_units::tube_case_b as case_b;
use nush2_units::tube_ht::{self, BedGas};
use nush2_units::xf::{Kinetics, Partial, R_KJ_MOL_K, TABLE6};

/// On a grid of 700 to 1200 K and 15 to 40 bar, for both K sources, every
/// Xu-Froment driving force divided by its forward term is at most 1e-9 at the
/// equilibrium composition of the same K. The forward term is the rate with K set
/// to infinity, so the ratio of the two rates is the ratio of the two terms.
#[test]
fn eq_d_16_driving_forces_vanish_on_the_equilibrium_grid() {
    let mut feed = [0.0; PNSP];
    feed[PSp::Ch4.index()] = 1.0;
    feed[PSp::H2o.index()] = 3.0;
    let kin = Kinetics::default();
    let mut worst = 0.0f64;
    for src in [KSource::Zhang, KSource::NistGibbs] {
        for i in 0..=5 {
            let t = 700.0 + 100.0 * f64::from(i);
            for j in 0..=5 {
                let p_bar = 15.0 + 5.0 * f64::from(j);
                let eq = equilibrium::reform(&feed, t, p_bar * 1.0e5, src).unwrap();
                let y = mixture::mole_fractions(&eq.n).unwrap();
                let pp = |s: PSp| y[s.index()] * p_bar;
                let part = Partial {
                    ch4: pp(PSp::Ch4),
                    h2o: pp(PSp::H2o),
                    h2: pp(PSp::H2),
                    co: pp(PSp::Co),
                    co2: pp(PSp::Co2),
                };
                let (k1, k2) = src.pair(t).unwrap();
                let at_eq = kin.rates(t, &part, k1, k2).unwrap();
                let fwd = kin.rates(t, &part, f64::INFINITY, f64::INFINITY).unwrap();
                for (a, b) in [(at_eq.r1, fwd.r1), (at_eq.r2, fwd.r2), (at_eq.r3, fwd.r3)] {
                    let r = (a / b).abs();
                    worst = worst.max(r);
                    assert!(r <= 1e-9, "{src:?} {t} K {p_bar} bar: {r:e}");
                }
            }
        }
    }
    println!("worst driving force over forward term on the grid: {worst:.2e}");
}

fn isothermal_tube(length_m: f64, activity: f64) -> TubeSpec {
    TubeSpec {
        tag: "LIMIT".into(),
        length_m,
        cat_start_m: 0.0,
        bed_area_m2: 0.01,
        void: 0.45,
        dp_m: 0.005,
        bulk_density_kg_m3: 1300.0,
        sections: vec![Section {
            end_m: length_m,
            up_he_w_m_k: 2.0e3,
            up_ret_w_m_k: 0.0,
            q_he_w_m: 0.0,
        }],
        heat: HeatModel::Sections,
        kinetics: Kinetics::with_activity(activity).unwrap(),
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

fn limit_feed() -> Stream {
    let mut n = [0.0; NSP];
    n[Sp::Ch4 as usize] = 0.75;
    n[Sp::H2o as usize] = 2.6;
    Stream::gas("F", 800.0, 30.0e5, n)
}

/// Gap between a tube's conversion and SMR-EQ at the tube's own outlet state.
fn eq_gap(t: &TubeResult, feed: &Stream) -> f64 {
    let eq = smr_eq::solve(
        feed,
        &SmrEqSpec {
            tag: "EQ".into(),
            t_out_k: t.outlet.t_k,
            p_out_pa: t.outlet.p_pa,
            k_source: KSource::Zhang,
            outlet_tag: "OUT".into(),
        },
    )
    .unwrap();
    t.conversion - eq.conversion
}

/// Long-bed half: the equilibrium limit belongs to thermodynamics, so on
/// the isothermal limit tube a long bed lands on SMR-EQ at its own outlet state
/// whatever the activity, and kinetics set only how fast it gets there. The review
/// band is 1e-4 on CH4 conversion between activity 1 over 40 m and activity 0.1
/// over 400 m.
///
/// Finding: at activity 10 the adaptive march stops at z = 0 with its step floor
/// (`NotConverged`), because the seeded inlet is too stiff for the step limits. The
/// test records which outcome it met; an error is the declared policy, a panic or a
/// value past equilibrium is not.
#[test]
fn eq_d_21_long_bed_approach_is_set_by_thermodynamics() {
    let feed = limit_feed();
    let mut out = Vec::new();
    for (name, spec) in [
        ("activity 1, 40 m", isothermal_tube(40.0, 1.0)),
        ("activity 0.1, 400 m", isothermal_tube(400.0, 0.1)),
    ] {
        let t = smr_tube::solve(&spec, &feed).unwrap();
        let gap = eq_gap(&t, &feed);
        println!(
            "{name}: X {:.6}, exit {:.3} K, X - X_eq {gap:+.2e}",
            t.conversion, t.outlet.t_k
        );
        assert!(
            t.conversion.is_finite() && gap.abs() <= 1e-6,
            "{name}: {gap}"
        );
        out.push(t.conversion);
    }
    let change = out[1] - out[0];
    println!("change with activity /10 and length x10: {change:+.2e} (review band 1e-4)");
    assert!(change.abs() < 1e-4, "{change}");
    match smr_tube::solve(&isothermal_tube(40.0, 10.0), &feed) {
        Ok(t) => {
            let gap = eq_gap(&t, &feed);
            println!(
                "activity 10, 40 m: X {:.6}, X - X_eq {gap:+.2e}",
                t.conversion
            );
            assert!(gap.abs() <= 1e-6, "{gap}");
            assert!((t.conversion - out[0]).abs() < 1e-4);
        }
        Err(e) => {
            println!("activity 10, 40 m: refused, {e}");
            assert!(
                matches!(e, nush2_numerics::EngineError::NotConverged(_)),
                "{e}"
            );
        }
    }
}

/// Hard ceilings. Hydrogen out of any unit stays at or below the
/// stoichiometric limit of its feed (H2 + CO + (3n+1) CnH2n+2), every mole fraction
/// is non-negative, they sum to 1 within 1e-12, and the reformer duty is positive.
fn h2_ceiling(s: &Stream) -> f64 {
    let n = &s.n;
    let hc = [
        (Sp::Ch4, 1.0),
        (Sp::C2h6, 2.0),
        (Sp::C3h8, 3.0),
        (Sp::Nc4h10, 4.0),
        (Sp::Nc5h12, 5.0),
    ];
    n[Sp::H2 as usize]
        + n[Sp::Co as usize]
        + hc.iter()
            .map(|(sp, c)| (3.0 * c + 1.0) * n[*sp as usize])
            .sum::<f64>()
}

fn fractions_are_sane(s: &Stream) {
    let y = s.wet_fractions().unwrap();
    assert!(y.iter().all(|v| *v >= 0.0), "{}: {y:?}", s.tag);
    let sum: f64 = y.iter().sum();
    assert!((sum - 1.0).abs() <= 1e-12, "{}: sum {sum}", s.tag);
}

#[test]
fn eq_a_05_hydrogen_ceiling_and_fractions() {
    let feed = case5::reformer_feed().unwrap();
    let ceiling = h2_ceiling(&feed);
    for src in [KSource::Zhang, KSource::NistGibbs] {
        let r = case5::reformer(src).unwrap();
        assert!(r.duty_w > 0.0, "{src:?} duty {}", r.duty_w);
        fractions_are_sane(&r.outlet);
        assert!(r.outlet.flow(Sp::H2) <= ceiling, "{src:?} reformer");
        let train = case5::train(&r.outlet, &TrainOpts::sized_on_ieaghg(src).unwrap()).unwrap();
        for s in [&train.hts.outlet, &train.lts.outlet, &train.drum.vapour] {
            fractions_are_sane(s);
            let h2 = s.flow(Sp::H2);
            println!(
                "{src:?} {}: H2 {h2:.4} mol/s against ceiling {ceiling:.4}",
                s.tag
            );
            assert!(h2 <= ceiling, "{src:?} {}", s.tag);
        }
    }
    let (_, tube) = case_b::run(case_b::HA_TUBES, A2_FIT_ACTIVITY).unwrap();
    let tube_feed = case_b::feed_total().unwrap();
    fractions_are_sane(&tube.outlet);
    assert!(tube.outlet.flow(Sp::H2) <= h2_ceiling(&tube_feed));
    assert!(tube.process_heat_w > 0.0);
}

/// Le Chatelier holds for SMR-EQ at the Case 5 feed. CH4 conversion falls
/// strictly with pressure, rises strictly with temperature and with steam to
/// carbon. The kinetic tube is not asserted, since rate and heat transfer limits may
/// reverse a trend; its pressure trend is printed as a finding.
#[test]
fn eq_d_11_le_chatelier_on_smr_eq() {
    let feed = case5::reformer_feed().unwrap();
    let base = case5::reformer(KSource::Zhang).unwrap();
    let (t0, p0) = (base.outlet.t_k, base.outlet.p_pa);
    let x = |f: &Stream, t: f64, p: f64| {
        smr_eq::solve(
            f,
            &SmrEqSpec {
                tag: "EQ".into(),
                t_out_k: t,
                p_out_pa: p,
                k_source: KSource::Zhang,
                outlet_tag: "OUT".into(),
            },
        )
        .unwrap()
        .conversion
    };
    let by_p: Vec<f64> = (0..6)
        .map(|i| x(&feed, t0, (15.0 + 5.0 * f64::from(i)) * 1.0e5))
        .collect();
    let by_t: Vec<f64> = (0..6)
        .map(|i| x(&feed, 973.15 + 40.0 * f64::from(i), p0))
        .collect();
    let carbon: f64 = [
        (Sp::Ch4, 1.0),
        (Sp::C2h6, 2.0),
        (Sp::C3h8, 3.0),
        (Sp::Nc4h10, 4.0),
        (Sp::Nc5h12, 5.0),
    ]
    .iter()
    .map(|(sp, c)| c * feed.flow(*sp))
    .sum();
    let by_sc: Vec<f64> = (0..6)
        .map(|i| {
            let mut f = feed.clone();
            f.n[Sp::H2o as usize] = (2.5 + 0.3 * f64::from(i)) * carbon;
            x(&f, t0, p0)
        })
        .collect();
    println!("SMR-EQ X by pressure 15 to 40 bar: {by_p:.5?}");
    println!("SMR-EQ X by temperature 973 to 1173 K: {by_t:.5?}");
    println!("SMR-EQ X by S/C 2.5 to 4.0: {by_sc:.5?}");
    assert!(by_p.windows(2).all(|w| w[1] < w[0]));
    assert!(by_t.windows(2).all(|w| w[1] > w[0]));
    assert!(by_sc.windows(2).all(|w| w[1] > w[0]));
}

/// On the two heated tubes the engine uses (Case B on the HA
/// tube, JAERI A-2): helium enters at z = L at its prescribed temperature, the
/// process gas enters at z = 0 at its feed pressure, the march ends exactly at the
/// tube length, helium stays above the bed at every node, the helium outlet lies
/// between the two inlets, and the helium duty is m cp dT.
#[test]
fn eq_d_28_tube_inlets_ends_and_bounds() {
    let (_, cb) = case_b::run(case_b::HA_TUBES, A2_FIT_ACTIVITY).unwrap();
    let cb_spec = case_b::spec(case_b::HA_TUBES, A2_FIT_ACTIVITY).unwrap();
    let cb_feed = case_b::feed_total().unwrap();
    let ja = jaeri::run(&A2, A2_FIT_ACTIVITY).unwrap();
    let ja_spec = jaeri::spec(&A2, A2_FIT_ACTIVITY, UBasis::Calculated).unwrap();
    let ja_feed = A2.feed();
    for (name, t, spec, feed) in [
        ("Case B", &cb, &cb_spec, &cb_feed),
        ("JAERI A-2", &ja, &ja_spec, &ja_feed),
    ] {
        let first = t.nodes.first().unwrap();
        let last = t.nodes.last().unwrap();
        assert!(
            (last.z_m - spec.length_m).abs() <= 1e-12 * spec.length_m,
            "{name} end"
        );
        assert!(first.z_m.abs() <= 1e-12 * spec.length_m, "{name} start");
        assert!(
            (last.t_he_k / spec.he_t_in_k - 1.0).abs() <= 1e-9,
            "{name}: helium at z = L {} against {}",
            last.t_he_k,
            spec.he_t_in_k
        );
        assert!(
            (first.p_pa / feed.p_pa - 1.0).abs() <= 1e-9,
            "{name} feed pressure"
        );
        if spec.bayonet {
            // The return gas turns at z = L at the bed exit temperature.
            assert!(
                (last.t_ret_k / last.t_k - 1.0).abs() <= 1e-9,
                "{name} turnaround"
            );
        }
        let cross = t
            .nodes
            .iter()
            .filter(|n| n.z_m >= spec.cat_start_m && n.t_he_k <= n.t_k)
            .count();
        assert_eq!(
            cross, 0,
            "{name}: helium at or below the bed at {cross} nodes"
        );
        assert!(
            feed.t_k <= t.he_out_k && t.he_out_k <= spec.he_t_in_k,
            "{name} He out"
        );
        assert!(
            feed.t_k <= t.outlet.t_k && t.outlet.t_k <= spec.he_t_in_k,
            "{name} out"
        );
        let m_cp_dt =
            nush2_props::helium::duty_w(spec.he_m_kg_s, spec.he_t_in_k, t.he_out_k).unwrap();
        assert!(
            (t.he_heat_w / m_cp_dt - 1.0).abs() <= 1e-6,
            "{name} helium duty"
        );
        println!(
            "{name}: {} nodes, end {:.12} m, He in {:.3} K, He out {:.3} K, process out \
             {:.3} K, He duty {:.4} kW",
            t.nodes.len(),
            last.z_m,
            last.t_he_k,
            t.he_out_k,
            t.outlet.t_k,
            t.he_heat_w / 1.0e3
        );
    }
}

/// Defaults hold through the ordinary constructors. Zhang is the default K
/// source, Table 6 runs at activity 1.0 until a fit sets it, the JAERI benchmarks
/// run with the bayonet, and radiation carries a nonzero share of the bed
/// coefficient at 1073 K on the Case B bed. The gas state is a representative one
/// for the share, not a literature value.
#[test]
fn eq_d_32_defaults_are_pinned() {
    assert_eq!(KSource::default(), KSource::Zhang);
    assert_eq!(Kinetics::default().activity(), 1.0);
    assert!(
        jaeri::spec(&A2, A2_FIT_ACTIVITY, UBasis::Calculated)
            .unwrap()
            .bayonet
    );
    let spec = case_b::spec(case_b::HA_TUBES, A2_FIT_ACTIVITY).unwrap();
    assert_eq!(spec.k_source, KSource::Zhang);
    let HeatModel::Correlated(c) = spec.heat else {
        panic!("Case B tube is expected on correlated heat transfer");
    };
    let gas = BedGas {
        t_c: 800.0,
        lambda_g: 0.15,
        nu_m2_s: 6.0e-6,
        prandtl: 0.97,
        u0_m_s: 0.6,
    };
    let with = tube_ht::alpha_i(&c.bed, &gas).unwrap();
    let mut dark = c.bed;
    dark.emissivity = 1.0e-12;
    let without = tube_ht::alpha_i(&dark, &gas).unwrap();
    let share = 1.0 - without / with;
    println!("radiation share of the bed coefficient at 1073 K: {share:.4}");
    assert!(share > 0.0, "{share}");
}

/// One gas constant per purpose. The MEA prototype's 8.314462 equals the
/// exact SI value to 1e-6. Xu and Froment name R in their notation (Part I, PDF p8)
/// and print no value, so no page fixes it; `xf` keeps 8.314e-3 kJ/mol/K, and the
/// change in k1 at 800 K had it been the exact value is printed and bounded.
#[test]
fn eq_d_13_gas_constants_are_exact_or_explained() {
    let exact = nush2_props::units::R_GAS;
    let mea = nush2_mea::pstar::R_GAS;
    assert!((mea / exact - 1.0).abs() < 1e-6, "{mea}");
    let xf_rel = R_KJ_MOL_K * 1.0e3 / exact - 1.0;
    let t = 800.0;
    let k1 = |r: f64| TABLE6.a_k[0] * (-TABLE6.e_k[0] / (r * t)).exp();
    let change = k1(exact * 1.0e-3) / k1(R_KJ_MOL_K) - 1.0;
    println!(
        "R in xf {R_KJ_MOL_K} kJ/mol/K, {xf_rel:+.2e} from exact; k1 at 800 K moves \
         {change:+.2e} on the exact value"
    );
    assert!(xf_rel.abs() < 1e-4);
    assert!(change.abs() < 3e-3);
}
