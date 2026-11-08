#!/usr/bin/env python3
"""Plan step 2, test T0: a skeleton docx with cover section, TOC field, two pages of text, a
table, an image, an equation and twenty unit objects, used to prove the PDF route."""
import sys
from pathlib import Path
from docx import Document
from docx.enum.section import WD_SECTION
from docx.shared import Mm, Pt
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
import omml

B = Path(__file__).resolve().parent
doc = Document()
s = doc.sections[0]
s.page_width, s.page_height = Mm(210), Mm(297)
st = doc.styles["Normal"]
st.font.name = "Times New Roman"
st.font.size = Pt(12)
doc.add_paragraph("Cover")
p = doc.add_paragraph()
for kind, text in (("begin", None), (None, ' TOC \\o "1-2" \\h \\z \\u '), ("separate", None), ("end", None)):
    r = p.add_run()
    if kind:
        f = OxmlElement("w:fldChar")
        f.set(qn("w:fldCharType"), kind)
        r._r.append(f)
    else:
        i = OxmlElement("w:instrText")
        i.text = text
        i.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
        r._r.append(i)
doc.add_section(WD_SECTION.NEW_PAGE)
doc.add_heading("Heading one", 1)
for i in range(60):
    p = doc.add_paragraph("Body text with a unit ")
    p._p.append(omml.omath(r"\mathrm{MW_{th}}"))
    p.add_run(" and another ")
    p._p.append(omml.omath(r"\mathrm{Nm^{3}\ h^{-1}}"))
    p.add_run(" in a sentence long enough to fill the page width and wrap at least once here.")
p = doc.add_paragraph()
p._p.append(omml.omath_para(r"\mathrm{CRF} = \frac{r}{1 - \left(1 + r\right)^{-L}}"))
t = doc.add_table(rows=2, cols=2)
t.cell(0, 0).text = "A"
doc.add_picture(str(B / "media" / "R_image4.png"), width=Mm(50))
doc.save(sys.argv[1])
