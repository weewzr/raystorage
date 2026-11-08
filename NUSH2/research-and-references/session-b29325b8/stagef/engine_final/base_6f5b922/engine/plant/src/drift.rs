//! Drift between the presets, plan section 7. One record holds every term with
//! all its modelled slots, the Headline and Modelled registries built from the
//! gates, the Level 4 rebuild of the reactor side and the reboiler grade check.
//! `drift` runs the three presets on Case B REC_F and reads the section 7
//! limits. A limit that is crossed is a finding: the record states it and no
//! input is changed to avoid it.

use crate::anchored::{Model, Scenario};
use crate::flowsheet::{self, Suite, T_REFORMER_OUT_C};
use crate::gates::Gates;
use crate::heatrec::{self, Pinch};
use crate::params::ParamSet;
use crate::terms::{self, Source, Term};
use crate::utility::{self, Level4};
use crate::Preset;
use nush2_numerics::Result;
use nush2_units::bench::GasTerms;
use nush2_units::case5::{self, T05_LIMIT_K};
use nush2_units::columns::ColumnTerms;
use serde::Serialize;

/// Section 7 limits.
pub const R_LIMIT_MWTH: f64 = 210.0;
pub const ABATEMENT_REF_T_Y: f64 = 578_000.0;
pub const ABATEMENT_TOL_T_Y: f64 = 12_000.0;
pub const SRD_RANGE_GJ_T: (f64, f64) = (3.4, 4.2);
pub const HEIGHT_LIMIT_M: f64 = 40.0;
pub const RICH_CAP: f64 = 0.50;
pub const NG_REF_MMSCFD: f64 = 23.2;
pub const NG_TOL: f64 = 0.05;
pub const SPARE_RANGE_MWTH: (f64, f64) = (27.0, 33.0);

/// Everything the exporter and the drift report need about the terms.
#[derive(Debug, Clone)]
pub struct TermRecord {
    /// Every term with all the slots that computed, sourced for Headline.
    pub headline: Vec<Term>,
    /// The same, sourced for Modelled.
    pub modelled: Vec<Term>,
    pub p_head: ParamSet,
    pub p_mod: ParamSet,
    pub gas: Result<GasTerms>,
    pub columns: Result<ColumnTerms>,
    /// Level 4 rebuild on the Headline balance at Case B REC_F.
    pub level4: Result<Level4>,
    /// Reboiler grade check at the M6 reboiler temperature.
    pub pinch: Option<Pinch>,
    /// Carbon split after the shift in the Level 3 flowsheet at Case B REC_F.
    pub phi_flowsheet: Option<f64>,
    /// Syngas heat above the cut on the flowsheet's own EX3 outlet, MW.
    pub syngas_flowsheet_mw: Option<f64>,
}

/// Build the record on `base`, the flowsheet suite of the anchored model and
/// the gate outcomes.
pub fn record(base: &ParamSet, fs: &Result<Suite>, gates: &Gates) -> Result<TermRecord> {
    let m0 = Model::new(base)?;
    let b0 = m0.balance(Scenario::RecF, &m0.opts(m0.s_b))?;
    let mut list = terms::terms(&m0, &b0);
    let gas = terms::gas_terms_cached();
    if let Ok(g) = &gas {
        terms::fill_m4(&mut list, &m0, m0.s_b, g);
    }
    let columns = terms::column_terms_cached();
    if let Ok(c) = &columns {
        terms::fill_m6(&mut list, c);
    }
    let mut phi_flowsheet = None;
    let mut syngas_flowsheet_mw = None;
    if let Ok(s) = fs {
        terms::fill_m7(&mut list, &s.terms);
        if let Some(run) = s.run("FS_B_REC_F") {
            phi_flowsheet = run.gas("S11").ok().map(case5::phi_after_shift);
            if let Ok(g) = run.gas("S09A") {
                syngas_flowsheet_mw = heatrec::syngas_above_w(g, T05_LIMIT_K)
                    .ok()
                    .map(|w| w / 1.0e6);
            }
        }
    }
    let mut head = list.clone();
    terms::resolve(&mut head, Preset::Headline, gates);
    let p_head = terms::apply(base, &head)?;
    let mh = Model::new(&p_head)?;
    let bh = mh.balance(Scenario::RecF, &mh.opts(mh.s_b))?;
    let level4 = utility::level4(&mh, &bh, T_REFORMER_OUT_C);
    if let Ok(l) = &level4 {
        terms::fill_m8(&mut list, l.t10.total_kpa);
    }
    let mut headline = list.clone();
    terms::resolve(&mut headline, Preset::Headline, gates);
    let mut modelled = list;
    terms::resolve(&mut modelled, Preset::Modelled, gates);
    let p_mod = terms::apply(base, &modelled)?;
    let pinch = columns
        .as_ref()
        .ok()
        .and_then(|c| c.reboiler_t_c)
        .map(|t_reb| {
            heatrec::check_grade(
                t_reb,
                &[
                    ("SELEXOL", m0.t_lp_steam_c),
                    ("EXPORT", m0.t_hp_steam_c),
                    ("SYNGAS", T05_LIMIT_K - 273.15),
                    ("ZLD", m0.t_lp_steam_c),
                    ("HOT_WATER", m0.t_hot_water_c),
                ],
            )
        });
    Ok(TermRecord {
        headline,
        modelled,
        p_head,
        p_mod,
        gas,
        columns,
        level4,
        pinch,
        phi_flowsheet,
        syngas_flowsheet_mw,
    })
}

/// One preset on Case B REC_F.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PresetPoint {
    pub r_mwth: f64,
    pub spare_mwth: f64,
    pub abatement_t_y: f64,
    pub gap_mw: f64,
    pub e_total_mwe: f64,
}

/// Case B REC_F on one registry.
pub fn point(p: &ParamSet) -> Result<PresetPoint> {
    let m = Model::new(p)?;
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b))?;
    Ok(PresetPoint {
        r_mwth: b.r,
        spare_mwth: utility::margin(m.r_rated, b.r),
        abatement_t_y: m.emissions(&b.c, m.s_b).a_atm,
        gap_mw: b.gap,
        e_total_mwe: b.e_total,
    })
}

/// Case B REC_F at the high corner of card J.27, against the rating.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HighCorner {
    pub r_mwth: f64,
    pub rated_mwth: f64,
    /// Heat duties above the rating when the reactor is held at it, MWth.
    pub unmet_heat_mwth: f64,
    /// Electricity import at the rating, heat duties first, MWe.
    pub unmet_e_mwe: f64,
}

/// High corner of card J.27 on one registry.
///
/// eq: J.27
pub fn high_corner(p: &ParamSet) -> Result<HighCorner> {
    let m = Model::new(p)?;
    let b = m.balance_high(Scenario::RecF, m.s_b)?;
    let (short, heat_first) = m.shortfall_at(m.r_rated, &b);
    Ok(HighCorner {
        r_mwth: b.r,
        rated_mwth: m.r_rated,
        unmet_heat_mwth: (heat_first - m.r_rated).max(0.0),
        unmet_e_mwe: short.max(0.0),
    })
}

/// A section 7 check.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    pub check: &'static str,
    pub limit: String,
    pub value: String,
    pub raised: bool,
    /// True for the two checks that stop downstream number use.
    pub blocking: bool,
    pub action: &'static str,
}

/// Drift record: the three presets and the section 7 checks.
#[derive(Debug, Clone)]
pub struct Drift {
    pub record: TermRecord,
    pub anchored: PresetPoint,
    pub headline: PresetPoint,
    pub modelled: PresetPoint,
    pub findings: Vec<Finding>,
}

/// Run the drift on `base` with the gate outcomes.
pub fn drift(base: &ParamSet, fs: &Result<Suite>, gates: &Gates) -> Result<Drift> {
    let record = record(base, fs, gates)?;
    let anchored = point(base)?;
    let headline = point(&record.p_head)?;
    let modelled = point(&record.p_mod)?;
    let mut f = Vec::new();
    for (name, pt) in [("Headline", headline), ("Modelled", modelled)] {
        let raised = pt.r_mwth > R_LIMIT_MWTH;
        f.push(Finding {
            check: if name == "Headline" {
                "Headline R above 210 MWth"
            } else {
                "Modelled R above 210 MWth"
            },
            limit: format!("R > {R_LIMIT_MWTH:.0} MWth"),
            value: if raised {
                format!(
                    "{:.2} MWth: shortfall {:.2} MWth, minimum feasible rating {:.2} MWth",
                    pt.r_mwth,
                    pt.r_mwth - R_LIMIT_MWTH,
                    pt.r_mwth
                )
            } else {
                format!("{:.2} MWth", pt.r_mwth)
            },
            raised,
            blocking: true,
            action: "Blocking scientific issue under brief s4; tell the user",
        });
    }
    let hc = high_corner(&record.p_head)?;
    f.push(Finding {
        check: "Headline high-corner R above the 210 MWth rating",
        limit: format!("high-corner R > {R_LIMIT_MWTH:.0} MWth"),
        value: format!(
            "{:.2} MWth, {:+.2} MWth against the rating; with the reactor held at {:.0} MWth \
             the unmet heat is {:.2} MWth and the unmet electricity {:.2} MWe",
            hc.r_mwth,
            hc.r_mwth - hc.rated_mwth,
            hc.rated_mwth,
            hc.unmet_heat_mwth,
            hc.unmet_e_mwe
        ),
        raised: hc.r_mwth > R_LIMIT_MWTH,
        blocking: false,
        action: "State in report: the high corner needs more than the rated reactor",
    });
    let off = modelled.abatement_t_y - ABATEMENT_REF_T_Y;
    f.push(Finding {
        check: "Modelled abatement against 0.578 Mt/y",
        limit: "off by more than 0.012 Mt/y".into(),
        value: format!(
            "{:.4} Mt/y, {:+.4} Mt/y",
            modelled.abatement_t_y / 1.0e6,
            off / 1.0e6
        ),
        raised: off.abs() > ABATEMENT_TOL_T_Y,
        blocking: true,
        action: "Blocking scientific issue under brief s4; tell the user",
    });
    let srd = record.columns.as_ref().ok().and_then(|c| c.t06_srd_gj_t);
    f.push(Finding {
        check: "Modelled SRD",
        limit: "outside 3.4 to 4.2 GJ/t".into(),
        value: srd.map_or("not run".into(), |v| format!("{v:.3} GJ/t")),
        raised: srd.is_some_and(|v| !(SRD_RANGE_GJ_T.0..=SRD_RANGE_GJ_T.1).contains(&v)),
        blocking: false,
        action: "State in report, keep 3.8, show the modelled line",
    });
    let (height, rich) = record
        .columns
        .as_ref()
        .map(|c| (c.t02_height_m, c.t02_rich_loading))
        .unwrap_or((None, f64::NAN));
    f.push(Finding {
        check: "90% reachable at rich loading 0.50",
        limit: "needs above 0.50, or packing above 40 m".into(),
        value: match height {
            Some(z) => format!("{z:.2} m at rich loading {rich:.3}"),
            None => "90% not reached within 40 m".into(),
        },
        raised: height.is_none_or(|z| z > HEIGHT_LIMIT_M) || rich > RICH_CAP + 1e-6,
        blocking: false,
        action: "State in report; absorber flagged",
    });
    let ng = record
        .headline
        .iter()
        .find(|t| t.id == "T13")
        .and_then(|t| t.modelled);
    f.push(Finding {
        check: "Modelled NG for 8,988 kg/h",
        limit: "off 23.2 MMSCFD by over 5%".into(),
        value: ng.map_or("not run".into(), |v| {
            format!("{v:.3} MMSCFD, {:+.2}%", 100.0 * (v / NG_REF_MMSCFD - 1.0))
        }),
        raised: ng.is_some_and(|v| (v / NG_REF_MMSCFD - 1.0).abs() > NG_TOL),
        blocking: false,
        action: "State in report",
    });
    f.push(Finding {
        check: "Headline spare",
        limit: "outside 27 to 33 MWth".into(),
        value: format!("{:.2} MWth", headline.spare_mwth),
        raised: !(SPARE_RANGE_MWTH.0..=SPARE_RANGE_MWTH.1).contains(&headline.spare_mwth),
        blocking: false,
        action: "Log it, print the computed value, keep going",
    });
    Ok(Drift {
        record,
        anchored,
        headline,
        modelled,
        findings: f,
    })
}

/// Markdown table of the drift, one row per term, for the design decisions doc.
pub fn markdown(d: &Drift) -> String {
    let mut out = String::new();
    out.push_str(
        "| Term | Name | Unit | Anchored | Modelled | Headline uses | Modelled uses | Gate | Gate outcome | Modelled slot |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
    for (h, m) in d.record.headline.iter().zip(&d.record.modelled) {
        let fmt = |v: f64| {
            if v.abs() >= 100.0 {
                format!("{v:.2}")
            } else {
                format!("{v:.4}")
            }
        };
        let src = |t: &Term| match t.used {
            Source::Anchored => format!("{} (anchored)", fmt(t.anchored)),
            Source::Modelled => format!("{} (model)", fmt(t.used_value())),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            h.id,
            h.name,
            h.unit,
            fmt(h.anchored),
            h.modelled.map_or("none".into(), fmt),
            src(h),
            src(m),
            h.gate,
            h.gate_state.map_or("no gate", |g| g.label()),
            if h.modelled_note.is_empty() {
                "no model"
            } else {
                h.modelled_note
            },
        ));
    }
    out
}

/// Re-export for callers that hold only this module.
pub fn suite(m: &Model) -> Result<Suite> {
    flowsheet::suite(m)
}
