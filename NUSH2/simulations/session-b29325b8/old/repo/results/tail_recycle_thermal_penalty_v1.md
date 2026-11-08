# Tail-recycle thermal penalty screen v1

## Scope

This pass quantifies the two largest currently identifiable thermal effects in the reduced 80% Configuration-B recycle case:

1. reaction enthalpy of recycled CO/CH4;
2. MDEA solvent-regeneration steam.

It also corrects a potential double-counting error: gross stoichiometric water consumed by recycled species is not automatically additional plant steam because fresh NG is simultaneously displaced.

## 1. Reaction enthalpy

Authoritative NETL SMR references give standard reaction enthalpies:

CH4 + H2O -> CO + 3H2
Delta H°298 ~= +205.8 kJ/mol.

CO + H2O -> CO2 + H2
Delta H°298 ~= -41.2 kJ/mol.

Therefore complete CH4 reforming + shift is approximately:

+164.6 kJ/mol CH4.

For the 80% tail-recycle sensitivity, the recycled-species standard reaction-heat contribution is approximately:

Q_rxn,recycle ~= +4.5 MW.

This is small because the exothermic CO-shift term offsets part of the CH4 reforming heat.

This is a standard-state reaction screen, not a high-temperature reactor duty.

## 2. Displaced fresh-feed reaction heat

The 80% recycle case displaces ~387 kmol/h fresh NG mixture.

Using only its 89 mol% CH4 content and deliberately ignoring positive C2+ reforming duties, the displaced fresh-feed CH4 standard reaction heat is approximately:

~15-16 MW.

Therefore:

Q_rxn,recycle - Q_rxn,displaced-fresh-CH4
~= -11 MW.

The sign is important: the reduced recycle model should **lower**, not raise, chemical reforming reaction duty relative to the fresh feed it replaces.

Because C2+ reforming heat is omitted from the displaced-feed term, this reaction-heat saving is conservative in magnitude.

This does not yet include sensible heating, steam generation/superheat, equilibrium or pressure effects.

## 3. Steam balance correction

The reduced 80% recycle calculation reports ~563 kmol/h of stoichiometric H2O consumed by recycled CO/CH4 chemistry.

It would be wrong to add all 563 kmol/h as extra plant steam.

The same recycle case removes about 420 kmol-C/h of fresh feed while converting about 404 kmol-C/h of recycled CO+CH4. Carbon throughput is therefore nearly replaced rather than simply added.

A proper steam requirement must be recomputed from the chosen S/C constraint on the **combined fresh + recycle carbon feed**.

Until that is done, gross stoichiometric water consumption is a reaction diagnostic only.

## 4. MDEA regeneration steam

IEAGHG Case 2A reports approximately 66.9 t/h LP steam for tail-gas MDEA regeneration.

For saturated LP steam in the approximate 4-7 barg range, steam tables give latent heat roughly:

2048-2108 kJ/kg.

Thus the latent-heat service represented by 66.9 t/h is approximately:

**38.1-39.2 MWth.**

This is a major thermal service.

However, it is lower-temperature heat (~150-170 C saturation range), not 800-900 C reformer heat.

It should therefore be supplied from the lowest-grade economically available source:
- retained syngas/WGS heat recovery;
- secondary-helium cascade after higher-temperature duties;
- cogeneration/waste heat;
- or another steam source.

Charging it as fresh 900 C nuclear heat would overstate exergy demand.

## 5. Preliminary net thermal direction

At the 80% recycle sensitivity:
- MDEA latent heat: +~38-39 MWth;
- conservative standard reaction-heat change: ~-11 MW.

A crude arithmetic combination is therefore order:

+~27-28 MWth.

This is **not yet a valid total HTGR increment**, because:
- MDEA steam may be supplied by retained low-grade recovery;
- sensible steam duty is not included;
- fresh-feed preheat/steam duties decrease;
- C2+ reaction heat is omitted;
- recycle compression adds electricity;
- actual reaction enthalpies at reformer temperature differ from 298 K.

The value is retained only as a direction/order-of-magnitude screen.

## 6. Economic consequence

The previous 80% partial operating value was ~S$19/33/46m/y at S$10/15/20 per GJ NG and S$150/MWh electricity.

The ~38-39 MW low-grade MDEA heat requirement can materially erode that value if it requires dedicated paid heat.

Conversely, if it can be met predominantly by heat that would otherwise be rejected, its marginal economic penalty can be much smaller.

Therefore the next scientific bottleneck is now **heat integration**, not just reaction stoichiometry.

## Next task

Construct a temperature-grade heat cascade that places:
1. primary reformer;
2. reformer/pre-reformer preheat;
3. HP steam superheat;
4. MDEA LP-steam regeneration

against:
- secondary-helium cooling;
- syngas/WGS heat recovery.

Acceptance criterion: determine what fraction of the ~38-39 MW MDEA duty is genuinely incremental reactor heat versus recoverable low-grade process heat.
