import sys
from pptx import Presentation
from pptx.util import Emu
sys.stdout.reconfigure(encoding="utf-8")
p = Presentation(sys.argv[1]); idx = [int(x) for x in sys.argv[2].split(",")]
k = p.slide_width / 914400
for i in idx:
    s = p.slides[i - 1]
    print("slide", i, "layout", s.slide_layout.name, "title-ph text:", repr(s.shapes.title.text_frame.text if s.shapes.title is not None else None))
    for sh in s.shapes:
        t = sh.text_frame.text.replace("\n", " / ")[:70] if sh.has_text_frame else ""
        ph = sh.placeholder_format.type if sh.is_placeholder else ""
        print("   ", sh.shape_type, ph, sh.name[:22], round(sh.left / 914400 / k * 13.333, 2) if sh.left is not None else None, round(sh.top / 914400 / k * 13.333, 2) if sh.top is not None else None, round(sh.width / 914400 / k * 13.333, 2) if sh.width else None, round(sh.height / 914400 / k * 13.333, 2) if sh.height else None, "|", t)
    # layout and master placeholders (footer/slide number)
    for sh in s.slide_layout.placeholders:
        print("    layout ph:", sh.placeholder_format.type, sh.name)
