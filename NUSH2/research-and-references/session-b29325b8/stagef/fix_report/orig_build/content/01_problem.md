# 1. Problem and Proposed Solution

## 1.1 The problem and its Singapore context

Singapore's energy transition requires reliable industrial energy while reducing carbon emissions. With no domestic natural gas resources, Singapore relies on imported pipeline gas and liquefied natural gas (LNG); natural gas accounted for approximately 94% of electricity-generation fuel in 2024 [@r1,r2]. This dependence highlights the strategic importance of diversifying low-carbon energy supply, particularly for continuous high-temperature industrial processes.

Hydrogen offers a pathway for decarbonising industrial applications and supporting Singapore's future energy system. The National Hydrogen Strategy (October 2022) identifies low-carbon hydrogen as a potential contributor to up to half of the country's power needs by 2050 [@r3]. SMR is an established hydrogen-production technology; it generates CO₂ through methane conversion and conventionally burns fossil fuel to supply the endothermic reforming duty [@r4]. Addressing both emissions sources creates an opportunity to integrate nuclear process heat with carbon capture and storage (CCS).

CCS captures the CO₂ generated in the reforming train [@r5], while nuclear process heat displaces natural gas combustion for the reforming duty [@r8]. Steam reforming of natural gas operates within approximately 500 to 900[[degC]] ({fig:f_heat}) [@r6], a range that HTGRs can serve and light-water reactors cannot.

!!figure f_heat

This report proposes NUS-H₂, a single-train HTGR-heated SMR integrated with MEA carbon capture, as a new build on the proposed Western Island. The site is a proposal of this study and has no official approval. The plant is sized on the SOXAL hydrogen plant on Jurong Island, about {{H2_B_NM3H|u}} [@airliquide2008,airliquide2011], and is compared with a same-size unabated SMR delivering equal hydrogen in equal hours. The design targets are net abatement above {{TARGET_ABATEMENT_MT|f2|u:MtCO2e/y}} within Singapore, at a cost below {{TARGET_COST_S_T|f0|u}}.

## 1.2 Proposed solution

The proposed configuration integrates a {{R_RATED_MWTH|f0|u}} HTGR with an SMR hydrogen-production system ({fig:f_pfd}). Nuclear fission heats helium in the primary reactor circuit, and an IHX transfers thermal energy to an independent secondary helium loop supplying the reformer. This arrangement replaces fossil-fired reformer heating and keeps the nuclear and chemical systems physically separate [@r9]. Reactor capacity beyond the process duty covers the MEA reboiler gap and {{E_TOTAL_B_MWE|u}} of site electricity, and the remaining {{MARGIN_B_MWTH|u}} stays as spare with no revenue.

Natural gas and steam react over a nickel catalyst in the helium-heated reformer to produce hydrogen-rich synthesis gas [@r4,r10]. Water-gas shift (WGS) conversion increases hydrogen yield and converts carbon monoxide to CO₂. After syngas cooling and condensate removal, a 30~[[wt%]] aqueous MEA system absorbs CO₂ from the hydrogen-rich gas. A stripper regenerates the rich solvent, releasing concentrated CO₂ for compression and shipment to offshore storage. The Idaho National Laboratory (INL) reference removes CO₂ with Selexol [@r8,r10], so MEA regeneration energy is assessed separately from INL's utility data. Pilot tests of 30~[[wt%]] MEA on flue gas report specific reboiler duties of approximately 3.46 to 4.0~[[GJ/tCO2]] [@r31,r32]. The lower value excludes heat loss at 26 to 27~[[wt%]], the upper value is at 85% capture on 3.5~[[vol%]] CO₂, and no measurement on syngas was found.

!!figure f_pfd

The PSA stays an unsimulated external block. Its reject returns across the boundary through a recycle compressor and a cryogenic unit, as in INL's Case 6 [@r8]. Methane-rich gas goes back to the reformer feed, hydrogen returns ahead of the PSA, and a nitrogen-rich cut is burned.

## 1.3 Design basis

{tab:t_basis} sets the headline Case B beside the INL reference Case A. Case B scales the extensive items of INL's 130 MMSCFD Case 6 by {{SCALE_S}} to the SOXAL rate, with the reactor resized. Hydrogen reaching the PSA is {{RES_PSA_H2_FEED_KG_H|u}}. At the published {{PSA_RECOVERY_PCT|f0}}% recovery, {{H2_B_KG_H|u}} leaves as purified product and {{RES_PSA_H2_REJECT_KG_H|u}} returns in the reject. Annual output counts the {{AVAILABILITY_PCT|f0}}% availability hours only ({{HOURS_Y|u}}).

!!table t_basis
