W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
V = "54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16"
APP = "./app.js?v=" + V
VIEWS = "./views.js?v=" + V
vp(1440, 900)
nav("http://127.0.0.1:18765/index.html#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(0.5)

def solve_with(values):
    return ev(r"""(async()=>{const m=await import('%s');const prev=m.state.lastOk;m.resetInputs();m.setInputs(%s);const t0=performance.now();while(m.state.lastOk===prev&&performance.now()-t0<15000)await new Promise(r=>setTimeout(r,50));await new Promise(r=>setTimeout(r,300));location.hash='#dashboard';await new Promise(r=>setTimeout(r,400));return {targets:document.getElementById('targets').innerText, feas:document.getElementById('feasibility').hidden?'(hidden)':document.getElementById('feasibility').innerText, line:(document.querySelector('#dash-cost .target-line')||{}).innerText, sens:document.getElementById('sens-tag').hidden?'(hidden)':document.getElementById('sens-tag').innerText, flags:document.getElementById('flag-list').innerText, surplus:(document.querySelector('#kpi-strip [data-kpi=surplus_mwth]')||{}).innerText}})()""" % (APP, values))

out("M1_rating100", solve_with('{rating_mwth:100}'))
out("M2_10oak_gas25", solve_with('{stage:"10OAK_P46", gas_s_gj:25}'))
out("m5_eta036", solve_with('{eta_power:0.36}'))
out("m8_burn", solve_with('{tail_gas:"burn_on_site"}'))
out("m8_export", solve_with('{tail_gas:"export"}'))
out("reset", solve_with('{}'))
# m4: type 200.5 into rating box
ev("location.hash='#parameters'")
sleep(0.6)
ev(r"""(()=>{const e=document.getElementById('p-rating_mwth'); e.focus(); e.select();})()""")
text("200.5")
key("Enter")
sleep(2)
out("m4", ev(r"""(async()=>{const m=await import('%s');return [document.getElementById('p-rating_mwth').value, m.state.inputs.rating_mwth, m.state.lastOk.kpi.rating_mwth]})()""" % APP))
out("m5_ihx_ctrl", ev(r"""(()=>{const e=document.getElementById('p-ihx_approach_k'); const box=e.closest('.ctrl, .control, fieldset, div'); return box? box.innerText.replace(/\s+/g,' ').slice(0,600):null})()"""))
out("m5_eta_ctrl", ev(r"""(()=>{const e=document.getElementById('p-eta_power'); const box=e.closest('.ctrl, .control, fieldset, div'); return box? box.innerText.replace(/\s+/g,' ').slice(0,600):null})()"""))
solve_with('{}')
# m1 tornado
ev("location.hash='#results'")
sleep(1)
out("m1_tornado", ev(r"""(()=>{const f=document.querySelector('figure[data-chart=tornado-pre]'); if(!f) return null; const t=f.querySelector('table'); return (t? t.innerText : f.innerText).replace(/\t/g,' | ').slice(0,3000)})()"""))
out("m15_s13", ev(r"""(()=>{const r=document.querySelector('tr[data-stream-row=S13]'); if(!r) return null; const tb=r.closest('table'); const cap=tb.querySelector('caption'); const lede = tb.closest('section, div').querySelector('p'); return [r.innerText.replace(/\s+/g,' '), cap?cap.innerText:'', Array.from(tb.parentElement.parentElement.querySelectorAll('p.note, p.lede, p')).map(p=>p.innerText).filter(t=>/S13|PSA|headline/i.test(t)).slice(0,4)]})()"""))
out("stream_rows", ev(r"""Array.from(document.querySelectorAll('tr[data-stream-row]')).map(r=>r.dataset.streamRow).join(',')"""))
# m12 streams export
out("m12", ev(r"""(async()=>{const v=await import('%s');const s=v.streamsCsv();const t=typeof s==='string'?s:(s&&s.text)||JSON.stringify(s).slice(0,300);const lines=t.trim().split(/\r?\n/);return {n:lines.length, head:lines[0], first:lines[1], ids:lines.slice(1).map(l=>l.split(',')[1]).join(' ')}})()""" % VIEWS))
out("m16", ev(r"""(()=>{const t=document.body.textContent; const find=(re)=>{const m=t.match(re); return m? t.slice(Math.max(0,m.index-150), m.index+200).replace(/\s+/g,' '):null}; return {circ_secondary: find(/secondary circulator|circulators? .{0,40}both|both (?:helium )?(?:loops|circulators)/i), recycle_split: find(/recycle compressor and the cryogenic|split into|recycle_compressor/i), reported: find(/status word reported/i)}})()"""))
