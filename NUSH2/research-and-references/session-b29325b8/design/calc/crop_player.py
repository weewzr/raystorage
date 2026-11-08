import sys
from pathlib import Path
from PIL import Image
S = Path(sys.argv[1])
fr = S/'video_frames'
names = ['t00002.jpg','t00006.jpg','t00010.jpg','t00014.jpg']
im0 = Image.open(fr/names[0]); print(im0.size)
W,H = im0.size
sheet = Image.new('RGB',(W*1, 4*120*2),'white')
for i,n in enumerate(names):
    im = Image.open(fr/n)
    c = im.crop((0,H-70,W,H)).resize((W,240) if False else (W, 140))
    sheet.paste(c,(0,i*150))
sheet = sheet.crop((0,0,W,600))
sheet = sheet.resize((W*2,1200))
sheet.save(S/'design'/'calc'/'player_crop.png')
