# Complete natural-gas feed-preheater duty

## Boundary

IEAGHG states that:
- raw NG is first heated to 135 C by shifted-syngas heat recovery;
- the feedstock NG is then heated from 135 C to 370 C in the fired-reformer Feed Pre-Heater Coil;
- a recycled-H2 slipstream is mixed with the feedstock before pretreatment.

This calculation closes the **natural-gas portion** of the 135->370 C furnace-convection duty. Recycled H2 is still separate because the source flowrate of that slipstream has not yet been recovered.

## Property basis

For CH4, CO2 and N2, NIST SRD 69 Shomate enthalpy correlations are used.

For C2H6, C3H8, n-C4H10 and n-C5H12, NIST SRD 69 tabulated ideal-gas Cp values are integrated piecewise linearly between 408.15 and 643.15 K. This avoids fabricating Shomate coefficients where the public species page instead provides recommended Cp tables.

The published IEAGHG NG composition is represented as:
- CO2 2.00 mol%;
- CH4 89.00%;
- C2H6 7.00%;
- C3H8 1.00%;
- n-C4H10 0.10%;
- n-C5H12 0.01%;
- residual 0.89% treated as N2.

## Result

The mixture sensible-enthalpy increase is approximately:

Delta h_NG = 11.922 kJ/mol mixture.

At 1455.8 kmol/h:

Q_feed-preheat,NG = 4.821 MW.

This supersedes the previous 4.05 MW methane-only lower bound for the NG portion of this coil.

It is still not the complete coil duty because recycled H2 is omitted.

## Updated furnace-service accounting

Source-constrained/derived:
- radiant reformer duty: 96.04 MW;
- HP steam superheat: ~16.0-16.25 MW;
- NG portion of 135->370 C feed preheat: 4.82 MW.

Therefore the presently quantified furnace-dependent services are already approximately:

116.9-117.1 MW,

before:
- recycled-H2 feed preheat;
- pre-reformer feed-preheat coil;
- reformer preheat coil;
- furnace-only share of saturated-steam generation;
- nuclear/IHX/loop heat losses.

This remains a lower bound on the total thermal services that must be replaced or reconfigured when the fired furnace is removed.

## Verification rule

The next step is not to guess the recycled-H2 flow or missing coil inlet temperatures. Those quantities must be recovered from the IEAGHG stream table/process description or represented as explicit sensitivities.
