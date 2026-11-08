# Helium-loop scale and pumping benchmark

## Purpose

This note tests whether the ~80 kg/s secondary-helium flow obtained from the project's first screening calculation is physically outlandish or consistent with published large-scale HTGR heat-transport designs.

It does **not** select GTHTR300C as the Singapore reactor.

## Published large-scale benchmark

A recent review of JAEA HTGR hydrogen systems reports a GTHTR300C IHX design with approximately:

- IHX heat duty: 170 MWth;
- primary helium: 950 C inlet, 850 C outlet, ~324 kg/s;
- secondary helium: 500 C inlet, 900 C outlet;
- secondary helium flow: 81 kg/s;
- secondary helium pressure: 5.15 MPa;
- secondary-side IHX pressure loss: 58 kPa;
- heat-transfer area: 1448 m2.

The same source reports the HTTR IHX at 10 MWth and 3 kg/s secondary helium, illustrating the substantial scale-up between research-reactor and larger HTGR designs.

Source:
Energies 2025, 18, 4632, *Sustainable Hydrogen Production from Nuclear Energy*, summarising JAEA HTTR and GTHTR300C IHX designs.

## Comparison with project screening

The project independently obtained:

Q = 96.04 MW

and, using the temporary assumptions:

cp_He = 5.2 kJ/kg-K
T_hot = 880 C
T_cold = 650 C,

m_dot_He ~= 80 kg/s.

The numerical similarity to the published 81 kg/s GTHTR300C secondary flow is useful only as an order-of-magnitude plausibility check. The temperature spans and duties differ, so the two systems are not interchangeable.

## Pumping-power lower layer

For small pressure rise relative to absolute pressure, a first screening approximation is:

W_circ ~= m_dot * DeltaP / (rho * eta).

With ideal-gas helium:

rho ~= P/(R_He T).

Using the GTHTR300C IHX-only values:
- m_dot = 81 kg/s;
- DeltaP_IHX = 58 kPa;
- P = 5.15 MPa;
- T = 500 C;
- eta = 0.80 as an explicit screening assumption;

gives an IHX-only circulation requirement of order 1.8 MW.

This is **not total loop circulator power**.

The complete loop also includes:
- hot-gas duct/piping;
- reformer;
- steam generator/superheater;
- isolation valves;
- bends/fittings;
- cooler/return path.

Therefore 1.8 MW is a component-level lower layer, not the parasitic load to use in the final comparison.

## Engineering implication

The published 170 MW / 81 kg/s / 58 kPa benchmark weakens the hypothesis that an ~80 kg/s helium loop is inherently implausible.

It does not establish:
- Singapore siting feasibility;
- acceptable total pressure drop;
- acceptable circulator size;
- acceptable IHX area/cost;
- acceptable transient behaviour;
- compatibility with the exact 96 MW reformer temperature profile.

These remain explicit tests.

## Next requirement

Construct a loop pressure-drop budget:

DeltaP_loop =
DeltaP_IHX +
DeltaP_hot-duct +
DeltaP_reformer +
DeltaP_steam-generator +
DeltaP_valves +
DeltaP_return.

Then compute:

W_circ = f(m_dot, P, T, DeltaP_loop, eta).

The model should use published component pressure losses where available and sensitivity ranges otherwise. A reactor candidate should not be selected until the parasitic fraction W_circ/Q_delivered is bounded.
