1. Problem and proposed solution

1.1 The problem and its Singapore context

Singapore’s energy transition requires reliable industrial energy while reducing carbon emissions. With no domestic natural gas resources, Singapore relies on imported pipeline gas and liquefied natural gas (LNG); natural gas accounted for approximately 94% of electricity-generation fuel in 2024 [@r1,r2]. This dependence highlights the strategic importance of diversifying Singapore’s low-carbon energy supply, particularly for continuous high-temperature industrial processes.

Hydrogen offers a pathway for decarbonising industrial applications and supporting Singapore’s future energy system. Singapore’s National Hydrogen Strategy  2022) identifies low-carbon hydrogen as a potential contributor to up to half of the country’s power needs by 2050 [@r3]. Steam methane reforming (SMR) is an established hydrogen-production technology; it generates CO₂ through methane conversion and conventionally burns fossil fuel to supply the endothermic reforming duty [@r4]. Addressing both emissions sources creates an opportunity to integrate nuclear process heat with carbon capture and storage (CCS).

CCS captures the CO₂ generated in the reforming train [@r5], while nuclear process heat displaces natural gas combustion for the reforming duty [@r8]. Steam reforming of natural gas operates within approximately 500 to 900 [[degC]] (Figure 1) [@r6], and the INL reference reformer operates at 871 [[degC]] [@r8]. High-temperature gas-cooled reactors (HTGRs) can deliver heat approaching 950 [[degC]] [@r7], closely matching this duty. Direct thermal integration provides continuous heat to the reformer and reduces natural gas consumption for hydrogen production [@r8].

This report proposes a single-train HTGR-assisted SMR system integrated with monoethanolamine (MEA) carbon capture for future deployment on Singapore’s Jurong Island. The configuration builds on the island’s established energy and petrochemical infrastructure [@r29]. Its performance is evaluated against conventional unabated SMR at equal hydrogen output, using annual in-scope operational abatement, incremental abatement cost and implementation feasibility as the principal assessment criteria.

[[IMAGE]]

Figure 1. Process heat temperature ranges of industrial applications, with the ranges that light-water reactors (LWRs) and HTGRs can serve.

1.2 Proposed solution

The proposed configuration integrates a 230 [[MWth]] HTGR with an SMR hydrogen-production system (Figure 2). Nuclear fission heats helium in the primary reactor circuit, and an intermediate heat exchanger (IHX) transfers thermal energy to an independent secondary helium loop supplying the reformer. This arrangement replaces fossil-fired reformer heating and keeps the nuclear and chemical systems physically separate [@r9]. Reactor thermal capacity beyond the reforming duty is allocated to auxiliary electricity generation.

[TABLE]

[/TABLE]

Figure 2. Proposed HTGR-assisted SMR + MEA CCS process arrangement. A 230 [[MWth]] HTGR supplies reformer heat through an IHX and secondary helium circuit; PSA and external tail-gas handling lie outside the quantitative boundary.

The thermal design adopts the INL HTGR-integrated SMR reference conditions of TEV-953 and TEV-961 [@r8,r10] (Table A1). In the 925 [[degC]] reactor-outlet case, secondary helium reaches the reformer at 900 [[degC]] and 78.49 [[kg/s]] and returns at approximately 466 [[degC]] after delivering 176.8 [[MWth]] of process heat [@r8]. For this case, INL reports an equivalent reactor size of 178.8 [[MWth]] and an electricity demand of 17.3 [[MWe]] [@r8]. Converting the remaining electricity deficit to reactor heat at INL’s 39.7% power-generation efficiency gives a combined screening requirement of 220.4 [[MWth]]. A nominal 230 [[MWth]] reactor is selected, leaving 9.6 [[MWth]] (4.4%) unallocated (Table A2). The allocation is a screening estimate; IHX losses, steam integration and MEA reboiler heat recovery require reconciliation.

Natural gas and steam react over a nickel catalyst in the helium-heated reformer to produce hydrogen-rich synthesis gas [@r4,r10]. Water-gas shift (WGS) conversion increases hydrogen yield and converts carbon monoxide to CO₂. After syngas cooling and condensate removal, a 30 wt% aqueous MEA absorption system separates CO₂ from the hydrogen-rich gas. A stripper regenerates the rich solvent, releasing concentrated CO₂ for compression, transport and geological storage, and the lean solvent returns to the absorber. The INL reference removes CO₂ with a Selexol physical solvent [@r8,r10]; the proposed design uses 30 wt% MEA, whose regeneration energy is assessed separately from the INL utility data. Pilot tests of 30 wt% MEA on flue gas report specific reboiler duties of approximately 3.46 to 4.0 [[GJ/t]] under their tested conditions [@r31,r32]. The INL reference sets a hydrogen-production basis of 130 MMSCFD [@r8], approximately 97,946 [[tH2/y]] at 85% availability, applied to both the proposed system and the unabated comparator. This value is the INL purified-product reference; the quantitative study covers nuclear heat delivery, SMR, WGS and MEA capture and ends at the hydrogen-rich gas and captured CO₂ outlets. Downstream PSA and external tail-gas handling lie outside the agreed boundary; their emissions are excluded from the calculation, and reported results are in-scope operational abatement.
