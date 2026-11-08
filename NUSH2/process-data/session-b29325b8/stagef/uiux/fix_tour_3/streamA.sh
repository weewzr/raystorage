#!/bin/bash
PY="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/venv/Scripts/python.exe"; S="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/site"; O="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/final"
cd "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/tools"
"$PY" -B focusall.py "$S" > "$O/focusall.log" 2>&1
"$PY" -B focusall.py "$S" --reduced > "$O/focusall_rm.log" 2>&1
"$PY" -B buttons.py "$S" > "$O/buttons.log" 2>&1
"$PY" -B buttons.py "$S" --reduced > "$O/buttons_rm.log" 2>&1
"$PY" -B startview.py "$S" > "$O/startview.log" 2>&1
"$PY" -B startview.py "$S" --reduced > "$O/startview_rm.log" 2>&1
"$PY" -B nextfocus.py "$S" "$O/nextfocus" > "$O/nextfocus.log" 2>&1
echo streamA done > "$O/streamA.done"
