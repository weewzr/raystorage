# Final report deduplication and de-assignment audit

## Before and after outline
**Before:** course-labelled title -> course thresholds in abstract/introduction -> proposed system -> detailed model/heat/hydrogen/carbon -> historical one-train economics before controlling two-train result -> course-labelled assessment -> deployment -> course-framed conclusion -> traceability.

**After:** independent research title -> industrial problem in abstract/introduction -> process and boundary -> thermodynamic/hydrogen/carbon models -> economics (controlling E2B annual ledger before historical screen) -> sensitivity -> integrated feasibility assessment -> deployment -> research conclusion -> technical reproducibility and originating-criteria appendix.

## Section-level disposition
| Active part | Edit/cut/move | Preserved |
|---|---|---|
| Title and author | Removed course identifier and course-submission subtitle | Nuclear-assisted SMR+CCS Singapore research identity |
| Abstract | Replaced assignment opening with industrial decarbonisation motivation | All E2B quantities, adverse FOAK and projected maturity qualifications |
| Introduction | Removed repeated course branding and changed research question to specified screening criteria | SMR chemistry, CCS limitations, HTGR motivation, Singapore context and citations |
| Proposed system | Retained user figures and process narrative | Original artwork and conceptual caveats |
| Design basis | Replaced workflow-specific W3 phrase; consolidated threshold origin here | >0.25 MtCO2e/y, <S$100/tCO2e, source branch vs IHX |
| Model formulation | Retained top-down calculation map | Literature-source assumptions and equations |
| Heat/hydrogen/carbon/energy | Retained derivations and source balances | MWth, tH2/y, captured vs avoided emissions, historical one-train distinctions |
| Economics | Kept newly added controlling ledger first; retained old one-train bridge explicitly as historical | Exact Rust E2B terms, FOAK/BOAK/10-OAK and S$3.725 provenance |
| Sensitivity | Retained historical one-train scope warning | Sensitivity figures and limitations |
| Integrated assessment | Renamed to scientific feasibility/performance assessment; changed table caption and originality language | Decision criteria and E2 comparator |
| Deployment | Retained decision-gated evidence logic | Site/safety/CCS/offtake and STOP/REDESIGN |
| Conclusions | Removed course identity; led with scientific verdict | FOAK failure, projected mature pass, unresolved deployment |
| Reproducibility | Retained build/test and model paths | Provenance and equations |
| Traceability appendix | Renamed to originating criteria traceability | Academic provenance, thresholds and section references |
| README | Demoted assignment framing; retained current results and two-minute guide | Reproduction and links to canonical files |

## Claims and units preserved
600 MWth reactor; ~370 MWth source process branch; 2 x 176.8 = 353.6 MWth chemical heat; ~195,892 tH2/y; ~1.085 MtCO2/y captured; ~1.834 MtCO2e/y lifecycle avoided; FOAK S$137.74/t FAIL; BOAK S$74.14/t projected; 10-OAK S$42.84/t projected. The old one-train S$3.725/t differential remains historical, not controlling. No electricity or co-product revenue, invented EPZ, approved Jurong site, or demonstrated nuclear/process integration is claimed.

## Deduplication limit
This pass removes repeated **course identity** and redundant framing, not all repeated numerical summaries: the abstract, assessment and conclusion each legitimately need a concise answer for different reading purposes. The long historical economic bridge remains in the body for traceability and can be moved later only with careful reference/figure repair. A complete word-count reduction is not yet verified; do not claim one without counting the compiled sources.

## Verification needed
Rust tests, paper build, citations/reference warnings, and final PDF page inspection must pass on the final commit. Earlier green CI does not certify this rewrite.
