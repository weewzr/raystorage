import sys, zipfile, io, re, hashlib
from PIL import Image
sys.stdout.reconfigure(encoding="utf-8")
z = zipfile.ZipFile(sys.argv[1])
# map media -> slides
rels = {}
for n in z.namelist():
    m = re.match(r"ppt/slides/_rels/slide(\d+)\.xml\.rels", n)
    if m:
        for t in re.findall(r'Target="\.\./media/([^"]+)"', z.read(n).decode("utf8")):
            rels.setdefault(t, []).append(int(m.group(1)))
for n in sorted(z.namelist()):
    if n.startswith("ppt/media/"):
        b = z.read(n); nm = n.split("/")[-1]
        try:
            im = Image.open(io.BytesIO(b)); sz = im.size
            ex = im.getexif() if hasattr(im, "getexif") else {}
            exs = {k: ex.get(k) for k in (271, 272, 306) if ex.get(k)}
        except Exception as e:
            sz, exs = ("?",), {}
        sl = sorted(set(rels.get(nm, [])))
        print(nm, sz, len(b), "slides", sl, exs if exs else "")
