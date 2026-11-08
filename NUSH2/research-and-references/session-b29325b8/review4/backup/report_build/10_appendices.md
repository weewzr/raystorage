!!appendix A

# Appendix A. Reactor design basis

This appendix records the source-traceable design basis and the reactor sizing used in Chapters 1 and 2. Reference values are from INL TEV-961, Table 2, 925[[degC]] case [@r8], with the process configuration of TEV-953 [@r10]. Sizing rests on reduced-order balances and does not demonstrate self-sufficiency of a built plant.

!!table t_a1

!!table t_walk

Note 1. INL quotes an equivalent reactor size of 178.8~[[MWth]] for Case 6 because it imports 16.5~[[MWe]] [@r8], so that figure carries almost no site electricity.

Note 2. Losses and the reboiler are now counted in the walk, and Case 5 duties stand in for Case 6, whose stream sheets are not held.

Note 3. The {{RES_E_PROCESS_MWE_AINL|f1}}~[[MWe]] demand belongs to the INL Selexol and PSA flowsheet, including CO₂ compression and purge recovery [@r8]. The purge row is Case 6 fuel-gas recycle and nitrogen removal equipment, split not published.

Note 4. INL's rate of 29,000 lb/h over {{HOURS_Y|u}} gives {{H2_A_T_Y|u}} of product including 0.79 mass% methane. At Case B, {{H2_B_NM3H|u}} gives {{H2_B_T_Y|u}}.

!!appendix B

# Appendix B. Mass and energy balance

Stream values come from the integrated balance files. The PSA streams are external boundary values, and the WGS heat release and the hydrogen and total mass closures were not run (Appendix E).

!!table t_b_carbon

!!table t_b_steam

Helium circulation at the need is {{HE_PRIM_B_KG_S|u}} primary and {{HE_SEC_B_KG_S|u}} secondary, and the circulators draw {{EB_B_REC_F_HE_CIRCULATION|t|u:MWe}}. The sizing range of {{R_BEST_B_MWTH|u}} to {{R_HIGH_B_MWTH|u}} spans the best and adverse corners of SRD, losses and circulator terms. SRD alone moves the need to {{SENS_SRD_3P4_R_NEEDED|u}} and {{SENS_SRD_4P2_R_NEEDED|u}}.

!!appendix C

# Appendix C. Emissions

The direct method is the headline. Baseline {{BASELINE_B_T_Y|u:tCO2e/y}} less residual {{ATM_B_T_Y|u:tCO2e/y}} gives net abatement of {{A_ATM_B_T|u:tCO2e/y}}. Lifecycle factors for nuclear and upstream gas are taken through an earlier project table and were not re-verified [@unece,ieaupstream]. Ship fuel uses an assumed {{TS_BURDEN|f3}}~[[t/t]] of CO₂ stored.

!!table t_c_steps

!!appendix D

# Appendix D. Economics

Table entries come from the ledger and DCF results. INL's overnight cost already holds owner costs, licensing and decommissioning shares [@inlcost], so the extra licensing and decommissioning lines are labelled proxies that may overlap them.

!!table t_d_inputs

!!table t_d_reactor

!!table t_d_matrix

!!table t_d_fx

!!table t_comp

!!figure f_ledger25

!!figure f_ledger60

!!appendix E

# Appendix E. Model status

This appendix prints only what ran. The results hold {{STATUS_ROWS_ANCHORED_PORT|f0}} rows marked as an anchored port, {{STATUS_ROWS_INPUT|f0}} input rows and {{STATUS_ROWS_NOT_RUN|f0}} rows not run. An anchored port is an algebraic balance anchored on INL cases and reproduced in the Rust engine. PSA streams are external boundary values, and the {{ETA_POWER_PCT|f1}}% conversion, availability and capture fraction are fixed assumptions.

The validation file records {{VAL_N_TOTAL|f0}} checks: {{VAL_N_PASS|f0}} pass, {{VAL_N_FAIL|f0}} fail and {{VAL_N_NOT_RUN|f0}} not run. {{VAL_N_SAME_DATASET|f0}} of them use the same dataset as the model and {{VAL_N_INDEPENDENT|f0}} use independent data, so a pass shows that the engine reproduces the screening balances and proves no physics.

!!table t_e_val

Not run in this model: reformer outlet at 825 and 800[[degC]]; rate-based absorber and stripper columns; WGS heat release; hydrogen and total mass closure rows. These stay open, and every result in this report rests on the stated assumptions in their place.
