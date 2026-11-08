# 4. Economics and Implementation

## 4.1 Method and boundary

Two methods feed one result, both in real 2025 Singapore dollars at {{DISCOUNT_PCT|f0}}%, with {{FX|f4|u:S$/US$}} [@fredfx] and escalation factors of {{F_2019USD_TO_2025SGD|f4}} from 2019 US dollars and {{F_2014EUR_TO_2025SGD|f3}} from 2014 euros [@fredgdp,fredeur]. The first is an annual incremental ledger against a same-size new-build unabated SMR at {{H2_B_T_Y|u:tH2/y}}, with levels L0 to L5 adding items step by step:

$$\Delta C = \sum_{i} {C_{i}}, \quad \mathrm{AC} = \frac{\Delta C}{A}, \quad \mathrm{CRF} = \frac{r}{1 - \left(1 + r\right)^{-L}}$$ {eq:ledger}

Here $C_{i}$ is an annual cost line, $A$ is net abatement and CRF annualises capital over life $L$ at rate $r$, giving {{CRF_25Y}} at 25~[[y]] and {{CRF_60Y}} at 60~[[y]]. Common items cancel, including reformer capital differences by assumption. The target sets a ceiling of {{CEILING_SM_Y|u}}.

The second is a year-by-year discounted cash flow (DCF) of the unabated SMR and NUS-H₂ at equal output on a common horizon:

$$\mathrm{LCOH}_{x} = \frac{\sum_{t} {c_{x,t} \left(1 + r\right)^{-t}}}{\sum_{t} {m_{t} \left(1 + r\right)^{-t}}}, \quad \mathrm{LAC} = \frac{\sum_{t} {\Delta c_{t} \left(1 + r\right)^{-t}}}{\sum_{t} {A_{t} \left(1 + r\right)^{-t}}}$$ {eq:dcf}

Reactor spend runs uniformly over {{CONSTRUCTION_YEARS|f1}}~[[y]] [@nzw09], then operation runs at {{AVAILABILITY_PCT|f0}}%. Items with a 25~[[y]] life are replaced on longer horizons with unused life credited at the end, and decommissioning is paid from cash flow at end of life. Both plants carry the SMR island of the IEAGHG Base Case, {{SMR_ISLAND_CAPITAL_SM|u}} of capital and {{SMR_ISLAND_FOM_SM_Y|u}} of fixed O&M [@r5]. The two methods share inputs and differ in timing, so their agreement tests the annualisation and does not validate the inputs.

## 4.2 Reactor capital

INL's recipe removes the 9% conversion share and converts at 40% efficiency [@inlcost]:

$$c_{\mathrm{BOAK}} = 6{,}000 \times 0.91 \times 0.40 = {{OCC_CENTRAL|f0}}\ \mathrm{US\$_{2019}/kW_{t}}, \quad c_{\mathrm{FOAK}} = {{FOAK_CENTRAL|f1}}\ c_{\mathrm{BOAK}}$$ {eq:occ}

BOAK is INL's term for a build between a first and Nth of a kind. Central capital uses $n = 1.0$, which charges INL's unit cost at any size: {{REACTOR_CAPITAL_BOAK_N1P00_SM|f0|u}} at BOAK and {{REACTOR_CAPITAL_FOAK_N1P00_SM|f0|u}} at FOAK. Two size treatments print beside it at equal prominence, with BOAK capital of {{REACTOR_CAPITAL_BOAK_N0P60_SM|f0|u}} and {{REACTOR_CAPITAL_BOAK_N0P31_SM|f0|u}}. Here $n = 0.6$ is a project sensitivity, and its {{REACTOR_COST_REF_SIZE|f0}}~[[MWth]] reference size is a project assumption. The $n = 0.31$ value is the ratio of two single-unit FOAK estimates from INL, so it carries that label [@inlcost]. Learning at {{LEARNING_PCT|f0}}% per doubling reaches a tenth unit by two published routes: {{OCC_10OAK_P47_USD_KWT|u}} from FOAK and {{OCC_10OAK_P46_USD_KWT|u}} from BOAK [@inlcost].

## 4.3 Capture, storage, integration and operating costs

Capture capital is {{CAPTURE_K_1A_SM_TH|u}} from IEAGHG Case 1A [@r5], with fixed O&M at an assumed {{CAPTURE_FOM_PCT|f1}}% of capital. Case 1A is an aMDEA case, and {tab:t_unc} carries the MEA flue-gas alternative. Capture energy sits in the heat and power balance, so no energy line appears and nothing is counted twice. MEA make-up and ship liquefaction have no cost line. CO₂ transport and storage (T&S) is {{TS_CENTRAL_S_T|f0|u}}, with {{TS_LOW|f1}} low [@r5] and {{TS_HIGH|f0}} high [@eria2024]; no published Singapore-Indonesia tariff was found [@metipq,navigant]. Gas is {{GAS_PRICE_S_GJ|f0|u}} [@fredgas], and the plant saves {{NG_SAVED_B_MMSCFD|u}} against the unabated SMR. Reactor O&M and fuel is {{OPEX_CENTRAL|f0|u:US$/MWth-h}} [@inlcost]. IHX and loop, licensing and decommissioning are labelled proxies, and the first two rest on primary sources not verified here. Reactor life changes only the reactor, licensing and decommissioning lines.

## 4.4 Headline result and cost ladder

{tab:t_head} prints the headline. Every real plant cost is in, the reactor is charged at {{R_RATED_MWTH|f0|u}} and $n = 1.0$. The full cost by ledger is {{AC_L5_BOAK_25Y|u}} and {{AC_L5_BOAK_60Y|u}} at BOAK, and {{AC_L5_FOAK_25Y|u}} and {{AC_L5_FOAK_60Y|u}} at FOAK. The DCF lands within {{DCF_TEST_MAX_GAP_PCT}}% on every cell of the grid, inside the 10% test. None of the {{TARGET_COST_N_CENTRAL}} central cases reaches {{TARGET_COST_S_T|f0|u}}. Hydrogen costs {{LCOH_PREMIUM_BOAK_25Y|u}} more than from the unabated SMR at BOAK, 25~[[y]], which is the premium a buyer would have to carry.

!!table t_head

{tab:t_ladder} shows what each item adds. The gas saved offsets most of the reactor capital line, and T&S and reactor O&M then take the cost past the target. L3 is a named step, {{AC_L3_BOAK_25Y|u}} and {{AC_L3_BOAK_60Y|u}}, and it omits interest during construction, the power island, the recycle train and lost steam, all real costs of this plant.

!!table t_ladder

## 4.5 Uncertainty

{tab:t_grid} holds the three size treatments at equal weight, and {fig:f_grid} draws them. {tab:t_unc} varies one input around BOAK, 25~[[y]], $n = 1.0$; corner runs carry no probability. The transfer of recycle and the reactor cost treatment move the result most, and availability comes next. A discount rate of 5% or 10% gives {{SENS_DISCOUNT_0P05_AC_L5_25|u}} or {{SENS_DISCOUNT_0P1_AC_L5_25|u}}, a swing close to that of the gas saving, where INL's {{GAS_SAVING_INL_PCT}}% and Ahn and Lee's {{GAS_SAVING_AHN_PCT}}% remain unreconciled [@r8,ahn2026]. SRD moves the spare at a fixed {{R_RATED_MWTH|f0|u}} and leaves the cost almost unchanged. Charging only the {{R_NEEDED_B_MWTH|u}} the process needs gives {{SENS_CHARGED_178P494_AC_L5_25|u}} and {{SENS_CHARGED_178P494_AC_L5_60|u}} and removes the design margin.

!!table t_grid

!!table t_unc

## 4.6 SMR plus CCS comparator

One labelled comparator is IEAGHG Case 3, an SMR with MEA on flue gas at {{COMPARATOR_CAPTURE_PCT}}% overall capture, scaled to {{H2_B_NM3H|u}} [@r5]. At {{AVAILABILITY_PCT|f0}}% availability it abates {{COMPARATOR_A_MT|u:MtCO2e/y}} against its own {{BASELINE_IEAGHG_KG_KG}} baseline. Its published cost converts to {{RES_COMPARATOR_CAC_TS23P2_S_T|u}} with T&S at {{COMPARATOR_TS_EUR|f0|u:EUR/t}}, or {{RES_COMPARATOR_CAC_TS60_S_T|u}} at this report's {{TS_CENTRAL_S_T|f0|u}}. At those T&S values NUS-H₂ at BOAK, 25~[[y]], gives {{SENS_TS_23P2_AC_L5_25|u}} and {{AC_L5_BOAK_25Y|u}}, while FOAK gives {{AC_L5_FOAK_25Y|u}} at central T&S. The row is indicative, since gas price and scale differ ({tab:t_comp}).

## 4.7 Closing the gap to the target

At BOAK, 25~[[y]], the full cost is {{COST_L5_BOAK_25Y|u}}, which is {{EXCESS_L5_BOAK_25Y_SM_Y|u}} over the ceiling. {tab:t_gap} tests the levers one at a time. No single lever closes this gap within its tested range: gas would need {{LEVER_GAS_L5_BOAK_25Y_S_GJ|u}}, reactor capital would need {{LEVER_OCC_L5_BOAK_25Y_USD_KWT|u}} against INL's low of {{OCC_LOW|f0|u:US$2019/kWt}}, and T&S would have to go negative.

!!table t_gap

The named scenario MATURE_60Y stacks the changes that a mature reactor fleet and dearer gas would bring ({fig:f_step}). A tenth-of-a-kind reactor on INL's BOAK learning route brings the base {{STEPDOWN_1_L5|u}} down to {{STEPDOWN_2_L5|u}} at 25~[[y]], and a 60~[[y]] life takes it to {{STEPDOWN_3_L5|u}}. Gas at {{STEPDOWN_4_GAS_S_GJ|u}} against the central {{STEPDOWN_1_GAS_S_GJ|f0|u}} then gives {{STEPDOWN_4_L5|u}} on the ledger and {{STEPDOWN_4_DCF|u}} by DCF, below the target. The printed price is the larger of two solves for {{TARGET_MATURE_COST|f0|u}}, {{MATURE_60Y_L5_GAS_NEEDED_S_GJ|u}} on the ledger and {{MATURE_60Y_DCF_GAS_NEEDED_S_GJ|u}} by DCF, rounded up to 0.1; at {{GAS_HIGH|f0|u}}, the top of the tested range, the scenario sits at {{MATURE_60Y_L5_AT_20|u}}. Dearer gas lowers the abatement cost because NUS-H₂ burns {{NG_B_MMSCFD|u}} against {{NG_UNABATED_B_MMSCFD|u}} for the unabated plant, so each rise in price enlarges the credit for the gas saved. No central case meets these conditions, and the base result stays the reference.

!!figure f_step

For deployment, the result favours waiting for a design replicated overseas before any Singapore commitment, and replacing proxies with reactor quotations and a contracted T&S tariff. {fig:f_road} places these steps in time. Carbon tax stays out of the headline because it is a transfer. As an operator view only, the {{CARBON_TAX|f0|u:S$/t}} of 2026 to 2027 [@nccstax] would leave {{TAX_NET_BOAK_25Y|u}}, and the {{CARBON_TAX_2030_LOW|f0|u:S$/t}} to {{CARBON_TAX_2030_HIGH|f0|u:S$/t}} band by 2030 remains a stated intention. The Future Energy Fund gives a policy context with no allocation assumed [@edb2025].
