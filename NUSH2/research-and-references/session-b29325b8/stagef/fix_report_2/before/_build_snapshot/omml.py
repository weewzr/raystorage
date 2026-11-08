#!/usr/bin/env python3
"""Small LaTeX subset to native Word math (OMML).

Every run is written as normal-text math in Times New Roman at the size asked for, so a
unit stays a real m:oMath object with real sub and superscripts while matching 12 pt body
text. Supported: \\mathrm, \\text, ^, _, \\ , \\, , \\frac, \\sum, \\left( \\right),
Greek letters, \\times, \\cdot, \\%, \\$ and plain characters.
"""
import re
from lxml import etree

M = "http://schemas.openxmlformats.org/officeDocument/2006/math"
W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
XML_SPACE = "{http://www.w3.org/XML/1998/namespace}space"
FONT = "Times New Roman"

GREEK = {
    "alpha": "α", "beta": "β", "gamma": "γ", "delta": "δ", "epsilon": "ε", "eta": "η",
    "theta": "θ", "kappa": "κ", "lambda": "λ", "mu": "μ", "pi": "π", "rho": "ρ",
    "sigma": "σ", "tau": "τ", "phi": "φ", "omega": "ω",
    "Delta": "Δ", "Gamma": "Γ", "Omega": "Ω", "Phi": "Φ", "Sigma": "Σ",
}
SYMBOLS = {"times": "×", "cdot": "·", "leq": "≤", "geq": "≥", "approx": "≈", "circ": "°",
           "ldots": "…", "minus": "−"}
BINARY = {"=": "=", "+": "+", "-": "−", "×": "×", "≈": "≈", "≤": "≤", "≥": "≥"}


def q(prefix, name):
    return "{%s}%s" % ({"m": M, "w": W}[prefix], name)


class Text:
    def __init__(self, s, style):
        self.s, self.style = s, style


class Script:
    def __init__(self, base, sub=None, sup=None):
        self.base, self.sub, self.sup = base, sub, sup


class Frac:
    def __init__(self, num, den):
        self.num, self.den = num, den


class Nary:
    def __init__(self, sub, sup, body):
        self.sub, self.sup, self.body = sub, sup, body


class Paren:
    def __init__(self, inner):
        self.inner = inner


TOKEN = re.compile(r"\\[a-zA-Z]+|\\.|[{}^_~]|\s+|.", re.S)


class Parser:
    def __init__(self, latex):
        self.toks = [t for t in TOKEN.findall(latex)]
        self.i = 0

    def peek(self):
        return self.toks[self.i] if self.i < len(self.toks) else None

    def next(self):
        t = self.peek()
        self.i += 1
        return t

    def group(self, style, depth):
        """A {...} group, or a single token when no brace follows."""
        while self.peek() is not None and self.peek().isspace():
            self.next()
        if self.peek() == "{":
            self.next()
            nodes = self.seq(style, depth, closing=True)
            return nodes
        nodes = []
        self.atom(nodes, style, depth)
        return nodes

    def seq(self, style, depth, closing=False, until=None):
        nodes = []
        while True:
            t = self.peek()
            if t is None:
                if closing:
                    raise ValueError("unclosed brace")
                return nodes
            if closing and t == "}":
                self.next()
                return nodes
            if until is not None and t == until:
                self.next()
                return nodes
            self.atom(nodes, style, depth)

    @staticmethod
    def add_text(nodes, s, style):
        if nodes and isinstance(nodes[-1], Text) and nodes[-1].style == style:
            nodes[-1].s += s
        else:
            nodes.append(Text(s, style))

    @staticmethod
    def pop_base(nodes):
        if not nodes:
            return []
        last = nodes[-1]
        if isinstance(last, Text):
            if last.style == "roman":
                m = re.search(r"[A-Za-z0-9.]+$", last.s)
                cut = m.start() if m else len(last.s) - 1
            else:
                cut = len(last.s) - 1
            base = Text(last.s[cut:], last.style)
            last.s = last.s[:cut]
            if not last.s:
                nodes.pop()
            return [base]
        nodes.pop()
        return [last]

    def atom(self, nodes, style, depth):
        t = self.next()
        if t in ("^", "_"):
            base = self.pop_base(nodes)
            first = self.group(style, depth + 1)
            sub, sup = (first, None) if t == "_" else (None, first)
            if self.peek() in ("^", "_") and self.peek() != t:
                t2 = self.next()
                second = self.group(style, depth + 1)
                if t2 == "_":
                    sub = second
                else:
                    sup = second
            nodes.append(Script(base, sub, sup))
        elif t.startswith("\\") and len(t) > 1:
            name = t[1:]
            if name in ("mathrm", "text"):
                nodes.extend(self.group("roman", depth))
            elif name == "mathbf":
                nodes.extend(self.group("bold", depth))
            elif name == "frac":
                num = self.group(style, depth + 1)
                den = self.group(style, depth + 1)
                nodes.append(Frac(num, den))
            elif name == "sum":
                sub = sup = None
                while self.peek() in ("^", "_"):
                    which = self.next()
                    g = self.group(style, depth + 1)
                    if which == "_":
                        sub = g
                    else:
                        sup = g
                while self.peek() is not None and self.peek().isspace():
                    self.next()
                body = self.group(style, depth) if self.peek() == "{" else []
                if self.peek() is not None and not body:
                    self.atom(body, style, depth)
                nodes.append(Nary(sub, sup, body))
            elif name == "left":
                self.next()
                inner = []
                while True:
                    if self.peek() == "\\right":
                        self.next()
                        self.next()
                        break
                    if self.peek() is None:
                        raise ValueError("\\left without \\right")
                    self.atom(inner, style, depth)
                nodes.append(Paren(inner))
            elif name == "quad":
                self.add_text(nodes, " ", "roman")
            elif name in GREEK:
                glyph = GREEK[name]
                use = "roman" if glyph.isupper() else style
                self.add_text(nodes, glyph, use)
            elif name in SYMBOLS:
                glyph = SYMBOLS[name]
                if glyph in BINARY and depth == 0 and style != "roman":
                    self.add_text(nodes, " " + glyph + " ", "roman")
                else:
                    self.add_text(nodes, glyph, "roman" if style == "math" else style)
            elif len(name) == 1:
                self.add_text(nodes, name, style)
            else:
                raise ValueError(f"unsupported command {t}")
        elif t == "\\ ":
            self.add_text(nodes, " ", style if style != "math" else "roman")
        elif t == "\\,":
            self.add_text(nodes, " ", style if style != "math" else "roman")
        elif t == "~":
            self.add_text(nodes, " ", style if style != "math" else "roman")
        elif t.isspace():
            if style != "math":
                self.add_text(nodes, " ", style)
        elif t == "{":
            nodes.extend(self.seq(style, depth, closing=True))
        elif t == "-" and depth > 0:
            self.add_text(nodes, "−", style if style != "math" else "roman")
        elif t in BINARY and style == "math" and depth == 0:
            self.add_text(nodes, " " + BINARY[t] + " ", "roman")
        elif t == "-":
            self.add_text(nodes, "−", style if style != "math" else "roman")
        else:
            self.add_text(nodes, t, style)


def parse(latex):
    return Parser(latex).seq("math", 0)


def m_run(text, italic=False, bold=False, size=24):
    r = etree.Element(q("m", "r"))
    pr = etree.SubElement(r, q("m", "rPr"))
    etree.SubElement(pr, q("m", "nor"))
    w = etree.SubElement(r, q("w", "rPr"))
    fonts = etree.SubElement(w, q("w", "rFonts"))
    for a in ("ascii", "hAnsi", "cs", "eastAsia"):
        fonts.set(q("w", a), FONT)
    if bold:
        etree.SubElement(w, q("w", "b"))
    if italic:
        etree.SubElement(w, q("w", "i"))
    etree.SubElement(w, q("w", "sz")).set(q("w", "val"), str(size))
    etree.SubElement(w, q("w", "szCs")).set(q("w", "val"), str(size))
    t = etree.SubElement(r, q("m", "t"))
    t.text = text
    t.set(XML_SPACE, "preserve")
    return r


MINUS = "−"


def minus_run(bold=False, size=24):
    """A minus sign as a math-mode run, not normal text. Word stores a math minus as a hyphen in the
    file and draws it as a minus only outside normal-text runs, so a minus inside an m:nor run would
    reopen as a hyphen."""
    r = etree.Element(q("m", "r"))
    pr = etree.SubElement(r, q("m", "rPr"))
    etree.SubElement(pr, q("m", "sty")).set(q("m", "val"), "b" if bold else "p")
    w = etree.SubElement(r, q("w", "rPr"))
    fonts = etree.SubElement(w, q("w", "rFonts"))
    for a in ("ascii", "hAnsi"):
        fonts.set(q("w", a), "Cambria Math")
    if bold:
        etree.SubElement(w, q("w", "b"))
    etree.SubElement(w, q("w", "sz")).set(q("w", "val"), str(size))
    etree.SubElement(w, q("w", "szCs")).set(q("w", "val"), str(size))
    t = etree.SubElement(r, q("m", "t"))
    t.text = MINUS
    return r


def text_runs(node, size, bold):
    """In math style letters are italic, digits and operators upright. Every minus sign becomes its
    own math-mode run."""
    if MINUS in node.s:
        out = []
        for piece in re.split(f"({MINUS})", node.s):
            if piece == MINUS:
                out.append(minus_run(bold or node.style == "bold", size))
            elif piece.strip():
                out.extend(text_runs(Text(piece, node.style), size, bold))
            elif piece:
                out.append(m_run(piece, False, bold or node.style == "bold", size))
        return out
    if node.style in ("roman", "bold"):
        return [m_run(node.s, False, bold or node.style == "bold", size)]
    out, buf, ital = [], "", None
    for ch in node.s:
        is_letter = ch.isalpha()
        if ital is None or is_letter == ital:
            buf += ch
            ital = is_letter
        else:
            out.append(m_run(buf, ital, bold, size))
            buf, ital = ch, is_letter
    if buf:
        out.append(m_run(buf, bool(ital), bold, size))
    return out


def emit(nodes, parent, size, bold):
    for n in nodes:
        if isinstance(n, Text):
            if n.s:
                for r in text_runs(n, size, bold):
                    parent.append(r)
        elif isinstance(n, Script):
            kind = "sSubSup" if n.sub is not None and n.sup is not None else ("sSub" if n.sub is not None else "sSup")
            el = etree.SubElement(parent, q("m", kind))
            emit(n.base, etree.SubElement(el, q("m", "e")), size, bold)
            if n.sub is not None:
                emit(n.sub, etree.SubElement(el, q("m", "sub")), size, bold)
            if n.sup is not None:
                emit(n.sup, etree.SubElement(el, q("m", "sup")), size, bold)
        elif isinstance(n, Frac):
            el = etree.SubElement(parent, q("m", "f"))
            emit(n.num, etree.SubElement(el, q("m", "num")), size, bold)
            emit(n.den, etree.SubElement(el, q("m", "den")), size, bold)
        elif isinstance(n, Nary):
            el = etree.SubElement(parent, q("m", "nary"))
            pr = etree.SubElement(el, q("m", "naryPr"))
            etree.SubElement(pr, q("m", "chr")).set(q("m", "val"), "∑")
            etree.SubElement(pr, q("m", "limLoc")).set(q("m", "val"), "undOvr")
            etree.SubElement(pr, q("m", "subHide")).set(q("m", "val"), "0" if n.sub else "1")
            etree.SubElement(pr, q("m", "supHide")).set(q("m", "val"), "0" if n.sup else "1")
            emit(n.sub or [], etree.SubElement(el, q("m", "sub")), size, bold)
            emit(n.sup or [], etree.SubElement(el, q("m", "sup")), size, bold)
            emit(n.body, etree.SubElement(el, q("m", "e")), size, bold)
        elif isinstance(n, Paren):
            el = etree.SubElement(parent, q("m", "d"))
            pr = etree.SubElement(el, q("m", "dPr"))
            etree.SubElement(pr, q("m", "begChr")).set(q("m", "val"), "(")
            etree.SubElement(pr, q("m", "endChr")).set(q("m", "val"), ")")
            emit(n.inner, etree.SubElement(el, q("m", "e")), size, bold)


def omath(latex, size=24, bold=False):
    """An m:oMath element for inline use."""
    el = etree.Element(q("m", "oMath"), nsmap={"m": M, "w": W})
    emit(parse(latex), el, size, bold)
    return el


def omath_para(latex, size=24, jc="center"):
    """An m:oMathPara element for a display equation."""
    para = etree.Element(q("m", "oMathPara"), nsmap={"m": M, "w": W})
    pr = etree.SubElement(para, q("m", "oMathParaPr"))
    etree.SubElement(pr, q("m", "jc")).set(q("m", "val"), jc)
    para.append(omath(latex, size))
    return para


if __name__ == "__main__":
    for s in (r"\mathrm{MW_{th}}", r"\mathrm{Nm^{3}\ h^{-1}}", r"\mathrm{S\$189.8/tCO_{2}e}",
              r"\mathrm{US\$_{2019}\ kW_{t}^{-1}}", r"\Delta C = \sum_{i} C_{i}",
              r"\mathrm{CRF} = \frac{r}{1 - \left(1 + r\right)^{-L}}"):
        print(etree.tostring(omath(s), encoding="unicode")[:160])
