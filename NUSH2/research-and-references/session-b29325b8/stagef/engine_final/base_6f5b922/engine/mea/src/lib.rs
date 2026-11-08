//! MEA solvent thermodynamics and transport, milestone M3.
//!
//! Equilibrium: extended UNIQUAC after Aronu et al. (2011), nine species, in `aronu`
//! and `speciate`, with p* in `pstar`. Properties: Weiland density and viscosity in
//! `density` and `viscosity`. Rates: the Khan et al. (2011) closure in `kinetics`.
//! Heat of absorption: `enthalpy`. The published model columns run from `validation`.
//!
//! Temperatures are K. p* stays in kPa on the prototype's convention, and the name of
//! each function states its unit.

// Index loops over fixed species arrays read closest to the printed equations.
#![allow(clippy::needless_range_loop)]

pub mod aronu;
pub mod density;
pub mod enthalpy;
pub mod kinetics;
pub mod pstar;
mod solve;
pub mod speciate;
pub mod validation;
pub mod viscosity;
pub mod water;

pub use pstar::{alpha_at, pstar_kpa, pstar_lookup_kpa};
pub use speciate::{speciate, Speciation};
