import sys, re
import pymupdf
from pptx import Presentation
sys.stdout.reconfigure(encoding="utf-8")
V = sys.argv[1]
prs = Presentation(V + "/deck_copy.pptx")
titles, notes = [], []
for s in prs.slides:
    t = s.shapes.title.text_frame.text if s.shapes.title is not None else ""
    titles.append(" ".join(t.split()))
    notes.append(s.notes_slide.notes_text_frame.text if s.has_notes_slide else "")
doc = pymupdf.open(V + "/deck.pdf")
ptext = [" ".join(p.get_text().split()) for p in doc]
lines = open(V + "/script.txt", encoding="utf-8").read().splitlines()
# running order
ro = [l for l in lines if re.match(r"T: \d+ \|", l)]
bad = 0
for l in ro:
    parts = [p.strip() for p in l[3:].split("|")]
    n = int(parts[0]); t = parts[1]
    if n <= 56:
        if " ".join(t.split()) != titles[n - 1]:
            bad += 1; print("TITLE MISMATCH", n, repr(t), "vs", repr(titles[n - 1]))
    else:
        if " ".join(t.split()) != titles[n - 1]:
            bad += 1; print("QA TITLE MISMATCH", n, repr(t), "vs", repr(titles[n - 1]))
print("running-order rows", len(ro), "title mismatches", bad)
# script body: "N. title" then timing line with "On screen:" then text
body = {}
for i, l in enumerate(lines):
    m = re.match(r"P: (\d+)\. (.*)$", l)
    if m and int(m.group(1)) <= 56:
        n = int(m.group(1)); body[n] = (m.group(2), lines[i + 1][3:] if i + 1 < len(lines) else "", lines[i + 2][3:] if i + 2 < len(lines) else "")
nm = 0
for n, (t, timing, text) in sorted(body.items()):
    if " ".join(t.split()) != titles[n - 1]:
        print("BODY TITLE MISMATCH", n, t, "|", titles[n - 1])
    # spoken text vs pptx notes
    spoken = text if n <= 46 else timing
    nt = " ".join(notes[n - 1].split())
    if " ".join(spoken.split()) not in nt and nt not in " ".join(spoken.split()):
        nm += 1; print("NOTES DIFFER", n, "\n   script:", spoken[:160], "\n   notes :", nt[:160])
    # on-screen numbers present on slide
    m = re.search(r"On screen: (.*)$", timing)
    if m:
        nums = re.findall(r"\d[\d,]*\.?\d*", m.group(1))
        miss = [x for x in nums if x.rstrip(".") not in ptext[n - 1]]
        if miss: print("ON-SCREEN NUMBER NOT ON SLIDE", n, miss, "|", m.group(1)[:120])
print("notes mismatches", nm)
