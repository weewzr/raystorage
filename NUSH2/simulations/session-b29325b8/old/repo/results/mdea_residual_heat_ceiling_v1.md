# Residual MDEA heat ceiling after source-explicit feed preheat

## New source allocation

IEAGHG explicitly states that natural gas is heated to 135 C in the Feed Pre-heater by the cooled shifted syngas leaving the BFW pre-heater.

The project has already reconstructed this feed-preheater duty from source streams and NIST properties:

Q_feed-preheat ~= 4.899 MW.

This duty is therefore not an optional external load; it is an existing shifted-syngas heat-recovery commitment.

Because the cold-side target is only 135 C, it is temperature-compatible with the same shifted-syngas heat interval being considered for a 160 C MDEA reboiler with a 10 K approach.

## Updated residual ceiling

Full shifted-syngas sensible heat above 170 C:

Q_shift,412->170 = 18.756-18.842 MW.

Preserve feed preheat:

Q_residual,max
= Q_shift - Q_feed-preheat
~= 13.86-13.94 MW.

Relative to the Case-2A MDEA regeneration latent duty of ~38.1-39.2 MW:

f_MDEA,residual,max
~= 35-37%.

This remains an **optimistic upper bound** because it still subtracts none of:
- Shift Converter WHB steam generation;
- BFW preheating;
- condensate preheating;
- demi-water preheating.

All are explicitly present in the shifted-syngas cooling train.

Therefore the actual directly recoverable MDEA fraction under the 160 C / 10 K screen must be lower unless those existing duties are reassigned to another heat source.

## Consequence

The external 63% post-shift/condensing-syngas benchmark is now clearly not transferable to this IEAGHG configuration under the current MDEA temperature/pinch assumptions.

Even before the other source-required duties are allocated, the maximum residual sensible-heat fraction is already below ~37%.

## Remaining allocation problem

The IEAGHG public equipment-list text identifies the Shift WHB, BFW preheater, condensate heater and demi-water preheater but does not expose their numerical duties in the searchable table.

Do not fabricate them.

The next defensible options are:
1. reconstruct each duty from source cold-stream states where available;
2. derive lower bounds from known steam generation and water flows;
3. if exact states remain unavailable, carry an explicit residual-heat sensitivity bounded above by ~13.9 MW.

This is now sufficiently constrained to update the MDEA incremental-heat envelope without pretending the exact pinch allocation is known.
