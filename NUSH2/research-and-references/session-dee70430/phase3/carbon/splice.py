import sys
# usage: splice.py FILE START_MARKER END_MARKER SNIPPET
path, start, end, snip = sys.argv[1:5]
t = open(path, encoding="utf-8", newline="").read()
new = open(snip, encoding="utf-8", newline="").read()
a = t.index(start)
b = t.index(end, a)
t = t[:a] + new + t[b:]
open(path, "w", encoding="utf-8", newline="").write(t)
print("spliced", len(new), "bytes into", path)
