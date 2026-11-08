# Editorial overhaul - scientific narrative, README and repository usability

## Role and objective
Perform a bounded, comprehensive editorial improvement of the **current manuscript and GitHub presentation**. The user wants prose that is clear, compelling, confident and memorable, while retaining strict scientific accuracy. Do not perform new core scientific research or alter model assumptions to improve the story.

## First verify repository state
Read STATUS.md, README.md, paper/main.tex, all paper/sections/*.tex, paper/REQUIREMENTS.md, results/E2B_ARCHITECTURE_MATURATION.md, results/POST_E8_ARCHITECTURE_SIZING_REOPTIMIZATION.md, current CI results, and the source/number provenance registers. Establish the current controlling result. The README currently foregrounds superseded one-train figures (97,946 t/y, 917,139 tCO2e/y, S$3.725/t) even though the E2B manuscript reports two trains, FOAK S$137.74/t, BOAK S$74.14/t, and mature 10-OAK S$42.84/t. Reconcile the README's current-facing claims with the controlling result; keep historical results clearly labeled and linked. Do not silently erase history.

## Writing direction
Open with the industrial problem and a memorable, evidence-grounded thesis. Suggested narrative:
1. Conventional steam methane reforming (SMR) is a commercially established, large-scale hydrogen-production route.
2. SMR is not itself a CO2-removal technology; adding CCS can capture important process emissions, but capture performance and lifecycle emissions depend on boundary, upstream methane and storage.
3. Nuclear-assisted SMR proposes replacing fossil-fired reformer heat with high-temperature nuclear process heat while retaining the familiar SMR+CCS chemical pathway.
4. Explain why this is attractive for Singapore but conditional on siting, licensing, IHX qualification, safety, cooling, CCS transport/storage and cost maturity.
5. Explain the scientific question, first-principles balances, evidence hierarchy, results and limitations.

Aim for elegant, vivid but restrained technical prose. Strong topic sentences, smooth paragraph transitions, precise verbs, low jargon density, meaningful figure/table lead-ins, and a coherent narrative from motivation to mechanism to evidence to conditional verdict. Use persuasive language only where supported. Avoid 'always', 'highest efficiency', 'carbon-free', 'proven nuclear integration', 'approved Jurong site', 'commercially viable today', or other unsourced superlatives. Do not confuse steam methane reforming (SMR) with small modular reactor (SMR). Expand acronyms on first use.

## Scope
- Thoroughly edit README.md and all active manuscript sections, captions, abstract, conclusion, limitations and future-work framing.
- Review top-level GitHub navigation, quick-start/build instructions, source links, reproducibility, current results, evidence maturity, visual gallery, status, historical archive and contribution/readme structure.
- Research a limited set of strong external GitHub README/scientific-software repository examples and scientific-writing conventions; extract transferable practices, not copied prose. Cite/reference any external technical claims properly.
- Correct stale cross-links, broken links and misleading current-facing figures.
- Preserve equations, citations, numerical provenance, LaTeX labels, scientific boundaries and deterministic output. No fabricated new claims.
- Check all major calculations in prose against current source files. Ensure FOAK failure and projected maturity are unmistakable.
- Avoid broad ornamental rewrite of technically correct passages just for length. Keep the final report readable and suitably concise.

## Execution and validation
Work on a safe branch if appropriate. Before editing diagnose and repair any existing failing CI independently of editorial changes, or explicitly isolate the failures and avoid conflating them with editorial QA. Build the paper, run model/tests, check citations/references, inspect generated PDF pages, and validate README links. Record before/after editorial changes with a short rationale.

## Durable outputs
- results/EDITORIAL_OVERHAUL_AUDIT.md : section-by-section audit, inaccurate/overstated/stale claims, concrete fixes, and remaining issues.
- results/README_AND_MANUSCRIPT_EDITORIAL_CHANGELOG.md : summary of actual edits, factual safeguards and build/test outcomes.
- Edited README.md and manuscript source sections where warranted.

## Final report
EDITORIAL OVERHAUL STATUS:
README UPDATED:
MANUSCRIPT SECTIONS UPDATED:
STALE CLAIMS CORRECTED:
SCIENTIFIC NUMBERS PRESERVED:
CI RESEARCH:
CI PAPER:
PDF VISUAL QA:
FILES:
COMMIT:
OPEN ISSUES:
Then STOP. Do not initiate another independent review automatically.
