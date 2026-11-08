//! `nush2` command line. The only crate that touches the disk.
//!
//! nush2 export --out results     write the seven CSVs
//! nush2 json --preset headline   print one plant run as JSON
//! nush2 drift                    print Headline against Modelled

use nush2_numerics::{EngineError, Result};
use nush2_plant::anchored::Model;
use nush2_plant::export::tables;
use nush2_plant::flowsheet;
use nush2_plant::params::ParamSet;
use nush2_plant::{run, Inputs, Preset};
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
    let t = tables(&p)?;
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
    let rows = nush2_vv::harness::run_all()?;
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

fn drift() -> Result<()> {
    let m = Model::embedded()?;
    let s = run(&Inputs::headline())?;
    println!(
        "no modelled term has passed a gate yet, so Headline and Modelled both equal Anchored"
    );
    println!(
        "reactor need {:.2} MWth, rated {:.2} MWth",
        s.heat.r_needed, m.r_rated
    );
    println!("abatement {:.0} t/y", s.emissions.a_atm);
    let fs = flowsheet::suite(&m)?;
    let mut terms = s.terms.clone();
    nush2_plant::terms::fill_m7(&mut terms, &fs.terms);
    for t in &terms {
        match t.modelled {
            Some(v) => println!(
                "{} {} {:.4} {} gate {}, modelled {:.4} (not used)",
                t.id, t.name, t.anchored, t.unit, t.gate, v
            ),
            None => println!(
                "{} {} {:.4} {} gate {}",
                t.id, t.name, t.anchored, t.unit, t.gate
            ),
        }
    }
    println!(
        "cryogenic H2 return fraction {:.6}, calculated by closure",
        fs.closure.h2_return
    );
    for c in flowsheet::checks(&m, &fs)? {
        if c.test_id.starts_with("FS_CARBON") || c.test_id == "FS_GATE_M7" {
            println!(
                "{} anchored {:.3} flowsheet {:.3} {}",
                c.test_id,
                c.literature.unwrap_or(f64::NAN),
                c.simulated,
                if c.pass { "pass" } else { "fail" }
            );
        }
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
        Some("drift") => drift(),
        _ => Err(EngineError::Parse(
            "usage: nush2 export --out DIR | json --preset NAME | drift".into(),
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
