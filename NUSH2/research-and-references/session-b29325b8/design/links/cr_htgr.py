"""Crossref lookup. Usage: python -I cr_htgr.py "bibliographic query" ...   or  doi:10.xxx/yyy"""
import sys, json, urllib.request, urllib.parse

UA = "NUS-H2-link-check/1.0"


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def fmt(i):
    t = (i.get("title") or [""])[0]
    a = "; ".join((x.get("family") or x.get("name") or "") for x in (i.get("author") or [])[:6])
    c = (i.get("container-title") or [""])[0]
    y = (i.get("issued", {}).get("date-parts") or [[None]])[0][0]
    return f"{y} | {a} | {t[:130]} | {c[:50]} {i.get('volume','')} {i.get('page','')} | {i.get('DOI')} | lic={len(i.get('license',[]) or [])}"


for q in sys.argv[1:]:
    print("##", q)
    try:
        if q.startswith("doi:"):
            print("  ", fmt(get("https://api.crossref.org/works/" + urllib.parse.quote(q[4:]))["message"]))
        else:
            r = get("https://api.crossref.org/works?rows=4&query.bibliographic=" + urllib.parse.quote(q))
            for i in r["message"]["items"]:
                print("  ", fmt(i))
    except Exception as e:
        print("   ERR", type(e).__name__, str(e)[:120])
