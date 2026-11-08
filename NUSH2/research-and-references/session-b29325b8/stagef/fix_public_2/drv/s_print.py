import base64
from pathlib import Path

WAIT = "document.getElementById('status-text') && /Solved/.test(document.getElementById('status-text').textContent) && document.querySelector('.cost-table') ? true : false"


def run(d):
    d.nav("index.html")
    d.until(WAIT, 30000)
    d.wait(1500)
    r = d.call("Page.printToPDF", {"printBackground": True, "preferCSSPageSize": True}, 120)
    out = Path(__file__).resolve().parent / "out" / "summary.pdf"
    out.write_bytes(base64.b64decode(r["data"]))
    d.out("pdf", str(out))
