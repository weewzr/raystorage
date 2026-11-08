V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
ev("location.hash='#validation'")
sleep(1)
out("val_summary", ev(r"""document.querySelector('#route-validation').innerText.slice(0,900)"""))
out("result_csv", ev(r"""(async()=>{const v=await import('./views.js?v=%s');const a=await import('./app.js?v=%s');const s=v.resultCsv(a.state);const t=typeof s==='string'?s:(s.text||'');const lines=t.trim().split(/\r?\n/);const cls={};let na=0;for(const l of lines.slice(1)){const c=l.split(',');const k=c[c.length-1];cls[k]=(cls[k]||0)+1; if(/,n\/a,/.test(l)) na++;} return {n:lines.length-1, head:lines[0], cls, na, kpiRows: lines.filter(l=>l.startsWith('kpi,')).slice(0,4)}})()""" % (V, V)))
ev("location.hash='#export'")
sleep(0.8)
out("export_route", ev(r"""document.querySelector('#route-export').innerText.slice(0,1500)"""))
