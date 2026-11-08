# Inverse S$100/t economic feasibility surface v1

## Why invert the economics

A Singapore nuclear-heat tariff is not publicly established. Choosing one as a central value would create false precision.

Instead solve the assignment threshold for:
1. maximum delivered nuclear-heat price;
2. maximum annual hydrogen-side HTGR/IHX/integration cost;
3. total common-reactor annual-cost headroom under dedicated versus cogeneration allocation.

## Governing equations

Let:
- B = allowed annual incremental cost at the S$100/t abatement threshold;
- S_NG = annual fresh-feed + supplementary-furnace NG saving;
- C_e = tail-gas separation electricity;
- C_TS = CCS transport/storage;
- C_other = other fixed incremental annual costs;
- Q_N = annual delivered nuclear heat [GJ/y].

Then:

p_N,max =
(B + S_NG - C_e - C_TS - C_other)/Q_N.

At an assumed delivered nuclear-heat price p_N, hydrogen-side annual capital/fixed-O&M headroom is:

C_cap,H2,max =
B + S_NG - C_e - C_TS - p_N Q_N.

If common reactor cost is allocated with fraction f:

C_common,max = C_cap,H2,max/f.

Dedicated:
f = 1.

GTHTR300C-like thermal-share cogeneration anchor:
f = 170/600 = 0.2833.

## Screening basis

Use the existing illustrative assignment-scale case:
- annual allowed incremental-cost budget B ~= S$54m/y;
- 80% recycle combined NG displacement ~=145.95 MW_LHV;
- separation electricity = 6.309 MWe;
- electricity sensitivity = S$150/MWh;
- HTGR process service = 145 / 162 / 179 MWth;
- Group-A-like T&S annual costs ~=S$31.9m/y (USD50/t case) and S$47.85m/y (USD75/t case).

These are screening combinations, not final Singapore tariffs.

## Maximum delivered nuclear-heat price

Approximate maximum nuclear-heat price [S$/GJ] compatible with the annual budget when other fixed incremental costs are temporarily set to zero:

### Low Group-A T&S (~S$31.9m/y)

| NG price | 145 MWth | 162 MWth | 179 MWth |
|---:|---:|---:|---:|
| S$10/GJ | ~12.8 | ~11.5 | ~10.4 |
| S$15/GJ | ~17.8 | ~15.9 | ~14.4 |
| S$20/GJ | ~22.8 | ~20.4 | ~18.5 |

### High Group-A T&S (~S$47.85m/y)

| NG price | 145 MWth | 162 MWth | 179 MWth |
|---:|---:|---:|---:|
| S$10/GJ | ~9.1 | ~8.1 | ~7.4 |
| S$15/GJ | ~14.1 | ~12.6 | ~11.4 |
| S$20/GJ | ~19.1 | ~17.1 | ~15.5 |

These values are analytical screening boundaries. Any additional capture/integration fixed cost lowers them.

## Capital headroom at the legacy JAEA heat anchor

Use the legacy JAEA nuclear-heat assumption translated numerically to ~S$5.69/GJ.

For the 162 MWth midpoint:

### NG S$10/GJ
Operating-energy net before T&S/capital is only order single-digit millions per year. Once Group-A T&S is charged, little/no hydrogen-side capital headroom remains.

### NG S$15/GJ
Operating-energy net is order ~S$30m/y. Low Group-A T&S leaves only modest hydrogen-side annualised capital/fixed-O&M headroom; high Group-A T&S can erase most/all of it.

### NG S$20/GJ
Substantial operating-energy value remains and a meaningful annualised-capital region survives even after Group-A T&S, especially at the low T&S end.

## Dedicated versus cogeneration

If hydrogen-side annual headroom is C_H2:

Dedicated:
C_common,max = C_H2.

Cogeneration thermal-share anchor:
C_common,max ~= 3.53 C_H2.

This does not create free value. The non-hydrogen product must genuinely support the remaining common-reactor cost.

Therefore cogeneration materially enlarges the feasible reactor-cost region, but cannot rescue a case whose hydrogen-side headroom is already negative before allocation.

## Main falsification condition

For any defensible scenario, if:

p_N,credible > p_N,max

or:

annualised allocated HTGR/IHX/integration cost > C_cap,H2,max,

then that scenario fails the S$100/t criterion.

The next step is to place literature-derived HTGR annualised-cost ranges onto this boundary rather than continuing to manipulate the boundary itself.
