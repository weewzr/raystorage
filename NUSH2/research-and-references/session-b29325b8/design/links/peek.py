"""Read a remote PDF in memory and print lines that match a regex. Writes nothing to disk.
Usage: python -I peek.py "<regex>" url [url ...]
"""
import sys, re, ssl, urllib.request

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import pymupdf as fitz

UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
CTX = ssl._create_unverified_context()
CAP = 40_000_000
MAXHITS = int(__import__("os").environ.get("MAXHITS", "40"))


def fetch(u):
    req = urllib.request.Request(u.replace(" ", "%20"), headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=120, context=CTX) as r:
        b = r.read(CAP + 1)
        return b, r.status, r.headers.get("Content-Type", "")


pat = re.compile(sys.argv[1], re.I)
for u in sys.argv[2:]:
    print("=====", u)
    try:
        b, st, ct = fetch(u)
    except Exception as e:
        print("  ERR", type(e).__name__, str(e)[:120])
        continue
    print("  status", st, ct, len(b), "bytes")
    if b[:5] != b"%PDF-":
        print("  not a PDF")
        continue
    d = fitz.open(stream=b, filetype="pdf")
    print("  pages", len(d), "| title:", (d.metadata or {}).get("title"))
    n = 0
    empty = 0
    for i, p in enumerate(d):
        t = p.get_text()
        if len(t.strip()) < 20:
            empty += 1
        lines = t.split("\n")
        for j, line in enumerate(lines):
            if pat.search(line):
                ctx = " ".join(x.strip() for x in lines[max(0, j - 1) : j + 2])
                print(f"  p{i+1}: {ctx[:230]}")
                n += 1
                if n >= MAXHITS:
                    break
        if n >= MAXHITS:
            break
    print("  hits", n, "| near-empty pages", empty)
