# Source-bounded recycle HTGR envelope v1

## Stopping rule applied

A focused primary-source search confirms IEAGHG explicitly lists the shifted-syngas cooling sequence:
- Shift Converter Waste Heat Boiler;
- BFW Pre-heater;
- Feed Pre-heater;
- Condensate Pre-heater;
- Raw-H2 air cooler;
- Demi-water Pre-heater;
- process-condensate separator.

The accessible report does not provide numerical duties for the remaining Shift-WHB/BFW/condensate/demi-water exchangers.

Therefore those duties are not fabricated and the project stops trying to resolve them exactly from absent data.

## MDEA heat uncertainty

Case-2A MDEA latent regeneration:
~38.1-39.2 MWth.

Maximum shifted-syngas heat remaining after preserving the verified 4.899 MW feed-preheater service:
~13.9 MWth.

Because the other downstream duties are unresolved, direct recoverable MDEA heat is carried as:

0 <= Q_MDEA,recovered <= ~13.9 MW.

Thus incremental MDEA heat is approximately:

~24-39 MWth.

## 80% recycle thermal increment

The reduced 80% recycle reaction-heat change is approximately -11 MW relative to displaced fresh methane.

Combining with the source-bounded MDEA interval gives a current recycle thermal increment of order:

~13-28 MWth.

This remains incomplete because:
- changed feed sensible/preheat duties are not fully recomputed;
- recycle compression is not included;
- full high-temperature equilibrium is not solved.

## HTGR service consequence

The existing bounded conventional furnace-service envelope is ~131-151 MWth depending on reformer-preheat and furnace-steam assumptions.

At the 625 C reformer-inlet sensitivity, adding the source-bounded 80% recycle thermal increment moves the service requirement into roughly the mid-140s to high-170s MWth range.

This is now the appropriate reactor-side screening envelope for the recycle configuration, rather than the earlier ~131-151 MWth furnace-only range.

It still excludes nuclear-loop thermal losses and keeps electrical circulator/separation loads separate.

## Interpretation

Tail-gas recycle can reduce fresh NG materially, but high-capture recycle also creates a substantial low-grade solvent-regeneration duty.

The current uncertainty is therefore not whether recycle is energetically free—it is not—but whether the fresh-feed savings outweigh:
- added low-grade heat;
- separation electricity;
- recycle compression;
- CAPEX.

The economic screen should now use this bounded thermal requirement instead of optimistic zero/3-MW recycle-heat assumptions.
