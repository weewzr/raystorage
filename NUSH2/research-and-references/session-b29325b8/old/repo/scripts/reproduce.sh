#!/bin/sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT/model"
cargo test --all-targets
sh scripts/generate_gate5_results.sh
cd "$ROOT/paper"
latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex
