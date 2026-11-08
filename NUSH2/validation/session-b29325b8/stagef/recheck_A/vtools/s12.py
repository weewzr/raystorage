V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
firsts = []
for i in range(8):
    nav("http://127.0.0.1:18765/index.html?r=%d" % i, 0.5)
    waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
    firsts.append(ev("document.getElementById('status-text').innerText"))
out("first_solve_status", firsts)
out("full_solves", ev(r"""(async()=>{const m=await import('./app.js?v=%s');const d=Object.fromEntries(m.state.registry.params.map(p=>[p.id,p.default]));const full=[],kpi=[];for(let i=0;i<20;i++){const r=await m.rpc('solve',{schema:1,inputs:d,kpi_only:false});full.push(r.ms)}for(let i=0;i<20;i++){const r=await m.rpc('solve',{schema:1,inputs:d,kpi_only:true});kpi.push(r.ms)}const med=a=>{const s=[...a].sort((x,y)=>x-y);return Math.round(s[Math.floor(s.length/2)]*10)/10};return {full_median:med(full),full_max:Math.round(Math.max(...full)*10)/10,kpi_median:med(kpi),kpi_max:Math.round(Math.max(...kpi)*10)/10}})()""" % V, timeout=120))
