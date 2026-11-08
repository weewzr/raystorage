import re
import sys

BASE = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/econ_txt/"


def scan(key, pat, width=170, limit=80):
    rx = re.compile(pat, re.I)
    txt = open(BASE + key + ".txt", encoding="utf-8").read()
    pages = re.split(r"=====PDFPAGE (\d+)=====", txt)
    n = 0
    for i in range(1, len(pages), 2):
        pg = pages[i]
        body = pages[i + 1]
        flat = re.sub(r"\s+", " ", body)
        for m in rx.finditer(flat):
            s = max(0, m.start() - width)
            e = min(len(flat), m.end() + width)
            print("[p%s] ...%s..." % (pg, flat[s:e]))
            n += 1
            if n >= limit:
                return


if __name__ == "__main__":
    scan(sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 170, int(sys.argv[4]) if len(sys.argv) > 4 else 80)
