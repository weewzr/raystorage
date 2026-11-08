"""Link tester. Usage: python -I lt.py url1 url2 ...
HEAD first, then ranged GET of the first 2048 bytes. Saves nothing to disk."""
import sys
import urllib.request
import urllib.error
import ssl

sys.stdout.reconfigure(encoding="utf-8")
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
CTX = ssl.create_default_context()


def one(url, method, headers):
    req = urllib.request.Request(url, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=25, context=CTX) as r:
            first = r.read(8) if method == "GET" else b""
            return r.status, r.headers.get("Content-Type"), r.headers.get("Content-Length"), r.headers.get("Content-Range"), r.geturl(), first
    except urllib.error.HTTPError as e:
        return e.code, e.headers.get("Content-Type"), e.headers.get("Content-Length"), None, url, b""
    except Exception as e:  # noqa: BLE001
        return "ERR", type(e).__name__ + ": " + str(e)[:100], None, None, url, b""


for u in sys.argv[1:]:
    base = {"User-Agent": UA, "Accept": "*/*"}
    h = one(u, "HEAD", base)
    g = one(u, "GET", dict(base, Range="bytes=0-2047"))
    print("URL ", u)
    print("  HEAD", h[0], h[1], "len", h[2])
    print("  GET ", g[0], g[1], "len", g[2], "range", g[3], "magic", g[5])
    if g[4] != u:
        print("  FINAL", g[4])
