# Citation-to-Claim Verification — Batch 04: Sensitivity, Assessment, Deployment, Safety and Singapore

## Current state
Batch 01, 02 and 03 are committed, covering 127 citation-key occurrences across introduction through economics. All 56 bibliography links were attempted in separate committed batches. Batch 03 found 15 partial and 5 unverifiable citation uses, and cautioned against calling partial-cost results verified full-project passes. Preserve all previous audits.

## Mandatory scope
Audit all citation occurrences in the active sections:
- paper/sections/10_sensitivity.tex
- paper/sections/11_cn4252_assessment.tex
- paper/sections/12_deployment_constraints.tex
- active nested inputs paper/sections/11_limitations.tex, 05_nuclear_feasibility_safety.tex, 10_singapore.tex
- paper/sections/13_conclusions.tex
Read paper/main.tex to verify nested inclusion order; do not skip citations in figures, tables or captions.

For each occurrence, document file:line, exact claim, key, primary source location/passage actually inspected, verdict SUPPORTED/PARTIAL/UNSUPPORTED/UNVERIFIABLE, and correction/residual risk. Do not invent page numbers or treat link accessibility as claim verification.

Prioritize:
- TRISO retention evidence and qualified limits, HTTR loss-of-forced-cooling evidence, IHX tritium and isolation claims;
- EPZ research versus implemented HTR-PM zones, source term, dose and protective action; no Singapore numerical EPZ;
- Singapore nuclear policy, licensing, Jurong siting, water/cooling, H2/NG/CCS infrastructure and cross-border storage status;
- maturity and sensitivity assumptions; no complete chemical plant CAPEX/OPEX or PSA carbon closure claim;
- correct any overly strong claim such as 'threshold compliance' where the economics remain partial-cost.

## Deliverable
Create results/citation_claim_batches/BATCH_04_SENSITIVITY_SAFETY_SINGAPORE.md with every occurrence, counts and outstanding evidence. Make source-supported prose/citation corrections only, preserving mathematical results and provenance. Run relevant paper checks, commit, report SHA and STOP.

Do not attempt the appendices or full consolidation in this run. Follow with a separate appendix batch and then final consolidated citation-to-claim audit.
