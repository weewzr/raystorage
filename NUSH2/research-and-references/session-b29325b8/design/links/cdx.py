import urllib.request, urllib.parse, json, sys, time
UA="Mozilla/5.0 (research link check)"
def cdx(url, match='prefix', filters=(), limit=2000):
    params=[('url',url),('matchType',match),('output','json'),('fl','original,timestamp,statuscode,mimetype,length'),('collapse','urlkey'),('limit',str(limit))]
    for f in filters: params.append(('filter',f))
    u='https://web.archive.org/cdx/search/cdx?'+urllib.parse.urlencode(params)
    for k in range(3):
        try:
            r=urllib.request.urlopen(urllib.request.Request(u,headers={'User-Agent':UA}),timeout=120)
            t=r.read().decode('utf-8','replace')
            return json.loads(t) if t.strip() else []
        except Exception as e:
            print('retry',repr(e)[:120]); time.sleep(8)
    return []
if __name__=='__main__':
    for spec in sys.argv[1:]:
        url,*fl=spec.split('||')
        d=cdx(url,filters=fl)
        print('#####',spec,len(d))
        for x in d[1:]:
            print(x[1],x[2],x[3],x[4],urllib.parse.unquote(x[0])[:200])
        time.sleep(4)
