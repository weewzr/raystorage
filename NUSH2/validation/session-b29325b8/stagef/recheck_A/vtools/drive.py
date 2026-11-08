"""Drive the page of a running serve.py over the DevTools protocol.

Usage: python drive.py OUTDIR SCRIPTS_DIR STEPS.json
STEPS is a list of steps run in one session:
  {"vp": [w, h, mobile]}         set the viewport for this session
  {"nav": url}                   navigate and wait for load
  {"eval": "js", "name": key}    evaluate (await promise), store result under name
  {"evalfile": path, "name": k}  same, from a file
  {"key": "Tab", "n": 3, "shift": false}  real key presses
  {"type": "text"}               insert text
  {"shot": path, "full": false}  screenshot
  {"pdf": path}                  Page.printToPDF
  {"sleep": s}
Prints a JSON object of the stored results.
"""
import base64
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, sys.argv[2])
from wasm_smoke import Session, WebSocket  # noqa: E402

KEYS = {
    "Tab": (9, "Tab", "\t"), "Enter": (13, "Enter", "\r"), "Space": (32, " ", " "),
    "ArrowRight": (39, "ArrowRight", ""), "ArrowLeft": (37, "ArrowLeft", ""),
    "ArrowDown": (40, "ArrowDown", ""), "ArrowUp": (38, "ArrowUp", ""),
    "Escape": (27, "Escape", ""), "Home": (36, "Home", ""), "End": (35, "End", ""),
    "Backspace": (8, "Backspace", ""),
}


def key(s, name, shift=False):
    code, k, text = KEYS[name]
    mods = 8 if shift else 0
    base = {"key": k, "code": k if name != "Space" else "Space", "windowsVirtualKeyCode": code,
            "nativeVirtualKeyCode": code, "modifiers": mods}
    s.call("Input.dispatchKeyEvent", {"type": "rawKeyDown" if not text or name == "Tab" else "keyDown", **base,
                                      **({"text": text} if text and name not in ("Tab",) else {})})
    if text and name not in ("Tab", "Enter") and False:
        pass
    s.call("Input.dispatchKeyEvent", {"type": "keyUp", **base})


def evaluate(s, expr, timeout=60):
    r = s.call("Runtime.evaluate", {"expression": expr, "returnByValue": True, "awaitPromise": True,
                                    "userGesture": True}, timeout=timeout)
    if "exceptionDetails" in r:
        d = r["exceptionDetails"]
        return {"__error__": d.get("text"), "desc": d.get("exception", {}).get("description")}
    return r.get("result", {}).get("value")


def main():
    out = Path(sys.argv[1])
    page = json.loads((out / "page.json").read_text(encoding="utf-8"))
    steps = json.loads(Path(sys.argv[3]).read_text(encoding="utf-8"))
    s = Session(WebSocket(page["webSocketDebuggerUrl"]))
    s.call("Page.enable")
    s.call("Runtime.enable")
    res = {}
    for st in steps:
        if "vp" in st:
            w, h, mob = st["vp"]
            s.call("Emulation.setDeviceMetricsOverride", {"width": w, "height": h, "deviceScaleFactor": 1,
                                                          "mobile": bool(mob)})
        elif "nav" in st:
            s.call("Page.navigate", {"url": st["nav"]})
            time.sleep(st.get("wait", 2.0))
        elif "eval" in st or "evalfile" in st:
            expr = st.get("eval") or Path(st["evalfile"]).read_text(encoding="utf-8")
            res[st.get("name", f"e{len(res)}")] = evaluate(s, expr, st.get("timeout", 60))
        elif "key" in st:
            for _ in range(st.get("n", 1)):
                key(s, st["key"], st.get("shift", False))
                time.sleep(st.get("gap", 0.05))
        elif "type" in st:
            s.call("Input.insertText", {"text": st["type"]})
        elif "shot" in st:
            r = s.call("Page.captureScreenshot", {"format": "png", "captureBeyondViewport": bool(st.get("full"))},
                       timeout=60)
            Path(st["shot"]).write_bytes(base64.b64decode(r["data"]))
        elif "pdf" in st:
            r = s.call("Page.printToPDF", {"printBackground": True, "paperWidth": 8.27, "paperHeight": 11.69,
                                           "preferCSSPageSize": True}, timeout=120)
            Path(st["pdf"]).write_bytes(base64.b64decode(r["data"]))
        elif "sleep" in st:
            time.sleep(st["sleep"])
    sys.stdout.reconfigure(encoding="utf-8")
    print(json.dumps(res, indent=1, ensure_ascii=False))


if __name__ == "__main__":
    main()
