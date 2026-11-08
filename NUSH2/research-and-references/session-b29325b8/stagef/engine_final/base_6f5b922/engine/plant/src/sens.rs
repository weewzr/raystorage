//! One-at-a-time sweeps, plan section 11, plus the cost and carbon sweeps that
//! build order section 5 item 5 moves out of the report builder. Overrides stay
//! in memory. Corner runs carry no probability.
//!
//! Most points run the anchored balance with one input changed. Cost points
//! change one ledger input through the cost query instead, so the balance is
//! unchanged.
//!
//! Flowsheet points (decisions S2, S8 and S14) take the Level 3 flowsheet
//! result and carry it into the anchored balance as a ratio to the flowsheet's
//! own 871 C Zhang run, the same factor method the term flips use. Process heat
//! moves the reactor need, and the gas and carbon flows move the ledger and the
//! emissions. The reboiler duty stays on the anchored captured flow. Their rows
//! also print the flowsheet's own numbers under metrics that start with `FS_`.
//!
//! Backup firing and the levers to the cost target have metrics of their own.
//! A point that errors writes `not_converged` rows with empty results, so the
//! sweep stays visible.

use crate::anchored::{Balance, BalanceOpts, Model, Scenario};
use crate::economics::{CostQuery, Stage};
use crate::params::ParamSet;
use nush2_numerics::{EngineError, Result};

/// Metric names, freeze section 8, the L5 pair of the rebuilt headline and the
/// DCF pair of method 2.
pub const METRICS: [&str; 10] = [
    "A_T_Y",
    "AC_L3_25",
    "AC_L3_60",
    "AC_L5_25",
    "AC_L5_60",
    "LAC_DCF_25",
    "LAC_DCF_60",
    "R_NEEDED",
    "MARGIN_210",
    "NG_MMSCFD",
];

/// Metrics a flowsheet point prints from the Level 3 run itself.
pub const FS_METRICS: [&str; 4] = [
    "FS_NG_MMSCFD",
    "FS_PROCESS_HEAT_MW",
    "FS_CAPTURED_KMOL_H",
    "FS_EMITTED_C_KMOL_H",
];

/// Status of rows that rest on a Level 3 flowsheet result.
pub const FLOWSHEET_STATUS: &str = "flowsheet_m8";
/// Status of rows whose point did not solve.
pub const NOT_CONVERGED: &str = "not_converged";

/// One row of the results file. `status` is set only where the row rests on
/// something other than the anchored balance; the exporter fills the rest.
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
    pub note: String,
    pub status: Option<&'static str>,
}

/// Level 3 numbers of one flowsheet run that a sweep carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsPoint {
    pub ng_mmscfd: f64,
    pub process_heat_mw: f64,
    pub captured_kmol_h: f64,
    /// Carbon of the nitrogen-rich cut burned on site, kmol C/h.
    pub emitted_c_kmol_h: f64,
}

impl FsPoint {
    fn values(&self) -> [f64; 4] {
        [
            self.ng_mmscfd,
            self.process_heat_mw,
            self.captured_kmol_h,
            self.emitted_c_kmol_h,
        ]
    }
}

/// Flowsheet results the sweeps need, computed once by the exporter. `None`
/// marks a run that did not converge.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FsInputs {
    /// FS_B_REC_F: 871 C, Zhang constants.
    pub base: Option<FsPoint>,
    /// Case B REC_F at the S8 reformer exits, C.
    pub reformer: Vec<(f64, Option<FsPoint>)>,
    /// Case B REC_F with NIST Gibbs constants on every equilibrium unit.
    pub k_nist: Option<FsPoint>,
    /// CO2 in the process condensate S10 of FS_B_REC_F, kmol/h.
    pub condensate_co2_kmol_h: Option<f64>,
    /// Syngas heat above 130 C on the EX3 outlet S09A of FS_B_REC_F, MW.
    pub syngas_mw: Option<f64>,
}

/// Changes a point makes to the balance after it solves: factors on natural
/// gas, captured carbon and emitted carbon, and condensate CO2 moved from the
/// captured stream to the atmosphere, kmol/h at the plant.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Adjust {
    ng: f64,
    cap: f64,
    atm: f64,
    vent_kmol_h: f64,
}

const NO_ADJUST: Adjust = Adjust {
    ng: 1.0,
    cap: 1.0,
    atm: 1.0,
    vent_kmol_h: 0.0,
};

/// Ledger inputs a cost point changes through the query.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CostTweak {
    rx_mult: f64,
    capt_k: Option<f64>,
    fom: Option<f64>,
    /// Natural gas charged in the ledger, MMSCFD at Case A scale.
    ng_override: Option<f64>,
}

const NO_COST: CostTweak = CostTweak {
    rx_mult: 1.0,
    capt_k: None,
    fom: None,
    ng_override: None,
};

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
    adj: Adjust,
    cost: CostTweak,
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
    adj: NO_ADJUST,
    cost: NO_COST,
};

/// Apply the post-solve changes of a point.
fn adjust(m: &Model, b: &mut Balance, a: Adjust) {
    b.c.ng *= a.ng;
    b.c.cap *= a.cap;
    b.cap_t *= a.cap;
    b.c.burn *= a.atm;
    b.c.exported *= a.atm;
    if a.vent_kmol_h != 0.0 {
        let before = b.c.cap;
        b.c = m.vent_condensate(&b.c, b.s, a.vent_kmol_h);
        if before > 0.0 {
            b.cap_t *= b.c.cap / before;
        }
    }
}

/// The balance, model and abatement of one point.
struct Solved {
    m: Model,
    b: Balance,
    a: f64,
    r: f64,
    n: f64,
}

fn solve(base: &ParamSet, pt: Point<'_>) -> Result<Solved> {
    let mut p = base.clone();
    for (id, v) in pt.params {
        p.set(id, *v)?;
    }
    let m = Model::new(&p)?;
    let s = m.s_b;
    let mut o = m.opts(s);
    (pt.tweak)(&m, &mut o);
    let mut b = m.balance(pt.scn, &o)?;
    adjust(&m, &mut b, pt.adj);
    let h2 = m.h2_a_kgh * s * b.c.h_scale * m.hours / 1000.0;
    let a = m.emissions(&b.c, s).a_atm
        + match pt.baseline {
            Baseline::Inl => 0.0,
            Baseline::Cho => (m.base_cho - m.base_inl) * h2,
            Baseline::Ieaghg => (m.base_ieaghg - m.base_inl) * h2,
        };
    let r = if pt.r_at_need { b.r } else { m.r_rated };
    let n = pt.n.unwrap_or(m.n_linear);
    Ok(Solved { m, b, a, r, n })
}

fn query(pt: &Point<'_>, life: f64, n: f64, r: f64) -> CostQuery {
    CostQuery {
        rx_mult: pt.cost.rx_mult,
        capt_k: pt.cost.capt_k,
        fom: pt.cost.fom,
        ng_override: pt.cost.ng_override,
        ..CostQuery::central(pt.stage, life, n, r)
    }
}

fn evaluate(base: &ParamSet, pt: Point<'_>) -> Result<[f64; 10]> {
    let Solved { m, b, a, r, n } = solve(base, pt)?;
    let ac = |life: f64| -> Result<(f64, f64, f64)> {
        let q = query(&pt, life, n, r);
        let l = m.ledger(&b, &q)?;
        let d = m.dcf(&b, &q)?;
        // The DCF prices tonnes on the INL baseline; restate on this point's.
        Ok((
            l.l3() * 1e6 / a,
            l.l5() * 1e6 / a,
            d.lac * d.annual_abatement_t / a,
        ))
    };
    let (l3_25, l5_25, d25) = ac(m.lives[0])?;
    let (l3_60, l5_60, d60) = ac(m.lives[2])?;
    let ng = pt.cost.ng_override.unwrap_or(b.c.ng) * m.s_b;
    Ok([
        a,
        l3_25,
        l3_60,
        l5_25,
        l5_60,
        d25,
        d60,
        b.r,
        m.r_rated - b.r,
        ng,
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

const CORNER: &str = "a corner run carries no probability";

struct Sweep<'a> {
    rows: Vec<SensRow>,
    base: &'a ParamSet,
    central: [f64; 10],
}

impl Sweep<'_> {
    fn row(
        &mut self,
        l: &Label<'_>,
        metric: &'static str,
        v: f64,
        d: f64,
        note: &str,
        st: Option<&'static str>,
    ) {
        self.rows.push(SensRow {
            sweep_id: l.id,
            run_id: l.run_id.to_string(),
            parameter: l.parameter,
            unit: l.unit,
            base_value: l.base_value,
            test_value: l.test_value,
            metric,
            result: v,
            delta: d,
            note: note.to_string(),
            status: st,
        });
    }

    /// Rows of a point that did not solve: every metric, empty, with the reason.
    fn failed(&mut self, l: &Label<'_>, metrics: &[&'static str], why: &str) {
        for metric in metrics {
            self.row(
                l,
                metric,
                f64::NAN,
                f64::NAN,
                &format!("not converged: {why}"),
                Some(NOT_CONVERGED),
            );
        }
    }

    fn push_with(&mut self, l: Label<'_>, pt: Point<'_>, note: &str, st: Option<&'static str>) {
        match evaluate(self.base, pt) {
            Ok(r) => {
                for (i, metric) in METRICS.iter().enumerate() {
                    let d = r[i] - self.central[i];
                    self.row(&l, metric, r[i], d, note, st);
                }
            }
            Err(e) => self.failed(&l, &METRICS, &e.to_string()),
        }
    }

    fn push(&mut self, l: Label<'_>, pt: Point<'_>) {
        self.push_with(l, pt, CORNER, None);
    }

    fn values(
        &mut self,
        id: &'static str,
        param: &'static str,
        parameter: &'static str,
        unit: &'static str,
        base_value: f64,
        test: &[f64],
    ) {
        for v in test {
            let set = [(param, *v)];
            let pt = Point {
                params: &set,
                ..CENTRAL
            };
            self.push(lab(id, "B_REC_F", parameter, unit, base_value, *v), pt);
        }
    }

    /// A flowsheet point: its own Level 3 numbers, then the standard metrics
    /// with the flowsheet ratios carried into the anchored balance.
    fn flowsheet_point(
        &mut self,
        m: &Model,
        l: Label<'_>,
        fs_base: Option<FsPoint>,
        fs: Option<FsPoint>,
        note: &str,
    ) {
        let (Some(f0), Some(f)) = (fs_base, fs) else {
            let mut all: Vec<&'static str> = FS_METRICS.to_vec();
            all.extend(METRICS);
            self.failed(&l, &all, "the Level 3 flowsheet run did not converge");
            return;
        };
        for (i, metric) in FS_METRICS.iter().enumerate() {
            let (v, v0) = (f.values()[i], f0.values()[i]);
            self.row(&l, metric, v, v - v0, note, Some(FLOWSHEET_STATUS));
        }
        let ratio = |a: f64, b: f64| if b.abs() > 0.0 { a / b } else { 1.0 };
        let fq = ratio(f.process_heat_mw, f0.process_heat_mw);
        let set = [("Q_PROCESS", m.q_proc * fq), ("HE_FLOW", m.he_flow * fq)];
        let pt = Point {
            params: &set,
            adj: Adjust {
                ng: ratio(f.ng_mmscfd, f0.ng_mmscfd),
                cap: ratio(f.captured_kmol_h, f0.captured_kmol_h),
                atm: ratio(f.emitted_c_kmol_h, f0.emitted_c_kmol_h),
                vent_kmol_h: 0.0,
            },
            ..CENTRAL
        };
        self.push_with(l, pt, note, Some(FLOWSHEET_STATUS));
    }
}

/// Secant solve of `f(x) = target`, exact in one step for an affine `f`.
fn solve_for(f: impl Fn(f64) -> Result<f64>, x0: f64, x1: f64, target: f64) -> Result<f64> {
    let (mut a, mut fa) = (x0, f(x0)?);
    let (mut b, mut fb) = (x1, f(x1)?);
    for _ in 0..30 {
        if fb == fa {
            break;
        }
        let c = b - (fb - target) * (b - a) / (fb - fa);
        let fc = f(c)?;
        if (fc - target).abs() <= 1e-9 * target.abs().max(1.0) {
            return Ok(c);
        }
        (a, fa, b, fb) = (b, fb, c, fc);
    }
    Err(EngineError::NotConverged(format!(
        "lever solve for {target} stopped at {b}"
    )))
}

#[derive(Clone, Copy, PartialEq)]
enum Method {
    L5,
    Dcf,
}

/// Levers to the cost target one at a time, freeze section 6.6 recomputed on
/// the full cost set: L5 and the DCF, BOAK and FOAK, 25 y and 60 y, n = 1.0,
/// the rated reactor charged. Each lever is solved for directly.
///
/// eq: L.31
pub fn levers(base: &ParamSet) -> Result<Vec<SensRow>> {
    let m = Model::new(base)?;
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let a = m.emissions(&b.c, m.s_b).a_atm;
    let target = m.target_c;
    let cost = |method: Method, q: CostQuery| -> Result<f64> {
        Ok(match method {
            Method::L5 => m.ledger(&b, &q)?.l5() * 1e6 / a,
            Method::Dcf => m.dcf(&b, &q)?.lac,
        })
    };
    let mut rows = Vec::new();
    let note = "one lever at a time to the cost target; a value below zero means the lever \
                cannot close the gap alone";
    let mut push = |run: String,
                    parameter: &'static str,
                    unit: &'static str,
                    base_value: f64,
                    metric: &'static str,
                    v: f64| {
        rows.push(SensRow {
            sweep_id: "LEVER_100",
            run_id: run,
            parameter,
            unit,
            base_value,
            test_value: target,
            metric,
            result: v,
            delta: v - base_value,
            note: note.to_string(),
            status: None,
        });
    };
    for (method, mid) in [(Method::L5, "L5"), (Method::Dcf, "DCF")] {
        for stage in [Stage::Boak, Stage::Foak] {
            for life in [m.lives[0], m.lives[2]] {
                let run = format!("{mid}_{}_{life:.0}Y", stage.id());
                let q0 = CostQuery::central(stage, life, m.n_linear, m.r_rated);
                let c0 = cost(method, q0)?;
                push(
                    run.clone(),
                    "Cost at central inputs",
                    "S$/t",
                    target,
                    "COST_S_PER_T",
                    c0,
                );
                let gas = solve_for(
                    |g| cost(method, CostQuery { gas: Some(g), ..q0 }),
                    m.gas.low,
                    m.gas.high,
                    target,
                )?;
                push(
                    run.clone(),
                    "Gas price that brings the cost to the target",
                    "S$/GJ",
                    m.gas.central,
                    "GAS_NEEDED",
                    gas,
                );
                let mult = solve_for(
                    |x| cost(method, CostQuery { rx_mult: x, ..q0 }),
                    0.5,
                    1.0,
                    target,
                )?;
                push(
                    run.clone(),
                    "Reactor OCC that brings the cost to the target, before the stage factor",
                    "US$2019/kWt",
                    m.occ_kwt.central,
                    "OCC_NEEDED",
                    m.occ_kwt.central * mult,
                );
                let ts = solve_for(
                    |x| cost(method, CostQuery { ts: Some(x), ..q0 }),
                    m.ts.low,
                    m.ts.high,
                    target,
                )?;
                push(
                    run,
                    "T&S price that brings the cost to the target",
                    "S$/t",
                    m.ts.central,
                    "TS_NEEDED",
                    ts,
                );
            }
            let q = CostQuery::central(stage, m.lives[1], m.n_linear, b.r);
            let c0 = cost(
                method,
                CostQuery::central(stage, m.lives[0], m.n_linear, m.r_rated),
            )?;
            push(
                format!("{mid}_{}_40Y_NEED", stage.id()),
                "Reactor charged at the need with a 40 y life, against the 25 y rated cost",
                "S$/t",
                c0,
                "COST_S_PER_T",
                cost(method, q)?,
            );
        }
    }
    Ok(rows)
}

/// Life sweep at BOAK, n = 1.0, the rated reactor charged: both methods and
/// both LCOH at each life the ledger carries, against the 25 y values.
fn life_rows(m: &Model, b: &Balance) -> Result<Vec<SensRow>> {
    let a = m.emissions(&b.c, b.s).a_atm;
    let at = |life: f64| -> Result<[f64; 5]> {
        let q = CostQuery::central(Stage::Boak, life, m.n_linear, m.r_rated);
        let l = m.ledger(b, &q)?;
        let d = m.dcf(b, &q)?;
        Ok([
            l.l3() * 1e6 / a,
            l.l5() * 1e6 / a,
            d.lac,
            d.lcoh_nush2,
            d.lcoh_smr,
        ])
    };
    let names: [(&'static str, &'static str); 5] = [
        ("AC_L3", "S$/t"),
        ("AC_L5", "S$/t"),
        ("LAC_DCF", "S$/t"),
        ("LCOH_NUSH2", "S$/kg"),
        ("LCOH_SMR", "S$/kg"),
    ];
    let base = at(m.lives[0])?;
    let mut rows = Vec::new();
    for life in m.lives {
        let v = at(life)?;
        for (i, (metric, _)) in names.iter().enumerate() {
            rows.push(SensRow {
                sweep_id: "LIFE",
                run_id: "B_REC_F".into(),
                parameter: "Reactor life and DCF horizon",
                unit: "y",
                base_value: m.lives[0],
                test_value: life,
                metric,
                result: v[i],
                delta: v[i] - base[i],
                note: "BOAK, n = 1.0, 210 MWth charged; 25 y items bought again on a longer \
                       horizon"
                    .into(),
                status: None,
            });
        }
    }
    Ok(rows)
}

/// Backup firing over the outage hours, a sensitivity only (Q5). Route 1 and
/// route 2 are the TEV-953 conventional plant without and with capture; route 3
/// is this plant on a gas-fired heater, card K.20. Abatement counts against an
/// unabated SMR making the same extra H2.
fn backup_rows(m: &Model, b: &Balance) -> Vec<SensRow> {
    let s = b.s;
    let a0 = m.emissions(&b.c, s).a_atm;
    let conv = m.backup(s);
    let heat = m.backup_heater(b, s);
    let routes: [(f64, &str, [f64; 3]); 4] = [
        (0.0, "B_REC_F", [0.0, 0.0, 0.0]),
        (
            1.0,
            "B_REC_F_CONV_UNABATED",
            [conv.h2_extra_t_y, conv.emit_nocc_t_y, 0.0],
        ),
        (
            2.0,
            "B_REC_F_CONV_CAPTURE",
            [
                conv.h2_extra_t_y,
                conv.emit_cc_t_y,
                conv.a_extra_cc_vs_new_smr,
            ],
        ),
        (
            3.0,
            "B_REC_F_HEATER_K20",
            [heat.h2_extra_t_y, heat.emit_t_y, heat.a_extra_vs_new_smr],
        ),
    ];
    let note = "backup firing is a sensitivity only; furnace and heater capital are unpriced, \
                so no cost metric; route 3 fires process heat only, card K.20";
    let mut rows = Vec::new();
    for (test, run, [h2, emit, extra]) in routes {
        for (metric, v) in [
            ("H2_EXTRA_T_Y", h2),
            ("EMIT_BACKUP_T_Y", emit),
            ("A_EXTRA_T_Y", extra),
            ("A_T_Y", a0 + extra),
        ] {
            rows.push(SensRow {
                sweep_id: "BACKUP_FIRING",
                run_id: run.into(),
                parameter: "Backup route over the outage hours",
                unit: "index",
                base_value: 0.0,
                test_value: test,
                metric,
                result: v,
                delta: v - if metric == "A_T_Y" { a0 } else { 0.0 },
                note: note.into(),
                status: None,
            });
        }
    }
    rows
}

/// Every sweep. `base` is the registry of the preset the rows describe.
pub fn sweeps(base: &ParamSet, fs: &FsInputs) -> Result<Vec<SensRow>> {
    let m = Model::new(base)?;
    let central = evaluate(base, CENTRAL)?;
    let b0 = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
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
    );
    w.values(
        "AVAILABILITY",
        "AVAILABILITY",
        "Availability",
        "-",
        m.avail,
        &[0.65, 0.70, 0.75, 0.85, 0.90],
    );
    w.values(
        "SRD",
        "SRD_CENTRAL",
        "Specific reboiler duty",
        "GJ/t",
        m.srd.central,
        &[m.srd.low, m.srd.central, m.srd.high],
    );
    w.values(
        "GAS",
        "GAS_CENTRAL",
        "Gas price",
        "S$/GJ",
        m.gas.central,
        &[m.gas.low, m.gas.central, m.gas.high],
    );
    w.values(
        "TS",
        "TS_CENTRAL",
        "CO2 transport and storage",
        "S$/t",
        m.ts.central,
        &[m.ts.low, m.ts.central, m.ts.high],
    );
    w.values(
        "ETA_POWER",
        "ETA_POWER",
        "Power conversion efficiency",
        "-",
        m.eta_pwr,
        &[m.eta_pwr, m.eta_pwr_alt],
    );
    w.values(
        "DISCOUNT",
        "DISCOUNT",
        "Real discount rate, derivations L.30",
        "-",
        m.rate,
        &[0.05, m.rate, 0.10],
    );
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
    );
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
    );

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
        );
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
    );

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
        );
    }
    for (i, stage) in Stage::ALL.iter().enumerate() {
        let pt = Point {
            stage: *stage,
            ..CENTRAL
        };
        let run = format!("B_REC_F_{}", stage.id());
        w.push(lab("STAGE", &run, "Stage index", "-", 1.0, i as f64), pt);
    }
    let need = b0.r;
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
    );
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
    );
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
        );
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
        );
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
    );
    w.push(
        lab("ZLD", "B_REC_F", "ZLD steam freed", "flag", 0.0, 1.0),
        Point {
            tweak: zld_on,
            ..CENTRAL
        },
    );
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
    );
    w.push(
        lab("SOLVENT", "B_REC_F", "Solvent MEA", "-", 0.0, 0.0),
        CENTRAL,
    );
    w.push(
        lab("SOLVENT", "B_INL_SEL", "Solvent Selexol", "-", 0.0, 1.0),
        Point {
            scn: Scenario::InlSel,
            ..CENTRAL
        },
    );
    w.push(
        lab("SOLVENT", "B_REC_F", "Solvent aMDEA", "-", 0.0, 2.0),
        Point {
            tweak: amdea,
            ..CENTRAL
        },
    );

    // Cost inputs for the tornado, through the ledger query only.
    let occ = m.occ_kwt;
    for v in [
        occ.low,
        0.7 * occ.central,
        occ.central,
        occ.high,
        1.3 * occ.central,
    ] {
        w.push_with(
            lab(
                "REACTOR_CAPITAL",
                "B_REC_F",
                "Reactor OCC, INL low and high and the central value plus and minus 30%",
                "US$2019/kWt",
                occ.central,
                v,
            ),
            Point {
                cost: CostTweak {
                    rx_mult: v / occ.central,
                    ..NO_COST
                },
                ..CENTRAL
            },
            "reactor capital and its IDC scale with OCC; the stage factor still applies",
            None,
        );
    }
    let k = m.capt_k;
    for v in [k.low, k.central, k.high] {
        w.push_with(
            lab(
                "CAPTURE_CAPITAL",
                "B_REC_F",
                "Capture capital per t/h of CO2; the high value is IEAGHG Case 3, MEA on flue gas",
                "S$M per t/h",
                k.central,
                v,
            ),
            Point {
                cost: CostTweak {
                    capt_k: Some(v),
                    ..NO_COST
                },
                ..CENTRAL
            },
            "capture fixed O&M follows capital at its central rate",
            None,
        );
    }
    for v in [m.fom.central, m.fom.high] {
        w.push(
            lab(
                "CAPTURE_FOM",
                "B_REC_F",
                "Capture fixed O&M, share of capture capital per year",
                "1/y",
                m.fom.central,
                v,
            ),
            Point {
                cost: CostTweak {
                    fom: Some(v),
                    ..NO_COST
                },
                ..CENTRAL
            },
        );
    }

    // Gas saving on Ahn's 18.8% in place of the INL pair.
    let saving = |ng: f64| 1.0 - ng / (m.ng_conv * b0.c.h_scale);
    let s_inl = saving(b0.c.ng);
    let s_ahn = saving(m.ng_ahn_equivalent);
    w.push(
        lab(
            "GAS_SAVING_AHN",
            "B_REC_F",
            "Gas saved against the unabated SMR",
            "-",
            s_inl,
            s_inl,
        ),
        CENTRAL,
    );
    w.push_with(
        lab(
            "GAS_SAVING_AHN",
            "B_REC_F_CARBON_HELD",
            "Gas saved against the unabated SMR",
            "-",
            s_inl,
            s_ahn,
        ),
        Point {
            cost: CostTweak {
                ng_override: Some(m.ng_ahn_equivalent),
                ..NO_COST
            },
            ..CENTRAL
        },
        "gas line only; abatement and T&S held, the screening basis",
        None,
    );
    let f_ng = m.ng_ahn_equivalent / b0.c.ng;
    w.push_with(
        lab(
            "GAS_SAVING_AHN",
            "B_REC_F_CARBON_FOLLOWS",
            "Gas saved against the unabated SMR",
            "-",
            s_inl,
            s_ahn,
        ),
        Point {
            adj: Adjust {
                ng: f_ng,
                cap: f_ng,
                atm: f_ng,
                vent_kmol_h: 0.0,
            },
            ..CENTRAL
        },
        "the extra gas carbon leaves as the plant's carbon does, so captured and emitted \
         carbon scale with the gas and T&S follows; the heat balance is held",
        None,
    );

    // Flowsheet sweeps: reformer exit (S8) and equilibrium constants (S2).
    let s8 = format!(
        "decision S8: the assumed HA tube cannot reach 871 C with helium at 900 C, so the \
         headline stays on INL's 871 C basis; the Level 3 ratio to 871 C is carried into the \
         anchored balance. The cryogenic recycle share of the reject carbon is held at INL's \
         {:.3} (card J.7) at every exit temperature, so the burned cut takes the extra reject \
         carbon",
        crate::flowsheet::recycle_fraction(&m)
    );
    let s8 = s8.as_str();
    w.flowsheet_point(
        &m,
        lab(
            "REFORMER_T",
            "FS_B_REC_F",
            "Reformer exit temperature",
            "C",
            crate::flowsheet::T_REFORMER_OUT_C,
            crate::flowsheet::T_REFORMER_OUT_C,
        ),
        fs.base,
        fs.base,
        s8,
    );
    for (t, run) in &fs.reformer {
        let id = format!("FS_B_REC_F_T{t:.0}");
        w.flowsheet_point(
            &m,
            lab(
                "REFORMER_T",
                &id,
                "Reformer exit temperature",
                "C",
                crate::flowsheet::T_REFORMER_OUT_C,
                *t,
            ),
            fs.base,
            *run,
            s8,
        );
    }
    let s2 = "decision S2: the flowsheet takes one constant source per run, so NIST Gibbs \
              constants switch on SMR-EQ, the HTS sizing and the LTS together; Zhang stays \
              the default";
    w.flowsheet_point(
        &m,
        lab(
            "LTS_K_SOURCE",
            "FS_B_REC_F",
            "Equilibrium constants, 0 Zhang 2021, 1 NIST Gibbs",
            "index",
            0.0,
            0.0,
        ),
        fs.base,
        fs.base,
        s2,
    );
    w.flowsheet_point(
        &m,
        lab(
            "LTS_K_SOURCE",
            "FS_B_REC_F_K_NIST",
            "Equilibrium constants, 0 Zhang 2021, 1 NIST Gibbs",
            "index",
            0.0,
            1.0,
        ),
        fs.base,
        fs.k_nist,
        s2,
    );

    // Condensate CO2 vented, decision S14.
    w.push(
        lab(
            "CONDENSATE_CO2",
            "B_REC_F",
            "Condensate CO2, 0 held in the carbon split, 1 vented",
            "index",
            0.0,
            0.0,
        ),
        CENTRAL,
    );
    let vent_label = lab(
        "CONDENSATE_CO2",
        "B_REC_F_VENTED",
        "Condensate CO2, 0 held in the carbon split, 1 vented",
        "index",
        0.0,
        1.0,
    );
    match fs.condensate_co2_kmol_h {
        Some(v) => w.push_with(
            vent_label,
            Point {
                adj: Adjust {
                    vent_kmol_h: v,
                    ..NO_ADJUST
                },
                ..CENTRAL
            },
            "decision S14: the CO2 the Level 3 flash leaves in the process condensate (S10 of \
             FS_B_REC_F) is vented and taken from the captured stream; the headline ledger \
             stays on the anchored split",
            Some(FLOWSHEET_STATUS),
        ),
        None => w.failed(
            &vent_label,
            &METRICS,
            "the Level 3 flowsheet run FS_B_REC_F did not converge",
        ),
    }

    // Term T05 on the plant's own flowsheet: the gated value is the Case 5 chain
    // scaled on plant CO2; the Level 3 EX3 outlet of FS_B_REC_F gives its own.
    let t05_note = "term T05 at the syngas heat of the plant's own Level 3 flowsheet (S09A \
                    of FS_B_REC_F) in place of the Case 5 chain scaled on plant CO2, which \
                    the T961-7 gate validates on Case 5 only";
    let t05_base = lab(
        "T05_BASIS",
        "B_REC_F",
        "Syngas heat above 130 C to the reboiler",
        "MW",
        b0.stack.ex4,
        b0.stack.ex4,
    );
    w.push_with(t05_base, CENTRAL, t05_note, None);
    w.row(
        &lab(
            "T05_BASIS",
            "B_REC_F",
            "Syngas heat above 130 C to the reboiler",
            "MW",
            b0.stack.ex4,
            b0.stack.ex4,
        ),
        "GAP_MW",
        b0.gap,
        0.0,
        t05_note,
        None,
    );
    let t05_label = lab(
        "T05_BASIS",
        "B_REC_F_FS_SYNGAS",
        "Syngas heat above 130 C to the reboiler",
        "MW",
        b0.stack.ex4,
        fs.syngas_mw.unwrap_or(f64::NAN),
    );
    match fs.syngas_mw {
        Some(q) if b0.stack.ex4 > 0.0 => {
            let set = [("Q_EX4_ABOVE_130C", m.q_ex4_above130 * q / b0.stack.ex4)];
            let pt = Point {
                params: &set,
                ..CENTRAL
            };
            let gap = solve(base, pt).map(|s| s.b.gap);
            w.push_with(t05_label, pt, t05_note, Some(FLOWSHEET_STATUS));
            let l = lab(
                "T05_BASIS",
                "B_REC_F_FS_SYNGAS",
                "Syngas heat above 130 C to the reboiler",
                "MW",
                b0.stack.ex4,
                q,
            );
            match gap {
                Ok(g) => w.row(
                    &l,
                    "GAP_MW",
                    g,
                    g - b0.gap,
                    t05_note,
                    Some(FLOWSHEET_STATUS),
                ),
                Err(e) => w.failed(&l, &["GAP_MW"], &e.to_string()),
            }
        }
        _ => w.failed(
            &t05_label,
            &METRICS,
            "the Level 3 flowsheet run FS_B_REC_F did not converge",
        ),
    }

    let mut rows = w.rows;
    rows.extend(life_rows(&m, &b0)?);
    rows.extend(backup_rows(&m, &b0));
    rows.extend(levers(base)?);
    rows.extend(mature_rows(base)?);
    rows.extend(cofunding_rows(base)?);
    Ok(rows)
}

/// MATURE_60Y, the author's stacked scenario: tenth of a kind on INL's BOAK
/// learning route (INL PDF p46), a 60 y life, n = 1.0 and the rated reactor
/// charged. Each method gives its cost at the central gas price and at
/// S$20/GJ, and the gas price that brings it to the mature-scenario target.
///
/// eq: L.31
pub fn mature_rows(base: &ParamSet) -> Result<Vec<SensRow>> {
    let m = Model::new(base)?;
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let a = m.emissions(&b.c, m.s_b).a_atm;
    let target = m.target_mature_c;
    let q0 = CostQuery::central(Stage::Oak10FromBoak, m.lives[2], m.n_linear, m.r_rated);
    let cost = |method: Method, q: CostQuery| -> Result<f64> {
        Ok(match method {
            Method::L5 => m.ledger(&b, &q)?.l5() * 1e6 / a,
            Method::Dcf => m.dcf(&b, &q)?.lac,
        })
    };
    let note = "stacked scenario MATURE_60Y: 10OAK_P46 stage, 60 y life, n = 1.0, reactor \
                charged at the 210 MWth rating, Case B REC_F; the base result stays BOAK 25 y";
    let mut rows = Vec::new();
    for (method, mid) in [(Method::L5, "L5"), (Method::Dcf, "DCF")] {
        let run = format!("MATURE_60Y_{mid}");
        let c0 = cost(method, q0)?;
        let c20 = cost(
            method,
            CostQuery {
                gas: Some(MATURE_GAS_POINT),
                ..q0
            },
        )?;
        let gas = solve_for(
            |g| cost(method, CostQuery { gas: Some(g), ..q0 }),
            m.gas.low,
            m.gas.high,
            target,
        )?;
        for (parameter, unit, base_value, test_value, metric, v) in [
            (
                "Cost at the central gas price, against the mature-scenario target",
                "S$/t",
                target,
                m.gas.central,
                "COST_S_PER_T",
                c0,
            ),
            (
                "Cost with gas at S$20/GJ, against the mature-scenario target",
                "S$/t",
                target,
                MATURE_GAS_POINT,
                "COST_S_PER_T",
                c20,
            ),
            (
                "Gas price that brings the cost to the mature-scenario target",
                "S$/GJ",
                m.gas.central,
                target,
                "GAS_NEEDED",
                gas,
            ),
        ] {
            rows.push(SensRow {
                sweep_id: "MATURE_60Y",
                run_id: run.clone(),
                parameter,
                unit,
                base_value,
                test_value,
                metric,
                result: v,
                delta: v - base_value,
                note: note.to_string(),
                status: None,
            });
        }
    }
    Ok(rows)
}

/// Gas price of the second MATURE_60Y point, S$/GJ, the author's chosen value.
pub const MATURE_GAS_POINT: f64 = 20.0;

/// Public co-funding of reactor capital that brings each method to the
/// mature-scenario target, BOAK, 25 y and 60 y, n = 1.0, the rated reactor
/// charged: the share of reactor capital funded and the S$M it comes to. A share
/// above one means co-funding alone cannot reach the target.
///
/// eq: L.31
pub fn cofunding_rows(base: &ParamSet) -> Result<Vec<SensRow>> {
    let m = Model::new(base)?;
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let a = m.emissions(&b.c, m.s_b).a_atm;
    let target = m.target_mature_c;
    let cost = |method: Method, q: CostQuery| -> Result<f64> {
        Ok(match method {
            Method::L5 => m.ledger(&b, &q)?.l5() * 1e6 / a,
            Method::Dcf => m.dcf(&b, &q)?.lac,
        })
    };
    let note = "public co-funding of reactor capital to the mature-scenario target, one lever \
                alone; the share is of the reactor capital the ledger charges";
    let mut rows = Vec::new();
    for (method, mid) in [(Method::L5, "L5"), (Method::Dcf, "DCF")] {
        for life in [m.lives[0], m.lives[2]] {
            let run = format!("{mid}_BOAK_{life:.0}Y");
            let q0 = CostQuery::central(Stage::Boak, life, m.n_linear, m.r_rated);
            let capital = m.cost_items(&b, &q0)?.reactor_capital;
            let mult = solve_for(
                |x| cost(method, CostQuery { rx_mult: x, ..q0 }),
                0.5,
                1.0,
                target,
            )?;
            let share = 1.0 - mult;
            for (parameter, unit, base_value, metric, v) in [
                (
                    "Reactor capital the ledger charges, before co-funding",
                    "S$M",
                    0.0,
                    "REACTOR_CAPITAL_S_M",
                    capital,
                ),
                (
                    "Share of reactor capital publicly funded",
                    "-",
                    0.0,
                    "COFUND_SHARE",
                    share,
                ),
                (
                    "Public co-funding of reactor capital",
                    "S$M",
                    0.0,
                    "COFUND_S_M",
                    share * capital,
                ),
            ] {
                rows.push(SensRow {
                    sweep_id: "COFUNDING",
                    run_id: run.clone(),
                    parameter,
                    unit,
                    base_value,
                    test_value: target,
                    metric,
                    result: v,
                    delta: v - base_value,
                    note: note.to_string(),
                    status: None,
                });
            }
        }
    }
    Ok(rows)
}
