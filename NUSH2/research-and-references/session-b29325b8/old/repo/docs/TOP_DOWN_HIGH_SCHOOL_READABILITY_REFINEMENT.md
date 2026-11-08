# Whole-report top-down teaching refinement - high-school-accessible, research-rigorous

## Objective
Perform a second complete manuscript refinement so a motivated high-school student can follow the argument, the physics, the equations and the conclusions without sacrificing technical accuracy. This is a real manuscript-editing task, not an outline or recommendations-only exercise. Coordinate with and incorporate the pending equation-first economics task in docs/ECONOMICS_EQUATION_FIRST_REWRITE.md; avoid concurrent conflicting edits.

## Global writing pattern
For EVERY active major section and substantive subsection use the sequence, adapted naturally:
1. **Why this matters** - one or two plain-English sentences giving the question.
2. **Big picture first** - explain the physical mechanism or whole-system identity before details.
3. **Overall governing equation** - where applicable, state the complete equation before component equations.
4. **Symbols and units** - define every variable when first used; explain unfamiliar terms with a concrete analogy only when scientifically accurate.
5. **Breakdown** - derive or justify each component from first principles, source-backed engineering correlations, or clearly identified modelling assumptions.
6. **Numbers** - show source inputs, numerical substitution, intermediate values and final result with correct units and rounding.
7. **Meaning** - interpret what the number means physically and for the Singapore decision.
8. **Limitations** - distinguish demonstrated technology, reference design, modelled/projected results, unknowns and regulatory gates.
9. **Transition** - explain why the next section follows.

Use short readable paragraphs and descriptive headings. Write with confidence and narrative energy, not marketing hyperbole. Define HTGR, SMR (steam methane reforming), CCS/CCUS, IHX, WGS, PSA, EPZ, FOAK/BOAK/NOAK, CAPEX/OPEX and lifecycle CO2e in context. Include a compact high-school-friendly 'How to read this paper' guide early in the manuscript and a compact notation/glossary aid; do not duplicate the entire nomenclature unnecessarily.

## Section-by-section priorities
- Abstract: problem -> intervention -> physical mechanism -> quantified outcomes -> qualifications.
- Introduction: established industrial SMR, its heat/emissions problem, why CCS alone has limits, why nuclear heat is proposed, Singapore relevance, precise question.
- Process flow: methane + steam -> reforming -> WGS -> CO2 capture -> PSA -> H2; distinguish feedstock carbon and fired-heating carbon.
- System boundary: compare equal H2 service; show included/excluded emissions and cost categories before numbers.
- Thermodynamics/heat integration: total energy balance first, then duties, temperature constraints, IHX and 600/370/353.6 MWth distinctions; clarify residual heat is not electricity.
- Hydrogen/material balance: whole-system mass/carbon balance first, then reactions, conversion, PSA recovery and throughput.
- CO2/lifecycle: E_baseline - E_candidate, then direct/upstream/transport/storage components; clarify capture fraction versus avoided lifecycle emissions.
- Economics: complete C_baseline and C_candidate ledgers first, component equations, annualization/learning, Delta C and S$/t, with FOAK/BOAK/10-OAK worked examples; preserve exact Rust ledger, no double counting.
- Uncertainty/sensitivity: identify parameter, mechanism, direction of effect, result and what it does not prove.
- Nuclear/chemical safety: barrier sequence and accident mechanism before PRA/QRA; EPZ as consequence-chain unknown, no invented radius.
- Singapore deployment: physical/logistical constraints, regulatory/CCS/cooling gates, decision-gated roadmap without fabricated years.
- Conclusion: plain-English answer first, then numbers and honest caveats.

## Mathematical and scientific safeguards
Do not dilute equations or remove rigorous derivations. Add readable explanations around them. For every active displayed equation verify source, algebra, units, variable definitions, numerical consistency and boundary. Do not invent missing first-principles closure. Flag assumptions and extrapolations clearly. Reconcile legacy one-train vs preferred two-train figures. Keep the distinction between modelled 10-OAK economic PASS and FOAK FAIL, and between research-level EPZ methodology and unknown Singapore EPZ. Preserve citations, figure references, labels and all canonical provenance.

## Deliverables
1. Actually edit all active manuscript sections that need improvement, including captions, headings, abstract and conclusion; do not just append explanations.
2. Update README's 'start here' and results narrative so a newcomer understands the project in two minutes.
3. Create results/TOP_DOWN_READABILITY_REFINEMENT.md with section-by-section before/after examples, a glossary coverage checklist, equations/units QA, remaining pedagogical difficulties and a high-school-reader walkthrough.
4. Rebuild PDF, run Rust and paper CI, check references and inspect all pages for equation wrapping, page flow, captions, tables and legibility.
5. Commit, report SHA, actual files changed, pass/fail checks and open scientific issues, then STOP.

## Acceptance test
A motivated reader with high-school chemistry, physics and algebra should be able to explain: (a) what the plant does; (b) why nuclear heat is used; (c) where CO2 comes from and goes; (d) why two trains; (e) how annual cost and avoided emissions are assembled; (f) why FOAK fails but projected mature case passes; (g) why Singapore site/EPZ is not proven. If not, refine further.
