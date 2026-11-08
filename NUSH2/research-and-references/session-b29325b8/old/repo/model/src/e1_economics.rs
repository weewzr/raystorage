//! E1 engineering-economic replacement model.
//! Historical final_design economics remain untouched for audit provenance.
use crate::deployment::{ccs_capex_sgd, crf, JAEA_AVAIL, USD_SGD_2026_09_29};
use crate::final_design::{
    final_design, final_cost_ledger, CAPTURED_SHORT_T_D, GAS_PRICE_SGD_GJ, HEAT_MWTH, NG_BASE_MMSCFD,
    NG_FINAL_MMSCFD, NG_HHV_BTU_SCF, T_AND_S_SGD_T,
};

pub const TARGET_COST_YEAR: i32 = 2025;
pub const IAEA2075_ASSESSMENT_YEAR: i32 = 2021;
pub const US_GDP_DEFLATOR_2021: f64 = 110.159;
pub const US_GDP_DEFLATOR_2025: f64 = 128.893;
pub const IAEA_HTGR200_CAPEX_MUSD: f64 = 2065.0;
pub const IAEA_HTGR200_OM_MUSD_Y: f64 = 192.0;
pub const IAEA_HTGR200_TOTAL_MWTH: f64 = 800.0;
pub const IAEA_HTGR200_MODULE_MWTH: f64 = 200.0;
pub const IAEA_MHRT_CAPEX_MUSD: f64 = 2748.0;
pub const IAEA_MHRT_OM_MUSD_Y: f64 = 324.0;
pub const IAEA_MHRT_TOTAL_MWTH: f64 = 2400.0;
pub const IAEA_MHRT_MODULE_MWTH: f64 = 600.0;
pub const IAEA_SOURCE_MODULES: f64 = 4.0;
// TECDOC-1682 publication-year proxy is used only for the legacy integration
// sensitivity because the underlying estimate's exact price year is not preserved.
pub const IAEA1682_PUBLICATION_YEAR_PROXY: i32 = 2012;
pub const US_GDP_DEFLATOR_2012_PROXY: f64 = 93.1835;
pub const IAEA1682_IHX_LOOP_MUSD: f64 = 69.0;
pub const IAEA1682_IHX_LOOP_MWTH: f64 = 170.0;

fn short_t_to_t(x: f64) -> f64 { x * 0.90718474 }
fn ng_gj_day(mmscfd: f64) -> f64 {
    mmscfd * 1e6 * NG_HHV_BTU_SCF * 1.05505585262e-6
}
pub fn usd2021_to_sgd2025(x_usd: f64) -> f64 {
    x_usd * (US_GDP_DEFLATOR_2025 / US_GDP_DEFLATOR_2021) * USD_SGD_2026_09_29
}
pub fn usd2012proxy_to_sgd2025(x_usd: f64) -> f64 {
    x_usd * (US_GDP_DEFLATOR_2025 / US_GDP_DEFLATOR_2012_PROXY) * USD_SGD_2026_09_29
}
pub fn source_annual_npp_sgd(capex_musd: f64, om_musd_y: f64) -> f64 {
    usd2021_to_sgd2025((capex_musd * crf(0.08, 25) + om_musd_y) * 1e6)
}
pub fn mhrt_one_module_annual_sgd() -> f64 {
    source_annual_npp_sgd(IAEA_MHRT_CAPEX_MUSD, IAEA_MHRT_OM_MUSD_Y) / IAEA_SOURCE_MODULES
}
pub fn htgr200_three_module_annual_sgd() -> f64 {
    source_annual_npp_sgd(IAEA_HTGR200_CAPEX_MUSD, IAEA_HTGR200_OM_MUSD_Y)
        * (600.0 / IAEA_HTGR200_TOTAL_MWTH)
}
pub fn mhrt_shared_heat_allocation_sgd() -> f64 {
    source_annual_npp_sgd(IAEA_MHRT_CAPEX_MUSD, IAEA_MHRT_OM_MUSD_Y)
        * (HEAT_MWTH / IAEA_MHRT_TOTAL_MWTH)
}
pub fn htgr200_shared_heat_allocation_sgd() -> f64 {
    source_annual_npp_sgd(IAEA_HTGR200_CAPEX_MUSD, IAEA_HTGR200_OM_MUSD_Y)
        * (HEAT_MWTH / IAEA_HTGR200_TOTAL_MWTH)
}
pub fn ihx_loop_annual_sgd_unscaled() -> f64 {
    usd2012proxy_to_sgd2025(IAEA1682_IHX_LOOP_MUSD * 1e6) * crf(0.08, 25)
}
pub fn ihx_loop_annual_sgd_linear_duty_sensitivity() -> f64 {
    ihx_loop_annual_sgd_unscaled() * HEAT_MWTH / IAEA1682_IHX_LOOP_MWTH
}

#[derive(Clone, Copy, Debug)]
pub enum NuclearCase {
    MhrtOneModuleCentral,
    Htgr200ThreeModule,
    MhrtSharedConditional,
    Htgr200SharedConditional,
}
#[derive(Clone, Copy, Debug)]
pub struct E1Case {
    pub baseline_sgd_y: f64,
    pub candidate_sgd_y: f64,
    pub net_incremental_sgd_y: f64,
    pub nuclear_sgd_y: f64,
    pub integration_sgd_y: f64,
    pub ccs_sgd_y: f64,
    pub ts_sgd_y: f64,
    pub avoided_t_y: f64,
    pub abatement_sgd_t: f64,
    pub unresolved_margin_to_100_sgd_y: f64,
    pub pass_cost: bool,
}
pub fn e1_case(case: NuclearCase, ng_price: f64, ts_price: f64) -> E1Case {
    let baseline = ng_gj_day(NG_BASE_MMSCFD) * 365.0 * JAEA_AVAIL * ng_price;
    let candidate_ng = ng_gj_day(NG_FINAL_MMSCFD) * 365.0 * JAEA_AVAIL * ng_price;
    let nuclear = match case {
        NuclearCase::MhrtOneModuleCentral => mhrt_one_module_annual_sgd(),
        NuclearCase::Htgr200ThreeModule => htgr200_three_module_annual_sgd(),
        NuclearCase::MhrtSharedConditional => mhrt_shared_heat_allocation_sgd(),
        NuclearCase::Htgr200SharedConditional => htgr200_shared_heat_allocation_sgd(),
    };
    // Modern IAEA NPP source scope may already include heat-interface equipment.
    // Therefore no separate TECDOC-1682 IHX cost is stacked here.
    let integration = 0.0;
    let captured = short_t_to_t(CAPTURED_SHORT_T_D) * 365.0 * JAEA_AVAIL;
    let ccs = ccs_capex_sgd(captured) * crf(0.08, 25);
    let ts = captured * ts_price;
    let candidate = candidate_ng + nuclear + integration + ccs + ts;
    let net = candidate - baseline;
    let avoided = final_design(0.0, false).lifecycle_avoided_t;
    let cost = net / avoided;
    let margin = 100.0 * avoided - net;
    E1Case {
        baseline_sgd_y: baseline,
        candidate_sgd_y: candidate,
        net_incremental_sgd_y: net,
        nuclear_sgd_y: nuclear,
        integration_sgd_y: integration,
        ccs_sgd_y: ccs,
        ts_sgd_y: ts,
        avoided_t_y: avoided,
        abatement_sgd_t: cost,
        unresolved_margin_to_100_sgd_y: margin,
        pass_cost: cost < 100.0,
    }
}
pub fn legacy_nishihara_with_source_integration() -> E1Case {
    let old = final_cost_ledger();
    let avoided = final_design(0.0, false).lifecycle_avoided_t;
    let net = old.net_incremental - old.integration_added + ihx_loop_annual_sgd_unscaled();
    let candidate = old.baseline_ng + net;
    let cost = net / avoided;
    E1Case {
        baseline_sgd_y: old.baseline_ng,
        candidate_sgd_y: candidate,
        net_incremental_sgd_y: net,
        nuclear_sgd_y: old.reactor_added,
        integration_sgd_y: ihx_loop_annual_sgd_unscaled(),
        ccs_sgd_y: old.ccs_annual_added,
        ts_sgd_y: old.transport_storage_added,
        avoided_t_y: avoided,
        abatement_sgd_t: cost,
        unresolved_margin_to_100_sgd_y: 100.0 * avoided - net,
        pass_cost: cost < 100.0,
    }
}
pub fn break_even_ng_price(case: NuclearCase, ts_price: f64) -> f64 {
    let at0 = e1_case(case, 0.0, ts_price);
    let saving_per_sgd_gj = (ng_gj_day(NG_BASE_MMSCFD) - ng_gj_day(NG_FINAL_MMSCFD))
        * 365.0 * JAEA_AVAIL;
    (at0.net_incremental_sgd_y - 100.0 * at0.avoided_t_y) / saving_per_sgd_gj
}
pub fn break_even_ts_price(case: NuclearCase, ng_price: f64) -> f64 {
    let at0 = e1_case(case, ng_price, 0.0);
    let captured = short_t_to_t(CAPTURED_SHORT_T_D) * 365.0 * JAEA_AVAIL;
    (100.0 * at0.avoided_t_y - at0.net_incremental_sgd_y) / captured
}
pub fn cases_csv() -> String {
    let mut s = String::from("case,baseline_sgd_y,candidate_sgd_y,net_incremental_sgd_y,nuclear_sgd_y,integration_sgd_y,ccs_sgd_y,ts_sgd_y,avoided_t_y,abatement_sgd_t,margin_to_100_sgd_y,pass_cost\n");
    for (name, case) in [
        ("MHR-T one 600 MWth module - central", NuclearCase::MhrtOneModuleCentral),
        ("HTGR-200 three modules - alternative", NuclearCase::Htgr200ThreeModule),
        ("MHR-T thermal-share - conditional", NuclearCase::MhrtSharedConditional),
        ("HTGR-200 thermal-share - conditional", NuclearCase::Htgr200SharedConditional),
    ] {
        let x = e1_case(case, GAS_PRICE_SGD_GJ, T_AND_S_SGD_T);
        s.push_str(&format!("{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.6},{:.3},{}\n",
            name,x.baseline_sgd_y,x.candidate_sgd_y,x.net_incremental_sgd_y,x.nuclear_sgd_y,
            x.integration_sgd_y,x.ccs_sgd_y,x.ts_sgd_y,x.avoided_t_y,x.abatement_sgd_t,
            x.unresolved_margin_to_100_sgd_y,x.pass_cost));
    }
    let x=legacy_nishihara_with_source_integration();
    s.push_str(&format!("Legacy Nishihara + IAEA1682 integration replacement,{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.6},{:.3},{}\n",
        x.baseline_sgd_y,x.candidate_sgd_y,x.net_incremental_sgd_y,x.nuclear_sgd_y,
        x.integration_sgd_y,x.ccs_sgd_y,x.ts_sgd_y,x.avoided_t_y,x.abatement_sgd_t,
        x.unresolved_margin_to_100_sgd_y,x.pass_cost));
    s
}
pub fn provenance_csv() -> String {
    String::from("id,value,unit,original_year,currency,target_year,normalization,provenance,evidence_class,double_count_treatment\nIAEA2075_MHRT_CAPEX,2748,million USD,2021,USD,2025,US BEA GDP deflator 128.893/110.159 then 1.2776 SGD/USD,IAEA TECDOC 2075 Table 73,SOURCE CASE,replaces historical favourable nuclear bridge in E1 central\nIAEA2075_MHRT_OM,324,million USD/y,2021,USD,2025,US BEA GDP deflator 128.893/110.159 then 1.2776 SGD/USD,IAEA TECDOC 2075 Table 73,SOURCE CASE,included once in NPP annual burden\nIAEA2075_HTGR200_CAPEX,2065,million USD,2021,USD,2025,US BEA GDP deflator 128.893/110.159 then 1.2776 SGD/USD,IAEA TECDOC 2075 Table 73,SOURCE CASE,alternative nuclear case\nIAEA2075_HTGR200_OM,192,million USD/y,2021,USD,2025,US BEA GDP deflator 128.893/110.159 then 1.2776 SGD/USD,IAEA TECDOC 2075 Table 73,SOURCE CASE,included once in NPP annual burden\nIAEA1682_IHX_LOOP,69,million USD,2012 publication-year proxy,USD,2025,US BEA GDP deflator proxy then 1.2776 SGD/USD,IAEA TECDOC 1682 Table 3.17,PRELIMINARY SOURCE ANCHOR,tested separately; not stacked on modern NPP cases because scope overlap unresolved\nNG_PRICE,15,SGD/GJ,screening,SGD,2025,no conversion,project assumption A-08,SCREENING ASSUMPTION,baseline and candidate consistently\nTS_PRICE,15,SGD/tCO2,screening,SGD,2025,no conversion,project assumption A-16,SCREENING ASSUMPTION,separate from capture CAPEX\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn usd_2021_normalization_is_reproducible() {
        assert!((usd2021_to_sgd2025(1.0) - 1.4948).abs() < 0.002);
    }
    #[test] fn central_is_one_600mw_module_not_four() {
        let all = source_annual_npp_sgd(IAEA_MHRT_CAPEX_MUSD, IAEA_MHRT_OM_MUSD_Y);
        assert!((mhrt_one_module_annual_sgd() * 4.0 - all).abs() < 1e-6);
    }
    #[test] fn central_case_fails_cost_threshold() {
        let x = e1_case(NuclearCase::MhrtOneModuleCentral, 15.0, 15.0);
        assert!(x.abatement_sgd_t > 100.0);
        assert!(!x.pass_cost);
        assert!(x.unresolved_margin_to_100_sgd_y < 0.0);
    }
    #[test] fn shared_case_is_explicitly_conditional_and_below_threshold_before_omissions() {
        let x = e1_case(NuclearCase::MhrtSharedConditional, 15.0, 15.0);
        assert!(x.abatement_sgd_t < 100.0);
    }
    #[test] fn dedicated_htgr200_alternative_is_adverse() {
        assert!(e1_case(NuclearCase::Htgr200ThreeModule,15.0,15.0).abatement_sgd_t > 100.0);
    }
    #[test] fn integration_anchor_replaces_old_allowance_when_used() {
        assert!(ihx_loop_annual_sgd_unscaled() > 10_000_000.0);
        assert!(ihx_loop_annual_sgd_linear_duty_sensitivity() > ihx_loop_annual_sgd_unscaled());
    }
    #[test] fn source_integration_replaces_old_allowance_not_stacks() {
        let old=final_cost_ledger();
        let x=legacy_nishihara_with_source_integration();
        let expected=old.net_incremental-old.integration_added+ihx_loop_annual_sgd_unscaled();
        assert!((x.net_incremental_sgd_y-expected).abs()<1e-6);
        assert!(x.abatement_sgd_t>3.725);
    }
    #[test] fn break_even_sensitivities_are_finite() {
        assert!(break_even_ng_price(NuclearCase::MhrtOneModuleCentral,15.0).is_finite());
        assert!(break_even_ts_price(NuclearCase::MhrtOneModuleCentral,15.0).is_finite());
    }
    #[test] fn emissions_denominator_is_unchanged() {
        let x=e1_case(NuclearCase::MhrtOneModuleCentral,15.0,15.0);
        assert!((x.avoided_t_y-final_design(0.0,false).lifecycle_avoided_t).abs()<1e-8);
    }
}
