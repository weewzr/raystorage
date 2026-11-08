# 3. Net Emissions Abatement

## 3.1 Method

Net abatement is the difference between the operational emissions of a same-size unabated SMR and the carbon the proposed plant sends to air:

$$\Delta E_{\mathrm{net}} = E_{\mathrm{baseline}} - E_{\mathrm{residual}}$$ {eq:net}

$$E_{\mathrm{residual}} = E_{\mathrm{cut}} + E_{\mathrm{grid}}$$ {eq:res}

The furnace is removed, yet the nitrogen-rich cut of recycled PSA reject is burned, and that carbon $E_{\mathrm{cut}}$ is the residual. Absorber slip returns through the reformer and reaches air only through this cut. The grid term is zero as a calculated result at central inputs: unmet electricity is {{EB_B_REC_F_UNMET_ELECTRICAL|f0}}~[[MWe]] at the rated reactor. All values are in tonnes per year at {{AVAILABILITY_PCT|f0}}% availability.

## 3.2 System boundary and assumptions

The boundary follows {fig:f_pfd}. It holds a nuclear island (reactor, primary and secondary helium circuits, small power block) and a chemical plant (reformer, WGS, MEA capture, recycle compressor, cryogenic unit, CO₂ compressor and ship offshore storage). The PSA sits outside the line, with its reject returning across it. Ship and storage sit inside the line for mass balance and cost, while ship fuel stays in the lifecycle sensitivity. The CO₂ compressor load is INL's pipeline basis scaled to Case B, and the liquefaction a ship needs is neither modelled nor priced.

The baseline uses SOXAL's scale of about {{H2_B_NM3H|u}} [@airliquide2008]. At {{RHO_H2_NORMAL|f5}}~[[kg/Nm3]] and {{HOURS_Y|u}}, output is {{H2_B_T_Y|u:tH2/y}}. Baseline intensity is INL's {{BASELINE_INL_KG_KG|u}} [@r10], with {{BASELINE_CHO_KG_KG|f2}} from Cho et al. [@cho2022] and {{BASELINE_IEAGHG_KG_KG}}, the unrounded IEAGHG Base Case value [@r5], as bounds. Per-pass capture of {{PER_PASS_CAPTURE_PCT|f0}}%, a project assumption, gives {{PLANT_CAPTURE_B_PCT}}% plant capture because slip returns to the reformer, so the two always print together.

## 3.3 Net emissions abated

{tab:t_carbon} gives the carbon ledger. Abatement is {{A_ATM_B_T|u:tCO2e/y}} ({{A_ATM_B_MT|u:MtCO2e/y}}), a {{REDUCTION_B_PCT}}% reduction that takes direct intensity from {{BASELINE_INL_KG_KG|f2}} to {{INTENSITY_B_KG_KG|u}} and stands at {{ABATEMENT_MULTIPLE}} times the target. As a size comparison only, it equals {{NDC_SHARE_LOW_PCT}}% to {{NDC_SHARE_HIGH_PCT}}% of Singapore's 2035 target level of {{NDC_2035_LOW_MT}} to {{NDC_2035_HIGH_MT}}~[[MtCO2e]] [@nccsndc].

!!table t_carbon

## 3.4 Sensitivity and interpretation

{tab:t_absens} tests the result. Every case with working capture and recycle stays above the target. Two dependencies matter most. With capture offline, the only abatement left is carbon never generated, {{NEVER_GENERATED_MT|u:MtCO2e/y}}, which is below the target. If recycle fails to transfer from INL's Selexol case to MEA, gas rises to {{SENS_TAIL_GAS_2_NG_MMSCFD|u}} and abatement falls to {{A_NORECYCLE_MT|u:MtCO2e/y}}. A cooler reformer outlet leaves more methane unconverted, and more carbon then leaves in the burned nitrogen-rich cut.

!!table t_absens

Backup firing over the {{OUTAGE_H_Y|u}} of outage would add {{SENS_BACKUP_FIRING_1_H2_EXTRA_T_Y|f0|u:tH2/y}}. An unabated backup train emits {{SENS_BACKUP_FIRING_1_EMIT_BACKUP_T_Y|f0|u:tCO2e/y}}, which matches the unabated SMR in those hours and adds no abatement, while a backup train with capture would add {{SENS_BACKUP_FIRING_2_A_EXTRA_T_Y|f0|u:tCO2e/y}}; neither train's capital is priced. Abatement is counted at the Singapore source, storage abroad follows the Singapore-Indonesia memorandum [@r30], and how offshore storage counts across the border remains unresolved.
