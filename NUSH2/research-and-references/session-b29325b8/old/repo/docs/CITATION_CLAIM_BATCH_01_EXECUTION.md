# Citation-to-Claim Verification — Batch 01 Execution

## Purpose
Continue the incomplete citation-to-claim verification without attempting the entire manuscript in one run. This is an execution instruction, not a planning exercise.

Read:
- `docs/COMPLETE_CITATION_TO_CLAIM_FINAL_GATE.md`
- `docs/FULL_CITATION_TO_CLAIM_VERIFICATION_PROMPT.md`
- `results/ALL_REFERENCE_LINK_VERIFICATION.md`
- `paper/main.tex`
- `paper/references.bib`

The 56-reference link-testing phase is already complete. Do not repeat it.

## Batch 01 — Mandatory scope
Audit the active manuscript's:
1. `paper/sections/01_introduction.tex`
2. `paper/sections/02_background.tex`
3. `paper/sections/03_system_boundary.tex`

Include any nested LaTeX inputs within these sections, if present.

For EVERY citation occurrence:
- Identify the exact file, subsection and scientific statement.
- Open/read the original publication where accessible.
- Determine whether the cited source directly supports the statement, partially supports it, contradicts it or cannot be verified.
- Record the supporting page, figure, table or passage when actually established.
- Do not invent page numbers, source details or access results.
- Distinguish unavailable full text from a disproven claim.
- Correct confirmed citation errors or overstated manuscript claims, preserving citation keys and mathematical provenance.

## Required committed output
Create `results/citation_claim_batches/BATCH_01_INTRO_BACKGROUND_BOUNDARY.md` with a row for every citation occurrence, including manuscript location, claim, citation key, source evidence/location, verdict, correction and remaining uncertainty.

Record total occurrences examined and counts for supported, partially supported, unsupported and unverifiable claims. Explicitly identify any claims that require deeper technical investigation.

Commit the batch report and any verified corrections. Run appropriate LaTeX/citation checks for files changed. Report commit SHA, exact completed scope, counts and remaining issues, then STOP.

## Continuation rule
Do not proceed to later manuscript sections in this run. Subsequent batches should cover 2–3 sections each, committing results separately. Only after every active section and appendix has been audited should Main Research consolidate `results/FULL_CITATION_TO_CLAIM_AUDIT.md`, rebuild the final PDF, run both CI workflows and declare the citation gate complete.

Do not modify unrelated scientific models or repeat the completed link tests.
