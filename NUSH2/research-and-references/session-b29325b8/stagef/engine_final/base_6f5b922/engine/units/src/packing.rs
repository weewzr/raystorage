//! Structured packing geometry for the column models. Every number is read from a
//! local page.
//!
//! Mellapak 250Y, the packing the derivations assume for the plant, has no local
//! geometry (derivation note M17: channel side and the Billet and Schultes
//! constants are not in the library). RSP-250 is the nearest packing with a full
//! local data set: the same 250 m2/m3 class, read from Plaza 2012 Table 4.18. The
//! plant absorber and the Sonderby benchmark therefore run on RSP-250 geometry,
//! labelled as a stand-in. No random packing in the library carries a specific
//! area or a critical surface tension, so Onda's Eqs 5a to 5c cannot run on any
//! local packing, and the Khan benchmark runs cannot be set up (see `bench`).

/// Corrugated sheet packing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Structured {
    pub name: &'static str,
    /// Specific area, m2/m3.
    pub a_m2_m3: f64,
    /// Corrugation channel side S, m.
    pub channel_side_m: f64,
    /// Void fraction.
    pub void: f64,
    /// Corrugation angle, degrees.
    pub angle_deg: f64,
    pub source: &'static str,
}

/// RSP-250, Plaza 2012 Table 4.18, PDF p156 (thesis p121).
pub const RSP_250: Structured = Structured {
    name: "RSP-250",
    a_m2_m3: 250.0,
    channel_side_m: 0.014,
    void: 0.99,
    angle_deg: 45.0,
    source: "Plaza 2012 Table 4.18 PDF p156",
};

/// Flexipac 1Y. Area 420 m2/m3 from Isa 2017 PDF p9. Channel side, void fraction
/// and angle are the Flexipac AQ Style 20 column of Plaza 2012 Table 4.18, PDF p156,
/// whose footnote a states that packing was modelled as Flexipac 1Y except for the
/// surface area. The angle there is footnoted to Chen (2007) and may belong to the
/// AQ Style 20 alone; no correlation used here reads it.
pub const FLEXIPAC_1Y: Structured = Structured {
    name: "Flexipac 1Y",
    a_m2_m3: 420.0,
    channel_side_m: 0.009,
    void: 0.91,
    angle_deg: 50.0,
    source: "Isa 2017 PDF p9 (area); Plaza 2012 Table 4.18 PDF p156 (geometry)",
};
