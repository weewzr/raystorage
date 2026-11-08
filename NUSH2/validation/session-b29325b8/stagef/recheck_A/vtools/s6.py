W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
ev("location.hash = '#parameters'"); sleep(0.8)
ev(r"""Array.from(document.querySelectorAll('button')).find(b=>b.innerText.trim()==='Reset to headline basis').focus()""")
out("focused", ev("document.activeElement.innerText.slice(0,60)"))
key("Enter")
sleep(2.5)
out("diff_from_default", ev("(async()=>{const m=await import('./app.js?v=%s');const r=m.state.registry;return r.params.filter(p=>String(m.state.inputs[p.id])!==String(p.default)).map(p=>[p.id,m.state.inputs[p.id],p.default])})()" % V))
out("tiles", ev(r"""Array.from(document.querySelectorAll('#kpi-strip [data-kpi^=cost]')).map(e=>[e.dataset.kpi,e.innerText.replace(/\s+/g,' ')])"""))
out("focus_after", ev("document.activeElement.tagName + ' ' + (document.activeElement.innerText||'').slice(0,40)"))
out("radios", ev("Array.from(document.querySelectorAll('input[name=scenario]')).map(e=>[e.value,e.checked])"))
# the 'Cost scenario MATURE_60Y' button: where is it?
out("scn_btn", ev(r"""(()=>{const b=Array.from(document.querySelectorAll('button')).find(b=>b.innerText.includes('Cost scenario MATURE_60Y'));return b? [b.closest('section').id, b.outerHTML.slice(0,200), b.parentElement.innerText.slice(0,300)]:null})()"""))
