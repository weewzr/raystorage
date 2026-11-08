"""Attach to the page with auto-attach to its workers (flat sessions) and log
worker network and console events across a fresh load and a few solves."""
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, sys.argv[2])
from wasm_smoke import WebSocket  # noqa: E402

OUT = Path(sys.argv[1])
page = json.loads((OUT / "page.json").read_text(encoding="utf-8"))
ws = WebSocket(page["webSocketDebuggerUrl"])
nid = [0]
events = []


def send(method, params=None, session=None):
    nid[0] += 1
    msg = {"id": nid[0], "method": method, "params": params or {}}
    if session:
        msg["sessionId"] = session
    ws.send(msg)
    return nid[0]


def pump(seconds):
    end = time.time() + seconds
    while time.time() < end:
        try:
            m = ws.recv(0.5)
        except Exception:
            continue
        if m.get("method") == "Target.attachedToTarget":
            sid = m["params"]["sessionId"]
            info = m["params"]["targetInfo"]
            events.append({"attached": info.get("type"), "url": info.get("url")})
            for d in ("Network.enable", "Runtime.enable", "Log.enable"):
                send(d, session=sid)
            send("Runtime.runIfWaitingForDebugger", session=sid)
        elif "method" in m and m.get("sessionId"):
            events.append({"s": m["sessionId"][:8], "method": m["method"], "params": m.get("params", {})})


send("Target.setAutoAttach", {"autoAttach": True, "waitForDebuggerOnStart": True, "flatten": True})
send("Page.enable")
pump(0.5)
send("Page.navigate", {"url": "http://127.0.0.1:18765/index.html?w=1#results"})
pump(8)
send("Runtime.evaluate", {"expression": "(async()=>{const m=await import('./app.js?v=54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16');m.setInputs({basis:'case_a'});await new Promise(r=>setTimeout(r,1500));m.resetInputs();return 1})()", "awaitPromise": True})
pump(4)
summary = {"attached": [e for e in events if "attached" in e], "requests": [], "fails": [], "console": []}
for e in events:
    m = e.get("method")
    p = e.get("params", {})
    if m == "Network.requestWillBeSent":
        summary["requests"].append(p["request"]["url"].split("?")[0])
    elif m == "Network.loadingFailed":
        summary["fails"].append(p)
    elif m in ("Runtime.consoleAPICalled", "Runtime.exceptionThrown", "Log.entryAdded"):
        summary["console"].append(json.dumps(p)[:300])
    elif m == "Network.responseReceived" and p["response"]["status"] >= 400:
        summary["fails"].append(p["response"]["url"])
sys.stdout.reconfigure(encoding="utf-8")
print(json.dumps(summary, indent=1))
