# 80% recycle operating-energy economic screen v2

## Purpose

Propagate the source-bounded recycle thermal requirement into annual real operating-energy economics before imposing HTGR CAPEX/fixed O&M.

This is not a final abatement-cost result.

## Physical basis

80% reduced recycle:
- fresh-feed NG energy displaced: ~90.0 MW_LHV;
- purchased supplementary furnace NG removed: 55.94 MW_LHV;
- combined NG resource displacement: ~145.95 MW_LHV.

Source-bounded reactor-side process-service sensitivity:
- approximately 145-179 MWth before nuclear-loop thermal losses.

Tail-gas separation anchor:
- 6.309 MWe.

Operating hours:
- 8322 h/y.

## Nuclear-heat price anchor

JAEA's HTGR hydrogen economic study used:
- nuclear heat: 0.7 JPY/MJ;
- electricity: 5.8 JPY/kWh.

This is a legacy Japanese study assumption, not a current Singapore nuclear-heat tariff.

Using late-Sep-2026 mid-market FX of approximately:

1 JPY ~= 0.00813 SGD,

0.7 JPY/MJ corresponds numerically to approximately:

5.69 SGD/GJ.

No inflation adjustment is applied here. The conversion is retained only as a transparent technology-cost anchor/sensitivity.

## Electricity sensitivity

Use:
150 SGD/MWh

for the 6.309 MWe separation load in this screen.

This is a sensitivity, not a nuclear-electricity cost claim.

Annual separation-electricity cost:
~S$7.88m/y.

## Gross NG savings

At combined ~145.95 MW displacement:

| NG price | Gross annual NG saving |
|---:|---:|
| S$10/GJ | ~S$43.73m/y |
| S$15/GJ | ~S$65.59m/y |
| S$20/GJ | ~S$87.45m/y |

## Operating-energy net before CAPEX/O&M

Charge the full 145-179 MWth process-service envelope at 5.69 SGD/GJ and subtract the 6.309 MWe separation electricity.

| NG price | Net operating-energy value |
|---:|---:|
| S$10/GJ | ~S$5.34-11.13m/y |
| S$15/GJ | ~S$27.20-33.00m/y |
| S$20/GJ | ~S$49.06-54.86m/y |

Positive means operating-energy savings remain before HTGR/IHX/recycle/capture CAPEX and fixed O&M.

## Break-even gas price

For the legacy 5.69 SGD/GJ nuclear-heat anchor and S$150/MWh separation electricity, the gas price needed merely to break even on operating energy is approximately:

- 145 MWth service: S$7.45/GJ;
- 162 MWth midpoint: S$8.12/GJ;
- 179 MWth service: S$8.78/GJ.

This does not imply project break-even. CAPEX/fixed O&M are still zero in this diagnostic.

## Compare with required savings gap

Previous Singapore-adjusted conventional CCS screen indicated ~S$17-29m/y net savings would be needed to move the fixed-denominator comparator toward S$100/t.

Under the legacy nuclear-heat anchor:
- S$10/GJ gas: operating-energy savings alone are insufficient;
- S$15/GJ: operating-energy savings are of comparable magnitude to / above the gap before CAPEX;
- S$20/GJ: substantial pre-CAPEX headroom exists.

Thus the concept is not yet economically falsified, but the feasible region is conditional on:
1. gas value being sufficiently above delivered nuclear-heat cost;
2. the lower-to-middle HTGR service envelope;
3. affordable allocated nuclear/integration CAPEX;
4. CCS T&S not consuming the remaining budget.

## Most important limitation

The 0.7 JPY/MJ JAEA heat price is an old design-study assumption. Converting it at current FX does not make it a 2026 Singapore nuclear-heat price.

The next economic step must therefore solve the inverse problem:

**What maximum delivered nuclear-heat price and allocated annualised HTGR cost are compatible with S$100/tCO2e across the NG-price and CCS-T&S ranges?**

That break-even surface is more defensible than selecting a speculative Singapore nuclear heat price.
