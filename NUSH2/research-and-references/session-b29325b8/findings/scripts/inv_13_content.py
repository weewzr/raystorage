"""Page-level content evidence for documents not in the existing CSV."""
import os, re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
D = SP + "/findings/inv_txt/"

def pages(sha):
    fn = [f for f in os.listdir(D) if f.startswith(sha) and f.endswith("_full.txt")][0]
    t = open(D + fn, encoding="utf-8").read()
    parts = re.split(r"\n=== PDF p\.(\d+) ===\n", t)
    return {int(parts[i]): re.sub(r"\s+", " ", parts[i + 1]) for i in range(1, len(parts), 2)}

def abstract(sha, n=900, start=r"(?i)a\s*b\s*s\s*t\s*r\s*a\s*c\s*t"):
    p = pages(sha)
    t = p[1]
    m = re.search(start, t)
    print("  ABS:", t[m.end():m.end() + n] if m else t[:n])

def grep(sha, pat, width=150, maxn=12):
    p = pages(sha)
    n = 0
    for k in sorted(p):
        for m in re.finditer(pat, p[k]):
            s = max(0, m.start() - 40)
            print(f"  p{k}: {p[k][s:m.end() + width]}")
            n += 1
            if n >= maxn:
                return

docs = {
 "Aronu": ("9009af", [r"Table \d+\s*\.? ?[A-Z]", r"Corrigendum|corrigendum", r"(?i)average absolute"]),
 "Khan": ("9aeed4", [r"Table 1", r"(?i)validat", r"(?i)Onda|Billet|Rocha"]),
 "Ahn": ("28106d", [r"(?i)Table \d\.", r"(?i)MDEA|MEA|amine", r"(?i)reactor capacity|MWth|MW th", r"(?i)LCOH"]),
 "Zhang2011": ("d8d7c0", [r"(?i)temperature range|up to \d+ ?K", r"(?i)average absolute|AARD|deviation"]),
 "Isa": ("cdf5ff", [r"(?i)pressure modification", r"(?i)Aspen|Hysys|rate-based|average"]),
 "Salvinder": ("61119f", [r"(?i)Aspen", r"(?i)deviation|% error|error of"]),
 "Hartono": ("5a9ce3", [r"J\. Chem\. Eng\. Data", r"(?i)Received:|Published:", r"(?i)temperature range|from \d+ to \d+"]),
 "Sema": ("12793b", [r"(?i)Table 2", r"(?i)MEA.{0,60}(activation|rate constant)"]),
 "Versteeg": ("0cfd96", [r"(?i)monoethanolamine|MEA", r"(?i)Table 5|Table 6"]),
}
sel = sys.argv[1:] or list(docs)
for name in sel:
    sha, pats = docs[name]
    print("#" * 60, name)
    abstract(sha)
    for pat in pats:
        print(" [", pat, "]")
        grep(sha, pat, maxn=7)
