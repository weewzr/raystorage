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
        for route in ("dashboard", "pfd", "results"):
            s.call("Page.navigate", {"url": f"{origin}/index.html#{route}"})
            time.sleep(1.5)
            solved()
            pdf = s.call("Page.printToPDF", {"preferCSSPageSize": True}, timeout=60)
            (out / f"print_{route}_a4.pdf").write_bytes(base64.b64decode(pdf["data"]))
        ev("1")
        errs, notes = problems(s.events)
        res["console_errors"] = errs
        res["requests_other_host"] = sorted({e['params']['request']['url'] for e in s.events if e.get('method') == 'Network.requestWillBeSent' and not e['params']['request']['url'].startswith((origin, 'data:', 'blob:'))})
        res["failed_loads"] = [e['params'].get('errorText') for e in s.events if e.get('method') == 'Network.loadingFailed' and not e['params'].get('canceled')]
    finally:
        stop_browser(br, dport)
        srv.shutdown()

import pymupdf
for route in ("dashboard", "pfd", "results"):
    d = pymupdf.open(out / f"print_{route}_a4.pdf")
    res[route] = [d.page_count, round(d[0].rect.width), round(d[0].rect.height)]
print(json.dumps(res))
