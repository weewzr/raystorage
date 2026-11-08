# Deep-Feasibility V&V / Uncertainty / Extrapolation Matrix

## Definitions

- **Verification:** was the equation/code implemented and solved as intended?
- **Validation:** does the model reproduce independent physical/experimental/operating evidence for the intended use?
- **Uncertainty:** how uncertain are inputs/model form and how is that represented?
- **Extrapolation:** how far is the project applying evidence outside the conditions where it was demonstrated/validated?

A passing Rust test is verification evidence. It is **not automatically physical validation**.

| Submodel / claim | Verification | Validation | Uncertainty treatment | Extrapolation | Maturity / allowed claim |
|---|---|---|---|---|---|
| SMR reaction stoichiometry / elemental balances | automated conservation/equation tests | chemistry is established; project source values independently cross-checked | source operating-point uncertainty not probabilistically propagated | low for stoichiometry | VERIFIED / PHYSICALLY ESTABLISHED |
| Equilibrium / Shomate thermochemistry support | equation/unit/limiting tests in repository | compared against source/known thermochemical behaviour where implemented | correlations have range/source limits | moderate if used outside fitted T range | VERIFIED; VALIDATION BOUNDED BY CORRELATION RANGE |
| INL final process operating point | direct source transcription + consistency checks | INL TEV-953/961 process analysis is external model evidence, not project experiment | source model uncertainty not fully quantified | source case → Singapore deployment | SOURCE-MODEL ANCHORED |
| H2 annualisation | deterministic arithmetic + regression tests | no physical validation needed beyond source production rate | availability sensitivity added | annual availability extrapolation | VERIFIED SCREENING CALCULATION |
| 176.8 MWth process heat | source transcription + model reconciliation | INL process model; no 176.8 MWth nuclear-SMR demonstration | source/model uncertainty qualitative | large scale from experiments | SOURCE-MODEL SUPPORTED; NOT DEMONSTRATED |
| 925→900→871 C temperature cascade | deterministic source-value checks | HTTR demonstrates 950 C reactor operation; German/JAEA high-T component/reforming precedents | local heat-transfer/pinch/lifetime uncertainty unresolved | component geometry/scale | TEMPERATURE FEASIBILITY SUPPORTED; COMPONENT QUALIFICATION REQUIRED |
| Helium mass-flow screen | equation/test; source value 78.49 kg/s reproduced approximately | source INL process model | Cp approximation/model-form uncertainty | no project piping geometry | SCREENING ONLY |
| Helium pressure drop/circulator power | not implemented for project | none | unresolved | high | NOT MODELLED |
| Reactor thermal rating 600 MWth | source architecture identity | GTHTR300C design study, not operating plant | design-study uncertainty | 30 MWth HTTR → 600 MWth design | DESIGNED, NOT VALIDATED BY OPERATION |
| IHX capacity | capacity arithmetic verified | 10 MW HTTR IHX hardware; 170 MW GTHTR300C conceptual design | scale/material/lifetime uncertainty high | 17.68x demonstrated IHX duty | COMPONENT QUALIFICATION REQUIRED |
| Direct CO2 ledger | deterministic ledger reconciliation tests | source INL direct/captured stream data | source/model uncertainty | same operating point | VERIFIED SOURCE-BASED SCREEN |
| Lifecycle emissions | deterministic ledger reconciliation | external emission factors, not project LCA | proxy factors and boundary uncertainty | Singapore gas/nuclear/CCS chains | VERIFIED CALCULATION; LCA VALIDATION LIMITED |
| Nuclear heat LCA allocation | arithmetic verified | no project-specific process-heat LCA | high proxy/model-form uncertainty | electricity/reactor LCA → process heat allocation | SCREENING PROXY |
| Upstream natural-gas LCA | arithmetic verified | generic factor | Singapore supply-chain uncertainty | generic → Singapore imports | SCREENING PROXY |
| CCS T&S lifecycle factor | arithmetic verified | generic screening factor | route/storage-specific uncertainty | generic → future cross-border chain | SCREENING PROXY |
| Economic source-product conversion | deterministic currency/escalation/ledger tests | Nishihara design-study economics | escalation/FX/source design uncertainty | mature design study → Singapore FOAK | SCREENING ECONOMICS ONLY |
| CCS CAPEX scaling | deterministic | IEAGHG reference case | scale exponent/boundary/project uncertainty | reference plant → project | SCREENING |
| S$15/t T&S | deterministic | no contracted Singapore service | very high | assumption → future market | ASSUMPTION, NOT VALIDATED COST |
| Zero project electricity revenue | explicit boundary/test | conservative accounting choice | no project power-cycle model | none | VERIFIED BOUNDARY, NOT PHYSICAL OUTPUT |
| 85% availability base | regression-tested | design-study assumption, not operating GTHTR300C fleet | sensitivity implemented | design availability → project reliability | ASSUMPTION / SCREENING |
| No-backup availability sensitivity | automated base reconciliation + monotonic test | no physical reliability validation | availability varied parametrically | availability fraction stands in for complex outage process | VERIFIED SENSITIVITY; NOT RELIABILITY MODEL |
| Gas-backup sensitivity | automated base/monotonic tests | architecture supported by Herd/Schroders; heater efficiency external | backup CAPEX/O&M/start-up/integration omitted | generic backup → project | LOWER-BOUND OPERATING SENSITIVITY |
| CCS capture sensitivity | automated base/monotonic tests | IEAGHG shows capture configurations but project turndown not validated | capture energy/cost turndown omitted | steady fraction → storage outage | VERIFIED SCREEN; NOT DYNAMIC CAPTURE MODEL |
| TRISO retention | project does not simulate | AGR/HTTR external experiments | nuclide/T/irradiation/failure dependence | test particles → project source term | EXTERNAL VALIDATION EVIDENCE ONLY |
| Core/graphite fission-product retention | not implemented | external literature | high/design-specific | unresolved | NOT MODELLED |
| Primary-circuit plate-out/dust | not implemented | AVR external operating evidence | high/design-specific | AVR pebble bed → prismatic project | MECHANISM SUPPORTED; QUANTITATIVE TRANSFER NOT VALID |
| Mechanistic source term | not implemented | HTR-PM/Petti methods external | unresolved | project-specific | NOT MODELLED |
| Accident frequencies / PRA | not implemented | none | unresolved | project-specific | NOT MODELLED |
| Atmospheric dispersion/dose | not implemented | none | meteorology/population/regulator dependent | site-specific | NOT MODELLED |
| EPZ radius | not implemented | HTR-PM methodology/example only | site/regulator dependent | cannot transfer radius | NO PROJECT EPZ CLAIM |
| Process–nuclear fire/explosion propagation | hazard register only | NRC/IAEA mechanisms external | inventories/layout/SSC fragility unknown | site-specific | QUALITATIVE HAZARD IDENTIFICATION ONLY |
| Heat rejection/cooling | accounting boundary clarified | GTHTR300C source cycle only | off-design power/heat split unknown | source cycle → project | EXACT DUTY NOT MODELLED |
| Jurong Island suitability | screening criteria only | JTC/PUB context | parcel hazards/population/flood/security unknown | site-specific | SCREENING CASE ONLY |
| Singapore regulatory readiness | evidence matrix | authoritative MTI/EMA/NEA/IAEA current status | future policy evolution | national decision process | CURRENT-STATUS ASSESSMENT, NOT LICENSING APPROVAL |
| Spent-fuel dry storage | no project model | JAEA design + HTR-PM commissioning external | vendor/fuel-specific | pebble/prismatic differences | TECHNOLOGY FEASIBILITY SUPPORTED; PROJECT DESIGN UNRESOLVED |
| Cross-border CCS service | sensitivity + policy evidence | no operating Singapore chain | contract/cost/availability high uncertainty | future infrastructure | CONDITIONAL / UNRESOLVED |

## Model-validity boundary

The Rust model is valid for:
- deterministic reproduction of the selected source process/economic screening assumptions;
- mass/energy/lifecycle/cost bookkeeping under those assumptions;
- bounded parametric sensitivities whose equations and omissions are explicitly stated.

It is **not valid for**:
- reactor neutronics or thermal-hydraulic transients;
- IHX mechanical qualification;
- reliability/PRA;
- accident source term;
- atmospheric dispersion/dose/EPZ;
- chemical QRA;
- site selection;
- licensing;
- bankable FOAK project cost.

## Evidence-driven consequence for the final manuscript

Every major conclusion should carry one of four evidence labels where ambiguity is possible:

1. **DEMONSTRATED / EXPERIMENTAL** — e.g. HTTR 950 C operation, AGR TRISO tests.
2. **DESIGNED / SOURCE-MODELLED** — e.g. GTHTR300C architecture, INL Case 6.
3. **PROJECT-DERIVED / VERIFIED SCREENING** — e.g. annual lifecycle/cost ledgers and bounded sensitivities.
4. **UNRESOLVED / SITE-SPECIFIC / HIGHER-FIDELITY REQUIRED** — e.g. EPZ, Jurong site acceptability, mechanistic source term.

This prevents code verification from being presented as physical validation.
