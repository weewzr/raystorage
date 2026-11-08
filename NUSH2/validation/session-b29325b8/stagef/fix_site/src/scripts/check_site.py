#!/usr/bin/env python3
"""Content guard for the built site and the repository. Python stdlib only.

Site scope (every file under --site):
  text rules     personal identifiers, cohort and schedule wording, course
                 codes and academic framing, local paths, the em dash and
                 email addresses
  withdrawn      the deck ledger figures and the superseded headline values
  pointer        pointers into working notes that no public file defines
  files          raster images, PDFs and web fonts
  markup         <image> and raster <img> tags, raster data URIs, @font-face
  hosts          any http(s) URL outside the allowlist, protocol-relative URLs
  absolute       resource references that start at the server root
  footer         the whole footer reads the approved line and nothing else

Repository scope (tracked files, and untracked files that are not ignored,
leaving out LICENSE and Cargo.lock):
  text rules     as above, on text files
  local only     any path under report/ or presentation/ other than the four
                 published deliverables

The patterns are base64 encoded, so this file carries none of the strings it
looks for and passes its own scan and the pre-push scan. Hits are reported by
rule, file, line and column, never by the matched text.

--prove copies the site to a temporary folder, plants one sample per rule
there, and fails unless every planted sample is caught and the unplanted copy
is clean.

Usage:
  python scripts/check_site.py --site _site --repo .
  python scripts/check_site.py --site _site --repo . --prove
Exit 0 when clean (and, with --prove, when every plant is caught), 1 otherwise.
"""
import argparse
import base64
import re
import shutil
import subprocess
import sys
import tempfile
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]


def _d(s):
    return base64.b64decode(s).decode("utf-8")


# label: (pattern, flags, planted sample), all base64.
TEXT_RULES = {
    "cohort": ("XGJncm91cFtcc18tXSooPzo3fHNldmVuKVxi", "i", "R3JvdXAgNyBwcmVzZW50cw=="),
    "schedule": ("XGJ0aW1lW1xzLV0/c2xvdHM/XGJ8XGJwcmVzZW50YXRpb24gc2xvdFxifFxiZGVhZGxpbmVzP1xifFxiZHVlIGRhdGVcYg==",
                 "i", "dGhlIGRlYWRsaW5lIGlzIG5lYXI="),
    "allocation": ("XGJzcGVha2VyIGFsbG9jYXRpb25cYnxcYnNwZWFraW5nICg/OnNwbGl0fHRpbWV8b3JkZXIpXGI=", "i",
                   "c3BlYWtlciBhbGxvY2F0aW9uOiBvbmU="),
    "matric": ("XGJBXGR7N31bQS1aXVxi", "i", "QTAwMDAwMDBY"),
    "declaration": ("XGJBSVstIF11c2VcYnxcYkFJIGRlY2xhcmF0aW9uXGI=", "i", "QUkgZGVjbGFyYXRpb24gZm9sbG93cw=="),
    "course": ("KD88IVtcdy1dKSg/OkNOfEVFfENTfE1BfE1FfENFfENNfEJOfENHfEdFfEVTfElFfFBDfFNUfEVHfExTTXxHRUF8R0VDfEdFTnxHRVNTKVxkezR9W0EtWl17MCwyfSg/IVtcdy1dKQ==",
               "", "Q04xMjM0IHByb2plY3Q="),
    "academic": ("XGJBWVxzP1xkezR9KD86XHM/L1xzP1xkezIsNH0pP1xifFxiYWNhZGVtaWMgeWVhclxifFxic2VtZXN0ZXJcYnxcYmh5ZHJvZ2VuIGFuZCBsb3dbXHMtXWNhcmJvbiB0ZWNobm9sb2dpZXNcYg==",
                 "i", "QVkyMDMwLzIwMzEgU2VtZXN0ZXIgMQ=="),
    "drive_path": ("KD88IVtBLVphLXpdKVtBLVpdOltcXC9d", "", "RDovd29yay9maWxl"),
    "home_path": ("L1VzZXJzL3xcXFVzZXJzXFx8L2hvbWUvW2Etel18QXBwRGF0YXxzY3JhdGNocGFk", "i",
                  "L2hvbWUvc29tZW9uZS9maWxl"),
    "owner": ("XGJBbGZcYg==", "", "QWxm"),
}
# Rules for binary files (the wasm module): random bytes match a bare drive
# letter too often, so binaries get the anchored local-path rules only.
BINARY_RULES = ("matric", "home_path", "owner")
DRIVE_ROOT = "KD9pKSg/PCFbQS1aYS16XSlbQS1aXTpbXFwvXSg/OlVzZXJzfFkzUzF8UmF5IENsYXVkZXxQcm9ncmFtIEZpbGVzfFdpbmRvd3Mp"
# label: (pattern, planted sample). Site scope only, since docs/ may record the
# change once.
WITHDRAWN = {
    "deck_cost": ("U1wkXHM/OTZcYg==", "UyQ5Ng=="),
    "deck_sum_25": ("XGIxMDZcLjVcYig/IVxzKiUp", "MTA2LjU="),
    "deck_sum_60": ("XGI5OVwuNFxiKD8hXHMqJSk=", "OTkuNA=="),
    "deck_term": ("XCtccyo3XC4yXGI=", "KyA3LjI="),
    "deck_us_cost": ("VVNcJFxzPzgwXHMqL1xzKnQ=", "VVMkODAvdA=="),
    "deck_capex": ("U1wkXHM/NTQ2XHMqTQ==", "UyQ1NDZN"),
    "deck_gas": ("XGI0M1wuNVxzKk1NU0NGRA==", "NDMuNSBNTVNDRkQ="),
    "deck_abatement": ("XGIwXC41ODVccypNdA==", "MC41ODUgTXQveQ=="),
    "deck_h2": ("XGI2Niw5MjdcYg==", "NjYsOTI3"),
    "deck_claim": ("XGJBY2hpZXZlZFxi", "QWNoaWV2ZWQ="),
    "q8_units": ("U1wkXHM/KD86MTExXC4xfDk5XC4wfDExMlwuNHwxMDBcLjIpXGJ8XChccyoxMDBccyotXHMqNTFcLjlccypcKQ==",
                 "UyQxMTIuNC90"),
}
# Site scope only: the engine sources may still name working notes in
# comments, which the site never shows.
POINTER = ("XGIoPzphbWluZV9zcGVjfG1lYV9jb2x1bW5zfG1lYV90aGVybW98ZWNvbl9saXR8aHRncl9paHhcLm1kfGlubFwubWR8aW5sX2NhbGNzX291dHB1dHxmaW5hbF92MV83fGF1ZGl0X3JlY29tcHV0ZXxVU0VSX0FOU1dFUlN8c2NyZWVuaW5nXC5weXxmcmVlemUgc2VjdGlvbnxwbGFuIHNlY3Rpb258UmVzdCByb3d8b2xkIHJlcG8oPzpzaXRvcnkpPylcYg==",
           "c2VlIGFtaW5lX3NwZWMgbGluZSAzNA==")
EMDASH = chr(0x2014)
EMAIL = re.compile(r"[\w.+-]+@[\w-]+(?:\.[\w-]+)+")
OK_MAIL = ("@users.noreply.github.com", "noreply@anthropic.com", "noreply@github.com")
# Hosts a site file may name. www.w3.org is the SVG namespace; the two GitHub
# accounts are the source link and the reference credit.
OK_URL = (
    ("www.w3.org", ""),
    ("github.com", "/weewzr"),
    ("github.com", "/theodoreOnzGit"),
    ("doi.org", ""),
)
URL = re.compile(r"https?://[^\s\"'`<>()\\]+", re.I)
PROTO_REL = re.compile(r"""(?:\b(?:src|href|action|data|poster)\s*=\s*["']?|url\(\s*["']?)//[A-Za-z0-9]""", re.I)
ROOT_REL = re.compile(
    r"""(?:\b(?:src|href|action|data|poster)\s*=\s*["']|url\(\s*["']?|\bfetch\s*\(\s*["'`]|new\s+Worker\s*\(\s*["'`]"""
    r"""|new\s+URL\s*\(\s*["'`]|\bfrom\s*["'`]|\bimport\s*\(\s*["'`]|importScripts\s*\(\s*["'`])/(?!/)""")
RASTER = (".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico", ".avif", ".tif", ".tiff", ".pdf",
          ".woff", ".woff2", ".ttf", ".otf", ".eot")
IMAGE_TAG = re.compile(r"<image\b", re.I)
IMG_TAG = re.compile(r"""<img\b[^>]*\bsrc\s*=\s*["']?([^"'\s>]+)""", re.I)
RASTER_URI = re.compile(r"data:image/(?:png|jpe?g|gif|webp|bmp|avif)", re.I)
FONT_FACE = re.compile(r"@font-face", re.I)
TEXT_SUFFIX = {".html", ".htm", ".js", ".mjs", ".css", ".json", ".csv", ".svg", ".txt", ".md", ".xml",
               ".webmanifest", ".map"}
FOOTER = ("NUS-H₂: Nuclear-Upgraded Steam Methane Reforming with Carbon Capture for Low-Carbon Hydrogen "
          "in Singapore. Wee Zhern Ray. MIT licence. OUTRAM PARK (github.com/theodoreOnzGit/outram-park-backend) "
          "was consulted as a reference; no OUTRAM PARK code is included and no endorsement is implied.")
REPO_SKIP = {"LICENSE", "Cargo.lock"}
LOCAL_ONLY = ("report/", "presentation/")
# The four final files are published from those folders; every other path
# there stays local.
PUBLISHED = {"report/NUSH2_Final_Report.pdf", "report/NUSH2_Final_Report.docx",
             "presentation/NUSH2_Final_Presentation.pptx", "presentation/NUSH2_Final_Presentation.pdf"}


def compiled():
    text = {k: re.compile(_d(p), re.I if f else 0) for k, (p, f, _s) in TEXT_RULES.items()}
    text["emdash"] = re.compile(EMDASH)
    binary = {k: text[k] for k in BINARY_RULES}
    binary["drive_root"] = re.compile(_d(DRIVE_ROOT))
    withdrawn = {k: re.compile(_d(p)) for k, (p, _s) in WITHDRAWN.items()}
    withdrawn["pointer"] = re.compile(_d(POINTER[0]), re.I)
    return text, binary, withdrawn


def where(text, pos):
    line = text.count("\n", 0, pos) + 1
    col = pos - (text.rfind("\n", 0, pos) + 1) + 1
    return f"{line}:{col}"


class Hits:
    def __init__(self):
        self.items = []

    def add(self, rule, name, detail=""):
        self.items.append((rule, name, detail))

    def rules_for(self, name):
        return {r for r, n, _ in self.items if n == name}


def is_binary(data):
    return b"\0" in data[:8000]


def scan_text(hits, name, text, rules):
    for rule, rx in rules.items():
        m = rx.search(text)
        if m:
            hits.add(rule, name, where(text, m.start()))
    for m in EMAIL.finditer(text):
        if not m.group(0).lower().endswith(OK_MAIL):
            hits.add("email", name, where(text, m.start()))
            break


def url_ok(url):
    s = urlsplit(url)
    host = (s.hostname or "").lower()
    return any(host == h and s.path.startswith(p) for h, p in OK_URL)


class FooterText(HTMLParser):
    """Every text node inside <footer>, so a second line there is caught too."""

    def __init__(self):
        super().__init__()
        self.depth_footer = 0
        self.parts = []
        self.found = False

    def handle_starttag(self, tag, attrs):
        if tag == "footer":
            self.depth_footer += 1
        elif tag == "p" and self.depth_footer:
            self.found = True

    def handle_endtag(self, tag):
        if tag == "footer":
            self.depth_footer -= 1

    def handle_data(self, data):
        if self.depth_footer:
            self.parts.append(data)


def check_footer(hits, site):
    index = site / "index.html"
    if not index.is_file():
        hits.add("footer", "index.html", "missing")
        return
    p = FooterText()
    p.feed(index.read_text(encoding="utf-8"))
    got = " ".join("".join(p.parts).split())
    if not p.found:
        hits.add("footer", "index.html", "no footer paragraph")
    elif got != FOOTER:
        hits.add("footer", "index.html", "the footer text differs from the approved line")


def scan_site(site, rules):
    text_rules, binary_rules, withdrawn = rules
    hits = Hits()
    files = sorted(p for p in site.rglob("*") if p.is_file())
    for p in files:
        name = p.relative_to(site).as_posix()
        low = name.lower()
        if low.endswith(RASTER):
            hits.add("files", name, "raster, PDF or font file")
            continue
        data = p.read_bytes()
        if is_binary(data) or p.suffix.lower() not in TEXT_SUFFIX:
            text = data.decode("latin-1")
            for rule, rx in binary_rules.items():
                m = rx.search(text)
                if m:
                    hits.add(rule, name, f"byte {m.start()}")
            continue
        text = data.decode("utf-8", "replace")
        scan_text(hits, name, text, text_rules)
        for rule, rx in withdrawn.items():
            m = rx.search(text)
            if m:
                hits.add(rule, name, where(text, m.start()))
        for rx, rule in ((IMAGE_TAG, "markup"), (RASTER_URI, "markup"), (FONT_FACE, "markup")):
            m = rx.search(text)
            if m:
                hits.add(rule, name, where(text, m.start()))
        for m in IMG_TAG.finditer(text):
            if not urlsplit(m.group(1)).path.lower().endswith(".svg"):
                hits.add("markup", name, where(text, m.start()))
        for m in URL.finditer(text):
            if not url_ok(m.group(0)):
                hits.add("hosts", name, where(text, m.start()))
        m = PROTO_REL.search(text)
        if m:
            hits.add("hosts", name, where(text, m.start()))
        if p.suffix.lower() in (".html", ".htm", ".js", ".mjs", ".css", ".svg"):
            m = ROOT_REL.search(text)
            if m:
                hits.add("absolute", name, where(text, m.start()))
    check_footer(hits, site)
    return hits, len(files)


def repo_files(repo):
    out = subprocess.run(["git", "-C", str(repo), "ls-files", "--cached", "--others", "--exclude-standard"],
                         capture_output=True, text=True, check=True).stdout
    return [f for f in out.splitlines() if f]


def scan_repo(repo, names, rules):
    text_rules = rules[0]
    hits = Hits()
    scanned = 0
    for name in names:
        if name.startswith(LOCAL_ONLY) and name not in PUBLISHED:
            hits.add("local_only", name, "path under a local-only folder")
        if name in REPO_SKIP:
            continue
        p = repo / name
        if not p.is_file():
            continue
        data = p.read_bytes()
        if is_binary(data):
            continue
        scanned += 1
        scan_text(hits, name, data.decode("utf-8", "replace"), text_rules)
    return hits, scanned


def report(title, hits, count):
    print(f"{title}: {count} files scanned, {len(hits.items)} hit(s)")
    for rule, name, detail in hits.items:
        print(f"  {rule}: {name} {detail}".rstrip())


# ------------------------------------------------------------------ proof

def plants():
    """(file name, content, expected rule) for every rule, written into a copy of the site."""
    out = []
    for rule, (_p, _f, sample) in TEXT_RULES.items():
        out.append((f"planted/{rule}.txt", f"Planted line: {_d(sample)} end.\n", rule))
    for rule, (_p, sample) in WITHDRAWN.items():
        out.append((f"planted/{rule}.txt", f"Planted value {_d(sample)} here.\n", rule))
    out.append(("planted/pointer.txt", f"Planted note: {_d(POINTER[1])}.\n", "pointer"))
    out += [
        ("planted/emdash.txt", f"one {EMDASH} two\n", "emdash"),
        ("planted/email.txt", "write to someone" + chr(64) + "example.com\n", "email"),
        ("planted/raster.png", "not really a picture\n", "files"),
        ("planted/font.woff2", "not really a font\n", "files"),
        ("planted/image.svg", '<svg xmlns="http://www.w3.org/2000/svg"><image href="x.svg"/></svg>\n', "markup"),
        ("planted/img.html", '<!doctype html><img src="photo.jpg" alt="">\n', "markup"),
        ("planted/font.css", "@font-face { font-family: x; }\n", "markup"),
        ("planted/host.js", 'fetch("https://example.com/track.js");\n', "hosts"),
        ("planted/protorel.html", '<!doctype html><script src="//example.com/x.js"></script>\n', "hosts"),
        ("planted/rootrel.html", '<!doctype html><link rel="stylesheet" href="/style.css">\n', "absolute"),
        ("planted/rootrel.js", 'const w = new Worker("/worker.js");\n', "absolute"),
    ]
    return out


def prove(site, repo, rules):
    fails = []
    with tempfile.TemporaryDirectory() as tmp:
        copy = Path(tmp) / "site"
        shutil.copytree(site, copy)
        clean, _ = scan_site(copy, rules)
        if clean.items:
            fails.append(f"the unplanted copy has {len(clean.items)} hit(s)")
        planted = plants()
        for name, content, _rule in planted:
            p = copy / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(content, encoding="utf-8")
        index = copy / "index.html"
        text = index.read_text(encoding="utf-8")
        marker = "<footer"
        at = text.find(marker)
        p_at = text.find("<p>", at) if at >= 0 else -1
        if p_at < 0:
            fails.append("no footer paragraph to plant into")
        else:
            index.write_text(text[:p_at + 3] + "Planted Name. " + text[p_at + 3:], encoding="utf-8")
            planted.append(("index.html", "", "footer"))
        hits, _ = scan_site(copy, rules)
        caught = 0
        for name, _content, rule in planted:
            got = hits.rules_for(name)
            if rule in got:
                caught += 1
            else:
                fails.append(f"site plant {name} not caught by {rule}; got {sorted(got)}")
        print(f"site proof: {len(planted)} plants, {caught} caught")

        # Repository scope: one text plant per rule, and a local-only path.
        fake = Path(tmp) / "repo"
        fake.mkdir()
        names = []
        repo_plants = []
        for rule, (_p, _f, sample) in TEXT_RULES.items():
            n = f"docs/{rule}.md"
            (fake / "docs").mkdir(exist_ok=True)
            (fake / n).write_text(f"Planted line: {_d(sample)} end.\n", encoding="utf-8")
            names.append(n)
            repo_plants.append((n, rule))
        (fake / "docs" / "emdash.md").write_text(f"a {EMDASH} b\n", encoding="utf-8")
        names.append("docs/emdash.md")
        repo_plants.append(("docs/emdash.md", "emdash"))
        (fake / "report").mkdir()
        (fake / "report" / "draft.md").write_text("clean text\n", encoding="utf-8")
        names.append("report/draft.md")
        repo_plants.append(("report/draft.md", "local_only"))
        # A published deliverable is allowed; it must not be flagged.
        (fake / "report" / "NUSH2_Final_Report.pdf").write_bytes(b"%PDF-1.7\n\0\n")
        names.append("report/NUSH2_Final_Report.pdf")
        (fake / "LICENSE").write_text("clean\n", encoding="utf-8")
        names.append("LICENSE")
        rh, _ = scan_repo(fake, names, rules)
        if rh.rules_for("report/NUSH2_Final_Report.pdf"):
            fails.append("the published report PDF was flagged as local only")
        rcaught = 0
        for n, rule in repo_plants:
            if rule in rh.rules_for(n):
                rcaught += 1
            else:
                fails.append(f"repo plant {n} not caught by {rule}; got {sorted(rh.rules_for(n))}")
        print(f"repo proof: {len(repo_plants)} plants, {rcaught} caught")
    if fails:
        for f in fails:
            print(f"  {f}")
        print("FAIL: the guard missed a plant")
        return 1
    print("PASS: every plant caught, unplanted copy clean")
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="Content guard for _site and the repository")
    ap.add_argument("--site", type=Path, default=ROOT / "_site")
    ap.add_argument("--repo", type=Path, default=ROOT)
    ap.add_argument("--prove", action="store_true", help="plant one sample per rule in a scratch copy")
    args = ap.parse_args(argv)
    if not (args.site / "index.html").is_file():
        print(f"FAIL: {args.site} holds no index.html; run scripts/build_site.py")
        return 1
    rules = compiled()
    if args.prove:
        return prove(args.site, args.repo, rules)
    site_hits, n_site = scan_site(args.site, rules)
    report("site", site_hits, n_site)
    repo_hits, n_repo = scan_repo(args.repo.resolve(), repo_files(args.repo), rules)
    report("repository", repo_hits, n_repo)
    if site_hits.items or repo_hits.items:
        print("FAIL")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
