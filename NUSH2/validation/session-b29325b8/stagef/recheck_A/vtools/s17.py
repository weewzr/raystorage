W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(375, 812, True)
ev("location.hash='#results'")
sleep(1)
out("pos", ev(r"""(()=>{const t=Array.from(document.querySelectorAll('#route-results text.viz-rowlabel')).find(t=>t.textContent.startsWith('Capture capital per')); t.scrollIntoView({block:'center'}); const r=t.getBoundingClientRect(); return [Math.round(r.top), Math.round(r.right)]})()"""))
sleep(0.4)
shot(W + "/shots/tornado_375.png", False)
