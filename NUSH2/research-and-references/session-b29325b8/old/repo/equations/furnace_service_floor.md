# Quantified furnace-replacement service floor

## Source-backed components

The conventional fired reformer provides multiple thermal services. Two are now quantitatively constrained.

### 1. Radiant reformer duty

IEAGHG equipment list:

Q_radiant = 82.63 MMkcal/h = 96.04 MW.

### 2. HP steam superheating

IEAGHG heat/material balance:
- 95.301 t/h HP steam to process at 400 C / 4.29 MPa;
- 46.053 t/h HP steam export at 395 C / 4.23 MPa.

Combined:

m_HP ~= 141.354 t/h = 39.265 kg/s.

IEAGHG states that saturated HP steam is routed through the furnace Steam Superheater Coil before process/export use.

For a transparent property bracket around the source pressure, IAPWS-consistent tables give:

At 4.0 MPa:
- T_sat = 250.36 C;
- h_g,sat = 2800.9 kJ/kg;
- h(400 C) = 3214.5 kJ/kg;
- Delta h = 413.6 kJ/kg.

At 4.5 MPa:
- T_sat = 257.4 C;
- h_g,sat = 2797.9 kJ/kg;
- h(400 C) = 3205.6 kJ/kg;
- Delta h = 407.7 kJ/kg.

The IEAGHG pressure 4.23-4.29 MPa lies between these anchors. Applying the full 141.354 t/h as a first bracket gives:

Q_superheat ~= 16.0-16.25 MW.

The export stream is actually 395 C rather than 400 C, so a later exact IAPWS calculation will resolve the two streams separately. The present bracket is a screening bound, not the final value.

## Quantified thermal-service floor

Adding only the two source-constrained furnace services:

Q_quantified = Q_radiant + Q_superheat

gives approximately:

Q_quantified ~= 112.0-112.3 MW.

This is **not** the final HTGR heat requirement.

It still excludes:
- reformer-feed preheat;
- pre-reformer-feed preheat;
- feed preheat;
- the furnace-convection share of saturated-steam generation;
- heat-exchanger/IHX losses.

It also excludes retained heat recovery that should not be replaced:
- reformer syngas WHB;
- shift heat recovery.

Therefore ~112 MW is best interpreted as the currently quantified thermal-service floor for replacing the fired-furnace functions at the IEAGHG reference scale, before nuclear-loop losses and before unresolved convection duties.

## Why this matters

The earlier 96 MW radiant-duty anchor understated the known furnace-dependent service because the conventional furnace also superheats the plant's HP steam.

Conversely, adding all steam generation to the nuclear requirement would overstate it because IEAGHG reports around 75% of saturated steam generation from the syngas WHB, which remains available if comparable reformer outlet conditions are preserved.

The correct model must therefore replace only lost services and retain internal recovery.
