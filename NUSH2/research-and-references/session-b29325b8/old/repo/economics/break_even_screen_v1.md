# Break-even economic screen v1

## Purpose

Before choosing a central HTGR cost, calculate what the assignment threshold can economically tolerate.

## Reference production

IEAGHG reference:
- 8.994 tH2/h;
- illustrative 8000 h/y;
- 71.952 ktH2/y.

If lifecycle abatement is Delta e kgCO2e/kgH2:

A [t/y] = 71,952 * Delta e.

At S$100/tCO2e, maximum annual incremental cost is:

Delta C_max [S$/y] = 100 * A.

Examples:

| Delta e (kgCO2e/kgH2) | Avoided Mt/y | Max incremental cost |
|---:|---:|---:|
| 6.0 | 0.432 | S$43.2m/y |
| 7.0 | 0.504 | S$50.4m/y |
| 7.5 | 0.540 | S$54.0m/y |
| 8.0 | 0.576 | S$57.6m/y |
| 9.0 | 0.648 | S$64.8m/y |

Thus the lifecycle result directly determines the economic headroom.

## CAPEX-only ceiling

If, unrealistically, the entire incremental-cost budget were available for capital recovery and incremental OPEX were zero:

CAPEX_max = Delta C_max / CRF.

At 8% and 25 years:
CRF ~= 0.09368.

For the minimum assignment abatement of 0.25 Mt/y:
S$25m/y / CRF ~= S$267m.

For the illustrative 7.5 kg/kg reference-plant abatement:
S$54m/y / CRF ~= S$576m.

These are **optimistic ceilings**, not affordable project CAPEX estimates.

Real incremental costs also include:
- nuclear fuel/O&M allocation;
- CCS capture OPEX;
- CO2 T&S;
- helium circulation;
- additional maintenance;
- financing during construction;
- contingencies;
- potentially lost electricity/steam credits.

Conversely, avoided costs/credits include:
- avoided furnace natural gas;
- avoided carbon tax where applicable;
- potentially retained/exported steam/electricity;
- avoided conventional equipment.

## Key economic implication

The S$100/t criterion is stringent enough that a Singapore-specific nuclear installation cannot be evaluated by asking whether its total capital cost is below S$267-576m.

The correct incremental comparison may allocate only a fraction of a multi-purpose HTGR to hydrogen/process heat, especially in cogeneration.

This is consistent with JAEA's GTHTR300C economic work, where cogenerated electricity and waste-heat utilisation materially change hydrogen cost.

Therefore the project must test at least:
1. dedicated HTGR-to-hydrogen;
2. cogeneration / shared-reactor allocation;
3. nuclear-electric eSMR.

A dedicated reactor charged fully to the hydrogen plant and a cogeneration reactor are economically different hypotheses.
