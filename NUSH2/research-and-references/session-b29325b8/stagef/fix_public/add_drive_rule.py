"""Add a generic text-only drive and home path rule to scripts/check_clean.py."""
import base64
import re
import sys
from pathlib import Path

p = Path(sys.argv[1]) / "scripts" / "check_clean.py"
s = p.read_text(encoding="utf-8")
BS = chr(92)
drive = (r"(?<![\w$" + BS + BS + r"])[A-Za-z]:[" + BS + BS + r"/]+(?=[A-Za-z0-9_$~.-])"
         r"|(?<![\w.])/(?:Users|home)/[A-Za-z0-9_.-]+")
rx = re.compile(drive)
good = ["C:" + BS + "Users" + BS + "x", "D:/data/file", "E:" + BS + BS + "share", "/Users/someone",
        "/home/someone", "path C:/a b"]
bad = ["https://example.org/home/x", "ratio 3:/4", "a:b", "http://x", "eq: X.1", "S$15/GJ",
       "z=Z_c:" + BS + " L_n", "T_c:/x", BS + "a:/x"]
for g in good:
    assert rx.search(g), g
for b in bad:
    assert not rx.search(b), b
enc = base64.b64encode(drive.encode()).decode()
assert base64.b64decode(enc).decode() == drive
if '"drive":' in s:
    raise SystemExit("already present")
a = '    "profile": "KD9pOkFwcERhdGF8c2NyYXRjaHBhZCk=",\n'
assert s.count(a) == 1
s = s.replace(a, a + "    # Any absolute drive or home path, in text files only, because compressed bytes can\n"
                     "    # match a pattern this short.\n"
                     f'    "drive": "{enc}",\n')
p.write_text(s, encoding="utf-8", newline="")
print("pattern:", drive)
