#!/bin/bash
PY="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/venv/Scripts/python.exe"; S="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/site"; O="C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/final"
SIZES="1024x768 1200x800 1280x720 1366x768 1440x900 1536x864 1920x1080 2560x1440 812x375m 1024x600 768x1024m 375x667m 375x812m"
cd "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/uiux/fix_tour_3/tools"
for c in start startkey play next; do "$PY" -B playscroll.py "$S" $c $SIZES > "$O/playscroll_$c.log" 2>&1; done
"$PY" -B playscroll2.py "$S" next --no-anchor $SIZES > "$O/playscroll2_next_noanchor.log" 2>&1
"$PY" -B playscroll2.py "$S" next $SIZES > "$O/playscroll2_next.log" 2>&1
"$PY" -B firstnext.py "$S" $SIZES > "$O/firstnext.log" 2>&1
"$PY" -B firstnext2.py "$S" $SIZES > "$O/firstnext2.log" 2>&1
"$PY" -B everynext.py "$S" next $SIZES > "$O/everynext.log" 2>&1
"$PY" -B everynext.py "$S" prev $SIZES > "$O/everyprev.log" 2>&1
for m in card panel; do "$PY" -B readerscroll.py "$S" fast $m $SIZES > "$O/reader_fast_$m.log" 2>&1; "$PY" -B readerscroll.py "$S" fast $m --no-anchor $SIZES > "$O/reader_fast_${m}_noanchor.log" 2>&1; done
"$PY" -B readerscroll.py "$S" real card 1440x900 1024x600 375x812m > "$O/reader_real_card.log" 2>&1
"$PY" -B readerscroll.py "$S" real panel 1440x900 375x812m > "$O/reader_real_panel.log" 2>&1
echo streamB done > "$O/streamB.done"
