# E4 Later Visual Specification — Integrated Safety

## Purpose
Data specification only. Do not treat this as a licensed layout, QRA result, separation distance, EPZ, or final artwork.

## Main conceptual safety figure
Show, not to scale:
- nuclear island: 600 MWth GTHTR300C-class source;
- TRISO/core -> primary helium pressure boundary -> IHX;
- two secondary-helium branches/manifold concept -> two 130 MMSCFD SMR-H2+CCS trains;
- visible physical separation gap labelled "distance determined by QRA/PRA/consequence analysis";
- nuclear barriers: fuel/coatings, graphite/core, pressure boundary, reactor-building/confinement functions as applicable, IHX, isolation;
- chemical barriers: pressure containment, gas detection, isolation/ESD, ventilation, fire/blast protection;
- independent/diverse nuclear safety power and ultimate heat sink;
- chemical utilities shown separately unless a shared dependency is intentionally highlighted.

## Propagation arrows
Use two clearly different arrow families:
1. Nuclear -> chemical:
   reactor trip/loss of heat; secondary-loop failure; IHX leakage/radiological transfer; shared utility/cooling disturbance.
2. Chemical -> nuclear:
   H2/NG fire/explosion; projectile/steam rupture; CO/CO2 personnel/access effects; shared utility/cooling loss.

Every arrow must terminate at a named barrier/safe-state function, not imply inevitable propagation.

## Evidence tags
Add compact labels:
- DEMONSTRATED/EXPERIMENTAL: HTTR high-temperature operation and LOFC; HTTR IHX; measured secondary-loop tritium; AGR TRISO tests.
- SOURCE DESIGN/MODELLED: GTHTR300C scale/process interface and separation concepts.
- PROJECT SCREENING: 353.6 MWth two-train heat-sink step; Jurong external-hazard context.
- UNRESOLVED/E5: PRA, mechanistic source term, QRA contours, blast distance, dose, EPZ.

## Deterministic technical diagrams that should NOT be AI artwork
Keep these as reproducible vector/table/graph diagrams:
- barrier/defence-in-depth chain;
- nuclear->chemical and chemical->nuclear propagation matrix;
- safe-state function logic;
- E5 evidence/data-flow: initiator -> frequency -> event tree/fault tree -> physical consequence -> source term/chemical consequence -> dispersion -> dose/risk -> acceptance;
- any future risk contours, event trees, fault trees, or quantitative blast/dispersion plots.

## Prohibited claims
Do not show:
- an approved Jurong nuclear parcel;
- a numeric separation distance;
- a project EPZ radius;
- probabilities/frequencies;
- risk contours;
- a guaranteed passive-safe outcome;
- zero tritium/radiological transfer;
- shared cooling/power as safety-qualified;
- LCT3 as nuclear infrastructure.
