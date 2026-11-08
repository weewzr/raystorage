import sys as _s
W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
js = open(W + "/tools/contrast2.js", encoding="utf-8").read()
vp(1440, 900)
nav("http://127.0.0.1:18765/index.html#dashboard", 1)
waitfor("document.querySelector('#kpi-strip [data-kpi]') ? 1 : 0")
sleep(1.5)
out("w1440", ev(js, timeout=180))
vp(375, 812, True)
nav("http://127.0.0.1:18765/index.html#dashboard", 1)
waitfor("document.querySelector('#kpi-strip [data-kpi]') ? 1 : 0")
sleep(1.5)
out("w375", ev(js, timeout=180))
