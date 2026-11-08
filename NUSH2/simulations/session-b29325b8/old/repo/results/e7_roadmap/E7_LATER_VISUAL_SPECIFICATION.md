# E7 Later Visual Specification — Decision-Gated Implementation Roadmap

## Purpose
Create a submission-facing roadmap graphic from deterministic E7 gate data. It is a logical maturity/decision sequence, not a calendar promise.

## Main spine
CURRENT RESEARCH / PRE-DEPLOYMENT
-> COMPONENT + INTEGRATION QUALIFICATION
-> FOAK PROCESS-HEAT DEMONSTRATION
-> TWO-TRAIN 353.6 MWth DEMONSTRATION
-> REPLICATION / EARLY-COMMERCIAL BOAK
-> MATURE / 10-OAK ECONOMIC GATE
-> SINGAPORE SITE + LICENSING + CONTRACT CLOSURE
-> CONSTRUCTION + COMMISSIONING
-> COMMERCIAL OPERATION + MEASURED VERIFICATION

## Cross-cutting workstreams
Show beneath/above the spine:
- Singapore national nuclear readiness/regulatory capability;
- integrated nuclear/process safety;
- candidate-site/Jurong studies;
- cooling/water/UHS;
- H2 market/offtake;
- NG/feedstock;
- CCS transport/storage;
- supply chain/workforce;
- economic data/learning validation;
- financing/commercial closure;
- environment/EPR/security.

Use dependency arrows from the canonical `e7_dependencies.csv`; do not make every box appear independent.

## Gate callouts
Include only deterministic labels:
- FOAK two-train economics: ~S$137.74/tCO2e — FAIL;
- BOAK target/model result: ~S$74.14/tCO2e — PROJECTED/MODELLED PASS;
- 10-OAK target/model result: ~S$42.84/tCO2e — PROJECTED/MODELLED PASS;
- architecture: 1 x 600 MWth -> 353.6 MWth -> 2 x 130 MMSCFD;
- H2: ~195,892 t/y;
- CO2 captured: ~1.085 Mt/y;
- availability model basis: 85%.

## STOP/REDESIGN notation
Use explicit red/stop-style symbols in final design workflow only for logical outcomes, not invented probabilities:
- REDESIGN
- RELOCATE
- REDUCE SCALE
- DELAY
- STOP

Each major gate should visibly have a failure branch.

## Jurong
Label: "Jurong Island — candidate industrial context; not selected/approved nuclear site."
If Jurong fails, arrow to "evaluate another Singapore context", not automatically to technology death.

## No fake timeline
Do not put calendar years or invented durations on the roadmap. If a real external programme date is later shown, label it as external context, not a project commitment.

## Visual-generation boundary
The roadmap, gate matrix, dependency graph and economic-gate labels should be deterministic vector/data graphics. Do not use AI image generation for quantitative gate logic. AI artwork may later support a conceptual site/equipment illustration only.
