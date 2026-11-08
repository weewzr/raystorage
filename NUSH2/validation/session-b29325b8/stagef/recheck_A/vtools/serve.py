"""Serve a built site on 127.0.0.1 and start headless Chrome with a fixed
debugging port. A monitor client logs every Network, Runtime and Log event of
the page to events.jsonl. Runs until killed.

Usage: python serve.py SITE PORT DPORT OUTDIR
"""
import json
import subprocess
import sys
import threading
import time
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, sys.argv[5] if len(sys.argv) > 5 else "")
from wasm_smoke import Session, WebSocket, get_json  # noqa: E402


class Quiet(SimpleHTTPRequestHandler):
    extensions_map = {**SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm",
                      ".js": "text/javascript", ".json": "application/json",
                      ".svg": "image/svg+xml", ".csv": "text/csv"}

    def log_message(self, *a):
        pass


def main():
    site, port, dport, out = Path(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]), Path(sys.argv[4])
    out.mkdir(parents=True, exist_ok=True)
    server = ThreadingHTTPServer(("127.0.0.1", port), partial(Quiet, directory=str(site)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    chrome = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
    cmd = [chrome, "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
           "--disable-extensions", "--disable-background-networking", "--disable-component-update",
           "--disable-sync", f"--user-data-dir={out / 'profile'}", f"--remote-debugging-port={dport}",
           "about:blank"]
    browser = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=open(out / "chrome.log", "wb"))
    targets = get_json(f"http://127.0.0.1:{dport}/json/list")
    page = next(t for t in targets if t.get("type") == "page")
    (out / "page.json").write_text(json.dumps(page), encoding="utf-8")
    s = Session(WebSocket(page["webSocketDebuggerUrl"]))
    for dom in ("Runtime.enable", "Log.enable", "Network.enable", "Page.enable"):
        s.call(dom)
    print("ready", flush=True)
    with open(out / "events.jsonl", "a", encoding="utf-8") as f:
        while browser.poll() is None:
            try:
                msg = s.ws.recv(1.0)
            except Exception:
                time.sleep(0.2)
                continue
            if "method" in msg:
                m = msg["method"]
                if m.startswith(("Network.requestWillBeSent", "Network.responseReceived", "Network.loadingFailed",
                                 "Runtime.consoleAPICalled", "Runtime.exceptionThrown", "Log.entryAdded",
                                 "Page.frameNavigated")):
                    f.write(json.dumps({"t": time.time(), **msg}) + "\n")
                    f.flush()


if __name__ == "__main__":
    main()
