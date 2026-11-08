V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
vp(1440, 900)
out("lastok_cost", ev("(async()=>{const m=await import('./app.js?v=%s');return [m.state.lastOk.kpi.cost_s_per_t, m.state.lastOk.kpi.cost_lac_dcf_s_per_t, m.state.busy, m.state.seq]})()" % V))
out("status", ev("document.getElementById('status-text').innerText"))
ev("location.hash = '#dashboard'")
sleep(1)
out("tiles", ev(r"""Array.from(document.querySelectorAll('#kpi-strip [data-kpi^=cost]')).map(e=>[e.dataset.kpi,e.innerText.replace(/\s+/g,' ')])"""))
out("status2", ev("document.getElementById('status-text').innerText"))
