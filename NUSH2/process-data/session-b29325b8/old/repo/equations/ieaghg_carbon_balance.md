# IEAGHG reference carbon balance and nuclear-heat implication

## Source basis

IEAGHG 2017-02 publishes a heat and material balance for a 100,000 Nm3/h H2 standalone SMR plant.

Base-case streams used here:

| Stream | Flow | Relevant composition |
|---|---:|---|
| NG feedstock | 1455.8 kmol/h; 26,231 kg/h | 2.00% CO2, 89.00% CH4, 7.00% C2H6, 1.00% C3H8, 0.10% n-C4H10, 0.01% n-C5H12 |
| NG make-up fuel | 240.4 kmol/h; 4,332 kg/h | same NG composition |
| PSA inlet | 6596.9 kmol/h | 16.27% CO2, 4.64% CO, 3.02% CH4 |
| PSA tail gas | 2106.3 kmol/h | 50.95% CO2, 14.54% CO, 9.45% CH4, 23.69% H2 |
| Flue gas | 8659.4 kmol/h | 21.23% CO2 |

IEAGHG's technical review independently explains the carbon pathway: about 60% of total CO2 production is present in shifted gas; after PSA it enters the tail gas. The tail gas contains roughly 15% CO and 9% CH4 and supplies most reformer fuel. The remaining ~40% of final CO2 arises from combustion of make-up fuel plus CO/CH4 in the tail gas.

## Carbon-atom balance

For the published natural gas mixture, carbon atoms per mole of NG mixture are:

C_NG = 0.0200 + 0.8900 + 2(0.0700) + 3(0.0100)
       + 4(0.0010) + 5(0.0001)
     = 1.0845 mol-C/mol-NG.

Feedstock carbon:

C_feed = 1455.8 * 1.0845
       = 1578.56 kmol-C/h.

Make-up furnace-fuel carbon:

C_fuel = 240.4 * 1.0845
       = 260.72 kmol-C/h.

Total carbon input:

C_in = 1839.28 kmol-C/h.

Published flue-gas carbon:

C_flue = 8659.4 * 0.2123
       = 1838.39 kmol-C/h.

Closure:

C_flue / C_in = 0.9995.

The discrepancy is about 0.05%, consistent with rounded published stream values. This is a successful independent carbon-balance reconstruction of the published reference case.

## The critical PSA result

Before combustion, carbon in the PSA tail gas is:

C_tail = 2106.3 * (0.5095 + 0.1454 + 0.0945)
       = 1578.46 kmol-C/h.

Therefore:

C_tail / C_feed = 0.99994.

**Essentially all feedstock carbon entering the reference hydrogen plant reaches the PSA tail gas before it is burned in the reformer furnace.**

This quantitatively explains why deleting the furnace from a process diagram does not delete feedstock carbon.

## How much carbon does nuclear heat directly displace?

The separately supplied make-up NG contributes:

260.72 / 1839.28 = 14.18%

of total incoming carbon.

Thus a hypothetical substitution that only removes the make-up NG fuel while leaving the feedstock chemistry and PSA unchanged directly eliminates only this ~14% input-carbon source.

This is NOT the maximum possible emissions reduction from nuclear integration. Nuclear heat may enable a redesigned capture/recycle system. It is instead a falsification check against the naive assumption that replacing furnace heat automatically removes all SMR carbon emissions.

## What if CO2 is removed from shifted syngas?

At the published PSA inlet:

CO2 carbon = 6596.9 * 0.1627 = 1073.3 kmol-C/h.

But CO + CH4 carbon remains:

C_CO+CH4 = 6596.9 * (0.0464 + 0.0302)
         = 505.3 kmol-C/h.

So even perfect removal of the existing shifted-gas CO2 would leave roughly 505 kmol-C/h in CO and CH4 at this point, before considering process redesign.

A nuclear-heated configuration therefore needs an explicit strategy for this residual carbon and the H2-rich PSA off-gas.

## Design implication

The nuclear-assisted process should no longer be represented as merely:

HTGR -> reformer heat -> PSA -> H2.

A mass-conserving architecture must include a tail-gas treatment/recycle branch. Candidate hypotheses to test are:

A. shifted-syngas CO2 capture + PSA + recycle/treatment of tail gas;
B. PSA-tail-gas CO2 capture + recovery/use of H2/CO/CH4;
C. deeper shift/conversion followed by CO2 separation and H2 purification;
D. alternative membrane/adsorption architecture that maximizes H2 and carbon recovery.

No option is selected yet.

## Next model requirement

The next model must resolve H, C and O species around reformer/WGS/PSA rather than using only aggregate CO2 intensity. It must reproduce the published IEAGHG stream table within tolerances before nuclear heat is introduced.
