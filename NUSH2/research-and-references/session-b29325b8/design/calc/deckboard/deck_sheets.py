"""Contact sheets of deck renders with slide numbers. Reads renders only."""
import sys
from pathlib import Path
from PIL import Image, ImageDraw
S=Path(sys.argv[1]); out=Path(sys.argv[2])
cols=3; tw=640
files=sorted((S/'deck_png').glob('s*.png'))
th=int(tw*Image.open(files[0]).size[1]/Image.open(files[0]).size[0])
sel=[files[i:i+9] for i in range(0,31,9)]
for k,grp in enumerate(sel):
    rows=(len(grp)+cols-1)//cols
    sh=Image.new('RGB',(cols*tw,rows*th),'white')
    for i,f in enumerate(grp):
        im=Image.open(f).convert('RGB').resize((tw,th))
        d=ImageDraw.Draw(im); d.rectangle((0,0,44,16),fill='black'); d.text((3,2),f.stem,fill='yellow')
        sh.paste(im,((i%cols)*tw,(i//cols)*th))
    sh.save(out/f'deck_{k}.jpg',quality=85)
print(th)
