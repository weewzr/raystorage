# EXECUTE NOW — Link-by-link bibliography verification in bounded batches

## Purpose and scope
The user requires actual independent testing of every bibliography URL/DOI, not another generic audit plan. Execute this task to completion, in batches, and commit evidence. Do not substitute a green CI check, presence of a URL, or a search snippet for verification.

Read paper/references.bib and the existing reference audits. Work from the current HEAD. Do not modify the carbon/economics model or undertake unrelated manuscript rewrites.

## Batch execution (mandatory)
Divide all bibliography records into batches of no more than 10 citations, prioritizing INL, JAEA, IAEA and IEAGHG, then complete the remaining entries. For EACH record:
1. Extract citation key, supplied URL/DOI and bibliographic title, authors, year, report number.
2. Actually open/follow the link, including DOI redirects. Record whether a publisher page or PDF loads and its final URL; where possible capture HTTP status and redirect chain. Do not claim an HTTP status if only a browser/tool error is available.
3. Check that the opened page or document matches the cited publication's title, author/institution, year and report number. A publisher homepage or similar paper is NOT a verified match.
4. If a link fails, search for the exact title/report/DOI at its official publisher, Crossref, OSTI, JOPSS, IAEA, INL or relevant government archive; test the replacement link. Do not invent or silently substitute documents.
5. Assign precisely one status: VERIFIED_DIRECT, VERIFIED_DOI, CORRECTED_VERIFIED, BROKEN_CONFIRMED, ACCESS_RESTRICTED, IDENTITY_MISMATCH or UNRESOLVED. Distinguish a 403, blocked PDF, expired cache, DOI redirect failure and confirmed 404.
6. Record source URL, tested URL, final landing URL (if available), test date, method, observed outcome, publication-identity evidence, action and remaining limitation.

## Durable per-batch evidence
Commit a file `results/reference_link_batches/BATCH_01.md`, then BATCH_02, etc., containing a complete row for each reference in that batch. A batch is not complete until all its records have an actual attempted link test and documented evidence; never fill status from URL-field presence alone. Commit each batch before proceeding.

## Final consolidation
Create or update `results/ALL_REFERENCE_LINK_VERIFICATION.md` with every bibliography key exactly once, status counts and exact evidence links to batch files. Update `paper/references.bib` ONLY for independently verified repairs, preserving citation keys and avoiding duplicate fields. For records that remain restricted/unresolved, say so without claiming success.

After the 56-link audit is complete, continue citation-to-claim verification using `docs/FULL_CITATION_TO_CLAIM_VERIFICATION_PROMPT.md`; record it separately in `results/FULL_CITATION_TO_CLAIM_AUDIT.md` with exact claim, manuscript location, supporting publication page/table/figure or unresolved status. Do not block the link-verification deliverable on unavailable full text.

## Acceptance criteria
- Every bibliography entry has one documented actual link/DOI test.
- Every status has reproducible evidence, with no invented HTTP codes, page numbers or source identities.
- All known incorrect metadata/link matches are fixed or explicitly flagged.
- Both final audit files exist, all batches are committed, and the reference list builds without undefined citations.
- Run paper and research CI, inspect bibliography links in the rendered PDF, record final HEAD and unresolved cases.

Report completed batches, verified/broken/restricted/unresolved counts, exact files and final commit SHA, then STOP. Do not produce another planning document in lieu of execution.
