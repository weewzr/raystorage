# Furnace-convection feed-preheat lower bound

## Source sequence

IEAGHG 2017-02 states:

1. raw NG is first heated to 135 C by cooled shifted syngas;
2. feedstock NG is then heated to 370 C in the fired-reformer **Feed Pre-Heater Coil**;
3. purified feedstock is mixed with HP superheated steam;
4. that mixture is heated in the **Pre-Reformer Feed Pre-Heater Coil** before the ~500 C pre-reformer;
5. pre-reformer product receives a second HP steam addition and is further heated in the **Reformer Pre-Heater Coil** before entering the radiant reformer.

Therefore only step 1 is clearly supplied by retained shifted-syngas heat recovery. Steps 2, 4 and 5 are furnace-convection services in the reference plant.

## First rigorous lower bound: feed-preheater methane only

The published NG feedstock is 1455.8 kmol/h and 89 mol% CH4.

For the furnace Feed Pre-Heater Coil:

T1 = 135 C = 408.15 K
T2 = 370 C = 643.15 K.

NIST Chemistry WebBook (SRD 69, Chase 1998) gives methane Shomate coefficients for 298-1300 K:

Cp = A + Bt + Ct^2 + Dt^3 + E/t^2

and

H(T)-H(298.15)
= At + Bt^2/2 + Ct^3/3 + Dt^4/4 - E/t + F - H,

with t=T/1000.

Using the NIST coefficients gives:

Delta h_CH4(408.15 -> 643.15 K)
= 11.2453 kJ/mol.

Methane flow:

n_CH4 = 1455.8*0.89
       = 1295.7 kmol/h.

Thus the methane contribution alone is:

Q_CH4
= n_CH4 Delta h_CH4
= approximately 4.05 MW.

This is a **strict partial lower bound**, not the complete Feed Pre-Heater Coil duty, because it omits:
- CO2;
- ethane;
- propane;
- butane/pentane;
- nitrogen/inerts;
- recycled H2 that IEAGHG says is mixed with feedstock after the first feed preheater.

All of those sensible-heating contributions are positive over 135->370 C.

## Updated quantified floor

Previously source-constrained:
- radiant reformer duty: 96.04 MW;
- HP steam superheat: ~16.0-16.25 MW.

Adding only the methane portion of the furnace feed-preheater duty raises the currently demonstrated floor to:

Q_furnace,known > approximately 116 MW.

This is still deliberately conservative/incomplete.

## Remaining convection duties

Still not quantified:
- non-methane + recycled-H2 part of Feed Pre-Heater Coil;
- Pre-Reformer Feed Pre-Heater Coil;
- Reformer Pre-Heater Coil;
- furnace Steam Generation Coil share.

The next property-model extension should add NIST Shomate functions for the remaining NG/H2/H2O species and use the published source-stream temperatures. Where an inlet temperature is not explicit, the model must report a sensitivity range rather than invent a point value.

## Independent contextual evidence

Published SMR furnace design work shows why the convection section is energetically important: a 200 MMSCFD hydrogen-plant example reports total absorbed furnace duty around 1322 MMBtu/h versus fired duty 1452 MMBtu/h, and modifying convection/pre-reforming changes both fuel use and steam export. This is not used numerically for the IEAGHG plant because it is a different design; it is retained only as evidence that convection-service changes materially affect overall heat integration.
