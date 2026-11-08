# Editorial overhaul audit

## Controlling result and provenance
The canonical proposed solution is **nuclear-assisted steam methane reforming + CCS** using one 600 MWth GTHTR300C-class source and two 130 MMSCFD Case-6 chemical trains. Preferred process heat = 353.6 MWth; hydrogen = ~195,892 t/y; E2B lifecycle avoidance = ~1.834 MtCO2e/y. Two-train FOAK = S$137.74/t (FAIL); BOAK = S$74.14/t (projected); 10-OAK = S$42.84/t (projected). Current deployment/licensing feasibility is not demonstrated. These values supersede the historical one-train S$3.725/t differential screen as current-facing economics.

## Repository-facing issues and fixes
1. **README stale result — fixed at 8549d421.** Previous README presented one train (97,946 t/y), 917,139 tCO2e/y and S$3.725/t as the controlling final result, while the manuscript and E2B had moved to two trains and adverse FOAK economics. The rewritten README leads with the industrial mechanism and honest conditional verdict, and preserves the old numbers in a labelled historical section.
2. **Navigation — fixed in README.** Direct links to status, canonical LaTeX, E2B, architecture reoptimization, E3-E7 evidence, Rust code and visual-source plan. Build instructions point to `sh paper/build.sh`. Source PDF is not maintained independently of LaTeX.
3. **One-train thermal allocation in active text — fixed.** Section 08 now uses the preferred 353.6 MWth process duty; Section 05 identifies 423.2 MWth and 29.5% as the historical one-train benchmark.
4. **Sensitivity figures — fixed.** Section 10 explicitly labels generated curves as historical one-train sensitivities, not forecasts for the two-train proposal.
5. **Economic framing — fixed.** Section 09 labels the historical differential as superseded and presents E1/E2B controlling economics. The rounded mature ledger equality was changed to approximate, because 616.814−538.230=78.584 while unrounded underlying values yield ~78.583.
6. **Scientific overstatement safeguards — retained.** No approved Jurong nuclear parcel, commercial 353.6 MWth integrated facility, site EPZ, full-power cooling duty, guaranteed tritium isolation, validated QRA/PRA, bankable costs or electricity revenue is claimed.

## Active-section editorial coverage
- Abstract/nomenclature: strong controlling summary and terminology.
- Introduction/background: industrial motivation, chemical mechanism, nuclear process-heat intervention.
- Design/model: literature evidence vs project duplication and screening assumptions.
- Heat/hydrogen/carbon/energy: dimensional physical narrative, historical-vs-preferred scope corrected.
- Economics/sensitivity: adverse FOAK, projected mature economics, legacy plots demoted.
- Assessment/deployment/conclusion: current failure and conditional future feasibility; falsifiable decision gates.
- Reproducibility/traceability: canonical files and assignment mapping.
- Nested lifecycle/economic formulation, safety, limitations and Singapore sections: preserve explicit evidence boundaries.

## Editorial conventions adopted
Prefer a clear research question, early controlling results, a reproducibility entry point and explicit limitations. Avoid unsupported superlatives (e.g., 'highest CO2 removal efficiency' or 'commercially proven nuclear integration'). Expand steam methane reforming on first mention; do not confuse it with small modular reactors.

## Remaining issues
Scientific: validated reactor/process transients, IHX qualification, site/EPZ, QRA/PRA/source term, CO2 transport/storage, off-take, availability and actual learning remain unresolved. Presentation: exact PDF artifact must be inspected page-by-page after final build, with separate CI confirmation. No external visuals or independent review were initiated in this editorial pass.

## PDF visual finding (post-build)
The LaTeX-generated 43-page workflow PDF was downloaded from GitHub Actions and all pages were rendered into four contact sheets. Page 29 had a confirmed oversized displayed roadmap that clipped at the right margin. The roadmap was replaced with a numbered nine-stage list in the active deployment section; this correction is subject to final rebuilt-PDF verification. Other visible pages had no similarly obvious page-edge clipping in the contact-sheet review; isolated figure/table pages retain some whitespace.
