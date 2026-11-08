#!/usr/bin/env python3
"""Scan tracked and untracked repo text for local paths, personal identifiers and
withdrawn figures. Patterns are base64 encoded so this file does not trip its own
scan or the pre-push hook.

Usage: python scripts/check_clean.py
Exit 0 clean, 1 on any hit.
"""
import base64
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP = (".lock",)
ENCODED = {
    "path": "XGJbQS1aYS16XTpbXFwvXSg/OlVzZXJzfFkzUzF8UmF5IENsYXVkZSk=",
    "profile": "KD9pOkFwcERhdGF8c2NyYXRjaHBhZCk=",
    "matric": "XGJBXGR7N31bQS1aXVxi",
    "cohort": "KD9pOlxiZ3JvdXBbXHNfLV0qKD86N3xzZXZlbilcYik=",
    "schedule": "KD9pOlxidGltZVtccy1dP3Nsb3RzP1xifFxiZGVhZGxpbmVzP1xiKQ==",
    "allocation": "KD9pOlxic3BlYWtlciBhbGxvY2F0aW9uXGIp",
    "declaration": "KD9pOlxiQUkgZGVjbGFyYXRpb25cYik=",
}
# Withdrawn figures, checked in results only, because the design record may name
# them once in a short history paragraph.
WITHDRAWN = "UyRcczk2XGJ8UyQ1NDZNfFVTJDgwL3R8XGIxMDZcLjVcYnxcYjk5XC40XGI="


def decode(s):
    return re.compile(base64.b64decode(s).decode("utf-8"))


def files():
    out = subprocess.run(["git", "-C", str(ROOT), "ls-files", "--cached", "--others", "--exclude-standard"],
                         capture_output=True, text=True, check=True).stdout
    return [f for f in out.splitlines() if f and not f.endswith(SKIP)]


def main():
    rules = {k: decode(v) for k, v in ENCODED.items()}
    rules["emdash"] = re.compile(chr(0x2014))
    withdrawn = decode(WITHDRAWN)
    hits = []
    names = files()
    for f in names:
        p = ROOT / f
        if not p.is_file():
            continue
        text = p.read_bytes().decode("utf-8", "replace")
        for label, rx in rules.items():
            m = rx.search(text)
            if m:
                hits.append(f"{label}: {f}: offset {m.start()}")
        if f.startswith("results/"):
            m = withdrawn.search(text)
            if m:
                hits.append(f"withdrawn: {f}: offset {m.start()}")
    print(f"scanned {len(names)} files, {len(hits)} hits")
    for h in hits:
        print(h)
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
