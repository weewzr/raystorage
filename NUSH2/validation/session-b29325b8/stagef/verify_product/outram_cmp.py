"""Compare engine Rust sources with the OUTRAM PARK extract (read only).

python outram_cmp.py <engine_dir> <ext_dir> <out_dir>
1. Code shingles: comments and string literals removed, every number replaced by N,
   K consecutive tokens hashed. Reports engine windows found in OUTRAM.
2. Identifier overlap: identifiers of 3 or more parts (snake or camel) present in both.
3. Comment shingles: 10-word windows of comment text present in both.
"""
import os, re, sys, json
from collections import defaultdict
from pathlib import Path

ENG, EXT, OUT = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
K = 20
W = 10
COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING = re.compile(r'r#*"(?:.|\n)*?"#*|"(?:\\.|[^"\\])*"')
TOK = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?(?:f64|f32|usize|i32|u32)?|::|->|=>|==|!=|<=|>=|&&|\|\||[{}()\[\];,.<>:=+\-*/&|!?^%#@]")
WORD = re.compile(r"[A-Za-z]{2,}")
KW = set("""as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn f64 f32 usize u32 i32 u64 i64 u8 bool str String Vec Option Some None Ok Err Result Box new len iter map collect clone into unwrap expect push sum abs sqrt powi powf exp ln max min test assert assert_eq derive Debug Clone Copy PartialEq Default""".split())


def code_tokens(src):
    s = COMMENT.sub(" ", src)
    s = STRING.sub(' "S" ', s)
    toks = []
    for m in TOK.finditer(s):
        t = m.group(0)
        if t[0].isdigit():
            t = "N"
        toks.append((t, s.count("\n", 0, m.start()) if False else 0))
    return [t for t, _ in toks]


def lines_tokens(src):
    """Tokens with line numbers, for reporting."""
    s = COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), src)
    s = STRING.sub(lambda m: '"' + re.sub(r"[^\n]", " ", m.group(0)[1:-1]) + '"' if len(m.group(0)) > 1 else m.group(0), s)
    out = []
    line = 1
    pos = 0
    for m in TOK.finditer(s):
        line += s.count("\n", pos, m.start())
        pos = m.start()
        t = m.group(0)
        if t[0].isdigit():
            t = "N"
        out.append((t, line))
    return out


def comments(src):
    return " ".join(m.group(0) for m in COMMENT.finditer(src))


def ident_parts(name):
    parts = [p for p in re.split(r"_|(?<=[a-z0-9])(?=[A-Z])", name) if p]
    return parts


eng_files = sorted(p for p in ENG.rglob("*.rs") if "target" not in p.parts)
eng_shingles = defaultdict(list)
eng_idents = defaultdict(set)
eng_cwin = defaultdict(list)
for p in eng_files:
    src = p.read_text(encoding="utf-8", errors="replace")
    lt = lines_tokens(src)
    toks = [t for t, _ in lt]
    for i in range(len(toks) - K + 1):
        win = toks[i:i + K]
        # skip windows with no identifier other than keywords (pure punctuation or number runs)
        idn = [t for t in win if re.match(r"[A-Za-z_]", t) and t not in KW]
        if len(set(idn)) < 4:
            continue
        eng_shingles[hash(tuple(win))].append((str(p.relative_to(ENG)), lt[i][1]))
    for t in set(toks):
        if re.match(r"[A-Za-z_]", t) and len(ident_parts(t)) >= 3:
            eng_idents[t].add(str(p.relative_to(ENG)))
    words = [w.lower() for w in WORD.findall(comments(src))]
    for i in range(len(words) - W + 1):
        eng_cwin[hash(tuple(words[i:i + W]))].append(str(p.relative_to(ENG)))

print("engine files", len(eng_files), "shingles", len(eng_shingles), "idents", len(eng_idents), flush=True)

code_hits = defaultdict(set)   # engine (file,line) -> outram file
ident_hits = defaultdict(set)
comment_hits = defaultdict(set)
ext_files = [p for p in EXT.rglob("*.rs") if ".git" not in p.parts and "target" not in p.parts]
print("outram files", len(ext_files), flush=True)
for n, p in enumerate(ext_files):
    try:
        src = p.read_text(encoding="utf-8", errors="replace")
    except OSError:
        continue
    if len(src) > 3_000_000:
        continue
    toks = code_tokens(src)
    for i in range(len(toks) - K + 1):
        h = hash(tuple(toks[i:i + K]))
        if h in eng_shingles:
            for loc in eng_shingles[h]:
                code_hits[loc].add(str(p.relative_to(EXT)))
    for t in set(toks):
        if t in eng_idents:
            ident_hits[t].add(str(p.relative_to(EXT)))
    words = [w.lower() for w in WORD.findall(comments(src))]
    for i in range(len(words) - W + 1):
        h = hash(tuple(words[i:i + W]))
        if h in eng_cwin:
            comment_hits[" ".join(words[i:i + W])].add(str(p.relative_to(EXT)))
    if n % 500 == 0:
        print("scanned", n, flush=True)

# group code hits into runs per engine file
runs = defaultdict(list)
for (f, line), srcs in code_hits.items():
    runs[f].append((line, sorted(srcs)[:3]))
summary = {f: sorted(set(l for l, _ in v)) for f, v in runs.items()}
OUT.mkdir(parents=True, exist_ok=True)
(OUT / "outram_code_hits.json").write_text(json.dumps({f: {"lines": v, "sources": sorted({s for _, ss in runs[f] for s in ss})[:5]} for f, v in summary.items()}, indent=1))
(OUT / "outram_ident_hits.json").write_text(json.dumps({k: {"engine": sorted(eng_idents[k]), "outram_files": len(v), "example": sorted(v)[:2]} for k, v in sorted(ident_hits.items())}, indent=1))
(OUT / "outram_comment_hits.json").write_text(json.dumps({k: sorted(v)[:3] for k, v in comment_hits.items()}, indent=1))
print("code windows matched", len(code_hits), "in", len(summary), "engine files")
print("identifiers (3+ parts) shared", len(ident_hits))
print("comment 10-word windows shared", len(comment_hits))
