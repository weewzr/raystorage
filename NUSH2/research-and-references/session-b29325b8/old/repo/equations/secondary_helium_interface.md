# Secondary-helium / IHX screening model

## Evidence basis

JAEA's HTTR steam-reforming system is the closest identified physical precedent to the proposed architecture.

Authoritative JAEA sources report:
- HTTR primary helium heated to 950 C;
- heat transferred through an intermediate heat exchanger (IHX) to secondary helium;
- secondary helium supplying a steam reformer, steam superheater and steam generator;
- 10 MW nuclear heat supplied to the IHX in the HTTR steam-reforming system design;
- the mock-up facility used 4 MPa helium heated to 880 C at the chemical-reactor inlet, stated to match the HTTR hydrogen-production-system condition;
- the mock-up used a full-scale reaction tube and approximately 110-120 Nm3/h H2 capacity;
- thermal disturbance from the chemical plant is buffered by a downstream steam generator to protect the reactor/IHX inlet condition.

Sources:
JAEA-Technology 2018-004, section 4.11;
JAEA-Technology 2007-022;
JAEA 2025 HTTR heat-application licensing announcement.

## Temperature hierarchy

A physically feasible direct-heat path requires:

T_primary,out > T_secondary,hot > T_process,hot.

Define:
- T_R = primary helium reactor outlet temperature;
- T_He = secondary helium hot temperature after IHX;
- T_P = required process-side hot temperature;
- DeltaT_IHX = T_R - T_He;
- DeltaT_SR = T_He - T_P.

Then:

T_R - T_P = DeltaT_IHX + DeltaT_SR.

Both terminal differences must remain positive.

This immediately falsifies any design that equates a 950 C reactor outlet with a 950 C process requirement while also assuming finite heat-exchanger driving forces.

## JAEA precedent

JAEA demonstrates an important feasible temperature scale:
- primary reactor outlet: 950 C;
- steam-reformer helium inlet: about 880 C in the mock-up / design condition.

The total primary-to-reformer helium temperature difference is therefore of order 70 K in this precedent.

It should NOT be copied directly into the Singapore model as a fixed design value; it is an empirical/design anchor.

## Helium heat-carrier equation

At screening level:

Q = m_dot_He * cp_He * (T_hot - T_cold).

Therefore:

m_dot_He = Q / [cp_He DeltaT_He].

For the verified IEAGHG radiant duty:

Q_radiant = 96.04 MW.

Using a temporary screening assumption:
- cp_He = 5.2 kJ/kg-K;
- secondary helium 880 -> 650 C;

gives:

m_dot_He ~= 80 kg/s.

This is an **assumption-based screening result**, not a final design result. Temperature-dependent helium properties and pressure drop are not yet included.

## Scaling insight

JAEA's older HTTR steam-reforming design used 10 MW nuclear heat and targeted roughly 4000 Nm3/h H2, while the IEAGHG industrial reference is 100,000 Nm3/h H2 with 96.04 MW radiant duty.

The systems are not geometrically similar and should not be scaled linearly without checking process integration. Nevertheless, JAEA establishes:
- material/component feasibility at relevant temperature;
- use of an IHX and secondary loop;
- a real control problem caused by chemical-plant thermal transients;
- the need for a steam generator/thermal absorber downstream.

## Safety/control implication

Direct nuclear process heat creates a dynamic coupling problem absent from a simple steady-state heat balance. JAEA specifically designed the downstream steam generator to mitigate secondary-helium temperature fluctuations and avoid reactor scram.

Therefore the final project should include, at minimum, a steady-state constraint that preserves the required IHX return temperature. A dynamic model is desirable if time permits but is not required before the steady-state feasibility gate.

## Current acceptance envelope

For any candidate reactor/process pair:

1. T_primary,out > T_secondary,hot.
2. T_secondary,hot > T_process,hot.
3. Q_available >= Q_required after IHX/loop losses.
4. helium mass flow and pressure drop must be physically credible.
5. the secondary-loop return temperature must be compatible with reactor/IHX requirements.
6. the process must retain or replace the conventional steam/preheat heat-recovery services.

No commercial reactor is selected yet.
