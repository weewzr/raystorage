import sys, re
# usage: htgr_pages.py key p1 p2 ...
base = "C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/htgr_txt/"
key = sys.argv[1]
pages = set(int(x) for x in sys.argv[2:])
txt = open(base + key + ".txt", encoding="utf-8").read()
parts = re.split(r"\n=====PDFPAGE (\d+)=====\n", txt)
for i in range(1, len(parts), 2):
    n = int(parts[i])
    if n in pages:
        print("#### %s PDF page %d" % (key, n))
        body = parts[i + 1]
        # compress runs of blank lines
        body = re.sub(r"\n\s*\n+", "\n", body)
        sys.stdout.buffer.write(body.encode("utf-8", "replace"))
        sys.stdout.buffer.write(b"\n")
