"""Probe the built site for the fix:site findings over the DevTools protocol.
Usage: python probe.py <site dir> <repo scripts dir> <out dir>"""
import base64, json, subprocess, sys, tempfile, threading, time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

site, scripts, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
out.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(scripts))
from wasm_smoke import Session, WebSocket, free_port, get_json, problems  # noqa
from selftest import find_chrome, stop_browser  # noqa


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


port, dport = free_port(), free_port()
srv = ThreadingHTTPServer(("127.0.0.1", port), partial(Quiet, directory=str(site)))
threading.Thread(target=srv.serve_forever, daemon=True).start()
origin = f"http://127.0.0.1:{port}"
chrome = find_chrome(None)
res = {}
with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp:
    br = subprocess.Popen([chrome, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
                           "--disable-extensions", "--disable-background-networking", f"--user-data-dir={tmp}/p",
                           f"--remote-debugging-port={dport}", "about:blank"],
                          stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        page = next(t for t in get_json(f"http://127.0.0.1:{dport}/json/list") if t.get("type") == "page")
        s = Session(WebSocket(page["webSocketDebuggerUrl"]))
        for d in ("Runtime.enable", "Log.enable", "Page.enable", "Network.enable"):
            s.call(d)
        s.call("Target.setAutoAttach", {"autoAttach": True, "waitForDebuggerOnStart": False, "flatten": True})

        def ev(expr, wait=True):
            r = s.call("Runtime.evaluate", {"expression": expr, "returnByValue": True, "awaitPromise": wait}, timeout=60)
            if "exceptionDetails" in r:
                return {"exception": r["exceptionDetails"].get("exception", {}).get("description")}
            return r.get("result", {}).get("value")

        def key(k, code, vk):
            for t in ("rawKeyDown", "keyUp"):
                s.call("Input.dispatchKeyEvent", {"type": t, "key": k, "code": code, "windowsVirtualKeyCode": vk})

        def size(w, h, mobile):
            s.call("Emulation.setDeviceMetricsOverride", {"width": w, "height": h, "deviceScaleFactor": 1, "mobile": mobile})

        def solved():
            # wait for a fresh "Solved in" after a change
            ev("new Promise(r => setTimeout(r, 400))")
            for _ in range(100):
                if ev("document.getElementById('status-text').textContent.startsWith('Solved')"):
                    break
                time.sleep(0.1)
            ev("new Promise(r => setTimeout(r, 300))")

        size(1440, 900, False)
        s.call("Page.navigate", {"url": f"{origin}/index.html#parameters"})
        time.sleep(1.5)
        solved()
        # m4: typed off-step value
        ev("(() => { const b = document.getElementById('p-rating_mwth'); b.value = '200.5'; b.dispatchEvent(new Event('input', {bubbles:true})); b.dispatchEvent(new Event('change', {bubbles:true})); })()")
        solved()
        res["m4"] = ev("({box: document.getElementById('p-rating_mwth').value, sent: (window.__nush2 || {}).x})")
        ev("location.hash = '#parameters'")
        ev("document.querySelector('[data-act=reset]').click()")
        solved()
        # m5: eta 0.36 flag text, ihx evidence text
        ev("(() => { const b = document.getElementById('p-eta_power'); if (!b) return; b.value = '0.36'; b.dispatchEvent(new Event('input', {bubbles:true})); b.dispatchEvent(new Event('change', {bubbles:true})); })()")
        solved()
        res["m5_flags"] = ev("[...document.querySelectorAll('.p-flag, .flag, [data-flag]')].map(e => e.textContent.trim()).filter(Boolean).slice(0,4)")
        res["m5_ihx"] = ev("(() => { const r = document.querySelector('[data-row=ihx_approach_k]'); return r ? r.textContent.replace(/\s+/g,' ').match(/Evidence range[^.]*/g) : null; })()")
        ev("document.querySelector('[data-act=reset]').click()")
        solved()
        # m6: rating 100 tile label
        ev("(() => { const b = document.getElementById('p-rating_mwth'); b.value = '100'; b.dispatchEvent(new Event('input', {bubbles:true})); b.dispatchEvent(new Event('change', {bubbles:true})); })()")
        solved()
        ev("location.hash = '#dashboard'")
        time.sleep(0.6)
        res["m6"] = ev("(() => { const t = document.querySelector('#kpi-strip [data-kpi=surplus_mwth]'); return t ? t.textContent.replace(/\s+/g,' ').trim() : null; })()")
        res["M1_strip"] = ev("document.querySelector('[data-target=cost]').dataset.met + ' ' + document.querySelector('[data-target=cost]').textContent.slice(0,160)")
        ev("location.hash = '#parameters'")
        time.sleep(0.3)
        ev("document.querySelector('[data-act=reset]').click()")
        solved()
        ev("location.hash = '#dashboard'")
        time.sleep(0.6)
        res["m6_default"] = ev("document.querySelector('#kpi-strip [data-kpi=surplus_mwth]').textContent.replace(/\s+/g,' ').trim()")
        res["m7_m8"] = ev("({h2rich: !!document.querySelector('#kpi-strip [data-kpi=h2_rich_kg_h]')})")
        # m9 contrast of chart tick text
        ev("location.hash = '#results'")
        time.sleep(1.2)
        res["m9"] = ev("""(() => { const lum = (c) => { const m = c.match(/\d+(\.\d+)?/g).map(Number); const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(m[0]) + 0.7152 * f(m[1]) + 0.0722 * f(m[2]); }; const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); }; const bg = getComputedStyle(document.body).backgroundColor; const out = {}; for (const sel of ['.viz-tick', 'text.viz-axis', '.viz-axis text', 'td.na', '.na']) { const e = document.querySelector(sel); if (e) { const c = getComputedStyle(e); out[sel] = [c.fill !== 'none' && e instanceof SVGElement ? c.fill : c.color, bg, Math.round(ratio(e instanceof SVGElement ? c.fill : c.color, bg) * 100) / 100]; } } return out; })()""")
        res["m10_results_tabstops"] = ev("[...document.querySelectorAll('#route-results a[href], #route-results button, #route-results input, #route-results select, #route-results [tabindex]')].filter(e => e.tabIndex >= 0 && !e.closest('[hidden]')).length")
        # m11 at 375: B.20 in the HTGR panel
        size(375, 812, True)
        ev("location.hash = '#pfd'")
        time.sleep(1.2)
        ev("(() => { const u = document.querySelector('[data-unit=htgr]'); if (u) u.dispatchEvent(new MouseEvent('click', {bubbles:true})); })()")
        time.sleep(0.8)
        res["m11"] = ev("""(() => { const eqs = [...document.querySelectorAll('*')].filter(e => e.children.length < 40 && /B\.20/.test(e.textContent) && e.textContent.length < 400); const e = eqs[eqs.length - 1]; if (!e) return 'B.20 not found'; const box = e.closest('.eq, .equation, li, div') || e; const r = box.getBoundingClientRect(); let sc = box; while (sc && sc !== document.body && !(sc.scrollWidth > sc.clientWidth && /auto|scroll/.test(getComputedStyle(sc).overflowX))) sc = sc.parentElement; return {right: Math.round(r.right), vw: window.innerWidth, scroller: sc && sc !== document.body ? sc.className : null, hscroll: document.documentElement.scrollWidth > window.innerWidth}; })()""")
        size(1440, 900, False)
        ev("1")
        errs, notes = problems(s.events)
        res["console_errors"] = errs
        res["requests_other_host"] = sorted({e['params']['request']['url'] for e in s.events if e.get('method') == 'Network.requestWillBeSent' and not e['params']['request']['url'].startswith((origin, 'data:', 'blob:'))})
        res["failed_loads"] = [e['params'].get('errorText') for e in s.events if e.get('method') == 'Network.loadingFailed' and not e['params'].get('canceled')]
    finally:
        stop_browser(br, dport)
        srv.shutdown()

try:
    import pymupdf
except ImportError:
    import fitz as pymupdf
doc = None
text = ""
pass

print(json.dumps(res, indent=1, ensure_ascii=False))
