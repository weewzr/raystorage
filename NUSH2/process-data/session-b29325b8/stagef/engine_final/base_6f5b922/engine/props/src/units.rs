//! Physical constants and unit conversions. These are standard definitions,
//! so they live here rather than in `engine/data/parameters.toml`.
//!
//! Internal units are SI with mol/s and W. Project units such as MMSCFD, st/d
//! and kmol/h appear only at source-data and export edges.

/// kg per pound.
pub const LB: f64 = 0.45359237;
/// t per short ton.
pub const ST: f64 = 0.90718474;
/// GJ per MMBtu.
pub const MMBTU_GJ: f64 = 1.05505585;
/// scf per lbmol at 60 F and 14.696 psia, the INL standard (TEV-961 Table 2 note 1).
pub const SCF_LBMOL: f64 = 379.49;
/// kg/kmol.
pub const MW_CO2: f64 = 44.0095;
/// kg/kmol.
pub const MW_H2: f64 = 2.01588;
/// kg/kmol of monoethanolamine.
pub const MW_MEA: f64 = 61.08;
/// MJ per MWh.
pub const MJ_PER_MWH: f64 = 3600.0;
/// GJ per MWh, also the factor between GJ/h and MW.
pub const GJ_PER_MWH: f64 = 3.6;
/// h per calendar year.
pub const HOURS_PER_YEAR: f64 = 8760.0;
/// d per calendar year.
pub const DAYS_PER_YEAR: f64 = 365.0;
/// Lower heating value of H2, kJ/mol.
pub const LHV_H2: f64 = 241.8;
/// Lower heating value of CH4, kJ/mol.
pub const LHV_CH4: f64 = 802.3;
/// Lower heating value of CO, kJ/mol.
pub const LHV_CO: f64 = 283.0;
/// kg of water per kmol.
pub const MW_H2O: f64 = 18.015;
/// Molar gas constant, J/mol/K, the product of the exact SI values of the Avogadro
/// and Boltzmann constants.
pub const R_GAS: f64 = 6.022_140_76e23 * 1.380_649e-23;
/// Pa per bar.
pub const BAR_PA: f64 = 1.0e5;

/// CO2 flow from st/d to kmol/h.
pub fn st_d_to_kmol_h(st_d: f64) -> f64 {
    st_d * ST * 1000.0 / 24.0 / MW_CO2
}

/// Gas flow from MMSCFD to kmol/h.
pub fn mmscfd_to_kmol_h(mmscfd: f64) -> f64 {
    mmscfd * 1.0e6 / SCF_LBMOL / 24.0 * LB
}

/// CO2 flow from kmol/h to t/h.
pub fn kmol_h_to_t_h(kmol_h: f64) -> f64 {
    kmol_h * MW_CO2 / 1000.0
}

/// Heat flow from MMBtu/h to MW.
pub fn mmbtu_h_to_mw(mmbtu_h: f64) -> f64 {
    mmbtu_h * MMBTU_GJ / GJ_PER_MWH
}

/// Heat flow from MW to MMBtu/h.
pub fn mw_to_mmbtu_h(mw: f64) -> f64 {
    mw * GJ_PER_MWH / MMBTU_GJ
}
