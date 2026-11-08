W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
vp(1440, 900)
FIND = r"""(()=>{const t=document.body.textContent; const out=[]; const re=/exceeds the headline|0\.1 mol%|9,059\.6|9059\.6/g; let m; while((m=re.exec(t))&&out.length<6){out.push(t.slice(Math.max(0,m.index-200), m.index+150).replace(/\s+/g,' '))} return out})()"""
for r in ["results", "pfd"]:
    ev(f"location.hash='#{r}'")
    sleep(0.8)
    out("find_" + r, ev(FIND))
ev("location.hash='#pfd'")
sleep(0.8)
ev("document.querySelector('#pfd-svg [data-stream=\"S13\"]').focus()")
sleep(0.3)
out("tip_s13", ev("document.getElementById('pfd-tip').innerText"))
out("stream_table_head", ev(r"""(()=>{const r=document.querySelector('tr[data-stream-row=S13]'); const tb=r.closest('table'); return Array.from(tb.querySelectorAll('thead th')).map(t=>t.innerText).join(' | ')})()"""))
out("ihx_text", ev(r"""(()=>{const e=document.getElementById('p-ihx_approach_k-range-text'); return e? e.innerText : Array.from(document.querySelectorAll('[id*=ihx_approach]')).map(x=>x.id+': '+x.innerText).join(' || ')})()"""))
