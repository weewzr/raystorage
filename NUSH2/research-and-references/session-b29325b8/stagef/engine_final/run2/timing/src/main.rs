use std::time::Instant;
fn main() {
    let a = nush2_units::jaeri::A2_FIT_ACTIVITY;
    for i in 0..3 {
        let t = Instant::now();
        let (p, _) = nush2_units::tube_case_b::run(199.0, a).unwrap();
        let tube = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let d = nush2_units::columns::design_duties().unwrap();
        let cols = t.elapsed().as_secs_f64() * 1e3;
        println!("run {i}: tube_case_b::run(199) {tube:.1} ms (exit {:.1} C); columns::design_duties {cols:.1} ms (reboiler {:.2} MW)", p.exit_c, d.reboiler_mw);
    }
}
