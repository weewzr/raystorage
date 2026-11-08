//! `nush2` command line. The only crate that touches the disk.
//!
//! nush2 export --out results     write the seven CSVs
//! nush2 json --preset headline   print one plant run as JSON
//! nush2 drift [--markdown]       print Headline against Modelled and the
//!                                section 7 checks

use nush2_numerics::{EngineError, Result};
use nush2_plant::anchored::Model;
use nush2_plant::export::tables_with;
use nush2_plant::flowsheet;
use nush2_plant::gates::Gates;
use nush2_plant::params::ParamSet;
use nush2_plant::{drift, run, Inputs, Preset};
use nush2_vv::report::Status;
use std::path::Path;
use std::process::ExitCode;

fn io_err(path: &Path, e: std::io::Error) -> EngineError {
    EngineError::Parse(format!("{}: {e}", path.display()))
}

fn write(dir: &Path, name: &str, text: &str) -> Result<()> {
    let path = dir.join(name);
    std::fs::write(&path, text).map_err(|e| io_err(&path, e))?;
    println!("wrote {} ({} bytes)", path.display(), text.len());
    Ok(())
}

fn export(out: &str) -> Result<()> {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    let p = ParamSet::embedded()?;
    // Benchmarks first: the gate rows they end with decide the presets.
    let rows = nush2_vv::harness::run_all()?;
    let pairs: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.test_id.as_str(), r.status.label()))
        .collect();
    let gates = Gates::from_gate_rows(&pairs);
    let recorded = Gates::recorded();
    if gates != recorded {
        let diff: Vec<String> = gates
            .states
            .iter()
            .zip(&recorded.states)
            .filter(|(a, b)| a != b)
            .map(|((t, a), (_, b))| format!("{t} {} against recorded {}", a.label(), b.label()))
            .collect();
        return Err(EngineError::Parse(format!(
            "gate outcomes of this export differ from gates::RECORDED ({}); update RECORDED \
             and export again, so the browser and the tables use the same presets",
            diff.join(", ")
        )));
    }
    let t = tables_with(&p, &gates)?;
    write(dir, "Final_Design_Basis.csv", &t.final_design_basis)?;
    write(
        dir,
        "Integrated_Mass_Balance.csv",
        &t.integrated_mass_balance,
    )?;
    write(
        dir,
        "Integrated_Energy_Balance.csv",
        &t.integrated_energy_balance,
    )?;
    write(dir, "Emissions_Ledger.csv", &t.emissions_ledger)?;
    write(dir, "Economic_Ledger.csv", &t.economic_ledger)?;
    write(dir, "Sensitivity_Results.csv", &t.sensitivity_results)?;
    write(dir, "Validation_Results.csv", &nush2_vv::report::csv(&rows))?;
    let count = |s: Status| rows.iter().filter(|r| r.status == s).count();
    println!(
        "validation rows {}: pass {}, fail {}, reported {}, not_run {}",
        rows.len(),
        count(Status::Pass),
        count(Status::Fail),
        count(Status::Reported),
        count(Status::NotRun)
    );
    Ok(())
}

fn json(preset: &str) -> Result<()> {
    let mut inputs = Inputs::headline();
    inputs.preset = match preset {
        "anchored" => Preset::Anchored,
        "headline" => Preset::Headline,
        "modelled" => Preset::Modelled,
        other => {
            return Err(EngineError::Parse(format!("unknown preset {other}")));
        }
    };
    let solution = run(&inputs)?;
    let text =
        serde_json::to_string_pretty(&solution).map_err(|e| EngineError::Parse(e.to_string()))?;
    println!("{text}");
    Ok(())
}

fn drift(markdown: bool) -> Result<()> {
    let p = ParamSet::embedded()?;
    let m = Model::new(&p)?;
    let fs = flowsheet::suite(&m);
    let d = drift::drift(&p, &fs, &Gates::recorded())?;
    if markdown {
        print!("{}", drift::markdown(&d));
        return Ok(());
    }
    for (name, pt) in [
        ("Anchored", d.anchored),
        ("Headline", d.headline),
        ("Modelled", d.modelled),
    ] {
        println!(
            "{name}: reactor need {:.3} MWth, spare {:.3} MWth, abatement {:.0} t/y, gap {:.3} MW, \
             electricity {:.3} MWe",
            pt.r_mwth, pt.spare_mwth, pt.abatement_t_y, pt.gap_mw, pt.e_total_mwe
        );
    }
    for (h, mo) in d.record.headline.iter().zip(&d.record.modelled) {
        println!(
            "{} {}: anchored {:.4} {}, modelled {}, gate {} {}, Headline {:?}, Modelled {:?}",
            h.id,
            h.name,
            h.anchored,
            h.unit,
            h.modelled.map_or("none".to_string(), |v| format!("{v:.4}")),
            h.gate,
            h.gate_state.map_or("none", |g| g.label()),
            h.used,
            mo.used
        );
    }
    if let Some(v) = d.record.phi_flowsheet {
        println!("Level 3 carbon split after the shift at Case B REC_F {v:.4}");
    }
    if let Some(v) = d.record.syngas_flowsheet_mw {
        println!("Level 3 syngas heat above 130 C at Case B REC_F {v:.3} MW");
    }
    if let Ok(fs) = &fs {
        println!(
            "cryogenic H2 return fraction {:.6}, calculated by closure",
            fs.closure.h2_return
        );
    }
    for f in &d.findings {
        println!(
            "check {}: limit {}; value {}; {}{}",
            f.check,
            f.limit,
            f.value,
            if f.raised { "RAISED, " } else { "not raised" },
            if f.raised { f.action } else { "" }
        );
    }
    Ok(())
}

fn flag(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default.to_string())
}

fn dispatch(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("export") => export(&flag(args, "--out", "results")),
        Some("json") => json(&flag(args, "--preset", "headline")),
        Some("drift") => drift(args.iter().any(|a| a == "--markdown")),
        _ => Err(EngineError::Parse(
            "usage: nush2 export --out DIR | json --preset NAME | drift [--markdown]".into(),
        )),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
