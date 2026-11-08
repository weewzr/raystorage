"""Slow ranged GET, spaced out to respect the Wayback rate limit. Saves nothing. Run with python -I."""
import sys
import ssl
import time
import urllib.request
import urllib.error

UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
GAP = float(sys.argv[2]) if len(sys.argv) > 2 else 30.0


def get(u):
    req = urllib.request.Request(u, headers={"User-Agent": UA, "Range": "bytes=0-1023", "Accept": "*/*"})
    try:
        with urllib.request.urlopen(req, timeout=60, context=ssl.create_default_context()) as r:
            b = r.read(1024)
            return r.status, r.headers.get("Content-Type"), r.headers.get("Content-Range"), r.headers.get("Content-Length"), b[:8], r.headers.get("X-Archive-Orig-Last-Modified")
    except urllib.error.HTTPError as e:
        return e.code, e.headers.get("Content-Type"), e.headers.get("Retry-After"), None, b"", None
    except Exception as e:  # noqa: BLE001
        return "ERR", repr(e)[:160], None, None, b"", None


for line in open(sys.argv[1], encoding="utf-8"):
    u = line.strip()
    if not u or u.startswith("#"):
        continue
    res = get(u)
    tries = 0
    while res[0] == 429 and tries < 3:
        time.sleep(GAP * 2)
        res = get(u)
        tries += 1
    print(u[:150])
    print("   ->", res, flush=True)
    time.sleep(GAP)
