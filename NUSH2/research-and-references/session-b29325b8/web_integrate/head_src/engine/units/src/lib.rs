//! Unit operations. M4 adds the gas units, M5 the reformer tube and M6 the
//! absorber, stripper and regeneration train.

pub mod absorber;
pub mod bench;
pub mod case5;
pub mod columns;
pub mod cooler;
pub mod flash;
pub mod gas;
pub mod jaeri;
mod newton;
pub mod packing;
pub mod regen;
pub mod smr_eq;
pub mod smr_tube;
mod solve;
pub mod solvent;
pub mod stream;
pub mod stripper;
pub mod tube_case_b;
pub mod tube_ht;
pub mod wgs;
pub mod xf;

pub use newton::SolvePath;
