# Comprehensive manuscript polish and equation verification - execution addendum

Execute together with docs/EDITORIAL_OVERHAUL_PROMPT.md. This is an actual editing/verification assignment, not merely a plan.

## Scope
Read paper/main.tex and every ACTIVE included section in compilation order. Distinguish dormant/historical .tex files from active sections. Polish the entire active report for narrative cohesion, clarity, concise but vivid technical explanation, terminology, figure/table captions, equation introductions, transitions, abstract and conclusions. Improve README accuracy/navigation in the same pass.

## Mathematical verification
Inventory every substantive displayed equation and every key numerical result. For each:
- derive or trace from governing principle, source design basis, or declared empirical model;
- check variable definitions, dimensions/units, sign conventions, reference state and time basis;
- verify algebra, conversions, substitutions, rounding and reported values against deterministic Rust/CSV outputs;
- check mass, carbon and energy balance consistency and no double counting;
- audit H2 production, reformer heat, source 370/371 MWth branch vs physical IHX rating, reactor residual heat, CO2 capture/storage, lifecycle avoided emissions, availability, cost numerator/denominator, FX/escalation, FOAK/BOAK/10-OAK learning and uncertainty;
- distinguish process capture rate vs lifecycle abatement, and capacity vs duty vs electricity;
- verify two-train sizing is evidence-supported, not a proven global optimum;
- check EPZ remains unknown/site-specific; no imported radius;
- preserve source-backed citations and exact technical caveats.

Do not invent missing first-principles closure. Mark empirical inputs, model approximations and unsupported extrapolations explicitly. If an actual error is found, correct source/model/tests and regenerate outputs with traceable before/after changes; do not cosmetically alter a result or overwrite historical records.

## Output and QA
Create results/FULL_MANUSCRIPT_EQUATION_AUDIT.md with equation-by-equation or grouped-by-section ledger: file/label, equation, source, dimensional check, numerical check, verdict, fix/limitation.
Create results/FULL_MANUSCRIPT_EDITORIAL_AUDIT.md with section-by-section changes and examples of substantive improvements.
Apply edits to active paper sections and README, not only the audit documents.
Build paper PDF, run Rust tests/CI, check citations/references, inspect all pages for layout and equation wrapping. Record precise pass/fail and open issues.
Use a safe branch/PR if appropriate. Avoid concurrent overwrites of Main Research's work.
Report files changed, test outcomes, manuscript artifact, commit SHA; then STOP. Do not initiate independent review without controller decision.
