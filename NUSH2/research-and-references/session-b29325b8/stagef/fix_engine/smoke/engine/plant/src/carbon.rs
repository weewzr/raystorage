//! Carbon ledger read from flowsheet streams.
//!
//! Rewritten from an earlier PSA carbon closure for the new stream
//! set, which adds C4 and C5 and tracks N2. The rule carried over is that a
//! missing stream is unknown: a closure with any stream absent returns `None`,
//! and no missing carbon is ever set to zero.

use nush2_units::stream::{Sp, Stream, NSP};
use serde::Serialize;

/// Carbon atoms per molecule, by `Sp` index.
pub const CARBON_ATOMS: [f64; NSP] = [0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 3.0, 4.0, 5.0];

/// Carbon flow of a species array, in the unit of the array.
pub fn carbon_of(n: &[f64; NSP]) -> f64 {
    n.iter().zip(CARBON_ATOMS).map(|(v, c)| v * c).sum()
}

/// Carbon flow of a stream, kmol C/h.
pub fn carbon_kmol_h(s: &Stream) -> f64 {
    carbon_of(&s.n) * 3.6
}

/// Nitrogen flow of a stream as N2, kmol/h. Carried so the ledger shows where the
/// feed nitrogen leaves, since the cryogenic cut is its only outlet.
pub fn n2_kmol_h(s: &Stream) -> f64 {
    s.n[Sp::N2 as usize] * 3.6
}

/// Elemental closure over carbon flows, kmol C/h.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarbonClosure {
    pub input_kmol_c_h: f64,
    pub output_kmol_c_h: f64,
    pub residual_kmol_c_h: f64,
    pub relative_residual: f64,
}

/// Close carbon over the given flows, kmol C/h. `None` when any flow is unknown,
/// when either side is empty, or when the sums are not finite.
///
/// eq: J.11
pub fn close_carbon(inputs: &[Option<f64>], outputs: &[Option<f64>]) -> Option<CarbonClosure> {
    if inputs.is_empty() || outputs.is_empty() {
        return None;
    }
    let mut inp = 0.0;
    for v in inputs {
        inp += (*v)?;
    }
    let mut out = 0.0;
    for v in outputs {
        out += (*v)?;
    }
    if !(inp.is_finite() && out.is_finite()) || inp <= 0.0 || out < 0.0 {
        return None;
    }
    Some(CarbonClosure {
        input_kmol_c_h: inp,
        output_kmol_c_h: out,
        residual_kmol_c_h: inp - out,
        relative_residual: (inp - out) / inp,
    })
}

/// Carbon flows of one flowsheet run, kmol C/h, named as in the anchored ledger
/// (cards J.8 to J.11) so the two can be compared line by line. `condensate` has
/// no anchored counterpart: the screening balance leaves dissolved CO2 out.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarbonLedger {
    /// Fresh natural gas, S01.
    pub feed: f64,
    /// Reformer feed after the recycle joins, S04.
    pub tube: f64,
    /// CO2 in the absorber feed, S11.
    pub co2_abs: f64,
    /// CO2 to storage, S20.
    pub captured: f64,
    /// CO2 left in the treated gas, S12.
    pub slip: f64,
    /// PSA reject, S14.
    pub tail_gas: f64,
    /// Recycle to the reformer, S03.
    pub recycled: f64,
    /// Nitrogen-rich cut to the burner, S15.
    pub burned: f64,
    /// Methane in the H2 product, S13.
    pub product: f64,
    /// CO2 dissolved in the process condensate, S10.
    pub condensate: f64,
    /// Feed nitrogen leaving in the cut, kmol N2/h.
    pub n2_cut: f64,
}

impl CarbonLedger {
    /// Carbon closure over the plant: fresh feed in; storage, stack, product and
    /// condensate out.
    ///
    /// eq: J.11
    pub fn closure(&self) -> Option<CarbonClosure> {
        close_carbon(
            &[Some(self.feed)],
            &[
                Some(self.captured),
                Some(self.burned),
                Some(self.product),
                Some(self.condensate),
            ],
        )
    }

    /// Recycled share of the tail-gas carbon, the quantity INL's split fixes.
    ///
    /// eq: J.10
    pub fn recycle_share(&self) -> f64 {
        self.recycled / self.tail_gas
    }

    /// Per-pass capture and plant capture, printed together.
    ///
    /// eq: J.33
    /// eq: K.14
    pub fn capture_pair(&self) -> (f64, f64) {
        (self.captured / self.co2_abs, self.captured / self.feed)
    }
}

/// Ratio of two reported CO2 streams. Plant capture comes from
/// [`CarbonLedger::capture_pair`] instead.
pub fn reported_co2_stream_ratio(captured: f64, emitted: f64) -> Option<f64> {
    if !(captured.is_finite() && emitted.is_finite())
        || captured < 0.0
        || emitted < 0.0
        || captured + emitted <= 0.0
    {
        return None;
    }
    Some(captured / (captured + emitted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_j_11_closed_synthetic_case() {
        let r = close_carbon(&[Some(100.0)], &[Some(60.0), Some(40.0)]).unwrap();
        assert!(r.residual_kmol_c_h.abs() < 1e-12);
    }

    #[test]
    fn eq_j_11_unclosed_case_keeps_its_residual() {
        let r = close_carbon(&[Some(100.0)], &[Some(90.0)]).unwrap();
        assert!((r.residual_kmol_c_h - 10.0).abs() < 1e-12);
        assert!((r.relative_residual - 0.1).abs() < 1e-12);
    }

    #[test]
    fn eq_j_11_missing_tail_gas_is_never_zero() {
        assert!(close_carbon(&[Some(100.0)], &[Some(90.0), None]).is_none());
        assert!(close_carbon(&[], &[Some(1.0)]).is_none());
        assert!(close_carbon(&[Some(f64::NAN)], &[Some(1.0)]).is_none());
    }

    #[test]
    fn carbon_counts_every_carbon_species() {
        let mut n = [0.0; NSP];
        n[Sp::Ch4 as usize] = 1.0;
        n[Sp::Co as usize] = 1.0;
        n[Sp::Co2 as usize] = 1.0;
        n[Sp::C2h6 as usize] = 1.0;
        n[Sp::C3h8 as usize] = 1.0;
        n[Sp::Nc4h10 as usize] = 1.0;
        n[Sp::Nc5h12 as usize] = 1.0;
        n[Sp::H2 as usize] = 5.0;
        n[Sp::H2o as usize] = 5.0;
        n[Sp::N2 as usize] = 5.0;
        assert_eq!(carbon_of(&n), 17.0);
        let s = Stream::gas("X", 300.0, 1.0e5, n);
        let el = s.elements().unwrap();
        assert!((carbon_of(&s.n) - el[0]).abs() < 1e-12);
    }

    #[test]
    fn reported_ratio_is_a_stream_ratio() {
        let r = reported_co2_stream_ratio(1927.0, 142.0).unwrap();
        assert!((r - 1927.0 / 2069.0).abs() < 1e-12);
        assert!(reported_co2_stream_ratio(-1.0, 1.0).is_none());
    }
}
