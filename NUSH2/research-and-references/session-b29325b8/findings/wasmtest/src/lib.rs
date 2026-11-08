//! Zero-dependency smoke test: a tiny property function with floating point
//! exp/ln/powf, exported with the plain C ABI so no glue tool is needed.
#[no_mangle]
pub extern "C" fn k_wgs(t_kelvin: f64) -> f64 {
    (4400.0 / t_kelvin - 4.063).exp()
}

#[no_mangle]
pub extern "C" fn he_cp0() -> f64 {
    2.5 * 8.3144598 / 0.004002602
}

#[no_mangle]
pub extern "C" fn pow_check(x: f64, y: f64) -> f64 {
    x.powf(y).ln()
}
