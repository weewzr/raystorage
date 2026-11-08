#!/usr/bin/env python3
"""Run the browser self test of a built site in headless Chrome.

Serves the site folder on 127.0.0.1 with a free port, starts headless Chrome
with a remote debugging port, opens index.html?selftest=1 over the DevTools
protocol and polls document.body.dataset.selftest until frontend/selftest.js
writes "pass" or "fail". It then reads the JSON verdict from
<pre id="selftest-verdict"> and prints each check. The engine runs in a Web
Worker, which --dump-dom does not wait for, so the protocol is used.

The run fails when the page verdict is not "pass", when no verdict arrives
within --timeout seconds, when Chrome reports a console error, an uncaught
exception, an error log entry or a failed request, when the page asks any host
other than the local server, or when --expect-commit differs from build.json.
Python stdlib only; the DevTools client is the one in scripts/wasm_smoke.py.

Usage:
  python scripts/selftest.py --site _site [--chrome PATH] [--no-sandbox]
      [--expect-commit SHA] [--timeout 90]
Exit 0 on pass, 1 on any failure, Chrome missing included.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wasm_smoke import Session, WebSocket, free_port, get_json, problems  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
POLL_S = 0.5


def find_chrome(given):
    if given:
        if Path(given).is_file():
            return given
        return shutil.which(given)
    env = os.environ.get("CHROME")
    if env and Path(env).is_file():
        return env
    for base in (os.environ.get("PROGRAMFILES"), os.environ.get("PROGRAMFILES(X86)")):
        if not base:
            continue
        for rel in ("Google/Chrome/Application/chrome.exe", "Microsoft/Edge/Application/msedge.exe"):
            p = Path(base) / rel
            if p.is_file():
                return str(p)
    for name in ("google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "chrome"):
        hit = shutil.which(name)
        if hit:
            return hit
    return None


class Quiet(SimpleHTTPRequestHandler):
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm",
                      ".js": "text/javascript", ".mjs": "text/javascript", ".json": "application/json",
                      ".svg": "image/svg+xml", ".csv": "text/csv"}

    def log_message(self, *args):
        pass


def network_problems(events, origin):
    """Requests to any other host, HTTP errors and failed loads."""
    out, urls = [], {}
    for e in events:
        m, p = e["method"], e.get("params", {})
        if m == "Network.requestWillBeSent":
            url = p.get("request", {}).get("url", "")
            urls[p.get("requestId")] = url
            s = urlsplit(url)
            if s.scheme in ("data", "blob", "about"):
                continue
            if f"{s.scheme}://{s.netloc}" != origin:
                out.append(f"request to another host: {s.scheme}://{s.netloc}")
        elif m == "Network.responseReceived":
            status = p.get("response", {}).get("status", 0)
            if status >= 400:
                out.append(f"HTTP {status}: {urlsplit(p.get('response', {}).get('url', '')).path}")
        elif m == "Network.loadingFailed":
            if not p.get("canceled"):
                url = urls.get(p.get("requestId"), "")
                out.append(f"load failed: {urlsplit(url).path} {p.get('errorText', '')}".strip())
    return out, len(urls)


def evaluate(s, expr):
    r = s.call("Runtime.evaluate", {"expression": expr, "returnByValue": True})
    return r.get("result", {}).get("value")


def run(args):
    site = args.site.resolve()
    if not (site / "index.html").is_file():
        print(f"FAIL: {args.site} holds no index.html; run scripts/build_site.py")
        return 1
    chrome = find_chrome(args.chrome)
    if not chrome:
        print("FAIL: Chrome not found; pass --chrome")
        return 1
    build = json.loads((site / "build.json").read_text(encoding="utf-8"))
    fails = []
    if args.expect_commit and build.get("commit") != args.expect_commit:
        fails.append(f"build.json commit {build.get('commit')} differs from the expected {args.expect_commit}")

    port, dport = free_port(), free_port()
    server = ThreadingHTTPServer(("127.0.0.1", port), partial(Quiet, directory=str(site)))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{port}"
    verdict, events, state = None, [], None
    with tempfile.TemporaryDirectory() as tmp:
        cmd = [chrome, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
               "--disable-extensions", "--disable-background-networking", "--disable-component-update",
               "--disable-sync", f"--user-data-dir={Path(tmp) / 'profile'}", f"--remote-debugging-port={dport}"]
        if args.no_sandbox:
            cmd.append("--no-sandbox")
        log = open(Path(tmp) / "chrome.log", "wb")
        browser = subprocess.Popen(cmd + ["about:blank"], stdout=subprocess.DEVNULL, stderr=log)
        try:
            targets = get_json(f"http://127.0.0.1:{dport}/json/list")
            page = next(t for t in targets if t.get("type") == "page")
            s = Session(WebSocket(page["webSocketDebuggerUrl"]))
            for dom in ("Runtime.enable", "Log.enable", "Page.enable", "Network.enable"):
                s.call(dom)
            t0 = time.time()
            s.call("Page.navigate", {"url": f"{origin}/index.html?selftest=1"})
            while time.time() - t0 < args.timeout:
                state = evaluate(s, "document.body ? (document.body.dataset.selftest || '') : ''")
                if state in ("pass", "fail"):
                    break
                time.sleep(POLL_S)
            elapsed = time.time() - t0
            if state in ("pass", "fail"):
                text = evaluate(s, "(() => { const e = document.getElementById('selftest-verdict'); "
                                   "return e ? e.textContent : null; })()")
                verdict = json.loads(text) if text else None
            time.sleep(0.5)
            evaluate(s, "1")
            events = s.events
        finally:
            browser.terminate()
            try:
                browser.wait(timeout=10)
            except subprocess.TimeoutExpired:
                browser.kill()
                browser.wait()
            log.close()
            err = (Path(tmp) / "chrome.log").read_bytes()
            server.shutdown()
    if state not in ("pass", "fail"):
        tail = (err or b"").decode("utf-8", "replace").strip().splitlines()[-5:]
        print(f"FAIL: no verdict within {args.timeout} s")
        for line in tail:
            print(f"chrome: {line}")
        return 1
    if verdict is None:
        print(f"FAIL: verdict {state} but no #selftest-verdict element")
        return 1

    print(f"site commit {build.get('commit')}, run {build.get('run_id')}, verdict {verdict['verdict']} "
          f"after {elapsed:.1f} s, page time {verdict.get('ms')} ms")
    if verdict.get("note"):
        print(f"note: {verdict['note']}")
    for c in verdict.get("checks", []):
        print(f"{c['status'].upper():4} {c['id']}: {c.get('detail', '')}")
    if verdict.get("commit") != build.get("commit"):
        fails.append(f"page stamp {verdict.get('commit')} differs from build.json {build.get('commit')}")
    if verdict["verdict"] != "pass":
        fails.append(f"page verdict {verdict['verdict']}, {verdict.get('failed')} check(s) failed")
    errs, notes = problems(events)
    net, n_req = network_problems(events, origin)
    print(f"browser: {len(errs)} error(s), {len(notes)} other console or log line(s), {n_req} request(s), "
          f"{len(net)} network problem(s)")
    for line in errs[:10] + net[:10]:
        print(f"  {line}")
    if errs:
        fails.append(f"{len(errs)} browser error(s)")
    if net:
        fails.append(f"{len(net)} network problem(s)")
    if fails:
        print("FAIL: " + "; ".join(fails))
        return 1
    print("PASS")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="Headless Chrome self test of a built site")
    ap.add_argument("--site", type=Path, default=ROOT / "_site")
    ap.add_argument("--chrome", default=None, help="Chrome or Edge executable, path or name on PATH")
    ap.add_argument("--no-sandbox", action="store_true", help="pass --no-sandbox, for CI containers")
    ap.add_argument("--expect-commit", default=None, help="commit build.json must carry")
    ap.add_argument("--timeout", type=float, default=90.0, help="seconds to wait for the verdict")
    return run(ap.parse_args(argv))


if __name__ == "__main__":
    sys.exit(main())
