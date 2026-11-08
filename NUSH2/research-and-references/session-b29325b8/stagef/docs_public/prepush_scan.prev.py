#!/usr/bin/env python3
# NUSH2 pre-push scan. Kept outside the repo because it names forbidden strings.
# Usage: python prepush_scan.py <repo>. Exit 0 clean, 1 on any hit.
import re, subprocess, sys

REPO = sys.argv[1]
ALLOWED_PDF = "math/NUSH2_First_Principles_Derivations.pdf"
MAX_BLOB = 2 * 1024 * 1024
MAX_PDF = 5 * 1024 * 1024
BAD_SUFFIX = (".pdf", ".doc", ".docx", ".ppt", ".pptx", ".xls", ".xlsx", ".xlsm",
              ".mp4", ".mov", ".wasm", ".key", ".pem", ".p12", ".pfx")
BAD_PREFIX = ("report/", "presentation/", ".claude/")
OK_MAIL = ("@users.noreply.github.com", "noreply@anthropic.com",
           "noreply@github.com", "git@github.com")

TEXT = {k: re.compile(v) for k, v in {
    "matric": r"(?i:\bA\d{7}[A-Z]\b)",
    "nusnet": r"(?i:\be\d{7}\b)",
    "group": r"(?i:\bgroup[\s_-]*(?:7|seven)\b)",
    "slot": r"(?i:\btime[\s-]?slots?\b|\bpresentation slot\b|\bdeadlines?\b"
            r"|\bdue date\b|\bteammates\b)",
    "speaker": r"(?i:\bspeaker allocation\b|\bspeaking (?:split|time|order)\b)",
    "ai_decl": r"(?i:\bAI[- ]use\b|\bAI declaration\b)",
    "path": r"(?i:\b[A-Z]:[\\/](?:Users|Y3S1|Ray Claude)\b|/c/(?:Users|Y3S1)\b"
            r"|\\Users\\|AppData|scratchpad)",
    "owner": r"\bAlf\b",
    "secret": r"gh[pousr]_[A-Za-z0-9]{20,}|github_pat_\w{20,}|sk-[A-Za-z0-9_-]{20,}"
              r"|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----"
              r"|(?i:(?:password|passwd|api[_-]?key|secret|token)\s*[:=]\s*"
              r"['\"][^'\"]{6,}['\"])",
    "emdash": "\u2014",
    "people": r"(?i:\bjesslynn\b|\bmoovendran\b|\bshruti\b)",
    "course": r"(?i:\bCN\s?4252\b|low[\s-]carbon technologies|\bAY\s?20\d\d"
              r"|\bsemester\s?[12]\b)",
}.items()}
NAME_RULES = {k: TEXT[k] for k in ("matric", "group", "slot", "people", "course")}
BIN_RULES = {k: TEXT[k] for k in ("matric", "path")}
MAIL = re.compile(r"[\w.+-]+@[\w-]+\.[\w.-]+")
hits = []


def git(*args, data=None):
    return subprocess.run(["git", "-C", REPO, *args], input=data,
                          capture_output=True, check=True).stdout


def scan(label, text, rules=TEXT, mail=True):
    for name, rx in rules.items():
        m = rx.search(text)
        if m:  # location only, so the log never repeats a matched secret
            hits.append(f"{name}: {label}: offset {m.start()}")
    if mail:
        for m in MAIL.finditer(text):
            if not m.group(0).lower().endswith(OK_MAIL):
                hits.append(f"email: {label}: offset {m.start()}")
                break


paths = {}
for line in git("rev-list", "--objects", "--all").decode("utf-8", "replace").splitlines():
    sha, _, path = line.partition(" ")
    if path:
        paths.setdefault(sha, path)

raw = git("cat-file", "--batch", data="".join(s + "\n" for s in paths).encode())
pos = 0
while pos < len(raw):
    end = raw.index(b"\n", pos)
    sha, kind, size = raw[pos:end].decode().split()
    size = int(size)
    blob = raw[end + 1:end + 1 + size]
    pos = end + 1 + size + 1
    if kind != "blob":
        continue
    path = paths[sha]
    low = path.lower()
    scan("name " + path, path, NAME_RULES, mail=False)
    if low.startswith(BAD_PREFIX):
        hits.append(f"local-only path: {path}")
    if path == ALLOWED_PDF:
        if size > MAX_PDF:
            hits.append(f"size: {path}: {size}")
    else:
        if low.endswith(BAD_SUFFIX) or blob[:5] == b"%PDF-" or blob[:4] == b"PK\x03\x04":
            hits.append(f"file type: {path}")
        if size > MAX_BLOB:
            hits.append(f"size: {path}: {size}")
    if b"\0" in blob[:8000]:
        scan(path, blob.decode("latin-1"), BIN_RULES, mail=False)
    else:
        scan(path, blob.decode("utf-8", "replace"))

log = git("log", "--all", "--format=%H%x00%ae%x00%ce%x00%B%x01").decode("utf-8", "replace")
for rec in log.split("\x01"):
    parts = rec.strip().split("\0")
    if len(parts) < 4:
        continue
    for addr in parts[1:3]:
        if not addr.lower().endswith(OK_MAIL):
            hits.append(f"commit email: {parts[0][:10]}")
    scan("commit " + parts[0][:10], parts[3])

scan("refs", git("for-each-ref", "--format=%(refname)").decode(), NAME_RULES, mail=False)

for h in hits[:200]:
    print(h)
print(f"{len(hits)} hit(s)")
sys.exit(1 if hits else 0)
