# Main Research: Full Citation-to-Claim and Bibliography Verification

## Purpose
Perform a comprehensive, source-grounded citation audit of the FINAL ACTIVE MANUSCRIPT, not merely a URL-presence check. This is an execution instruction. Read the repository's latest STATUS.md and active paper/main.tex inclusion order before starting. Preserve ongoing PSA carbon-closure and full chemical/CCS cost research; do not overwrite its work.

## Existing audits to reconcile
- results/BIBLIOGRAPHY_LINK_AUDIT.md
- results/INDEPENDENT_REFERENCE_LINK_ACCESS_AUDIT_2026_10_13.md
- results/TARGETED_BIBLIOGRAPHY_REPAIR_2026_10_13.md
- results/BIBLIOGRAPHY_METADATA_RECONCILIATION_2026_10_13.md
- paper/references.bib

## Required verification
1. Inventory every BibTeX record and every citation in the active manuscript and nested inputs; flag missing keys, uncited references, duplicate publications, duplicate BibTeX fields, stale metadata, and unreferenced claims.
2. For every one of the 56 existing bibliography records, independently match the **actual publication**: title, authors/institution, publication year, report number, DOI and official publisher landing page or full-text URL. Distinguish valid, corrected, access-restricted, ambiguous, and incorrect links. A web fetch restriction does not prove a broken URL. Do not invent URLs.
3. Check citation-to-claim support at each in-text use, not just that the publication exists. Record the specific page, figure, table, equation or quoted source passage where available. Mark unsupported and only partially supported claims.
4. Prioritize all controlling numbers and scientific boundaries: INL TEV-953/961 process temperatures, 130 MMSCFD H2, 176.8 MWth heat, PSA recovery, 1927/142 short tons/day CO2 streams, PSA offgas fate and any 90% capture claim; JAEA/JAERI 600 MWth GTHTR300C, ~370 MWth process-heat branch versus ~170 MWth physical IHX; IEAGHG chemical plant and CCS costs; INL/RPT-23-72972 FOAK/BOAK/10-OAK cost assumptions; Singapore nuclear, Jurong and cross-border CCS context; TRISO, accident and EPZ sources.
5. Explicitly re-evaluate the workshop attribution for jaeri2004gthtr300c, the IAEA-TECDOC-1682 correction, the MTI 2025 speech title, and the Hamamoto JAEA record. Verify every corrected citation actually supports its surrounding claim.
6. Fix confirmed BibTeX metadata/URLs and misleading citations; if a source does not support a claim, correct the prose, narrow the claim, replace with a truly supporting source, or mark the evidence gap. Never silently swap publications, inflate certainty or fabricate page numbers.
7. Keep the existing 56-key citation inventory auditable; if merging/deleting entries is justified, document the key mapping and fix every use. Preserve historic source registers separately.

## Deliverables
- results/FULL_CITATION_TO_CLAIM_AUDIT.md: all active citation uses, claims, supporting source location, verdict and fixes.
- results/ALL_REFERENCE_LINK_VERIFICATION.md: one row per bibliography entry with exact metadata, link, access/identity verdict, and changes.
- Updated paper/references.bib and affected active manuscript passages, only where evidence justifies edits.
- Run BibTeX/LaTeX build, check undefined citations, reference formatting, DOI/URL clickability in rendered PDF, Rust/research and paper CI.
- Record final HEAD, files changed, unresolved references/claims, CI and PDF QA; commit changes and STOP.

Do not claim all references are verified if some remain access-restricted or scientifically unsupported. Do not start another independent review or broad editorial rewrite.
