# Post-E8 Architecture Sizing Re-Optimization

## 1. Decision

**Decision: A — RETAIN TWO TRAINS as the best-supported architecture.**

The current architecture — one 600 MWth GTHTR300C-class source plus **2 x 130 MMSCFD / 176.8 MWth SMR-H2+CCS trains**, 353.6 MWth total — survives the broader sizing pass.

This is **not** a proof that two trains are a global mathematical optimum. It is the strongest **evidence-supported Pareto choice** in the design space currently available. Alternatives can be made heat-equivalent by linear arithmetic, but the repository does not contain defensible equipment, cost, reliability, footprint or safety scaling that would establish them as superior.

The E2B rule
[
N_{max}=leftlfloor rac{Q_{m source,process}}{Q_{m train}}ightfloor
=leftlfloorrac{370}{176.8}ightfloor=2
]
is therefore a **feasibility rule**, not an optimization theorem. Two trains remain preferred only after applying the wider evidence/multi-criterion screen below.

## 2. Benchmark and design space

Canonical source anchors:
- Case-6 process train: 130 MMSCFD H2, 176.8 MWth heat, 34 MMSCFD NG, ~97,946 tH2/y at 85% availability;
- GTHTR300C-class reactor: 600 MWth;
- rounded source process-heat branch: ~370 MWth;
- reference physical IHX: ~170 MWth;
- E2B two-train totals: 353.6 MWth, ~195,892 tH2/y, ~1.834 MtCO2e/y.

Candidates:

| Candidate | Heat structure | Evidence status | Quantitative status |
|---|---:|---|---|
| 1 x Case-6 benchmark | 1 x 176.8 = 176.8 MWth | source-anchored process train | canonical E2B economics |
| **2 x Case-6** | **2 x 176.8 = 353.6 MWth** | **source-anchored train + transparent identical replication** | **canonical E2B economics** |
| 3 x smaller sensitivity | 3 x 117.87 = 353.6 MWth | bounded topology sensitivity only | aggregate service linearized; equipment/economics unresolved |
| 1 x custom-large sensitivity | 1 x 353.6 MWth | bounded topology sensitivity only | aggregate service linearized; scale-up/economics unresolved |

Three or more **unchanged** Case-6 trains are physically excluded by the published branch: 3 x 176.8 = 530.4 MWth > 370 MWth.

## 3. Heat and utilization

For the retained two-train case:
[
f_{m branch}=rac{353.6}{370}=0.956,
qquad
f_{m reactor}=rac{353.6}{600}=0.589.
]

Thus two trains use ~95.6% of the rounded source process-heat branch and ~58.9% of total reactor thermal rating.

One unchanged train uses only 176.8/370 = 47.8% of the source process-heat branch and 29.5% of total reactor thermal rating. This under-utilization is the physical reason the one-train architecture performs poorly economically.

The 3-smaller and 1-custom sensitivities were deliberately fixed at the same 353.6 MWth aggregate duty. They therefore cannot beat the two-train case on aggregate heat utilization without additional evidence; their potential advantages would have to come from real equipment scaling, BOP, reliability, safety or footprint effects that are currently unquantified.

## 4. Throughput and lifecycle abatement

Under the bounded assumption of linear service scaling with identical thermochemical performance, any topology delivering the same 353.6 MWth at the same per-unit service intensity would reproduce:
- ~195,892 tH2/y;
- ~1.834 MtCO2e/y lifecycle abatement.

For the two-train case this is a transparent replication of the verified Case-6 basis and remains canonical.

For three smaller or one larger train these values are **sensitivities only**. The project does not have evidence that a 117.9 MWth smaller train or 353.6 MWth custom train preserves the same conversion, heat intensity, PSA recovery, capture performance, equipment efficiency or availability.

## 5. Economics

Canonical E2B two-train economics remain:
- FOAK: **S$137.74/tCO2e — FAIL**;
- early-commercial/BOAK: **S$74.14/tCO2e — projected/modelled PASS**;
- mature 10-OAK: **S$42.84/tCO2e — projected/modelled PASS**.

One-train results remain:
- FOAK: ~S$345.90/tCO2e;
- mature 10-OAK: ~S$156.11/tCO2e — FAIL.

No defensible full economics can be assigned to the smaller/custom alternatives because reformer, CCS, manifold, IHX/secondary-loop and BOP scale effects are not established.

### Bounded integration-cost sensitivity only

E2B conservatively represents one TECDOC-anchored IHX/secondary-loop burden per Case-6 process train. Holding **all other two-train service/economic terms unchanged**, changing only the represented loop count gives:

| Sensitivity | represented loops | mature S$/tCO2e | status |
|---|---:|---:|---|
| canonical two-train | 2 | 42.84 | canonical |
| custom-large if one loop were valid | 1 | 36.61 | bounded sensitivity, **not validated** |
| three-small if three loops were required | 3 | 49.07 | bounded sensitivity, **not validated** |

This +/- ~S$6.23/t range demonstrates that topology could matter economically. It does **not** prove a custom large train is cheaper: a 353.6 MWth single interface greatly exceeds the ~170 MWth reference physical-IHX duty and may require multiple exchangers/manifolds, while process equipment scale-up costs are absent. Conversely, smaller trains could share BOP or use smaller interfaces; assigning three full loop burdens is deliberately only an adverse sensitivity.

No architecture change is justified from these hypothetical credits/debits.

## 6. Reliability, modularity and maintainability

### One Case-6 train
A train outage removes the entire chemical production service. It has the fewest chemical train interfaces but leaves the nuclear source strongly under-utilized and fails mature economics.

### Two Case-6 trains
Two identical source-based trains provide the strongest current modularity argument. A single-train outage can **in principle** leave ~50% chemical capacity available, subject to reactor/secondary-loop turndown and CCS/common-BOP behavior. The project has no train-level reliability database, so no availability credit is added. Identical replication simplifies evidence transfer relative to custom scaling.

### Three smaller trains
Finer outage granularity could be attractive in principle, but no smaller-train reliability, turndown, cost, reformer/CCS design or shared-BOP evidence exists. More trains also add isolation/control branches, maintenance items and chemical interfaces. Reliability benefit is therefore **unresolved**, not scored as positive.

### One custom large train
It minimizes nominal train count but creates a single chemical-train outage point and requires unvalidated scale-up. Maintainability, fabrication, reformer geometry and CCS train scale are unresolved.

**Conclusion:** two trains offer a plausible balance between replication/modularity and interface count without relying on unsupported reliability improvements.

## 7. IHX and secondary-loop implications

The ~170 MWth published physical-IHX reference already makes each 176.8 MWth Case-6 interface a qualification issue (~4% above the reference duty). E2B does not claim that two 176.8 MWth duties are served by one 353.6 MWth physical exchanger.

- **2 x Case-6:** two source-scale process duties; interface topology/qualification still required but closest to the existing process evidence.
- **3 x smaller:** ~117.9 MWth per train is numerically below the 170 MWth reference, but this does not establish a qualified smaller chemical train or prove three independent IHXs are optimal.
- **1 x custom large:** a 353.6 MWth single-interface interpretation would greatly exceed the ~170 MWth reference; multiple exchangers/manifolds would likely have to be investigated, defeating any unsupported "one loop" simplification.

This criterion favors retaining the replicated source-scale architecture until real exchanger/manifold design exists.

## 8. Chemical hazard and nuclear-interface implications

Architecture affects chemical hazard topology but cannot yet be converted to risk numbers.

- More trains may reduce inventory per train if total inventory scales favorably, but increase leak points, isolation branches, congestion, controls and interface count.
- One larger train may reduce nominal interface count but concentrate inventory and single-event production loss.
- Two trains retain two source-scale inventories/interfaces already mapped by E4/E5.

No QRA, blast contour or separation distance is assigned. The candidate ranking therefore uses **evidence maturity and interface count qualitatively**, not fabricated safety scores.

## 9. Singapore/Jurong footprint, utilities and cooling

No candidate has a validated total nuclear/chemical site footprint.

- One train likely has the smallest chemical-plant footprint but fails source utilization/economics.
- Three smaller trains may require more equipment spacing, pipework and access; modular construction could help, but no footprint model exists.
- One custom large train could consolidate some equipment but may require larger vessels/reformer and separation; no scale law exists.
- Two trains increase chemical area relative to one train but remain the only high-utilization configuration with source-based train geometry/performance.

At equal aggregate service, 3-small and 1-large sensitivities were not given artificial utility/cooling advantages. Aggregate NG/H2/CO2 scales remain 68/260 MMSCFD/~1.085 Mt/y only under linear scaling. E3 cooling conclusions remain unchanged: actual cooling/UHS duties require a complete design and are not optimized here.

Jurong remains a conditional industrial context, not an approved nuclear site.

## 10. Alternative reactor/process-heat source

The repository does not contain a better independently evidenced high-temperature reactor/source option that simultaneously closes:
- ~900 C secondary-helium process heat;
- ~353.6 MWth useful process duty;
- a stronger commercial/component evidence basis;
- a directly comparable cost basis.

HTTR demonstrates temperature but is too small. Lower-temperature/water-cooled SMRs require a different heat-upgrading/electrification architecture. Other HTGR references are valuable technology evidence but do not establish a superior Singapore project architecture on the present data.

Therefore no alternative reactor is promoted in this bounded pass.

## 11. Cogeneration/residual-heat allocation

No new electricity or co-product credit is added. The published GTHTR300C source includes cogeneration concepts, but the project still lacks a validated off-design power-cycle model for the selected 353.6 MWth process allocation.

Cogeneration remains an explicit future sensitivity only.

## 12. EPZ and emergency-planning constraint

**Project EPZ: UNKNOWN / SITE-SPECIFIC.**

No candidate is ranked against a numerical radius. Required chain remains:

**accident sequence -> frequency -> radionuclide inventory/release -> mechanistic source term -> site dispersion -> dose -> protective action -> EPZ.**

Architecture can affect chemical propagation, interface count, physical footprint and emergency access, but there is no basis to infer that 1, 2 or 3 process trains produces a smaller Singapore EPZ. The reactor/core inventory is unchanged in the compared 600 MWth-source topologies unless a different reactor is selected.

HTR-PM research/implemented zones are not imported as the Singapore answer.

## 13. Data gaps that prevent a global optimum

| Variable | Why needed | Current status |
|---|---|---|
| reformer cost/efficiency vs train size | compare 118, 177 and 354 MWth trains | unavailable |
| CCS CAPEX/OPEX scale law at these train sizes | BOP/economics | unavailable |
| actual IHX/manifold architecture and cost | determine interface count/burden | unresolved |
| common vs duplicated BOP fractions | modularity economics | unresolved |
| train/component reliability and common-cause failures | outage/availability value | unavailable |
| turndown/load-following with partial train outage | value of modular trains | unresolved |
| inventory vs train size | QRA/consequence | unavailable |
| equipment/parcel footprint vs train size | Singapore siting | unavailable |
| construction/modularization productivity | BOAK/NOAK learning by topology | unavailable |
| selected-design PRA/source term/site met | EPZ/site safety | unavailable |
| alternative reactor comparable cost/process-heat data | source-size optimization | insufficient |

These missing variables are decision-relevant and are not replaced with fabricated scores.

## 14. Multi-criterion / Pareto conclusion

The evidence-supported candidate set is asymmetric:

1. **2 x Case-6 trains** — high process-branch utilization, passes projected BOAK/mature economic thresholds, preserves identical source-based process trains, offers plausible partial-outage modularity without taking credit for it, and has the deepest E3-E5 safety/infrastructure analysis.
2. **1 x Case-6 train** — strongest single-train evidence but dominated for this 600 MWth source by low utilization and mature cost failure.
3. **3 x smaller trains** — potentially attractive modularity/interface-duty sensitivity, but not an evidence-supported process design; neither economics nor reliability/footprint benefit can be demonstrated.
4. **1 x custom large train** — potentially reduces nominal train/BOP count, but requires unsupported scale-up and creates a severe IHX/interface evidence gap; apparent cost reduction from assuming one loop is not defensible.

Thus the two-train case is **not selected because floor(370/176.8)=2 alone**. It survives because it is the only high-utilization candidate whose process performance, throughput scaling and economics can be traced without inventing a new train technology.

## 15. Final recommendation

### A — RETAIN TWO TRAINS

Retain:
**1 x 600 MWth GTHTR300C-class source -> 2 x 130 MMSCFD / 176.8 MWth SMR-H2+CCS trains -> 353.6 MWth total process heat.**

Evidence status:
**BEST-SUPPORTED ARCHITECTURE / NOT A PROVEN GLOBAL OPTIMUM.**

No downstream E2B/E3/E4/E5/E6/E7/manuscript numerical recomputation is required because the benchmark architecture is retained.

The final conceptual architecture visual may continue to use two trains, but only after this bounded pass is closed and under the existing "project-proposed / conceptual / not to scale" qualification.

## 16. Reproducibility

Deterministic implementation:
- `model/src/post_e8_architecture_sizing.rs`
- `model/src/bin/post_e8_architecture_sizing.rs`

Machine-readable outputs:
- `results/post_e8_architecture_sizing/candidate_comparison.csv`
- `results/post_e8_architecture_sizing/integration_economics_sensitivity.csv`
- `results/post_e8_architecture_sizing/multicriterion_matrix.csv`

The model explicitly tests that the integer rule is a feasibility constraint and keeps unsupported topology economics labelled as bounded sensitivities.
