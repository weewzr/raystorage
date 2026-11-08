# Full chemical-plant and CCS cost ledger — scope gate

## Correct comparison
Both conventional baseline and proposed nuclear-assisted system must deliver the same hydrogen quantity, quality and annual service. Define
`AC=(C_nuclear_assisted,y-C_baseline,y)/(E_baseline,y-E_nuclear_assisted,y)`,
with costs S$/y and lifecycle emissions tCO2e/y. Neither annual ledger may omit required items merely because they are unpriced.

## Cost scope matrix
| Category | Conventional baseline | Nuclear-assisted system | Present evidence/decision |
|---|---|---|---|
| Reformer, fired furnace, process equipment | Fired reformer + WGS | Nuclear-heated reformer + WGS | Full installed costs **UNPRICED** |
| PSA, gas feed treatment, compression | Required | Required | **UNPRICED**, no unjustified cancellation without equal scope |
| Steam generation/recovery, utilities, cooling, water | Required | Required | **UNPRICED** |
| Catalyst, consumables, fixed and variable maintenance | Required | Required | **UNPRICED** |
| Electricity and fuel | NG fuel/feed, purchased power | NG feed, purchased power, nuclear heat | NG screening price modelled; other complete utility costs **UNPRICED** |
| Amine absorber/stripper, regeneration, solvent | Baseline configuration-dependent | Required in proposed CCS case | Capture capital screen represented; full capture OPEX **UNPRICED** |
| CO2 drying, compression/liquefaction, storage buffer | If CCS comparator | Required | Scope and CAPEX/OPEX **UNPRICED** |
| CO2 transport, storage, MRV/liability | If CCS comparator | Required | S$15/t screening tariff, not a contract |
| Nuclear reactor, IHX/secondary helium | Not applicable | Required | FOAK/BOAK/10-OAK scenario cost represented; project qualification and full BOP unresolved |
| EPC/owner costs, contingency, land/site | Required | Required | **UNPRICED**, may differ |
| Financing/IDC, insurance, licensing/security | Required where applicable | Additional nuclear-specific burdens | **UNPRICED** |
| Waste/decommissioning, working capital | Required where applicable | Required | **UNPRICED** |

## CAPEX/OPEX boundary
The existing E2B annual ledger represents baseline natural gas and proposed-system natural gas, capture annualized capital, a T&S tariff, nuclear annualized capital/OPEX and IHX/loop allowance. It is **not a full chemical-plant cost ledger**. Do not sum a total installed plant estimate with equipment costs already included; source cost definitions and battery limits (ISBL/OSBL) must be reconciled before doing so.

## Current stage economics
Previous represented-scope model: two-train FOAK ~S$137.74/tCO2e; BOAK ~S$74.14/tCO2e; mature 10-OAK ~S$42.84/tCO2e. These are **historical represented-scope scenarios, not fully costed results**. They cannot be promoted as revised full-system passes. With multiple mandatory cost categories unpriced, a defensible central full-project abatement cost or interval is **NOT CALCULABLE** without sourced bounds. Assigning arbitrary zero or arbitrary positive values would create false precision.

## Data required
A matched-service IEAGHG/NETL/DOE or vendor cost breakdown with explicit capacity, year/currency, scope (ISBL/OSBL), installed vs overnight vs total capital, fixed/variable OPEX, CCS integration and PSA offgas handling. Separate Singapore land, licensing, financing, transport/storage and nuclear integration quotations/engineering estimates are required. Apply source-year indices and currency conversion consistently, then calculate bounds and test whether the original economic ranking survives.

## Status
**Scope identified; complete costing NOT VERIFIED.** No unsupported revised FOAK/BOAK/10-OAK central numbers are issued.

## Newly recovered primary cost benchmark (source recovery)
IEAGHG 2017-02 (https://ieaghg.org/publications/techno-economic-evaluation-of-smr-based-standalone-merchant-hydrogen-plant-with-ccs/) assesses 100,000 Nm3/h H2 standalone SMR, reporting total-plant capital increases of 18–79% (approximately EUR40–176 million, Q4 2014), operating-cost increases of 18–33%, and CO2 avoidance costs EUR47–70/t for differing capture configurations. **These are source results, not nuclear-assisted project cost estimates.** They cannot be added directly to the existing annualized capture CAPEX because that risks double counting; nor can they be transplanted to the 130 MMSCFD Case-6 train without matched scope/capacity, price-year, geography, power export and equipment boundary reconciliation. See `results/TARGETED_PSA_COST_SOURCE_RECOVERY.md`. Full FOAK/BOAK/10-OAK economics remain unverified.
