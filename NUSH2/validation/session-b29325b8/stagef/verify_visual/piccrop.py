import sys, zipfile, re, io
from PIL import Image
sys.stdout.reconfigure(encoding="utf-8")
z = zipfile.ZipFile(sys.argv[1]); sl = sys.argv[2]
x = z.read(f"ppt/slides/slide{sl}.xml").decode("utf8")
rels = dict(re.findall(r'Id="(rId\d+)"[^>]*Target="\.\./media/([^"]+)"', z.read(f"ppt/slides/_rels/slide{sl}.xml.rels").decode("utf8")))
rels.update({a: b for b, a in re.findall(r'Target="\.\./media/([^"]+)"[^>]*Id="(rId\d+)"', z.read(f"ppt/slides/_rels/slide{sl}.xml.rels").decode("utf8"))})
for m in re.finditer(r"<p:pic>.*?</p:pic>", x, re.S):
    p = m.group(0)
    name = re.search(r'name="([^"]*)"', p).group(1)
    rid = re.search(r'r:embed="(rId\d+)"', p).group(1)
    src = re.search(r"<a:srcRect([^/]*)/>", p)
    ext = re.search(r'<a:ext cx="(\d+)" cy="(\d+)"', p)
    fill = "stretch" if "<a:stretch>" in p else ""
    med = rels.get(rid)
    im = Image.open(io.BytesIO(z.read("ppt/media/" + med)))
    W, H = im.size
    crop = {k: int(v) / 100000 for k, v in re.findall(r'(\w)="(-?\d+)"', src.group(1))} if src else {}
    cw = W * (1 - crop.get("l", 0) - crop.get("r", 0)); ch = H * (1 - crop.get("t", 0) - crop.get("b", 0))
    cx, cy = int(ext.group(1)), int(ext.group(2))
    print(f"  {name[:30]:30s} media {med} {W}x{H} srcRect {crop} -> cropped aspect {cw/ch:.3f}, frame aspect {cx/cy:.3f}, distortion {(cx/cy)/(cw/ch):.3f}")
