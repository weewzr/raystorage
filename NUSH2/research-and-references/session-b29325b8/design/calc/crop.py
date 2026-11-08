import sys, fitz
pdf,page,dpi,x0,y0,x1,y1,out = sys.argv[1],int(sys.argv[2]),int(sys.argv[3]),*map(float,sys.argv[4:8]),sys.argv[8]
d=fitz.open(pdf); p=d[page-1]; r=p.rect
clip=fitz.Rect(r.width*x0,r.height*y0,r.width*x1,r.height*y1)
px=p.get_pixmap(dpi=dpi,clip=clip); px.save(out); print(out,px.width,px.height)
