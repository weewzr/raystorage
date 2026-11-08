//! One-at-a-time sweeps. Overrides stay in memory. Corner runs carry no
//! probability. Sweeps that need a later milestone (reformer outlet temperature)
//! or a different metric (backup firing) are absent until those exist.

use crate::anchored::{Balance, BalanceOpts, Model, Scenario};
use crate::economics::{CostQuery, Stage};
use crate::params::ParamSet;
use nush2_numerics::Result;

/// Metric names, freeze section 8 plus the L5 pair of the rebuilt headline.
pub const METRICS: [&str; 8] = [
    "A_T_Y",
    "AC_L3_25",
    "AC_L3_60",
    "AC_L5_25",
    "AC_L5_60",
    "R_NEEDED",
    "MARGIN_210",
    "NG_MMSCFD",
];

/// One row of the results file.
#[derive(Debug, Clone, PartialEq)]
pub struct SensRow {
    pub sweep_id: &'static str,
    pub run_id: String,
    pub parameter: &'static str,
    pub unit: &'static str,
    pub base_value: f64,
    pub test_value: f64,
    pub metric: &'static str,
    pub result: f64,
    pub delta: f64,
    pub note: &'static str,
}

/// How one point differs from the central run.
#[derive(Clone, Copy)]
struct Point<'a> {
    params: &'a [(&'a str, f64)],
    scn: Scenario,
    tweak: fn(&Model, &mut BalanceOpts),
    stage: Stage,
    n: Option<f64>,
    r_at_need: bool,
    baseline: Baseline,
}

#[derive(Clone, Copy, PartialEq)]
enum Baseline {
    Inl,
    Cho,
    Ieaghg,
}

fn no_tweak(_: &Model, _: &mut BalanceOpts) {}

const CENTRAL: Point<'static> = Point {
    params: &[],
    scn: Scenario::RecF,
    tweak: no_tweak,
    stage: Stage::Boak,
    n: None,
    r_at_need: false,
    baseline: Baseline::Inl,
};

fn evaluate(base: &ParamSet, pt: Point<'_>) -> Result<[f64; 8]> {
    let mut p = base.clone();
    for (id, v) in pt.params {
        p.set(id, *v)?;
    }
    let m = Model::new(&p)?;
    let s = m.s_b;
    let mut o = m.opts(s);
    (pt.tweak)(&m, &mut o);
    let b = m.balance(pt.scn, &o)?;
    let h2 = m.h2_a_kgh * s * b.c.h_scale * m.hours / 1000.0;
    let a = m.emissions(&b.c, s).a_atm
        + match pt.baseline {
            Baseline::Inl => 0.0,
            Baseline::Cho => (m.base_cho - m.base_inl) * h2,
            Baseline::Ieaghg => (m.base_ieaghg - m.base_inl) * h2,
        };
    let r = if pt.r_at_need { b.r } else { m.r_rated };
    let n = pt.n.unwrap_or(m.n_linear);
    let ac = |life: f64, b: &Balance| -> Result<(f64, f64)> {
        let l = m.ledger(b, &CostQuery::central(pt.stage, life, n, r))?;
        Ok((l.l3() * 1e6 / a, l.l5() * 1e6 / a))
    };
    let (l3_25, l5_25) = ac(m.lives[0], &b)?;
    let (l3_60, l5_60) = ac(m.lives[2], &b)?;
    Ok([
        a,
        l3_25,
        l3_60,
        l5_25,
        l5_60,
        b.r,
        m.r_rated - b.r,
        b.c.ng * s,
    ])
}

/// What a sweep point is called in the results.
struct Label<'a> {
    id: &'static str,
    run_id: &'a str,
    parameter: &'static str,
    unit: &'static str,
    base_value: f64,
    test_value: f64,
}

fn lab<'a>(
    id: &'static str,
    run_id: &'a str,
    parameter: &'static str,
    unit: &'static str,
    base_value: f64,
    test_value: f64,
) -> Label<'a> {
    Label {
        id,
        run_id,
        parameter,
        unit,
        base_value,
        test_value,
    }
}

struct Sweep<'a> {
    rows: Vec<SensRow>,
    base: &'a ParamSet,
    central: [f64; 8],
}

impl Sweep<'_> {
    fn push(&mut self, l: Label<'_>, pt: Point<'_>) -> Result<()> {
        let r = evaluate(self.base, pt)?;
        for (i, metric) in METRICS.iter().enumerate() {
            self.rows.push(SensRow {
                sweep_id: l.id,
                run_id: l.run_id.to_string(),
                parameter: l.parameter,
                unit: l.unit,
                base_value: l.base_value,
                test_value: l.test_value,
                metric,
                result: r[i],
                delta: r[i] - self.central[i],
                note: "anchored port; a corner run carries no probability",
            });
        }
        Ok(())
    }

    fn values(
        &mut self,
        id: &'static str,
        param: &'static str,
        parameter: &'static str,
        unit: &'static str,
        base_value: f64,
        test: &[f64],
    ) -> Result<()> {
        for v in test {
            let set = [(param, *v)];
            let pt = Point {
                params: &set,
                ..CENTRAL
            };
            self.push(lab(id, "B_REC_F", parameter, unit, base_value, *v), pt)?;
        }
        Ok(())
    }
}

/// Every sweep the anchored port can run.
pub fn sweeps(base: &ParamSet) -> Result<Vec<SensRow>> {
    let m = Model::new(base)?;
    let central = evaluate(base, CENTRAL)?;
    let mut w = Sweep {
        rows: Vec::new(),
        base,
        central,
    };
    w.values(
        "CAPTURE",
        "ETA_MEA",
        "Per-pass capture",
        "-",
        m.eta_mea,
        &[0.80, 0.85, 0.90, 0.95],
    )?;
    w.values(
        "AVAILABILITY",
        "AVAILABILITY",
        "Availability",
        "-",
        m.avail,
        &[0.65, 0.70, 0.75, 0.85, 0.90],
    )?;
    w.values(
        "SRD",
        "SRD_CENTRAL",
        "Specific reboiler duty",
        "GJ/t",
        m.srd.central,
        &[m.srd.low, m.srd.central, m.srd.high],
    )?;
    w.values(
        "GAS",
        "GAS_CENTRAL",
        "Gas price",
        "S$/GJ",
        m.gas.central,
        &[m.gas.low, m.gas.central, m.gas.high],
    )?;
    w.values(
        "TS",
        "TS_CENTRAL",
        "CO2 transport and storage",
        "S$/t",
        m.ts.central,
        &[m.ts.low, m.ts.central, m.ts.high],
    )?;
    w.values(
        "ETA_POWER",
        "ETA_POWER",
        "Power conversion efficiency",
        "-",
        m.eta_pwr,
        &[m.eta_pwr, m.eta_pwr_alt],
    )?;
    let (dp_sec, _) = m.circ_dp_central();
    let dp_high = base
        .entries()
        .find(|e| e.id == "CIRC_CENTRAL_DP_SEC_KPA")
        .and_then(|e| e.high)
        .unwrap_or(dp_sec);
    w.values(
        "SEC_DP",
        "CIRC_CENTRAL_DP_SEC_KPA",
        "Secondary helium pressure drop",
        "kPa",
        dp_sec,
        &[dp_sec, dp_high],
    )?;
    let approach_high = base
        .entries()
        .find(|e| e.id == "IHX_APPROACH_K")
        .and_then(|e| e.high)
        .unwrap_or(m.ihx_approach);
    w.values(
        "IHX_APPROACH",
        "IHX_APPROACH_K",
        "IHX approach",
        "K",
        m.ihx_approach,
        &[m.ihx_approach, approach_high],
    )?;

    // Heat basis, with helium flow following duty.
    let q_ahn_a = m.q_proc_ahn_b / m.s_b;
    let he_ahn = m.he_flow * q_ahn_a / m.q_proc;
    for (q_b, set) in [
        (
            m.q_proc * m.s_b,
            [("Q_PROCESS", m.q_proc), ("HE_FLOW", m.he_flow)],
        ),
        (
            m.q_proc_ahn_b,
            [("Q_PROCESS", q_ahn_a), ("HE_FLOW", he_ahn)],
        ),
    ] {
        let pt = Point {
            params: &set,
            ..CENTRAL
        };
        w.push(
            lab(
                "HEAT_BASIS",
                "B_REC_F",
                "Process heat at Case B",
                "MWth",
                m.q_proc * m.s_b,
                q_b,
            ),
            pt,
        )?;
    }
    // Same heat basis with helium flow held at the INL value.
    let held = [("Q_PROCESS", q_ahn_a)];
    w.push(
        lab(
            "HEAT_BASIS_HE_HELD",
            "B_REC_F",
            "Process heat at Case B, helium flow held",
            "MWth",
            m.q_proc * m.s_b,
            m.q_proc_ahn_b,
        ),
        Point {
            params: &held,
            ..CENTRAL
        },
    )?;

    for n in [m.n_linear, m.n_team, m.n_ngnp] {
        let pt = Point {
            n: Some(n),
            ..CENTRAL
        };
        w.push(
            lab(
                "SIZE_EXPONENT",
                "B_REC_F",
                "Size exponent",
                "-",
                m.n_linear,
                n,
            ),
            pt,
        )?;
    }
    for (i, stage) in Stage::ALL.iter().enumerate() {
        let pt = Point {
            stage: *stage,
            ..CENTRAL
        };
        let run = format!("B_REC_F_{}", stage.id());
        w.push(lab("STAGE", &run, "Stage index", "-", 1.0, i as f64), pt)?;
    }
    let need = m.balance(Scenario::RecF, &m.opts(m.s_b))?.r;
    w.push(
        lab(
            "CHARGED",
            "B_REC_F",
            "Reactor charged",
            "MWth",
            m.r_rated,
            m.r_rated,
        ),
        CENTRAL,
    )?;
    w.push(
        lab(
            "CHARGED",
            "B_REC_F",
            "Reactor charged",
            "MWth",
            m.r_rated,
            need,
        ),
        Point {
            r_at_need: true,
            ..CENTRAL
        },
    )?;
    for (id, value, baseline) in [
        ("BASE_INL", m.base_inl, Baseline::Inl),
        ("BASE_CHO", m.base_cho, Baseline::Cho),
        ("BASE_IEAGHG", m.base_ieaghg, Baseline::Ieaghg),
    ] {
        let pt = Point {
            baseline,
            ..CENTRAL
        };
        let run = format!("B_REC_F_{id}");
        w.push(
            lab("BASELINE", &run, "Baseline", "kg/kg", m.base_inl, value),
            pt,
        )?;
    }
    for (i, scn) in [
        Scenario::RecF,
        Scenario::RecFix,
        Scenario::CombH,
        Scenario::CombN,
        Scenario::Export,
    ]
    .iter()
    .enumerate()
    {
        let pt = Point {
            scn: *scn,
            ..CENTRAL
        };
        let run = format!("B_{}", scn.id());
        w.push(
            lab("TAIL_GAS", &run, "Scenario index", "-", 0.0, i as f64),
            pt,
        )?;
    }
    fn zld_on(_: &Model, o: &mut BalanceOpts) {
        o.use_zld = true;
    }
    fn primary_net(_: &Model, o: &mut BalanceOpts) {
        o.eta_net_of_primary_circ = true;
    }
    fn amdea(m: &Model, o: &mut BalanceOpts) {
        o.eta_c = m.amdea_eta;
        o.srd = m.amdea_srd;
    }
    w.push(
        lab("ZLD", "B_REC_F", "ZLD steam freed", "flag", 0.0, 0.0),
        CENTRAL,
    )?;
    w.push(
        lab("ZLD", "B_REC_F", "ZLD steam freed", "flag", 0.0, 1.0),
        Point {
            tweak: zld_on,
            ..CENTRAL
        },
    )?;
    w.push(
        lab(
            "ETA_NET_OF_PRIMARY",
            "B_REC_F",
            "Efficiency net of primary circulation",
            "flag",
            0.0,
            1.0,
        ),
        Point {
            tweak: primary_net,
            ..CENTRAL
        },
    )?;
    w.push(
        lab("SOLVENT", "B_REC_F", "Solvent MEA", "-", 0.0, 0.0),
        CENTRAL,
    )?;
    w.push(
        lab("SOLVENT", "B_INL_SEL", "Solvent Selexol", "-", 0.0, 1.0),
        Point {
            scn: Scenario::InlSel,
            ..CENTRAL
        },
    )?;
    w.push(
        lab("SOLVENT", "B_REC_F", "Solvent aMDEA", "-", 0.0, 2.0),
        Point {
            tweak: amdea,
            ..CENTRAL
        },
    )?;
    Ok(w.rows)
}
