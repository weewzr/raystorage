#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p ../results/generated
cargo run --quiet --bin gate5_threshold_scatter_csv > ../results/generated/gate5_threshold_scatter.csv
cargo run --quiet --bin gate5_threshold_scatter_finite_csv > ../results/generated/gate5_threshold_scatter_finite.csv
cargo run --quiet --bin gate5_binding_counts_csv > ../results/generated/gate5_binding_counts.csv
cargo run --quiet --bin gate5_driver_csv > ../results/generated/gate5_driver_effects.csv
cargo run --quiet --bin gate5_driver_cost_finite_csv > ../results/generated/gate5_driver_cost_finite.csv
cargo run --quiet --bin gate5_case1a_comparator_csv > ../results/generated/gate5_case1a_comparator.csv
cargo run --quiet --bin gate5_case1a_comparator_table_csv > ../results/generated/gate5_case1a_comparator_table.csv
cargo run --quiet --bin gate6_singapore_scale_csv > ../results/generated/gate6_singapore_scale.csv
cargo run --quiet --bin gate6_lifecycle_decomposition_csv > ../results/generated/gate6_lifecycle_decomposition.csv
cargo run --quiet --bin gate6_domain_table_csv > ../results/generated/gate6_domain_table.csv
cargo run --quiet --bin gate6_threshold_magnitude_csv > ../results/generated/gate6_threshold_magnitude.csv
cargo run --quiet --bin gate6_threshold_plot_bounds_csv > ../results/generated/gate6_threshold_plot_bounds.csv
cargo run --quiet --bin gate5_results_synthesis > ../results/generated/gate5_results_synthesis.md
cargo run --quiet --bin gate5_figure_manifest > ../results/generated/gate5_figure_manifest.md


cargo run --quiet --bin review5_results_csv > ../results/generated/review5_results.csv
cargo run --quiet --bin review5_margin_csv > ../results/generated/review5_margin.csv
cargo run --quiet --bin review5_cost_ledger_csv > ../results/generated/review5_cost_ledger.csv
cargo run --quiet --bin review5_cogeneration_csv > ../results/generated/review5_cogeneration.csv

mkdir -p ../results/final_design/generated
cargo run --quiet --bin final_design_results_csv > ../results/final_design/generated/final_design_results.csv
cargo run --quiet --bin final_design_source_balance_csv > ../results/final_design/generated/source_balance.csv
cargo run --quiet --bin final_design_temperature_csv > ../results/final_design/generated/temperature_sensitivity.csv

cargo run --quiet --bin final_design_visual_data_csv > ../results/final_design/generated/visual_data.csv
cargo run --quiet --bin final_design_heat_figure_tex > ../results/final_design/generated/heat_flow_figure.tex
cargo run --quiet --bin final_process_schematics
cargo run --quiet --bin final_design_lifecycle_ledger_csv > ../results/final_design/generated/lifecycle_ledger.csv
cargo run --quiet --bin final_design_cost_ledger_csv > ../results/final_design/generated/cost_ledger.csv
cargo run --quiet --bin final_design_figures

cargo run --quiet --bin final_design_availability_csv > ../results/final_design/generated/availability_sensitivity.csv

cargo run --quiet --bin final_design_gas_backup_csv > ../results/final_design/generated/gas_backup_sensitivity.csv

cargo run --quiet --bin final_design_reliability_summary > ../results/final_design/generated/reliability_sensitivity.md

cargo run --quiet --bin final_design_ccs_capture_csv > ../results/final_design/generated/ccs_capture_sensitivity.csv

cargo run --quiet --bin deep_feasibility_tikz

cargo run --quiet --bin e1_engineering_economics

cargo run --quiet --bin e2_incremental_nuclear_benefit

cargo run --quiet --bin e2b_architecture_maturation

cargo run --quiet --bin e3_jurong_siting
