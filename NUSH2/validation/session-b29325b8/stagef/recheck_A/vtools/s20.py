V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
ev("location.hash='#results'")
sleep(1)
out("figs", ev(r"""Array.from(document.querySelectorAll('figure[data-chart]')).map(f=>[f.dataset.chart, f.dataset.state||'', (f.querySelector('figcaption, .viz-title, h3, h4')||{}).innerText||'', f.querySelectorAll('svg path, svg rect, svg circle, svg line').length, (f.querySelector('.viz-reason')||{}).innerText||''])"""))
out("missing", ev(r"""(async()=>{const a=await import('./app.js?v=%s');return (a.state.lastOk.engine_fields_missing||[]).map(m=>m.field+': '+m.reason.slice(0,120))})()""" % V))
out("na_on_results", ev(r"""(()=>{const t=document.querySelector('#route-results').innerText; const out=[]; const re=/.{0,80}\bn\/a\b.{0,40}/g; let m; while((m=re.exec(t))&&out.length<20) out.push(m[0].replace(/\s+/g,' ')); return out})()"""))
