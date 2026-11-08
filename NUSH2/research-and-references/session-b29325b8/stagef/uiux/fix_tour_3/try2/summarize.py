"""Print a short summary of the try2 final logs: the self test verdict lines and
the last lines of each tool log. Usage: python summarize.py <final dir> [names...]"""
import glob
import os
import re
import sys

SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
d = sys.argv[1]
want = sys.argv[2:]
for f in sorted(glob.glob(os.path.join(d, "*.log"))):
    n = os.path.basename(f)
    if want and not any(n.startswith(w) for w in want):
        continue
    t = open(f, encoding="utf-8", errors="replace").read().replace(SP, "SP").splitlines()
    if n.startswith(("selftest", "planted")):
        npass = sum(1 for l in t if l.startswith("PASS "))
        nfail = sum(1 for l in t if l.startswith("FAIL "))
        nskip = sum(1 for l in t if l.startswith("SKIP "))
        print(f"== {n}: {npass} PASS, {nskip} SKIP, {nfail} FAIL; " + " | ".join(l[:120] for l in t if l.startswith(("site commit", "browser:", "exit"))))
        for l in t:
            if l.startswith("FAIL ") or re.match(r"(PASS|FAIL) tour_(auto_stays|head_shown|focus_kept)", l):
                print("   ", l[:700])
    else:
        print(f"== {n} ({len(t)} lines)")
        for l in t[-4:]:
            print("   ", l[:400])
