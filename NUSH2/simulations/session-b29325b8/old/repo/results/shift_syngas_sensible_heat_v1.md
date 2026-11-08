# IEAGHG shifted-syngas sensible-heat reconstruction v1

## Newly recovered primary-source states

The full IEAGHG 2017-02 heat/material balance resolves the previously missing HTS states:

- Stream 5, HTS inlet: 320 C, 2.80 MPa, 8370.3 kmol/h.
- Stream 6, HTS outlet: 412 C, 2.77 MPa, 8370.3 kmol/h.
- Stream 7, PSA inlet after downstream cooling/condensate separation: 35 C, 2.58 MPa, 6596.9 kmol/h.

Stream-6 composition:
- CO2 0.1283;
- CO 0.0366;
- H2 0.5961;
- N2 0.0015;
- CH4 0.0238;
- H2O 0.2137.

The HTS therefore raises gas temperature from 320 to 412 C through the exothermic WGS reaction before the downstream heat-recovery train.

IEAGHG explicitly lists that train as:
1. Shift Converter Waste Heat Boiler;
2. BFW pre-heater;
3. Feed pre-heater;
4. Condensate pre-heater;
5. Raw-H2 air cooler;
6. Demi-water pre-heater;
7. process-condensate separator.

## Property model

NIST SRD 69 Shomate enthalpies are used for CO2, CO, H2, N2, CH4 and H2O.

The NIST water-vapour Shomate correlation is verified only from 500 K upward. Therefore the first source-based sensible-heat calculation deliberately cools stream 6 only to:

500 K = 226.85 C,

rather than extrapolating the water-vapour correlation down to the ~160 C MDEA region.

This is conservative for recoverable heat and leaves condensation heat entirely uncredited.

## Result

Cooling the source stream-6 mixture from:

412 C -> 226.85 C

gives approximately:

**14.49 MWth**

of ideal-gas sensible heat.

Compared with the Case-2A MDEA latent-heat service of ~38.1-39.2 MWth, this corresponds to approximately:

**37-38% of the MDEA duty**

from shifted-syngas sensible heat alone, before any condensation heat is credited.

## Interpretation

This result is stronger than the earlier steam-ledger argument in one sense: it is calculated directly from the published shifted-syngas stream state and composition.

But it is still an **availability ceiling**, not an allocatable heat result, because the existing IEAGHG downstream train already uses this heat for:
- shift WHB steam generation;
- BFW preheat;
- NG feed preheat;
- condensate/demi-water heating.

A furnace-free redesign can re-optimise those services, but they cannot simply be deleted.

Thus:

Q_shift,sensible(412->226.85 C) ~=14.49 MW

does NOT imply 14.49 MW is freely available to MDEA.

## Relationship to prior bounds

Current evidence hierarchy:

1. Explicit non-reformer-WHB steam-group scale: ~16.45-16.82 MW, but shared between shift recovery and furnace SG.
2. Direct stream-6 sensible heat, 412->226.85 C: ~14.49 MW.
3. External integrated-SMR MDEA benchmark: 63% regeneration heat from post-shift/condensing syngas.

The direct stream calculation shows that sensible heat alone can cover ~37-38% of MDEA duty while staying above 226.85 C.

To reach the external ~63% benchmark would require additional recoverable heat from:
- stream-6 cooling below 226.85 C;
- water-vapour condensation;
- other low-grade process sources;

after preserving competing duties.

## Critical next property/model requirement

The source data show a large water-flow reduction between streams 6 and 7:
- stream 6: 8370.3 kmol/h, 21.37 mol% H2O;
- stream 7: 6596.9 kmol/h, 0.24 mol% H2O.

Therefore substantial water condensation occurs in the cooling train.

The next model must use pressure-dependent water phase equilibrium / steam properties rather than ideal-gas Shomate extrapolation.

Acceptance criterion:
- calculate stream-6 dew point at 2.77 MPa;
- cool/condense to a declared MDEA pinch temperature;
- quantify sensible + latent recoverable heat;
- preserve existing BFW/feed/condensate heating duties;
- derive the residual heat genuinely available to MDEA.
