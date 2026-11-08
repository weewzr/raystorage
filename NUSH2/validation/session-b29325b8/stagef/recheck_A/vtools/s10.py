W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
DESC = r"""(()=>{const a=document.activeElement; if(!a) return 'none'; return `${a.tagName.toLowerCase()} '${(a.getAttribute('aria-label')||a.textContent||'').trim().slice(0,60).replace(/\s+/g,' ')}' in ${a.closest('figure')?a.closest('figure').dataset.chart:'-'}`})()"""
COUNT = r"""(()=>{const sel='a[href],button,input,select,textarea,summary,[tabindex]';const els=Array.from(document.querySelectorAll(sel)).filter(e=>{if(e.closest('[hidden]'))return false; if(e.hasAttribute('disabled'))return false; const t=e.getAttribute('tabindex'); if(t!==null&&Number(t)<0)return false; if(e.tagName==='INPUT'&&e.type==='radio'&&!e.checked&&document.querySelector(`input[type=radio][name="${e.name}"]:checked`))return false; const r=e.getClientRects(); if(!r.length)return false; const cs=getComputedStyle(e); return cs.visibility==='visible'}); const perFig={}; els.forEach(e=>{const f=e.closest('figure[data-chart]'); if(f) perFig[f.dataset.chart]=(perFig[f.dataset.chart]||0)+1}); return {total: els.length, perFig}})()"""
vp(1440, 900)
res = {}
for r in ["dashboard", "pfd", "parameters", "results", "export", "validation"]:
    ev(f"location.hash='#{r}'")
    sleep(0.8)
    res[r] = ev(COUNT)
out("tabstops", res)
# chart keyboard: focus first chart's tab stop on results, arrow right
ev("location.hash='#results'")
sleep(0.8)
out("first_chart", ev(r"""(()=>{const f=document.querySelector('#route-results figure[data-chart]'); const t=f.querySelector('[tabindex="0"]'); t.focus(); return f.dataset.chart})()"""))
out("c0", ev(DESC))
key("ArrowRight")
out("c1", ev(DESC))
key("ArrowRight")
out("c2", ev(DESC))
out("charttip", ev(r"""(()=>{const t=document.querySelector('.viz-tip:not([hidden]), [role=tooltip]:not([hidden])'); return t? t.innerText.slice(0,120): null})()"""))
key("Tab")
out("after_tab", ev(DESC))
