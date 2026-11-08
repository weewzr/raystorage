# PSA species reconstruction and tail-gas design space

## Status

This note reconstructs the C/H-bearing PSA inlet and tail-gas streams from the published IEAGHG 2017-02 base-case heat/material balance. It is a source-data reconstruction, not yet a predictive PSA adsorption model.

## Published stream reconstruction

For PSA inlet total flow 6596.9 kmol/h:

- H2 = 75.07% -> 4952.5 kmol/h
- CO2 = 16.27% -> 1073.3 kmol/h
- CO = 4.64% -> 306.1 kmol/h
- CH4 = 3.02% -> 199.2 kmol/h

For PSA tail gas total flow 2106.3 kmol/h:

- H2 = 23.69% -> 499.0 kmol/h
- CO2 = 50.95% -> 1073.1 kmol/h
- CO = 14.54% -> 306.3 kmol/h
- CH4 = 9.45% -> 199.0 kmol/h

The near identity of CO2, CO and CH4 molar flows across the PSA is expected for a hydrogen-selective PSA: those species report overwhelmingly to the tail gas.

Reconstructed H2 product:

4952.5 - 499.0 = approximately 4453.5 kmol/h.

Reconstructed H2 recovery:

4453.5 / 4952.5 = approximately 89.9%.

This independently reproduces the approximately 90% PSA H2-recovery basis from rounded source values.

## Tail gas is not simply a CO2 waste stream

On a molar basis, H2 + CO + CH4 comprise:

23.69% + 14.54% + 9.45% = 47.68%

of the tail gas.

Therefore almost half of the tail-gas molecules in this reference case are combustible species. Removing the furnace creates both a carbon-management problem and an energy-recovery problem.

## CO2-only capture does not solve all tail carbon

Tail carbon is distributed between:

- CO2: 50.95 mol% of the total tail stream;
- CO: 14.54%;
- CH4: 9.45%.

On a carbon-atom basis:

fraction as CO2
= 0.5095 / (0.5095 + 0.1454 + 0.0945)
= approximately 68.0%.

Thus a hypothetical perfect CO2-only separator on the unmodified tail gas would still leave about 32.0% of tail-gas carbon in CO + CH4.

This suggests that a nuclear-heated architecture should test additional conversion/recovery rather than assuming a CO2 absorber alone closes the carbon problem.

## Candidate architecture hypotheses for the next comparison

### T0 — Conventional combustion
PSA tail gas + make-up NG -> fired reformer.
This is the reference configuration.

### T1 — Tail-gas CO2 removal then combustion/use of remaining fuel
Remove CO2 from PSA tail gas, retain H2/CO/CH4 as a combustible stream. If nuclear heat already supplies the reformer duty, a useful destination for this fuel must be identified. Burning it elsewhere still produces CO2 from CO/CH4.

### T2 — Additional shift / oxidation-state conversion before separation
Convert residual CO toward CO2 and H2, then separate CO2. CH4 remains a separate conversion/recycle problem.

### T3 — H2 recovery + carbon-species recycle
Increase H2 recovery and recycle CH4/CO toward the reforming/shift section. This could reduce carbon discharge but raises recycle, separation and equilibrium constraints.

### T4 — Alternative membrane/adsorption sequence
Use a purification architecture selected around the nuclear-heated flowsheet rather than preserving the conventional PSA topology by default.

No architecture is selected yet.

## Acceptance criterion before HTGR integration

The reduced model must:
1. reproduce the source PSA inlet and tail species flows within rounding tolerance;
2. reproduce approximately 90% H2 recovery;
3. close carbon through feed -> shifted gas -> PSA tail gas;
4. expose the residual carbon and combustible energy in every candidate tail-gas treatment;
5. avoid treating captured CO2, converted carbon and avoided CO2e as interchangeable quantities.

The next scientific task is to extend upstream from the PSA inlet to reformer/WGS reaction extents and steam balance.
