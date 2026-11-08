# 2. Technical and Practical Feasibility

## 2.1 Process heat and integration

The proposed system combines established SMR and CO₂-capture operations with continuous high-temperature heat. Helium heats the reformer directly, so no natural gas is burned for reforming heat, and the chemical plant retains established reforming and amine equipment. Japan's High Temperature Engineering Test Reactor reached a 950[[degC]] outlet [@r11], yet it is a 30~[[MWt]] test reactor. China's HTR-PM entered commercial operation in December 2023 [@r12] and runs near 750[[degC]] [@wnn2023]. This work identified no commercial reactor at 900[[degC]], so the design assumes a generic {{ROT|f0}}[[degC]] HTGR and treats Xe-100, also near 750[[degC]] [@xenergy], as a labelled analogue only. Secondary helium reaches the reformer at {{T_HE_SUPPLY|f0}}[[degC]] and returns at {{T_HE_RETURN|f0}}[[degC]] [@r8]. The {{SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C|f0}}[[degC]] reformer outlet is INL's equilibrium assumption; sensitivities at 825 and 800[[degC]] are named in Appendix E and were not run in this model.

## 2.2 Thermal and electrical balance

{tab:t_heat} closes the balance at the headline basis. The sizing walk starts from {{VAL_SIZING_WALK_STEP_0_SIM|u:MWth}} at INL's rate with no losses, which scales to {{VAL_SIZING_WALK_STEP_1_SIM|u:MWth}} at Case B; {tab:t_walk} gives every step. Counting the purge load, heat losses, helium circulators and the reboiler gap gives a need of {{R_NEEDED_B_MWTH|u}}, leaving {{MARGIN_B_MWTH|u}} of the rating spare with no revenue. The spare depends on the heat basis: on Ahn and Lee's {{Q_PROCESS_AHN_CASE_B|u:MWth}} of process heat [@ahn2026] it falls to {{SENS_HEAT_BASIS_143P4_MARGIN_210|u}}, and the adverse corner of the sizing range needs {{R_HIGH_B_MWTH|u}}, {{ADVERSE_SHORT_MWTH|u}} above the rating. Case A with MEA and recycle needs {{R_NEEDED_A_MEA_MWTH|u}}, so it stays a reference and is never printed as a {{R_RATED_MWTH|f0|u}} plant.

!!table t_heat

## 2.3 MEA carbon capture

The design uses 30~[[wt%]] MEA as the reference solvent, with Selexol and activated MDEA (aMDEA) as sensitivities. The literature is mixed: IEAGHG 2022-07 calls MEA the preferred chemical solvent [@ieaghg2022], while IEAGHG 2017-02 favours aMDEA on shifted syngas [@r5]. The absorber runs at 40[[degC]] and {{P_ABSORBER|f1|u:bar}} with rich loading capped at {{LOADING_RICH|f2}}~[[mol/mol]]. Per-pass capture of {{PER_PASS_CAPTURE_PCT|f0}}% is a project assumption. The specific reboiler duty (SRD) of {{SRD_CENTRAL|f1|u:GJ/tCO2}}, range {{SRD_LOW|f1}} to {{SRD_HIGH|f1}}, is assumed from flue-gas pilots, since no syngas measurement exists. The {{REBOILER_B_MW|u}} reboiler draws first on freed Selexol steam ({{SELEXOL_FREED_B_MW|f1}}~[[MW]]), then diverted export steam ({{EXPORT_STEAM_B_MW|f1}}~[[MW]]), syngas heat above 130[[degC]] ({{EB_B_REC_F_SYNGAS_RECOVERY|f1}}~[[MW]]) and reactor steam for the remaining {{REBOILER_GAP_B_MW|u}}. The Aronu extended UNIQUAC model [@aronu2011] is the named solvent basis in the project derivations; Appendix E states which results rest on it.

## 2.4 Reactor safety and emergency planning

Tristructural isotropic (TRISO) fuel particles incorporate multiple protective layers that retain fission products, while helium cooling and the graphite-based core support high-temperature operation [@r9]. The primary helium boundary, IHX and independent secondary loop provide successive physical barriers between the reactor and hydrogen-production equipment.

Singapore's compact geography makes site-specific emergency planning essential [@r13]. Advanced-reactor assessments use accident frequencies and projected radiological consequences to determine protective actions [@r14,r15,r16]. {fig:f_epz} illustrates this method using the Xe-100 safety-case example [@r17]. That assessment presents a 400~[[m]] emergency-planning reference distance for its specific case [@r17,r18], and HTR-PM studies evaluate offsite arrangements [@r19,r20]. Here 400~[[m]] is a conceptual reference only and is no approved Singapore boundary. Emergency arrangements would come from a reactor-specific safety case and regulatory review.

!!figure f_epz

## 2.5 Regulation and community engagement

Singapore is strengthening its capability to assess advanced nuclear technologies and has made no decision on nuclear energy [@r21,r22]. A Phase 1 International Atomic Energy Agency (IAEA) Integrated Nuclear Infrastructure Review (INIR) is tentatively set for 2027 and covers 19 areas of infrastructure, including emergency preparedness and radioactive waste management [@r21]. Manpower schemes for public officers begin in 2027 [@emanmdp], and separately NUS and NTU are working towards a joint Master's in nuclear engineering in 2027 [@metilecture]. The Singapore Nuclear Research and Safety Institute contributes reactor modelling and safety analysis alongside public education [@r22,r23,r24]. The National Environment Agency's responsibilities and the Energy Market Authority's feasibility study add to the institutional basis, supported by regulatory cooperation abroad [@r25,r26,r27]. A future project needs one licensing framework that spans the reactor and its industrial heat interface, with emergency preparedness built in.

Early engagement with Western Island planners, neighbouring Jurong Island operators, workers and residents would support informed assessment [@r23,r24,r28]. Clear communication of the plant's industrial purpose and protective design features is central to project development.

## 2.6 Risks and limitations

The assessment adopts INL process conditions [@r8,r10], a {{R_RATED_MWTH|f0|u}} rating and reduced-order balances (Appendix E). IHX performance and the transfer of INL's recycle to MEA need integrated verification, as does the {{ETA_POWER_PCT|f1}}% power-conversion assumption. {tab:t_risk} summarises the principal constraints.

!!table t_risk

## 2.7 Location and roadmap

The proposed Western Island is the team's proposal and has no official approval. The government has announced that smaller islands around Jurong Island will be connected into a new western island, reclaimed over coming decades, and its page names new power generation infrastructure with no mention of nuclear [@govndr,britcham]. The reference case is a new build at SOXAL scale, so it retires no existing source. Central hydrogen runs only in the {{AVAILABILITY_PCT|f0}}% availability hours, and backup firing stays a sensitivity. Captured CO₂ is shipped to offshore storage abroad under Singapore-Indonesia cooperation [@r30]. Pipeline links to the island are unpriced.

{fig:f_road} sets out an indicative roadmap to potential operation by 2050, and every date is illustrative. National capability and overseas reactor and IHX qualification come before Singapore-specific site assessment, licensing, investment and construction, and each gate opens only once its requirements are met. Land may not exist at the 2039 milestone, since reclamation has no published date. Stakeholders also include the Economic Development Board and CO₂ storage partners. The roughly 11 years between HTR-PM construction start and commercial operation give one historical reference for the schedule [@r12].

!!figure f_road
