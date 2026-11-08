//! Property layer: ideal-gas thermochemistry, mixtures, reforming equilibrium,
//! IF97 water, helium and syngas transport. Unit conversions and physical
//! constants live in `units`.
//!
//! Enthalpy is on a formation basis throughout, so reactors need no separate heat
//! of reaction and every energy closure is a plain sum of stream enthalpies.

pub mod equilibrium;
pub mod helium;
pub mod mixture;
pub mod shomate;
pub mod species;
pub mod steam;
pub mod transport;
pub mod units;
