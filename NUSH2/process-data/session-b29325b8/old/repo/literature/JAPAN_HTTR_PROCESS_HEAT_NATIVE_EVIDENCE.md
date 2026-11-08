# HTTR / GTHTR300C Japanese-Native Process-Heat Evidence Ladder

## Purpose

Deepen DF-01 through DF-06 using JAEA/J-STAGE/Atomic Energy Society of Japan native evidence and distinguish **DEMONSTRATED**, **CONCEPTUALLY DESIGNED**, and **UNDER QUALIFICATION**.

## Evidence ladder

| Topic | Native Japanese evidence | Maturity | Project implication |
|---|---|---|---|
| HTTR 950 C reactor operation | JAEA / AESJ: 30 MWth HTTR completed 50-day full-power 950 C continuous operation in Jan-Mar 2010 | **DEMONSTRATED** | 950 C-class nuclear heat is experimentally demonstrated at 30 MWth |
| HTTR IHX | AESJ structural-design paper: 10 MW He-He IHX; 950 C primary → max 905 C secondary; Hastelloy XR ~930 C | **DEMONSTRATED HARDWARE** | direct precedent for hot primary/secondary helium isolation |
| HTTR IHX creep life | complete creep analysis satisfies design limit for 10^5 h HTTR life | **ANALYSED FOR BUILT COMPONENT** | high-T creep is an explicit design basis, not an afterthought |
| Hydrogen-isotope permeation | J-STAGE/JAEA tests on Hastelloy XR, 570-850 C | **EXPERIMENTAL** | IHX/reformer metal is permeable to H isotopes; oxide films reduce but do not eliminate permeation |
| HTTR tritium transport | JAEA measured primary and secondary helium tritium during 50-day 950 C operation | **MEASURED IN OPERATING REACTOR** | radiological isolation is finite; tritium must be assessed |
| HTTR heat-utilisation connection | JAEA current programme: primary 950 C → IHX → secondary helium → steam-methane reformer | **UNDER LICENSING / DEMONSTRATION PREPARATION** | direct nuclear-heated SMR coupling has not yet been demonstrated in HTTR |
| HTTR heat-utilisation legal boundary | 2025 JAEA/NRA review converged on reactor-law boundary at reactor-building isolation valve; H2 plant outside reactor-law scope conceptually | **REGULATORY REVIEW / DESIGN BASIS** | supports possibility of non-nuclear chemical plant, conditional on safety/isolation/radiological criteria |
| GTHTR300C 600 MWth | JAEA commercial cogeneration design | **CONCEPTUALLY / ENGINEERING DESIGNED** | source architecture, not operating evidence |
| GTHTR300C reference IHX | ~170 MWth, ~81 kg/s secondary He, ~900 C | **CONCEPTUALLY DESIGNED** | close interface match to INL 176.8 MWth / 78.49 kg/s |
| Horizontal GTHTR300C IHX | JAEA R&D reduces max tube stress from ~4.3 to 1.9 MPa; 40-year equivalent creep allowable comparison | **ADVANCED CONCEPTUAL DESIGN** | shows path to longer-life high-T exchanger |
| Horizontal IHX long-term validity | JAEA explicitly states long-duration creep-data acquisition / validation remains necessary | **UNDER QUALIFICATION** | 40-year lifetime is not experimentally established |
| Project 176.8 MWth duty | INL Case 6 | **SOURCE-MODELLED** | 4.0% above one 170 MW reference IHX; cannot claim one reference exchanger is qualified |
| Project 925→900→871 C cascade | INL Case 6 + JAEA 950→905 C hardware precedent | **SOURCE-MODEL + DEMONSTRATED TEMPERATURE PRECEDENT** | strong temperature compatibility, component lifetime still conditional |

## HTTR 950 C / 50-day operation

Iyoku et al. (AESJ Transactions, 2011) report full-power 950 C continuous operation for 50 days. The paper explicitly interprets the result as demonstrating the potential for stable high-temperature heat supply to applications such as hydrogen production.

Evidence tag: **DEMONSTRATED AT 30 MWth FOR 50 DAYS**.

Decision consequence:
- if high-temperature reactor operation itself were not demonstrated, the concept would be much weaker;
- it is demonstrated, so the remaining problem is scale/integration/lifetime rather than basic temperature feasibility.

## HTTR IHX and creep-fatigue / creep

The HTTR IHX structural paper states:
- 10 MW heat capacity;
- Hastelloy XR heat-transfer tubes;
- normal tube/internal structure temperature around 930 C;
- maximum secondary helium ~905 C;
- primary helium 950 C;
- creep strain/damage cannot be assessed adequately by simple elastic margins;
- complete creep analysis was required;
- calculated creep strain/damage met the 10^5 h design limit.

Evidence tag: **BUILT COMPONENT + HIGH-TEMPERATURE STRUCTURAL ANALYSIS**.

The project must therefore avoid wording such as “Hastelloy XR is proven for any 900 C industrial IHX.” What is proven is a particular HTTR component/design envelope.

## GTHTR300C 170 MWth IHX and lifetime

JAEA's later horizontal-IHX development for GTHTR300C addresses the large 170 MWth commercial-scale exchanger. The Japanese R&D summary reports:
- vertical design max generated stress ~4.3 MPa;
- horizontal design ~1.9 MPa;
- allowable stress corresponding to 40 years continuous operation ~2.6 MPa in the cited assessment;
- material/cost reduction relative to vertical concept.

But JAEA explicitly says **validation using long-duration creep data remains necessary** and detailed design should continue.

Evidence tag:
**170 MWth: CONCEPTUAL/ENGINEERING DESIGN.**
**40-year lifetime: ANALYTICAL TARGET, UNDER QUALIFICATION.**

This is exactly the distinction required for the project's 176.8 MWth duty.

## HTTR heat-utilisation test: direct relevance to nuclear-heated SMR

Current JAEA Japanese programme material is unusually relevant. The planned HTTR heat-utilisation test will:
1. heat primary helium to 950 C;
2. transfer heat through the IHX to secondary helium;
3. route secondary helium outside the reactor building;
4. transport it through new insulated high-temperature piping;
5. heat a **steam-methane reformer**.

JAEA states that the initial demonstration uses technically established steam methane reforming specifically to demonstrate reactor-to-hydrogen-plant coupling.

As of the current JAEA/NRA material, this remains **planned/under licensing**, with hydrogen-production testing targeted for 2028. It is not yet a demonstrated integrated nuclear-SMR plant.

This is direct Japanese evidence that the project's core integration question is scientifically relevant and still under qualification.

## Safety boundary / transient coupling

JAEA's heat-utilisation design adds:
- high-temperature isolation valves;
- secondary-helium circulator;
- high-temperature insulated transport pipe;
- plant simulation/control methods;
- separation distance between reactor and H2 facility;
- protection so H2-plant fire/explosion cannot affect the reactor.

Earlier HTTR-IS work uses steam generators/coolers as thermal buffers so rapid secondary-helium temperature changes do not propagate unmitigated to the reactor.

Evidence tag: **DESIGN / MOCK-UP / CURRENT DEMONSTRATION PROGRAMME**, not project validation.

## Material selection

### Hastelloy XR
Strongest direct HTTR evidence:
- actual IHX material;
- high-temperature operation;
- creep analysis;
- hydrogen/tritium permeation experiments and operating-reactor measurements.

### Alloy 617
Relevant to broader VHTR/high-temperature exchanger development, but it is not the material of the built HTTR IHX. The manuscript should not substitute Alloy 617 evidence for Hastelloy XR evidence unless discussing alternative future component designs.

## Decision consequences

- If no qualified 176.8 MWth-class exchanger/material solution can meet creep-fatigue, permeation, inspection and lifetime requirements → **CURRENT ARCHITECTURE INFEASIBLE AT PROJECT SCALE**.
- If one 170 MWth reference IHX cannot be scaled/modified safely → project requires multiple exchangers or redesigned IHX → **CAPEX/layout/reliability must be recalculated**.
- If coupled heat-load transients cannot be isolated from reactor safety functions → **NUCLEAR/CHEMICAL INTEGRATION ARCHITECTURE MUST CHANGE**.
- If secondary-helium radiological levels cannot support the intended legal/classification boundary → **CHEMICAL PLANT CLASSIFICATION, COST AND LICENSING ASSUMPTIONS CHANGE**.

## Native sources

- 伊与久達夫ほか, 「HTTRの高温連続運転」, 日本原子力学会和文論文誌 10(4), 290-300 (2011).
- 「高温工学試験研究炉の中間熱交換器の構造設計」, 日本原子力学会誌 37(4), 316-326 (1995).
- 武田哲明ほか, 「HTTR水素製造システムにおけるハステロイXRの水素同位体透過係数」, 日本原子力学会誌 42(3), 204-211 (2000).
- 坂場成昭ほか / JAEA, HTTR 950 C operation hydrogen-permeation measurements.
- JAEA研究開発成果 2017-18, 高温ガス炉熱利用向け高温機器 / horizontal IHX.
- JAEA-Technology 2022-011, HTTR heat-utilisation safety design.
- JAEA 2024/2025 Japanese HTTR heat-utilisation programme and licensing material.
