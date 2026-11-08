"""List overfull boxes by section file in a Tectonic log."""
import re
import sys

log = open(sys.argv[1], encoding="utf-8", errors="replace").read()
want = sys.argv[2] if len(sys.argv) > 2 else "EFGH"
cur = None
pat = re.compile(r"sections/(\w+)|Overfull \\hbox \(([\d.]+)pt too wide\)[^\n]*?lines? (\d+)")
for m in pat.finditer(log):
    if m.group(1):
        cur = m.group(1)
    elif cur and cur[0] in want:
        print(cur, m.group(2), m.group(3))
