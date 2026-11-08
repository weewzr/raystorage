"""Word-level diff of two PDFs' text with page numbers and the TOC left in; prints changed runs."""
import sys, re, difflib, pymupdf
def words(path):
    out = []
    for p in pymupdf.open(path):
        out += re.findall(r"\S+", p.get_text())
    return out
a, b = words(sys.argv[1]), words(sys.argv[2])
sm = difflib.SequenceMatcher(a=a, b=b, autojunk=False)
n = 0
for op, i1, i2, j1, j2 in sm.get_opcodes():
    if op == "equal":
        continue
    n += 1
    print(op, "| OLD:", " ".join(a[i1:i2])[:300], "| NEW:", " ".join(b[j1:j2])[:300])
print("changed runs:", n, "words old", len(a), "new", len(b))
