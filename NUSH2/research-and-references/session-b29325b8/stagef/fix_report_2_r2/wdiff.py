import sys, difflib, re
a = open(sys.argv[1], encoding="utf-8").read().replace("\r\n", "\n")
b = open(sys.argv[2], encoding="utf-8").read().replace("\r\n", "\n")
ta = re.findall(r"\S+|\s+", a); tb = re.findall(r"\S+|\s+", b)
sm = difflib.SequenceMatcher(None, ta, tb, autojunk=False)
for op, i1, i2, j1, j2 in sm.get_opcodes():
    if op == "equal": continue
    ctx = "".join(ta[max(0, i1-12):i1]).replace("\n", " ")
    print(f"... {ctx[-90:]}\n   - {''.join(ta[i1:i2])!r}\n   + {''.join(tb[j1:j2])!r}\n")
