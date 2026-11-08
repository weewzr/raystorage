import sys
import docx
sys.stdout.reconfigure(encoding="utf-8")
W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"; M = "{http://schemas.openxmlformats.org/officeDocument/2006/math}"
d = docx.Document(sys.argv[1])
def txt(el):
    return "".join((x.text or "") for x in el.iter() if x.tag in (W + "t", M + "t"))
for el in d.element.body.iterchildren():
    if el.tag == W + "p":
        t = txt(el)
        if t.strip(): print("P:", t)
    elif el.tag == W + "tbl":
        for tr in el.iter(W + "tr"):
            print("T:", " | ".join(txt(tc) for tc in tr.iter(W + "tc")))
