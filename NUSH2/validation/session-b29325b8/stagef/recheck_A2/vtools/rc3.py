V = "9c13c49a13c164712be409a377e4c0a013dcb183"
APP = "./app.js?v=" + V
BASE = "http://127.0.0.1:18865/index.html"
vp(1440, 900)
nav(BASE + "#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)
for r in [178.4, 179.5, 181, 182, 210]:
    out("r%s" % r, ev(r"""(async()=>{const m=await import('%s');const prev=m.state.lastOk;m.resetInputs();m.setInputs({rating_mwth:%s});const t0=performance.now();while(m.state.lastOk===prev&&performance.now()-t0<15000)await new Promise(r=>setTimeout(r,50));await new Promise(r=>setTimeout(r,300));const s=m.state.lastOk;const e=(id)=>{const x=(s.balances.energy||[]).find(z=>z.id===id);return x?x.value:null};const p=(id)=>{const x=(s.balances.power||[]).find(z=>z.id===id);return x?x.value:null};return {feasible:s.feasibility.feasible, kpi_unmet_power:s.kpi.unmet_power_mwe, kpi_surplus:s.kpi.surplus_mwth, need:e('htgr_heat_need'), unmet_electrical:e('unmet_electrical'), import_at_rated:p('import_at_rated'), unmet_thermal:e('unmet_thermal')}})()""" % (APP, r)))
out("kpi_tile_unmet", ev(r"""(()=>{const t=document.querySelector('#kpi-strip [data-kpi=unmet_power_mwe]'); return t? t.innerText.replace(/\s+/g,' '):null})()"""))
out("unmet_power_on_page", ev(r"""(async()=>{const res={}; for(const r of ['dashboard','results']){location.hash='#'+r; await new Promise(x=>setTimeout(x,900)); const t=document.body.innerText; const m=t.match(/.{0,80}(?:grid import|Grid import|unmet power|Unmet power|import at rat).{0,80}/g); res[r]=(m||[]).slice(0,5).map(s=>s.replace(/\s+/g,' '));} return res})()"""))
