V = "9c13c49a13c164712be409a377e4c0a013dcb183"
APP = "./app.js?v=" + V
BASE = "http://127.0.0.1:18865/index.html"
vp(1440, 900)
nav(BASE + "#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)
PROBE = r"""(async()=>{const m=await import('%s');const prev=m.state.lastOk;m.resetInputs();m.setInputs(%s);const t0=performance.now();while(m.state.lastOk===prev&&performance.now()-t0<15000)await new Promise(r=>setTimeout(r,50));await new Promise(r=>setTimeout(r,300));const s=m.state.lastOk;const e=(id)=>{const x=(s.balances.energy||[]).find(z=>z.id===id);return x?x.value:null};const p=(id)=>{const x=(s.balances.power||[]).find(z=>z.id===id);return x?[x.value,x.label]:null};const msg=(s.messages||[]).filter(z=>z.code==='import_memo_high').map(z=>z.text);return {feasible:s.feasibility.feasible, kpi_unmet_power:s.kpi.unmet_power_mwe, need:e('htgr_heat_need'), output:e('htgr_heat_output'), gen:e('electricity_generation'), circ:e('helium_circulation'), proc:e('process_electricity'), unmet_electrical:e('unmet_electrical'), import_at_rated:p('import_at_rated'), memo_msg:msg}})()"""
for name, inp in [("r178.4", "{rating_mwth:178.4}"), ("r178.494", "{rating_mwth:178.494}"), ("r179.5", "{rating_mwth:179.5}"),
                  ("r181", "{rating_mwth:181}"), ("r210", "{rating_mwth:210}"), ("r170", "{rating_mwth:170}"), ("r100", "{rating_mwth:100}"),
                  ("caseA_mea", "{basis:'case_a'}"), ("caseA_sel", "{basis:'case_a', tail_gas:'inl_selexol'}")]:
    out(name, ev(PROBE % (APP, inp)))
# KPI label in the registry and the flag text on Results
out("kpi_label", ev(r"""(async()=>{const m=await import('%s'); const k=(m.state.registry&&m.state.registry.kpis)||[]; const x=Array.isArray(k)? k.find(z=>z.id==='unmet_power_mwe'):null; return x? x.label : null})()""" % APP))
ev(r"""(async()=>{const m=await import('%s'); m.resetInputs(); return 1})()""" % APP)
sleep(3)
ev("location.hash='#results'"); sleep(1.5)
out("results_memo", ev(r"""(()=>{const t=document.querySelector('#route-results').innerText; return (t.match(/.{0,120}(?:grid import|Grid import|J\.29).{0,160}/g)||[]).slice(0,6).map(s=>s.replace(/\s+/g,' '))})()"""))
out("results_status_flag", ev(r"""(()=>{const t=document.body.innerText; return (t.match(/.{0,60}status=\w+.{0,40}/g)||[]).slice(0,4)})()"""))
ev("location.hash='#dashboard'"); sleep(1.5)
out("dash_import_row", ev(r"""(()=>{const t=document.querySelector('#route-dashboard').innerText; return (t.match(/.{0,40}Grid import.{0,120}/g)||[]).slice(0,4).map(s=>s.replace(/\s+/g,' '))})()"""))
