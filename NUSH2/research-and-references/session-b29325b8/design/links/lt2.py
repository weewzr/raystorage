"""Link tester, two User-Agents. Reads URLs from argv.
Ranged GET of first 2048 bytes with the default library User-Agent, then with a
browser-style User-Agent to predict what a person sees. Saves nothing."""
import sys
import ssl
import urllib.request
import urllib.error
import urllib.parse

sys.stdout.reconfigure(encoding="utf-8")
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
CTX = ssl.create_default_context()


def go(url, ua):
    h = {"Range": "bytes=0-2047", "Accept": "*/*"}
    if ua:
        h["User-Agent"] = ua
    req = urllib.request.Request(url, headers=h)
    try:
        with urllib.request.urlopen(req, timeout=25, context=CTX) as r:
            return r.status, r.headers.get("Content-Type"), r.headers.get("Content-Range"), r.geturl()
    except urllib.error.HTTPError as e:
        return e.code, e.headers.get("Content-Type"), None, url
    except Exception as e:  # noqa: BLE001
        return "ERR", type(e).__name__ + " " + str(e)[:80], None, url


for raw in sys.argv[1:]:
    u = urllib.parse.quote(raw, safe=":/?&=%+#-_.~()")
    a = go(u, None)
    b = go(u, UA)
    print(raw)
    print("   plain  ", a[0], a[1], a[2] or "")
    print("   browser", b[0], b[1], b[2] or "")
    if b[3] != u:
        print("   final  ", b[3])
