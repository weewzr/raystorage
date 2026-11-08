BASE = "http://127.0.0.1:18865/index.html"
vp(1440, 900)
nav(BASE + "#results", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1.5)
out("gated_ctx", ev(r"""(()=>{const t=document.querySelector('#route-results').innerText; const out=[]; const re=/.{0,160}gated_T0\w*.{0,160}/g; let m; while((m=re.exec(t))&&out.length<5) out.push(m[0].replace(/\s+/g,' ')); return out})()"""))
out("raw_status_all_routes", ev(r"""(async()=>{const res={}; for(const r of ['dashboard','pfd','results','parameters','validation','export']){location.hash='#'+r; await new Promise(x=>setTimeout(x,900)); const t=document.body.innerText; res[r]=(t.match(/\b(?:flowsheet_m[78]|anchored_port|gated_T0\w*|level4_m8|model_m[4-6])\b/g)||[]).slice(0,8);} return res})()"""))
