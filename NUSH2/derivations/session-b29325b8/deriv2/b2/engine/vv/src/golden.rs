//! Comparison with the frozen screening values. A mismatch means a port error, so
//! the code gets fixed and the golden files stay as they are.

use nush2_numerics::{EngineError, Result};
use nush2_plant::anchored::{Balance, BalanceOpts, Case, CircKey, LossKey, Model, Scenario};
use nush2_plant::economics::{CostQuery, Ledger, Stage};
use nush2_plant::params::ParamSet;
use serde_json::Value;

const ENGINE_GOLDENS: &str = include_str!("../data/engine_goldens.json");
const FREEZE_VALUES: &str = include_str!("../data/freeze_values.json");

/// Relative tolerance of every golden comparison.
pub const GOLD_TOL: f64 = 1e-9;

/// Running tally of one golden test.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GoldStats {
    pub compared: usize,
    pub failed: usize,
    pub max_err: f64,
    pub worst: String,
}

impl GoldStats {
    /// Relative error, or absolute error for a golden near zero.
    fn cmp(&mut self, label: &str, got: f64, want: f64) {
        let err = if want.abs() > 1e-6 {
            (got - want).abs() / want.abs()
        } else {
            (got - want).abs()
        };
        self.compared += 1;
        if err > GOLD_TOL || err.is_nan() {
            self.failed += 1;
        }
        if err > self.max_err || err.is_nan() {
            self.max_err = err;
            self.worst = format!("{label}: got {got}, want {want}");
        }
    }

    fn get(&mut self, prefix: &str, obj: &Value, key: &str, got: f64) -> Result<()> {
        let want = obj
            .get(key)
            .and_then(Value::as_f64)
            .ok_or_else(|| missing(prefix, key))?;
        self.cmp(&format!("{prefix}.{key}"), got, want);
        Ok(())
    }

    pub fn passed(&self) -> bool {
        self.failed == 0 && self.compared > 0
    }
}

fn missing(prefix: &str, key: &str) -> EngineError {
    EngineError::Parse(format!("golden missing {prefix}.{key}"))
}

fn parse(text: &str) -> Result<Value> {
    serde_json::from_str(text).map_err(|e| EngineError::Parse(e.to_string()))
}

fn scenario(id: &str) -> Result<Scenario> {
    Ok(match id {
        "INL_SEL" => Scenario::InlSel,
        "REC_F" => Scenario::RecF,
        "REC_FIX" => Scenario::RecFix,
        "COMB_H" => Scenario::CombH,
        "COMB_N" => Scenario::CombN,
        "EXPORT" => Scenario::Export,
        other => return Err(EngineError::Parse(format!("unknown scenario {other}"))),
    })
}

fn stage_of(id: &str) -> Result<Stage> {
    Ok(match id {
        "FOAK" => Stage::Foak,
        "BOAK" => Stage::Boak,
        "10-OAK INL" => Stage::Oak10FromFoak,
        "10-OAK v1_7" => Stage::Oak10FromBoak,
        other => return Err(EngineError::Parse(format!("unknown stage {other}"))),
    })
}

fn exponent(m: &Model, tag: &str) -> Result<f64> {
    Ok(match tag {
        "1.00" | "1.0" => m.n_linear,
        "0.60" | "0.6" => m.n_team,
        "0.31" => m.n_ngnp,
        other => return Err(EngineError::Parse(format!("unknown exponent {other}"))),
    })
}

fn split_case(key: &str) -> Result<(Case, Scenario)> {
    let (c, s) = key
        .split_once('|')
        .ok_or_else(|| EngineError::Parse(format!("bad case key {key}")))?;
    let case = if c == "A" { Case::A } else { Case::B };
    Ok((case, scenario(s)?))
}

fn compare_balance(st: &mut GoldStats, key: &str, m: &Model, g: &Value, b: &Balance) -> Result<()> {
    let c = &b.c;
    let e = m.emissions(c, b.s);
    let v = &g["carbon"];
    let p = format!("{key}.carbon");
    for (k, x) in [
        ("NG", c.ng),
        ("H2", c.h2),
        ("h_scale", c.h_scale),
        ("P", c.p),
        ("F", c.f),
        ("T", c.t),
        ("co2_abs", c.co2_abs),
        ("cap", c.cap),
        ("unconv", c.unconv),
        ("G", c.g),
        ("R", c.r),
        ("burn", c.burn),
        ("exported", c.exported),
        ("slip", c.slip),
        ("closure", c.closure),
    ] {
        st.get(&p, v, k, x)?;
    }
    let v = &g["emissions"];
    let p = format!("{key}.emissions");
    for (k, x) in [
        ("baseline", e.baseline),
        ("site", e.site),
        ("atm", e.atm),
        ("captured", e.captured),
        ("A_site", e.a_site),
        ("A_atm", e.a_atm),
    ] {
        st.get(&p, v, k, x)?;
    }
    let v = &g["balance"];
    let p = format!("{key}.balance");
    for (k, x) in [
        ("s", b.s),
        ("srd", b.srd),
        ("q_proc", b.q_proc),
        ("q_recyc", b.q_recyc),
        ("q_reb", b.q_reb),
        ("avail", b.avail),
        ("gap", b.gap),
        ("q_ihx", b.q_ihx),
        ("q_duct", b.q_duct),
        ("e_proc", b.e_proc),
        ("R", b.r),
        ("q_tg", b.q_tg),
        ("lhv", b.lhv),
        ("export_kept", b.export_kept),
        ("tg_surplus", b.tg_surplus),
        ("gap_after_tg", b.gap_after_tg),
        ("cap_t", b.cap_t),
        ("w_s", b.w_s),
        ("w_p", b.w_p),
        ("e_circ", b.e_circ),
        ("e_total", b.e_total),
        ("e_tg", b.e_tg),
        ("e_nuc", b.e_nuc),
        ("branch", b.branch),
        ("qv", b.qv),
    ] {
        st.get(&p, v, k, x)?;
    }
    compare_stack(st, key, m, &g["stack"], b)?;
    compare_e_rows(st, key, &g["e_rows"], b)?;
    Ok(())
}

fn compare_stack(st: &mut GoldStats, key: &str, _m: &Model, v: &Value, b: &Balance) -> Result<()> {
    let p = format!("{key}.stack");
    for (k, x) in [
        ("selexol", b.stack.selexol),
        ("export", b.stack.export),
        ("ex4", b.stack.ex4),
        ("zld", b.stack.zld),
    ] {
        st.get(&p, v, k, x)?;
    }
    Ok(())
}

fn compare_e_rows(st: &mut GoldStats, key: &str, v: &Value, b: &Balance) -> Result<()> {
    let p = format!("{key}.e_rows");
    let e = &b.e_rows;
    for (k, x) in [
        ("reforming", e.reforming),
        ("purification", e.purification),
        ("steam", e.steam),
        ("co2comp", e.co2comp),
        ("purge", e.purge),
        ("cooling", e.cooling),
        ("water", e.water),
        ("mea_pump", e.mea_pump),
        ("mea_cooling", e.mea_cooling),
    ] {
        st.get(&p, v, k, x)?;
    }
    Ok(())
}

/// `golden_balance_all_cases`: constants, every case and the swap table.
pub fn balance_all_cases() -> Result<GoldStats> {
    let g = parse(ENGINE_GOLDENS)?;
    let p = ParamSet::embedded()?;
    let m = Model::new(&p)?;
    let mut st = GoldStats::default();
    let c = &g["constants"];
    for (k, x) in [
        ("s_B", m.s_b),
        ("H2_A_kgh", m.h2_a_kgh),
        ("H2_B_kgh", m.h2_b_kgh),
        ("hours", m.hours),
        ("days", m.days),
        ("F_C6", m.f_c6),
        ("P_C", m.p_c),
        ("C_per_mol", m.c_per_mol),
        ("BASE_INL", m.base_inl),
        ("PHI_NR", m.phi_nr),
        ("LF10", m.lf10),
        ("IDC", m.idc_factor()),
        ("n_NGNP", m.n_ngnp),
    ] {
        st.get("constants", c, k, x)?;
    }
    for (i, life) in m.lives.iter().enumerate() {
        let key = format!("{life:.0}");
        st.get(
            "constants.CRF",
            &c["CRF"],
            &key,
            nush2_plant::anchored::crf(m.rate, m.lives[i])?,
        )?;
    }
    for (k, x) in [
        ("low", m.capt_k.low),
        ("central", m.capt_k.central),
        ("high", m.capt_k.high),
    ] {
        st.get("constants.CAPT_K", &c["CAPT_K"], k, x)?;
    }
    for (k, x) in [
        ("low", m.recycle_capex_a.low),
        ("central", m.recycle_capex_a.central),
        ("high", m.recycle_capex_a.high),
    ] {
        st.get("constants.RECYCLE_CAPEX_A", &c["RECYCLE_CAPEX_A"], k, x)?;
    }
    let cases = g["cases"]
        .as_object()
        .ok_or_else(|| missing("root", "cases"))?;
    for (key, v) in cases {
        let (case, scn) = split_case(key)?;
        let b = m.balance(scn, &m.opts(m.scale(case)))?;
        compare_balance(&mut st, key, &m, v, &b)?;
    }
    compare_swaps(&mut st, &p, &m, &g["swap_sensitivity_case_B_REC_F"])?;
    Ok(st)
}

fn compare_swaps(st: &mut GoldStats, p: &ParamSet, m: &Model, sw: &Value) -> Result<()> {
    let s = m.s_b;
    let scn = Scenario::RecF;
    let a0 = m.emissions(&m.carbon(scn, m.phi_rec, m.eta_mea), s).a_atm;
    let o = m.opts(s);
    let mut one = |label: &str, opts: BalanceOpts, a: f64| -> Result<()> {
        let b = m.balance(scn, &opts)?;
        let v = &sw[label];
        st.get(label, v, "R", b.r)?;
        st.get(label, v, "A", a)
    };
    one("base", o, a0)?;
    for phi in [0.76, 0.79] {
        let a = m.emissions(&m.carbon(scn, phi, m.eta_mea), s).a_atm;
        one(&format!("phi {phi}"), BalanceOpts { phi, ..o }, a)?;
    }
    for eta_c in [0.85, 0.95] {
        let a = m.emissions(&m.carbon(scn, m.phi_rec, eta_c), s).a_atm;
        one(&format!("eta {eta_c}"), BalanceOpts { eta_c, ..o }, a)?;
    }
    for srd in [m.srd.low, m.srd.high] {
        one(&format!("SRD {srd}"), BalanceOpts { srd, ..o }, a0)?;
    }
    for (label, circ) in [
        ("circ none", CircKey::None),
        ("circ low", CircKey::Low),
        ("circ central_5p15", CircKey::Central5p15),
        ("circ high", CircKey::High),
    ] {
        one(label, BalanceOpts { circ, ..o }, a0)?;
    }
    for (label, loss) in [
        ("loss none", LossKey::None),
        ("loss low", LossKey::Low),
        ("loss high", LossKey::High),
    ] {
        one(label, BalanceOpts { loss, ..o }, a0)?;
    }
    // Process heat basis, helium flow following duty.
    let (q0, h0) = (m.q_proc, m.he_flow);
    for (label, q_s) in [
        ("Q -5%", 0.95 * q0 * s),
        ("Q +5%", 1.05 * q0 * s),
        ("Q Ahn 143.4", m.q_proc_ahn_b),
    ] {
        let mut pq = p.clone();
        pq.set("Q_PROCESS", q_s / s)?;
        pq.set("HE_FLOW", h0 * (q_s / s) / q0)?;
        let mq = Model::new(&pq)?;
        let b = mq.balance(scn, &mq.opts(s))?;
        st.get(label, &sw[label], "R", b.r)?;
        st.get(label, &sw[label], "A", a0)?;
    }
    Ok(())
}

/// `golden_ledger_grid`: every L0 to L5 cell of every case.
pub fn ledger_grid() -> Result<GoldStats> {
    let g = parse(ENGINE_GOLDENS)?;
    let m = Model::embedded()?;
    let mut st = GoldStats::default();
    let cases = g["cases"]
        .as_object()
        .ok_or_else(|| missing("root", "cases"))?;
    for (key, v) in cases {
        let (case, scn) = split_case(key)?;
        let b = m.balance(scn, &m.opts(m.scale(case)))?;
        let grid = v["ladder_SM_y"]
            .as_object()
            .ok_or_else(|| missing(key, "ladder_SM_y"))?;
        for (cell, want) in grid {
            let f: Vec<&str> = cell.split('|').collect();
            if f.len() != 4 {
                return Err(EngineError::Parse(format!("bad cell {cell}")));
            }
            let stage = stage_of(f[0])?;
            let n = exponent(&m, f[1].trim_start_matches('n'))?;
            let life: f64 = f[2]
                .trim_start_matches('y')
                .parse()
                .map_err(|_| EngineError::Parse(format!("bad cell {cell}")))?;
            let r = if f[3] == "R210.0" { m.r_rated } else { b.r };
            let lad = m
                .ledger(&b, &CostQuery::central(stage, life, n, r))?
                .ladder();
            let w = want.as_array().ok_or_else(|| missing(cell, "ladder"))?;
            for (i, x) in w.iter().enumerate() {
                let want = x.as_f64().ok_or_else(|| missing(cell, "rung"))?;
                st.cmp(&format!("{key}.{cell}.L{i}"), lad[i], want);
            }
        }
    }
    Ok(st)
}

fn compare_summary(st: &mut GoldStats, label: &str, m: &Model, v: &Value, s: f64) -> Result<()> {
    let sum = m.case_summary(Scenario::RecF, s)?;
    for (k, x) in &sum {
        st.get(label, v, k, *x)?;
    }
    let b = m.balance(Scenario::RecF, &m.opts(s))?;
    compare_stack(st, label, m, &v["stack"], &b)?;
    compare_e_rows(st, label, &v["e_rows"], &b)?;
    Ok(())
}

fn ac(l: &Ledger, a: f64) -> [(&'static str, f64); 4] {
    let lad = l.ladder();
    [
        ("L0", lad[0] * 1e6 / a),
        ("L3", lad[3] * 1e6 / a),
        ("L4", lad[4] * 1e6 / a),
        ("L5", lad[5] * 1e6 / a),
    ]
}

fn compare_grid(st: &mut GoldStats, m: &Model, g: &Value) -> Result<()> {
    let bb = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let ba = m.balance(Scenario::RecF, &m.opts(1.0))?;
    let ab = m.emissions(&bb.c, m.s_b).a_atm;
    let aa = m.emissions(&ba.c, 1.0).a_atm;
    let grid = g["grid"]
        .as_object()
        .ok_or_else(|| missing("root", "grid"))?;
    for (key, want) in grid {
        let f: Vec<&str> = key.split('|').collect();
        if f.len() != 5 {
            return Err(EngineError::Parse(format!("bad grid key {key}")));
        }
        let (b, a) = if f[0] == "A" { (&ba, aa) } else { (&bb, ab) };
        let r = if f[0] == "B" && f[1] == "210.0" {
            m.r_rated
        } else {
            b.r
        };
        let n = exponent(m, f[2].trim_start_matches('n'))?;
        let stage = stage_of(f[3])?;
        let life: f64 = f[4]
            .parse()
            .map_err(|_| EngineError::Parse(format!("bad grid key {key}")))?;
        let l = m.ledger(b, &CostQuery::central(stage, life, n, r))?;
        for (k, x) in ac(&l, a) {
            st.get(key, want, k, x)?;
        }
    }
    let lines = g["lines"]
        .as_object()
        .ok_or_else(|| missing("root", "lines"))?;
    for (key, want) in lines {
        let f: Vec<&str> = key.split('|').collect();
        if f.len() != 3 {
            return Err(EngineError::Parse(format!("bad line key {key}")));
        }
        let r = if f[0] == "B210" { m.r_rated } else { bb.r };
        let stage = stage_of(f[1])?;
        let life: f64 = f[2]
            .parse()
            .map_err(|_| EngineError::Parse(format!("bad line key {key}")))?;
        let l = m.ledger(&bb, &CostQuery::central(stage, life, m.n_linear, r))?;
        for (k, x) in [
            ("gas", l.gas),
            ("reactor_capital", l.reactor_capital),
            ("capture_capital", l.capture_capital),
            ("capture_fom", l.capture_fom),
            ("capture_allin", l.capture_allin),
            ("ts", l.ts),
            ("reactor_om", l.reactor_om),
            ("ihx", l.ihx),
            ("licence", l.licence),
            ("decom", l.decom),
            ("idc", l.idc),
            ("power_island", l.power_island),
            ("recycle_train", l.recycle_train),
            ("lost_steam", l.lost_steam),
            ("baseline_power", l.baseline_power),
            ("export_revenue", l.export_revenue),
            ("dC_L3", l.l3()),
        ] {
            st.get(key, want, k, x)?;
        }
        for (k, x) in ac(&l, ab) {
            st.get(key, want, k, x)?;
        }
    }
    Ok(())
}

fn compare_inl(st: &mut GoldStats, m: &Model, g: &Value) -> Result<()> {
    let bi = m.balance(Scenario::InlSel, &m.opts(1.0))?;
    let ei = m.emissions(&bi.c, 1.0);
    let v = &g["INL"];
    let l = |life: f64| -> Result<f64> {
        let q = CostQuery::central(Stage::Boak, life, m.n_linear, bi.r);
        Ok(m.ledger(&bi, &q)?.l3() * 1e6 / ei.a_atm)
    };
    for (k, x) in [
        ("R", bi.r),
        ("e_proc", bi.e_proc),
        ("e_total", bi.e_total),
        ("circ_sec", bi.w_s / m.eta_motor),
        ("circ_prim", bi.w_p / m.eta_motor),
        ("q_ihx", bi.q_ihx),
        ("q_duct", bi.q_duct),
        ("qv", bi.qv),
        ("branch", bi.branch),
        ("cap_t_y", ei.captured),
        ("atm_t_y", ei.atm),
        ("A", ei.a_atm),
        ("plant", ei.captured / (ei.captured + ei.atm)),
        ("L3_25", l(m.lives[0])?),
        ("L3_60", l(m.lives[2])?),
        ("complete_220", m.q_proc + m.e_rows.sum() / m.eta_pwr),
        ("margin_210", m.r_rated - bi.r),
    ] {
        st.get("INL", v, k, x)?;
    }
    Ok(())
}

fn compare_sens(st: &mut GoldStats, m: &Model, g: &Value) -> Result<()> {
    let sb = m.s_b;
    let bb = m.balance(Scenario::RecF, &m.opts(sb))?;
    let ab = m.emissions(&bb.c, sb).a_atm;
    let h2 = m.h2_b_kgh * m.hours / 1000.0;
    let base = CostQuery::central(Stage::Boak, m.lives[0], m.n_linear, m.r_rated);
    let l3 =
        |b: &Balance, q: CostQuery, a: f64| -> Result<f64> { Ok(m.ledger(b, &q)?.l3() * 1e6 / a) };
    let b34 = m.balance(
        Scenario::RecF,
        &BalanceOpts {
            srd: m.srd.low,
            ..m.opts(sb)
        },
    )?;
    let b42 = m.balance(
        Scenario::RecF,
        &BalanceOpts {
            srd: m.srd.high,
            ..m.opts(sb)
        },
    )?;
    let v = &g["sens_cost"];
    let life60 = m.lives[2];
    for (k, x) in [
        ("central", l3(&bb, base, ab)?),
        (
            "ts_low",
            l3(
                &bb,
                CostQuery {
                    ts: Some(m.ts.low),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "ts_high",
            l3(
                &bb,
                CostQuery {
                    ts: Some(m.ts.high),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "ts_low_60y",
            l3(
                &bb,
                CostQuery {
                    ts: Some(m.ts.low),
                    life: life60,
                    ..base
                },
                ab,
            )?,
        ),
        (
            "ts_high_60y",
            l3(
                &bb,
                CostQuery {
                    ts: Some(m.ts.high),
                    life: life60,
                    ..base
                },
                ab,
            )?,
        ),
        (
            "gas_10",
            l3(
                &bb,
                CostQuery {
                    gas: Some(m.gas.low),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "gas_20",
            l3(
                &bb,
                CostQuery {
                    gas: Some(m.gas.high),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "n06",
            l3(
                &bb,
                CostQuery {
                    n: m.n_team,
                    ..base
                },
                ab,
            )?,
        ),
        (
            "n031",
            l3(
                &bb,
                CostQuery {
                    n: m.n_ngnp,
                    ..base
                },
                ab,
            )?,
        ),
        ("srd34", l3(&b34, base, ab)?),
        ("srd42", l3(&b42, base, ab)?),
        (
            "capt_case3",
            l3(
                &bb,
                CostQuery {
                    capt_k: Some(m.capt_k.high),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "fom6",
            l3(
                &bb,
                CostQuery {
                    fom: Some(m.fom.high),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "rx_p30",
            l3(
                &bb,
                CostQuery {
                    rx_mult: 1.3,
                    ..base
                },
                ab,
            )?,
        ),
        (
            "rx_m30",
            l3(
                &bb,
                CostQuery {
                    rx_mult: 0.7,
                    ..base
                },
                ab,
            )?,
        ),
        (
            "ahn_gas",
            l3(
                &bb,
                CostQuery {
                    ng_override: Some(m.ng_ahn_equivalent),
                    ..base
                },
                ab,
            )?,
        ),
        ("cho", l3(&bb, base, ab + (m.base_cho - m.base_inl) * h2)?),
        (
            "ieaghg",
            l3(&bb, base, ab + (m.base_ieaghg - m.base_inl) * h2)?,
        ),
        (
            "R179",
            l3(
                &bb,
                CostQuery {
                    r: Some(bb.r),
                    ..base
                },
                ab,
            )?,
        ),
        (
            "R179_40",
            l3(
                &bb,
                CostQuery {
                    r: Some(bb.r),
                    life: m.lives[1],
                    ..base
                },
                ab,
            )?,
        ),
        (
            "R179_60",
            l3(
                &bb,
                CostQuery {
                    r: Some(bb.r),
                    life: life60,
                    ..base
                },
                ab,
            )?,
        ),
    ] {
        st.get("sens_cost", v, k, x)?;
    }
    Ok(())
}

fn compare_es(st: &mut GoldStats, m: &Model, g: &Value) -> Result<()> {
    let sb = m.s_b;
    let v = &g["es"];
    let row = |c: &nush2_plant::anchored::Carbon| {
        let e = m.emissions(c, sb);
        (
            e.a_atm,
            e.captured,
            e.atm,
            e.captured / (e.captured + e.atm),
        )
    };
    for (key, eta) in [
        ("eta80", 0.80),
        ("eta85", 0.85),
        ("eta90", 0.90),
        ("eta95", 0.95),
    ] {
        let (a, cap, atm, plant) = row(&m.carbon(Scenario::RecF, m.phi_rec, eta));
        for (k, x) in [("A", a), ("cap", cap), ("atm", atm), ("plant", plant)] {
            st.get(key, &v[key], k, x)?;
        }
    }
    for key in ["INL_SEL", "REC_FIX", "COMB_H", "COMB_N", "EXPORT"] {
        let c = m.carbon(scenario(key)?, m.phi_rec, m.eta_mea);
        let (a, cap, atm, plant) = row(&c);
        for (k, x) in [
            ("A", a),
            ("cap", cap),
            ("atm", atm),
            ("plant", plant),
            ("NG", c.ng * sb),
        ] {
            st.get(key, &v[key], k, x)?;
        }
    }
    st.get(
        "es",
        v,
        "heat_alone_no_recycle",
        (m.co2_conv - m.co2_htgr_nocc) * nush2_props::units::ST * m.days * sb,
    )?;
    let bb = m.balance(Scenario::RecF, &m.opts(sb))?;
    let e = m.emissions(&bb.c, sb);
    st.get(
        "es",
        v,
        "heat_plus_recycle_no_capture",
        e.baseline - e.captured - e.atm,
    )?;
    let lc = m.lifecycle_terms(&bb, sb, bb.r);
    let ab = e.a_atm;
    let net = lc.up_credit - lc.nuclear - lc.ts_burden;
    for (k, x) in [
        ("up_credit", lc.up_credit),
        ("nuclear", lc.nuclear),
        ("ts_burden", lc.ts_burden),
        ("net_adjust", net),
        ("A_burdens_only", ab - lc.nuclear - lc.ts_burden),
        ("A_with_upstream", ab + net),
        ("A_ts_inside", ab - lc.ts_burden),
    ] {
        st.get("lifecycle", &v["lifecycle"], k, x)?;
    }
    let sys = m.system_adjust(&bb, sb);
    st.get("es", v, "system_adjust", sys)?;
    st.get("es", v, "system_view", ab + sys)?;
    let bk = m.backup(sb);
    for (k, x) in [
        ("hours", bk.hours),
        ("h2_extra_t_y", bk.h2_extra_t_y),
        ("emit_cc_t_y", bk.emit_cc_t_y),
        ("emit_nocc_t_y", bk.emit_nocc_t_y),
        ("A_extra_cc_vs_new_smr", bk.a_extra_cc_vs_new_smr),
        ("gas_MMSCFD_cc", bk.gas_mmscfd_cc),
        ("gas_MMSCFD_nocc", bk.gas_mmscfd_nocc),
    ] {
        st.get("backup", &v["backup"], k, x)?;
    }
    Ok(())
}

fn compare_misc(st: &mut GoldStats, m: &Model, g: &Value) -> Result<()> {
    let bb = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    let v = &g["capex"];
    use nush2_plant::anchored::CostLevel::Central as C;
    let r = m.r_rated;
    for (k, x) in [
        (
            "BOAK_210_n1",
            m.reactor_capex(r, C, Stage::Boak, m.n_linear),
        ),
        (
            "FOAK_210_n1",
            m.reactor_capex(r, C, Stage::Foak, m.n_linear),
        ),
        ("BOAK_210_n06", m.reactor_capex(r, C, Stage::Boak, m.n_team)),
        (
            "BOAK_210_n031",
            m.reactor_capex(r, C, Stage::Boak, m.n_ngnp),
        ),
        (
            "BOAK_179_n1",
            m.reactor_capex(bb.r, C, Stage::Boak, m.n_linear),
        ),
        ("USD_BOAK_210", m.occ_kwt.central * r * 1e3 / 1e6),
        (
            "USD_FOAK_210",
            m.occ_kwt.central * r * 1e3 / 1e6 * m.foak.central,
        ),
        ("F2019", m.f2019),
    ] {
        st.get("capex", v, k, x)?;
    }
    let v = &g["capture"];
    for (k, x) in [
        ("K", m.capt_k.central),
        ("capex", m.capt_k.central * bb.cap_t),
        ("K_high", m.capt_k.high),
    ] {
        st.get("capture", v, k, x)?;
    }
    for life in m.lives {
        st.get(
            "crf",
            &g["crf"],
            &format!("{life:.0}"),
            nush2_plant::anchored::crf(m.rate, life)?,
        )?;
    }
    let c = m.comparator();
    let v = &g["comparator"];
    for (k, x) in [
        ("A_t_y", c.a_t_y),
        ("cac_published", c.cac_published),
        ("ts_adj", c.ts_adj),
        ("cac_T60", c.cac_t60),
        ("cac_T232", c.cac_t232),
        ("cac_T112", c.cac_t112),
        ("overall_capture", c.overall_capture),
    ] {
        st.get("comparator", v, k, x)?;
    }
    for (key, s) in [("walk_B", m.s_b), ("walk_A", 1.0)] {
        let walk = g[key].as_array().ok_or_else(|| missing("root", key))?;
        let got: Vec<f64> = if key == "walk_B" {
            m.sizing_walk(s)?.to_vec()
        } else {
            // Case A walk has one fewer step: the Case B scaling row drops out.
            let w = m.sizing_walk(s)?;
            vec![w[0], w[2], w[3], w[4], w[5], w[6]]
        };
        for (i, x) in walk.iter().enumerate() {
            let want = x.as_f64().ok_or_else(|| missing(key, "step"))?;
            st.cmp(&format!("{key}[{i}]"), got[i], want);
        }
    }
    Ok(())
}

/// `golden_availability`: every value of every availability key.
pub fn availability() -> Result<GoldStats> {
    let g = parse(FREEZE_VALUES)?;
    let mut st = GoldStats::default();
    let keys = g.as_object().ok_or_else(|| missing("root", "keys"))?;
    for (key, v) in keys {
        let avail: f64 = key
            .parse()
            .map_err(|_| EngineError::Parse(format!("bad availability {key}")))?;
        let mut p = ParamSet::embedded()?;
        p.set("AVAILABILITY", avail)?;
        let m = Model::new(&p)?;
        compare_summary(&mut st, &format!("{key}.B"), &m, &v["B"], m.s_b)?;
        compare_summary(&mut st, &format!("{key}.A"), &m, &v["A"], 1.0)?;
        compare_inl(&mut st, &m, v)?;
        compare_grid(&mut st, &m, v)?;
        compare_sens(&mut st, &m, v)?;
        compare_es(&mut st, &m, v)?;
        compare_misc(&mut st, &m, v)?;
    }
    Ok(st)
}
