"""Link tester. Sends a ranged GET for the first 4 KB. Saves nothing. Usage: python -I lt_htgr.py url [url ...]"""
import sys, ssl, urllib.request, urllib.error, concurrent.futures as cf

UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
CTX = ssl.create_default_context()
CTX_NV = ssl._create_unverified_context()


def test(url):
    u = url.replace(" ", "%20")
    for ctx, tag in ((CTX, ""), (CTX_NV, " [no cert check]")):
        try:
            req = urllib.request.Request(u, headers={"User-Agent": UA, "Range": "bytes=0-4095", "Accept": "*/*"})
            with urllib.request.urlopen(req, timeout=30, context=ctx) as r:
                b = r.read(4096)
                ct = r.headers.get("Content-Type", "")
                cl = r.headers.get("Content-Range") or r.headers.get("Content-Length") or ""
                sig = "PDF" if b[:5] == b"%PDF-" else ""
                return f"{r.status} ct={ct} len={cl} final={r.geturl()[:120]} {sig}{tag}"
        except urllib.error.HTTPError as e:
            return f"HTTP {e.code} ct={e.headers.get('Content-Type','')} {e.reason}{tag}"
        except ssl.SSLError as e:
            continue
        except Exception as e:
            if "CERTIFICATE" in str(e).upper():
                continue
            return f"ERR {type(e).__name__}: {str(e)[:100]}{tag}"
    return "ERR ssl"


urls = sys.argv[1:]
with cf.ThreadPoolExecutor(8) as ex:
    res = list(ex.map(test, urls))
for u, r in zip(urls, res):
    print(u, "\n   ->", r)
