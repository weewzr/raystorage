//! Runs the 104 model-column points and stores the rows for `nush2_vv`. A library
//! crate may not touch files, so this step lives in an example.

use nush2_mea::validation::{rows_csv, run_model_points, summarise};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rows = run_model_points()?;
    let s = summarise(&rows)?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/aronu_104_rows.csv");
    std::fs::write(path, rows_csv(&rows))?;
    println!(
        "wrote {} rows: AARD {:.3}%, worst {:.3}%, failed {}",
        s.n, s.aard_pct, s.max_abs_dev_pct, s.failed_points
    );
    Ok(())
}
