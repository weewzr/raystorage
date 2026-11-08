import urllib.request,urllib.parse,json,sys,io,time
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
UA={'User-Agent':'Mozilla/5.0 (mailto:weewzrresearch@example.invalid) academic reference check'}
UA={'User-Agent':'Mozilla/5.0 academic reference check'}
for d in ['10.1007/s11814-016-0186-3','10.1016/j.jct.2019.07.007','10.1021/je900188m']:
    try:
        m=json.load(urllib.request.urlopen(urllib.request.Request('https://api.crossref.org/works/'+urllib.parse.quote(d),headers=UA),timeout=20))['message']
        print(d,'|',m.get('title',[''])[0][:130],'|',', '.join((a.get('family') or '') for a in m.get('author',[])[:5]),'|',m.get('container-title',[''])[0],m.get('volume'),m.get('page'))
    except Exception as e: print(d,'ERR',e)
    time.sleep(1.2)
qs=['Yan Chen thermodynamic modeling CO2 solubility aqueous NaCl Na2SO4 electrolyte NRTL Journal of Supercritical Fluids 55 623',
'Koryabkina Phatak Ruettinger Farrauto Ribeiro Determination of kinetic parameters for the water-gas shift reaction on copper catalysts under realistic conditions',
'Mendes Chibante Zheng Tosti Borgognoni Mendes Madeira Determination of the low-temperature water-gas shift reaction kinetics using a Cu-based catalyst',
'Effect of CO2 loading on the solubility of ethane and propane in aqueous alkanolamine solutions',
'Salmi Hakkarainen Kinetic study of the low-temperature water-gas shift reaction over a Cu-ZnO catalyst',
'solubility of hydrogen in aqueous solutions of alkanolamine',
'Specific heat capacity of CO2 loaded aqueous monoethanolamine solution calorimeter measurement']
for q in qs:
    time.sleep(2)
    u='https://api.crossref.org/works?rows=3&select=DOI,title,author,container-title,volume,page,issued&query.bibliographic='+urllib.parse.quote(q)
    for tr in range(3):
        try:
            print('Q:',q[:80])
            for m in json.load(urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=25))['message']['items']:
                print('   ',m['DOI'],'|',(m.get('title') or [''])[0][:100],'|',', '.join((a.get('family') or '') for a in m.get('author',[])[:4]),'|',(m.get('container-title') or [''])[0],m.get('volume'),m.get('page'),m.get('issued',{}).get('date-parts'))
            break
        except Exception as e:
            print('  retry',e); time.sleep(6)
