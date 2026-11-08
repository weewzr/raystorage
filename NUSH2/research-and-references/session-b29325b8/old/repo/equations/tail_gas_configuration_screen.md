# PSA tail-gas configuration screen v1

## Source inventory

IEAGHG base PSA tail gas:
- total 2106.3 kmol/h;
- H2 23.69%;
- CO2 50.95%;
- CO 14.54%;
- CH4 9.45%.

Derived:
- H2 ~= 499 kmol/h;
- CO2 ~= 1073 kmol/h;
- CO ~= 306 kmol/h;
- CH4 ~= 199 kmol/h;
- existing CO2 ~= 47.2 t/h;
- non-CO2 carbon (CO+CH4) ~= 505 kmol-C/h;
- combustible LHV inventory ~=85 MW.

## Configuration A — untreated recycle

Recycle the entire PSA tail stream toward feed/reforming.

Advantages:
- simplest separation topology;
- retains H2/CO/CH4 chemical value;
- contemporary Ahn & Lee h-SMR/eSMR uses tail-gas recycle conceptually.

Problems:
- ~51 mol% of the stream is already CO2;
- recycling that CO2 increases inert/CO2 circulation;
- additional compression is required from PSA tail-gas pressure;
- equilibrium/reformer duty and capture flow change;
- untreated recycle may be thermodynamically inferior to separating CO2 first.

No economic credit is assigned yet.

## Configuration B — CO2 removal then combustible recycle

IEAGHG Case 2A is a direct source precedent:
- PSA tail gas is compressed from ~0.2 MPa to ~1 MPa;
- MDEA removes CO2;
- CO2-depleted tail gas is reheated/expanded and returned to reformer burners;
- capture-plant power consumption = 4.575 MWe;
- CO2 compression/dehydration = 2.874 MWe;
- tail-gas expander recovers 1.140 MWe;
- net grid import = 1.070 MWe;
- LP steam for solvent regeneration ~=66.9 t/h in the technical-review summary.

This proves tail-gas CO2 separation is technically modelled at the same reference scale, but Case 2A burns the sweet tail gas rather than recycling it to feed.

For a nuclear case, the same separation location could instead route CO2-depleted H2/CO/CH4 toward reforming/recovery.

This is currently the most source-anchored candidate for further modelling.

## Configuration C — enhanced H2 + CO2 recovery

Recent literature proposes cryogenic separation plus staged PSA for SMR tail gas, simultaneously increasing H2 recovery and CO2 removal.

This may create more product value than simple recycle but adds:
- refrigeration/separation CAPEX;
- power;
- complexity;
- additional PSA equipment.

It remains a competing architecture, not selected.

## Stoichiometric upper bound

If, unrealistically ideally:
- all existing tail-gas H2 is recovered;
- all CO is shifted: CO + H2O -> CO2 + H2;
- all CH4 is fully reformed+shifted: CH4 + 2H2O -> CO2 + 4H2;

then potential molecular H2 is:

n_H2,max = n_H2 + n_CO + 4 n_CH4
         ~= 1,600 kmol/h.

Relative to current ~4,490 kmol/h H2 product, this is a large chemical inventory.

It is **not an achievable product increment**:
- extra steam and heat are required;
- equilibrium limits conversion;
- PSA/recovery losses remain;
- recycle changes the whole flowsheet.

Its purpose is to show that treating tail gas only as waste/fuel would miss a first-order process opportunity.

## Carbon consequence

The tail gas already contains ~47 t/h CO2.

Additionally ~505 kmol-C/h remains as CO+CH4. If fully converted, this becomes another ~22 t/h CO2-equivalent carbon stream.

Therefore a furnace-free architecture must explicitly capture/recycle/manage essentially all of this carbon if high lifecycle capture is claimed.

## Current ranking of scientific information value

Do not select a winner yet.

Highest-value next detailed model:
**B — CO2 removal + combustible recycle**, because IEAGHG Case 2A supplies direct same-scale compression/capture utility evidence and the nuclear modification is conceptually isolated.

Configuration A is simpler but lacks a source-backed compression/equilibrium model.
Configuration C may be attractive but requires a new separation model beyond the current research critical path.

## Acceptance criterion for next model

For configuration B:
1. remove source-resolved tail-gas CO2;
2. recycle H2/CO/CH4;
3. add required compression;
4. solve revised reformer carbon/H2 balance;
5. update process heat;
6. update H2 output or reduce fresh NG at fixed H2 output;
7. update capture mass and lifecycle emissions;
8. calculate annual real savings.

Only then compare the result with the S$17-29m/y economic gap.
