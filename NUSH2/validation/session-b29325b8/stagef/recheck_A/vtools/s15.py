W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(375, 812, True)
nav("http://127.0.0.1:18765/index.html#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(0.8)
res = {}
for r in ["dashboard", "pfd", "parameters", "results", "export", "validation"]:
    ev(f"location.hash='#{r}'")
    sleep(0.8)
    res[r] = ev(r"""(()=>{const d=document.documentElement; const clipped=[]; const main=document.querySelector('#route-%s'); for(const e of main.querySelectorAll('*')){ if(e.closest('[hidden]')) continue; const r=e.getBoundingClientRect(); if(!r.width) continue; if(r.right>window.innerWidth+1 || r.left < -1){ let p=e.parentElement, scroll=false; while(p && p!==document.body){const cs=getComputedStyle(p); if(/(auto|scroll|hidden)/.test(cs.overflowX) && p.getBoundingClientRect().right<=window.innerWidth+1){scroll=cs.overflowX; break;} p=p.parentElement} if(!scroll) clipped.push(e.tagName+'.'+(e.className&&e.className.baseVal!==undefined?e.className.baseVal:e.className)+' right '+Math.round(r.right)); } } return {scrollW:d.scrollWidth, clientW:d.clientWidth, bodyScrollW: document.body.scrollWidth, outside: clipped.slice(0,8), nOutside: clipped.length}})()""" % r)
out("layout375", res)
# m11: select HTGR in the unit picker
ev("location.hash='#pfd'")
sleep(0.6)
ev(r"""(()=>{const s=document.getElementById('pfd-select'); s.value='htgr'; s.dispatchEvent(new Event('change',{bubbles:true})); s.dispatchEvent(new Event('input',{bubbles:true}));})()""")
sleep(0.8)
out("m11", ev(r"""(()=>{const p=document.getElementById('pfd-panel'); const head=(p.querySelector('h3,h2')||{}).innerText; const items=[]; for(const e of p.querySelectorAll('*')){ if(!/B\.20/.test(e.textContent) || e.children.length>3) continue; const r=e.getBoundingClientRect(); const cs=getComputedStyle(e); items.push({tag:e.tagName, cls:String(e.className), text:e.textContent.trim().slice(0,80), right:Math.round(r.right), sw:e.scrollWidth, cw:e.clientWidth, ox:cs.overflowX, ws:cs.whiteSpace}); } const pr=p.getBoundingClientRect(); const wide=[]; for(const e of p.querySelectorAll('*')){ if(e.scrollWidth>e.clientWidth+1 && !/(auto|scroll)/.test(getComputedStyle(e).overflowX) && e.clientWidth>0) wide.push(e.tagName+'.'+e.className+' sw '+e.scrollWidth+' cw '+e.clientWidth+' ox '+getComputedStyle(e).overflowX+' '+e.textContent.trim().slice(0,50)); } return {head, panelRight:Math.round(pr.right), items:items.slice(0,6), wide:wide.slice(0,8)}})()"""))
shot(W + "/shots/pfd_htgr_375.png", True)
