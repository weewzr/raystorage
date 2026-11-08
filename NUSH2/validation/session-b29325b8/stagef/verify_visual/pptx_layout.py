import sys
from pptx import Presentation
from pptx.util import Emu
sys.stdout.reconfigure(encoding="utf-8")
p = Presentation(sys.argv[1])
print("slide size", p.slide_width, p.slide_height, "masters", len(p.slide_masters))
for i, s in enumerate(p.slides, 1):
    lay = s.slide_layout
    mi = [m for m in p.slide_masters].index(lay.slide_master)
    ph = [(sh.placeholder_format.type.__str__().split('.')[-1] if sh.is_placeholder else '') for sh in s.shapes if sh.is_placeholder]
    title = s.shapes.title.text_frame.text[:60] if s.shapes.title is not None and s.shapes.title.has_text_frame else ''
    if not title:
        texts = [sh.text_frame.text for sh in s.shapes if sh.has_text_frame and sh.text_frame.text.strip()]
        title = (texts[0] if texts else '')[:60].replace('\n', ' ')
    notes = s.has_notes_slide and s.notes_slide.notes_text_frame is not None and s.notes_slide.notes_text_frame.text.strip()
    print(i, f"m{mi}", lay.name[:28], ph, "|", title.replace('\n', ' '), "| notes:", len(notes) if notes else 0)
