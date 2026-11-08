# Secondary-helium loop pressure-drop sensitivity

## Evidence status

A component-complete pressure-drop dataset for the exact proposed HTGR -> IHX -> reformer -> steam-generator loop has not yet been found.

Therefore this analysis does **not** invent pressure losses for the reformer, piping, valves or steam generator.

The authoritative anchor currently available is the published GTHTR300C secondary-side IHX pressure loss:

DeltaP_IHX = 58 kPa

at approximately:
- 81 kg/s secondary helium;
- 5.15 MPa;
- 500 -> 900 C;
- 170 MWth IHX duty.

Source: large-scale JAEA GTHTR300C IHX design summarised in Energies 2025, 18, 4632.

## Sensitivity representation

Until component-specific losses are sourced, define:

DeltaP_loop = k_dp * DeltaP_IHX

with k_dp >= 1.

This is deliberately transparent:
- k_dp = 1 means IHX-only lower layer;
- k_dp = 2 means all other loop losses together equal one additional IHX loss;
- k_dp = 3 means total loop loss is three times the published IHX loss;
- larger values can be tested without pretending they are measurements.

## Pumping relation

For small DeltaP/P:

W_circ ~= m_dot DeltaP / (rho eta)

and

rho ~= P/(R_He T).

Using the published GTHTR300C benchmark and eta=0.80 as an explicit screening assumption:

- k_dp=1 gives a parasitic fraction of about 1.1% of the 170 MW heat duty;
- k_dp=3 gives about 3.2%.

Because the screening expression is linear in DeltaP, the result can be rescaled directly when better component data become available.

## Interpretation for the Singapore research case

This sensitivity weakens a possible objection that helium circulation must necessarily consume a very large fraction of delivered heat. Published IHX pressure loss alone corresponds to a low-single-digit percentage parasitic burden under the stated assumptions.

However, it does NOT yet demonstrate a low total-loop burden. The reformer tube-side/shell-side arrangement, hot-gas duct, steam generator, isolation valves and return piping could materially increase DeltaP.

The scientific acceptance condition is therefore:

W_circ / Q_delivered must be bounded using either:
1. source-backed component pressure losses, or
2. a declared conservative sensitivity envelope whose effect on the nuclear-vs-electric comparison is shown.

## Conventional heat-recovery constraint

IEAGHG confirms that the fired reformer's convection section contains:
- reformer feed preheater;
- pre-reformer feed preheater;
- steam superheater;
- feed preheater;
- steam generator.

The syngas waste-heat boiler supplies about 75% of saturated HP steam generation and can remain if reformer outlet conditions remain comparable.

Therefore the next thermal-service ledger should distinguish:
A. 96.04 MW radiant duty to replace directly;
B. flue-gas convection services that disappear with the furnace;
C. syngas/shift heat recovery that remains;
D. CCS-specific steam/electricity loads.

This prevents double-counting heat that is already internally recovered.
