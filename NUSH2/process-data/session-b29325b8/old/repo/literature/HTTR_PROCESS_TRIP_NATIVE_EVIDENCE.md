# HTTR Process-Trip / Heat-Load Transient Evidence — Japanese Native Sources

## Question

What happens to the reactor when the connected hydrogen plant suddenly loses chemical reaction / heat demand?

## Why this matters

A nuclear-heated reformer is a coupled system. If the chemical reaction stops, secondary helium can return much hotter to the IHX. Without buffering/control, that thermal disturbance can propagate toward the reactor and trigger a trip or challenge component limits.

## Japanese experimental evidence ladder

### 1/30-scale HTTR hydrogen-production mock-up
JAEA/JAERI tested a steam-methane-reforming mock-up representing about 1/30 scale of the HTTR hydrogen-production system.

In a simulated **chemical reaction loss accident**:
- reformer-outlet helium temperature rose by about **189 C**;
- a downstream steam-generator thermal buffer reduced the helium temperature fluctuation to within the target **±10 C** at its outlet;
- analysis agreed well with experiment;
- the study concluded that reactor operation could avoid scram for the tested disturbance/control concept.

Evidence class:
**EXPERIMENTAL MOCK-UP + CODE VALIDATION**, not full-reactor coupled demonstration.

### Full-scale single-reformer-tube / cooling-system tests
JAERI-Tech 2005-014 used a full-scale single reaction-tube test facility to verify dynamic analysis of the secondary-helium cooling system using a steam generator and radiator. Pressure, temperature, flow and heat-transfer responses were reproduced well by the analysis code.

Evidence class:
**COMPONENT-SCALE EXPERIMENT + DYNAMIC-CODE VALIDATION**.

### 2024 HTTR heat-load variation test
JAEA then performed a reactor-level heat-load variation test in HTTR to simulate abnormal load change at a future connected hydrogen-production facility.

JAEA reports:
- heat removal on the secondary side was deliberately reduced;
- reactor inlet temperature temporarily rose by about **11 C**;
- without reactor shutdown/control-rod insertion for the disturbance, reactor power self-adjusted from about **90% to 88%** over ~6 h;
- reactor outlet-temperature change remained within about **1 C**.

Evidence class:
**OPERATING-REACTOR TRANSIENT EXPERIMENT**, but with the existing HTTR cooling system emulating the future heat-utilisation disturbance rather than an actually connected SMR plant.

## Interpretation

This is substantially stronger than saying “coupled transients have never been tested.”

The evidence now supports:
- thermal disturbances from a process-side load change can be buffered/controlled;
- HTTR has experimentally shown a mild reactor response to an emulated heat-load disturbance;
- dynamic analysis codes have been validated against relevant mock-up/component tests.

It does **not** prove:
- the 600 MWth GTHTR300C-class project has the same transient response;
- the 176.8 MWth reformer can trip without project-specific bypass/dump-heat design;
- the project's IHX/piping stresses remain acceptable;
- every process accident can be isolated without reactor trip.

## Project architecture consequence

A defensible project coupling should include a thermal-transient management function such as:
- secondary-loop thermal buffer / steam generator / dump cooler;
- bypass flow;
- rapid isolation;
- reactor control/trip logic;
- reformer feed/steam safe-state controls.

The exact system should not be selected until project dynamic modelling is performed.

## Decision consequence

If project-scale process-trip transients cannot be kept within reactor/IHX temperature, pressure and safety limits:

**THE CURRENT DIRECT COUPLING ARCHITECTURE REQUIRES ADDITIONAL BUFFER/DUMP-HEAT CAPACITY, DIFFERENT CONTROL, GREATER SEPARATION, OR REDESIGN.**

If those measures make the project economically or spatially infeasible, the selected architecture fails.

## Question-register disposition

- DF-10 loss of process heat load: **PARTIALLY ANSWERED — MOCK-UP + HTTR REACTOR-LEVEL TRANSIENT EVIDENCE; PROJECT-SCALE DYNAMIC MODEL STILL REQUIRED**.
- DF-11 loss of nuclear heat to reformer: remains **UNRESOLVED**, because the evidence above primarily tests process-load disturbance propagating toward the reactor, not reformer/catalyst safe-state response to nuclear heat loss.

## Native sources

- 大橋弘史ほか, chemical reaction loss accident mock-up study, ICONE-12 / JOPSS.
- 佐藤博之ほか, HTTR-IS thermal load control methods, 日本原子力学会和文論文誌 7(4), 328-337 (2008).
- JAERI-Tech 2005-014, full-scale single reaction-tube dynamic-code validation.
- JAEA press release, 12 Jun 2024, HTTR heat-load variation test.
- JAEA-Review 2025-053, FY2024 HTTR operation/test report.


## 2026 simulator validation update

JAEA's 2026 R&D review reports a RELAP5-based plant simulator with added atmospheric heat-loss and natural-circulation models. Its predictive performance was checked against the mock-up experiment in which hydrogen production was stopped, reproducing post-stop helium/water temperature and pressure behaviour.

This strengthens the V&V classification:

**process-load-loss transient: mock-up experimental data + dynamic-code validation + HTTR reactor-level emulation exist.**

It still does not constitute project-scale validation for a 600 MWth reactor and 176.8 MWth reformer.

## Opposite-direction transient: loss of nuclear heat

The available Japanese programme is much stronger for disturbances originating in the hydrogen plant and propagating toward the reactor than for the chemical consequences of a reactor trip/rapid heat-source loss.

The project still lacks source-backed quantitative answers for:
- catalyst-bed cooling rate;
- methane/steam feed-isolation timing;
- minimum steam/carbon ratio during heat loss;
- coking/carbon-deposition risk;
- depressurisation sequence;
- product/flammable inventory during shutdown;
- restart criteria.

Therefore DF-11 remains **UNRESOLVED — PROCESS DYNAMICS / SAFE-STATE DESIGN REQUIRED**.

A future model should begin from the chemical reactor inventory and reaction/coking kinetics, not simply reverse the HTTR process-trip transient.


## Chemical-side shutdown evidence cross-check

Industrial steam-reformer guidance (CGA H-11 / harmonised AIGA guidance) treats startup/shutdown as safety-critical transitional states and requires controlled steam/feed management, hot-restart criteria, adequate cooling steam, procedures/training and avoidance of conditions that damage catalyst/tubes.

Peer-reviewed industrial catalyst evidence confirms that steam-deficient/high-temperature conditions can promote methane-decomposition/CO-disproportionation carbon deposition, Ni sintering, catalyst fragmentation and rising pressure drop.

This supports the mechanism basis for DF-11:
**a loss of nuclear heat cannot be represented as “hydrogen production simply stops.”**
The chemical train requires a defined safe-state sequence.

What remains missing is the selected project's quantitative transient:
- how fast 176.8 MWth falls;
- reformer thermal inertia;
- steam inventory/availability;
- feed-isolation response;
- catalyst temperature/composition trajectory;
- carbon potential.

Decision consequence remains unchanged: a project-scale dynamic reformer model and shutdown philosophy are required before the coupling can be called operationally feasible.
