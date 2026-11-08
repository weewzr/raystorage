"""Replace the clear-text people, owner, course and path patterns in qa_report.py with hashed and
encoded forms, computed from the patterns themselves so no name is typed here."""
import base64
import hashlib
import re
import sys
from pathlib import Path

B = Path(sys.argv[1])
p = B / "qa_report.py"
s = p.read_text(encoding="utf-8")
BS = "\\"

people_m = re.search(r'"people": r"([^"]+)",?', s)
owner_m = re.search(r'"owner": r"([^"]+)",?\s*', s)
course_m = re.search(r'    "course": r"([^"]+)",\n', s)
path_m = re.search(r'    "path": r"[^"]+",\n', s)
assert people_m and owner_m and course_m and path_m

clean = people_m.group(1).replace("(?i:", "").replace(")", "").replace(BS + "b", "")
words, phrases = set(), set()
for alt in clean.split("|"):
    ws = alt.split()
    h = hashlib.sha256(" ".join(ws).lower().encode()).hexdigest()
    (words if len(ws) == 1 else phrases).add(h)
owner = owner_m.group(1).replace(BS + "b", "")
owner_h = hashlib.sha256(owner.encode()).hexdigest()
deck = sorted(set(re.findall(r'"([0-9a-f]{64})"', (B / "run_all.py").read_text(encoding="utf-8"))))
assert words <= set(deck), "deck hash set does not cover the people words"

s = s.replace(people_m.group(0), "")
s = s.replace(owner_m.group(0), "")
course_b64 = base64.b64encode(course_m.group(1).encode()).decode()
s = s.replace(course_m.group(0), '    "course": base64.b64decode("' + course_b64 + '").decode(),\n')
new_path = '    "path": r"(?i:' + BS + 'b[A-Z]:[' + BS + BS + '/]|/c/Users' + BS + 'b|' + BS + BS + 'Users' + BS + BS + '|AppData|scratchpad)",\n'
s = s.replace(path_m.group(0), new_path)

block = '''# People other than the author, and the local account, are checked by SHA-256 of lower-case words and
# word triples (the account as written), so this file names nobody. The word set is the deck builder's.
NAME_WORD_HASHES = {''' + ", ".join(f'"{h}"' for h in deck) + '''}
NAME_PHRASE_HASHES = {''' + ", ".join(f'"{h}"' for h in sorted(phrases)) + '''}
OWNER_HASHES = {"''' + owner_h + '''"}


def name_hits(text):
    """Positions of words or word triples whose hash marks a person other than the author."""
    words = [(m.start(), m.group(0)) for m in re.finditer(r"[A-Za-z]+", text)]

    def sha(w):
        return hashlib.sha256(w.encode()).hexdigest()
    hits = [pos for pos, w in words if sha(w.lower()) in NAME_WORD_HASHES or sha(w) in OWNER_HASHES]
    for i in range(len(words) - 2):
        if sha(" ".join(w for _, w in words[i:i + 3]).lower()) in NAME_PHRASE_HASHES:
            hits.append(words[i][0])
    return hits


'''
anchor = "# The S$96/t solve point of MATURE_60Y"
assert s.count(anchor) == 1
s = s.replace(anchor, block + anchor)
s = s.replace("import csv\nimport datetime as dt\n", "import base64\nimport csv\nimport datetime as dt\nimport hashlib\n", 1)

# Check 4b: names are also looked for in attribute values (theme typefaces, docPr names), where the
# tag-stripped text never sees them.
old = '''        for name, rx in PRODUCT_RX.items():
            m = re.search(rx, plain_t)
            if m:
                phits.append(f"{name}: {label}: {plain_t[max(0, m.start() - 30):m.end() + 30]!r}")
'''
new = '''        for name, rx in PRODUCT_RX.items():
            m = re.search(rx, plain_t)
            if m:
                phits.append(f"{name}: {label}: {plain_t[max(0, m.start() - 30):m.end() + 30]!r}")
        attrs = " ".join(re.findall(r'="([^"]*)"', t)) if label.endswith((".xml", ".rels")) else ""
        for where, blob in (("text", plain_t), ("attribute", attrs)):
            hits_n = name_hits(blob)
            if hits_n:
                phits.append(f"people: {label}: {len(hits_n)} {where} hit(s) at offsets {hits_n[:3]}")
'''
assert s.count(old) == 1
s = s.replace(old, new)
p.write_text(s, encoding="utf-8")
print("patched: words", len(words), "phrases", len(phrases))
