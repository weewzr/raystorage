# Shifted-syngas dew point and MDEA pinch v1

## Question

Can the large water-condensation latent heat between IEAGHG streams 6 and 7 actually supply an MDEA reboiler at ~160 C while maintaining a positive exchanger temperature approach?

## Source state

IEAGHG stream 6:
- T = 412 C;
- P = 2.77 MPa;
- y_H2O = 0.2137.

Ideal-mixture water partial pressure:

p_H2O = y_H2O P
       = 0.2137 * 2.77
       ~= 0.592 MPa.

## IAPWS phase-equilibrium model

The Rust model now implements the IAPWS-IF97 Region-4 saturation-temperature equation directly.

The implementation is independently regression-tested against the normal boiling point at 0.101325 MPa.

Using p_H2O ~=0.592 MPa gives an ideal-mixture stream-6 water dew point in the high-150 C range.

This is a first dew-point estimate:
- it assumes ideal gas fugacity;
- ignores pressure drop during cooling;
- ignores non-ideal multicomponent effects.

## Pinch implication

For an illustrative MDEA reboiler temperature of 160 C and:

DeltaTmin = 10 K,

the required minimum hot-stream temperature is:

T_hot,min = 170 C.

But:

T_dew,stream6 < 170 C.

Therefore bulk water condensation from stream 6 does **not** begin above the MDEA hot-side pinch under this screening condition.

Consequently the large condensation latent heat cannot simply be counted as usable reboiler heat at 160 C with 10 K approach.

## Physical interpretation

The stream has three relevant heat regions:

1. 412 C -> 170 C:
   temperature-feasible sensible heat for a 160 C reboiler.

2. 170 C -> dew point (~high 150s C):
   sensible heat below the chosen 10 K pinch and therefore not directly usable in a simple exchanger for the 160 C reboiler.

3. below dew point:
   large condensation latent heat, but released at approximately/below the reboiler temperature grade.

That latent heat remains valuable for lower-temperature duties:
- BFW/demi-water preheating;
- condensate heating;
- lower-temperature solvent systems;
- heat-pump assisted regeneration.

But it is not automatically available to conventional 160 C MDEA regeneration.

## Consequence for the external 63% benchmark

The external study reporting 63% direct post-shift/condensing-syngas contribution may have:
- a lower reboiler temperature;
- a different syngas pressure/water fraction;
- a smaller exchanger approach;
- direct process-to-solvent integration with different temperature glide;
- different capture solvent/process configuration.

Therefore 63% cannot be transferred to the IEAGHG stream without matching these conditions.

## Next calculation

The remaining property issue is to calculate the 412 -> 170 C sensible heat directly with water properties valid below 500 K.

The current NIST H2O Shomate correlation stops at 500 K, so the project should not extrapolate it.

Options:
1. implement an ideal-gas water Cp correlation valid through 443-500 K from an authoritative source;
2. use an IAPWS low-density vapor enthalpy formulation for water partial-pressure conditions;
3. bound the missing 226.85 -> 170 C sensible interval using authoritative Cp limits.

Once this interval is added, the maximum temperature-feasible MDEA heat can be quantified without relying on condensation.
