# Citation-to-Claim Verification — Batch 02: Model, Heat, Hydrogen

## Starting state
Batch 01 is committed at ffad0f45e391de8a2a6be9ecf99c38c00121407a with 73 citation-key occurrences (7 supported, 43 partial, 23 unverifiable, 0 unsupported) in Introduction, Background and System Boundary. Both CI workflows pass. Preserve Batch 01; do not repeat link testing or claim that partially supported sources are fully verified.

## Execute only these active sections
- paper/sections/04_model_formulation.tex (and any nested active inputs)
- paper/sections/05_heat_integration.tex (and any nested active inputs)
- paper/sections/06_hydrogen.tex (and any nested active inputs)

Read paper/main.tex, paper/references.bib and the prior audit. Enumerate EVERY citation-key occurrence (including figure captions, tables, equations and nested inputs) with exact file and line.

For each occurrence, record the precise claim, cited key, primary publication identity, specific page/table/figure/equation or source passage actually inspected, and verdict SUPPORTED / PARTIAL / UNSUPPORTED / UNVERIFIABLE. Do not infer support from a valid URL alone. If source full text cannot be accessed, say so. Correct only confirmed manuscript overstatements or citation errors; preserve numerical provenance and the distinction between source design basis and constructed demonstration.

Prioritize the 925/900/871 C temperature cascade, 176.8 MWth duty, helium mass-flow/cp basis, 600 MWth reactor rating, 370/371 MWth source process branch vs ~170 MWth physical IHX, 130 MMSCFD hydrogen service, 85% availability, and two-train 353.6 MWth derived sizing. Explicitly flag any source-claim attribution not verified.

## Deliverable
Create results/citation_claim_batches/BATCH_02_MODEL_HEAT_HYDROGEN.md with a complete per-occurrence table, counts by verdict, precise corrections and unresolved source evidence. Commit the report and verified corrections. Run relevant LaTeX/citation checks and report CI status/commit SHA, then STOP. Do not continue to carbon/economics sections in this run.

After all batches, consolidate the final FULL_CITATION_TO_CLAIM_AUDIT.md; not in this batch.
