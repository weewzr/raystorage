# Configuration B: CO2 removal + combustible tail-gas recycle — fixed-output upper bound

## Purpose

Quantify how much fresh natural-gas feed could possibly be displaced if PSA-tail CO2 is removed and the remaining H2/CO/CH4 is recycled, while holding H2 production fixed.

This is a **material-balance upper bound**, not yet a solved recycle flowsheet.

## Source tail-gas carbon

From the IEAGHG base PSA tail gas:

CO ~= 306 kmol/h
CH4 ~= 199 kmol/h.

Therefore non-CO2 recyclable carbon is:

n_C,recycle ~= 505 kmol-C/h.

Published feedstock NG carbon is approximately:

n_C,feed ~= 1579 kmol-C/h.

Thus, if every recycled carbon atom displaced a fresh-feed carbon atom:

f_displacement,max
= n_C,recycle/n_C,feed
~= 0.32.

So the absolute material-balance upper bound is roughly:

**32% of fresh-feed carbon**.

This is not an achievable process result because:
- recycled CO is already partially oxidised;
- CH4 and CO have different H2/heat consequences;
- equilibrium and PSA losses remain;
- steam demand changes;
- recycle compression/separation requires energy.

## Equivalent fresh-NG flow/energy upper bound

Using the published NG carbon content, the carbon-equivalent fresh NG displacement is roughly:

~470 kmol/h NG mixture.

On the source feedstock-energy basis, this corresponds to order:

~110 MW_LHV

of fresh feedstock NG.

This is separate from the ~55.94 MW_LHV purchased supplementary furnace NG that disappears when the fired furnace is removed.

The two terms must not be combined as though they were equally certain:
- supplementary-fuel removal is a direct architecture consequence;
- fresh-feed reduction is only a recycle upper bound until the chemistry is solved.

## Source-anchored utility penalty

IEAGHG Case 2A provides a same-scale tail-gas CO2 separation precedent:
- capture plant: 4.575 MWe;
- CO2 compression/dehydration: 2.874 MWe;
- tail-gas expander recovery: 1.140 MWe.

Net of that expander only:

W_anchor ~= 6.309 MWe.

This is not the complete Configuration-B electrical penalty:
- recycle recompression may differ;
- nuclear configuration pressure levels differ;
- solvent steam regeneration is also required;
- downstream reformer heat changes.

It is a useful lower-layer utility anchor.

## Economic sensitivity

At an explicit gas price p_NG [S$/GJ], the upper-bound fresh-feed saving is:

S_feed,max
= E_feed,displaced,max * p_NG.

At 8322 h/y, ~110 MW corresponds to roughly 3.3 million GJ/y.

Illustratively:
- S$10/GJ -> ~S$33m/y;
- S$15/GJ -> ~S$50m/y;
- S$20/GJ -> ~S$66m/y.

These are **upper-bound credits**, not predicted savings.

They show why tail-gas recycle could materially change the economic conclusion if a substantial fraction of the theoretical displacement survives a real process calculation.

## Next refinement

Replace the carbon-only displacement with a reduced reaction model.

At fixed H2 output:
1. recycle source H2 directly;
2. shift recycled CO;
3. reform recycled CH4;
4. calculate H2 generated per recycled carbon;
5. reduce fresh NG until H2 target is restored;
6. recalculate steam demand and reforming duty;
7. apply PSA recovery;
8. iterate to recycle convergence.

This will turn the ~32% fresh-feed displacement ceiling into a physically achievable estimate.
