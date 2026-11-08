# Source-resolved pre-reformer / reformer / HTS reconstruction

## Corrected source interpretation after Independent Review 1

The earlier repository wording treated IEAGHG stream 4 too loosely as a generic
"pre-reformer feed" and correctly refused to force stream 4 -> stream 5 to close.
The full IEAGHG 2017-02 process description resolves the reason for that
non-closure.

Published stream 4 is explicitly **purified feedstock to the pre-reformer** at
500 C. It is not the pre-reformer outlet and it is not the primary-reformer
inlet.

IEAGHG states that:
1. purified natural gas is mixed with HP superheated steam before the
   pre-reformer;
2. the pre-reformer converts C2+ hydrocarbons toward CH4, CO2, CO and H2;
3. the pre-reformer product is then mixed with a **second, smaller HP
   superheated-steam stream** to fine-tune steam/carbon ratio;
4. BFW injection is also used for reformer-feed temperature control;
5. the mixture is preheated and enters the radiant primary reformer;
6. primary-reformer syngas normally leaves at about 900-950 C;
7. the reformer waste-heat boiler cools it to 320 C before the HTS;
8. published stream 5 is the **HTS reactor inlet**.

Source: IEAGHG 2017-02, process description pp. 34-36 and base-case heat/material
balance pp. 43-44.

Therefore streams 4 and 5 do not by themselves define a closed two-stream
control volume. They bracket unnumbered water/steam additions plus the
pre-reformer, primary reformer and reformer waste-heat boiler.

## Source states

| State | T (C) | P (MPa) | Flow (kmol/h) | Key composition |
|---|---:|---:|---:|---|
| stream 4: purified feedstock + first steam, to pre-reformer | 500 | 3.39 | 5514.0 | H2O 0.7307, CH4 0.2350, C2H6 0.0185, C3H8 0.0026 |
| stream 5: HTS inlet after reformer + waste-heat boiler | 320 | 2.80 | 8370.3 | H2 0.5171, H2O 0.2927, CO 0.1156, CO2 0.0492, CH4 0.0238 |
| HTS outlet | 412 | 2.77 | 8370.3 | H2 0.5961, H2O 0.2137, CO 0.0366, CO2 0.1283, CH4 0.0238 |
| PSA inlet | 35 | 2.58 | 6596.9 | H2 0.7563, CO2 0.1627, CO 0.0464, CH4 0.0302 |

## Elemental diagnosis of the missing interstage stream

Using the rounded source compositions directly:

- stream-4 carbon ~= 1578.658 kmol-C/h;
- stream-5 carbon ~= 1578.639 kmol-C/h;
- carbon residual ~= -0.020 kmol-C/h, i.e. effectively closed at source-table
  rounding precision.

The H and O residuals are positive at stream 5:
- delta H ~= +310.34 kmol-H-atoms/h;
- delta O ~= +153.70 kmol-O-atoms/h.

If the omitted material is H2O, the independent estimates are:

n_H2O,H = delta H / 2 ~= 155.17 kmol/h

n_H2O,O = delta O ~= 153.70 kmol/h.

Their small difference is consistent with four-decimal composition rounding.
A least-squares reconciliation gives approximately **154.4 kmol/h H2O**.

This is not a claimed separately published utility flow. It is a source-table
reconstruction of the aggregate unnumbered water/steam addition, physically
consistent with IEAGHG's explicit second HP-steam and BFW additions.

With that reconstructed water addition, C/H/O close to much better than 0.1%
of the stream-4 elemental inventories without changing any published carbon
species values.

## Consequence

The Independent Review 1 blocker concerning the *material* control-volume
interpretation is therefore resolved: the previous non-closure was not evidence
of missing carbon chemistry. It was caused by comparing stream 4 and stream 5
while omitting explicitly described, unnumbered interstage water/steam addition.

This does **not** close the energy-balance blocker. IEAGHG does not publish a
numbered primary-reformer outlet state in the same heat/material table. The
900-950 C reformer outlet, second-steam split, BFW split, individual coil duties
and reformer waste-heat duty must still be reconstructed/benchmarked before
nuclear heat is substituted.

## HTS verification

For CO + H2O -> CO2 + H2, reaction extent is reconstructed independently from
CO consumption, CO2 production, H2 production and H2O consumption. All four
give approximately 661 kmol/h within source rounding tolerance. The Rust model
asserts C/H/O closure across the HTS section.

## Steam-to-carbon interpretation

The stream-4 water/carbon ratio reconstructs to approximately 2.55. It is not
the overall design steam/carbon ratio because stream 4 is downstream of the
first steam mixing/desuperheating step and upstream of the explicitly described
second steam addition. IEAGHG states an overall steam/carbon ratio around
2.7-2.8 mol/mol. The repository must retain these as different quantities rather
than forcing one to equal the other.

## Process-temperature hierarchy

- pre-reformer feed: 500 C;
- primary-reformer product: normally about 900-950 C;
- reformer waste-heat boiler cools syngas to 320 C;
- HTS outlet: 412 C;
- PSA inlet: 35 C after downstream cooling/condensation.

A future nuclear model must therefore distinguish high-grade reformer heat,
medium-grade preheat, steam generation, recoverable syngas heat and
low/medium-grade CCS duties.

## Next derivation

The next critical task is the **energy** reconstruction: quantify the primary
reformer/radiant duty and convective/steam-generation duties from authoritative
performance data and thermodynamic balances. Nuclear heat must not be attached
to the existing 298 K reaction-only duty.
