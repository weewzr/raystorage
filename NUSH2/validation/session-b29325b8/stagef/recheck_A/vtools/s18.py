W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(1440, 900)
for route in ["dashboard", "pfd"]:
    nav("http://127.0.0.1:18765/index.html?p=1#" + route, 0.5)
    waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
    sleep(1.5)
    pdf(W + "/out/print_" + route + ".pdf", preferCSSPageSize=True)
    out("done_" + route, True)
