W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
nav("http://127.0.0.1:18765/index.html?p=2#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)
ev(r"""(async()=>{const v=await import('./views.js?v=%s');const a=await import('./app.js?v=%s');const prev=a.state.lastOk;v.applyScenario('mature');const t0=performance.now();while(a.state.lastOk===prev&&performance.now()-t0<10000)await new Promise(r=>setTimeout(r,50));return 1})()""" % (V, V))
sleep(1.5)
pdf(W + "/out/print_mature.pdf", preferCSSPageSize=True)
out("ok", True)
