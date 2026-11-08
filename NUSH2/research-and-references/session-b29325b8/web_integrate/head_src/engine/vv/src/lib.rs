//! Verification and validation harness. Everything here runs against live code at
//! export time, so `results/Validation_Results.csv` holds only comparisons that
//! ran, plus `not_run` rows that say why a benchmark could not run.
//!
//! Plant rows reproduce the screening script and prove the port. Literature rows
//! compare the engine with the sources of `docs/VALIDATION_MATRIX.md`, read from
//! `data/benchmarks` and `data/props`, and judge each against its frozen limit.

pub mod bench;
pub mod fixture;
pub mod gas_bench;
pub mod golden;
pub mod harness;
pub mod mea_bench;
pub mod not_run;
pub mod props_bench;
pub mod report;
mod rowkit;
pub mod units_bench;
