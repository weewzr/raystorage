W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(1440, 900)
# select base radio via keyboard: focus mature radio then ArrowUp
ev("document.querySelector('input[name=scenario][value=mature]').focus()")
key("ArrowUp")
sleep(2.5)
out("radios", ev("Array.from(document.querySelectorAll('input[name=scenario]')).map(e=>[e.value,e.checked])"))
out("ts", ev("(async()=>{const m=await import('./app.js?v=54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16');return [m.state.inputs.ts_s_t, m.state.inputs.stage, m.state.inputs.gas_s_gj]})()"))
out("tiles", ev(r"""Array.from(document.querySelectorAll('#kpi-strip [data-kpi^=cost]')).map(e=>[e.dataset.kpi,e.innerText.replace(/\s+/g,' ')])"""))
out("note", ev("(document.querySelector('[data-scenario-note]')||{innerText:'(no note)'}).innerText"))
out("buttons", ev("Array.from(document.querySelectorAll('button')).map(b=>b.innerText.trim()).filter(Boolean).slice(0,40)"))
