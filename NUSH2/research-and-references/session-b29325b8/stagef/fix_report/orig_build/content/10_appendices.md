!!appendix A

# Appendix A. Reactor design basis

This appendix records the source-traceable design basis and the reactor sizing used in Chapters 1 and 2. Reference values are from INL TEV-961, Table 2, 925[[degC]] case [@r8], with the process configuration of TEV-953 [@r10]. Sizing rests on balances anchored on these cases and does not demonstrate self-sufficiency of a built plant.

!!table t_a1

!!table t_walk

Note 1. INL quotes an equivalent reactor size of 178.8~[[MWth]] for Case 6 because it imports 16.5~[[MWe]] [@r8], so that figure carries almost no site electricity.

Note 2. Losses and the reboiler are counted in the walk, and Case 5 duties stand in for Case 6, whose stream sheets are not held. The walk runs on the anchored terms; its last row is the headline need, with the syngas heat above 130[[degC]] on its gated model.

Note 3. The {{RES_E_PROCESS_MWE_AINL|f1}}~[[MWe]] demand belongs to the INL Selexol and PSA flowsheet, including CO₂ compression and purge recovery [@r8]. The purge row is Case 6 fuel-gas recycle and nitrogen removal equipment, split not published.

Note 4. INL's rate of 29,000 lb/h over {{HOURS_Y|u}} gives {{H2_A_T_Y|u}} of product including 0.79 mass% methane. At Case B, {{H2_B_NM3H|u}} gives {{H2_B_T_Y|u}}.

!!appendix B

# Appendix B. Mass and energy balance

Stream values come from the integrated balance files, and the PSA streams are external boundary values. The cryogenic unit returns {{MB_B_REC_F_CRYO_H2_RETURN_H2_KMOL_H|u:kmol/h}} of hydrogen ahead of the PSA, a stream with no carbon. The carbon, hydrogen and mass residuals all print as zero, and the shift releases {{EB_B_REC_F_WGS_RELEASE|u:MW}}.

!!table t_b_carbon

!!table t_b_steam

Helium circulation at the need is {{HE_PRIM_B_KG_S|u}} primary and {{HE_SEC_B_KG_S|u}} secondary, and the circulators draw {{EB_B_REC_F_HE_CIRCULATION|t|u:MWe}}. The sizing range of {{R_BEST_B_MWTH|u}} to {{R_HIGH_B_MWTH|u}} spans the best and adverse corners of SRD, losses and circulator terms. SRD alone moves the need to {{SENS_SRD_3P4_R_NEEDED|u}} and {{SENS_SRD_4P2_R_NEEDED|u}}.

!!appendix C

# Appendix C. Emissions

The direct method is the headline. Baseline {{BASELINE_B_T_Y|u:tCO2e/y}} less residual {{ATM_B_T_Y|u:tCO2e/y}} gives net abatement of {{A_ATM_B_T|u:tCO2e/y}}. Lifecycle factors for nuclear power and upstream gas come from the cited sources and were not re-verified against their primary data [@unece,ieaupstream]. Ship fuel uses an assumed {{TS_BURDEN|f3}}~[[t/t]] of CO₂ stored.

!!table t_c_steps

!!appendix D

# Appendix D. Economics

Table entries come from the ledger and DCF results. INL's overnight cost already holds owner costs, licensing and decommissioning shares [@inlcost], so the extra licensing and decommissioning lines are labelled proxies that may overlap them.

!!table t_d_inputs

!!table t_d_reactor

!!table t_d_matrix

!!figure f_grid

!!table t_d_fx

!!table t_comp

!!figure f_ledger25

!!figure f_ledger60

!!appendix E

# Appendix E. Model status

This appendix prints only what ran. Each result row carries a status. On the headline preset, term T05, the syngas heat recovered above 130[[degC]], runs on its gated model, and every other term keeps its anchored value: an algebraic balance anchored on INL cases and reproduced in the Rust engine. Among the rows of the six results tables are {{STATUS_ROWS_ANCHORED_PORT|f0}} anchored rows, {{STATUS_ROWS_GATED_T05|f0}} rows that move with term T05, {{STATUS_ROWS_FLOWSHEET_M7|f0}} Level 3 flowsheet rows, {{STATUS_ROWS_LEVEL4_M8|f0}} heat-recovery rows, {{STATUS_ROWS_INPUT|f0}} input rows and {{STATUS_ROWS_NOT_RUN|f0}} rows not run. PSA streams are external boundary values, and the {{ETA_POWER_PCT|f1}}% conversion, availability and capture fraction are fixed assumptions.

The validation file records {{VAL_N_TOTAL|f0}} rows: {{VAL_N_PASS|f0}} pass, {{VAL_N_FAIL|f0}} fail, {{VAL_N_REPORTED|f0}} are reported with no limit and {{VAL_N_NOT_RUN|f0}} did not run. {{VAL_N_INDEPENDENT|f0}} rows use independent data, {{VAL_N_SAME_DATASET|f0}} share a dataset with the model and {{VAL_N_CALIBRATION|f0}} are calibration, so a pass on a shared dataset verifies code and validates no physics. {tab:t_e_val} groups the rows by level.

!!table t_e_val

{tab:t_e_gates} lists the thirteen model terms. A term leaves its anchored value only when its gate passes. The column gate failed on six benchmark rows, among them the high-pressure absorber pilot at {{VAL_SA-1_SIM|f1}}% removal against {{VAL_SA-1_LIT|f0}}% and the two SRD rows of Section 2.3, so the capture terms stay anchored. The Level 3 flowsheet failed its carbon gate: reformer tube carbon is {{VALPCT_FS_CARBON_VS_ANCHOR_TUBE}}% below the anchor and tail-gas carbon {{VALPCT_FS_CARBON_VS_ANCHOR_TAIL_GAS}}% below, so the recycle, compression and gas terms stay anchored. Two gates could not run: the upstream preheat behind the process heat term rests on Case 6 stream states the library lacks, and no measured helium-side pressure drop exists for the secondary circulator.

!!table t_e_gates

Two open heat terms move the reactor need. The higher secondary helium pressure drop in the sweep raises it to {{SENS_SEC_DP_414_R_NEEDED|u}}, and reading the {{ETA_POWER_PCT|f1}}% conversion as already net of primary circulation lowers it to {{SENS_ETA_NET_OF_PRIMARY_1_R_NEEDED|u}}. Circulator power has no measured basis, and {{AVAILABILITY_PCT|f0}}% availability has no measured HTGR record behind it.
