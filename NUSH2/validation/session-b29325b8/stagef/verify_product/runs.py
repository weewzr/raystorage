"""Longest run of consecutive shared K-token windows per (engine file, OUTRAM file).

python runs.py <engine_dir> <ext_dir> K
Tokens: comments and strings removed, numbers replaced by N. A run of r consecutive
matched windows means K + r - 1 identical consecutive tokens.
"""
import re, sys
from collections import defaultdict
from pathlib import Path

ENG, EXT, K = Path(sys.argv[1]), Path(sys.argv[2]), int(sys.argv[3])
COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING = re.compile(r'r#*"(?:.|\n)*?"#*|"(?:\\.|[^"\\])*"')
TOK = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?(?:f64|f32|usize|i32|u32)?"
                 r"|::|->|=>|==|!=|<=|>=|&&|\|\||[{}()\[\];,.<>:=+\-*/&|!?^%#@]")


def toks(src):
    s = STRING.sub('"S"', COMMENT.sub(" ", src))
    return ["N" if t[0].isdigit() else t for t in TOK.findall(s)]


index = defaultdict(list)
etoks = {}
for p in sorted(ENG.rglob("*.rs")):
    t = toks(p.read_text(encoding="utf-8", errors="replace"))
    f = str(p.relative_to(ENG))
    etoks[f] = t
    for i in range(len(t) - K + 1):
        index[hash(tuple(t[i:i + K]))].append((f, i))

best = {}
for p in EXT.rglob("*.rs"):
    if ".git" in p.parts or "target" in p.parts:
        continue
    try:
        t = toks(p.read_text(encoding="utf-8", errors="replace"))
    except OSError:
        continue
    matched = defaultdict(set)
    for j in range(len(t) - K + 1):
        h = hash(tuple(t[j:j + K]))
        if h in index:
            for f, i in index[h]:
                matched[f].add(i)
    for f, idx in matched.items():
        s = sorted(idx)
        run, longest, start, bstart = 1, 1, s[0], s[0]
        for a, b in zip(s, s[1:]):
            if b == a + 1:
                run += 1
            else:
                run, start = 1, b
            if run > longest:
                longest, bstart = run, start
        if longest > best.get(f, (0,))[0]:
            best[f] = (longest, str(p.relative_to(EXT)), bstart)

for f, (r, src, st) in sorted(best.items(), key=lambda kv: -kv[1][0])[:25]:
    n = K + r - 1
    print(f"{n:4d} tokens  {f}  <-  {src}")
    print("      " + " ".join(etoks[f][st:st + n])[:600])
