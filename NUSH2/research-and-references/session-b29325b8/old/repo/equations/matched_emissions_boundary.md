# Matched emissions specification: direct HTGR heat vs nuclear-electric eSMR

## Purpose

The previous comparison established different reactor-energy requirements for delivering the same process heat. This note establishes the emissions accounting needed to compare the two architectures without accidentally changing the chemistry at the same time.

## Matched comparison specification

For the first controlled comparison, both cases must have identical:

- H2 product flow and purity;
- natural-gas feed quantity/composition;
- steam-to-carbon basis;
- methane/carbon conversion;
- CO2 capture fraction and storage permanence;
- PSA H2 recovery;
- PSA-tail-gas disposition;
- CO2 compression/transport boundary;
- upstream natural-gas supply chain.

The only intended difference is the route used to provide reformer/thermal services:

### D — direct heat
HTGR thermal -> IHX -> secondary He -> process.

### E — nuclear-electric
HTGR thermal -> power cycle -> electricity -> electric reformer/process heating.

## Plant-gate invariant

If feedstock carbon and capture/tail-gas handling are identical, the plant-gate carbon emission is identical.

For the IEAGHG feedstock:

feed carbon = 1578.56 kmol-C/h.

Carbon-equivalent CO2:

1578.56 * 44.0095
~= 69.47 tCO2/h.

At 8994 kg H2/h:

e_feed-C
~= 7.72 kgCO2/kgH2.

For a matched permanent carbon-capture fraction eta_C:

e_plant = 7.72(1-eta_C) kgCO2/kgH2.

At 90% carbon capture this screening identity gives about:

0.77 kgCO2/kgH2

of residual feedstock-carbon CO2.

This is **not yet the full lifecycle intensity**. It excludes upstream methane/NG emissions, nuclear lifecycle emissions, CO2 transport/storage emissions, fugitive H2 and other boundary terms.

## Why this matters for interpreting Ahn & Lee 2026

Ahn & Lee report different net GWPs for h-SMR and e-SMR (3.5 and 3.18 kgCO2/kgH2) while both remove the fired furnace and recycle PSA tail gas.

Under the strictly matched comparison defined above, plant-gate feedstock-carbon emissions cannot differ solely because heat arrived as helium rather than electricity.

Therefore their GWP difference must arise from differences in process configuration, resource consumption, allocation/lifecycle terms, or other model details.

Their result is a valuable external benchmark/contradiction, but it must not be copied into this model as an intrinsic emissions advantage of eSMR.

Source:
Ahn & Lee (2026), International Journal of Hydrogen Energy 247, 155890.

## Lifecycle boundary

DOE 45VH2-GREET uses a well-to-gate boundary that includes:
- feedstock extraction and delivery;
- electricity generation;
- facility emissions;
- CO2 capture and delivery;
- potential sequestration leakage.

This is a useful methodological template, but US natural-gas defaults are not Singapore supply-chain data.

IEA 2024 reports unabated natural-gas hydrogen at 10-12 kgCO2e/kgH2 and states that 75-95% occurs at the production site; upstream/midstream emissions remain important after CCS.

Therefore this project will use:

e_LCA =
e_plant-carbon
+ e_NG-upstream
+ e_nuclear
+ e_CCS-transport-storage
+ e_auxiliaries.

## Nuclear lifecycle term

UNECE's 2022 integrated LCA reports nuclear electricity around 5.1-6.4 gCO2e/kWh across its modelled regions, with a global-average value around 5.5 gCO2e/kWh.

This provides a sensitivity anchor for nuclear-electric lifecycle emissions.

It cannot yet be transferred directly to **nuclear process heat** because the functional output and allocation basis differ. Direct-heat lifecycle emissions therefore remain a separate parameter until a defensible thermal allocation is defined.

## Current conclusion

At this stage:

- plant-gate carbon: identical for D and E under matched chemistry;
- reactor thermal requirement: materially different due to power-cycle conversion;
- lifecycle nuclear emissions: expected to differ because the architectures require different reactor-energy throughput, but the direct-heat allocation method remains open;
- upstream NG and CCS transport/storage: identical under the first controlled comparison.

This is the correct baseline from which to test whether process intensification or altered tail-gas recycle makes the real h-SMR and e-SMR configurations diverge.
