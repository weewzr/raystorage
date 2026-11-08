# README and manuscript editorial changelog

## Before -> after
| File | Previous issue | Committed correction |
|---|---|---|
| `README.md` | Old one-train and S$3.725/t portrayed as final | Replaced with two-train current result, FOAK FAIL, projected BOAK/10-OAK, evidence maturity, navigation, reproducibility and historical section (8549d421) |
| `paper/sections/08_energy_performance.tex` | 423.2 MWth one-train residual presented as active final | Preferred 353.6 MWth process heat; conditional 246.4 MWth full-power residual explicitly not cooling/decay heat (63880af2) |
| `paper/sections/10_sensitivity.tex` | Historical one-train plots ambiguous | Added historical scope and no direct two-train transfer (8d421045) |
| `paper/sections/09_techno_economics.tex` | Historical differential bridge unqualified early; rounded ledger exact equality | Added historical subsection and warning; rounded 78.583 million/y marked approximate (40067afc) |
| `paper/sections/05_heat_integration.tex` | One-train 29.5%/423.2 MWth could read as current | Added explicit historical benchmark boundary; preferred 353.6/246.4 MWth pointer (a37e520e) |
| `results/FULL_MANUSCRIPT_EQUATION_AUDIT.md` | No grouped mathematical verification ledger | Added dimensional/numerical reconciliation and limitations |
| `results/FULL_MANUSCRIPT_EDITORIAL_AUDIT.md` | No compilation-order section review | Added active/nested section matrix |
| `results/EDITORIAL_OVERHAUL_AUDIT.md` | No repository-facing factual/edit audit | Added stale claims and safeguards |
| This file | No durable editorial changelog | Added traceable change history |

## Scientific invariants
No change to Rust source calculations or E2B/E3-E7 canonical scientific results. The two-train model still reports 353.6 MWth, ~195,892 tH2/y, ~1.834 MtCO2e/y, FOAK S$137.74/t FAIL, BOAK S$74.14/t projected PASS, 10-OAK S$42.84/t projected PASS. The old one-train values remain in historical sections and generated Rust ledgers, not deleted.

## Audit and verification protocol
The grouped equation ledger records the physical principle, dimensions, source/proxy status and numerical substitutions for the active manuscript's major displayed equations. It identifies a genuine **rounding presentation issue**, not a change in mature economics. Literature process states are not falsely presented as independently derived reactor or process simulation.

The build command is `sh paper/build.sh`; Research CI must pass `cargo test --all-targets`, and Paper/reproducibility CI must build and upload the canonical LaTeX-generated PDF. A green build verifies compilation/citations as configured; it does **not by itself prove page-by-page visual quality**. Record PDF page inspection separately once the artifact is accessible.

## Open technical work (not editorial)
Commercial IHX/manifold qualification, dynamic heat-rejection transients, project PRA/source term/EPZ, chemical QRA, Jurong parcel suitability, storage/offtake contracts and observed maturation economics. No independent review or final external artwork was started.

## PDF-specific repair
After both initial workflows were green, the exact GitHub Actions PDF artifact was downloaded and all 43 pages rendered. Page 29 showed the E7 arrow equation overflowing the right margin. The active `paper/sections/12_deployment_constraints.tex` was edited to replace the oversized mathematical display with a readable nine-stage numbered list; no scientific or roadmap gate changed. The manuscript must be rebuilt and the new artifact checked before declaring layout completion.
