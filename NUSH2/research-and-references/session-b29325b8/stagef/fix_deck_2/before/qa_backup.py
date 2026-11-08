#!/usr/bin/env python3
"""Append the 16 Q&A backup slides to the deck, in the format they were supplied in.

The source, presentation/inputs/qa_backup_source.pptx (local, git-ignored), is 13.33 x 7.5 in. Its slide
master, the one layout its slides use, its theme, pictures and chart are copied into the deck, and every
length and font size is scaled by 1.5 to the deck's 20 x 11.25 in. The slides keep their own layout, fonts
and colours, with no footer text and no slide number. The only edits are these:

- the bare abbreviation for small modular reactor is written in full;
- the theme, layout and section names that framed the source as recreated lecture slides are replaced,
  and the source's core properties and empty notes are not carried;
- slide 16 is rebuilt in its own layout from the COFUNDING rows of Sensitivity_Results.csv, through
  build/numbers.json, so it prints no withdrawn figure;
- a title that the full words push past its box is set smaller so that it stays on one line.

build_deck.py calls append() after it saves the main deck.
"""
import copy
import io
import math
import re
import zipfile
from pathlib import Path

from lxml import etree
from PIL import ImageFont
from pptx import Presentation
from pptx.chart.data import CategoryChartData
from PIL import Image as PILImage
from pptx.enum.shapes import MSO_SHAPE_TYPE, PP_PLACEHOLDER

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "inputs" / "qa_backup_source.pptx"
SCALE = 1.5
N_SLIDES = 16

A = "http://schemas.openxmlformats.org/drawingml/2006/main"
P = "http://schemas.openxmlformats.org/presentationml/2006/main"
R = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
C = "http://schemas.openxmlformats.org/drawingml/2006/chart"
PR = "http://schemas.openxmlformats.org/package/2006/relationships"
CT = "http://schemas.openxmlformats.org/package/2006/content-types"
RT = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/"
TYPES = {
    "master": "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml",
    "layout": "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml",
    "theme": "application/vnd.openxmlformats-officedocument.theme+xml",
    "slide": "application/vnd.openxmlformats-officedocument.presentationml.slide+xml",
    "chart": "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
}
DEFAULTS = {"jpg": "image/jpeg", "jpeg": "image/jpeg", "png": "image/png",
            "xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"}
THEME_NAME = "NUS-H2 Q&A"
LAYOUT_NAME = "Q&A"

# ---------------------------------------------------------------- slide 16, from the COFUNDING rows

CO_TITLE = "Can government co-funding lower the abatement cost further?"
CO_CATS = ["BOAK, {{LIVES_SHORT|f0}} y, no co-funding", "BOAK, {{LIVES_SHORT|f0}} y, {{COFUND_L5_25Y_SHARE_PCT}}% co-funded",
           "BOAK, {{LIVES_LONG|f0}} y, no co-funding", "BOAK, {{LIVES_LONG|f0}} y, {{COFUND_L5_60Y_SHARE_PCT}}% co-funded"]
CO_VALUES = ["AC_L5_BOAK_25Y", "COFUND_TARGET_S_T", "AC_L5_BOAK_60Y", "COFUND_TARGET_S_T"]
CO_SERIES = "Abatement cost, annual ledger (L5)"
CO_THRESHOLD = "S${{TARGET_COST_S_T|f0}}/tCO₂e threshold"
CO_COLOURS = ["1F3864", "548235", "1F3864", "548235"]
CO_BULLETS = [
    "Co-funding **S${{COFUND_L5_25Y_SM}}M, {{COFUND_L5_25Y_SHARE_PCT}}%** of the S${{COFUND_REACTOR_CAPITAL_SM}}M reactor "
    "capital, brings BOAK over {{LIVES_SHORT|f0}} years to **S${{COFUND_TARGET_S_T|f0}}/tCO₂e** on the annual ledger. "
    "The cash flow needs S${{COFUND_DCF_25Y_SM}}M, {{COFUND_DCF_25Y_SHARE_PCT}}%.",
    "Over {{LIVES_LONG|f0}} years, **S${{COFUND_L5_60Y_SM}}M, {{COFUND_L5_60Y_SHARE_PCT}}%**, is enough on the ledger. "
    "The cash flow needs S${{COFUND_DCF_60Y_SM}}M, {{COFUND_DCF_60Y_SHARE_PCT}}%.",
    "Without co-funding, BOAK costs S${{AC_L5_BOAK_25Y}} and S${{AC_L5_BOAK_60Y}}/tCO₂e on the ledger, and "
    "S${{LAC_DCF_BOAK_25Y}} and S${{LAC_DCF_BOAK_60Y}}/tCO₂e by cash flow (n = 1.0, {{R_RATED_MWTH|f0}} MWth charged).",
    "Co-funding moves cost to the public budget. The total resource cost stays the same.",
]
CO_NOTE = ("Q&A backup. The co-funding of reactor capital that brings BOAK to S${{COFUND_TARGET_S_T|f0}} a tonne is "
           "S${{COFUND_L5_25Y_SM}} million, {{COFUND_L5_25Y_SHARE_PCT}} percent of S${{COFUND_REACTOR_CAPITAL_SM}} million, "
           "over {{LIVES_SHORT|f0}} years on the ledger, and S${{COFUND_L5_60Y_SM}} million, {{COFUND_L5_60Y_SHARE_PCT}} "
           "percent, over {{LIVES_LONG|f0}} years. The cash flow needs S${{COFUND_DCF_25Y_SM}} million and "
           "S${{COFUND_DCF_60Y_SM}} million. Co-funding moves cost to the public budget, and the total resource cost "
           "stays the same.")
CO_GUARD = ("COFUND_L5_25Y_SM < COFUND_REACTOR_CAPITAL_SM and COFUND_L5_60Y_SM < COFUND_L5_25Y_SM"
            " and COFUND_DCF_60Y_SM < COFUND_DCF_25Y_SM and AC_L5_BOAK_25Y > COFUND_TARGET_S_T"
            " and AC_L5_BOAK_60Y > COFUND_TARGET_S_T and COFUND_TARGET_S_T < TARGET_COST_S_T")


def title_of(slide):
    """The title placeholder. The source titles carry idx 103, so python-pptx's shapes.title misses them."""
    for sh in slide.placeholders:
        if sh.placeholder_format.type in (PP_PLACEHOLDER.TITLE, PP_PLACEHOLDER.CENTER_TITLE):
            return sh
    return None


# ---------------------------------------------------------------- full words


def _full(match, before):
    """Replacement for one bare abbreviation, capitalised at the start of a sentence."""
    word = "small modular reactor" + match.group(1)
    head = before.rstrip()
    if not head or head[-1] in ".:?!":
        word = word[0].upper() + word[1:]
    return word


RULES = [
    (re.compile(r"\s*\(SMRs?\)"), lambda m, b: ""),
    (re.compile(r"\ballow SMR to\b"), lambda m, b: "allow small modular reactors to"),
    (re.compile(r"\bLW-SMR(s?)\b"), lambda m, b: "light-water small modular reactor" + m.group(1)),
    (re.compile(r"\bSMR(s?)\b"), _full),
]


def full_words(prs):
    """Write the abbreviation out in each run. A match that spans runs stops the build."""
    changed = []
    for i, slide in enumerate(prs.slides, 1):
        for shape in slide.shapes:
            if not shape.has_text_frame:
                continue
            for para in shape.text_frame.paragraphs:
                before = ""
                for run in para.runs:
                    text = run.text
                    for rx, fn in RULES:
                        out, pos = [], 0
                        for m in rx.finditer(text):
                            out.append(text[pos:m.start()])
                            out.append(fn(m, before + "".join(out)))
                            pos = m.end()
                        out.append(text[pos:])
                        text = "".join(out)
                    if text != run.text:
                        changed.append((i, run.text, text))
                        run.text = text
                    before += text
                if re.search(r"\bSMRs?\b", "".join(r.text for r in para.runs)):
                    raise SystemExit(f"Q&A slide {i}: abbreviation left across runs in: {para.text[:60]}")
    return changed


def fit_title(prs):
    """Shrink a title run that no longer fits its one-line box, measured with the run's own font."""
    font = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 400)
    fixed = []
    for i, slide in enumerate(prs.slides, 1):
        t = title_of(slide)
        if t is None:
            continue
        runs = [r for p in t.text_frame.paragraphs for r in p.runs]
        size = runs[0].font.size.pt
        width = font.getlength(t.text_frame.text) / 400 * size / 72
        room = t.width / 914400 - 0.05
        if width > room:
            new = math.floor(size * room / width * 2) / 2
            if new < size * 0.72:
                raise SystemExit(f"Q&A slide {i}: title needs {new} pt, under the floor")
            for r in runs:
                r.font.size = int(new * 12700)
            fixed.append((i, size, new))
    return fixed


def settle_cards(prs):
    """Slide 1: the full words add a line to the NuScale card, which then runs into its "Cancelled"
    stamp. The four card texts on that slide drop from 13 pt to 12 pt together, so the cards stay alike."""
    slide = prs.slides[0]
    cards = [s for s in slide.shapes if s.name == "Card text"]
    if len(cards) != 4 or not any("UAMPS" in s.text_frame.text for s in cards):
        raise SystemExit("Q&A source changed: slide 1 cards not found")
    for s in cards:
        for p in s.text_frame.paragraphs:
            for r in p.runs:
                if r.font.size is None or r.font.size.pt != 13:
                    raise SystemExit("Q&A source changed: slide 1 card text is no longer 13 pt")
                r.font.size = 12 * 12700
            end = p._p.find(f"{{{A}}}endParaRPr")
            if end is not None and end.get("sz") == "1300":
                end.set("sz", "1200")


# ---------------------------------------------------------------- slide 16


def _runs(p_el, text, template_rpr):
    """Fill a paragraph with runs; **bold** segments take b=1."""
    for k, piece in enumerate(re.split(r"\*\*", text)):
        if not piece:
            continue
        r = etree.SubElement(p_el, f"{{{A}}}r")
        rpr = copy.deepcopy(template_rpr)
        if k % 2:
            rpr.set("b", "1")
        elif "b" in rpr.attrib:
            del rpr.attrib["b"]
        r.append(rpr)
        etree.SubElement(r, f"{{{A}}}t").text = piece


def rebuild_cofunding(prs, book):
    env = book.env()
    if not eval(CO_GUARD, {"__builtins__": {}}, env):  # guard on token values only
        raise SystemExit("Q&A slide 16 guard failed: " + CO_GUARD)
    slide = prs.slides[N_SLIDES - 1]
    if title_of(slide).text_frame.text != CO_TITLE:
        raise SystemExit("Q&A source changed: slide 16 title not found")
    chart = next(s for s in slide.shapes if s.has_chart).chart
    data = CategoryChartData(number_format="0.0")
    data.categories = [book.r(c) for c in CO_CATS]
    values = [round(book.v(v), 1) for v in CO_VALUES]
    data.add_series(CO_SERIES, values)
    data.add_series(book.r(CO_THRESHOLD), [book.v("TARGET_COST_S_T")] * len(values))
    chart.replace_data(data)
    chart.value_axis.maximum_scale = math.ceil(max(values) / 25) * 25
    ser = chart._chartSpace.find(f".//{{{C}}}barChart/{{{C}}}ser")
    ser.find(f"{{{C}}}spPr/{{{A}}}solidFill/{{{A}}}srgbClr").set("val", CO_COLOURS[0])
    for dpt in ser.findall(f"{{{C}}}dPt"):
        idx = int(dpt.find(f"{{{C}}}idx").get("val"))
        dpt.find(f"{{{C}}}spPr/{{{A}}}solidFill/{{{A}}}srgbClr").set("val", CO_COLOURS[idx])
    body = next(s for s in slide.shapes if s.name == "Body text")
    tx = body.text_frame._txBody
    paras = tx.findall(f"{{{A}}}p")
    ppr = copy.deepcopy(paras[0].find(f"{{{A}}}pPr"))
    rpr = copy.deepcopy(paras[0].find(f"{{{A}}}r/{{{A}}}rPr"))
    end = copy.deepcopy(paras[0].find(f"{{{A}}}endParaRPr"))
    for p in paras:
        tx.remove(p)
    for text in CO_BULLETS:
        p = etree.SubElement(tx, f"{{{A}}}p")
        p.append(copy.deepcopy(ppr))
        _runs(p, book.r(text), rpr)
        p.append(copy.deepcopy(end))


# ---------------------------------------------------------------- scaling and merging


def _scale(v):
    return str(int(round(int(v) * SCALE)))


SCALED = {
    "off": ("x", "y"), "chOff": ("x", "y"), "ext": ("cx", "cy"), "chExt": ("cx", "cy"),
    "rPr": ("sz", "kern", "spc"), "defRPr": ("sz", "kern", "spc"), "endParaRPr": ("sz", "kern", "spc"),
    "ln": ("w",), "bodyPr": ("lIns", "tIns", "rIns", "bIns"), "spcPts": ("val",), "buSzPts": ("val",),
    "tab": ("pos",), "outerShdw": ("dist", "blurRad"), "innerShdw": ("dist", "blurRad"),
}
LEVELS = re.compile(r"^(pPr|lvl\dpPr|defPPr)$")


def scale_xml(blob):
    root = etree.fromstring(blob)
    for el in root.iter():
        if not isinstance(el.tag, str):
            continue
        ns, _, local = el.tag[1:].partition("}")
        if ns not in (A, C, P):
            continue
        attrs = SCALED.get(local, ())
        if LEVELS.match(local):
            attrs = ("marL", "indent", "defTabSz")
        for a in attrs:
            if a in el.attrib and re.fullmatch(r"-?\d+", el.attrib[a]):
                el.attrib[a] = _scale(el.attrib[a])
    return root


def _xml(root):
    return etree.tostring(root, xml_declaration=True, encoding="UTF-8", standalone=True)


def _rels(blob):
    return etree.fromstring(blob)


def _resolve(base, target):
    parts = (base.rsplit("/", 1)[0] + "/" + target).split("/")
    out = []
    for p in parts:
        if p == "..":
            out.pop()
        elif p and p != ".":
            out.append(p)
    return "/".join(out)


def _next_index(names, folder, stem, ext):
    rx = re.compile(rf"^{re.escape(folder)}/{re.escape(stem)}(\d+)\.{ext}$")
    return max([int(m.group(1)) for n in names for m in [rx.match(n)] if m] + [0]) + 1


def merge(main_bytes, qa_bytes):
    """Return the main deck with the Q&A master, layout, theme and slides appended."""
    zin, zqa = zipfile.ZipFile(io.BytesIO(main_bytes)), zipfile.ZipFile(io.BytesIO(qa_bytes))
    files = {n: zin.read(n) for n in zin.namelist()}
    src = {n: zqa.read(n) for n in zqa.namelist()}
    names = list(files)
    new = {}

    # Source slides in presentation order, and the one layout they use.
    sp = etree.fromstring(src["ppt/presentation.xml"])
    sp_rels = {r.get("Id"): r.get("Target") for r in _rels(src["ppt/_rels/presentation.xml.rels"])}
    slides = [_resolve("ppt/presentation.xml", sp_rels[s.get(f"{{{R}}}id")]) for s in sp.find(f"{{{P}}}sldIdLst")]
    if len(slides) != N_SLIDES:
        raise SystemExit(f"Q&A source has {len(slides)} slides, expected {N_SLIDES}")
    layouts = set()
    for s in slides:
        for r in _rels(src[s.replace("slides/", "slides/_rels/") + ".rels"]):
            if r.get("Type") == RT + "slideLayout":
                layouts.add(_resolve(s, r.get("Target")))
    if len(layouts) != 1:
        raise SystemExit(f"Q&A slides use {len(layouts)} layouts, expected one")
    src_layout = layouts.pop()
    lay_rels = _rels(src[src_layout.replace("slideLayouts/", "slideLayouts/_rels/") + ".rels"])
    src_master = _resolve(src_layout, next(r.get("Target") for r in lay_rels if r.get("Type") == RT + "slideMaster"))
    m_rels = _rels(src[src_master.replace("slideMasters/", "slideMasters/_rels/") + ".rels"])
    src_theme = _resolve(src_master, next(r.get("Target") for r in m_rels if r.get("Type") == RT + "theme"))

    master = f"ppt/slideMasters/slideMaster{_next_index(names, 'ppt/slideMasters', 'slideMaster', 'xml')}.xml"
    layout = f"ppt/slideLayouts/slideLayout{_next_index(names, 'ppt/slideLayouts', 'slideLayout', 'xml')}.xml"
    theme = f"ppt/theme/theme{_next_index(names, 'ppt/theme', 'theme', 'xml')}.xml"

    # Theme: same colours and fonts, neutral names.
    th = etree.fromstring(src[src_theme])
    th.set("name", THEME_NAME)
    for tag in ("clrScheme", "fontScheme", "fmtScheme"):
        for el in th.iter(f"{{{A}}}{tag}"):
            el.set("name", THEME_NAME)
    new[theme] = _xml(th)

    # Master: one layout only.
    pres = etree.fromstring(files["ppt/presentation.xml"])
    used = [int(x.get("id")) for x in pres.iter(f"{{{P}}}sldMasterId")]
    for n in names:
        if n.startswith("ppt/slideMasters/slideMaster") and n.endswith(".xml"):
            used += [int(x.get("id")) for x in etree.fromstring(files[n]).iter(f"{{{P}}}sldLayoutId")]
    master_id, layout_id = max(used) + 1, max(used) + 2
    mx = scale_xml(src[src_master])
    lst = mx.find(f"{{{P}}}sldLayoutIdLst")
    for el in list(lst):
        lst.remove(el)
    etree.SubElement(lst, f"{{{P}}}sldLayoutId", {"id": str(layout_id), f"{{{R}}}id": "rId1"})
    new[master] = _xml(mx)
    new[master.replace("slideMasters/", "slideMasters/_rels/") + ".rels"] = _relxml(
        [("rId1", RT + "slideLayout", "../slideLayouts/" + layout.rsplit("/", 1)[1]),
         ("rId2", RT + "theme", "../theme/" + theme.rsplit("/", 1)[1])])

    lx = scale_xml(src[src_layout])
    lx.find(f"{{{P}}}cSld").set("name", LAYOUT_NAME)
    new[layout] = _xml(lx)
    new[layout.replace("slideLayouts/", "slideLayouts/_rels/") + ".rels"] = _relxml(
        [("rId1", RT + "slideMaster", "../slideMasters/" + master.rsplit("/", 1)[1])])

    # Slides, pictures, chart and its workbook.
    first = _next_index(names, "ppt/slides", "slide", "xml")
    media_map, chart_n, emb_n = {}, 100, 100
    slide_names = []
    for k, s in enumerate(slides):
        name = f"ppt/slides/slide{first + k}.xml"
        slide_names.append(name)
        sx = scale_xml(src[s])
        sx.find(f"{{{P}}}cSld").set("name", f"Q&A {k + 1}")
        new[name] = _xml(sx)
        rels = []
        for r in _rels(src[s.replace("slides/", "slides/_rels/") + ".rels"]):
            typ, tgt, mode = r.get("Type"), r.get("Target"), r.get("TargetMode")
            if typ == RT + "notesSlide":
                continue
            if typ == RT + "slideLayout":
                tgt = "../slideLayouts/" + layout.rsplit("/", 1)[1]
            elif typ == RT + "image":
                full = _resolve(s, tgt)
                if full not in media_map:
                    ext = full.rsplit(".", 1)[1].lower()
                    media_map[full] = f"ppt/media/qa_image{len(media_map) + 1}.{ext}"
                    new[media_map[full]] = src[full]
                tgt = "../media/" + media_map[full].rsplit("/", 1)[1]
            elif typ == RT + "chart":
                full = _resolve(s, tgt)
                chart_n += 1
                cname = f"ppt/charts/chart{chart_n}.xml"
                new[cname] = _xml(scale_xml(src[full]))
                crels = []
                for cr in _rels(src[full.replace("charts/", "charts/_rels/") + ".rels"]):
                    emb_n += 1
                    ename = f"ppt/embeddings/Microsoft_Excel_Worksheet{emb_n}.xlsx"
                    new[ename] = src[_resolve(full, cr.get("Target"))]
                    crels.append((cr.get("Id"), cr.get("Type"), "../embeddings/" + ename.rsplit("/", 1)[1]))
                new[cname.replace("charts/", "charts/_rels/") + ".rels"] = _relxml(crels)
                tgt = "../charts/" + cname.rsplit("/", 1)[1]
            elif mode != "External":
                raise SystemExit(f"Q&A slide {k + 1}: unexpected relationship {typ}")
            rels.append((r.get("Id"), typ, tgt, mode))
        new[name.replace("slides/", "slides/_rels/") + ".rels"] = _relxml(rels)

    # Presentation part, its relationships and the content types.
    prels = _rels(files["ppt/_rels/presentation.xml.rels"])
    n_rid = max(int(r.get("Id")[3:]) for r in prels) + 1

    def add_rel(typ, target):
        nonlocal n_rid
        rid = f"rId{n_rid}"
        n_rid += 1
        etree.SubElement(prels, f"{{{PR}}}Relationship", Id=rid, Type=typ, Target=target)
        return rid

    mid = pres.find(f"{{{P}}}sldMasterIdLst")
    etree.SubElement(mid, f"{{{P}}}sldMasterId", {"id": str(master_id), f"{{{R}}}id": add_rel(RT + "slideMaster", master[4:])})
    sid = pres.find(f"{{{P}}}sldIdLst")
    next_id = max(int(x.get("id")) for x in sid) + 1
    for k, name in enumerate(slide_names):
        etree.SubElement(sid, f"{{{P}}}sldId", {"id": str(next_id + k), f"{{{R}}}id": add_rel(RT + "slide", name[4:])})
    files["ppt/presentation.xml"] = _xml(pres)
    files["ppt/_rels/presentation.xml.rels"] = _xml(prels)

    ct = etree.fromstring(files["[Content_Types].xml"])
    have = {d.get("Extension").lower() for d in ct.findall(f"{{{CT}}}Default")}
    for n in new:
        ext = n.rsplit(".", 1)[1].lower()
        if ext in DEFAULTS and ext not in have:
            etree.SubElement(ct, f"{{{CT}}}Default", Extension=ext, ContentType=DEFAULTS[ext])
            have.add(ext)
    kinds = [(master, "master"), (layout, "layout"), (theme, "theme")] + [(n, "slide") for n in slide_names]
    kinds += [(n, "chart") for n in new if n.startswith("ppt/charts/chart") and n.endswith(".xml")]
    for n, kind in kinds:
        etree.SubElement(ct, f"{{{CT}}}Override", PartName="/" + n, ContentType=TYPES[kind])
    files["[Content_Types].xml"] = _xml(ct)

    clash = set(new) & set(files)
    if clash:
        raise SystemExit(f"Q&A merge would overwrite {sorted(clash)}")
    files.update(new)
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for n in ["[Content_Types].xml"] + [n for n in files if n != "[Content_Types].xml"]:
            z.writestr(n, files[n])
    return out.getvalue(), len(slide_names), len(media_map)


def _relxml(rows):
    root = etree.Element(f"{{{PR}}}Relationships", nsmap={None: PR})
    for row in rows:
        rid, typ, tgt = row[:3]
        el = etree.SubElement(root, f"{{{PR}}}Relationship", Id=rid, Type=typ, Target=tgt)
        if len(row) > 3 and row[3]:
            el.set("TargetMode", row[3])
    return _xml(root)


# ---------------------------------------------------------------- entry point


STRETCH_TOL = 1.02   # a picture drawn more than 2% off its own aspect counts as stretched
PHOTO_ANCHOR = 0.45  # a photograph cropped in height keeps a little more of its upper part


def _pictures(shapes):
    for sh in shapes:
        if sh.shape_type == MSO_SHAPE_TYPE.GROUP:
            yield from _pictures(sh.shapes)
        elif sh.shape_type == MSO_SHAPE_TYPE.PICTURE:
            yield sh


def unstretch(prs):
    """Draw each stretched picture at its own aspect. The pictures stay as supplied, byte for byte.

    A photograph (JPEG) keeps its frame and is cropped to the frame's aspect, so the card layout does not
    move. A drawing (PNG) keeps all of its content: the frame narrows or shortens to the drawing, centred
    where it was.
    """
    fixed = []
    for i, slide in enumerate(prs.slides, 1):
        for sh in _pictures(slide.shapes):
            if any((sh.crop_left, sh.crop_right, sh.crop_top, sh.crop_bottom)):
                continue  # already cropped by the author
            with PILImage.open(io.BytesIO(sh.image.blob)) as im:
                ia = im.width / im.height
            fa = sh.width / sh.height
            if max(ia / fa, fa / ia) <= STRETCH_TOL:
                continue
            if sh.image.content_type == "image/jpeg":
                if ia > fa:  # too wide: trim both sides equally
                    cut = 1 - fa / ia
                    sh.crop_left = sh.crop_right = cut / 2
                else:  # too tall: trim height, keeping the upper part a little more
                    cut = 1 - ia / fa
                    sh.crop_top, sh.crop_bottom = cut * PHOTO_ANCHOR, cut * (1 - PHOTO_ANCHOR)
                how = "cropped"
            else:
                if ia < fa:
                    w = int(round(sh.height * ia))
                    sh.left, sh.width = sh.left + (sh.width - w) // 2, w
                else:
                    h = int(round(sh.width / ia))
                    sh.top, sh.height = sh.top + (sh.height - h) // 2, h
                how = "fitted"
            fixed.append((i, sh.name, round(max(ia / fa, fa / ia), 3), how))
    return fixed


def edited_source(book):
    """The source deck with its text edits and slide 16 rebuilt, at its own size."""
    if not SOURCE.is_file():
        raise SystemExit(f"Q&A source missing: {SOURCE.name} in presentation/inputs")
    prs = Presentation(str(SOURCE))
    if len(prs.slides) != N_SLIDES:
        raise SystemExit(f"Q&A source has {len(prs.slides)} slides, expected {N_SLIDES}")
    changed = full_words(prs)
    settle_cards(prs)
    rebuild_cofunding(prs, book)
    fitted = fit_title(prs)
    global UNSTRETCHED
    UNSTRETCHED = unstretch(prs)
    buf = io.BytesIO()
    prs.save(buf)
    return buf.getvalue(), changed, fitted


UNSTRETCHED = []


def append(pptx_path, book, notes_for):
    """Append the Q&A slides to the saved deck at pptx_path and give each one a note."""
    qa, changed, fitted = edited_source(book)
    merged, n, n_media = merge(pptx_path.read_bytes(), qa)
    pptx_path.write_bytes(merged)
    prs = Presentation(str(pptx_path))
    titles = []
    for k, slide in enumerate(list(prs.slides)[-n:], 1):
        title = title_of(slide).text_frame.text
        titles.append(title)
        slide.notes_slide.notes_text_frame.text = notes_for(k, title)
    prs.save(str(pptx_path))
    return dict(slides=n, media=n_media, words_changed=changed, titles_fitted=fitted, titles=titles, unstretched=UNSTRETCHED)
