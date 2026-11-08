import sys
sys_ = __import__("sys"); sys_.stdout.reconfigure(encoding="utf-8")
from pptx import Presentation
from pptx.util import Emu
p = Presentation(sys.argv[1])
print("size", p.slide_width, p.slide_height, "layouts", [l.name for l in p.slide_layouts])
cp = p.core_properties
print("core", repr(cp.author), repr(cp.title), repr(cp.last_modified_by), repr(cp.subject), repr(cp.comments), repr(cp.keywords), repr(cp.category))
for i, s in enumerate(p.slides, 1):
    print(f"=== slide {i} layout={s.slide_layout.name}")
    for sh in s.shapes:
        t = sh.text_frame.text.replace("\n", " / ")[:300] if sh.has_text_frame else ""
        extra = ""
        if sh.shape_type == 13:
            extra = f" img={sh.image.filename} {sh.image.content_type} part={sh._element.blipFill.blip.rEmbed if hasattr(sh._element,'blipFill') else ''}"
        print(f"  [{sh.shape_type}] {sh.name!r} x={Emu(sh.left).inches if sh.left is not None else None:.2f} y={Emu(sh.top).inches if sh.top is not None else 0:.2f} w={Emu(sh.width).inches if sh.width else 0:.2f} h={Emu(sh.height).inches if sh.height else 0:.2f}{extra} :: {t}")
    if s.has_notes_slide:
        print("  NOTES:", s.notes_slide.notes_text_frame.text[:500].replace("\n", " / "))
