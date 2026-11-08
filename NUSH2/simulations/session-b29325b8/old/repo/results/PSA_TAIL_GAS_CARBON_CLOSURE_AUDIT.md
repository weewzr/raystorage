# PSA tail-gas carbon closure audit — source-data gate

## Governing conservation law
For a steady process boundary, elemental carbon satisfies
```
C_in [kmol C/h] = C_captured + C_direct_emitted + C_carbon_products + C_other_out
```
with zero accumulation. Methane, CO, CO2 and all other carbon-bearing gas/liquid species must be included using their carbon stoichiometric coefficients. Combusting methane/CO in PSA tail gas changes molecular species but does not destroy elemental carbon.

## Source quantities presently used
INL Case-6 source/model anchors in `model/src/final_design.rs`: 34.0 MMSCFD natural gas, 1,927 short ton CO2/day captured, 142 short ton CO2/day emitted, and 3,205 short ton CO2/day conventional baseline emitted. At 85% availability, these become approximately 542,362 t/y captured and 39,966 t/y emitted **per train**. The baseline direct figure is approximately 902,060 t/y per train. The two-train duplication doubles these annual streams but does not prove a complete carbon balance.

## Partial arithmetic, not elemental closure
At source daily operating basis, captured/(captured + reported emitted) = 1927/(1927+142) = **0.93137**, or **93.14% of these two reported CO2 streams**. This is **not** a verified capture-unit inlet efficiency, plant-wide carbon capture fraction or lifecycle abatement fraction. The conventional baseline 3,205 short ton/day belongs to a different configuration and cannot be inserted as the missing carbon-outlet term. The apparent captured-plus-emitted total of 2,069 short ton/day is not evidence of total methane-derived carbon generated.

## PSA tail gas
The available model constants do not establish the composition, carbon flow or destination of PSA offgas. Possible recycle, combustion, purge and export routes must be traced from original INL TEV-953/961 process stream tables and PFD. The existence of a PSA unit or 88% hydrogen recovery does **not** imply a particular offgas carbon destination. A claim of '90% of remaining CO2' or '90% plant-wide capture' is therefore **NOT VERIFIED** from these model anchors.

## Required closure data and residual
Required: inlet NG species and standard-volume reference conditions; reformer/WGS outlet and PSA inlet stream species molar rates; PSA product and offgas composition/flow; tail-gas destination and any furnace/flue-gas flows; capture absorber inlet/outlet, vent/purge and CO2 product streams; all recycle loops and their boundary crossings.

Without these values, the exact elemental balance residual in kmol C/h and percent **cannot be calculated defensibly**. Report: **residual = NOT CALCULABLE; plant-wide 90% capture = NOT VERIFIED**. Do not set the missing residual to zero by construction.

## Implications
The existing E2B lifecycle avoidance and 1.085 Mt/y captured-throughput figures remain **historical conditional model results**, not verified plant-wide carbon closure. No project capture-efficiency or tail-gas fate is inferred from 1927/(1927+142). A revised central lifecycle result requires the streamwise balance first.

## Deterministic model added (current continuation)
`model/src/psa_carbon_closure.rs` now implements a streamwise kmol-C/h balance using explicit CO2, CO, CH4, C2H6 and C3H8 species, returning **None** when an essential stream is unknown. Synthetic tests cover exact closure, a 10 kmol-C/h nonzero residual, missing tail gas, and the 1927/2069 reported-stream ratio. **Synthetic test cases are verification of the arithmetic, not experimental INL Case-6 results.** The actual INL residual remains NOT CALCULABLE until source-complete molar streams are obtained. No source stream is silently set to zero.
