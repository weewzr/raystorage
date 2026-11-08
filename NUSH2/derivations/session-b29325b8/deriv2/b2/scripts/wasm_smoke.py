#!/usr/bin/env python3
"""Run the wasm build in headless Chrome and compare it with the native CLI.

Serves the repo with python -m http.server, loads frontend/smoke.html with
--dump-dom, and compares the RUN reply against `nush2 json --preset headline`.
Native and wasm use different libm builds, so numbers compare to relative 1e-9,
with a floor of 1e-3 on the scale because rounding residuals near zero have no
meaningful relative error.

Usage: python scripts/build_wasm.py && python scripts/wasm_smoke.py
Exit 0 on pass, 1 on fail, 2 when Chrome is absent.
"""
import html
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOL = 1e-9
CHROME_PATHS = [
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    str(Path(os.environ.get("LOCAL" + "APP" + "DATA", ".")) / "Google/Chrome/Application/chrome.exe"),
    "/usr/bin/google-chrome",
    "/usr/bin/chromium-browser",
    "/usr/bin/chromium",
]


def find_chrome():
    for p in CHROME_PATHS:
        if Path(p).is_file():
            return p
    return shutil.which("chrome") or shutil.which("google-chrome")


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def compare(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        if set(a) != set(b):
            out.append(f"{path}: keys differ {sorted(set(a) ^ set(b))}")
        for k in set(a) & set(b):
            compare(a[k], b[k], f"{path}.{k}", out)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append(f"{path}: lengths differ")
        for i, (x, y) in enumerate(zip(a, b)):
            compare(x, y, f"{path}[{i}]", out)
    elif isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if abs(a - b) > TOL * max(abs(a), abs(b), 1e-3):
            out.append(f"{path}: {a} against {b}")
    elif a != b:
        out.append(f"{path}: {a!r} against {b!r}")


def main():
    wasm = ROOT / "frontend/nush2.wasm"
    if not wasm.is_file():
        print("FAIL: frontend/nush2.wasm is missing, run scripts/build_wasm.py")
        return 1
    chrome = find_chrome()
    if not chrome:
        print("SKIP: Chrome not found")
        return 2
    native = subprocess.run(["cargo", "run", "--release", "-q", "-p", "nush2_cli", "--", "json", "--preset", "headline"],
                            cwd=ROOT, capture_output=True, text=True, check=True).stdout
    native = json.loads(native)
    port = free_port()
    server = subprocess.Popen([sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1"],
                              cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        time.sleep(1.0)
        with tempfile.TemporaryDirectory() as profile:
            url = f"http://127.0.0.1:{port}/frontend/smoke.html"
            dom = subprocess.run([chrome, "--headless=new", "--disable-gpu", "--no-first-run", f"--user-data-dir={profile}",
                                  "--virtual-time-budget=20000", "--dump-dom", url],
                                 capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120).stdout
    finally:
        server.terminate()
        server.wait(timeout=10)
    m = re.search(r'<pre id="out">(.*?)</pre>', dom, re.S)
    if not m or not m.group(1).strip():
        print("FAIL: smoke page produced no output")
        print(dom[:500])
        return 1
    reply = json.loads(html.unescape(m.group(1)))
    if not reply.get("ok"):
        print("FAIL: wasm returned an error", reply)
        return 1
    diffs = []
    compare(reply["data"], native, "data", diffs)
    print(f"abi {reply.get('abi')}, wasm against native differences {len(diffs)}")
    for d in diffs[:20]:
        print(d)
    if diffs:
        print("FAIL")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
