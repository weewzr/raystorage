"""Run a Python step script against the page of a running serve.py.

Usage: python drv.py OUTDIR SCRIPTS_DIR STEP_SCRIPT.py
The step script sees: ev(js), key(name, n=1, shift=False), text(s), shot(path, full=False),
pdf(path), nav(url, wait=2), vp(w, h, mobile=False), waitfor(js, ms), out(name, value), sleep(s), S (session).
"""
import base64
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, sys.argv[2])
from wasm_smoke import Session, WebSocket  # noqa: E402

sys.stdout.reconfigure(encoding="utf-8")
KEYS = {
    "Tab": (9, "Tab", ""), "Enter": (13, "Enter", "\r"), "Space": (32, " ", " "),
    "ArrowRight": (39, "ArrowRight", ""), "ArrowLeft": (37, "ArrowLeft", ""),
    "ArrowDown": (40, "ArrowDown", ""), "ArrowUp": (38, "ArrowUp", ""),
    "Escape": (27, "Escape", ""), "Home": (36, "Home", ""), "End": (35, "End", ""),
    "Backspace": (8, "Backspace", ""), "Delete": (46, "Delete", ""),
}
OUT = Path(sys.argv[1])
page = json.loads((OUT / "page.json").read_text(encoding="utf-8"))
S = Session(WebSocket(page["webSocketDebuggerUrl"]))
S.call("Page.enable")
S.call("Runtime.enable")
RES = {}


def ev(expr, timeout=60):
    r = S.call("Runtime.evaluate", {"expression": expr, "returnByValue": True, "awaitPromise": True,
                                    "userGesture": True}, timeout=timeout)
    if "exceptionDetails" in r:
        d = r["exceptionDetails"]
        return {"__error__": d.get("text"), "desc": d.get("exception", {}).get("description")}
    return r.get("result", {}).get("value")


def key(name, n=1, shift=False, gap=0.05):
    code, k, txt = KEYS[name]
    base = {"key": k, "code": "Space" if name == "Space" else k, "windowsVirtualKeyCode": code,
            "nativeVirtualKeyCode": code, "modifiers": 8 if shift else 0}
    for _ in range(n):
        if txt:
            S.call("Input.dispatchKeyEvent", {"type": "keyDown", "text": txt, "unmodifiedText": txt, **base})
        else:
            S.call("Input.dispatchKeyEvent", {"type": "rawKeyDown", **base})
        S.call("Input.dispatchKeyEvent", {"type": "keyUp", **base})
        time.sleep(gap)


def text(s):
    S.call("Input.insertText", {"text": s})


def shot(path, full=False):
    r = S.call("Page.captureScreenshot", {"format": "png", "captureBeyondViewport": bool(full)}, timeout=60)
    Path(path).write_bytes(base64.b64decode(r["data"]))


def pdf(path, **kw):
    params = {"printBackground": True, "paperWidth": 8.27, "paperHeight": 11.69, **kw}
    r = S.call("Page.printToPDF", params, timeout=120)
    Path(path).write_bytes(base64.b64decode(r["data"]))


def nav(url, wait=2.0):
    S.call("Page.navigate", {"url": url})
    time.sleep(wait)


def vp(w, h, mobile=False):
    S.call("Emulation.setDeviceMetricsOverride", {"width": w, "height": h, "deviceScaleFactor": 1,
                                                  "mobile": bool(mobile)})


def waitfor(js, ms=20000):
    t0 = time.time()
    while (time.time() - t0) * 1000 < ms:
        v = ev(js)
        if v and not (isinstance(v, dict) and "__error__" in v):
            return v
        time.sleep(0.1)
    return None


def out(name, value):
    RES[name] = value


def sleep(s):
    time.sleep(s)


exec(compile(Path(sys.argv[3]).read_text(encoding="utf-8"), sys.argv[3], "exec"))
print(json.dumps(RES, indent=1, ensure_ascii=False))
