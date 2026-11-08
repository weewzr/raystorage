# HTGR Human Factors, I&C and Security Evidence

## Question

What human-factors, control-system and security issues become deployment gates when a modular HTGR is coupled to an industrial hydrogen/CCS plant?

## Chinese operating/design evidence — HTR-PM

Tsinghua INET's Computer and Control Research Division documents a dedicated **multi-module HTGR main-control-room and human-factors engineering verification platform** capable of simulating:
- two reactor modules;
- six reactor modules;
- ten reactor modules;
- shared turbine/power and heat-use configurations.

Its research scope includes:
- digital I&C;
- software verification and validation;
- fault warning/diagnosis;
- condition monitoring/life prediction;
- operator support;
- industrial-control-system cybersecurity;
- electrical-system reliability;
- advanced-reactor control.

HTR-PM implemented a full digital I&C system for the unusual **two-reactors-to-one-turbine** architecture, and Tsinghua reports successful coordinated multi-module operation. HTR-PM entered commercial operation in December 2023.

Evidence class:
**NATIVE ENGINEERING / OPERATING-PLANT CONTROL EXPERIENCE**.

## Why this matters to the project

The Singapore screening architecture adds interfaces beyond HTR-PM:
- reactor;
- IHX;
- secondary helium loop;
- reformer;
- WGS;
- CCS/PSA;
- CO2 export;
- backup heat;
- shared utilities.

The control problem is therefore not only “operate the reactor.” Operators/control systems must manage:
- reactor/process load following;
- reformer trip;
- reactor trip;
- IHX isolation;
- secondary-loop pressure/temperature;
- backup-heat transition;
- CCS interruption;
- hydrogen/CO/fire alarms;
- nuclear/radiological alarms;
- conflicting nuclear vs chemical safe-state priorities.

## Human-factors issue

A coupled nuclear/chemical plant can create **cross-domain cognitive load**:
- nuclear operators optimise defence-in-depth/reactor safety;
- chemical operators manage flammable/toxic inventories and catalyst/process constraints;
- emergency actions can interact.

A future design therefore needs:
- clear responsibility boundaries;
- interface alarm philosophy;
- common-cause event procedures;
- simulator-based validation;
- staffing/workload analysis;
- human-system interface V&V;
- nuclear/chemical emergency coordination.

No project-specific human-reliability analysis exists.

## Cybersecurity / I&C

Tsinghua's explicit industrial-control-system cybersecurity test capability is a useful native precedent. A Singapore hybrid plant would create additional digital interfaces between nuclear and non-nuclear systems.

The project should not assume that physical secondary-loop separation implies cyber/I&C independence.

Required future analysis:
- safety-system independence;
- data-diode/network segmentation;
- process-control vs safety-control interfaces;
- cyber design-basis threat;
- common power/communications dependencies;
- software V&V;
- supply-chain cybersecurity.

## Physical security

The project has not defined a design-basis threat, security perimeter or nuclear/chemical co-location security architecture.

Potential coupled concerns:
- hydrogen/chemical inventories near protected area;
- external shipping/industrial access;
- fresh HALEU/TRISO transport;
- spent-fuel/graphite storage;
- CO2 pipelines;
- emergency access versus security access control.

Security information is partly sensitive/design-specific; the research should define required analysis categories without inventing threat parameters.

## Decision consequences

If human-factors validation shows operators cannot reliably manage coupled nuclear/chemical transients:
**CONTROL ARCHITECTURE, AUTOMATION, STAFFING OR PHYSICAL DECOUPLING MUST CHANGE.**

If cybersecurity or physical-security separation cannot be achieved without unacceptable interfaces/land/cost:
**CO-LOCATION ARCHITECTURE OR SITE MAY BE INFEASIBLE.**

## New questions

DF-38:
**Can the coupled nuclear/chemical control architecture be validated for operator workload, alarm management and conflicting safe states?**
Status: **UNRESOLVED — REQUIRES INTEGRATED SIMULATOR / HUMAN-FACTORS ENGINEERING.**

DF-39:
**Can nuclear safety I&C and industrial process control be cyber/functional separated while retaining required coordination?**
Status: **UNRESOLVED — REQUIRES CYBER/I&C ARCHITECTURE AND V&V.**

DF-40:
**Can physical-security requirements coexist with industrial co-location, fuel/waste logistics and emergency access at a Singapore site?**
Status: **REQUIRES SITE-SPECIFIC SECURITY ANALYSIS.**

## Native evidence

- 清华大学核能与新能源技术研究院 计算机与控制研究室: multi-module HTGR control-room/human-factors validation platform; digital I&C; cybersecurity; V&V.
- HTR-PM commercial-operation/control evidence.
- Chinese NNSA/MEE HTR-PM operating licence and QA oversight as regulatory context.

## English cross-check

Future manuscript integration should cross-check the human-factors/cybersecurity requirements against IAEA nuclear-security/human-factors guidance and NRC human-factors engineering review guidance. No project-specific compliance claim is made here.


## English-language cross-check

### NRC human factors
NRC NUREG-0711/NUREG-0700 establish formal human-factors engineering review programmes for advanced reactor human-system interfaces. Relevant concepts include operating-experience review, function allocation, task analysis, staffing/qualifications, human-system interface design, procedure development, training, human-factors V&V and design implementation.

This independently supports the conclusion that the project's coupled operator/control problem requires a structured HFE programme, not only a process-control narrative.

### IAEA computer security
IAEA Nuclear Security Series No. 33-T states that interfacing ICT/control systems can introduce risks to nuclear I&C and must be included in computer-security design. It emphasises independence, redundancy, defence in depth, diversity, configuration control, V&V, graded security levels/zones and lifecycle security.

This cross-checks the Tsinghua native evidence: a hybrid plant must explicitly control interfaces between nuclear I&C and industrial control systems.

### NRC 2026 co-located hydrogen external-hazard work
NRC Research Information Letter 2026-04 is explicitly titled **External Hazard Risk Assessment Framework for a Co-Located Hydrogen Production Facility**. Its existence confirms that hydrogen co-location is an active nuclear regulatory research topic rather than a solved generic separation-distance problem.

Project implication:
DF-20 and the hydrogen fire/explosion rows of the hazard register remain **SITE-/LAYOUT-SPECIFIC**, but the need for a structured external-hazard risk framework is now independently regulator-supported.


## Chinese operating-event learning — human / I&C / auxiliary systems

NNSA operating-event records provide useful counter-evidence to purely design-based reliability arguments:

- **2023 reactor-protection configuration event:** a setting deviation introduced during a protection-system modification left the negative nuclear-power-rate protection channels of both reactors unable to operate at their required setpoint/accuracy. The units remained safe and the event was INES 0. This is direct evidence that configuration management and testing are human/organisational safety functions.
- **2024 main-helium-circulator event:** failure of a digital amplifier in a magnetic-bearing control cabinet stopped the main helium circulator and triggered an automatic reactor trip through a primary/secondary mass-flow-ratio protection signal. The other module continued operating; barriers remained intact and the event was INES 0.
- **2025 main-helium-circulator frequency-converter event:** another circulator-control failure caused a reactor trip; the unit returned to criticality after component replacement. Again, barriers remained intact and the event was INES 0.

Project implications:
1. helium-circulator/control availability is not a theoretical auxiliary issue;
2. multi-module architectures can provide operational resilience unavailable to a single-module heat source;
3. I&C configuration control, surveillance testing and maintenance must be included in reliability/HFE reasoning;
4. none of these events provides a transferable project failure rate.

These operating records strengthen DF-08, DF-38 and DF-39 without changing the conclusion that project-specific reliability/HRA remains unresolved.
