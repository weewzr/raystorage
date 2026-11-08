# Matched lifecycle + economic screen for 80% recycle shared HTGR v1

## Purpose

Replace the illustrative 7.5 kgCO2e/kgH2 abatement denominator with a lifecycle calculation for the **same physical 80% recycle + shared direct-HTGR case** used in the economic model.

This is a screening LCA, not a final Singapore lifecycle assessment.

## Matched physical changes

Relative to IEAGHG unabated SMR:
- H2 output remains fixed;
- purchased supplementary furnace NG is eliminated;
- fresh feedstock NG is reduced by the reduced 80% recycle model (~26.6%);
- remaining fresh-feed carbon is subject to an explicit capture fraction;
- nuclear process heat receives an explicit lifecycle burden;
- CCS transport/storage chain receives an explicit emissions fraction.

This is more internally consistent than retaining a fixed 7.5 kgCO2e/kgH2 abatement while changing the process flowsheet.

## Baseline lifecycle boundary

Baseline includes:
1. IEAGHG direct plant CO2;
2. upstream NG burden for feedstock NG;
3. upstream NG burden for separately purchased furnace fuel.

It excludes infrastructure/construction and other minor lifecycle categories at this screening stage.

## Candidate lifecycle boundary

Candidate includes:
1. uncaptured carbon from remaining fresh-feed NG;
2. upstream burden of remaining fresh-feed NG;
3. direct nuclear-heat lifecycle proxy;
4. CCS transport/storage-chain emissions represented as a fraction of captured CO2.

Purchased furnace NG is zero.

## Explicit screening assumptions

The first regression uses:
- upstream delivered NG: 15 gCO2e/MJ;
- nuclear electricity LCA proxy: 12 gCO2e/kWh_e;
- net reactor electric efficiency proxy: 45%;
- direct process heat: 162 MWth;
- capture of remaining fresh-feed carbon: 90%;
- CCS chain emissions: 2% of captured CO2;
- operation: 8322 h/y.

These are **sensitivity assumptions**, not final Singapore-specific values.

The direct-heat nuclear term converts the electricity LCA factor to a thermal-throughput proxy using electric efficiency. This is an allocation proxy, not a published HTGR process-heat LCA.

## Economic coupling result

The matched lifecycle calculation regenerates the annual avoided CO2e and therefore the S$100/t annual incremental-cost allowance.

When that internally generated budget is inserted into the existing full-cost shared-reactor screen:
- 162 MWth;
- low Group-A T&S ~S$31.9m/y;
- allocated reactor ~S$50m/y;
- IHX/loop ~S$8.2m/y;
- reformer/recycle allowance S$5m/y;
- nuclear heat operating anchor S$5.69/GJ;
- separation electricity S$150/MWh;

the minimum NG value is:

**~S$14.28/GJ**

under the above lifecycle assumptions.

This is materially lower than the earlier ~S$17.5/GJ boundary that used the illustrative fixed S$54m/y budget.

## Why the boundary improves

The recycle architecture removes both:
- supplementary furnace NG and its upstream burden;
- a substantial fraction of fresh feedstock NG and its upstream burden.

Therefore its lifecycle abatement can be larger than the earlier fixed 7.5 kgCO2e/kgH2 illustrative value under some upstream-gas assumptions.

A larger lifecycle denominator creates a larger annual S$100/t cost allowance.

## Important uncertainty

The result is sensitive to upstream NG emissions.

For imported LNG into Singapore, upstream methane leakage, liquefaction, shipping and regasification can materially alter gCO2e/MJ. A generic 15 g/MJ value is not sufficient for the final paper.

Likewise:
- nuclear LCA must be sourced as a range;
- CCS shipping/injection emissions need route-specific evidence;
- capture fraction must come from the selected process topology.

Therefore S$14.28/GJ is a **matched-case demonstration**, not the final threshold.

## Scientific consequence

The previous conclusion that the shared architecture survives only at ~S$17.5/GJ gas is not robust to lifecycle consistency.

The correct conclusion is:

**shared-reactor economic feasibility is strongly coupled to the lifecycle intensity of Singapore's marginal/imported natural gas.**

Higher upstream NG emissions increase the emissions benefit of displacing NG and can enlarge the S$100/t budget; lower upstream emissions shrink it.

The next task should therefore establish a Singapore-relevant upstream NG/LNG lifecycle range and propagate it jointly with nuclear and CCS-chain LCA uncertainty.
