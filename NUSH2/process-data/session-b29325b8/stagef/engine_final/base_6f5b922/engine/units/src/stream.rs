//! Stream, energy and record types shared by every unit model.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Number of tracked gas species.
pub const NSP: usize = 10;

/// Index into `Stream::n`.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sp {
    H2,
    Co,
    Co2,
    Ch4,
    H2o,
    N2,
    C2h6,
    C3h8,
    Nc4h10,
    Nc5h12,
}

/// Stream tag such as "S05", "H1" or "PSA_FEED".
pub type StreamTag = String;
/// Unit tag such as "REFORMER".
pub type UnitTag = String;
/// Energy stream tag.
pub type EnergyTag = String;

/// Process gas and aqueous condensate. Flows are mol/s, total per species.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stream {
    pub tag: StreamTag,
    pub t_k: f64,
    pub p_pa: f64,
    pub n: [f64; NSP],
    /// mol/s of `n[H2o]` present as liquid.
    pub n_liq_h2o: f64,
    /// mol/s of `n[Co2]` dissolved in that liquid.
    pub n_co2_diss: f64,
}

/// Helium stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeStream {
    pub tag: StreamTag,
    pub m_kg_s: f64,
    pub t_k: f64,
    pub p_pa: f64,
}

/// Solvent in apparent species; loading alpha = n_co2 / n_mea.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolventStream {
    pub tag: StreamTag,
    pub t_k: f64,
    pub p_pa: f64,
    pub n_mea: f64,
    pub n_h2o: f64,
    pub n_co2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyKind {
    Heat,
    ShaftWork,
    Electric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyStream {
    pub tag: EnergyTag,
    pub kind: EnergyKind,
    pub w: f64,
    pub t_hot_k: Option<f64>,
    pub t_cold_k: Option<f64>,
    pub from: UnitTag,
    pub to: UnitTag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Port {
    Gas(Stream),
    He(HeStream),
    Solvent(SolventStream),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueClass {
    Published,
    Assumed,
    Calculated,
    UserDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelClass {
    Rigorous,
    ReducedOrder,
    Empirical,
    FixedAssumption,
    ExternalBoundary,
}

/// Written only by the V&V run from real results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Evidence {
    NotRun,
    CodeVerified,
    Validated,
    Extrapolated,
    GateFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boundary {
    Inside,
    External,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Closure {
    pub element_rel: [f64; 4],
    pub energy_rel: f64,
}

/// Free-text condition raised by a unit or the plant.
pub type Flag = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitRecord {
    pub tag: UnitTag,
    pub class: ModelClass,
    pub boundary: Boundary,
    pub inlets: Vec<StreamTag>,
    pub outlets: Vec<StreamTag>,
    pub duties: Vec<EnergyStream>,
    pub closure: Closure,
    pub flags: Vec<Flag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flowsheet {
    pub streams: BTreeMap<StreamTag, Port>,
    pub units: Vec<UnitRecord>,
}
