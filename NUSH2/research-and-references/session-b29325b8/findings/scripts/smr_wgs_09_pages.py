import re, sys
# usage: script txtfile p1 p2 ...  prints selected PDF pages from a cached text file
p = sys.argv[1]
want = {int(x) for x in sys.argv[2:]}
t = open(p, encoding="utf-8", errors="replace").read()
pages = re.split(r"\n\n=== PDF page (\d+) of \d+ ===\n", t)
out = []
for i in range(1, len(pages), 2):
    n = int(pages[i])
    if n in want:
        out.append(f"\n##### PDF page {n}\n" + pages[i + 1])
sys.stdout.reconfigure(encoding="utf-8")
print("\n".join(out))
