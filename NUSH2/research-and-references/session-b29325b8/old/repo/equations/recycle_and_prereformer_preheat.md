# Hydrogen recycle and pre-reformer feed-preheat bound

## Recovered source evidence

The IEAGHG base-case heat/material balance explicitly reports stream 13:

- description: H2 Recycle;
- temperature: 40 C;
- pressure: 2.51 MPa;
- molar flow: 29.1 kmol/h;
- mass flow: 59 kg/h;
- composition: >99.99 mol% H2.

The equipment list independently gives the recycle-hydrogen compressor package as approximately 653 Nm3/h, 2.505 -> 4.001 MPa, 16 kW.

This resolves the previously missing recycle flow.

## Feed-preheater implication

IEAGHG states that the feedstock NG is mixed with recycled H2 before feedstock pretreatment, and the preheated feedstock is raised to 370 C in the furnace Feed Pre-Heater Coil.

Using NIST SRD 69 H2 Shomate properties, the recycle stream contributes only about 0.08 MW of sensible heating from its published 40 C state to 370 C.

Therefore the complete known NG + recycle-H2 contribution to this coil is approximately:

Q_feed-preheater ~= 4.82 MW + 0.08 MW ~= 4.9 MW.

The recycle correction is small but is now explicit and source-traceable.

## Pre-reformer Feed Pre-Heater Coil

The source table reports stream 4, Purified Feedstock to Pre-reformer:

- 500 C;
- 3.39 MPa;
- 5514.0 kmol/h;
- H2O 73.07 mol%;
- CH4 23.50%;
- H2 0.53%;
- CO2 0.53%;
- C2H6 1.85%;
- C3H8 0.26%;
- n-C4H10 0.03%;
- N2 0.23%.

Thus the outlet state of the Pre-Reformer Feed Pre-Heater Coil is source-resolved.

The exact inlet state is complicated by the source-described BFW injection in the Pre-Reformer Steam De-superheater. The BFW injection flow/state is not separately numbered in the summary heat/material balance.

Therefore the model currently calculates a **lower-bound coil duty** by assuming:
- NG + recycled H2 enter at 370 C;
- all stream-4 water enters as already-superheated steam at 400 C;
- outlet is the published 500 C stream 4.

This is conservative because any BFW desuperheating lowers inlet enthalpy and increases the required furnace-coil heat.

Using NIST species enthalpies, the resulting lower-bound duty is of order tens of MW and is regression-bounded in Rust pending exact reporting from CI.

## Reformer Pre-Heater Coil

The standalone IEAGHG report states that pre-reformer product receives a smaller second HP-steam stream and is then heated to the required primary-reformer inlet temperature, but the summary stream table does not expose that intermediate temperature.

A related IEAGHG 2017 industrial-complex study describes an analogous pre-reformed stream being heated in a Reformer Preheat Coil to approximately 600-650 C before primary reforming.

That 600-650 C range is **external corroborating context, not a substituted value for the standalone reference plant**. The standalone model will retain reformer-inlet temperature as an explicit sensitivity until a source-specific state is recovered.

## Research implication

The direct-nuclear-heat service floor is rising because the fired furnace supplies a cascade of high- and medium-temperature services, not only reaction heat.

However, the same cascade is favorable to secondary helium: helium leaving the high-temperature reformer can potentially supply progressively lower-temperature preheat/steam services before returning to the IHX.

The final heat-integration model should therefore test serial/cascade heat use rather than summing every coil duty as if each required fresh 900 C helium.
