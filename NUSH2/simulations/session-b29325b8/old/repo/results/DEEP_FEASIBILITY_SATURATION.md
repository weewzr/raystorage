# Deep Feasibility Research Saturation Assessment

## Purpose

Determine whether another broad literature/research pass can materially improve the current screening conclusions, versus whether remaining questions now require new project-specific models, experiments, site data, vendor data, regulatory decisions or missing professor-source recovery.

## Saturation rule

A question is research-saturated for the present project when:
1. primary/authoritative evidence establishes the relevant mechanism or technology maturity;
2. supporting and contradictory evidence have been considered;
3. the project's allowed claim is clear;
4. the unresolved remainder cannot be closed by another generic literature search without inventing project-specific inputs;
5. the decision consequence if unfavourable is explicit.

Research saturation does **not** mean deployment feasibility is proven.

## Saturated evidence domains

### Process heat / reactor selection
Saturated for screening:
- 871 C reformer / 176.8 MWth process requirement;
- 900 C secondary-helium requirement;
- 925 C INL reactor-outlet case;
- HTTR 950 C demonstrated precedent;
- LWR direct-temperature mismatch;
- HTR-PM comparator role;
- GTHTR300C-class direct-heat selection;
- 370/371 MWth branch meaning;
- 170 MWth reference-IHX qualification gap.

Remaining work is component/vendor design, not broad literature.

### HTGR high-temperature materials / IHX
Saturated for maturity classification:
- built 10 MW HTTR IHX;
- Hastelloy XR creep/permeation evidence;
- conceptual 170 MW GTHTR300C IHX;
- long-life qualification gap.

Remaining work requires project exchanger design, creep-fatigue/inspection qualification and long-duration material data.

### Reliability / backup heat
Saturated for screening:
- single-source industrial continuity is a real issue;
- HTR process-heat literature supports redundancy/backup;
- HTR-PM operating events provide counter-evidence to idealised availability;
- Rust no-backup and gas-backup screens quantify assignment robustness.

Remaining work requires reliability/PRA, backup equipment design and cost, not more generic availability citations.

### Air ingress / water ingress
Saturated at mechanism level using Japanese, Chinese and NRC evidence.
Remaining work requires selected-design event sequence, thermal hydraulics, chemistry and source-term calculation.

### Source term / EPZ
Saturated for methodology and transferability:
- nuclide-specific TRISO evidence;
- graphite/primary-circuit/dust mechanisms;
- Chinese sequence-specific source-term principle;
- HTR-PM research EPZ versus implemented 3/7/30 km plan;
- scalable EPZ is possible but no radius transfers to Singapore.

Remaining work requires project PRA, mechanistic source term, site meteorology, dispersion, dose and Singapore protective-action policy.

### Tritium
Saturated for mechanism/maturity:
- production pathways established;
- Hastelloy XR permeation experimentally measured;
- non-zero secondary-helium tritium measured in HTTR;
- mitigation/classification approach exists.

Remaining work requires project inventory/permeation/purification/product model and Singapore limits.

### Spent fuel / graphite waste
Saturated for technology categories:
- dry storage is feasible and implemented in HTR programmes;
- dry storage still has ventilation/confinement/maintenance hazards;
- graphite/TRISO long-term disposal remains an R&D/regulatory issue.

Remaining work requires selected fuel/vendor inventory, package/storage design and national disposal policy.

### Cooling / siting
Saturated for option identification:
- 423.2 MWth is not cooling duty;
- Singapore has once-through seawater/SWCT/NEWater options;
- coastal access trades against flood/sea-level/intake/marine hazards;
- Jurong, future western island, other coastal, underground and offshore options remain open.

Remaining work requires an off-design heat-rejection model and candidate-site survey; no site can be selected from current evidence.

### Singapore regulatory readiness
Saturated for current-status assessment:
- no deployment decision;
- INIR Phase 1 from 2027;
- existing radiation/safeguards capabilities;
- power-reactor licensing/site/waste/EPR/security infrastructure remains to be developed.

Future national policy/INIR findings are external dependencies.

### CCS
Saturated for current screening:
- cross-border cooperation exists but no operating contracted project chain;
- partial-capture/storage Rust sensitivity implemented;
- assignment threshold is robust even to zero delivered captured stream in the narrow screen, but direct emissions worsen.

Remaining work requires real contracts, route/storage availability and dynamic outage/capture-unit modelling.

### Human factors / I&C / security
Saturated for issue identification:
- HTR-PM has operating digital I&C and HFE experience;
- NNSA events demonstrate real configuration/circulator/control issues;
- NRC/IAEA require structured HFE/cyber/external-hazard analysis.

Remaining work requires project simulator, HRA, cyber architecture, design-basis threat and site layout.

### Fuel supply
Saturated for dependency identification:
- GTHTR300-class fuel is HALEU-class LEU/TRISO;
- Japan has HTGR fabrication experience;
- commercial qualified project supply and Singapore procurement remain unresolved.

Remaining work is vendor/safeguards/procurement policy.

## Waived unavailable source

### Professor / research transcript 2
The requested second professor/research transcript/slides are not available in the repository/project sources inspected to date.

On 2026-10-01, the user explicitly **waived this source-recovery requirement** and instructed Main Research to focus on the evidence actually available.

This is recorded as a waiver, **not** as recovered/completed professor evidence. No claims are attributed to the missing source and no general literature is substituted for it.

## Higher-fidelity questions that should NOT trigger another broad literature search

- reactor-trip → reformer/catalyst safe-state transient;
- project 176.8 MWth IHX mechanical design/lifetime;
- helium piping pressure drop/circulator power without a selected layout;
- mechanistic project source term/PRA;
- atmospheric dispersion/dose/EPZ;
- chemical QRA/fire/explosion separation distance;
- exact project cooling duty;
- candidate-site flood/external-hazard/security assessment;
- project tritium concentrations;
- bankable FOAK cost;
- contracted CCS availability.

These require new inputs/models/designs rather than citation accumulation.

## Manuscript integration status

Integrated:
- process-first reactor selection;
- temperature/heat-duty derivation;
- reactor screen;
- IHX scale qualification;
- tritium;
- Chinese EPZ distinction;
- process-trip evidence;
- explicit CO2 and cost derivations;
- availability/CCS robustness;
- Singapore site/cooling option logic;
- deep-feasibility interpretation.

Still to verify:
- clean Research CI;
- clean Paper/reproducibility CI;
- bibliography convergence;
- exact PDF visual QA after the integrated build.

## Gate decision

**SCIENTIFIC LITERATURE SATURATION: REACHED FOR AVAILABLE SOURCES — INTEGRATED CI/PDF QA PASSED.**

The missing second professor/research source is not a review-readiness blocker because the user explicitly waived it; it remains documented as unavailable and is not treated as evidence.

## Final integration verification

Verified scientific/manuscript HEAD:
`c6a811b4756541e7aa2e3536c6804765f27a5c19`

- Research CI `36805989847`: PASS.
- Paper/reproducibility CI `36805989839`: PASS.
- Artifact `11137329060`, digest `sha256:b5953b5a193c1991c2fcc19ac60b4858ad7cc29b8e226907294dd0831323c68b`.
- `main.pdf`: 27 pages.
- Bibliography converged.
- Successful workflow integrity gate reports zero unresolved citations/references.
- Exact artifact rendered page-by-page and visually inspected: PASS.
- No scientific contradiction was exposed by integration QA.

## Gate decision

**DEEP FEASIBILITY RESEARCH: COMPLETE FOR AVAILABLE SOURCES.**

**READY FOR DEEP FEASIBILITY INDEPENDENT REVIEW.**

Do not continue broad Main Research. Remaining unresolved items require higher-fidelity engineering, site/vendor/regulatory inputs or experiments and should be assessed as limitations/gates by the Independent Reviewer rather than filled with additional generic literature.