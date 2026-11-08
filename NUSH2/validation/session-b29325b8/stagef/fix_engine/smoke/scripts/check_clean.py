#!/usr/bin/env python3
"""Scan tracked and untracked repo text for local paths, personal identifiers and
withdrawn figures. Patterns are base64 encoded so this file does not trip its own
scan or the pre-push hook.

The public documents (README.md, docs/ and references/) are also checked for
pointers into working notes that never ship. The engine test
engine/plant/tests/no_internal_pointers.rs holds the same list for
engine/data/parameters.toml and results/.

This scan has no rule for people's names or course text. A clean result here does
not make the repository clean for a push; the separate pre-push scan, kept outside
the repository, is the gate for that.

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
    "path": "XGJbQS1aYS16XTpbXFwvXSg/OlVzZXJzfFByb2dyYW0gRmlsZXN8V2luZG93cylcYg==",
    "profile": "KD9pOkFwcERhdGF8c2NyYXRjaHBhZCk=",
    # Any absolute drive or home path, in text files only, because compressed bytes can
    # match a pattern this short.
    "drive": "KD88IVtcdyRcXF0pW0EtWmEtel06W1xcL10rKD89W0EtWmEtejAtOV8kfi4tXSl8KD88IVtcdy5dKS8oPzpVc2Vyc3xob21lKS9bQS1aYS16MC05Xy4tXSs=",
    "matric": "XGJBXGR7N31bQS1aXVxi",
    "cohort": "KD9pOlxiZ3JvdXBbXHNfLV0qXGR7MSwyfVxiKQ==",
    "schedule": "KD9pOlxidGltZVtccy1dP3Nsb3RzP1xifFxiZGVhZGxpbmVzP1xiKQ==",
    "allocation": "KD9pOlxic3BlYWtlciBhbGxvY2F0aW9uXGIp",
    "declaration": "KD9pOlxiQUkgZGVjbGFyYXRpb25cYik=",
    # The withdrawn gas line of the old deck, in every file: the history paragraph
    # of the design record describes it without the number.
    "gasline": "XGI0M1wuNVxzKk1NU0NGRFxi",
}
# Withdrawn figures, checked in results only, because the design record may name
# them once in a short history paragraph. The currency signs are escaped, so the
# currency figures match, and the two bare sums are skipped when a percent sign
# follows, since the withdrawn sums are never percentages (a pilot state prints a
# capture of the same digits).
WITHDRAWN = (
    "U1wkXHM/OTZcYnxTXCRccz81NDZNfFVTXCRccz84MC90fFxiMTA2XC41XGIoPyFccyolKXxc"
    "Yjk5XC40XGIoPyFccyolKQ=="
)


# Working notes and keys that never ship, matched without case, as in
# engine/plant/tests/no_internal_pointers.rs. `via_old_repo` is a source_status value
# of the parameter schema, so it is removed before the match.
POINTER_SCOPE = ("README.md", "docs/", "references/")
BINARY_SUFFIX = (".pdf", ".docx", ".pptx", ".png", ".jpg", ".jpeg", ".gif", ".wasm", ".ico")
POINTERS = (
    "plan section", "screening script", "screening ledger", "smr_wgs", "critic.md",
    "audit.md", "inventory.md", "decision s8", "item m14", "inl.md", "econ_lit",
    "htgr_ihx.md", "htgr_calc", "amine_spec", "mea_columns", "mea_thermo", "mea_calc",
    "final_v1_7", "inl_calcs", "old repo", "old_repo", "old-repo", "audit_recompute",
    "freeze section", "freeze s", "screening.py", "user decision", "user_answers",
    "rest row", "deck slide", "q8_check", "qproc_kcheck", "build order section",
)
LINKS = re.compile(r"https?://\S+|\b10\.\d{4,9}/\S+")


def working_key(word):
    """A single-letter key of the working notes, such as s6, s9h or Q3."""
    return re.fullmatch(r"[sQ]\d+[a-z]?", word) is not None


def pointer_hits(name, text):
    out = []
    for i, line in enumerate(text.splitlines(), 1):
        low = line.lower().replace("via_old_repo", "")
        for p in POINTERS:
            if p in low:
                out.append(f"pointer: {name}:{i}: {p}")
        for word in re.split(r"[^A-Za-z0-9]", LINKS.sub(" ", line)):
            if working_key(word):
                out.append(f"pointer: {name}:{i}: working key {word}")
    return out


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
        data = p.read_bytes()
        text = data.decode("utf-8", "replace")
        # Compressed binaries (the published PDF and Office files, images) can hold any byte
        # sequence, so they get the local-path rules only; the pre-push scan opens them.
        binary = b"\0" in data[:8000] or f.lower().endswith(BINARY_SUFFIX)
        for label, rx in rules.items():
            if binary and label not in ("path", "profile"):
                continue
            m = rx.search(text)
            if m:
                hits.append(f"{label}: {f}: offset {m.start()}")
        if f.startswith("results/"):
            m = withdrawn.search(text)
            if m:
                hits.append(f"withdrawn: {f}: offset {m.start()}")
        if f.startswith(POINTER_SCOPE) and f.endswith(".md"):
            hits.extend(pointer_hits(f, text))
    print(f"scanned {len(names)} files, {len(hits)} hits")
    for h in hits:
        print(h)
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
