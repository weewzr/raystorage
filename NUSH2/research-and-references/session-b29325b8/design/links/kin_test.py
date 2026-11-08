import urllib.request,urllib.error,ssl,sys,io,socket
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
UA={'User-Agent':'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36','Accept':'*/*'}
def probe(u):
    out=[]
    for meth in ('HEAD','GET'):
        h=dict(UA)
        if meth=='GET': h['Range']='bytes=0-2047'
        try:
            r=urllib.request.urlopen(urllib.request.Request(u,headers=h,method=meth),timeout=30)
            b=r.read(2048) if meth=='GET' else b''
            out.append((meth,r.status,r.headers.get('Content-Type'),r.headers.get('Content-Length'),r.headers.get('Content-Range'),r.geturl()[:120],b[:8]))
            if meth=='GET': return out,b
        except urllib.error.HTTPError as e:
            out.append((meth,e.code,e.headers.get('Content-Type'),e.headers.get('Content-Length'),None,(e.geturl() or '')[:120],b''))
        except Exception as e:
            out.append((meth,'EXC',repr(e)[:100]))
    return out,b''
import re
for u in sys.argv[2:] or [l.strip() for l in open(sys.argv[1]) if l.strip()]:
    res,b=probe(u)
    print(u[:110])
    for x in res: print('   ',x)
    m=re.search(rb'<title[^>]*>(.{0,120})',b,re.I|re.S)
    if m: print('    title:',m.group(1).decode('utf-8','replace').strip()[:110])
