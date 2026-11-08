# Finish citation-to-claim verification after completed 56-link test

## Current verified state
At HEAD 5b033364fdc3432a9517302926a42203eff02180, all 56 bibliography links have been attempted and documented in six committed batches, consolidated in results/ALL_REFERENCE_LINK_VERIFICATION.md: 28 VERIFIED_DIRECT, 1 CORRECTED_VERIFIED, 5 ACCESS_RESTRICTED, 22 UNRESOLVED. Research and paper CI pass. The missing deliverable is results/FULL_CITATION_TO_CLAIM_AUDIT.md.

## Execution task — not another plan
Read docs/FULL_CITATION_TO_CLAIM_VERIFICATION_PROMPT.md, paper/main.tex and all actively included LaTeX sections (including nested inputs). Enumerate every actual citation occurrence with file/section and the exact technical statement supported. For each:
1. Read the original cited publication where accessible, verifying exact title/author/year and relevant passage/table/figure/equation/page.
2. Record whether the source fully supports, partially supports, contradicts or cannot be checked against the statement. Do not invent page numbers.
3. For restricted/unresolved references, use alternate official publisher records or clearly mark claim verification incomplete; a valid link alone is not scientific support.
4. Correct any unsupported or overstrong manuscript claims and proven citation metadata errors, retaining citation keys when possible.
5. Prioritize INL TEV-953/961 PSA offgas and CO2 capture, 600/370/170 MWth JAEA design distinctions, IEAGHG CCS/chemical costs, INL advanced reactor cost learning, TRISO/EPZ, Singapore siting and CCS.
6. Preserve historical provenance and the unresolved carbon/economic gates; do not silently promote screening estimates to full verified results.

## Required durable output
Create results/FULL_CITATION_TO_CLAIM_AUDIT.md with a claim-level table containing manuscript location, claim, citation key, exact source support/location or access limitation, verdict, correction and residual risk. Include counts of fully supported/partial/unsupported/unverifiable uses. Link to the completed six-batch results rather than redoing all link attempts.

Build PDF, check bibliography formatting and undefined citations, run both CI workflows, commit verified corrections and audit, report SHA and remaining claim-level issues, then STOP. Do not claim completion if citation occurrences are omitted.
