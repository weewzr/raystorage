"""Verifier driver: serve the built site, open #pfd in headless Chrome, and for
each input set walk the tour (still form), dumping every step's card text,
every caption number with its source path, and the full engine reply.

Usage: python drive.py <repo> <site> <out_dir>
"""
import json
import sys
import threading
import time
from functools import partial
from http.server import ThreadingHTTPServer
from pathlib import Path
import subprocess
import tempfile

REPO = Path(sys.argv[1])
SITE = Path(sys.argv[2]).resolve()
OUT = Path(sys.argv[3])
OUT.mkdir(parents=True, exist_ok=True)
sys.path.insert(0, str(REPO / "scripts"))
from wasm_smoke import Session, WebSocket, free_port, get_json  # noqa: E402
from selftest import Quiet, find_chrome, stop_browser, attach_workers  # noqa: E402

COMMIT = json.loads((SITE / "build.json").read_text(encoding="utf-8"))["commit"]

SETS = [("controls", {"p_abs_bar": 22, "p_strip_bar": 1.5, "srd": 4.2, "ihx_approach_k": 50, "eta_power": 0.45, "availability": 0.7, "ts_s_t": 100, "capture_target": 0.92}), ("ahn", {"heat_basis": "ahn_2026"}), ("recfix", {"tail_gas": "recycle_fixed"})]
OLD_SETS = [
    ("default", None),
    ("rating100", {"rating_mwth": 100}),
    ("case_a", {"basis": "case_a"}),
    ("export", {"tail_gas": "export"}),
    ("burn", {"tail_gas": "burn_on_site"}),
    ("selexol", {"tail_gas": "inl_selexol"}),
    ("case_a_burn", {"basis": "case_a", "tail_gas": "burn_on_site"}),
]


def ev(s, expr, timeout=30):
    r = s.call("Runtime.evaluate", {"expression": expr, "returnByValue": True, "awaitPromise": True}, timeout=timeout)
    if "exceptionDetails" in r:
        raise RuntimeError(json.dumps(r["exceptionDetails"])[:800])
    return r.get("result", {}).get("value")


WALK = r"""
(async () => {
  const app = await import('./app.js?v=%(c)s');
  const tour = await import('./tour.js?v=%(c)s');
  const panels = await import('./panels.js?v=%(c)s');
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const values = %(v)s;
  const prev = app.state.lastOk;
  app.resetInputs();
  if (values) app.setInputs(values);
  const t0 = performance.now();
  while (performance.now() - t0 < 20000) {
    if (app.state.lastOk && app.state.lastOk !== prev) break;
    await sleep(100);
  }
  await sleep(400);
  const res = app.state.lastOk;
  const st = tour.tourState();
  tour.startTour({ reduced: true, step: 0 });
  const steps = [];
  for (let i = 0; i < st.steps; i += 1) {
    tour.goTour(i);
    await sleep(80);
    const card = document.getElementById('tour-card');
    const nums = Array.from(card.querySelectorAll('.tn')).map((b) => {
      const p = b.dataset.path || null;
      const row = b.dataset.row || null;
      const raw = p ? panels.walk(res, p) : null;
      return { path: p, row, u: b.dataset.u, d: b.dataset.d, shown: b.textContent, raw,
               ctx: b.closest('dd') ? 'fig' : 'text', label: b.closest('div[data-fig]') ? b.closest('div[data-fig]').querySelector('dt').textContent : null };
    });
    steps.push({
      i: i + 1,
      title: document.getElementById('tour-title').textContent,
      text: Array.from(card.querySelectorAll('.tour-text p')).map((p) => p.textContent),
      tags: Array.from(card.querySelectorAll('.tour-tag')).map((t) => t.textContent),
      warn: Array.from(card.querySelectorAll('.tour-warn')).map((t) => t.textContent),
      scen: Array.from(card.querySelectorAll('.tour-scen')).map((t) => t.textContent),
      eq: Array.from(card.querySelectorAll('.tour-eq')).map((t) => t.textContent),
      status: Array.from(card.querySelectorAll('.tour-status')).map((t) => t.textContent),
      notes: Array.from(card.querySelectorAll('.tour-notes li')).map((t) => t.textContent),
      inset: Array.from(card.querySelectorAll('figure')).map((f) => ({ label: f.getAttribute('aria-label'), cap: f.querySelector('figcaption').textContent, svgText: Array.from(f.querySelectorAll('text')).map((t) => t.textContent) })),
      media: Array.from(card.querySelectorAll('.tour-media li')).map((t) => t.textContent),
      figs: Array.from(card.querySelectorAll('.tour-figs > div')).map((d) => d.textContent),
      live: document.getElementById('tour-live').textContent,
      lit: Array.from(document.querySelectorAll('#pfd-svg .pfd-units .tour-on')).map((g) => g.dataset.unit),
      moving: Array.from(document.querySelectorAll('#pfd-svg .tour-move')).map((g) => g.dataset.stream),
      full: card.textContent,
      nums,
    });
  }
  tour.exitTour(false);
  return JSON.stringify({ inputs: res && res.inputs, status: res && res.status, steps, reply: res });
})()
"""


def main():
    chrome = find_chrome(None)
    port, dport = free_port(), free_port()
    server = ThreadingHTTPServer(("127.0.0.1", port), partial(Quiet, directory=str(SITE)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    origin = f"http://127.0.0.1:{port}"
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp:
        cmd = [chrome, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
               "--disable-extensions", "--disable-background-networking", "--disable-component-update",
               "--disable-sync", f"--user-data-dir={Path(tmp) / 'profile'}", f"--remote-debugging-port={dport}"]
        browser = subprocess.Popen(cmd + ["about:blank"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            targets = get_json(f"http://127.0.0.1:{dport}/json/list")
            page = next(t for t in targets if t.get("type") == "page")
            s = Session(WebSocket(page["webSocketDebuggerUrl"]))
            for dom in ("Runtime.enable", "Log.enable", "Page.enable"):
                s.call(dom)
            s.call("Target.setAutoAttach", {"autoAttach": True, "waitForDebuggerOnStart": False, "flatten": True})
            s.call("Emulation.setDeviceMetricsOverride", {"width": 1440, "height": 900, "deviceScaleFactor": 1, "mobile": False})
            s.call("Page.navigate", {"url": f"{origin}/index.html#pfd"})
            t0 = time.time()
            while time.time() - t0 < 60:
                ok = ev(s, "(async () => { try { const app = await import('./app.js?v=%s'); const t = await import('./tour.js?v=%s'); return Boolean(app.state.lastOk && app.state.registry && t.tourState().ready); } catch (e) { return false; } })()" % (COMMIT, COMMIT))
                if ok:
                    break
                time.sleep(0.5)
            reg = ev(s, "(async () => { const app = await import('./app.js?v=%s'); return JSON.stringify(app.state.registry); })()" % COMMIT)
            (OUT / "registry.json").write_text(reg, encoding="utf-8")
            for name, vals in SETS:
                js = WALK % {"c": COMMIT, "v": json.dumps(vals)}
                out = ev(s, js, timeout=120)
                (OUT / f"walk_{name}.json").write_text(out, encoding="utf-8")
                d = json.loads(out)
                print(name, d["status"], {k: d["inputs"].get(k) for k in ("rating_mwth", "basis", "tail_gas")}, len(d["steps"]), "steps")
            errs = [e for e in s.events if e.get("method") in ("Runtime.exceptionThrown",) or
                    (e.get("method") == "Runtime.consoleAPICalled" and e["params"].get("type") == "error") or
                    (e.get("method") == "Log.entryAdded" and e["params"]["entry"].get("level") == "error")]
            print("errors", len(errs))
            for e in errs[:5]:
                print(json.dumps(e)[:400])
        finally:
            stop_browser(browser, dport)
            server.shutdown()


main()
