//! The stored 104-point rows must agree with a fresh run, so `nush2_vv` can never read
//! results the code no longer produces. Rewrite them with
//! `cargo run -p nush2_mea --example write_104_rows`.

use nush2_mea::validation::{rows_csv, run_model_points, split_csv, CSV_HEADER};

/// Column of `simulated` in the schema.
const SIMULATED: usize = 7;
/// Columns that carry computed numbers, compared by value.
const COMPUTED: [usize; 3] = [SIMULATED, 9, 10];

#[test]
fn stored_rows_match_a_fresh_run() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/aronu_104_rows.csv");
    let stored = std::fs::read_to_string(path).expect("run the write_104_rows example first");
    let fresh = rows_csv(&run_model_points().unwrap());
    assert_eq!(stored.lines().count(), 105);
    assert_eq!(stored.lines().next(), Some(CSV_HEADER));
    for (s, f) in stored.lines().zip(fresh.lines()).skip(1) {
        let (cs, cf) = (split_csv(s), split_csv(f));
        assert_eq!(cs.len(), 16);
        assert_eq!(cs.len(), cf.len());
        for i in 0..cs.len() {
            if COMPUTED.contains(&i) {
                let (a, b): (f64, f64) = (cs[i].parse().unwrap(), cf[i].parse().unwrap());
                assert!(
                    (a - b).abs() <= 1e-5 * b.abs().max(1e-3),
                    "{} column {i}: {a} against {b}",
                    cs[0]
                );
            } else {
                assert_eq!(cs[i], cf[i], "{} column {i}", cs[0]);
            }
        }
    }
}

#[test]
fn stored_rows_all_carry_the_status_of_their_comparison() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/aronu_104_rows.csv");
    let stored = std::fs::read_to_string(path).expect("stored rows");
    for line in stored.lines().skip(1) {
        let c = split_csv(line);
        let (lit, sim): (f64, f64) = (c[6].parse().unwrap(), c[SIMULATED].parse().unwrap());
        let tol: f64 = c[11].parse().unwrap();
        let status = if (sim / lit - 1.0).abs() <= tol {
            "pass"
        } else {
            "fail"
        };
        assert_eq!(c[12], status, "{}", c[0]);
    }
}
