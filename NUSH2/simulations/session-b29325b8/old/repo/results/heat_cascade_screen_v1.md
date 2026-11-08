# Temperature-grade heat-cascade screen v1

## Research question

What fraction of the ~38-39 MWth Case-2A MDEA regeneration duty must be supplied as incremental nuclear heat?

## Evidence against f_MDEA,nuclear = 1

The conventional IEAGHG plant already recovers heat from:
- reformer syngas waste-heat boiler;
- shift-converter waste heat;
- feed/condensate preheating.

Independent SMR literature confirms that WGS heat is commonly recovered into the steam system.

More importantly, a 2024/2025 integrated electrified/convective SMR + CCS study reports two integrated amine reboilers:
- 63% of solvent-regeneration heat is supplied directly from post-LTS/condensing hot syngas;
- 37% is supplied from low-pressure steam.

A 2025 advanced blue-H2 study uses pinch-analysis-based low-quality waste-heat recovery and reports configurations with no external heating demand for the integrated low-temperature services.

These studies do NOT prove that the IEAGHG Case-2A flowsheet has 63% recoverable MDEA heat. They provide an external sensitivity anchor showing that assigning 100% of low-temperature solvent regeneration to new high-grade heat is unnecessarily conservative.

## Sensitivity definition

Let:

f_WH = fraction of MDEA regeneration heat supplied by retained/recovered process waste heat.

Then:

Q_MDEA,incremental
= (1-f_WH) Q_MDEA,total.

With:

Q_MDEA,total ~= 38.1-39.2 MWth.

Sensitivity cases:

| Waste-heat fraction | Incremental MDEA heat |
|---:|---:|
| 0% | 38.1-39.2 MW |
| 25% | 28.5-29.4 MW |
| 50% | 19.0-19.6 MW |
| 63% external-literature anchor | 14.1-14.5 MW |
| 75% | 9.5-9.8 MW |
| 100% | 0 MW |

## Combine with recycle reaction-heat change

The 80% reduced tail-recycle case has a conservative standard reaction-heat change of approximately:

Delta Q_rxn ~= -11 MW

relative to displaced fresh methane.

At the 63% waste-heat anchor:

incremental MDEA heat ~= 14.1-14.5 MW;

so the incomplete net thermal increment is only order:

~3 MWth.

This is a major reduction from the earlier naive +27-28 MW screen that assumed all MDEA heat was incremental.

It remains incomplete because:
- feed sensible/preheat duties change;
- S/C steam network is not yet recomputed;
- recycle compression remains;
- actual high-temperature reaction enthalpies/equilibrium differ;
- the 63% value is external evidence, not a fitted IEAGHG value.

## Temperature hierarchy

The heat cascade now has four main grades:

1. Primary reforming: ~800-900 C process / highest-grade secondary He.
2. Reformer/pre-reformer preheat: ~500-650 C.
3. HP steam superheat: ~395-400 C.
4. MDEA regeneration: ~150-180 C low-grade steam/reboiler service.

The first three compete directly for high/intermediate-grade helium/process heat.

The fourth should preferentially consume low-grade syngas/WGS/condensation heat before additional reactor heat is assigned.

## Consequence for HTGR sizing

Do not add 38-39 MW directly to the 131-151 MW furnace-service envelope.

Instead:

Q_HTGR,new
= Q_high/intermediate services
+ Q_MDEA,incremental(f_WH)
+ other unresolved losses/loads.

The project should carry f_WH as a sensitivity until a proper composite-curve/pinch model of the IEAGHG streams is built.

## Next acceptance criterion

Build a reduced composite-curve heat ledger using source temperatures/duties for:
- hot reformer syngas;
- WGS cooling;
- steam generation/superheat;
- feed preheat;
- MDEA reboiler.

The model must demonstrate a feasible temperature approach, not merely an energy balance.

Until then, 63% waste-heat supply is an external benchmark/sensitivity rather than a design result.
