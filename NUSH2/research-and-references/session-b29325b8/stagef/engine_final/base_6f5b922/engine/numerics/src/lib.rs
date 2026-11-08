//! Shared error type and small numerical helpers. No dependencies, no I/O.

use std::fmt;

/// Every library API returns this error. User input never reaches a panic.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// A value lies outside its stated validity range.
    Range { param: String, msg: String },
    /// A parameter id is absent from the registry.
    UnknownParam(String),
    /// The embedded parameter file or a request could not be parsed.
    Parse(String),
    /// An iteration stopped without meeting its tolerance.
    NotConverged(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Range { param, msg } => write!(f, "range error on {param}: {msg}"),
            EngineError::UnknownParam(id) => write!(f, "unknown parameter {id}"),
            EngineError::Parse(msg) => write!(f, "parse error: {msg}"),
            EngineError::NotConverged(msg) => write!(f, "not converged: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type Result<T> = std::result::Result<T, EngineError>;

/// Plain fixed-point iteration x = g(x), stopping when |g(x) - x| < tol.
///
/// The last evaluation of `g` is left in `state`, and the returned value is the
/// iterate that met the tolerance.
pub fn fixed_point<S, G>(start: f64, tol: f64, max_iter: usize, mut g: G) -> Result<(f64, S)>
where
    G: FnMut(f64) -> (f64, S),
{
    let mut x = start;
    for _ in 0..max_iter {
        let (next, state) = g(x);
        if (next - x).abs() < tol {
            return Ok((x, state));
        }
        x = next;
    }
    Err(EngineError::NotConverged(format!(
        "fixed point after {max_iter} steps, last iterate {x}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_point_finds_cosine_root() {
        let (x, _) = fixed_point(1.0, 1e-12, 1000, |x| (f64::cos(x), ())).unwrap();
        assert!((x - 0.739_085_133_215_160_6).abs() < 1e-10);
    }

    #[test]
    fn fixed_point_failure_returns_err() {
        let r = fixed_point(1.0, 1e-12, 5, |x| (x * 2.0, ()));
        assert!(matches!(r, Err(EngineError::NotConverged(_))));
    }
}
