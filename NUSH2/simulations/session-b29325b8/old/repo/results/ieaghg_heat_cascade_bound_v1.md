# IEAGHG-source heat-cascade bound v1

## Objective

Replace the external 63% MDEA waste-heat benchmark with a source-derived constraint before building a full composite curve.

## Source heat hierarchy

IEAGHG 2017-02 states:
- primary-reformer product gas leaves at ~900-950 C;
- it is cooled in the Reformer Waste Heat Boiler (WHB) to ~320 C;
- that WHB generates high-pressure saturated steam;
- saturated HP steam is subsequently superheated in the fired-furnace convection section;
- saturated HP steam is generated from three sources: reformer WHB, shift-converter heat recovery and furnace-convection steam generation;
- around 75% of saturated HP steam is generated in the reformer WHB.

Therefore the large 900-950 -> 320 C syngas heat source is already heavily committed to the baseline steam system.

It must not be counted again as freely available MDEA heat.

## Source-derived flexible-heat upper group

Total HP steam scale:
~141.354 t/h.

Non-reformer-WHB group:
~25% ~=35.34 t/h.

This group is generated jointly by:
- shift-converter heat recovery;
- furnace steam-generation coil.

Using saturated-steam latent heat near 4.23 MPa gives an energy scale of:

~16.45-16.82 MWth.

When the fired furnace disappears, the furnace share of this group disappears rather than remaining as recoverable waste heat.

Therefore **16.45-16.82 MW is itself an upper bound on retained shift-derived steam-equivalent heat**, because the true shift share is smaller.

## Compare with MDEA regeneration

Case-2A MDEA latent-heat service:
~38.1-39.2 MWth.

If the entire non-WHB steam group were attributed to retained shift heat—an intentionally optimistic assumption—the source-based waste-heat fraction would be only approximately:

f_WH,max,source
~= 16.45-16.82 / 38.1-39.2
~= 0.42-0.44.

Thus the accessible IEAGHG steam ledger supports:

**f_WH <~ 44%**

from this explicitly quantified retained heat pool.

This does NOT prove the actual maximum is 44%, because additional low-grade sensible/condensing heat may exist downstream that is not represented by saturated-steam generation.

Conversely, it means the external 63% literature anchor cannot yet be adopted as the IEAGHG central case without identifying another ~7-8 MW or more of temperature-feasible recoverable heat.

## Incremental MDEA heat under source-ledger ceiling

At f_WH ~= 0.42-0.44:

Q_MDEA,incremental
~= 21-23 MWth.

Combining only with the conservative ~-11 MW standard reaction-heat change gives an incomplete net thermal increment of order:

~10-12 MWth.

This lies between:
- naive no-integration screen: +27-28 MW;
- external 63% anchor screen: ~+3 MW.

It is currently the most defensible **source-ledger** screening range.

## Temperature feasibility

The MDEA reboiler operates at low temperature relative to:
- reformer syngas WHB;
- shift-reactor outlet/cooling train.

Therefore the issue is primarily **heat availability and competing steam duties**, not insufficient source temperature.

A formal pinch model still requires:
- hot-stream inlet/outlet temperatures;
- heat-capacity/phase-change loads;
- cold-stream target temperatures;
- declared DeltaTmin.

## Key scientific correction

A composite-curve model must preserve the baseline steam-generation commitments.

The correct question is not:

"How much hot syngas heat exists?"

It is:

"How much heat remains temperature-feasible for MDEA after preserving the required steam/feed services of the furnace-free plant?"

This avoids double-counting the reformer WHB heat that already supplies ~75% of HP steam.

## Next data gap

The accessible IEAGHG source does not separately publish the shift-WHB duty in MW.

Therefore the next step is to reconstruct it from the source stream states around:
- reformer WHB outlet / HTS inlet;
- HTS outlet;
- downstream cooling/condensation;

using stream flow/composition and NIST/IAPWS enthalpies.

That reconstruction can replace the current 16.8 MW upper-group bound with an actual shift/syngas recoverable-heat estimate.
