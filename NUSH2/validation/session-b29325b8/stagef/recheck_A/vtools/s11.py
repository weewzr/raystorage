import base64
from pathlib import Path
W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(1440, 900)
ev("location.hash='#pfd'")
sleep(0.8)
def clipshot(sel, path, pad=40):
    r = ev(r"""(()=>{const e=document.querySelector('%s'); e.scrollIntoView({block:'center'}); e.focus(); const r=e.getBoundingClientRect(); return [r.left,r.top,r.width,r.height]})()""" % sel)
    sleep(0.3)
    x, y, w, h = r
    res = S.call("Page.captureScreenshot", {"format": "png", "clip": {"x": max(0, x - pad), "y": max(0, y - pad), "width": w + 2 * pad, "height": h + 2 * pad, "scale": 2}})
    Path(path).write_bytes(base64.b64decode(res["data"]))
    return r
# keyboard focus via Tab so :focus-visible applies: focus previous then Tab
ev("document.querySelector('#pfd-select').focus()")
key("Tab")
out("first_after_select", ev("document.activeElement.dataset ? JSON.stringify(document.activeElement.dataset) : ''"))
r = ev(r"""(()=>{const e=document.activeElement; const r=e.getBoundingClientRect(); return [r.left,r.top,r.width,r.height]})()""")
x, y, w, h = r
res = S.call("Page.captureScreenshot", {"format": "png", "clip": {"x": max(0, x - 40), "y": max(0, y - 40), "width": w + 80, "height": h + 80, "scale": 2}})
Path(W + "/shots/focus_first_pfd.png").write_bytes(base64.b64decode(res["data"]))
out("focus_visible_rule", ev(r"""(()=>{const out=[];for(const s of document.styleSheets){try{for(const r of s.cssRules){if(r.selectorText&&/focus/.test(r.selectorText)&&/(unit|stream|pfd)/.test(r.selectorText))out.push(r.cssText.slice(0,200))}}catch(e){}}return out})()"""))
