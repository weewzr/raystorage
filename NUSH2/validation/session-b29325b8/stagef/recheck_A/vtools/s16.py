import base64
from pathlib import Path
W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(375, 812, True)
ev("location.hash='#results'")
sleep(1)
info = ev(r"""(()=>{const out=[]; for(const t of document.querySelectorAll('#route-results text.viz-rowlabel')){const r=t.getBoundingClientRect(); if(r.right>window.innerWidth+1){const f=t.closest('figure'); const svg=t.ownerSVGElement; const sr=svg.getBoundingClientRect(); out.push({chart:f?f.dataset.chart:null, text:t.textContent, left:Math.round(r.left), right:Math.round(r.right), svgRight:Math.round(sr.right), svgOverflow:getComputedStyle(svg).overflow, anchor:t.getAttribute('text-anchor')}); t.scrollIntoView({block:'center'});}} return out})()""")
out("overflow_labels", info)
sleep(0.4)
r = ev(r"""(()=>{const t=Array.from(document.querySelectorAll('#route-results text.viz-rowlabel')).find(t=>t.getBoundingClientRect().right>window.innerWidth+1); t.scrollIntoView({block:'center'}); const f=t.closest('figure').getBoundingClientRect(); return [0, Math.max(0,f.top), 375, Math.min(812, f.height)]})()""")
x, y, w, h = r
res = S.call("Page.captureScreenshot", {"format": "png", "clip": {"x": x, "y": y, "width": w, "height": h, "scale": 1}})
Path(W + "/shots/rowlabel_375.png").write_bytes(base64.b64decode(res["data"]))
