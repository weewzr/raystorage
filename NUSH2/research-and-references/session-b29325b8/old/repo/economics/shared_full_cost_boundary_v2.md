# Shared-reactor full-cost feasibility boundary v2

## Purpose

Test whether the surviving shared/cogeneration architecture remains feasible after reserving:
- source-backed MHR-T thermal-share reactor cost;
- source-backed IAEA IHX + secondary-helium-loop cost;
- a deliberately nonzero reformer/recycle integration allowance.

## Screening case

Use:
- allowed annual incremental-cost budget: S$54m/y;
- HTGR process service: 162 MWth;
- 80% recycle NG displacement: ~145.95 MW_LHV;
- nuclear-heat operating anchor: S$5.69/GJ (legacy JAEA assumption translated at current FX; not a Singapore tariff);
- separation electricity: 6.309 MWe at S$150/MWh;
- allocated MHR-T reactor annual cost: ~S$50m/y;
- IAEA IHX + secondary-loop annualised cost: ~S$8.2m/y;
- other reformer/recycle integration allowance: **S$5m/y sensitivity**.

The S$5m/y term is intentionally nonzero but is NOT claimed as a source-derived helium-reformer retrofit cost. Modern eSMR literature supports preservation/reuse of existing reformer infrastructure, but no directly transferable helium-heated retrofit CAPEX was found.

## Minimum gas value at low Group-A T&S

Low Group-A-like T&S:
~S$31.9m/y.

The full-cost boundary requires approximately:

**p_NG,min ~= S$17.5/GJ**

for the above midpoint assumptions.

Thus:
- S$10/GJ: outside feasible region;
- S$15/GJ: outside feasible region;
- S$20/GJ: inside this screening boundary.

This is much narrower than the pre-integration result.

## Maximum T&S at S$20/GJ gas

At S$20/GJ gas, the maximum annual CCS T&S compatible with the same full-cost boundary is approximately:

**C_TS,max ~= S$42.8m/y.**

The earlier Group-A-like annual T&S sensitivities were:
- low: ~S$31.9m/y;
- high: ~S$47.85m/y.

Therefore:
- low Group-A T&S can fit at S$20/GJ gas;
- high Group-A T&S does not fit even at S$20/GJ gas under this midpoint reactor/integration allocation.

## Interpretation

A shared HTGR feasibility region still exists, but only under favourable combinations.

Representative surviving combination:
- NG value >= roughly S$17.5/GJ;
- CCS T&S toward the low end of Group A;
- MHR-T-like low thermal allocation fraction;
- IHX/secondary-loop near the legacy IAEA anchor;
- modest additional reformer/recycle cost.

The architecture fails if several adverse factors coincide:
- gas value around S$15/GJ or lower;
- T&S near the high Group-A end;
- higher reactor allocation;
- larger integration retrofit cost.

## Retrofit evidence

Recent eSMR studies demonstrate that conventional reformer infrastructure can sometimes be retained:
- Mehanovic et al. propose radiant electrification of conventional reformer tubes and emphasize reuse of existing infrastructure;
- Rossi et al. retain the existing terrace-wall reformer design and much of the convection heat-recovery system.

These studies support the hypothesis that retrofit cost can be below greenfield reformer replacement.

They do **not** provide a helium-heated reformer retrofit cost, so they cannot justify a precise S$5m/y allowance. That term remains a sensitivity.

## Current economic verdict

The economic evidence now supports three differentiated conclusions:

1. Dedicated large HTGR: outside the current S$100/t screening region.
2. Shared HTGR at representative S$15/GJ midpoint: outside once reactor + IHX/loop + nonzero integration are charged.
3. Shared HTGR under favourable high-gas / low-T&S conditions: still overlaps the S$100/t region.

Therefore the concept is not globally falsified, but its assignment-level economic feasibility is **conditional and narrow**.

## Next scientific dependency

The next high-information task is no longer another cost-boundary manipulation.

The model must replace the illustrative S$54m/y / 7.5 kgCO2e/kgH2 abatement denominator with a consistent lifecycle result for the same 80% recycle/shared-reactor case.

Upstream NG emissions decrease when fresh NG is displaced, capture mass changes, and nuclear lifecycle emissions change with reactor heat.

Those changes alter both:
- annual MtCO2e avoided;
- the S$100/t annual cost budget.

A matched lifecycle + economic case is therefore required before declaring assignment compliance or failure.
