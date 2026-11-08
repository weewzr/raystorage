# Reformer-preheater temperature sensitivity

## Why a sensitivity is required

The standalone IEAGHG 2017-02 source resolves:
- stream 4 into the pre-reformer at 500 C;
- the HTS inlet downstream of primary reforming and heat recovery;
- the existence of a second HP-steam addition and a Reformer Pre-Heater Coil.

It does **not** expose the exact intermediate primary-reformer inlet temperature in the summary stream table.

Therefore this project does not assign a single source-derived value to that state.

## External contextual anchor

A related IEAGHG 2017 industrial-complex SMR study describes an analogous sequence in which pre-reformed gas is mixed with additional steam and heated in the Reformer Preheat Coil to approximately 600-650 C before the primary reformer.

That range is used only to define a sensitivity window. It is not relabelled as a value from the standalone reference plant.

## Sensitivity model

For screening:

Q_RPH(T_R,in)
= sum_i n_i [h_i(T_R,in)-h_i(500 C)].

The currently encoded sensitivity uses the source stream-4 CH4/CO2/H2/N2/H2O composition and omits C2+.

This is not a final reconstructed duty because:
- actual pre-reformer chemistry changes composition;
- a second HP-steam addition occurs before the coil;
- the exact standalone outlet temperature is unpublished in the summary table.

Its purpose is to quantify how strongly the total heat-service envelope depends on the missing temperature rather than silently choosing one.

The Rust model evaluates 600, 625 and 650 C and asserts monotonic increase.

## Gate rule

The reformer-preheater sensitivity may be used in uncertainty plots and bounding calculations, but it must not be presented as an authoritative IEAGHG coil duty.

A final nominal value requires either:
1. recovery of the exact standalone stream state from source material, or
2. an explicitly declared modelling assumption with uncertainty propagated through the conclusions.
