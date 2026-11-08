"""Ranged GET of the first bytes of each URL. Saves nothing. Run with python -I."""
import sys
import ssl
import urllib.request
import urllib.parse
import urllib.error

UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"


def quote_url(u):
    p = urllib.parse.urlsplit(u)
    path = urllib.parse.quote(urllib.parse.unquote(p.path), safe="/()-_.~")
    return urllib.parse.urlunsplit((p.scheme, p.netloc, path, p.query, p.fragment))


def test(u):
    q = quote_url(u)
    ctx = ssl.create_default_context()
    req = urllib.request.Request(q, headers={"User-Agent": UA, "Range": "bytes=0-1023", "Accept": "*/*"})
    try:
        with urllib.request.urlopen(req, timeout=30, context=ctx) as r:
            b = r.read(1024)
            return (r.status, r.headers.get("Content-Type"), r.headers.get("Content-Range") or r.headers.get("Content-Length"), b[:8], r.geturl())
    except urllib.error.HTTPError as e:
        return (e.code, e.headers.get("Content-Type"), None, b"", q)
    except Exception as e:  # noqa: BLE001
        return ("ERR", repr(e)[:160], None, b"", q)


if __name__ == "__main__":
    for line in open(sys.argv[1], encoding="utf-8"):
        u = line.strip()
        if not u or u.startswith("#"):
            continue
        print(u)
        print("   ->", test(u))
