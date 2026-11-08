import sys,re,subprocess
# compact: join short lines
out=subprocess.run([sys.executable,"pg.py"]+sys.argv[1:],capture_output=True,text=True,encoding="utf-8",errors="replace").stdout
lines=[l.strip() for l in out.splitlines() if l.strip()]
buf=[];res=[]
for l in lines:
    buf.append(l)
    if len(" ".join(buf))>170 or l.startswith("==="):
        res.append(" ".join(buf));buf=[]
res.append(" ".join(buf))
print("\n".join(res))
