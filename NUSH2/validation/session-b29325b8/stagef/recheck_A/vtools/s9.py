W = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/stagef/recheck_A/vout"
DESC = r"""(()=>{const a=document.activeElement; if(!a) return 'none'; const r=a.getBoundingClientRect(); return `${a.tagName.toLowerCase()}${a.id?'#'+a.id:''}${a.dataset&&a.dataset.unit?'[unit='+a.dataset.unit+']':''}${a.dataset&&a.dataset.stream?'[stream='+a.dataset.stream+']':''} '${(a.innerText||a.textContent||a.getAttribute('aria-label')||'').trim().slice(0,40).replace(/\s+/g,' ')}' top ${Math.round(r.top)} left ${Math.round(r.left)} w ${Math.round(r.width)} outline ${getComputedStyle(a).outlineStyle}/${getComputedStyle(a).outlineWidth}`})()"""
import sys
w, h, mob = (375, 812, True) if "m" in sys.argv[-1:][0] else (1440, 900, False)
vp(1440, 900)
nav("http://127.0.0.1:18765/index.html", 1)
waitfor("document.querySelector('#kpi-strip [data-kpi]') ? 1 : 0")
sleep(1)
stops = []
for i in range(9):
    key("Tab")
    stops.append(ev(DESC))
out("first_stops", stops)
# skip link: shift-tab back to first, enter
nav("http://127.0.0.1:18765/index.html", 1)
waitfor("document.querySelector('#kpi-strip [data-kpi]') ? 1 : 0")
sleep(0.5)
key("Tab")
out("skip_focused", ev(DESC))
key("Enter")
sleep(0.3)
out("after_skip", ev(DESC))
key("Tab")
out("after_skip_tab", ev(DESC))
# go to nav link PFD with keyboard: focus Dashboard tab then Tab once and Enter
ev("document.querySelector('nav.tabs a[data-route=dashboard]').focus()")
key("Tab")
out("nav_focus", ev(DESC))
key("Enter")
sleep(1)
out("hash_after_enter", ev("location.hash"))
# PFD units: focus first unit, ArrowRight, Enter
out("unit_tabindex", ev("Array.from(document.querySelectorAll('#pfd-svg [data-unit]')).map(g=>g.getAttribute('tabindex')).join(',')"))
ev("document.querySelector('#pfd-svg [data-unit][tabindex=\"0\"]').focus()")
out("unit_focus", ev(DESC))
key("ArrowRight")
out("unit_after_arrow", ev(DESC))
key("Enter")
sleep(0.5)
out("unit_pressed", ev("Array.from(document.querySelectorAll('#pfd-svg [data-unit][aria-pressed=true]')).map(g=>g.dataset.unit)"))
out("panel_head", ev("(document.querySelector('#pfd-panel h3, #pfd-panel h2')||{}).innerText"))
# stream: tab from the unit until a stream gets focus (count)
n = 0
got = None
for i in range(60):
    key("Tab", gap=0.02)
    n += 1
    d = ev("document.activeElement.dataset && document.activeElement.dataset.stream || ''")
    if d:
        got = d
        break
out("tabs_to_first_stream", [n, got, ev(DESC)])
out("tip_on_focus", ev("(()=>{const t=document.getElementById('pfd-tip');return [t.hidden, t.innerText.slice(0,200)]})()"))
ev("document.querySelector('#pfd-svg [data-stream=\"S12\"]').focus()")
out("s12_focus", ev(DESC))
key("Enter")
sleep(0.3)
out("s12_enter_focus", ev(DESC))
out("tip_pinned", ev("(()=>{const t=document.getElementById('pfd-tip');return [t.hidden, t.innerText.slice(0,300)]})()"))
key("Enter")
sleep(2)
out("after_link", [ev("location.hash"), ev(DESC), ev("(()=>{const r=document.querySelector('tr.is-target');return r? r.dataset.streamRow + ' ' + r.innerText.replace(/\\s+/g,' ').slice(0,200):null})()")])
shot(W + "/shots/stream_link_1440.png", False)
# Escape test: back to pfd, focus S10, Enter, Escape
ev("location.hash='#pfd'")
sleep(0.8)
ev("document.querySelector('#pfd-svg [data-stream=\"S10\"]').focus()")
key("Enter")
sleep(0.2)
key("Escape")
sleep(0.2)
out("after_escape", [ev("document.getElementById('pfd-tip').hidden"), ev(DESC)])
