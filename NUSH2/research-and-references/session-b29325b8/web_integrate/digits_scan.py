"""List string literals that hold a digit, and bare numeric literals, in the
frontend JS, plus visible text with digits in index.html. Scratch review aid."""
import re
import sys
from html.parser import HTMLParser
from pathlib import Path

FRONT = Path(sys.argv[1])

STR = re.compile(r'''"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'|`(?:[^`\\]|\\.)*`''', re.S)
COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
NUM = re.compile(r"(?<![\w.$])(\d+\.\d+|\d{2,}(?:\.\d+)?|\d+e-?\d+)(?![\w])")


def strip_comments(text):
    out, i = [], 0
    # Remove comments that sit outside string literals.
    pos = 0
    while pos < len(text):
        m_s = STR.search(text, pos)
        m_c = COMMENT.search(text, pos)
        if m_c and (not m_s or m_c.start() < m_s.start()):
            # check the // is not inside a regex literal like /^\d+$/ ... crude
            out.append(text[pos:m_c.start()])
            out.append(" " * (m_c.end() - m_c.start()) if "\n" not in m_c.group(0) else "\n" * m_c.group(0).count("\n"))
            pos = m_c.end()
        elif m_s:
            out.append(text[pos:m_s.end()])
            pos = m_s.end()
        else:
            out.append(text[pos:])
            break
    return "".join(out)


for js in sorted(FRONT.glob("*.js")):
    text = js.read_text(encoding="utf-8")
    code = strip_comments(text)
    print(f"===== {js.name}")
    for m in STR.finditer(code):
        s = m.group(0)
        if re.search(r"\d", s):
            line = code.count("\n", 0, m.start()) + 1
            short = s if len(s) < 160 else s[:157] + "..."
            print(f"S {line}: {short!r}")
    no_str = STR.sub(lambda m: '""' + "\n" * m.group(0).count("\n"), code)
    for i, ln in enumerate(no_str.splitlines(), 1):
        for m in NUM.finditer(ln):
            print(f"N {i}: {m.group(1)}   | {ln.strip()[:140]}")


class Vis(HTMLParser):
    def __init__(self):
        super().__init__()
        self.stack = []
        self.hits = []

    def handle_starttag(self, tag, attrs):
        self.stack.append(tag)
        for k, v in attrs:
            if v and k in ("aria-label", "title", "alt", "placeholder") and re.search(r"\d", v):
                self.hits.append((self.getpos()[0], f"@{k}", v))

    def handle_endtag(self, tag):
        while self.stack and self.stack.pop() != tag:
            pass

    def handle_data(self, data):
        if self.stack and self.stack[-1] in ("script", "style"):
            return
        t = " ".join(data.split())
        if t and re.search(r"\d", t):
            self.hits.append((self.getpos()[0], self.stack[-1] if self.stack else "", t))


for html in sorted(FRONT.glob("*.html")):
    print(f"===== {html.name} visible text with digits")
    v = Vis()
    v.feed(html.read_text(encoding="utf-8"))
    for line, tag, t in v.hits:
        print(f"H {line} <{tag}>: {t[:160]}")
