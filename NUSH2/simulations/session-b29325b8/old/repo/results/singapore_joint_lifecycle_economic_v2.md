# Singapore-relevant lifecycle + economic sensitivity v2

## Purpose

Propagate credible Singapore gas-supply, nuclear-LCA and CCS-chain ranges through the same 80% recycle/shared-HTGR lifecycle and full-cost economic model.

This replaces a single assumed upstream-gas intensity with an explicit range.

## Singapore gas import context

EMA Singapore Energy Statistics reports for 2024:
- total natural-gas imports: 11 Mtoe;
- LNG imports: 6 Mtoe;
- therefore LNG represented approximately 55% of imported gas on this energy basis.

This is an import-mix statistic, not a lifecycle factor.

## Gas-supply lifecycle anchors

IEA (2026) reports:
- global-average extraction/processing/transport natural-gas supply: ~11.5 gCO2e/MJ;
- global-average delivered LNG, production through regasification: 18.6 gCO2e/MJ in 2025.

These values span materially different supply pathways.

A simple Singapore 2024 import-mix screening proxy is:

(6/11)*18.6 + (5/11)*11.5
~= **15.37 gCO2e/MJ**.

Important limitation: 11.5 g/MJ is a global-average gas-supply anchor, not a measured lifecycle intensity of Singapore's specific pipeline-gas contracts. Therefore 15.37 g/MJ is a screening proxy only.

Use:
- 11.5 g/MJ lower upstream anchor;
- 15.37 g/MJ Singapore-mix screen;
- 18.6 g/MJ LNG-like upper anchor.

## Nuclear lifecycle range

UNECE LCA reports nuclear electricity lifecycle emissions of:

**5.1-6.4 gCO2e/kWh_e.**

The direct-process-heat model converts this to a thermal-throughput allocation proxy using 50.4% net electric efficiency.

This remains a derived proxy rather than a published HTGR process-heat LCA.

## CCS transport-chain range

The existing IEAGHG shipping sensitivity uses:
- 2.5% of captured CO2 for a shorter-distance case;
- 3.5% for a longer-distance case.

These remain route sensitivities until a specific Singapore-to-storage route is selected.

## Joint corner results

Common physical/economic case:
- 80% reduced recycle;
- 90% capture of remaining external feed carbon;
- 162 MWth direct nuclear service;
- 8322 h/y;
- low Group-A T&S cost ~S$31.9m/y;
- MHR-T allocation ~S$50m/y;
- IHX/loop ~S$8.2m/y;
- other integration allowance S$5m/y;
- nuclear-heat operating anchor S$5.69/GJ;
- separation electricity S$150/MWh.

### Lower-abatement corner
Inputs:
- upstream NG = 11.5 g/MJ;
- nuclear = 6.4 g/kWh_e;
- CCS-chain = 3.5% captured CO2.

Results:
- baseline = **10.813 kgCO2e/kgH2**;
- candidate = **1.949 kgCO2e/kgH2**;
- abatement = **8.864 kgCO2e/kgH2**;
- annual abatement = **0.663 MtCO2e/y**;
- S$100/t annual budget = **S$66.34m/y**;
- minimum gas value at low Group-A T&S = **S$14.69/GJ**.

### Higher-abatement/LNG-like corner
Inputs:
- upstream NG = 18.6 g/MJ;
- nuclear = 5.1 g/kWh_e;
- CCS-chain = 2.5%.

Results:
- baseline = **11.935 kgCO2e/kgH2**;
- candidate = **2.594 kgCO2e/kgH2**;
- abatement = **9.341 kgCO2e/kgH2**;
- annual abatement = **0.699 MtCO2e/y**;
- S$100/t annual budget = **S$69.92m/y**;
- minimum gas value at low Group-A T&S = **S$13.88/GJ**.

## Assignment-target robustness

Across these two deliberately opposed lifecycle corners:

annual abatement = **0.663-0.699 MtCO2e/y**,

well above the required:

>0.25 MtCO2e/y.

Therefore the annual-abatement-scale criterion is robust within this screening lifecycle range.

The economic criterion is less robust.

At low Group-A T&S, the minimum gas value required is approximately:

**S$13.9-14.7/GJ**

across the lifecycle corners.

Thus:
- S$10/GJ remains outside;
- S$15/GJ is narrowly inside under the current cost assumptions;
- S$20/GJ has more margin.

High Group-A T&S and larger integration/reformer costs can still remove this overlap.

## Dominant lifecycle uncertainty

Nuclear LCA varies only from 5.1 to 6.4 g/kWh_e and contributes a small term to total candidate intensity.

The larger lifecycle lever is upstream natural gas:
- dirtier/LNG-like gas raises baseline emissions;
- recycle/nuclear displaces part of that upstream burden;
- therefore lifecycle abatement and the S$100/t allowable budget increase.

This creates a counterintuitive economic effect: a dirtier displaced gas supply can make the assignment abatement-cost criterion easier to satisfy because the denominator is larger.

That is not an environmental argument for dirtier gas; it is a property of incremental abatement-cost accounting.

## Scientific limitation

The central weakness remains the reduced 80% recycle closure:
- fresh-NG displacement is not from a converged recycle flowsheet;
- 90% capture is imposed on remaining external feed carbon;
- tail-gas composition is not iterated after recycle;
- heat duty is not recomputed to convergence.

Therefore the next scientific task is the converged fixed-H2 recycle model, not another lifecycle-factor refinement.
