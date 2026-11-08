import re
import sys

BASE = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/econ_txt/"
key = sys.argv[1]
want = []
for tok in sys.argv[2].split(","):
    if "-" in tok:
        a, b = tok.split("-")
        want += list(range(int(a), int(b) + 1))
    else:
        want.append(int(tok))
maxc = int(sys.argv[3]) if len(sys.argv) > 3 else 6000
txt = open(BASE + key + ".txt", encoding="utf-8").read()
pages = re.split(r"=====PDFPAGE (\d+)=====", txt)
d = {int(pages[i]): pages[i + 1] for i in range(1, len(pages), 2)}
for p in want:
    body = re.sub(r"[ \t]+", " ", d.get(p, ""))
    body = re.sub(r"\n\s*\n+", "\n", body)
    print("--- %s p%d" % (key, p))
    print(body[:maxc].encode("ascii", "replace").decode())
