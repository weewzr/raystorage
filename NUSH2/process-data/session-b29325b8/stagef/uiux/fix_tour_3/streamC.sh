#!/bin/bash
PY="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/venv/Scripts/python.exe"; S="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/site"; O="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/final"
cd "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/tools"
"$PY" -B fullrun.py "$S" "$O/full" 1440 900 > "$O/full_1440.log" 2>&1
"$PY" -B fullrun.py "$S" "$O/full" 1440 900 --reduced > "$O/full_1440_rm.log" 2>&1
"$PY" -B fullrun.py "$S" "$O/full" 375 812 --mobile > "$O/full_375.log" 2>&1
"$PY" -B fullrun.py "$S" "$O/full" 375 812 --mobile --reduced > "$O/full_375_rm.log" 2>&1
"$PY" -B keys.py "$S" "$O/keys1440" 1440 900 > "$O/keys.log" 2>&1
"$PY" -B tabpath.py "$S" 1440 900 > "$O/tab_1440.log" 2>&1
"$PY" -B tabpath.py "$S" 375 812 --mobile > "$O/tab_375.log" 2>&1
"$PY" -B kbselect.py "$S" > "$O/kbselect.log" 2>&1
"$PY" -B misc.py "$S" "$O/misc_side.png" > "$O/misc.log" 2>&1
"$PY" -B spacecard.py "$S" > "$O/spacecard.log" 2>&1
"$PY" -B closing.py "$S" > "$O/closing.log" 2>&1
"$PY" -B closecheck.py "$S" > "$O/closecheck.log" 2>&1
"$PY" -B burnbranch.py "$S" > "$O/burnbranch.log" 2>&1
"$PY" -B steps.py "$S" "$O/steps_auto_1440" 1440 900 auto > "$O/steps_auto_1440.log" 2>&1
"$PY" -B steps.py "$S" "$O/steps_auto_375" 375 812 auto --mobile > "$O/steps_auto_375.log" 2>&1
"$PY" -B steps.py "$S" "$O/steps_next_1440" 1440 900 next > "$O/steps_next_1440.log" 2>&1
"$PY" -B steps.py "$S" "$O/steps_next_375" 375 812 next --mobile > "$O/steps_next_375.log" 2>&1
"$PY" -B walk.py "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/base_tree" "$S" "$O/walks" > "$O/walk.log" 2>&1
"$PY" -B walk2.py "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/base_tree" "$S" "$O/walks2" > "$O/walk2.log" 2>&1
"$PY" -B numcheck.py "$O/walks" "$O/walks2" > "$O/numcheck.log" 2>&1
echo streamC done > "$O/streamC.done"
