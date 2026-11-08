# Economics equation-first rewrite - bounded task

## Goal
Rewrite the active manuscript economics exposition to be understandable from a complete cost identity before individual subcalculations. Do not alter scientific results without identifying and fixing an actual error.

## Mandatory approach
1. Read current active economics sections, canonical E1/E2/E2B Rust model and results, source registers, manuscript inclusion order and current editorial work.
2. Start with an annual-cost ledger and clear units (SGD/year) for both matched-service baseline and candidate. Use explicit symbols and a single nomenclature.
3. Introduce C_baseline as sum of represented baseline components (NG feed, fired heat, plant capital/O&M or other actual ledger terms); introduce C_candidate as sum of represented candidate components (NG feed, SMR plant terms, nuclear annualized CAPEX/O&M, IHX/secondary-loop, CCS, CO2 transport/storage and any other actual ledger terms). **Do not double count:** if SMR plant cost is included in NG/operating aggregate, identify it as such. The displayed decomposition MUST reproduce exact model terms, not a generic aspirational accounting list.
4. For each component give a compact equation, definition, source/assumption, units, substitution and numerical annual result, separately for per-train and two-train where relevant.
5. Explain capital recovery factor CRF and nuclear FOAK/BOAK/10-OAK learning and cost normalization transparently, without portraying projected maturity as observed.
6. Show Delta C = C_candidate - C_baseline and AC = Delta C / (E_baseline - E_candidate), with full units, scope, boundary, availability and a complete numerical worked example for the canonical mature two-train result (~S$616.814m/y, ~S$538.230m/y, ~S$78.583m/y, ~1.834278 MtCO2e/y, ~S$42.84/tCO2e). Explain rounding.
7. Include a clear summary ledger/table for FOAK, BOAK and 10-OAK and explain which terms change by maturity and which are held fixed.
8. Explicitly list excluded/unresolved costs (site, licensing, waste, liability, financing details, CCS contracting etc.) and explain the implication for threshold robustness.
9. Audit all cost figures and equations for internal consistency, source provenance, dimensions, duplicated charges, missing terms, and harmonized baseline/candidate H2 service. Correct actual mistakes in Rust/model and regenerate derived results only with tests and documented evidence.
10. Maintain readable academic prose and LaTeX typography, labels, references and figures.

## Deliverables
Edit the active LaTeX economics sections (not just a standalone note). Add results/ECONOMICS_EQUATION_FIRST_AUDIT.md documenting exact ledger-to-equation mapping, worked substitutions, numerical reconciliation, corrections and unresolved terms. Update README economic explanation if needed. Build PDF, run Rust tests, check references and layout, commit and STOP. Avoid conflicts with concurrent full editorial audit work.
