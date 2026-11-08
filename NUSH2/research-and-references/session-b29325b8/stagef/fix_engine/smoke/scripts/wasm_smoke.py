#!/usr/bin/env python3
"""Load the page shell with the wasm engine in headless Chrome and check it.

Assembles a local site from frontend/, frontend/nush2.wasm and results/ with
scripts/build_site.py (commit "local"), serves it with python -m http.server,
starts headless Chrome with a remote debugging port, attaches over the
DevTools protocol, opens index.html?selftest=1 and polls the self-test report
the page writes. The
engine runs in a Web Worker, which --dump-dom with a virtual time budget does
not wait for, so the protocol is used instead. Python stdlib only.

The check passes when the registry loads, the default solve returns status ok
with every required KPI key, the page caught no error, and Chrome reported no
console error, uncaught exception or error log entry. The browser KPIs are then
compared with the native solve of the same crate, written by its unit test
through NUSH2_WASM_DUMP. Native and wasm use different libm builds, so numbers
compare to relative 1e-9 with a floor of 1e-3.

Usage: python scripts/build_wasm.py && python scripts/wasm_smoke.py
Exit 0 on pass, 1 on fail, 2 when Chrome is absent.
"""
import base64
import json
import os
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "frontend"
TOL = 1e-9
WAIT_S = 60
# Windows install folders come from the environment, so no drive letter is
# written here.
CHROME_PATHS = [
    str(Path(base) / rel)
    for base in (os.environ.get("PROGRAMFILES"), os.environ.get("PROGRAMFILES(X86)"))
    if base
    for rel in ("Google/Chrome/Application/chrome.exe", "Microsoft/Edge/Application/msedge.exe")
] + [
    "/usr/bin/google-chrome",
    "/usr/bin/chromium-browser",
    "/usr/bin/chromium",
]


def find_chrome():
    env = os.environ.get("CHROME")
    if env and Path(env).is_file():
        return env
    for p in CHROME_PATHS:
        if Path(p).is_file():
            return p
    return shutil.which("chrome") or shutil.which("google-chrome") or shutil.which("chromium")


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class WebSocket:
    """Minimal RFC 6455 client: text frames, client masking, no extensions."""

    def __init__(self, url):
        u = urlparse(url)
        self.sock = socket.create_connection((u.hostname, u.port), timeout=10)
        key = base64.b64encode(os.urandom(16)).decode()
        req = (f"GET {u.path} HTTP/1.1\r\nHost: {u.hostname}:{u.port}\r\nUpgrade: websocket\r\n"
               f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n")
        self.sock.sendall(req.encode())
        head = b""
        while b"\r\n\r\n" not in head:
            chunk = self.sock.recv(4096)
            if not chunk:
                raise ConnectionError("websocket handshake closed")
            head += chunk
        if b" 101 " not in head.split(b"\r\n", 1)[0]:
            raise ConnectionError(head.split(b"\r\n", 1)[0].decode(errors="replace"))
        self.buf = head.split(b"\r\n\r\n", 1)[1]

    def send(self, obj):
        data = json.dumps(obj).encode()
        n = len(data)
        if n < 126:
            hdr = struct.pack("!BB", 0x81, 0x80 | n)
        elif n < 65536:
            hdr = struct.pack("!BBH", 0x81, 0x80 | 126, n)
        else:
            hdr = struct.pack("!BBQ", 0x81, 0x80 | 127, n)
        mask = os.urandom(4)
        body = bytes(b ^ mask[i % 4] for i, b in enumerate(data))
        self.sock.sendall(hdr + mask + body)

    def _read(self, n):
        while len(self.buf) < n:
            chunk = self.sock.recv(65536)
            if not chunk:
                raise ConnectionError("websocket closed")
            self.buf += chunk
        out, self.buf = self.buf[:n], self.buf[n:]
        return out

    def recv(self, timeout):
        self.sock.settimeout(timeout)
        parts = []
        while True:
            b0, b1 = self._read(2)
            n = b1 & 0x7F
            if n == 126:
                n = struct.unpack("!H", self._read(2))[0]
            elif n == 127:
                n = struct.unpack("!Q", self._read(8))[0]
            mask = self._read(4) if b1 & 0x80 else None
            payload = self._read(n)
            if mask:
                payload = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
            op = b0 & 0x0F
            if op == 0x8:
                raise ConnectionError("websocket closed by peer")
            if op in (0x9, 0xA):
                continue
            parts.append(payload)
            if b0 & 0x80:
                return json.loads(b"".join(parts).decode("utf-8"))


class Session:
    def __init__(self, ws):
        self.ws = ws
        self.next_id = 0
        self.events = []

    def call(self, method, params=None, timeout=15, session=None):
        """Send one command, to the page or, with `session`, to an attached target
        such as the engine worker, and wait for its reply. Events that arrive in
        the meantime, from any session, are kept in order."""
        self.next_id += 1
        mid = self.next_id
        msg = {"id": mid, "method": method, "params": params or {}}
        if session:
            msg["sessionId"] = session
        self.ws.send(msg)
        end = time.time() + timeout
        while time.time() < end:
            try:
                msg = self.ws.recv(max(0.1, end - time.time()))
            except socket.timeout:
                break
            if msg.get("id") == mid:
                if "error" in msg:
                    raise RuntimeError(f"{method}: {msg['error']}")
                return msg.get("result", {})
            if "method" in msg:
                self.events.append(msg)
        raise TimeoutError(method)


def get_json(url, tries=50):
    for _ in range(tries):
        try:
            with urllib.request.urlopen(url, timeout=2) as r:
                return json.loads(r.read().decode())
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(url)


def native_kpi(tmp):
    """Run the crate's default-solve test with a dump path and read the reply."""
    out = Path(tmp) / "native.json"
    env = dict(os.environ, NUSH2_WASM_DUMP=str(out))
    subprocess.run(["cargo", "test", "-q", "-p", "nush2_wasm", "--lib", "default_solve_has_every_section_and_kpi"],
                   cwd=ROOT, env=env, check=True, capture_output=True)
    return json.loads(out.read_text(encoding="utf-8"))["kpi"]


def problems(events):
    """Console errors, uncaught exceptions and error log entries."""
    errs, notes = [], []
    for e in events:
        m, p = e["method"], e.get("params", {})
        if m == "Runtime.consoleAPICalled":
            text = " ".join(str(a.get("value", a.get("description", ""))) for a in p.get("args", []))
            (errs if p.get("type") in ("error", "assert") else notes).append(f"console.{p.get('type')}: {text}")
        elif m == "Runtime.exceptionThrown":
            d = p.get("exceptionDetails", {})
            errs.append(f"exception: {d.get('text')} {d.get('exception', {}).get('description', '')}".strip())
        elif m == "Log.entryAdded":
            entry = p.get("entry", {})
            line = f"log.{entry.get('level')}: {entry.get('text')} {entry.get('url', '')}".strip()
            (errs if entry.get("level") == "error" else notes).append(line)
    return errs, notes


def build_site(tmp):
    """Assemble a local site from frontend/, the module and results/, since the
    page reads data/*.json that only scripts/build_site.py writes."""
    out = Path(tmp) / "site"
    r = subprocess.run([sys.executable, str(ROOT / "scripts" / "build_site.py"), "--wasm", str(SITE / "nush2.wasm"),
                        "--results", str(ROOT / "results"), "--commit", "local", "--out", str(out)],
                       cwd=ROOT, capture_output=True, text=True)
    if r.returncode != 0:
        raise RuntimeError(f"build_site.py failed: {(r.stdout + r.stderr).strip()}")
    return out


def main():
    # Page titles and engine text carry subscripts; a console on a legacy code
    # page would fail to print them.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if not (SITE / "nush2.wasm").is_file():
        print("FAIL: frontend/nush2.wasm is missing, run scripts/build_wasm.py")
        return 1
    chrome = find_chrome()
    if not chrome:
        print("SKIP: Chrome not found")
        return 2
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp:
        native = native_kpi(tmp)
        site = build_site(tmp)
        port, dport = free_port(), free_port()
        server = subprocess.Popen([sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1",
                                   "--directory", str(site)],
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        browser = subprocess.Popen([chrome, "--headless=new", "--disable-gpu", "--no-first-run",
                                    "--no-default-browser-check", "--disable-extensions",
                                    f"--user-data-dir={Path(tmp) / 'profile'}", f"--remote-debugging-port={dport}",
                                    "about:blank"],
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            targets = get_json(f"http://127.0.0.1:{dport}/json/list")
            page = next(t for t in targets if t.get("type") == "page")
            s = Session(WebSocket(page["webSocketDebuggerUrl"]))
            for dom in ("Runtime.enable", "Log.enable", "Page.enable"):
                s.call(dom)
            url = f"http://127.0.0.1:{port}/index.html?selftest=1"
            t0 = time.time()
            s.call("Page.navigate", {"url": url})
            report = None
            while time.time() - t0 < WAIT_S:
                r = s.call("Runtime.evaluate", {
                    "expression": "(() => { const e = document.getElementById('selftest-report'); return e ? e.textContent : null; })()",
                    "returnByValue": True,
                })
                text = r.get("result", {}).get("value")
                if text:
                    report = json.loads(text)
                    break
                time.sleep(0.25)
            title = s.call("Runtime.evaluate", {"expression": "document.title", "returnByValue": True})
            title = title.get("result", {}).get("value")
            time.sleep(0.5)
            try:
                s.call("Runtime.evaluate", {"expression": "1", "returnByValue": True})
            except TimeoutError:
                pass
            events = s.events
        finally:
            browser.terminate()
            server.terminate()
            for p in (browser, server):
                try:
                    p.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    p.kill()
    if report is None:
        print(f"FAIL: the page wrote no self-test report within {WAIT_S} s")
        return 1
    errs, notes = problems(events)
    print(f"title: {title}")
    print(f"engine line: {report.get('engine_line')}")
    print(f"registry loaded {report.get('registry_loaded')}, params {report.get('params')}, "
          f"solve {report.get('solve_status')} in {report.get('solve_ms')} ms, first solve {report.get('first_solve_ms')} ms, registry to first result {report.get('first_result_ms')} ms, page errors {len(report.get('errors', []))}, "
          f"browser errors {len(errs)}, other console or log lines {len(notes)}")
    for n in notes[:10]:
        print(f"note: {n}")
    fails = []
    if report.get("verdict") != "pass":
        fails.append(f"page verdict {report.get('verdict')}: {report.get('errors')} missing {report.get('missing_kpi_keys')} "
                     f"{report.get('load_error', '')}")
    if errs:
        fails.append("browser errors: " + " | ".join(errs[:5]))
    wasm = report.get("kpi") or {}
    diffs = []
    for k in sorted(set(native) | set(wasm)):
        a, b = native.get(k), wasm.get(k)
        if isinstance(a, (int, float)) and isinstance(b, (int, float)):
            if abs(a - b) > TOL * max(abs(a), abs(b), 1e-3):
                diffs.append(f"{k}: native {a} against wasm {b}")
        elif a != b:
            diffs.append(f"{k}: native {a!r} against wasm {b!r}")
    print(f"kpi keys {len(wasm)}, native against wasm differences {len(diffs)}")
    for d in diffs[:20]:
        print(d)
    if diffs:
        fails.append(f"{len(diffs)} kpi differences")
    if fails:
        print("FAIL: " + "; ".join(fails))
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
