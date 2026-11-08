"""Small helper: print wrapped lines from pre-extracted text with keyword hits."""
import re
import sys
import textwrap

path = sys.argv[1]
pat = re.compile(sys.argv[2], re.I)
width = int(sys.argv[3]) if len(sys.argv) > 3 else 400
with open(path, encoding="utf-8", errors="replace") as f:
    page = "?"
    for n, line in enumerate(f, 1):
        m = re.match(r"=+ PDF PAGE (\d+)", line)
        if m:
            page = m.group(1)
        if pat.search(line):
            s = line.strip()
            if len(s) > width:
                # show windows around first few hits
                out = []
                for hit in list(pat.finditer(s))[:4]:
                    a = max(0, hit.start() - width // 2)
                    b = min(len(s), hit.end() + width // 2)
                    out.append(s[a:b])
                s = " ... ".join(out)
            print(f"[L{n} p{page}] {s}")
