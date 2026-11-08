# Heat Rejection, Cooling and Singapore Siting Evidence

## Research questions

1. What heat must the selected 600 MWth HTGR ultimately reject after supplying 176.8 MWth to the hydrogen process?
2. What cooling system would be required?
3. Does Jurong Island, offshore/floating or underground siting improve feasibility?
4. What can be concluded without a site-specific nuclear design?

## Critical accounting correction

The project's **423.2 MWth remaining reactor thermal capacity is NOT a cooling duty**.

It is only:

600 MWth reactor rating - 176.8 MWth process heat = 423.2 MWth remaining thermal capacity.

In the source GTHTR300C architecture, remaining thermal energy can enter a direct Brayton power cycle. JAEA's design includes a gas turbine, recuperator and precooler; low-temperature heat removed in the precooler can itself be used for desalination/district heating. Therefore:

reactor thermal input
= high-temperature process heat
+ electrical work
+ recoverable lower-temperature heat
+ ultimate environmental heat rejection
+ losses.

Because the project intentionally makes **no exact off-design project gross/net/export MWe claim**, it also lacks the thermodynamic state points needed to calculate an exact project condenser/precooler heat-rejection duty.

**Current cooling-duty status: UNRESOLVED — REQUIRES A COHERENT OFF-DESIGN POWER/HEAT CYCLE OR A HEAT-ONLY REJECTION DESIGN.**

## Source GTHTR300C cooling architecture

JAEA's GTHTR300C source design is a direct helium Brayton cogeneration system. Published load-follow work describes:
- 600 MWth reactor;
- 950 C reactor outlet;
- topping IHX delivering 900 C process heat;
- direct-cycle gas turbine;
- recuperator;
- precooler removing low-temperature cycle heat;
- intermediate helium loop isolating the distant process plant.

The source literature notes that ~160 C waste heat from the precooler can be used for desalination/district heating without reducing gas-turbine output.

A JAEA cascade-energy concept explicitly shows seawater/freshwater desalination integrated with the low-temperature heat sink. This demonstrates a **design option**, not a Singapore cooling-system selection.

## Why a seawater-flow number is premature

A once-through seawater flow would require at least:

m_dot_water = Q_reject / (cp_water * DeltaT_water).

But neither Q_reject nor the allowable intake/outfall DeltaT is established for the project. Q_reject depends on:
- selected power-cycle operating point;
- gas-turbine/compressor/recuperator performance;
- whether remaining reactor capacity generates power;
- whether low-temperature heat is recovered;
- process heat returned/recovered from the chemical plant;
- auxiliary loads;
- environmental discharge limits.

Publishing a seawater flow now would therefore be false precision.

## Nuclear siting criteria relevant to Singapore

IAEA SSG-35 / SSR-1 require site survey/evaluation to address natural and human-induced hazards, population/emergency-response feasibility, cooling-water availability, security and environmental impacts.

For a Singapore coastal/industrial candidate, relevant criteria include:

### Natural / coastal
- coastal flooding, storm surge, tides, waves and combinations;
- extreme precipitation and inland/coastal compound flooding;
- sea-level change over plant lifetime;
- shoreline erosion/sedimentation;
- geotechnical stability, settlement/subsidence and liquefaction;
- extreme temperature/wind/lightning;
- intake blockage by debris or marine organisms;
- loss of ultimate heat sink.

### Human-induced / industrial
- refineries and oil/gas operations;
- hazardous-substance processing/storage;
- external fire/explosion/toxic releases;
- shipping collision and port hazards;
- aircraft/security threats;
- pipelines and neighbouring industrial infrastructure;
- electromagnetic/interference or grid/common-utility dependencies where relevant.

### Emergency / population
- population density and distribution;
- feasibility of sheltering/evacuation/protective actions;
- emergency access;
- ability to establish the regulator-approved emergency planning arrangements.

### Non-safety / deployment
- land footprint;
- access to cooling water;
- grid and industrial load proximity;
- construction/logistics access;
- ecological impacts and thermal discharge;
- public acceptance;
- security zoning.

## Jurong Island as a screening case

### Why it is attractive
JTC describes Jurong Island as a ~3,000 ha integrated energy-and-chemicals complex with >100 companies and >100 km of pipelines. This makes it an obvious **industrial-integration screening case** for natural gas, hydrogen, CO2 handling, utilities and industrial offtake.

### Why it is not selected
The same industrial concentration creates nuclear external-hazard questions: refineries, chemical plants, hazardous inventories, pipelines, shipping and common infrastructure are exactly the types of human-induced hazards IAEA siting guidance requires evaluating.

PUB has a dedicated Jurong Island coastal-protection study because coastal flood/sea-level-rise risk is material. Singapore is low-lying, and PUB states ~30% of the island is below 5 m above mean sea level. New critical infrastructure is being planned with higher coastal protection/platform levels.

No project-specific study currently establishes:
- nuclear-compatible parcel size;
- separation distance from hazardous neighbours;
- design-basis industrial explosions/fires;
- flood elevation/protection;
- intake/outfall design;
- emergency-planning feasibility;
- security perimeter;
- geotechnical suitability.

**Disposition: JURONG ISLAND = SCREENING CASE ONLY — REQUIRES SITE-SPECIFIC NUCLEAR/INDUSTRIAL HAZARD STUDY.**

## Offshore / floating option

Potential advantages:
- reduced competition for land;
- direct access to seawater heat sink;
- potentially greater separation from population/industrial hazards.

Potential disadvantages/questions:
- marine collision and anchoring/mooring hazards;
- storm/wave/sea-level/extreme-weather design;
- corrosion;
- security and territorial/control issues;
- emergency access/evacuation;
- spent-fuel/radioactive-material transport;
- subsea/ship-to-shore hydrogen/heat/electric/CO2 interfaces;
- maintenance/drydock strategy;
- regulator and safeguards framework.

**Disposition: FUTURE WORK / ALTERNATIVE SITE CONCEPT.** No evidence currently supports selecting it over land-based siting.

## Underground option

Potential advantages:
- shielding/protection from some external events;
- possible security/consequence benefits depending design.

Potential disadvantages/questions:
- excavation/geotechnical suitability;
- flooding/water ingress;
- heat removal;
- access/egress and emergency response;
- construction cost/schedule;
- hydrogen/chemical process cannot simply be assumed underground with reactor;
- maintenance/replacement logistics.

**Disposition: FUTURE WORK / ALTERNATIVE SITE CONCEPT.**

## Cooling / siting V&V maturity

| Item | Verification | Validation | Uncertainty | Extrapolation |
|---|---|---|---|---|
| 423.2 MWth residual capacity identity | exact arithmetic/testable | N/A | none | none |
| source GTHTR300C Brayton/precooler architecture | literature cross-check | component models partly tied to HTTR/mock-ups | design-study uncertainty | source design → project off-design state |
| project heat rejection | not modelled | none | high | cannot infer from residual capacity |
| Singapore cooling-water system | not modelled | none | site/environment dependent | requires candidate site |
| Jurong flood/external hazards | screening evidence | PUB/JTC authoritative context | parcel-specific unknown | requires site survey |
| site acceptability | not modelled | none | regulatory/site dependent | cannot be claimed |

## Current conclusions

- **DF-21 heat rejection:** remaining 423.2 MWth is not waste heat; exact rejection duty is **UNRESOLVED**.
- **DF-22 floating/offshore:** **FUTURE WORK**, no preferred-site conclusion.
- **DF-23 underground:** **FUTURE WORK**, no preferred-site conclusion.
- **DF-19 Jurong Island:** **REQUIRES SITE-SPECIFIC ANALYSIS**; industrial integration is attractive but external-hazard/flood/security/emergency issues are potentially decisive.
- Cooling-system numerical sizing should wait until a coherent project power/heat cycle is selected.

## Primary / authoritative sources

- Yan et al. / JAEA, GTHTR300C load-follow and cogeneration design literature.
- JAEA-Technology 2011-013, GTHTR300 series specifications/cascade-energy system.
- IAEA SSR-1, *Site Evaluation for Nuclear Installations*.
- IAEA SSG-35, *Site Survey and Site Selection for Nuclear Installations*.
- IAEA meteorological/hydrological site-evaluation safety guidance.
- PUB Singapore, coastal protection and Jurong Island site-specific studies.
- JTC, Jurong Island official industrial/infrastructure context.


## Singapore-native cooling evidence

Singapore industrial practice provides real cooling options without defining this project's heat-rejection duty.

### Once-through seawater
PUB states that seafront/Jurong Island companies can use seawater as a cooling medium in **once-through seawater cooling**. EMA's reference technical parameters for Singapore CCGT modelling use a once-through cooling-water system with an **8 C condenser temperature rise** and 29.2 C seawater reference temperature.

This is evidence that once-through seawater cooling is technically established in Singapore. It is not a nuclear-project design basis.

### Seawater cooling towers
PUB also identifies **seawater cooling towers (SWCT)** as technically feasible and economically viable, particularly for greenfield applications. Earlier PUB/Jurong Island work notes SWCT can reduce NEWater reliance and, in network form, may reduce environmental impacts relative to large once-through discharge.

Trade-offs include:
- land/structure footprint;
- salt drift/corrosion;
- pumping/fan power;
- plume;
- blowdown;
- maintenance/fouling.

No project SWCT footprint is calculated because Q_reject is unresolved.

### Freshwater / NEWater
NEWater is deliberately used for industrial and cooling applications in Singapore and is a valuable water resource. Using large volumes of NEWater for a coastal nuclear heat sink would create an opportunity-cost/resilience question when seawater alternatives exist.

Therefore freshwater/NEWater cooling is an **option to evaluate**, not the default.

### Thermal discharge and marine effects
Real Jurong Island CCGT EIAs model thermal plumes rather than assuming discharge is harmless. A 2026 PacificLight EIA predicts temperature elevations >3 C beyond 100 m for <10% of time and >2 C within roughly 300 m of its outfall under its own discharge conditions.

A separate Jurong Island cogeneration EIS modelled ~105,000 m3/h heated-water discharge at +7 C and assessed the resulting local thermal plume/marine effects.

These values are **site/project-specific examples**, not nuclear-project discharge assumptions. They demonstrate that:
- thermal plume modelling is required;
- marine receptors matter;
- chlorine/biocide residuals can matter;
- intake/outfall location is an environmental-design variable.

### Fouling / blockage
Singapore industrial evidence reports seawater fouling increasing thermal resistance and reducing flow in heat exchangers; back-flushing was used to restore performance. Nuclear ultimate-heat-sink analysis would additionally need to consider debris, marine organisms and common-cause intake blockage.

## Coastal cooling benefit versus coastal hazard

**COASTAL LOCATION → BENEFIT**
- abundant seawater heat sink;
- established Singapore once-through/SWCT experience;
- possible shared intake/outfall infrastructure;
- reduced freshwater demand;
- industrial/desalination integration opportunities.

**COASTAL LOCATION → HAZARD**
- sea-level rise;
- storm surge / coastal flooding;
- wave/tide effects;
- intake blockage/fouling;
- salt corrosion;
- marine thermal/chemical discharge;
- shipping/external hazards;
- need for long-life coastal protection.

A coastal location is therefore neither automatically favourable nor unfavourable. Cooling access and external-hazard resilience must be evaluated together.

## Singapore site-option set — no selection

The project will not settle on Jurong Island. Current options are:

| Option | Potential strengths | Potential weaknesses / unknowns | Current status |
|---|---|---|---|
| Jurong Island | existing energy/chemical infrastructure; gas/H2/CO2/utilities; seawater access | petrochemical external hazards; limited land; coastal flood; security; emergency planning | SCREENING OPTION |
| Future western island | long-term new power-generation infrastructure explicitly contemplated; potential new-build layout/separation; coastal cooling | reclamation decades-long; final land profile/use unknown; no nuclear decision/site designation; marine/coastal/security issues | FUTURE OPTION |
| Other coastal/industrial site | seawater access; possible grid/industrial integration | land, population, external hazards, environment/site geology unknown | SCREENING OPTION |
| Underground | possible shielding/land/security advantages | excavation, flood/water ingress, heat rejection, access/maintenance/cost | FUTURE CONCEPT |
| Offshore/floating | land relief, seawater access, population separation potential | marine hazards, collision, corrosion, security, emergency access, fuel/waste logistics, regulation | FUTURE CONCEPT |
| Other future site | preserves option value | insufficient evidence | UNRESOLVED |

### New western island evidence
At National Day Rally 2026, Singapore announced a long-term plan to connect several islands south of Jurong Island into a new western island that could support advanced manufacturing and **new power-generation infrastructure**. The Government did **not** identify nuclear power as the selected generation technology or designate a nuclear site.

This is therefore:
**EVIDENCE OF FUTURE POWER-SITING OPTION SPACE, NOT A NUCLEAR SITING DECISION.**

Singapore still states that no nuclear deployment decision has been made and INIR Phase 1 will begin from 2027.

## Decision consequences

- If exact heat rejection later requires a cooling system whose land/intake/outfall/thermal-discharge impacts cannot be accommodated → **that site/cycle configuration is infeasible or requires heat-recovery/cooling redesign**.
- If coastal flood/storm-surge protection cannot meet nuclear safety requirements over plant life → **candidate coastal site infeasible**.
- If petrochemical external hazards/separation cannot be bounded at Jurong → **Jurong infeasible; other site options remain open**.
- If no Singapore site can simultaneously satisfy cooling, external hazards, emergency planning, security and land constraints → **deployment concept infeasible under current siting options**.


## Jurong peer-reviewed site-screening evidence

Devanand, Karimi & Kraft (Computers & Chemical Engineering 125, 2019, 339-350) develop a mixed-integer nonlinear optimisation method for preliminary modular-nuclear site selection and demonstrate it using the **J-Park Simulator**, an imaginary/virtual representation of the Jurong Island eco-industrial park.

The paper explicitly describes the method as a **preliminary analysis tool** requiring geographical/energy-demand inputs and intended to identify candidate locations for further study.

Correct evidence classification:
**PRELIMINARY ACADEMIC SITE-SCREENING EVIDENCE.**

It is **not**:
- licensing evidence;
- a real parcel/site approval;
- EPZ evidence;
- a site-specific nuclear external-hazard assessment;
- evidence that Jurong is preferred over future western/offshore/other sites.

The paper is useful because it demonstrates why cooling-water availability, cost, earthquake/geographical constraints and energy-demand proximity can be integrated quantitatively. A future Singapore site study could extend that framework with the nuclear-specific constraints now identified in this project.

## Current official Singapore cross-check

Singapore's 2026 official energy policy remains technology- and site-open. PM Lawrence Wong states that nuclear is being studied as a long-term option, safety is the overriding priority, and the entire ecosystem—regulation, security, emergency response and waste management—must be established. No site has been selected.

Therefore academic Jurong screening does not override the national pre-decision status.
