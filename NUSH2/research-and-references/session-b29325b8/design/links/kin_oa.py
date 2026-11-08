import urllib.request,urllib.parse,json,sys,io,time
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
UA={'User-Agent':'Mozilla/5.0 academic reference check'}
def get(u):
    return json.load(urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=30))
def absd(m):
    ii=m.get('abstract_inverted_index')
    if not ii: return ''
    pos=[]
    for w,ps in ii.items():
        for p in ps: pos.append((p,w))
    return ' '.join(w for p,w in sorted(pos))[:420]
dois=['10.1021/je00051a011','10.1016/S0166-9834(00)81507-9','10.1021/ie9004578','10.1016/j.supflu.2010.09.039','10.1021/ie1006855','10.1021/acs.iecr.8b01471','10.1016/S1385-8947(00)00367-3','10.1021/je960314v','10.1002/cjce.5450760512','10.1002/aic.690420130','10.1021/je060031v','10.1021/acs.iecr.5c03283','10.2202/1542-6580.2238','10.1016/S0378-7753(03)00614-1','10.1021/ie049886w','10.1021/ie101137b','10.1016/0360-3199(94)00130-R','10.1006/jcat.1996.0016','10.1016/S0021-9517(03)00050-2','10.1016/S0166-9834(00)83024-9','10.1063/1.1564818']
for d in dois:
    try:
        m=get('https://api.openalex.org/works/doi:'+urllib.parse.quote(d))
        oa=m.get('open_access',{})
        print(d,'|',(m.get('title') or '')[:80],'| oa:',oa.get('oa_status'),oa.get('oa_url'))
        for l in (m.get('locations') or [])[:4]:
            if l.get('pdf_url') or l.get('is_oa'):
                print('    loc:',l.get('pdf_url') or l.get('landing_page_url'),'oa=',l.get('is_oa'),l.get('version'))
        a=absd(m)
        if a: print('    abs:',a)
    except Exception as e:
        print(d,'ERR',e)
    time.sleep(0.3)
print('======== search')
for q,flt in [('low temperature water gas shift kinetics Cu/ZnO/Al2O3','is_oa:true'),('low-temperature shift reactor copper zinc catalyst kinetic model simulation','is_oa:true'),('hydrogen solubility amine solvent CO2 capture syngas','is_oa:true'),('hydrogen solubility aqueous amine',None),('heat capacity CO2 loaded monoethanolamine','is_oa:true')]:
    u='https://api.openalex.org/works?per-page=8&search='+urllib.parse.quote(q)+(('&filter='+flt) if flt else '')
    try:
        print('Q',q)
        for m in get(u)['results']:
            print('   ',(m.get('doi') or '')[16:],'|',(m.get('title') or '')[:100],'|',m.get('publication_year'),'|',(m.get('open_access') or {}).get('oa_url'))
    except Exception as e: print('ERR',e)
