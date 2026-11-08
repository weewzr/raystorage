W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A2/vout"
V = "9c13c49a13c164712be409a377e4c0a013dcb183"
APP = "./app.js?v=" + V
VIEWS = "./views.js?v=" + V
BASE = "http://127.0.0.1:18865/index.html"
DESC = r"""(()=>{const a=document.activeElement; return a.tagName.toLowerCase()+(a.name?'[name='+a.name+']':'')+(a.value&&a.tagName==='INPUT'?'[value='+a.value+']':'')+(a.dataset&&a.dataset.stream?'[stream='+a.dataset.stream+']':'')+" '"+(a.innerText||a.getAttribute('aria-label')||'').trim().slice(0,40).replace(/\s+/g,' ')+"'"})()"""
RADIOS = "Array.from(document.querySelectorAll('input[name=scenario]')).map(e=>e.value+':'+e.checked).join(' ')"
TILES = r"""Array.from(document.querySelectorAll('#kpi-strip [data-kpi^=cost]')).map(e=>e.dataset.kpi+' = '+e.innerText.replace(/\s+/g,' '))"""
NOTE = "(document.querySelector('[data-scenario-note]')||{dataset:{scenarioNote:'(none)'}}).dataset.scenarioNote"


def wait_solve(prev_js="1"):
    sleep(2.5)


def settle():
    return ev(r"""(async()=>{const m=await import('%s');let n=0;const t0=performance.now();let last=m.state.lastOk;while(performance.now()-t0<15000){await new Promise(r=>setTimeout(r,300));if(m.state.lastOk===last&&!m.state.busy){n++; if(n>3) break;} else {n=0; last=m.state.lastOk;}}return 1})()""" % APP)


vp(1440, 900)
nav(BASE + "#dashboard", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)

# F19 a: pick MATURE_60Y with a real click
ev("document.querySelector('input[name=scenario][value=mature]').click()")
sleep(3); settle()
out("F19a_mature_radios", ev(RADIOS))
out("F19a_mature_tiles", ev(TILES))
out("F19a_note", ev(NOTE))
# F05: captions and verdict name the gas price
out("F05_captions", ev(r"""Array.from(document.querySelectorAll('#route-dashboard caption, #dash-cost caption')).map(c=>c.innerText.replace(/\s+/g,' ')).filter(t=>/S\$ per tonne|Same layout/.test(t))"""))
out("F05_targets", ev("document.getElementById('targets').innerText.replace(/\\s+/g,' ')"))
# F21: print at MATURE_60Y
pdf(W + "/out/print_mature.pdf", preferCSSPageSize=True)
out("F21_print_line", ev("(document.querySelector('[data-print-scenario]')||{}).textContent || null"))

# F19 b: MATURE, then T&S 23.2 through the number box
ev("location.hash = '#parameters'"); sleep(0.8)
ev(r"""(()=>{const e=document.getElementById('p-ts_s_t'); e.focus(); e.select(); return e.value})()""")
text("23.2")
key("Enter")
sleep(3); settle()
out("F19b_ts", ev("document.getElementById('p-ts_s_t').value"))
ev("location.hash = '#dashboard'"); sleep(1)
out("F19b_radios", ev(RADIOS))
out("F19b_tiles", ev(TILES))
out("F19b_note", ev(NOTE))
out("F19b_targets", ev("document.getElementById('targets').innerText.replace(/\\s+/g,' ')"))

# F19 c: MATURE, then basis Case A with the keyboard
ev("document.querySelector('input[name=scenario][value=mature]').click()")
sleep(3); settle()
out("F19c_mature_again", [ev(RADIOS), ev(TILES)])
ev("location.hash = '#parameters'"); sleep(0.8)
ev(r"""document.querySelector('select[data-param="basis"]').focus()""")
key("ArrowDown")
sleep(3); settle()
out("F19c_basis", ev(r"""document.querySelector('select[data-param="basis"]').value"""))
ev("location.hash = '#dashboard'"); sleep(1)
out("F19c_radios", ev(RADIOS))
out("F19c_tiles", ev(TILES))
out("F19c_note", ev(NOTE))

# F19 d: Base radio resets every input
ev("document.querySelector('input[name=scenario][value=base]').click()")
sleep(3); settle()
out("F19d_radios", ev(RADIOS))
out("F19d_tiles", ev(TILES))
out("F19d_inputs", ev(r"""(async()=>{const m=await import('%s');return [m.state.inputs.basis, m.state.inputs.ts_s_t, m.state.inputs.stage, m.state.inputs.gas_s_gj, m.state.lastOk.kpi.cost_s_per_t, m.state.lastOk.kpi.cost_lac_dcf_s_per_t]})()""" % APP))

# F22: arrow keys on the scenario radios keep focus in the group
ev("document.querySelector('input[name=scenario][value=base]').focus()")
out("F22_before", ev(DESC))
key("ArrowDown"); sleep(3); settle()
out("F22_after_down", [ev(DESC), ev(RADIOS)])
key("ArrowUp"); sleep(3); settle()
out("F22_after_up", [ev(DESC), ev(RADIOS)])

# F23: one Escape closes a pinned stream tip
nav(BASE + "#pfd", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)
ev("document.querySelector('#pfd-svg [data-stream=\"S10\"]').focus()")
sleep(0.2)
key("Enter"); sleep(0.3)
out("F23_pinned", [ev("document.getElementById('pfd-tip').hidden"), ev(DESC)])
key("Escape"); sleep(0.4)
out("F23_esc1", [ev("document.getElementById('pfd-tip').hidden"), ev(DESC)])

# F20, F25, F29 on Results at the base inputs
nav(BASE + "#results", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1.5)
out("F20_tornado", ev(r"""(()=>{const f=document.querySelector('figure[data-chart=tornado-pre]'); if(!f) return null; const t=f.querySelector('table'); const rows=Array.from((t||f).querySelectorAll('tr')).map(r=>r.innerText.replace(/\t/g,' | ').replace(/\s+/g,' ')); return rows.filter(r=>/solvent/i.test(r))})()"""))
out("F25_status_words", ev(r"""(()=>{const t=document.querySelector('#route-results').innerText; return {flowsheet_m7: (t.match(/flowsheet_m7/g)||[]).length, flowsheet_m8: (t.match(/flowsheet_m8/g)||[]).length, anchored_port: (t.match(/anchored_port/g)||[]).length, gated: (t.match(/gated_T0/g)||[]).length, level4_m8: (t.match(/level4_m8/g)||[]).length, model_m6: (t.match(/model_m6/g)||[]).length}})()"""))
out("F25_S12", ev(r"""(()=>{const r=document.querySelector('tr[data-stream-row=S12]'); return r? r.innerText.replace(/\s+/g,' '):null})()"""))
out("F29_ladder", ev(r"""(()=>{const f=document.querySelector('figure[data-chart=cost-ladder]'); if(!f) return null; const b=f.querySelector('button'); if(b) b.click(); const t=f.querySelector('table'); return t? t.innerText.split('\n').filter(l=>/Export revenue|-0\.00|None at/.test(l)):null})()"""))
out("F29_negzero", ev("(document.querySelector('#route-results').innerText.match(/-0\\.00\\b/g)||[]).length"))

# F40 notices
nav(BASE + "#export", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1)
out("F40_link", ev("(()=>{const a=document.querySelector('a[data-notices]'); return a? a.getAttribute('href'):null})()"))
out("F40_fetch", ev("(async()=>{const r=await fetch('THIRD_PARTY_NOTICES.txt'); const t=await r.text(); return [r.status, r.headers.get('content-type'), t.length, (t.match(/^## /gm)||[]).length]})()"))
out("F28_text", ev(r"""(()=>{const t=document.querySelector('#route-export').innerText; const m=t.match(/[^.\n]*A4[^.\n]*/g); return m})()"""))

# F24 at 375 px
vp(375, 812, True)
nav(BASE + "#results", 0.5)
waitfor("document.getElementById('status-text').innerText.startsWith('Solved') ? 1 : 0", 30000)
sleep(1.5)
out("F24_overflow", ev(r"""(()=>{const out=[]; let n=0; for(const t of document.querySelectorAll('text.viz-rowlabel')){n++; const r=t.getBoundingClientRect(); const svg=t.ownerSVGElement; const sr=svg.getBoundingClientRect(); if(r.width>0 && (r.right>sr.right+1 || r.left<sr.left-1 || r.right>window.innerWidth+1)) out.push([t.textContent, Math.round(r.left), Math.round(r.right), Math.round(sr.right)]);} return {labels:n, overflow:out, scrollW: document.documentElement.scrollWidth, innerW: window.innerWidth}})()"""))
ev("location.hash='#pfd'"); sleep(1.2)
out("F24_pfd_scroll", ev("[document.documentElement.scrollWidth, window.innerWidth]"))
ev("location.hash='#dashboard'"); sleep(1.2)
out("dash_scroll_375", ev("[document.documentElement.scrollWidth, window.innerWidth]"))
