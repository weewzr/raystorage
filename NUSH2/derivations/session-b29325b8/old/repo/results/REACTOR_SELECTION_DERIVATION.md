# Reactor / Temperature Selection Derivation — Deep Feasibility

## Governing rule

The reactor basis is selected **from the chemical-process requirement outward**. The project does not choose a reactor first and force the process to fit.

## 1. Verified process requirement

INL TEV-961 Case 6 is the controlling process source:
- H2 production: 130 MMSCFD;
- reformer/process outlet reference: **871 C**;
- nuclear process-heat duty: **176.8 MWth**;
- secondary helium supply: **900 C**;
- secondary helium return: **466 C**;
- helium flow: **78.49 kg/s**;
- reactor outlet-temperature case: **925 C**.

TEV-961 also reports an equivalent reactor thermal size of ~178.8 MWth for heat + process power at this operating point. The project nevertheless uses a 600 MWth-class source architecture because the selected JAEA design family is modular/commercial-scale and its published economic/heat-branch architecture is used coherently; 600 MWth is not derived by multiplying 176.8 MWth by an arbitrary margin.

## 2. Process-side temperature requirement

The chemical model requires the reforming process to reach the reported 871 C outlet condition.

A heating medium must be hotter than the process it heats. TEV-961's selected source case supplies helium at 900 C.

At the reported outlet reference:

DeltaT_process,reported
= T_He,supply - T_reformer,out
= 900 - 871
= **29 C**.

This is a positive source-backed approach at the reported locations.

**Important:** it is not a full local reformer pinch or tube-wall calculation. The local heating-side, tube-wall, catalyst-bed and gas temperature profiles are not all available from TEV-961, so no smaller local minimum approach is invented.

## 3. Primary-helium / reactor-outlet requirement

TEV-961 defines the secondary helium supply 25 C below the reactor outlet-temperature case.

DeltaT_IHX,source
= T_reactor,out - T_He,secondary,supply
= 925 - 900
= **25 C**.

Therefore the source process chain is:

**925 C reactor-outlet case
→ 25 C source primary/secondary temperature allowance
→ 900 C secondary helium
→ 29 C reported supply-to-reformer-outlet difference
→ 871 C reformer outlet.**

Question A — can a reactor produce this temperature?
Question B — can an IHX transfer useful heat at this temperature?
Question C — can the reformer receive the required 176.8 MWth at its required temperature?

A yes to A does not prove B or C.

## 4. Secondary-helium energy consistency

Screening energy balance:

Q = m_dot_He * cp_He * (T_supply - T_return).

Using:
Q = 176.8 MW = 176,800 kJ/s;
m_dot = 78.49 kg/s;
DeltaT = 900 - 466 = 434 K.

Implied average cp:

cp = 176,800 / (78.49 * 434)
≈ **5.19 kJ/(kg K)**,

consistent with the canonical 5.2 kJ/(kg K) screening value.

Thus heat duty, flow and source temperatures are internally consistent.

## 5. Reactor technology temperature filter

| Candidate family | Representative thermal scale | Core/outlet temperature | Demonstrated / designed status | Direct ~900 C heating-medium fit? | Disposition |
|---|---:|---:|---|---|---|
| Conventional / integral LWR-SMR | ~100–870 MWth examples | ~287–326 C | mature LWR physics; several SMR designs licensed/construction | **NO direct fit**; high-T process needs steam/electric/other augmentation | REJECT AS DIRECT HIGH-T PROCESS-HEAT BASIS; retain as alternative augmented-energy pathway |
| HTTR | 30 MWth | 950 C | operating experimental reactor; 950 C demonstrated, 50-day run | **YES temperature** | EVIDENCE / DEMONSTRATOR ONLY — scale mismatch |
| HTR-PM | 2 × 250 MWth modules | helium HTGR; demonstrated steam-cycle plant with ~567 C steam | commercial operation; strong safety/licensing maturity | current demonstrated architecture does **not directly establish 900 C secondary-He reformer supply** | COMPARATOR — safety/licensing/HTGR maturity; not selected direct process-heat basis |
| GTHTR300 | 600 MWth | 850 or 950 C design variants | JAEA engineering design | relevant high-T source family | RETAINED TECHNOLOGY FAMILY / precursor |
| GTHTR300C | 600 MWth | 950 C; secondary He 900 C | JAEA cogeneration/process-heat design | **YES by design** | RETAINED AS DESIGN BASIS, CONDITIONAL ON IHX qualification |
| Project | 600 MWth-class | **925 C project/source process case** feeding 900 C secondary He | screening architecture | designed to match verified process case | **GTHTR300C-INFORMED / GTHTR300C-CLASS**, not exact GTHTR300C |

## 6. Why not an LWR / integral LWR-SMR?

IAEA representative water-cooled SMRs have core-outlet temperatures around:
- BWRX-300 ~287 C;
- NuScale ~300–321 C depending design generation/source;
- ACP100/SMART/CAREM ~320–326 C.

These technologies can provide electricity, district heat, desalination and process steam. They are **not generally inferior reactors**.

For this application, however, the desired heating-medium temperature is ~900 C. Direct water-cooled reactor heat is hundreds of degrees below that requirement.

A water-cooled reactor route would therefore require a separate temperature-augmentation pathway such as:
- electrical resistance/electric heating;
- steam compression/superheating;
- another fired heater;
- a different high-temperature process architecture.

NuScale's own process-heat work explicitly uses steam conditioning/augmentation for higher-temperature applications.

Therefore the rejection is application-specific:

**LWR-SMR: not retained as the DIRECT 900 C-class heat source.**

It remains a valid alternative if the project question changes from direct nuclear heat to nuclear electricity + temperature augmentation.

## 7. Why not HTTR itself?

HTTR demonstrates the required temperature class:
- 30 MWth;
- 950 C maximum reactor outlet;
- 50-day continuous 950 C operation.

But:

176.8 / 30
= **5.89**.

The project's process duty alone is nearly 5.9 times HTTR total thermal power.

This ratio is illustrative only. Reactor scale-up is not achieved by placing 5.89 HTTRs together or linearly enlarging HTTR.

Disposition:

**HTTR = experimental evidence / demonstrator, not project-scale heat source.**

## 8. Why not HTR-PM as the process-heat basis?

HTR-PM is crucial evidence because it is an operating/commercial modular HTGR and has Chinese regulator/licensing, fuel, helium-system, source-term, human-factors and emergency-planning experience.

Its demonstrated plant architecture is:
- two 250 MWth pebble-bed modules;
- one steam turbine;
- ~212/210 MWe;
- steam parameter reported around 567 C in Tsinghua material.

That architecture does not directly establish the project's required 900 C secondary-helium supply to a reformer.

Therefore:

**HTR-PM = high-value safety/licensing/operational comparator, not the most direct process-heat source basis for this 900 C requirement.**

A future high-temperature HTR-PM-derived process-heat design could change this conclusion, but the operating demonstration itself is not that system.

## 9. Why GTHTR300 / GTHTR300C?

JAEA's GTHTR300C family directly matches the required interface:

Reference cogeneration design:
- reactor thermal power: **600 MWth**;
- reactor outlet: **950 C**;
- secondary helium at IHX outlet: **900 C**;
- reference H2 cogeneration process heat: **170 MWth**;
- higher-H2-production variant process heat: **371 MWth**;
- direct Brayton power cycle;
- prismatic TRISO HTGR;
- dedicated secondary helium loop, circulator, isolation/safety valves, purification and storage.

This is the only screened family here that simultaneously provides:
1. ~900 C secondary helium by published design;
2. process-heat / hydrogen cogeneration architecture;
3. a 600 MWth commercial-scale reactor design;
4. a published IHX/secondary-loop basis;
5. economics used by the project's screening model.

Therefore GTHTR300C is retained **because it passes the process temperature, duty/scale, architecture and evidence filters**, not because it gives the most favourable result.

## 10. Why “GTHTR300C-class” rather than “GTHTR300C”?

The project is not the exact published JAEA plant.

Borrowed from JAEA:
- 600 MWth prismatic HTGR class;
- high-temperature helium architecture;
- GTHTR300C source economic architecture;
- 370/371 MWth high-H2 process-heat branch class;
- secondary-helium/IHX system concept;
- published 170 MWth reference IHX evidence;
- 950/900 C hardware/design precedent.

Project-specific / INL-derived:
- reactor outlet-temperature operating case: **925 C**, not exact 950 C JAEA design point;
- secondary helium: 900 C;
- reformer outlet: 871 C;
- process duty: 176.8 MWth;
- helium flow: 78.49 kg/s;
- SMR + amine CCS process;
- no exact project electric output claimed.

Correct terminology:

**GTHTR300C-INFORMED 600 MWth SCREENING ARCHITECTURE**
or
**GTHTR300C-CLASS**.

## 11. Thermal-power fraction

Process fraction:

f_process = Q_process / Q_reactor
= 176.8 / 600
= **0.2947 = 29.5%**.

Remaining reactor thermal capacity identity:

600 - 176.8
= **423.2 MWth**.

This is remaining capacity, not waste heat and not claimed electricity.

## 12. What exactly is the 370/371 MWth branch?

JAEA-Technology 2011-013 distinguishes GTHTR300C variants:
- H2 + power cogeneration: process heat **170 MWth**, electricity 202 MWe;
- high-H2-production variant: process heat **371 MWth**, electricity 87 MWe.

The canonical repository uses **370 MWth** as the source heat/IHX-branch class and **88 MWe** as the corresponding source economic electricity product, reflecting the Nishihara/JAEA source basis/rounding.

Therefore 370 MWth is **source-design process-heat branch capability / economic architecture**, not the rating of one physical IHX.

Project process utilisation of this source branch:

176.8 / 370
= **47.8%**.

Nominal branch margin:

370 - 176.8
= **193.2 MWth**.

This is a thermal-capacity screen only. It does not prove a single exchanger can transfer 176.8 MWth.

## 13. The 170 MWth IHX issue

Published GTHTR300C literature includes a conceptual **170 MWth** IHX.

Project duty exceeds it by:

176.8 - 170
= **6.8 MWth**.

Ratio:

176.8 / 170
= **1.040 = 104.0%**.

Thus the project is ~4.0% above one published reference IHX duty.

Engineering possibilities include:
- modestly larger IHX;
- parallel/multiple IHXs;
- different duty allocation;
- redesigned exchanger/loop.

No option is selected without component-design evidence.

Disposition:

**IHX SCALE/LIFETIME REMAINS CONDITIONAL.**

If no qualified arrangement can transfer 176.8 MWth at the required temperature/lifetime, the current architecture is infeasible or must be redesigned.

## 14. Temperature selection table

| Location | Temperature | Meaning / why needed | Source | Evidence maturity |
|---|---:|---|---|---|
| HTTR reactor outlet | 950 C | demonstrates achievable HTGR temperature class | JAEA operating HTTR | DEMONSTRATED |
| GTHTR300C design reactor outlet | 950 C | JAEA commercial process-heat design basis | JAEA | DESIGNED |
| Project/INL reactor-outlet case | **925 C** | minimum selected INL case that supplies full reforming heat without fired trim | INL TEV-961 | SOURCE-MODELLED |
| GTHTR300C secondary He outlet | 900 C | JAEA secondary process-heat design | JAEA | DESIGNED |
| Project secondary He supply | **900 C** | heats reformer to source process condition | INL TEV-961 | SOURCE-MODELLED; temperature class supported by JAEA |
| Project secondary He return | **466 C** | closes 176.8 MWth / 78.49 kg/s source heat balance | INL TEV-961 | SOURCE-MODELLED |
| Project reformer outlet | **871 C** | selected chemical-process result | INL TEV-961 | SOURCE-MODELLED |
| Local tube-wall / catalyst pinch | not established | required for detailed exchanger/reformer qualification | — | UNRESOLVED — DO NOT GUESS |

## 15. Evidence chain

**SMR-H2 process**
→ 176.8 MWth process duty + 871 C reformer outlet
→ positive source-backed heating-medium margin
→ 900 C secondary helium
→ 25 C INL primary/secondary temperature allowance
→ 925 C reactor-outlet source case
→ water-cooled reactor families fail direct-temperature filter
→ HTTR passes temperature but fails scale
→ HTR-PM provides operating HTGR maturity but not demonstrated 900 C secondary-He process architecture
→ GTHTR300C passes temperature + scale + process-heat architecture filters
→ retain **GTHTR300C-class / GTHTR300C-informed screening basis**
→ keep 176.8 MWth IHX qualification conditional.

## Decision consequences

- If 900 C secondary heat cannot be supplied → direct nuclear-heated SMR architecture infeasible.
- If positive local reformer thermal margins cannot be achieved → reformer/IHX geometry or operating point must change.
- If 176.8 MWth-class IHX cannot be qualified → architecture must use another exchanger arrangement or fails.
- If GTHTR300C-class fuel/material/safety supply chain cannot mature → selected design basis becomes non-deployable even if thermodynamics pass.
- If an alternative reactor plus augmentation later proves more practical, it can be compared as a different architecture; it does not invalidate why GTHTR300C-class is the current **direct-heat** screening basis.

## Primary / authoritative sources

- INL TEV-961, Case 6.
- JAEA-Technology 2008-093.
- JAEA-Technology 2011-013.
- JAEA / AESJ HTTR 950 C / 50-day operating evidence.
- IAEA SMR technology status reports for representative water-cooled SMR outlet temperatures.
- Tsinghua INET HTR-PM demonstration-plant technical material.
