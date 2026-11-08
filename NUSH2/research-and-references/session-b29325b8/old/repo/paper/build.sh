#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
(
  cd model
  cargo test --all-targets
)
sh model/scripts/generate_gate5_results.sh
cat results/final_design/generated/reliability_sensitivity.md
cargo run --quiet --manifest-path model/Cargo.toml --bin final_design_ccs_summary
cd paper
latexmk -C
latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex
if grep -E "undefined references|undefined citations|Citation .* undefined|Reference .* undefined" main.log; then
  echo "Unresolved LaTeX references/citations detected" >&2
  exit 1
fi
