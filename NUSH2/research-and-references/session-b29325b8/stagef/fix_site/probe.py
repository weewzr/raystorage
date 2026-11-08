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
        s.call("Page.navigate", {"url": f"{origin}/index.html#dashboard"})
        time.sleep(1.5)
        solved()

        # F19 and F05: MATURE_60Y, then T&S 23.2 typed on Parameters.
        ev("document.querySelector('input[name=scenario][value=mature]').click()")
        solved()
        res["F05_mature"] = ev("""(() => ({
          caption: [...document.querySelectorAll('[data-mount=dashboard] caption, [data-mount=dashboard] .data-caption')].map(e => e.textContent).slice(0,2),
          dashtext: (document.querySelector('[data-mount=dashboard]').textContent.match(/reactor charged at [^.]*?gas at[^.]*?GJ/g) || []).slice(0,2),
          strip: document.querySelector('[data-target=cost]').textContent,
          radio: [...document.querySelectorAll('input[name=scenario]')].map(r => r.value + ':' + r.checked),
        }))()""")
        s.call("Page.printToPDF", {})  # warm
        pdf = s.call("Page.printToPDF", {"printBackground": False}, timeout=60)
        (out / "print_mature.pdf").write_bytes(base64.b64decode(pdf["data"]))
        ev("location.hash = '#parameters'")
        time.sleep(0.5)
        ev("(() => { const b = document.getElementById('p-ts_s_t'); b.value = '23.2'; b.dispatchEvent(new Event('input', {bubbles:true})); b.dispatchEvent(new Event('change', {bubbles:true})); })()")
        solved()
        ev("location.hash = '#dashboard'")
        time.sleep(0.6)
        res["F19_ts_after_mature"] = ev("""(() => ({
          radio: [...document.querySelectorAll('input[name=scenario]')].map(r => r.value + ':' + r.checked),
          note: (document.querySelector('[data-scenario-note]') || {}).dataset ? document.querySelector('[data-scenario-note]').dataset.scenarioNote : null,
          tile: document.querySelector('#kpi-strip [data-kpi=cost_s_per_t]').textContent.replace(/\\s+/g,' ').trim(),
        }))()""")
        # Case A after MATURE
        ev("document.querySelector('input[name=scenario][value=mature]').click()")
        solved()
        ev("location.hash = '#parameters'")
        time.sleep(0.4)
        ev("document.querySelector('[data-act=case_a]').click()")
        solved()
        ev("location.hash = '#dashboard'")
        time.sleep(0.6)
        res["F19_case_a_after_mature"] = ev("""(() => ({
          radio: [...document.querySelectorAll('input[name=scenario]')].map(r => r.value + ':' + r.checked),
          tile: document.querySelector('#kpi-strip [data-kpi=cost_s_per_t]').textContent.replace(/\\s+/g,' ').trim(),
        }))()""")
        # Base radio after modified inputs returns all defaults
        ev("document.querySelector('input[name=scenario][value=base]').click()")
        solved()
        res["F19_base"] = ev("""(() => ({
          radio: [...document.querySelectorAll('input[name=scenario]')].map(r => r.value + ':' + r.checked),
          tile: document.querySelector('#kpi-strip [data-kpi=cost_s_per_t] .k-value').textContent,
          dcf: document.querySelector('#kpi-strip [data-kpi=cost_lac_dcf_s_per_t] .k-value').textContent,
        }))()""")

        # F22: arrow keys on the scenario radios
        ev("document.querySelector('input[name=scenario][value=base]').focus()")
        key("ArrowDown", "ArrowDown", 40)
        solved()
        a1 = ev("(() => { const a = document.activeElement; return a.tagName + ':' + (a.name||'') + ':' + (a.value||'') + ':' + a.checked; })()")
        key("ArrowUp", "ArrowUp", 38)
        solved()
        a2 = ev("(() => { const a = document.activeElement; return a.tagName + ':' + (a.name||'') + ':' + (a.value||'') + ':' + a.checked; })()")
        res["F22_focus"] = [a1, a2]

        # F29 and F20 and F25 on Results
        ev("location.hash = '#results'")
        time.sleep(1.2)
        res["F29_ladder"] = ev("""(() => { const t = document.querySelector('#viz-cost-ladder-table'); return t ? [...t.querySelectorAll('tr')].map(r => r.textContent).filter(x => /Export|revenue|0\\.00/.test(x)) : null; })()""")
        res["F29_minus_zero"] = ev("document.querySelector('[data-mount=results]').textContent.includes('-0.00')")
        res["F20_tornado"] = ev("""(() => { const t = document.querySelector('#viz-tornado-pre-table'); return t ? [...t.querySelectorAll('tr')].map(r => [...r.children].map(c => c.textContent).join(' | ')).filter(x => /solvent/i.test(x)) : null; })()""")
        res["F25_status"] = ev("""(() => { const r = document.querySelector('[data-stream-row=S12]'); return r ? r.lastElementChild.textContent + ' / ' + r.children[1].textContent : null; })()""")
        res["F25_raw_words"] = ev("/flowsheet_m7|level4_m8|anchored_port/.test(document.querySelector('[data-mount=results]').textContent)")

        # F23 on the PFD
        ev("location.hash = '#pfd'")
        time.sleep(1.0)
        ev("document.querySelector('[data-stream=S10]').focus()")
        key("Enter", "Enter", 13)
        time.sleep(0.4)
        before = ev("(() => { const t = document.querySelector('.pfd-tip, #pfd-tip, [role=tooltip]'); return t ? !t.hidden : 'no tip'; })()")
        key("Escape", "Escape", 27)
        time.sleep(0.4)
        after = ev("(() => { const t = document.querySelector('.pfd-tip, #pfd-tip, [role=tooltip]'); const a = document.activeElement; return [t ? t.hidden : 'no tip', a.getAttribute('data-stream') || a.tagName]; })()")
        res["F23_escape"] = {"tip_shown_after_enter": before, "after_one_escape_hidden_focus": after}

        # Old review m10: skip link and focusable streams
        res["m10"] = ev("({skip: !!document.querySelector('[data-skip]'), streamTab: [...document.querySelectorAll('[data-stream]')].filter(g => g.tabIndex >= 0).length})")

        # F24 and m11 at 375 px
        size(375, 812, True)
        ev("location.hash = '#results'")
        time.sleep(1.5)
        res["F24_overflow"] = ev("""(() => { const out = []; for (const t of document.querySelectorAll('text.viz-rowlabel')) { const svg = t.ownerSVGElement.getBoundingClientRect(); const r = t.getBoundingClientRect(); if (r.width && r.right > svg.right + 0.5) out.push(t.textContent + ' ' + Math.round(r.right) + '>' + Math.round(svg.right)); } return {n: document.querySelectorAll('text.viz-rowlabel').length, over: out}; })()""")
        res["page_hscroll_results_375"] = ev("document.documentElement.scrollWidth > window.innerWidth")
        ev("location.hash = '#pfd'")
        time.sleep(1.0)
        res["page_hscroll_pfd_375"] = ev("document.documentElement.scrollWidth > window.innerWidth")
        size(1440, 900, False)
        # Export: notices link
        ev("location.hash = '#export'")
        time.sleep(0.6)
        res["F40_link"] = ev("(() => { const a = document.querySelector('[data-notices]'); return a ? a.getAttribute('href') : null; })()")
        res["F40_fetch"] = ev("fetch(document.querySelector('[data-notices]').href).then(r => r.status + ' ' + r.headers.get('content-type'))")
        time.sleep(0.5)
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
doc = pymupdf.open(out / "print_mature.pdf")
text = "".join(p.get_text() for p in doc)
res["F21_print"] = {"pages": doc.page_count, "MATURE": text.count("MATURE_60Y"), "base189.7": "189.7" in text,
                    "gas20.7": "S$20.7/GJ" in text}
print(json.dumps(res, indent=1, ensure_ascii=False))
