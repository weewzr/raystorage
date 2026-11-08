"""Run the dataviz palette validator inside headless Chrome, since this machine has no node."""
import json
import re
import subprocess
import tempfile
from pathlib import Path

SKILL = Path("C:/Users/Alf/AppData/Local/Temp/claude/bundled-skills/2.1.309/3eb482a7921e21aeba379afcff22d3f5/dataviz")
CHROME = "C:/Program Files/Google/Chrome/Application/chrome.exe"
SURFACE = "#ffffff"
CASES = {
    "n series, all pairs": (["#2a78d6", "#eb6834", "#1baf7a"], "all"),
    "polarity pair": (["#e34948", "#2a78d6"], "adjacent"),
}

js = (SKILL / "scripts/validate_palette.js").read_text(encoding="utf-8").replace("</script", "<\\/script")
runner = "const out = {};\n"
for name, (pal, pairs) in CASES.items():
    runner += (f"out[{json.dumps(name)}] = validate({json.dumps(pal)}, "
               f"{{mode: 'light', surface: '{SURFACE}', pairs: '{pairs}'}});\n")
runner += "document.getElementById('report').textContent = JSON.stringify(out);\n"
html = f'<!doctype html><html><body><pre id="report">pending</pre><script type="module">\n{js}\n{runner}\n</script></body></html>'

with tempfile.TemporaryDirectory(prefix="nush2_val_", ignore_cleanup_errors=True) as tmp:
    page = Path(tmp) / "v.html"
    page.write_text(html, encoding="utf-8")
    dom = subprocess.run([CHROME, "--headless=new", "--disable-gpu", f"--user-data-dir={Path(tmp) / 'p'}",
                          "--no-first-run", "--virtual-time-budget=4000", "--dump-dom", page.as_uri()],
                         capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=90, check=True).stdout
m = re.search(r'<pre id="report">(.*?)</pre>', dom, re.S)
text = m[1].replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
if text == "pending":
    raise SystemExit("validator did not run")
for name, res in json.loads(text).items():
    print("==", name, "| keys:", [k for k in res if k != "report"], "| pass:", res.get("pass", res.get("ok")))
    for row in res["report"]:
        print("  ", row[1], "|", row[0], "|", row[2])
