# Final manuscript structural cleanup — October 2026

## Decision and scope
The active report now leads with the preferred **one 600 MWth HTGR-class source plus two 130 MMSCFD SMR-H2+CCS trains**, requiring 353.6 MWth process heat. Historical one-train economics and sensitivity figures have been moved, **without deleting their equations, labels, citations, generated tables or numerical provenance**, to appendices compiled after reproducibility and originating-criteria traceability.

## Before -> after compilation outline
Before: physical basis -> hydrogen/carbon/energy -> controlling two-train ledger -> ~11,000 characters of superseded one-train cost derivations and cost bridge -> stage economics -> one-train availability/CCS sensitivity curves -> assessment/deployment/conclusions -> appendices A/B.

After: physical basis -> hydrogen/carbon/energy -> controlling two-train ledger and stage economics -> focused two-train uncertainty discussion -> assessment/deployment/conclusions -> appendices A/B -> **Appendix C historical one-train economic derivation** -> **Appendix D historical one-train sensitivity**.

## File-by-file change log
- `paper/sections/09_techno_economics.tex`: removed historical one-train subsection, long cost substitutions, source-price conversions, old cost ledger and bridge figure from the main narrative; replaced with a concise cross-reference to Appendix C. Kept two-train governing equation, exact represented annual cost ledger, 10-OAK worked substitution, missing-cost qualifications and FOAK/BOAK/10-OAK comparison.
- `paper/sections/C_historical_one_train_economics.tex`: preserved original historical section content verbatim after a labelled explanatory introduction. All original equation labels and citation keys remain compiled.
- `paper/sections/10_sensitivity.tex`: replaced historical figures and absolute one-train numerical thresholds with a current two-train uncertainty interpretation; explicitly explains why old curves cannot be extrapolated.
- `paper/sections/D_historical_one_train_sensitivity.tex`: preserved the original availability and CCS sensitivity text, both Rust-generated figures and figure labels, with historical boundary warning.
- `paper/main.tex`: compiles appendices C and D after A and B.

## Scientific invariants
No Rust model or generated numerical ledger changed. Current values remain: 353.6 MWth process heat; ~195,892 tH2/y; ~1.085 MtCO2/y captured; ~1.834 MtCO2e/y lifecycle avoided; FOAK S$137.74/tCO2e FAIL; BOAK S$74.14/tCO2e projected; 10-OAK S$42.84/tCO2e projected. The superseded one-train S$3.725/tCO2e remains traceable in Appendix C, not promoted as the current answer. The old one-train sensitivity values and figures remain in Appendix D.

## Figure readiness against docs/BALANCED_FIGURE_SPECIFICATION.md
The fifteen-figure specification is a **placement and source contract**, not permission to invent artwork. Figures 1-3 remain existing user images; Figure 3 must be replaced only with approved revised two-train artwork. The main economics section now has the controlling ledger and stage comparison near their intended deterministic figure insertion points; the sensitivity section has a concise uncertainty introduction rather than historical charts. Heat integration, hydrogen, carbon, energy and deployment sections retain their equation/paragraph anchors for future deterministic graphics and approved conceptual artwork. No figure placeholders, empty environments, fabricated risk contours or new external images have been inserted. Figure numbering is not frozen.

## Scientific and formatting verification
The old equations and labels are retained in compiled appendices so cross-references should remain resolvable. The final PDF and both GitHub Actions workflows must be checked at the last commit; do not infer a visual pass from LaTeX source or earlier CI. This audit is structural and does not certify missing nuclear/chemical site-specific safety analysis.
