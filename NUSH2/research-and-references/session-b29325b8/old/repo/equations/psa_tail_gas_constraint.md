# PSA tail gas and the nuclear-furnace substitution problem

## Why this matters

The original concept proposes replacing the fired SMR furnace with high-temperature nuclear heat. A conventional SMR furnace, however, is not fuelled only by purchased natural gas.

Published descriptions show that the PSA tail gas contains unrecovered H2 plus CH4, CO and CO2 and is normally used as the primary reformer-furnace fuel; make-up natural gas is added as required.

Therefore:

**removing the fired reformer does not automatically remove the carbon contained in PSA tail gas.**

The integrated nuclear design must explicitly decide what happens to that stream.

## Reference evidence

IEAGHG 2017-02 reports for its 100,000 Nm3/h H2 base case:
- natural gas feedstock: 26.231 t/h;
- separately supplied natural-gas fuel: 4.332 t/h;
- total NG: 30.563 t/h;
- H2: 8.994 t/h;
- direct CO2: 0.8091 kg/Nm3 H2;
- exported electricity: 9.918 MWe.

Source:
IEAGHG, *Techno-Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS*, 2017-02.
https://ieaghg.org/publications/techno-economic-evaluation-of-smr-based-standalone-merchant-hydrogen-plant-with-ccs/

A peer-reviewed SMR study describes PSA recovery near 90% and states that the PSA tail gas, containing CH4, CO and CO2 (and unrecovered H2), is used as primary reformer-furnace fuel. The furnace also recovers high-temperature flue-gas sensible heat for feed preheating and steam generation.

Source:
Herraiz et al., *Sequential Combustion in Steam Methane Reformers for Hydrogen and Power Production With CCUS in Decarbonized Industrial Clusters*, Frontiers in Energy Research 8 (2020), 180.
https://doi.org/10.3389/fenrg.2020.00180

## Screening calculation

IEAGHG's separately reported make-up NG fuel is:

4.332 / 8.994 = 0.4817 kg NG/kg H2.

If, only for a chemical upper-bound screening calculation, that fuel is treated as pure methane, complete combustion gives:

CO2/CH4 = 44.0095 / 16.04246 = 2.743 kg/kg,

hence:

0.4817 * 2.743 = approximately 1.32 kg CO2/kg H2.

This is **not** the total furnace emission. It only estimates the carbon associated with the separately supplied make-up NG fuel under a pure-CH4 approximation.

The IEAGHG base case emits about 9.00 kg CO2/kg H2 on the same H2 mass basis. The difference cannot be assigned to "process CO2" by subtraction because PSA tail gas recycles feed-derived carbon into the furnace.

## Consequence for the proposed nuclear architecture

A defensible nuclear-assisted flowsheet needs at least four tail-gas alternatives:

1. burn tail gas in a separate heater/boiler;
2. recover additional H2 and/or recycle CH4/CO;
3. capture CO2 from tail gas before using remaining combustible species;
4. use a different purification/integration architecture that changes the tail-gas composition.

Each option changes:
- hydrogen recovery;
- methane conversion;
- reformer heat balance;
- CO2 capture fraction;
- utility generation;
- equipment count;
- cost.

## Falsification condition

If nuclear heat removes only the 4.332 t/h make-up NG fuel while the PSA tail gas is still combusted elsewhere without capture, the achievable direct-emissions benefit may be far smaller than suggested by simply deleting the furnace icon from the original process diagram.

The next integrated model must therefore conserve carbon through the PSA and tail-gas disposition explicitly.
