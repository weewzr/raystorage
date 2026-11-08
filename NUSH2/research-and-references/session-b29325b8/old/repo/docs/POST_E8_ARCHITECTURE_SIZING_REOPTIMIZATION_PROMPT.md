# Post-E8 Architecture Sizing Re-Optimization Prompt

## Purpose

Re-open **only the architecture-sizing decision** before final conceptual visuals are locked.

The current E2B preferred case - one 600 MWth GTHTR300C-class source plus two identical 130 MMSCFD SMR-H2+CCS trains using 353.6 MWth - remains the **benchmark**, not a result to defend automatically.

This pass asks whether "two trains" is genuinely the best defensible architecture once a wider design space is considered, or merely the largest integer count of the existing 176.8 MWth Case-6 train that fits the published approximately 370 MWth process-heat branch.

Do not reopen unrelated closed work. Do not overwrite canonical E2B results unless this bounded re-optimization produces a clearly better, evidence-supported architecture.

## Mandatory starting point

Read at minimum:
- `STATUS.md`
- `MASTER_INSTRUCTIONS.md` if present
- `results/E2B_ARCHITECTURE_MATURATION.md`
- `model/src/e2b_maturation.rs`
- `results/E3_JURONG_SITING_COOLING.md`
- E4/E5 safety outputs
- `paper/sections/05_nuclear_feasibility_safety.tex`
- `literature/EPZ_SOURCE_TERM_EVIDENCE.md`
- `results/FINAL_VISUAL_SOURCE_PLAN.md`

Preserve the established distinction:
- existing E2B: constrained integer replication of a 130 MMSCFD / 176.8 MWth train;
- this pass: broader architecture sizing/re-optimization.

## Question to answer

Is the current **2 x 130 MMSCFD / 353.6 MWth** configuration still the preferred architecture when train count/size, reactor/process-heat allocation, utilization, economics, reliability/modularity and Singapore siting/safety implications are considered together?

## Design space

Evaluate only architectures that can be supported or transparently bounded by evidence. At minimum assess:

1. current benchmark: 1 x 176.8 MWth train;
2. current preferred: 2 x 176.8 MWth trains;
3. three or more smaller modular process trains where technically meaningful;
4. one larger/custom process train, but only if scale-up can be defensibly bounded;
5. alternative reactor/process-heat sizing or allocation if an independently evidenced high-temperature reactor/source option closes the required temperature and duty;
6. cogeneration/residual-heat allocation only as an explicit sensitivity unless actual off-design power-cycle evidence supports it.

Do not invent reactor products, reformer capacities, economies of scale, availability improvements or cost credits.

## Optimization criteria

Compare candidates on a multi-criterion basis, not S$/t alone:

- total useful process heat and source-branch utilization;
- H2 throughput and annual output;
- lifecycle CO2e abatement;
- represented FOAK, BOAK and mature economics where evidence permits;
- incremental S$/tCO2e;
- common versus duplicated BOP/integration burden;
- partial-outage behaviour and train-level availability/redundancy;
- maintainability/modularity;
- IHX/secondary-loop implications;
- nuclear/chemical interface count and isolation complexity;
- chemical inventory/congestion/fire/explosion implications;
- land/footprint implications for Singapore/Jurong;
- cooling and utility implications;
- licensing/safety-case complexity;
- evidence maturity and unresolved assumptions.

Where data are insufficient, report the criterion as unresolved rather than assigning a fabricated score.

## EPZ and safety constraint

Do **not** assign or optimize against a numerical Singapore EPZ radius.

The current project has no selected-design PRA, mechanistic source term, site meteorology, dispersion/dose model or regulator-approved protective-action criterion. Therefore the project EPZ remains **unknown and site-specific**.

Retain the required chain:

accident sequence -> frequency -> inventory/release -> source term -> dispersion -> dose -> protective action -> EPZ.

However, discuss qualitatively whether architecture choices could affect:
- radionuclide inventory/source-term pathways;
- number of nuclear/chemical interfaces;
- chemical hazard inventory and propagation;
- physical separation/footprint;
- emergency-planning feasibility.

Do not import HTR-PM research or implemented 3/7/30 km zones as the Singapore answer.

## Quantitative method

Extend the deterministic Rust model where possible.

Separate:
1. evidence-supported calculations;
2. bounded sensitivities;
3. unresolved design variables.

Do not force continuous optimization if the available technology evidence supports only discrete candidates.

For each candidate, show equations/derivations clearly enough that train count and sizing can be independently checked.

Explicitly test whether the current E2B rule

N_max = floor(Q_source_process / Q_train)

is merely a feasibility rule or remains a reasonable architecture-selection rule after the wider criteria are applied.

## Required outputs

Create:

`results/POST_E8_ARCHITECTURE_SIZING_REOPTIMIZATION.md`

and, where quantitative comparisons are possible:

`results/post_e8_architecture_sizing/`

with machine-readable CSV outputs and deterministic model changes/tests.

The report must include:
- benchmark definition;
- candidate architecture table;
- evidence basis for each candidate;
- quantitative heat/utilization comparison;
- economics comparison where defensible;
- reliability/modularity discussion;
- Singapore footprint/safety/EPZ implications;
- uncertainty/data-gap table;
- explicit ranking or Pareto conclusion;
- final architecture recommendation.

## Decision rule

Possible conclusions are:

A. **RETAIN TWO TRAINS** - if it remains the strongest evidence-supported architecture.

B. **CHANGE ARCHITECTURE** - only if another configuration is demonstrably superior with adequate evidence.

C. **NO UNIQUE OPTIMUM** - if evidence does not support a defensible optimum; identify the bounded candidate set and the specific data needed to decide.

Do not select two trains simply because E2B selected them.

Do not select another architecture merely to create novelty.

## Visual lock

Until this pass closes:
- do not finalize the two-train architecture artwork;
- do not finalize a numerical EPZ/site-radius graphic;
- do not integrate a changed architecture into the manuscript.

If the benchmark is retained, state why it survived the re-optimization.

If changed, identify every downstream result/figure/manuscript section that would require re-computation before any edit is made.

## Report and STOP

Report:

POST-E8 ARCHITECTURE SIZING RE-OPTIMIZATION:

CURRENT 2-TRAIN CASE RETAINED?: YES / NO / NO UNIQUE OPTIMUM

BEST-SUPPORTED ARCHITECTURE:

WHY:

ALTERNATIVES TESTED:

ECONOMIC EFFECT:

RELIABILITY/MODULARITY EFFECT:

SINGAPORE FOOTPRINT/SAFETY EFFECT:

EPZ STATUS: UNKNOWN / SITE-SPECIFIC (unless genuinely derived through the full required chain)

DOWNSTREAM RECALCULATION REQUIRED?: YES/NO

FILES CREATED/CHANGED:

TESTS:

COMMIT SHA:

Then STOP.

Do not begin another independent review.
Do not modify final visuals.
