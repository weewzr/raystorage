# Citation-to-Claim Verification — Batch 03: Carbon, Energy and Economics

## Starting state
Batch 01: 73 occurrences; Batch 02: 34 occurrences, both committed. The bibliography's 56 link tests are complete. Preserve prior batch records and do not repeat generic URL checks. Research and paper CI pass at 67e639fde3f8aa1f9138089f99308c49aa1746f1.

## Mandatory scope
Audit every citation-key occurrence, including nested active inputs, in:
- paper/sections/07_carbon_ccs.tex
- paper/sections/07_lifecycle_formulation.tex (if nested)
- paper/sections/08_energy_performance.tex
- paper/sections/09_techno_economics.tex
- paper/sections/09_economic_formulation.tex (if nested)

For each occurrence record file:line, exact claim, citation key, publication, verified supporting page/table/figure or access limitation, verdict SUPPORTED/PARTIAL/UNSUPPORTED/UNVERIFIABLE, action and residual risk. Do not infer exact page numbers from snippets or invent data. If original PDF accessible, inspect relevant original source tables rather than merely describing the report generally.

Prioritize:
1. INL TEV-953/961 1927 captured, 142 emitted and 3205 baseline short ton/day CO2; whether these are complete plant-carbon flows, and PSA tail-gas recycle/combustion boundaries. Distinguish 90% absorber capture from overall carbon/lifecycle abatement.
2. 34 vs 52.5 MMSCFD natural gas, 17.3 MWe auxiliary power, source-case assumptions, upstream NG and nuclear lifecycle proxies.
3. IEAGHG 2017-02 cost/capture process boundaries, CAPEX/OPEX, CCS equipment, CO2 conditioning and transport/storage, scaling assumptions.
4. INL/RPT-23-72972 cost normalization, 6000 USD/kWe, 25 USD/MWh, 9% energy-conversion share, 40% thermal conversion, 1.6 FOAK premium, 10% learning rate, and project application.
5. Explicitly distinguish historical partial-cost screening results from complete project costs. Do not change model numbers without substantiated source correction.

## Deliverable
Create results/citation_claim_batches/BATCH_03_CARBON_ENERGY_ECONOMICS.md containing EVERY citation occurrence and evidence/verdict, totals and source gaps. Make only source-justified citation/prose corrections, not an unrelated model rebuild. Run appropriate LaTeX/citation and CI checks, commit and STOP. Do not continue into sensitivity, safety or deployment sections in this run.
