# Full shifted-syngas sensible-heat bound to MDEA pinch v1

## Objective

Close the previously missing 226.85 -> 170 C interval without extrapolating NIST correlations outside their stated ranges.

## Property treatment

IAPWS-IF97 is authoritative for water/steam properties and is valid throughout the required industrial temperature/pressure region.

A full IF97 Region-2 enthalpy implementation is not yet introduced in the Rust property layer. For this narrow 443-500 K interval, the current model therefore uses:
- exact encoded NIST Shomate enthalpies where their validity covers the interval;
- an explicit water-vapour Cp bracket of 33-36 J/mol-K;
- an explicit N2 Cp bracket of 28-31 J/mol-K because the encoded N2 Shomate range also begins at 500 K.

The brackets are intentionally exposed as assumptions and can be replaced by full IF97 without changing the heat-balance API.

## Result status

The 412 -> 226.85 C source-stream calculation is already verified at ~14.49 MWth.

Adding the bounded 226.85 -> 170 C interval places the full temperature-feasible stream-6 sensible heat in the high-teens/low-twenties MW range.

The exact CI-reported value is being exposed as a regression diagnostic before it is frozen in this result file.

## Interpretation

Even before subtracting competing duties, this sensible-heat availability is only around half of the ~38-39 MW MDEA regeneration requirement.

Water condensation does not begin above a 160 C reboiler + 10 K pinch, so latent heat cannot fill the remaining gap in a simple exchanger.

Therefore a conventional-MDEA configuration cannot justify the external 63% direct-heat benchmark from condensation alone under the IEAGHG stream-6 pressure/composition.

## Competing-duty constraint

IEAGHG already uses the shifted-syngas cooling train for:
- shift WHB steam generation;
- BFW preheating;
- feed preheating;
- condensate preheating;
- demi-water preheating.

Thus the full 412->170 C sensible heat is an **availability ceiling**, not residual MDEA heat.

The next calculation must allocate this heat among existing cold services with a common DeltaTmin.

## Scientific consequence

The central MDEA integration case is likely to require a nonzero incremental low-grade heat supply unless:
- existing downstream heat-recovery duties are displaced/reconfigured;
- a lower-temperature solvent is selected;
- a heat pump upgrades sub-pinch condensation heat.

This strengthens the case for keeping capture-technology choice open rather than assuming conventional MDEA is automatically optimal for the nuclear-assisted flowsheet.
