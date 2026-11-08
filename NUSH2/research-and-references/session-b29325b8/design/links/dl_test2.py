"""Second pass. Everything stays in memory, nothing is saved. Run with python -I."""
import io
import json
import re
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
CTX = ssl.create_default_context()
try:
    import pypdf
    Reader = pypdf.PdfReader
    print("pypdf", pypdf.__version__)
except ImportError:
    Reader = None
    print("pypdf not available")


def req(u, rng=None, n=65536, timeout=40):
    h = {"User-Agent": UA, "Accept": "*/*"}
    if rng:
        h["Range"] = rng
    try:
        with urllib.request.urlopen(urllib.request.Request(u, headers=h), timeout=timeout, context=CTX) as f:
            return f.status, f.headers, f.read(n), f.geturl()
    except urllib.error.HTTPError as e:
        return e.code, e.headers, b"", u
    except Exception as e:  # noqa: BLE001
        return "ERR " + repr(e)[:100], {}, b"", u


def show(k, u, rng="bytes=0-2047"):
    s, h, b, f = req(u, rng=rng)
    g = h.get if h else (lambda *_: None)
    print(k, "|", s, g("Content-Type"), g("Content-Length"), g("Content-Range"), b[:5], f[:100], flush=True)


def first_page(k, u, cap=6000000, pat=None):
    s, h, b, f = req(u, n=cap, timeout=90)
    print(k, "|", s, len(b), flush=True)
    if s != 200 or not b.startswith(b"%PDF") or Reader is None:
        return
    r = Reader(io.BytesIO(b))
    print("   pages", len(r.pages), "meta title:", (r.metadata or {}).get("/Title"))
    for i in range(min(2, len(r.pages))):
        print("   p%d:" % (i + 1), re.sub(r"\s+", " ", r.pages[i].extract_text() or "")[:280])
    if pat:
        hits = 0
        for i, p in enumerate(r.pages):
            t = re.sub(r"\s+", " ", p.extract_text() or "")
            for m in re.finditer(pat, t):
                print("   hit p%d: %s" % (i + 1, t[max(0, m.start() - 70):m.end() + 70]))
                hits += 1
                if hits >= 6:
                    return


print("\n== identity of small INL PDFs and the X-energy deck ==")
first_page("INL 4886652", "https://inldigitallibrary.inl.gov/sites/sti/sti/4886652.pdf")
first_page("INL 4886648", "https://inldigitallibrary.inl.gov/sites/sti/sti/4886648.pdf")
first_page("INL 4633180", "https://inldigitallibrary.inl.gov/sites/sti/sti/4633180.pdf")
first_page("Xenergy", "https://x-energy.s3.amazonaws.com/x-energy-investor-presentation.pdf", pat=r"/\s?kWe?\b|per kW|2,900|9,200")

print("\n== Ishii PDF ==")
show("Ishii pdf", "https://www.jstage.jst.go.jp/article/jsmeicone/2023.30/0/2023.30_1074/_pdf/-char/en")
show("Ishii pdf2", "https://www.jstage.jst.go.jp/article/jsmeicone/2023.30/0/2023.30_1074/_pdf")

print("\n== Cho ScienceDirect ==")
show("Cho article", "https://www.sciencedirect.com/science/article/pii/S2352484722019874")
show("Cho pdf", "https://www.sciencedirect.com/science/article/pii/S2352484722019874/pdf")

print("\n== Porto bitstream ==")
s, h, b, f = req("https://repositorio-aberto.up.pt/handle/10216/103655", n=400000)
t = b.decode("utf-8", "replace")
links = sorted(set(re.findall(r'/bitstream/10216/103655/[^"\s]+', t)))
print(s, links[:6])
meta = re.findall(r'<meta name="(DC\.(?:creator|title|type|date)|DCTERMS\.issued|citation_title|citation_author|citation_date)" content="([^"]{1,120})"', t)
print(meta[:12])
for l in links[:2]:
    show("Porto file", "https://repositorio-aberto.up.pt" + l)

print("\n== full titles ==")
for d in ("10.1021/acs.iecr.5c03283",):
    s, h, b, _ = req("https://api.crossref.org/works/" + urllib.parse.quote(d, safe=""), n=400000)
    print(d, s, re.sub(r"<[^>]+>|\s+", " ", (json.loads(b)["message"].get("title") or [""])[0]) if s == 200 else "")

print("\n== wayback, 45 s spacing ==")
BASE = "art.inl.gov/NGNP/"
WB = {
    "TEV-962 wb": "https://web.archive.org/web/20161228034305id_/https://" + BASE + "NEAC%202010/INL_NGNP%20References/TEV-962%20Sesitivity%20of%20Hydrogen%20Prod.%20via%20SMR%20to%20HTGR%20ROT.pdf",
    "TEV-981 wb": "https://web.archive.org/web/20161228035850id_/https://" + BASE + "NEAC%202010/INL_NGNP%20References/TEV-981%20An%20Analysis%20of%20the%20Effect%20of%20Reactor%20Outlet%20Temp.%20of%20a%20High.pdf",
    "TEV-961 wb": "https://web.archive.org/web/20170130205123id_/https://" + BASE + "INL%20Documents/Year%202010/Sensitivity%20of%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20to%20High%20Temp%20Gas-Cooled%20Reactor%20Outlet%20Temp%20Process%20Analysis.pdf",
}
time.sleep(45)
for k, u in WB.items():
    s, h, b, f = req(u, rng="bytes=0-65535", timeout=60)
    g = h.get if h else (lambda *_: None)
    m = re.search(rb"/Title\s*\(([^)]{3,140})\)", b)
    print(k, "|", s, g("Content-Type"), g("Content-Range"), b[:5], m.group(1) if m else None, flush=True)
    time.sleep(45)

cdx = ("https://web.archive.org/cdx/search/cdx?url=art.inl.gov/NGNP/INL%2520Documents/Year%25202010/&matchType=prefix"
       "&collapse=urlkey&limit=30&fl=timestamp,statuscode,mimetype,original&filter=mimetype:.*(excel|spreadsheet|zip).*")
s, h, b, f = req(cdx, n=6000, timeout=70)
print("cdx", s, b[:2500].decode("utf-8", "replace"), flush=True)
