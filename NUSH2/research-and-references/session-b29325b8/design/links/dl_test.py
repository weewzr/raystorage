"""Link test for the download list. Ranged GET in memory, saves nothing. Run with python -I."""
import json
import re
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
CTX = ssl.create_default_context()
ART = "https://art.inl.gov/NGNP/"

DIRECT = {
    "TEV-954 art": ART + "INL%20Documents/Year%202010/HTGR-Integrated%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20-SMR-%20Economic%20Analysis.pdf",
    "TEV-962 art NEAC": ART + "NEAC%202010/INL_NGNP%20References/TEV-962%20Sesitivity%20of%20Hydrogen%20Prod.%20via%20SMR%20to%20HTGR%20ROT.pdf",
    "TEV-981 art NEAC": ART + "NEAC%202010/INL_NGNP%20References/TEV-981%20An%20Analysis%20of%20the%20Effect%20of%20Reactor%20Outlet%20Temp.%20of%20a%20High.pdf",
    "TEV-961 art": ART + "INL%20Documents/Year%202010/Sensitivity%20of%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20to%20High%20Temp%20Gas-Cooled%20Reactor%20Outlet%20Temp%20Process%20Analysis.pdf",
    "INL 4886652": "https://inldigitallibrary.inl.gov/sites/sti/sti/4886652.pdf",
    "INL 4886648": "https://inldigitallibrary.inl.gov/sites/sti/sti/4886648.pdf",
    "IAEA 1682 pdf": "https://www-pub.iaea.org/MTCD/Publications/PDF/TE_1682_web.pdf",
    "ETC": "https://www.energy-transitions.org/wp-content/uploads/2022/07/ETC-Bioresources-Report-Final.pdf",
    "Xenergy": "https://x-energy.s3.amazonaws.com/x-energy-investor-presentation.pdf",
    "Ishii jstage": "https://www.jstage.jst.go.jp/article/jsmeicone/2023.30/0/2023.30_1074/_article/-char/en",
    "Ishii doi": "https://doi.org/10.1299/jsmeicone.2023.30.1074",
    "Cho doi": "https://doi.org/10.1016/j.egyr.2022.10.053",
    "OSTI 814028": "https://www.osti.gov/servlets/purl/814028",
    "tika 404174": "https://corpora.tika.apache.org/base/docs/govdocs1/404/404174.pdf",
    "INL 4633180": "https://inldigitallibrary.inl.gov/sites/sti/sti/4633180.pdf",
    "GA SI summary": "https://art.inl.gov/NGNP/Subcontractors%20Documents/General%20Atomics/Sulfur%20Iodine%20Process%20Summary%20for%20the%20Hydrogen%20Technology%20Down-Selection.pdf",
    "Callaghan thesis": "https://my.che.utah.edu/~ring/Design%20II/Articles/Water%20Gas%20Shift%20Kinetics-c%20callaghan%20thesis.pdf",
    "Porto repo": "https://repositorio-aberto.up.pt/handle/10216/103655",
    "Vandenbussche": "https://terpconnect.umd.edu/~nsw/chbe446/VandenBussche-syngas2MeOH.pdf",
    "WVU etd": "https://researchrepository.wvu.edu/cgi/viewcontent.cgi?article=7726&context=etd",
}

WAYBACK = {
    "TEV-962 wb": "https://web.archive.org/web/20161228034305id_/https://art.inl.gov/NGNP/NEAC%202010/INL_NGNP%20References/TEV-962%20Sesitivity%20of%20Hydrogen%20Prod.%20via%20SMR%20to%20HTGR%20ROT.pdf",
    "TEV-981 wb": "https://web.archive.org/web/20161228035850id_/https://art.inl.gov/NGNP/NEAC%202010/INL_NGNP%20References/TEV-981%20An%20Analysis%20of%20the%20Effect%20of%20Reactor%20Outlet%20Temp.%20of%20a%20High.pdf",
    "TEV-954 wb": "https://web.archive.org/web/20161228040212id_/https://art.inl.gov/NGNP/INL%20Documents/Year%202010/HTGR-Integrated%20Hydrogen%20Production%20via%20Steam%20Methane%20Reforming%20-SMR-%20Economic%20Analysis.pdf",
}

DOIS = [
    "10.1299/jsmeicone.2023.30.1074",
    "10.1016/j.egyr.2022.10.053",
    "10.1002/cjce.5450760512",
    "10.1002/aic.690420130",
    "10.1021/je060031v",
    "10.1021/acs.iecr.5c03283",
    "10.1021/je00051a011",
    "10.1021/ie9004578",
    "10.1021/ie1006855",
    "10.1021/acs.iecr.8b01471",
    "10.1016/j.supflu.2010.09.039",
    "10.1016/S1385-8947(00)00367-3",
    "10.1021/je960314v",
    "10.1016/S0378-7753(03)00614-1",
    "10.1016/0360-3199(94)00130-R",
    "10.1006/jcat.1996.0016",
    "10.1016/S0021-9517(03)00050-2",
    "10.1021/ie049886w",
    "10.1021/ie101137b",
    "10.1016/S0166-9834(00)83024-9",
]

OSTI = [
    "https://www.osti.gov/api/v1/records?q=%22GA-A24265%22&rows=5",
    "https://www.osti.gov/api/v1/records?report_number=GA-A24265&rows=5",
    "https://www.osti.gov/api/v1/records/814028",
]

CDX = ("https://web.archive.org/cdx/search/cdx?url=art.inl.gov/NGNP/*&collapse=urlkey&limit=40"
       "&fl=timestamp,statuscode,original&filter=original:.*(xls|Stream%20Results|Appendix%20G).*")


def req(u, rng=None, method="GET", n=65536, timeout=30):
    h = {"User-Agent": UA, "Accept": "*/*"}
    if rng:
        h["Range"] = rng
    r = urllib.request.Request(u, headers=h, method=method)
    try:
        with urllib.request.urlopen(r, timeout=timeout, context=CTX) as f:
            b = f.read(n) if method == "GET" else b""
            return f.status, f.headers, b, f.geturl()
    except urllib.error.HTTPError as e:
        return e.code, e.headers, b"", u
    except Exception as e:  # noqa: BLE001
        return "ERR " + repr(e)[:110], {}, b"", u


def title_of(b):
    for pat in (rb"<dc:title>.*?<rdf:li[^>]*>(.*?)</rdf:li>", rb"/Title\s*\(([^)]{3,140})\)", rb"<title[^>]*>(.{3,140}?)</title>"):
        m = re.search(pat, b, re.S | re.I)
        if m:
            return m.group(1).decode("latin-1", "replace").strip()[:130]
    return None


def probe(item):
    k, u = item
    s, h, b, final = req(u, rng="bytes=0-65535")
    g = h.get if h else (lambda *_: None)
    return (k, s, g("Content-Type"), g("Content-Length"), g("Content-Range"), b[:5], title_of(b), final[:110])


print("== direct ==")
with ThreadPoolExecutor(8) as ex:
    for row in ex.map(probe, DIRECT.items()):
        print(row, flush=True)

print("\n== crossref + openalex ==")
for d in DOIS:
    s, h, b, _ = req("https://api.crossref.org/works/" + urllib.parse.quote(d, safe=""), n=400000)
    t = yr = c = None
    if s == 200:
        m = json.loads(b)["message"]
        t = (m.get("title") or [None])[0]
        c = (m.get("container-title") or [None])[0]
        yr = (m.get("issued", {}).get("date-parts") or [[None]])[0][0]
    s2, h2, b2, _ = req("https://api.openalex.org/works/doi:" + d, n=2000000)
    oa = None
    if s2 == 200:
        w = json.loads(b2)
        bl = w.get("best_oa_location") or {}
        oa = (w.get("open_access", {}).get("is_oa"), w.get("open_access", {}).get("oa_status"), bl.get("pdf_url"), bl.get("landing_page_url"))
    print(d, "|", s, yr, c, "|", (t or "")[:110], "|", s2, oa, flush=True)
    time.sleep(0.3)

print("\n== osti ==")
for u in OSTI:
    s, h, b, _ = req(u, n=300000)
    out = []
    if s == 200:
        try:
            j = json.loads(b)
            for r in (j if isinstance(j, list) else [j])[:5]:
                out.append((r.get("osti_id"), r.get("report_number"), (r.get("title") or "")[:100], r.get("publication_date", "")[:10]))
        except ValueError as e:
            out = [repr(e)[:80]]
    print(u[:80], s, out, flush=True)

print("\n== wayback (spaced) ==")
for k, u in WAYBACK.items():
    row = probe((k, u))
    print(row, flush=True)
    time.sleep(20)
s, h, b, _ = req(CDX, n=8000, timeout=60)
print("cdx", s, b[:3000].decode("utf-8", "replace"), flush=True)
