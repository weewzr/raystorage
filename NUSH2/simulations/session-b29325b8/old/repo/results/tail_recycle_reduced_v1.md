# Reduced fixed-H2 tail-recycle model v1

## Purpose and model class

This model replaces the previous carbon-only displacement ceiling with a reduced stoichiometric fixed-H2 screen.

It is still **not** a full equilibrium/recycle flowsheet. Its purpose is to determine whether Configuration B remains economically material after:
- source PSA H2 recovery;
- distinct H2, CO and CH4 chemistry;
- additional water consumption;
- same-scale Case-2A separation electricity.

## Source/reconstructed anchors

Base PSA H2 recovery reconstructed from the published inlet/tail streams:

eta_PSA ~= 0.900.

Tail inventory:
- H2 ~= 499 kmol/h;
- CO ~= 306 kmol/h;
- CH4 ~= 199 kmol/h.

Baseline reconstructed H2 product is used to define product H2 per kmol fresh NG.

## Reduced equations

For assumed conversion/recovery fractions:

x_CO, x_CH4, x_H2 in [0,1],

recoverable H2 is approximated by:

n_H2,recovered =
x_H2 n_H2,tail
+ eta_PSA [x_CO n_CO,tail + 4 x_CH4 n_CH4,tail].

The corresponding fresh-NG displacement at fixed H2 output is:

n_NG,displaced =
n_H2,recovered /
(n_H2,product / n_NG,base).

Additional stoichiometric water consumed is:

n_H2O,extra =
x_CO n_CO + 2 x_CH4 n_CH4.

This ignores equilibrium, altered PSA composition, pressure effects and recycle convergence.

## Sensitivity results

Using equal H2-recovery/CO-conversion/CH4-conversion sensitivities:

### 50% / 50% / 50%
- recovered H2 equivalent: ~746 kmol/h;
- fresh NG displaced: ~242 kmol/h;
- fresh-feed reduction: ~16.6%;
- fresh-feed energy displaced: ~56.3 MW;
- additional stoichiometric water: ~352 kmol/h.

### 80% / 80% / 80%
- recovered H2 equivalent: ~1193 kmol/h;
- fresh NG displaced: ~387 kmol/h;
- fresh-feed reduction: ~26.6%;
- fresh-feed energy displaced: ~90.0 MW;
- additional stoichiometric water: ~563 kmol/h.

### Ideal 100% screen
- recovered H2 equivalent: ~1491 kmol/h;
- fresh NG displaced: ~483 kmol/h;
- fresh-feed reduction: ~33.2%;
- fresh-feed energy displaced: ~112.5 MW;
- additional stoichiometric water: ~704 kmol/h.

The ideal fresh-NG reduction can slightly exceed the earlier ~32% carbon-only ceiling because recycling existing tail H2 can displace fresh NG without adding recycled carbon. The earlier 32% value remains specifically a **carbon-equivalent displacement ceiling**, not a total fresh-NG displacement ceiling once H2 recovery is credited.

## Partial economic screen

Using the 80% sensitivity:
- fresh-feed energy displaced ~=90.0 MW;
- annual energy ~=2.70 million GJ/y at 8322 h/y.

Case-2A electrical anchor:
W_sep ~=6.309 MWe.

At an illustrative S$150/MWh electricity value:
annual separation-electricity cost ~=S$7.88m/y.

Partial net operating value, excluding solvent steam, recycle-specific compression changes, extra reformer heat and CAPEX:

| NG price | Gross feed saving | Case-2A electricity | Partial net |
|---:|---:|---:|---:|
| S$10/GJ | S$27.0m/y | S$7.9m/y | S$19.1m/y |
| S$15/GJ | S$40.4m/y | S$7.9m/y | S$32.6m/y |
| S$20/GJ | S$53.9m/y | S$7.9m/y | S$46.1m/y |

These are **optimistic partial-net sensitivities**, not project savings.

They exclude:
- ~66.9 t/h Case-2A LP-steam regeneration demand;
- changed recycle compression;
- increased reformer/shift heat from recycled CH4/CO;
- altered capture/compression mass;
- equipment CAPEX/O&M;
- iterative recycle effects.

## Scientific implication

The 80% reduced model retains enough of the theoretical recycle opportunity that tail-gas integration remains first-order economically.

At S$10/GJ and S$150/MWh, the partial net (~S$19m/y) is already comparable to the lower ~S$17m/y fixed-denominator gap identified previously.

But because omitted thermal/CAPEX penalties are material, this does **not** establish economic feasibility.

The next critical calculation is the added heat/steam duty of converting recycled CO/CH4 and regenerating the MDEA solvent. That will determine how much of the partial operating value survives.
