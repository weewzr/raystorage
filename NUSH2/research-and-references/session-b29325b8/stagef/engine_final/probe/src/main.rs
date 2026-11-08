use nush2_units::tube_case_b as tb;
use std::time::Instant;
fn main() {
    let a = nush2_units::jaeri::A2_FIT_ACTIVITY;
    for n in [199.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0] {
        let t = Instant::now();
        match tb::run(n, a) {
            Ok((p, _)) => println!("N {n}: exit {:.3} C X {:.4} he_out {:.2} duty {:.2} ({:.0} ms)", p.exit_c, p.conversion, p.he_out_c, p.duty_mw, t.elapsed().as_secs_f64()*1e3),
            Err(e) => println!("N {n}: {e}"),
        }
    }
    println!("{:?}", tb::tubes_for_exit(871.0, tb::HA_TUBES, 5000.0, a).map(|p| p.n_tubes));
}
