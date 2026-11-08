//! Verification harness. Everything here runs against live code at export time,
//! so `results/Validation_Results.csv` holds only tests that ran. At M1 every row
//! is a reproduction of the screening script or an arithmetic check, which proves
//! the port and says nothing about the physics.

pub mod bench;
pub mod golden;
pub mod harness;
pub mod report;
