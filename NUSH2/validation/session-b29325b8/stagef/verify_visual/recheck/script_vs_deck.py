import sys, re, zipfile, unicodedata
from pptx import Presentation
import docx
W = sys.argv[1]
prs = Presentation(W + "/NUSH2_Final_Presentation.pptx")
titles, notes, hidden = [], [], []
for i, s in enumerate(prs.slides, 1):
    t = None
    if s.shapes.title is not None:
        t = s.shapes.title.text_frame.text
    titles.append((t or "").replace("\n", " ").replace("\v", " "))
    notes.append(s.notes_slide.notes_text_frame.text if s.has_notes_slide else "")
    if s._element.get("show") == "0": hidden.append(i)
print("slides", len(titles), "hidden", hidden)
z = zipfile.ZipFile(W + "/NUSH2_Final_Presentation.pptx")
print("show=0 in slide xml:", [n for n in z.namelist() if n.startswith("ppt/slides/slide") and b'show="0"' in z.read(n)])
d = docx.Document(W + "/NUSH2_Speaker_Script.docx")
# running order table
tb = d.tables[0]
rows = [[c.text for c in r.cells] for r in tb.rows]
mism = 0
for r in rows[1:]:
    if r[0].strip().isdigit():
        n = int(r[0]); st = titles[n-1]
        if r[1].strip() != st.strip():
            mism += 1; print("TABLE MISMATCH", n, repr(r[1]), "vs", repr(st))
print("table rows", len(rows)-1, "mismatches", mism)
# headings
heads = [p.text for p in d.paragraphs if re.match(r"^\d+\. ", p.text) and p.style.name.lower().startswith("heading")]
hm = 0
for h in heads:
    n, t = h.split(". ", 1); n = int(n)
    if t.strip() != titles[n-1].strip():
        hm += 1; print("HEAD MISMATCH", n, repr(t), "vs", repr(titles[n-1]))
print("headings", len(heads), "mismatches", hm)
# non-ascii hyphen check in titles
for n, t in enumerate(titles, 1):
    odd = [hex(ord(c)) for c in t if ord(c) in (0x2011, 0x2010, 0x00ad, 0x2013, 0x2014)]
    if odd: print("slide", n, "title special chars", odd)
for h in heads:
    odd = [hex(ord(c)) for c in h if ord(c) in (0x2011, 0x2010, 0x00ad, 0x2013, 0x2014)]
    if odd: print("heading", h[:30], "special chars", odd)
# Q&A list in script
qa = [titles[i] for i in range(56, 72)]
txt = "\n".join(p.text for p in d.paragraphs) + "\n" + "\n".join(c.text for t in d.tables for r in t.rows for c in r.cells)
for i, t in enumerate(qa, 57):
    if t not in txt: print("QA title not in script", i, t)
# notes vs script body: compare normalised
def norm(s): return re.sub(r"\s+", " ", unicodedata.normalize("NFKC", s)).strip()
body = norm(txt)
miss = 0
for n in range(1, 57):
    nt = norm(notes[n-1])
    if not nt: print("no notes", n); continue
    if nt not in body:
        miss += 1
        # find first differing sentence
        for sent in re.split(r"(?<=[.!?]) ", nt):
            if sent not in body:
                print("NOTE TEXT NOT IN SCRIPT", n, repr(sent[:120])); break
print("note mismatches", miss)
open(W + "/titles.txt", "w", encoding="utf-8").write("\n".join(f"{i}\t{t}" for i, t in enumerate(titles, 1)))
open(W + "/notes.txt", "w", encoding="utf-8").write("\n".join(f"=== n{i}\n{t}" for i, t in enumerate(notes, 1)))
