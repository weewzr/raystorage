//! Heat of absorption by Gibbs-Helmholtz on p*(T), Aronu Eq 22 (PDF p11), at fixed
//! apparent composition and a difference of 2 K either side of `t_k`.
//!
//! Evidence class: plausibility. Aronu, citing Kim and Svendsen (2007), warns that this
//! estimate is not necessarily accurate (PDF p11), the Kim and Svendsen data exist
//! locally only as figures, and K_MEA in Table 2 is rated for 0 to 50 C and H for 0 to
//! 100 C (PDF p4), yet both run to 120 C here. So the output carries no validation
//! claim.
//!
//! No solvent heat capacity function lives here. Aronu prints none, and the local
//! values are figures only (Plaza Fig 3.13, Zhang Fig 15), so the 3.6 kJ/kg/K in REGEN
//! remains an assumption of that unit.

use crate::pstar::{pstar_kpa, R_GAS};
use nush2_numerics::Result;

const DT_K: f64 = 2.0;

/// Heat released per mol of CO2 absorbed, J/mol, positive for exothermic absorption.
///
/// eq: G.24
pub fn dh_abs_j_mol(t_k: f64, w_mea: f64, alpha: f64) -> Result<f64> {
    let lo = pstar_kpa(t_k - DT_K, w_mea, alpha)?.ln();
    let hi = pstar_kpa(t_k + DT_K, w_mea, alpha)?.ln();
    Ok(-R_GAS * (hi - lo) / (1.0 / (t_k + DT_K) - 1.0 / (t_k - DT_K)))
}
