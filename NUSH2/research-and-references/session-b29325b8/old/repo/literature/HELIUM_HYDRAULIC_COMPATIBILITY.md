# Secondary-Helium Hydraulic Compatibility Evidence

## Research question

Is the INL Case-6 secondary-helium requirement compatible with published GTHTR300C-class heat-transport hardware, and can project pressure drop/circulator power be estimated defensibly?

## Direct source cross-check

### INL Case 6
TEV-961 reports for the 925 C reactor-outlet case:
- secondary helium supply: **900 C**;
- return: **466 C**;
- helium mass flow: **78.49 kg/s**;
- process heat: **176.8 MWth**.

### GTHTR300C reference IHX
JAEA-Technology 2008-093 reports for the 170 MWth GTHTR300C IHX/secondary loop:
- secondary helium outlet: **900 C**;
- inlet: **500 C**;
- secondary flow: **81 kg/s**;
- secondary pressure: **5.15 MPa**;
- reference IHX: **170 MWth**.

JAEA/IAEA tabulated design data report approximately:
- tube-side pressure loss: **58 kPa** for the 170 MWth reference IHX;
- shell-side pressure loss: ~31 kPa;
- 45 mm tube OD, 5 mm wall;
- ~1448 m2 heat-transfer area;
- planned 20-year design life for the cited concept.

A later horizontal-IHX design reports ~80.3 kg/s, 491→900 C, 5.15 MPa and <60 kPa secondary pressure loss.

## Compatibility ratios

Project/source-model helium flow relative to reference GTHTR300C secondary flow:

78.49 / 81 = **0.969**.

Thus the INL reformer requires about **3.1% less secondary helium mass flow** than the 170 MWth GTHTR300C reference loop, despite ~4.0% higher heat duty. This is physically consistent with the larger INL secondary-helium temperature drop (900→466 C = 434 K) compared with the GTHTR300C reference (900→500 C = 400 K).

This cross-source agreement is valuable because INL and JAEA arrive at nearly the same high-temperature helium flow scale from different process/design studies.

## What is supported

- ~80 kg/s secondary-helium flow scale: **SUPPORTED BY TWO INDEPENDENT SOURCE DESIGNS/MODELS**.
- ~5.1 MPa secondary pressure class: **SUPPORTED BY GTHTR300C DESIGN**.
- 900 C supply: **SUPPORTED BY BOTH**.
- A reference 170 MWth IHX pressure loss around 58 kPa: **SUPPORTED AS A CONCEPTUAL GTHTR300C COMPONENT VALUE**.
- Need for helium circulator, isolation/safety valves, purification and storage/supply systems: **SUPPORTED BY JAEA GTHTR300C system specification**.

## What is not supported

The project cannot claim:
- 58 kPa as its actual pressure drop;
- a specific circulator shaft/electric power;
- a specific transport-pipe diameter;
- a specific reactor-to-reformer separation distance;
- a specific heat loss along Singapore piping.

Those quantities require the selected IHX and piping geometry, length, roughness, fittings, insulation, pressure level, circulator efficiency and transient design.

JAEA's 2017 heat-transport-piping work shows that pipe structure/material/insulation materially affects diameter, material quantity, heat loss and temperature reduction. That confirms piping geometry is a design variable rather than a number to infer from mass flow alone.

## Approximate pressure-drop scale — evidence tag

The 58 kPa value is useful only as a **reference-component scale**:

58 kPa / 5.15 MPa ≈ **1.13% of loop absolute pressure**.

This indicates that high-pressure helium transport does not inherently imply a pressure drop comparable to system pressure in the published reference IHX. It is **not a project ΔP calculation**.

## V&V status

| Quantity | Status |
|---|---|
| 78.49 kg/s project process-model flow | SOURCE-MODELLED (INL) |
| 81 kg/s reference secondary flow | DESIGNED (JAEA) |
| ~5.15 MPa reference secondary pressure | DESIGNED |
| ~58 kPa reference IHX tube-side ΔP | DESIGNED / component calculation |
| project ΔP | NOT MODELLED |
| project circulator power | NOT MODELLED |
| project pipe diameter/length | NOT SELECTED |
| project heat loss | NOT MODELLED |

## Disposition

DF-05 changes to:

**PARTIALLY ANSWERED — HELIUM FLOW/PRESSURE CLASS SHOW STRONG SOURCE-LEVEL COMPATIBILITY; PROJECT HYDRAULICS/CIRCULATOR POWER REQUIRE DETAILED DESIGN.**

A new Rust pressure-drop calculation is **not justified yet** because it would require invented pipe geometry. The correct next model step is to wait until a defensible transport-loop layout is sourced/selected.

## Primary sources

- INL TEV-961, Table 2, Case 6.
- JAEA-Technology 2008-093, GTHTR300C specifications.
- GTHTR300C IHX design literature / IAEA hydrogen-production technical review.
- Nomoto et al. (2017), heat-transport piping design for GTHTR300C and HTTR-GT/H2.
