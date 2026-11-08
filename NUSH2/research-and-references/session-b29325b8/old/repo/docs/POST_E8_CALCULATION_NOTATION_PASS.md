# Post-E8 Calculation and Notation Compliance Pass

## Authority

E8 manuscript reintegration is CLOSED at `49fcadc80161` with both CI workflows passing.

After E8 closure, the coordinator added:
`docs/E8_CALCULATION_NOTATION_ADDENDUM.md`
at commit `ed1be0453e01`.

Because that addendum was committed after the E8 closure commit, its requirements must now be explicitly applied and verified before the final visual pass or independent review.

This is a **bounded manuscript-quality pass**.

Do not restart E8.
Do not redo E1–E7.
Do not change canonical scientific results unless a genuine contradiction is found.
Do not begin external visual generation.
Do not begin independent review.

## Task

Read and execute:
`docs/E8_CALCULATION_NOTATION_ADDENDUM.md`

against the CURRENT manuscript.

The purpose is to ensure every material calculation in the final report is legible in the pattern:

**physical question
→ governing equation
→ symbol/unit definitions
→ cited/justified inputs
→ numerical substitution
→ result
→ concise engineering interpretation.**

## Priority checks

### 1. Per-train to preferred two-train derivation

Make the transition explicit in the manuscript:
- 176.8 MWth/train → 353.6 MWth total;
- 97,946 tH2/y/train → ~195,892 tH2/y;
- 34 MMSCFD/train → 68 MMSCFD;
- per-train captured CO2 → ~1.085 MtCO2/y;
- E2B lifecycle scaling → ~1.834 MtCO2e/y.

Explain which quantities are source/per-train values and which are project-derived two-train values.

Do not globally replace valid per-train/source numbers.

### 2. Reactor utilization

Explicitly and correctly distinguish:
- 353.6/370 ≈ 95.6% of rounded source process-heat branch;
- 353.6/600 ≈ 58.9% of total reactor thermal rating.

Do not conflate the two.

### 3. Mature economic derivation

The report must show how the preferred mature result is obtained rather than only state S$42.84/tCO2e.

At minimum show:
- deployment-stage/maturation basis;
- normalized/annualized nuclear burden;
- NG;
- CCS;
- T&S;
- integration;
- annual baseline represented cost;
- annual candidate represented cost;
- net incremental cost;
- avoided lifecycle denominator;
- final S$/tCO2e.

Use the canonical E2B values and consistent units.

A clear final relation is:

[
C_{\rm abat}
=
\frac{C_{\rm cand,annual}-C_{\rm base,annual}}
{A_{\rm lifecycle}}.
]

For the mature case, show the actual substitution using canonical values and explain immediately that this is a projected/modelled 10-OAK case, not an observed quotation.

### 4. FOAK → BOAK → 10-OAK method

Explain the INL maturation/learning method with good notation.

Where applicable show:
[
C_N=C_1(1-LR)^{\log_2 N}.
]

Define every symbol and cite the source.

Make clear that the future result was forward-calculated from an independently selected source method, not back-solved from S$100/t.

### 5. CO2/lifecycle derivation

Ensure the final preferred architecture has a compact readable lifecycle ledger.

Preserve the one-train detailed equations where useful, but bridge them explicitly to two trains.

Explain:
- direct vs lifecycle;
- saved vs added burdens;
- no double counting of captured CO2;
- E2 comparator boundary vs E2B preferred architecture boundary.

### 6. Cooling/site calculations

Preserve:
[
\dot m=\frac{\dot Q}{c_p\Delta T}.
]

Define inputs and units.

State explicitly that the 246.4 MWth / 600 MWth cooling screens and 7.53 / 18.34 m3/s values are illustrative envelopes, not a designed nuclear cooling duty.

### 7. Safety notation

Do not add fake numbers.

Retain parameterised equations only where they clarify E5 analysis requirements.

Explain missing inputs in prose.

### 8. Local provenance

Every externally sourced material number should have a local citation.

Project-derived values should identify their cited/source inputs.

Project assumptions must look like assumptions, not literature facts.

### 9. Engineering prose

After important displayed equations, add one concise sentence explaining:
- what the result means;
- why it matters;
- what it does not prove where relevant.

Do not simply repeat the equation.

### 10. Units and notation

Audit:
- MWth vs MWe;
- tCO2/y vs tCO2e/y;
- tH2/y;
- MMSCFD;
- S$/y;
- S$/tCO2e;
- kg/s;
- temperatures and temperature differences.

Use consistent symbols for the same quantity.

Avoid excessive precision in submission-facing prose.

## Verification

After changes:

1. build canonical LaTeX;
2. run all relevant tests;
3. run Research CI;
4. run Paper/reproducibility CI;
5. require both PASS;
6. verify zero undefined citations;
7. verify zero undefined references;
8. inspect exact generated PDF;
9. inspect equation wrapping and readability;
10. inspect economic derivation;
11. inspect one-train→two-train transition;
12. verify no canonical E1–E7 number changed.

## Report and STOP

Report:

POST-E8 CALCULATION/NOTATION PASS:

FILES CHANGED:

PER-TRAIN → TWO-TRAIN DERIVATION EXPLICIT: YES/NO

353.6/370 AND 353.6/600 DISTINCTION EXPLICIT: YES/NO

MATURE S$42.84/t DERIVATION SHOWN: YES/NO

FOAK/BOAK/10-OAK METHOD EXPLAINED: YES/NO

LIFECYCLE TWO-TRAIN DERIVATION CLEAR: YES/NO

COOLING CALCULATIONS CORRECTLY QUALIFIED: YES/NO

SAFETY NUMBERS NOT INVENTED: YES/NO

HEADLINE NUMBERS HAVE LOCAL PROVENANCE: YES/NO

DISPLAYED EQUATIONS HAVE ENGINEERING INTERPRETATION: YES/NO

UNIT/SYMBOL CONSISTENCY: PASS/FAIL

LATEX BUILD:

RESEARCH CI:

PAPER CI:

PDF INSPECTION:

CANONICAL SCIENTIFIC NUMBERS CHANGED?: YES/NO

SCIENTIFIC CONTRADICTIONS FOUND:

COMMIT SHA:

Then STOP.

Do not begin the final visual pass.
Do not begin independent review.
