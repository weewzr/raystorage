import sys, re
key=sys.argv[1]; ps=[int(x) for x in sys.argv[2:]]
t=open("txt/"+key+".txt",encoding="utf-8").read()
parts=re.split(r"=====PDFPAGE (\d+)=====",t)
d={int(parts[i]):parts[i+1] for i in range(1,len(parts),2)}
for p in ps:
    print(f"--- {key} PDF p{p}"); print(re.sub(r"\n\s*\n+","\n",d.get(p,"")))
