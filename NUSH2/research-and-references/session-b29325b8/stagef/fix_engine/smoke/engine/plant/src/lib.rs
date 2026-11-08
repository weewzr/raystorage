//! NUS-H2 plant model: the anchored screening port, the annual ledger, the
//! discounted cash flow, the Level 3 flowsheet, the Level 4 heat recovery and
//! utility balance, the term gates and the exporter. No file access here: the
//! CLI owns the disk, and this crate returns strings.

pub mod anchored;
pub mod carbon;
pub mod comparator;
pub mod dcf;
pub mod drift;
pub mod economics;
pub mod emissions;
pub mod export;
pub mod flowsheet;
pub mod gates;
pub mod heatrec;
pub mod params;
pub mod sens;
pub mod terms;
pub mod utility;

use anchored::{Balance, Carbon, Case, EUse, Model, Scenario, Stack};
use dcf::DcfResult;
use economics::{CostQuery, Ledger, Stage};
use emissions::Emissions;
use gates::Gates;
use nush2_numerics::Result;
use nush2_units::stream::{Flag, Flowsheet};
use params::ParamSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use terms::Term;

/// Which source each term uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Anchored,
    Headline,
    Modelled,
}

/// Solver budget. Only `Fast` exists at M1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Fast,
    Full,
}

/// One plant run. `overrides` maps registry ids to values held in memory.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Inputs {
    pub case: Case,
    pub scenario: Scenario,
    pub preset: Preset,
    pub tier: Tier,
    #[serde(default)]
    pub overrides: BTreeMap<String, f64>,
}

impl Inputs {
    /// Case B on the recycle scenario.
    pub fn headline() -> Self {
        Self {
            case: Case::B,
            scenario: Scenario::RecF,
            preset: Preset::Headline,
            tier: Tier::Fast,
            overrides: BTreeMap::new(),
        }
    }

    /// Registry with the overrides applied and no preset, refused if any
    /// override is unknown or non-finite.
    pub fn base_params(&self) -> Result<ParamSet> {
        let mut p = ParamSet::embedded()?;
        for (id, v) in &self.overrides {
            p.set(id, *v)?;
        }
        Ok(p)
    }

    /// Registry of this run: the overrides, then the flips of the preset on the
    /// recorded gates, then the overrides again, so an override of an input a
    /// flip replaces wins.
    pub fn params(&self) -> Result<ParamSet> {
        let base = self.base_params()?;
        let mut p = terms::preset_params(&base, self.preset, &Gates::recorded())?;
        for (id, v) in &self.overrides {
            p.set(id, *v)?;
        }
        Ok(p)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HeatBalance {
    pub q_proc: f64,
    pub q_recyc: f64,
    pub q_reb: f64,
    pub stack: Stack,
    pub avail: f64,
    pub gap: f64,
    pub q_ihx: f64,
    pub q_duct: f64,
    pub qv: f64,
    pub r_needed: f64,
    pub r_rated: f64,
    pub margin: f64,
    /// Heat duty above the rated reactor, MWth (`Model::unmet_thermal`).
    pub unmet: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PowerBalance {
    pub rows: EUse,
    pub e_proc: f64,
    pub e_circ: f64,
    pub e_total: f64,
    pub e_nuc: f64,
    pub import_at_rated: f64,
}

/// One printed cost point: both methods at one stage, life and exponent.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CostPoint {
    pub stage: Stage,
    pub life_y: f64,
    pub n: f64,
    pub r_charged: f64,
    pub ledger: Ledger,
    pub ladder_sm_y: [f64; 6],
    pub ac_l3: f64,
    pub ac_l5: f64,
    pub dcf: DcfResult,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EconomicLedger {
    pub points: Vec<CostPoint>,
}

/// Full result of one run.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlantSolution {
    pub flowsheet: Option<Flowsheet>,
    /// Fresh natural gas of this run's case, MMSCFD (`Model::ng_mmscfd`). The
    /// term T13 stays on the Case B basis of the term table.
    pub ng_mmscfd: f64,
    pub terms: Vec<Term>,
    pub carbon: Carbon,
    pub heat: HeatBalance,
    pub power: PowerBalance,
    pub emissions: Emissions,
    pub economics: EconomicLedger,
    pub flags: Vec<Flag>,
}

/// Size charged in a run: the rated reactor at Case B, the need at Case A.
pub fn charged(m: &Model, case: Case, b: &Balance) -> f64 {
    match case {
        Case::A => b.r,
        Case::B => m.r_rated,
    }
}

/// Evaluate one printed cost point.
pub fn cost_point(
    m: &Model,
    b: &Balance,
    stage: Stage,
    life: f64,
    n: f64,
    r: f64,
) -> Result<CostPoint> {
    let a = m.emissions(&b.c, b.s).a_atm;
    let q = CostQuery::central(stage, life, n, r);
    let ledger = m.ledger(b, &q)?;
    let ladder = ledger.ladder();
    Ok(CostPoint {
        stage,
        life_y: life,
        n,
        r_charged: r,
        ledger,
        ladder_sm_y: ladder,
        ac_l3: ladder[3] * 1e6 / a,
        ac_l5: ladder[5] * 1e6 / a,
        dcf: m.dcf(b, &q)?,
    })
}

/// One line naming the terms a preset takes from a model and the gated terms
/// that stay anchored for want of a passed gate.
pub fn preset_flag(preset: Preset, list: &[Term]) -> Flag {
    if preset == Preset::Anchored {
        return "Anchored preset: every term on its anchored value".into();
    }
    let used: Vec<&str> = list
        .iter()
        .filter(|t| t.used == terms::Source::Modelled)
        .map(|t| t.id)
        .collect();
    let held: Vec<String> = list
        .iter()
        .filter(|t| terms::wants_model(preset, t.id, Some(gates::GateState::Passed)))
        .filter(|t| t.used == terms::Source::Anchored)
        .map(|t| {
            format!(
                "{} gate {}",
                t.id,
                t.gate_state.map_or("none", |g| g.label())
            )
        })
        .collect();
    let name = if preset == Preset::Headline {
        "Headline"
    } else {
        "Modelled"
    };
    let or_none = |v: Vec<String>| {
        if v.is_empty() {
            "none".to_string()
        } else {
            v.join(", ")
        }
    };
    format!(
        "{name} preset uses the model of {}; anchored for want of a passed gate: {}",
        or_none(used.iter().map(|s| s.to_string()).collect()),
        or_none(held)
    )
}

/// Run the plant.
pub fn run(inputs: &Inputs) -> Result<PlantSolution> {
    let p = inputs.params()?;
    let m = Model::new(&p)?;
    let s = m.scale(inputs.case);
    let b = m.balance(inputs.scenario, &m.opts(s))?;
    let r_charged = charged(&m, inputs.case, &b);
    let mut points = Vec::new();
    for stage in [Stage::Boak, Stage::Foak] {
        for life in [m.lives[0], m.lives[2]] {
            for n in [m.n_linear, m.n_team, m.n_ngnp] {
                points.push(cost_point(&m, &b, stage, life, n, r_charged)?);
            }
        }
    }
    let (shortfall, _) = m.shortfall_at(m.r_rated, &b);
    let mut term_list =
        terms::for_preset(&inputs.base_params()?, inputs.preset, &Gates::recorded())?;
    // The status the exporter gives rows of this preset: the gated terms it
    // took from their models, or the anchored port when it took none.
    let gated: Vec<&str> = term_list
        .iter()
        .filter(|t| t.used == terms::Source::Modelled)
        .map(|t| t.id)
        .collect();
    let mut flags: Vec<Flag> = vec![if gated.is_empty() {
        "status=anchored_port".to_string()
    } else {
        format!("status=gated_{}", gated.join("_"))
    }];
    flags.push(preset_flag(inputs.preset, &term_list));
    let mut sheet = None;
    if inputs.tier == Tier::Full {
        let fs = flowsheet::suite(&m)?;
        terms::fill_m7(&mut term_list, &fs.terms);
        let id = format!(
            "FS_{}_{}",
            match inputs.case {
                Case::A => "A",
                Case::B => "B",
            },
            inputs.scenario.id()
        );
        match fs.run(&id) {
            Some(r) => {
                sheet = Some(r.flowsheet.clone());
                flags.extend(r.flags.iter().cloned());
            }
            None => flags.push(format!(
                "{} has no Level 3 flowsheet at M7; terms carry the Case B REC_F slots",
                inputs.scenario.id()
            )),
        }
        flags.push(
            "full tier ran the Level 3 flowsheet with anchored columns, since the M6 column \
             gate failed"
                .into(),
        );
    }
    Ok(PlantSolution {
        flowsheet: sheet,
        ng_mmscfd: m.ng_mmscfd(s),
        terms: term_list,
        carbon: b.c,
        heat: HeatBalance {
            q_proc: b.q_proc,
            q_recyc: b.q_recyc,
            q_reb: b.q_reb,
            stack: b.stack,
            avail: b.avail,
            gap: b.gap,
            q_ihx: b.q_ihx,
            q_duct: b.q_duct,
            qv: b.qv,
            r_needed: b.r,
            r_rated: m.r_rated,
            margin: m.r_rated - b.r,
            unmet: m.unmet_thermal(m.r_rated, &b),
        },
        power: PowerBalance {
            rows: b.e_rows,
            e_proc: b.e_proc,
            e_circ: b.e_circ,
            e_total: b.e_total,
            e_nuc: b.e_nuc,
            import_at_rated: shortfall.max(0.0),
        },
        emissions: m.emissions(&b.c, s),
        economics: EconomicLedger { points },
        flags,
    })
}
