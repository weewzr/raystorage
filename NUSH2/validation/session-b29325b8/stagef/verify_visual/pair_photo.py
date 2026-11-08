import sys, zipfile, io
from PIL import Image, ImageOps
z = zipfile.ZipFile(sys.argv[1]); photo = sys.argv[2]; nm = sys.argv[3]; out = sys.argv[4]
box = tuple(float(v) for v in sys.argv[5:9])  # fractional crop of photo
P = ImageOps.exif_transpose(Image.open(photo)).convert("RGB")
W, H = P.size
P = P.crop((int(box[0] * W), int(box[1] * H), int(box[2] * W), int(box[3] * H)))
P.thumbnail((900, 700))
T = Image.open(io.BytesIO(z.read("ppt/media/" + nm))).convert("RGB"); T.thumbnail((700, 700))
S = Image.new("RGB", (P.width + T.width + 10, max(P.height, T.height)), (255, 0, 0)); S.paste(P, (0, 0)); S.paste(T, (P.width + 10, 0)); S.save(out)
