import re, sys, subprocess
from pathlib import Path
RX = re.compile(r"(?i)\b(?:not|never|no longer|isn't|is not|aren't|are not|wasn't|doesn't|does not|do not)\b[^.;:!?\n]{1,70}?\bbut\b(?! also)[^.;:!?\n]{0,60}|\bnot only\b[^.]{0,80}|\bisn't\b[^.]{0,50}\bit's\b|\bnot (?:a|an|the) \w+(?: \w+)?,? (?:it is|it's)\b")
def scan(label, text):
    t = " ".join(text.split())
    for m in RX.finditer(t):
        s = t[max(0, m.start()-40):m.end()+10]
        print(f"{label} | {s}")
V = Path(sys.argv[1]); repo = sys.argv[2]
for f in ["out/report_pdf_text.txt", "out/deck_pdf_text.txt", "out/deriv_pdf_text.txt", "out_local/script_pdf_text.txt", "out_local/qa_pdf_text.txt"]:
    t = (V / f).read_text(encoding="utf-8")
    for pg in t.split("===== page ")[1:]:
        n, b = pg.split("\n", 1)
        scan(f"{f.split('/')[-1]} p{n}", b)
names = subprocess.run(["git", "-C", repo, "ls-files"], capture_output=True, text=True).stdout.split()
for n in names:
    if n.endswith((".md", ".html", ".js", ".json", ".tex", ".csv", ".toml", ".py", ".rs", ".yml")):
        try:
            t = open(f"{repo}/{n}", encoding="utf-8").read()
        except Exception:
            continue
        for i, line in enumerate(t.splitlines(), 1):
            scan(f"{n}:{i}", line)
