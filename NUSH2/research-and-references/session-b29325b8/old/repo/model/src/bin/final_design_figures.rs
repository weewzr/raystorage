fn main(){
 use nuclear_assisted_smr::figures::{threshold_svg,availability_abatement_svg,ccs_robustness_svg};
 threshold_svg("../results/final_design/generated/cn4252_thresholds.svg").expect("threshold SVG generation failed");
 availability_abatement_svg("../results/final_design/generated/availability_abatement.svg").expect("availability abatement SVG generation failed");
 ccs_robustness_svg("../results/final_design/generated/ccs_robustness.svg").expect("CCS robustness SVG generation failed");
}