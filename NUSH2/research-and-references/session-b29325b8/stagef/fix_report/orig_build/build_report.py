#!/usr/bin/env python3
"""Plan step 7. Build the report docx from content/*.md, tables.py and numbers_resolved.json.

Content syntax:
  # H1, ## H2                    headings
  {{TOKEN|mods}}                 a number; mods: t (table format), fN (N decimals), u (with its
                                 unit), u:KEY (with the unit KEY from units.csv)
  [[KEY]]                        a unit as native Word math
  $...$                          inline math; $$...$$ {eq:key} a numbered equation
  [@a,b]                         citation; {fig:x} {tab:x} {eq:x} cross references
  **text**                       bold run; ~ a non-breaking space
  !!figure key, !!table key, !!appendix L   block directives

The script runs twice over the content: the first pass fixes citation and label numbers.
"""
import csv
import json
import os
import re
import sys
from pathlib import Path

from docx import Document
from docx.enum.section import WD_SECTION
from docx.enum.style import WD_STYLE_TYPE
from docx.enum.table import WD_TABLE_ALIGNMENT
from docx.enum.text import WD_ALIGN_PARAGRAPH, WD_LINE_SPACING
from docx.oxml import OxmlElement
from docx.oxml.ns import qn
from docx.shared import Mm, Pt, RGBColor
from PIL import Image

import omml
import tables as table_specs

B = Path(__file__).resolve().parent
ROOT = B.parents[1]
PRES = ROOT / "presentation" / "figures"
MEDIA = B / "media"
OUT = Path(os.environ.get("NUSH2_RAW_DOCX", B / "tmp" / "report_raw.docx"))
RESOLVED = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))
NAVY = RGBColor(0x1F, 0x38, 0x64)
PAGE_W = 159.2
FONT = "Times New Roman"
AUTHOR = "Wee Zhern Ray"
CREDIT = ("OUTRAM PARK was consulted as a reference. No OUTRAM PARK code is included and no endorsement "
          "is implied.")
TITLE = "NUS-H₂: Nuclear-Upgraded Steam Methane Reforming with Carbon Capture for Low-Carbon Hydrogen in Singapore"

FIGS = {
    "f_heat": dict(path="fig_heat.png", width=112,
                   caption="Process heat temperature ranges of industrial applications, with the ranges that light-water reactors (LWRs) and HTGRs can serve [@r6].",
                   alt="Bar chart of process temperature ranges with LWR and HTR arrows."),
    "f_pfd": dict(path=str(PRES / "Figure_2_NUS_H2_210MWth.png"), width=159,
                  caption="NUS-H₂ arrangement. A {{R_RATED_MWTH|f0|u}} HTGR supplies {{Q_PROCESS_B_MWTH|u}} of process heat through an IHX and secondary helium circuit. The dashed line is the boundary: CO₂ compression and ship offshore storage sit inside it, the external PSA sits outside, and its reject returns across it.",
                  alt="Process flow diagram of the reactor, reformer, shift, capture, recycle and external PSA."),
    "f_epz": dict(path="V_image3.png", width=98,
                  caption="How an EPZ is set from dose. Each accident sequence is plotted by frequency and 30-day dose at 400~[[m]], the Xe-100 example, and must fall below the frequency-consequence target. Adapted from the Xe-100 construction permit application [@r17].",
                  alt="Frequency against dose plot with the target line."),
    "f_road": dict(path=str(PRES / "Figure_Roadmap_NUS_H2.png"), width=159,
                   caption="Indicative roadmap with milestones, stakeholders and enabling requirements. All dates are illustrative.",
                   alt="Roadmap from 2027 to 2050 in four phases."),
    "f_step": dict(path="fig_stepdown.png", width=136,
                   caption="Step-down from the base full cost to the MATURE_60Y scenario. Every bar is an engine row, ledger L5 with the DCF beside it. The dotted line marks the {{TARGET_MATURE_COST|f0|u}} point that sets the printed gas price of {{STEPDOWN_4_GAS_S_GJ|u}}, against the central {{STEPDOWN_1_GAS_S_GJ|f0|u}}.",
                   alt="Paired bars of ledger and DCF cost for four steps falling below the target line."),
    "f_grid": dict(path="fig_cost_grid.png", width=150,
                   caption="Rebuilt full cost L5 for the three size treatments at equal weight, against the {{TARGET_COST_S_T|f0|u}} target.",
                   alt="Grouped bars of full cost at BOAK and FOAK for three size treatments with the target line."),
    "f_ledger25": dict(path="fig_ledger_25y.png", width=150,
                       caption="Cost ladder at BOAK, 25~[[y]], in [[S$/tCO2e]], with L3 as a named step.",
                       alt="Waterfall of ledger lines at 25 years."),
    "f_ledger60": dict(path="fig_ledger_60y.png", width=150,
                       caption="Cost ladder at BOAK, 60~[[y]], in [[S$/tCO2e]], with L3 as a named step.",
                       alt="Waterfall of ledger lines at 60 years."),
}


def grouped(v, dp):
    r = round(v, dp)
    return f"{(0.0 if r == 0 else r):,.{dp}f}"


class Ctx:
    def __init__(self):
        res = json.loads(RESOLVED.read_text(encoding="utf-8"))
        self.tokens = res["tokens"]
        self.units = {r["token"]: (r["latex"], r["style"]) for r in csv.DictReader(open(B / "units.csv", encoding="utf-8"))}
        self.refs = {r["key"]: r for r in csv.DictReader(open(B / "refs.csv", encoding="utf-8"))}
        self.cites = {}
        self.labels = {}
        self.specs = table_specs.build()
        self.used = set()
        self.reset()

    def reset(self):
        self.fig_n = self.tab_n = self.eq_n = 0
        self.chapter = "0"
        self.prefix = ""

    def cite(self, keys):
        nums = []
        for k in keys.split(","):
            if k not in self.refs:
                raise SystemExit(f"unknown reference key {k}")
            self.cites.setdefault(k, len(self.cites) + 1)
            nums.append(self.cites[k])
        return "[" + ",".join(str(n) for n in sorted(nums)) + "]"

    def unit_math(self, key, value=None):
        if key not in self.units:
            raise SystemExit(f"unit {key} not in units.csv")
        latex, style = self.units[key]
        if style == "suffix":
            return f"${latex}$"
        lead = {"money": r"S\$", "usmoney": r"US\$", "euromoney": r"EUR\ "}[style]
        val = "" if value is None else value
        sign = ""
        if val.startswith("−"):
            sign, val = "−", val[1:]
        return "$\\mathrm{" + sign + lead + val + latex + "}$"

    def token(self, m):
        name, mods = m.group(1), [x for x in m.group(2).split("|") if x]
        if name not in self.tokens:
            raise SystemExit(f"unknown token {name}")
        tok = self.tokens[name]
        self.used.add(name)
        text = tok["text"]
        unit = None
        for mod in mods:
            if mod == "t":
                text = tok["table"]
            elif re.fullmatch(r"f\d", mod):
                text = grouped(tok["value"], int(mod[1]))
            elif mod == "u":
                unit = tok["unit"]
            elif mod.startswith("u:"):
                unit = mod[2:]
            else:
                raise SystemExit(f"unknown modifier {mod} on {name}")
        if isinstance(tok["value"], (int, float)):
            text = text.replace("-", "−")
        if unit is None:
            return text
        if unit == "%":
            return text + "%"
        if unit not in self.units:
            raise SystemExit(f"token {name} unit {unit} not in units.csv")
        latex, style = self.units[unit]
        if style == "suffix":
            return f"{text}~${latex}$"
        return self.unit_math(unit, text)

    def ref_label(self, kind, key):
        return self.labels.get((kind, key), f"{kind}?")

    def expand(self, s):
        s = re.sub(r"\{\{([A-Za-z0-9_\-]+)((?:\|[^|}]+)*)\}\}", self.token, s)
        s = re.sub(r"\[\[([^\]]+)\]\]", lambda m: self.unit_math(m.group(1)), s)
        s = re.sub(r"\[@([A-Za-z0-9_,]+)\]", lambda m: self.cite(m.group(1)), s)
        s = re.sub(r"\{(fig|tab|eq):([a-z0-9_]+)\}", lambda m: self.ref_label(m.group(1), m.group(2)), s)
        return s


SPLIT = re.compile(r"(\$(?:\\.|[^$\\])+\$)|(\*\*.+?\*\*)")


def add_inline(par, text, ctx, bold=False, expanded=False):
    s = text if expanded else ctx.expand(text)
    pos = 0

    def plain(chunk):
        for i, piece in enumerate(chunk.split("\n")):
            if i:
                par.add_run().add_break()
            if piece:
                r = par.add_run(piece.replace("~", " "))
                r.bold = True if bold else None

    for m in SPLIT.finditer(s):
        plain(s[pos:m.start()])
        if m.group(1):
            par._p.append(omml.omath(m.group(1)[1:-1], 24, bold))
        else:
            add_inline(par, m.group(2)[2:-2], ctx, True, True)
        pos = m.end()
    plain(s[pos:])


def set_cell_border(cell, edge, sz=8):
    tcPr = cell._tc.get_or_add_tcPr()
    borders = tcPr.find(qn("w:tcBorders"))
    if borders is None:
        borders = OxmlElement("w:tcBorders")
        tcPr.append(borders)
    el = OxmlElement(f"w:{edge}")
    el.set(qn("w:val"), "single")
    el.set(qn("w:sz"), str(sz))
    el.set(qn("w:space"), "0")
    el.set(qn("w:color"), "000000")
    borders.append(el)


def clear_tbl_style(table, widths, left_right=60, center=False):
    tblPr = table._tbl.tblPr
    for name in ("w:tblStyle",):
        el = tblPr.find(qn(name))
        if el is not None:
            tblPr.remove(el)
    layout = OxmlElement("w:tblLayout")
    layout.set(qn("w:type"), "fixed")
    tblPr.append(layout)
    mar = OxmlElement("w:tblCellMar")
    for side, val in (("top", 0), ("left", left_right), ("bottom", 0), ("right", left_right)):
        e = OxmlElement(f"w:{side}")
        e.set(qn("w:w"), str(val))
        e.set(qn("w:type"), "dxa")
        mar.append(e)
    tblPr.append(mar)
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    grid = table._tbl.tblGrid
    for col, w in zip(grid.findall(qn("w:gridCol")), widths):
        col.set(qn("w:w"), str(int(w * 56.7)))
    for row in table.rows:
        for cell, w in zip(row.cells, widths):
            cell.width = Mm(w)


def add_table(doc, spec, ctx):
    ncol = len(spec["widths"])
    header = spec["header"]
    rows = spec["rows"]
    table = doc.add_table(rows=len(header) + len(rows), cols=ncol)
    clear_tbl_style(table, spec["widths"])
    align = spec.get("align", "l" * ncol)
    bold_rows = set(spec.get("bold", []))
    rule_rows = set(spec.get("rule", []))
    keep = spec.get("keep", True)
    amap = {"l": WD_ALIGN_PARAGRAPH.LEFT, "r": WD_ALIGN_PARAGRAPH.RIGHT, "c": WD_ALIGN_PARAGRAPH.CENTER}
    for i, row in enumerate(table.rows):
        trPr = row._tr.get_or_add_trPr()
        trPr.append(OxmlElement("w:cantSplit"))
        is_head = i < len(header)
        if is_head:
            trPr.append(OxmlElement("w:tblHeader"))
        cells = header[i] if is_head else rows[i - len(header)]
        ridx = i - len(header)
        for j, cell in enumerate(row.cells):
            par = cell.paragraphs[0]
            par.style = doc.styles["Table Text"]
            par.alignment = amap[align[j]]
            add_inline(par, cells[j], ctx, bold=is_head or ridx in bold_rows)
            if keep and i < len(table.rows) - 1 or is_head:
                par.paragraph_format.keep_with_next = True
            if i == 0:
                set_cell_border(cell, "top", 12)
            if i == len(header) - 1:
                set_cell_border(cell, "bottom", 6)
            if not is_head and ridx in rule_rows:
                set_cell_border(cell, "top", 4)
            if i == len(table.rows) - 1:
                set_cell_border(cell, "bottom", 12)
    return table


def add_field(par, instr, placeholder=" "):
    def run_with(el):
        r = par.add_run()
        r._r.append(el)
        return r

    a = OxmlElement("w:fldChar")
    a.set(qn("w:fldCharType"), "begin")
    run_with(a)
    i = OxmlElement("w:instrText")
    i.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
    i.text = instr
    run_with(i)
    b = OxmlElement("w:fldChar")
    b.set(qn("w:fldCharType"), "separate")
    run_with(b)
    par.add_run(placeholder)
    c = OxmlElement("w:fldChar")
    c.set(qn("w:fldCharType"), "end")
    run_with(c)


def setup_styles(doc):
    styles = doc.styles
    el = styles.element
    defaults = el.find(qn("w:docDefaults"))
    rpr = defaults.find(qn("w:rPrDefault")).find(qn("w:rPr"))
    for child in list(rpr):
        rpr.remove(child)
    fonts = OxmlElement("w:rFonts")
    for a in ("ascii", "hAnsi", "cs", "eastAsia"):
        fonts.set(qn(f"w:{a}"), FONT)
    rpr.append(fonts)
    for tag, val in (("w:sz", "24"), ("w:szCs", "24")):
        e = OxmlElement(tag)
        e.set(qn("w:val"), val)
        rpr.append(e)
    lang = OxmlElement("w:lang")
    lang.set(qn("w:val"), "en-GB")
    lang.set(qn("w:eastAsia"), "en-GB")
    lang.set(qn("w:bidi"), "ar-SA")
    rpr.append(lang)

    def fix(style, size, bold=False, color=None):
        rp = style.element.get_or_add_rPr()
        for child in list(rp):
            rp.remove(child)
        f = OxmlElement("w:rFonts")
        for a in ("ascii", "hAnsi", "cs", "eastAsia"):
            f.set(qn(f"w:{a}"), FONT)
        rp.append(f)
        style.font.size = Pt(size)
        style.font.bold = bold
        if color is not None:
            style.font.color.rgb = color

    normal = styles["Normal"]
    fix(normal, 12)
    pf = normal.paragraph_format
    # Word's 1.5 lines for Times New Roman 12 pt is 20.7 pt. Exact spacing stops inline math
    # from opening individual lines, per the plan's line-height fallback.
    pf.line_spacing = Pt(20.7)
    pf.line_spacing_rule = WD_LINE_SPACING.EXACTLY
    pf.space_after = Pt(6)
    pf.space_before = Pt(0)
    pf.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
    pf.widow_control = True
    for name, size, before, after in (("Heading 1", 14, 12, 6), ("Heading 2", 12, 10, 4)):
        s = styles[name]
        fix(s, size, True, NAVY)
        s.paragraph_format.space_before = Pt(before)
        s.paragraph_format.space_after = Pt(after)
        s.paragraph_format.keep_with_next = True
        s.paragraph_format.line_spacing = 1.0
        s.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    cap = styles.add_style("Report Caption", WD_STYLE_TYPE.PARAGRAPH)
    cap.base_style = normal
    fix(cap, 12)
    cap.paragraph_format.line_spacing = 1.0
    cap.paragraph_format.space_before = Pt(4)
    cap.paragraph_format.space_after = Pt(6)
    cap.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    tt = styles.add_style("Table Text", WD_STYLE_TYPE.PARAGRAPH)
    tt.base_style = normal
    fix(tt, 12)
    tt.paragraph_format.line_spacing = 1.0
    tt.paragraph_format.space_after = Pt(0)
    tt.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    ref = styles.add_style("Reference", WD_STYLE_TYPE.PARAGRAPH)
    ref.base_style = normal
    fix(ref, 12)
    ref.paragraph_format.line_spacing = 1.0
    ref.paragraph_format.space_after = Pt(4)
    ref.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    ref.paragraph_format.left_indent = Mm(9)
    ref.paragraph_format.first_line_indent = Mm(-9)
    for name, indent in (("toc 1", 0), ("toc 2", 6)):
        t = styles.add_style(name, WD_STYLE_TYPE.PARAGRAPH, builtin=True)
        t.base_style = normal
        fix(t, 12)
        t.paragraph_format.line_spacing = 1.0
        t.paragraph_format.space_after = Pt(3)
        t.paragraph_format.left_indent = Mm(indent)
        t.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    head = styles.add_style("Front Heading", WD_STYLE_TYPE.PARAGRAPH)
    head.base_style = normal
    fix(head, 14, True, NAVY)
    head.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.LEFT
    head.paragraph_format.space_after = Pt(10)
    cov = styles.add_style("Cover", WD_STYLE_TYPE.PARAGRAPH)
    cov.base_style = normal
    fix(cov, 12)
    cov.paragraph_format.alignment = WD_ALIGN_PARAGRAPH.CENTER
    cov.paragraph_format.line_spacing = 1.0
    cov.paragraph_format.space_after = Pt(6)


def set_page(sec):
    sec.page_width, sec.page_height = Mm(210), Mm(297)
    sec.left_margin = sec.right_margin = sec.top_margin = sec.bottom_margin = Mm(25.4)
    sec.header_distance = sec.footer_distance = Mm(12.7)


def page_number_footer(sec):
    sec.footer.is_linked_to_previous = False
    par = sec.footer.paragraphs[0]
    par.alignment = WD_ALIGN_PARAGRAPH.CENTER
    par.paragraph_format.line_spacing = 1.0
    add_field(par, " PAGE ", "1")


def cover(doc):
    """Cover lines: the title, the author, the licence and the reference credit, and nothing else."""
    p = doc.add_paragraph(style="Cover")
    p.paragraph_format.space_before = Pt(150)
    r = p.add_run(TITLE)
    r.font.size = Pt(20)
    r.bold = True
    r.font.color.rgb = NAVY
    p.paragraph_format.space_after = Pt(60)
    for text, size, after in ((AUTHOR, 14, 28), ("MIT licence", 12, 10), (CREDIT, 11, 0)):
        p = doc.add_paragraph(style="Cover")
        p.add_run(text).font.size = Pt(size)
        p.paragraph_format.space_after = Pt(after)


def add_figure(doc, key, ctx):
    fig = FIGS[key]
    path = Path(fig["path"]) if Path(fig["path"]).is_absolute() else MEDIA / fig["path"]
    ctx.fig_n += 1
    label = f"Figure {ctx.prefix}{ctx.fig_n}"
    ctx.labels[("fig", key)] = label
    par = doc.add_paragraph()
    par.alignment = WD_ALIGN_PARAGRAPH.CENTER
    par.paragraph_format.keep_with_next = True
    par.paragraph_format.line_spacing = 1.0
    par.paragraph_format.space_before = Pt(4)
    par.paragraph_format.space_after = Pt(2)
    par.add_run().add_picture(str(path), width=Mm(fig["width"]))
    docpr = par._p.xpath(".//wp:docPr")[0]
    docpr.set("descr", fig["alt"])
    docpr.set("name", label)
    cap = doc.add_paragraph(style="Report Caption")
    r = cap.add_run(label + ". ")
    r.bold = True
    add_inline(cap, fig["caption"], ctx)
    cap.paragraph_format.space_after = Pt(8)


def add_table_block(doc, key, ctx):
    spec = ctx.specs[key]
    ctx.tab_n += 1
    label = f"Table {ctx.prefix}{ctx.tab_n}"
    ctx.labels[("tab", key)] = label
    cap = doc.add_paragraph(style="Report Caption")
    cap.paragraph_format.keep_with_next = True
    cap.paragraph_format.space_before = Pt(6)
    cap.paragraph_format.space_after = Pt(3)
    r = cap.add_run(label + ". ")
    r.bold = True
    add_inline(cap, spec["caption"], ctx)
    add_table(doc, spec, ctx)
    gap = doc.add_paragraph()
    gap.paragraph_format.line_spacing = Pt(8)
    gap.paragraph_format.line_spacing_rule = WD_LINE_SPACING.EXACTLY
    gap.paragraph_format.space_after = Pt(0)


def add_equation(doc, latex, key, ctx):
    ctx.eq_n += 1
    label = f"({ctx.chapter}.{ctx.eq_n})"
    ctx.labels[("eq", key)] = label
    table = doc.add_table(rows=1, cols=2)
    clear_tbl_style(table, [PAGE_W - 18, 18], 0)
    left, right = table.rows[0].cells
    p = left.paragraphs[0]
    p.style = doc.styles["Table Text"]
    p.paragraph_format.space_before = Pt(2)
    p.paragraph_format.space_after = Pt(4)
    p._p.append(omml.omath_para(ctx.expand(latex)))
    q = right.paragraphs[0]
    q.style = doc.styles["Table Text"]
    q.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    q.add_run(label)
    tcPr = right._tc.get_or_add_tcPr()
    v = OxmlElement("w:vAlign")
    v.set(qn("w:val"), "center")
    tcPr.append(v)


def render_file(doc, path, ctx):
    blocks = re.split(r"\n\s*\n", path.read_text(encoding="utf-8").strip())
    for block in blocks:
        block = block.strip()
        if block.startswith("!!figure"):
            add_figure(doc, block.split()[1], ctx)
        elif block.startswith("!!table"):
            add_table_block(doc, block.split()[1], ctx)
        elif block.startswith("!!appendix"):
            ctx.prefix = block.split()[1]
            ctx.fig_n = ctx.tab_n = ctx.eq_n = 0
        elif block.startswith("$$"):
            m = re.fullmatch(r"\$\$(.+?)\$\$\s*\{eq:([a-z0-9_]+)\}", block, re.S)
            if not m:
                raise SystemExit(f"bad equation block: {block[:60]}")
            add_equation(doc, m.group(1).strip(), m.group(2), ctx)
        elif block.startswith("## "):
            doc.add_heading(block[3:].strip(), 2)
        elif block.startswith("# "):
            title = block[2:].strip()
            m = re.match(r"(\d+)\.", title)
            if m:
                ctx.chapter, ctx.eq_n = m.group(1), 0
            h = doc.add_heading(title, 1)
            if title.startswith("Appendix"):
                h.paragraph_format.page_break_before = True
        else:
            par = doc.add_paragraph()
            add_inline(par, block.replace("\n", " "), ctx)


def references(doc, ctx):
    doc.add_heading("References", 1)
    order = sorted(ctx.cites, key=ctx.cites.get)
    for key in order:
        r = ctx.refs[key]
        par = doc.add_paragraph(style="Reference")
        text = f"[{ctx.cites[key]}]\t{r['text']}"
        if r["url"]:
            text += " " + r["url"]
        par.add_run(text)
        par.paragraph_format.tab_stops.add_tab_stop(Mm(9))


def compat(doc):
    settings = doc.settings.element
    c = settings.find(qn("w:compat"))
    if c is None:
        c = OxmlElement("w:compat")
        settings.append(c)
    s = OxmlElement("w:compatSetting")
    s.set(qn("w:name"), "compatibilityMode")
    s.set(qn("w:uri"), "http://schemas.microsoft.com/office/word")
    s.set(qn("w:val"), "15")
    c.append(s)


def build(ctx):
    ctx.reset()
    doc = Document()
    setup_styles(doc)
    compat(doc)
    # Remove the template's empty paragraph, if any.
    body = doc.element.body
    for p in body.findall(qn("w:p")):
        body.remove(p)
    sec = doc.sections[0]
    set_page(sec)
    cover(doc)
    par = doc.add_paragraph(style="Front Heading")
    par.paragraph_format.page_break_before = True
    par.add_run("Table of contents")
    par = doc.add_paragraph()
    par.paragraph_format.line_spacing = 1.0
    add_field(par, ' TOC \\o "1-2" \\h \\z \\u ', "Update field to show the contents.")

    s2 = doc.add_section(WD_SECTION.NEW_PAGE)
    set_page(s2)
    page_number_footer(s2)
    pg = OxmlElement("w:pgNumType")
    pg.set(qn("w:start"), "1")
    s2._sectPr.append(pg)
    for name in ("00_summary", "01_problem", "02_feasibility", "03_abatement", "04_economics", "05_discussion"):
        render_file(doc, B / "content" / f"{name}.md", ctx)

    s3 = doc.add_section(WD_SECTION.NEW_PAGE)
    set_page(s3)
    s3.footer.is_linked_to_previous = True
    for el in s3._sectPr.findall(qn("w:pgNumType")):
        s3._sectPr.remove(el)
    references(doc, ctx)
    render_file(doc, B / "content" / "10_appendices.md", ctx)

    cp = doc.core_properties
    cp.title = TITLE.replace("₂", "2")
    cp.author = AUTHOR
    cp.last_modified_by = AUTHOR
    cp.subject = "Nuclear process heat and amine capture for low-carbon hydrogen"
    cp.comments = ""
    cp.keywords = ""
    cp.category = ""
    return doc


def prepare_media():
    im = Image.open(MEDIA / "R_image2.png").convert("RGB")
    w, h = im.size
    im.crop((0, 29, w, h)).save(MEDIA / "fig_heat.png")


def main():
    prepare_media()
    ctx = Ctx()
    build(ctx)
    doc = build(ctx)
    OUT.parent.mkdir(exist_ok=True)
    doc.save(OUT)
    (B / "logs").mkdir(exist_ok=True)
    (B / "logs" / "labels.json").write_text(json.dumps({f"{k[0]}:{k[1]}": v for k, v in ctx.labels.items()}, indent=0), encoding="utf-8")
    (B / "logs" / "citations.json").write_text(json.dumps(ctx.cites, indent=0), encoding="utf-8")
    (B / "logs" / "used_tokens.txt").write_text("\n".join(sorted(ctx.used)), encoding="utf-8")
    print(f"wrote {OUT}, {len(ctx.cites)} references, {len(ctx.used)} tokens used")
    return 0


if __name__ == "__main__":
    sys.exit(main())
