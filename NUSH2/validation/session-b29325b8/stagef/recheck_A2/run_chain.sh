#!/bin/bash
# Recheck chain on the git archive of HEAD. Logs go next to this script.
SP=/c/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad
R2=$SP/stagef/recheck_A2
H=$R2/head
PY=$SP/venv/Scripts/python.exe
export CARGO_TARGET_DIR=$SP/cargo/recheck_A2
export PYTHONIOENCODING=utf-8
export GIT_SHA=9c13c49a13c164712be409a377e4c0a013dcb183
cd $H
mkdir -p $R2/logs
s() { echo "=== $1 start $(date +%T)" >> $R2/logs/chain.log; }
e() { echo "=== $1 exit $2 at $(date +%T)" >> $R2/logs/chain.log; }

s test; ( time cargo test --workspace --release ) > $R2/logs/test_release.log 2>&1; e test $?
s export1; ( time cargo run --release -p nush2_cli -- export --out $R2/exp1 ) > $R2/logs/export1.log 2>&1; e export1 $?
s export2; ( time cargo run --release -p nush2_cli -- export --out $R2/exp2 ) > $R2/logs/export2.log 2>&1; e export2 $?
s cmp
for f in $H/results/*.csv; do b=$(basename $f); cmp $f $R2/exp1/$b >/dev/null 2>&1 && c1=same || c1=DIFF; cmp $f $R2/exp2/$b >/dev/null 2>&1 && c2=same || c2=DIFF; echo "$b exp1 $c1 exp2 $c2"; done > $R2/logs/cmp.log 2>&1
e cmp 0
s compare1; $PY scripts/compare_results.py results $R2/exp1 > $R2/logs/compare1.log 2>&1; e compare1 $?
s compare2; $PY scripts/compare_results.py results $R2/exp2 > $R2/logs/compare2.log 2>&1; e compare2 $?
s compare12; $PY scripts/compare_results.py $R2/exp1 $R2/exp2 > $R2/logs/compare12.log 2>&1; e compare12 $?
s wasm; ( time $PY scripts/build_wasm.py --out $R2/wasm/nush2.wasm ) > $R2/logs/build_wasm.log 2>&1; e wasm $?
s check_wasm; $PY scripts/check_wasm.py $R2/wasm/nush2.wasm > $R2/logs/check_wasm.log 2>&1; e check_wasm $?
echo "=== chain done $(date +%T)" >> $R2/logs/chain.log
