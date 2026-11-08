# Tritium Transport Across the IHX — HTTR Evidence

## Question

**Can radioactive tritium permeate the IHX and contaminate secondary helium or the hydrogen-production plant?**

## Answer

**YES — permeation/transport across the high-temperature metal boundary is physically real and has been measured in HTTR secondary helium.**

However, JAEA's HTTR-IS assessment found that the assessed secondary-loop tritium concentration/quantity could remain below relevant regulatory limits, supporting a design in which the hydrogen-production plant can be outside the nuclear-facility safety-function classification.

This is **not** equivalent to zero tritium, perfect radiological isolation, or automatic applicability to the 600 MWth project.

## Tritium generation in HTGRs

Chinese and Japanese HTGR literature identify several production routes:
- ternary fission in fuel;
- neutron activation of He-3 in helium;
- Li-6 activation;
- B-10 activation in boron-containing materials.

Tritium can enter the primary helium in chemical forms including hydrogen isotopologues (e.g. HT) and tritiated hydrocarbons depending coolant chemistry. Graphite sorption/chemistry also affects inventory and transport.

## Direct HTTR measurement

Dipu et al. (JAEA, Annals of Nuclear Energy 88, 2016) measured tritium during the HTTR's 50-day 950 C continuous operation.

Reported values:
- primary helium peak: **1.6e-1 Bq/cm3(STP)** at 60% reactor power;
- secondary helium peak during power rise: **4.7e-2 Bq/cm3(STP)**;
- secondary helium at end of normal full-power phase: **2.2e-2 Bq/cm3(STP)**.

The secondary-loop concentration being non-zero is direct evidence that the IHX is **not a perfect radiological barrier**.

The gradual primary-loop decrease was attributed in part to tritium chemisorption on graphite.

## Permeation through Hastelloy XR

Japanese experiments specifically studied hydrogen-isotope permeation through Hastelloy XR, the HTTR IHX material.

Takeda et al. report:
- permeation is diffusion-limited in the solid metal under studied conditions;
- hydrogen permeability has Arrhenius temperature dependence;
- an oxide film formed during ~140 h heating in helium reduced permeation.

A later evaluation using actual initial 950 C HTTR operation inferred that oxide film on heat-transfer tube surfaces suppresses permeation.

Important qualification:
**oxide films mitigate permeation; they do not make the wall impermeable.**

Oxide integrity and coolant chemistry therefore matter.

## Regulatory / classification significance

The Japanese HTTR-IS programme explicitly treats tritium transfer as a condition for making the hydrogen plant **non-nuclear grade / outside nuclear safety-function classification**.

JAEA-Technology 2023-019 states that the amount/concentration of tritium in HTTR secondary helium is extremely small and below the relevant radioisotope-law criteria, citing the Dipu et al. assessment.

In 2025 JAEA/NRA review of the current HTTR heat-utilisation test, the proposed legal boundary converged at the reactor-building isolation valve, with the hydrogen-production facility outside the Reactor Regulation Act scope in the discussed design approach.

This is strong regulatory-design precedent, but it is **HTTR-specific**.

## Why the project cannot directly inherit the HTTR conclusion

The project differs in:
- reactor thermal power: 600 MWth design vs 30 MWth HTTR;
- secondary heat duty: 176.8 MWth vs 10 MW HTTR IHX;
- exchanger surface area/material/design;
- fuel/core inventory;
- boron/lithium/He-3 inventories and impurities;
- primary/secondary purification rates;
- temperature history;
- plant lifetime;
- reformer/process geometry;
- applicable Singapore radiological/nuclear classification thresholds.

Therefore a project tritium concentration cannot be scaled simply by 600/30 or 176.8/10.

## Required project tritium model

A defensible assessment would require:

tritium generation in core
→ release to primary helium
→ graphite sorption/desorption
→ primary purification/removal
→ IHX wall permeation
→ secondary helium inventory
→ secondary purification/removal
→ reformer-wall permeation / chemical conversion
→ product hydrogen / process stream concentration
→ release / worker exposure
→ regulatory classification.

Parameters include:
- radionuclide production rates;
- chemical speciation;
- primary/secondary partial pressures;
- Hastelloy XR permeability vs T;
- oxide-film permeation reduction;
- heat-transfer area/thickness;
- purification flow/removal efficiency;
- operating time/transients.

## Mitigation hierarchy

Evidence-supported mitigation concepts:
1. **intermediate helium loop** — prevents direct primary coolant entry;
2. **material/oxide barrier** — reduces isotope permeation;
3. **helium purification** — removes hydrogenous/radioactive species;
4. **pressure/chemistry control** — affects driving force/speciation;
5. **isolation valves / legal boundary** — limits accident propagation and defines regulated interface;
6. **monitoring** — secondary helium and product/process tritium surveillance.

Additional coatings or alternative materials may be future options but require qualification.

## Decision consequence

If project-specific tritium assessment cannot keep secondary/process/product concentrations below the future Singapore regulatory/classification criteria:

- the chemical plant may require nuclear-grade classification or additional radiological controls;
- equipment, QA, monitoring, licensing and cost boundaries change;
- the current economic architecture may no longer be valid.

If mitigation cannot achieve acceptable levels:

**THE CURRENT DIRECT HIGH-TEMPERATURE COUPLING ARCHITECTURE REQUIRES REDESIGN.**

## Status

New question DF-32:

**Can tritium permeation remain low enough for the chemical plant to retain non-nuclear classification?**

Current status:

**PARTIALLY ANSWERED — HTTR MEASUREMENT AND MITIGATION EVIDENCE SUPPORT FEASIBILITY; 600 MWth PROJECT-SPECIFIC TRITIUM MODEL / SINGAPORE CRITERION UNRESOLVED.**

## Sources

- Dipu et al. (2016), Annals of Nuclear Energy 88, 126-134, DOI 10.1016/j.anucene.2015.10.028.
- 武田哲明ほか (2000), 日本原子力学会誌 42(3), 204-211, DOI 10.3327/jaesj.42.204.
- 坂場成昭ほか (2006), Journal of Nuclear Materials 353, 42-51, DOI 10.1016/j.jnucmat.2006.03.008.
- JAEA-Technology 2023-019, Q60.
- JAEA current HTTR heat-utilisation licensing material.
- Tsinghua native tritium/source-term research for HTGR production pathways as independent cross-check.
