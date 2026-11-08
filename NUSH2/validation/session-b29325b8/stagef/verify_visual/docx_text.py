import sys
import docx
sys.stdout.reconfigure(encoding="utf-8")
d = docx.Document(sys.argv[1])
cp = d.core_properties
print("CORE:", repr(cp.author), repr(cp.last_modified_by), repr(cp.title), repr(cp.subject), repr(cp.keywords), repr(cp.comments))
from docx.oxml.ns import qn
body = d.element.body
for el in body.iterchildren():
    if el.tag == qn("w:p"):
        t = "".join(x.text or "" for x in el.iter(qn("w:t")))
        if t.strip(): print("P:", t)
    elif el.tag == qn("w:tbl"):
        for tr in el.iter(qn("w:tr")):
            cells = ["".join(x.text or "" for x in tc.iter(qn("w:t"))) for tc in tr.iter(qn("w:tc"))]
            print("T:", " | ".join(cells))
