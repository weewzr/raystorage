# HTGR literature cost overlap with S$100/t boundary v1

## Correction to the inverse-boundary interpretation

The inverse equation is:

C_cap,H2,max =
B + S_NG - C_e - C_TS - p_N Q_N.

The allowed S$100/t abatement budget B is available **in addition to** net operating-energy savings.

For the illustrative midpoint:
- B = S$54m/y;
- NG = S$15/GJ;
- HTGR service = 162 MWth;
- nuclear heat anchor = S$5.69/GJ;
- separation electricity = S$150/MWh;
- low Group-A T&S = S$31.9m/y;

the hydrogen-side annualised capital/fixed-O&M headroom is approximately:

**S$52.2m/y**.

An earlier test/narrative expected only a few million/y and was wrong. CI exposed the mismatch; the equation was correct and the expectation has been fixed.

This headroom still excludes additional capture/integration fixed costs not already represented.

## Modern IAEA source cases

IAEA TECDOC 2075 reports:

### HTGR-200 + steam reforming
- 4 x 200 MWth = 800 MWth;
- NPP CAPEX = USD2.065b;
- NPP O&M = USD192m/y;
- hydrogen plant CAPEX = USD1.013b;
- hydrogen plant O&M = USD296m/y;
- H2 = 440 kt/y.

### MHR-T + steam reforming
- 4 x 600 MWth = 2400 MWth;
- NPP CAPEX = USD2.748b;
- NPP O&M = USD324m/y;
- hydrogen plant CAPEX = USD1.496b;
- hydrogen plant O&M = USD317m/y;
- H2 = 400 kt/y.

These are large multi-module source studies, not Singapore quotations.

## Reactor-only thermal-share allocation

Using the project's screening finance assumptions:
- discount rate = 8%;
- life = 25 y;
- CRF ~=0.09368;

annual common NPP cost is:

HTGR-200:
2065*CRF + 192
~= USD385m/y.

MHR-T:
2748*CRF + 324
~= USD581m/y.

Allocate only a 162 MWth hydrogen/process-heat share:

HTGR-200:
162/800 = 20.25%;
allocated NPP annual cost ~= **USD78m/y**.

MHR-T:
162/2400 = 6.75%;
allocated NPP annual cost ~= **USD39m/y**.

At an illustrative ~1.276 SGD/USD:
- HTGR-200 allocation ~= S$100m/y;
- MHR-T allocation ~= S$50m/y.

## Interpretation against midpoint headroom

Illustrative midpoint hydrogen-side headroom:
~S$52.2m/y.

Therefore, before adding project-specific IHX/secondary-loop/reformer-modification CAPEX:

- HTGR-200 thermal-share allocation: **does not fit** (~S$100m/y > S$52m/y).
- MHR-T thermal-share allocation: **approximately reaches the boundary** (~S$50m/y vs ~S$52m/y), leaving almost no room for integration costs.

This is not a final architecture verdict because:
1. source cases use different scale/configuration;
2. linear thermal-share allocation is an accounting screen, not a scaling law;
3. source O&M definitions may not map perfectly to Singapore;
4. FX/price-year harmonisation remains approximate;
5. the MHR-T source states reactor power for hydrogen production is only "part", so actual source allocation logic may differ;
6. dedicated reactor allocation is much more expensive;
7. cogenerated electricity/heat can carry common cost only if it has real market value.

## Dedicated-reactor implication

Charging the full common NPP annual cost to the hydrogen plant gives hundreds of millions USD/y in both IAEA cases.

That is far outside the current ~S$52m/y hydrogen-side headroom.

Thus a **dedicated large HTGR charged wholly to this single hydrogen train is economically incompatible with the current S$100/t screening boundary**.

A shared/cogeneration architecture remains potentially viable only at favourable combinations of:
- large reactor/shared output;
- low thermal allocation fraction;
- low Group-A CCS T&S;
- moderate/high NG value;
- low integration CAPEX.

## Scientific conclusion

The cost evidence now separates the architectures:

H_dedicated:
strongly disfavoured by the current screening boundary.

H_shared:
not yet falsified; the large MHR-T source case approximately overlaps the favourable midpoint boundary under simple thermal-share allocation, but with little integration-cost margin.

The next task is therefore not another generic HTGR cost search. It is to quantify IHX/secondary-loop/reformer-modification cost allowance and test how quickly that remaining shared-reactor overlap disappears.
