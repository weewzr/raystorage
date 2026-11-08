//! Gates of the screening terms. A term leaves its anchored value only when the
//! rows of `results/Validation_Results.csv` that test its replacing model pass.
//!
//! Each gated term names its rows by test id. The V&V harness writes one
//! `GATE_<term>` row per term from those rows, so the record shows the outcome
//! beside the evidence. `RECORDED` holds the outcome of the last export for the
//! runs that cannot read the file (the browser and the fast CLI); the test
//! `flip_requires_gate` fails if it and the committed file disagree, and the
//! exporter refuses to write tables on a table that disagrees with the rows it
//! has just produced.
//!
//! Rules: a row that failed blocks the gate; a row that did not
//! run blocks it as not run, except the Khan rows that lack source data, which
//! the M6 column gate leaves out by design; rows printed with no limit do not
//! count. A gate with no rows at all, because no complete replacing model or no
//! benchmark exists, is not run.

use serde::Serialize;

/// Outcome of one gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GateState {
    Passed,
    Failed,
    NotRun,
}

impl GateState {
    pub fn label(self) -> &'static str {
        match self {
            GateState::Passed => "passed",
            GateState::Failed => "failed",
            GateState::NotRun => "not_run",
        }
    }

    /// Status word of the `GATE_` row.
    pub fn status(self) -> &'static str {
        match self {
            GateState::Passed => "pass",
            GateState::Failed => "fail",
            GateState::NotRun => "not_run",
        }
    }
}

/// Which rows test a term's replacing model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rows {
    /// Every row whose id starts with one of these, plus every exact id.
    Select {
        prefixes: &'static [&'static str],
        exact: &'static [&'static str],
    },
    /// No row exists; the reason goes into the `GATE_` row.
    None(&'static str),
}

/// A gated term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateDef {
    pub term: &'static str,
    /// Milestone of the gate, as the term table prints it.
    pub gate: &'static str,
    pub rows: Rows,
    pub what: &'static str,
}

const M6_PREFIXES: &[&str] = &[
    "SA-", "TAN-", "SO-", "ABS-", "DICE-", "KW-", "MO-", "VI-", "KH-",
];
const M6_WHAT: &str = "M6 column rows with a limit (the set nush2_units::bench::column_gate \
                       judges); Khan rows that lack Onda data are left out";
const M7_WHAT: &str = "FS_GATE_M7, the acceptance of the Level 3 flowsheet";

/// Gate of every term that has one, in term order. T11 and T12 have none.
pub const GATES: [GateDef; 11] = [
    GateDef {
        term: "T01",
        gate: "M4",
        rows: Rows::Select {
            prefixes: &["T961-3-"],
            exact: &["T961-5-T", "T961-5-CO", "T961-6-T", "T961-6-CO"],
        },
        what: "SMR-EQ raw syngas and the HTS and LTS outlets on the bed sized to the IEAGHG \
               approach, the basis gas_terms() computes phi on; the first-basis -GHSV rows \
               are a basis the term does not use",
    },
    GateDef {
        term: "T02",
        gate: "M6",
        rows: Rows::Select {
            prefixes: M6_PREFIXES,
            exact: &[],
        },
        what: M6_WHAT,
    },
    GateDef {
        term: "T03",
        gate: "M4",
        rows: Rows::None(
            "no complete replacing model: the reformer section closes (T961-4) but the \
             upstream preheat depends on Case 6 stream states absent from the library",
        ),
        what: "",
    },
    GateDef {
        term: "T04",
        gate: "M7",
        rows: Rows::Select {
            prefixes: &[],
            exact: &["FS_GATE_M7"],
        },
        what: M7_WHAT,
    },
    GateDef {
        term: "T05",
        gate: "M4",
        rows: Rows::Select {
            prefixes: &["T961-7-"],
            exact: &[],
        },
        what: "cooler_case5_duties, the six TEV-961 Case 5 cooler duties",
    },
    GateDef {
        term: "T06",
        gate: "M6",
        rows: Rows::Select {
            prefixes: M6_PREFIXES,
            exact: &[],
        },
        what: M6_WHAT,
    },
    GateDef {
        term: "T07",
        gate: "M6",
        rows: Rows::Select {
            prefixes: M6_PREFIXES,
            exact: &[],
        },
        what: M6_WHAT,
    },
    GateDef {
        term: "T08",
        gate: "M6",
        rows: Rows::Select {
            prefixes: M6_PREFIXES,
            exact: &[],
        },
        what: M6_WHAT,
    },
    GateDef {
        term: "T09",
        gate: "M7",
        rows: Rows::Select {
            prefixes: &[],
            exact: &["FS_GATE_M7"],
        },
        what: M7_WHAT,
    },
    GateDef {
        term: "T10",
        gate: "M8",
        rows: Rows::None(
            "no measured helium-side pressure drop for the finned reformer annulus or an IHX \
             tube bank in the library (card C.16); JAERI-Tech 2005-031 App. A-6 \
             prints one mock-up drop measured with low accuracy, and no comparison against \
             it was built",
        ),
        what: "",
    },
    GateDef {
        term: "T13",
        gate: "M7",
        rows: Rows::Select {
            prefixes: &[],
            exact: &["FS_GATE_M7"],
        },
        what: M7_WHAT,
    },
];

/// Gate of a term, if it has one.
pub fn def(term: &str) -> Option<&'static GateDef> {
    GATES.iter().find(|g| g.term == term)
}

impl GateDef {
    /// True when row `id` belongs to this gate.
    pub fn selects(&self, id: &str) -> bool {
        match self.rows {
            Rows::Select { prefixes, exact } => {
                exact.contains(&id) || prefixes.iter().any(|p| id.starts_with(p))
            }
            Rows::None(_) => false,
        }
    }

    /// Outcome from `(test_id, status)` pairs, with the ids that blocked it.
    pub fn evaluate<'a, I>(&self, rows: I) -> (GateState, Vec<String>, usize)
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut failed = Vec::new();
        let mut not_run = Vec::new();
        let mut judged = 0usize;
        if let Rows::None(_) = self.rows {
            return (GateState::NotRun, Vec::new(), 0);
        }
        for (id, status) in rows {
            if !self.selects(id) {
                continue;
            }
            match status {
                "pass" => judged += 1,
                "fail" => {
                    judged += 1;
                    failed.push(id.to_string());
                }
                "not_run" if id.starts_with("KH-") => {}
                "not_run" => not_run.push(id.to_string()),
                _ => {}
            }
        }
        if !failed.is_empty() {
            (GateState::Failed, failed, judged)
        } else if !not_run.is_empty() || judged == 0 {
            (GateState::NotRun, not_run, judged)
        } else {
            (GateState::Passed, Vec::new(), judged)
        }
    }
}

/// Outcome of every gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gates {
    pub states: Vec<(&'static str, GateState)>,
}

impl Gates {
    /// Outcome of a term's gate; `None` for a term with no gate.
    pub fn of(&self, term: &str) -> Option<GateState> {
        self.states
            .iter()
            .find(|(t, _)| *t == term)
            .map(|(_, s)| *s)
    }

    pub fn passed(&self, term: &str) -> bool {
        self.of(term) == Some(GateState::Passed)
    }

    /// Evaluate every gate on `(test_id, status)` pairs.
    pub fn from_rows<'a>(rows: &[(&'a str, &'a str)]) -> Self {
        Self {
            states: GATES
                .iter()
                .map(|g| (g.term, g.evaluate(rows.iter().copied()).0))
                .collect(),
        }
    }

    /// Read the outcomes from `GATE_<term>` rows.
    pub fn from_gate_rows<'a>(rows: &[(&'a str, &'a str)]) -> Self {
        let state = |term: &str| {
            let id = format!("GATE_{term}");
            match rows.iter().find(|(t, _)| *t == id).map(|(_, s)| *s) {
                Some("pass") => GateState::Passed,
                Some("fail") => GateState::Failed,
                _ => GateState::NotRun,
            }
        };
        Self {
            states: GATES.iter().map(|g| (g.term, state(g.term))).collect(),
        }
    }

    /// The outcome recorded at the last export.
    pub fn recorded() -> Self {
        Self {
            states: RECORDED.to_vec(),
        }
    }

    /// Every gate passed; used by tests of the flip path only.
    pub fn all_passed() -> Self {
        Self {
            states: GATES.iter().map(|g| (g.term, GateState::Passed)).collect(),
        }
    }
}

/// Outcome of each gate at the last export, from `results/Validation_Results.csv`.
pub const RECORDED: [(&str, GateState); 11] = [
    ("T01", GateState::Passed),
    ("T02", GateState::Failed),
    ("T03", GateState::NotRun),
    ("T04", GateState::Failed),
    ("T05", GateState::Passed),
    ("T06", GateState::Failed),
    ("T07", GateState::Failed),
    ("T08", GateState::Failed),
    ("T09", GateState::Failed),
    ("T10", GateState::NotRun),
    ("T13", GateState::Failed),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_row_blocks_and_a_reported_row_does_not_count() {
        let g = def("T05").unwrap();
        let rows = [
            ("T961-7-A", "pass"),
            ("T961-7-B", "reported"),
            ("T961-8-LIQ", "fail"),
        ];
        assert_eq!(g.evaluate(rows).0, GateState::Passed);
        let rows = [("T961-7-A", "pass"), ("T961-7-B", "fail")];
        let (s, why, _) = g.evaluate(rows);
        assert_eq!(s, GateState::Failed);
        assert_eq!(why, ["T961-7-B"]);
        assert_eq!(g.evaluate([("X", "pass")]).0, GateState::NotRun);
    }

    #[test]
    fn khan_rows_without_data_are_left_out_of_the_column_gate() {
        let g = def("T07").unwrap();
        let rows = [
            ("SA-1", "pass"),
            ("KH-1-T13", "not_run"),
            ("SO-R1-CAP", "reported"),
        ];
        assert_eq!(g.evaluate(rows).0, GateState::Passed);
        let rows = [("SA-1", "pass"), ("MO-K16-TOUT", "not_run")];
        assert_eq!(g.evaluate(rows).0, GateState::NotRun);
    }

    #[test]
    fn a_gate_without_rows_is_not_run() {
        assert_eq!(
            def("T10").unwrap().evaluate([("GATE_T10", "pass")]).0,
            GateState::NotRun
        );
        assert!(def("T11").is_none());
    }
}
