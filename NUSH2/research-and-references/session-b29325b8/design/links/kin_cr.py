import urllib.request,urllib.parse,json,sys,io
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
UA={'User-Agent':'Mozilla/5.0 (academic reference check)'}
dois=['10.1021/je00051a011','10.1016/S0166-9834(00)81507-9','10.1016/S0166-9834(00)81506-7','10.1021/ie9004578','10.1016/j.supflu.2010.09.038','10.1021/ie1006855','10.1021/acs.iecr.8b01471','10.1021/acs.iecr.8b01472','10.1016/S1385-8947(00)00367-3','10.1021/je970029s','10.1002/cjce.5450760509','10.1002/aic.690420130','10.2202/1542-6580.2238']
for d in dois:
    u='https://api.crossref.org/works/'+urllib.parse.quote(d)
    try:
        r=urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=20)
        m=json.load(r)['message']
        au=', '.join((a.get('family') or '') for a in m.get('author',[])[:6])
        print(d,'OK|',m.get('title',[''])[0][:110],'|',au,'|',m.get('container-title',[''])[0],m.get('volume'),m.get('page'),m.get('issued',{}).get('date-parts'))
        print('   lic:',[l.get('URL') for l in m.get('license',[])][:2],' links:',[l.get('URL') for l in m.get('link',[])][:2])
    except Exception as e:
        print(d,'ERR',e)
qs=['Song Chen Thermodynamic modeling of CO2 aqueous alkanolamine electrolyte NRTL Ind Eng Chem Res 2009 48 7788',
'Yan Chen Thermodynamic modeling of CO2 solubility in aqueous solutions of NaCl and Na2SO4 J Supercrit Fluids 2010',
'Zhang Chen Thermodynamic modeling for CO2 absorption in aqueous MDEA solution with electrolyte NRTL model Ind Eng Chem Res 2011',
'solubility of methane in aqueous solutions of monoethanolamine diethanolamine triethanolamine Carroll Jou Mather Otto',
'Weiland heat capacity of aqueous monoethanolamine diethanolamine N-methyldiethanolamine blends with carbon dioxide',
'solubility of hydrogen in aqueous monoethanolamine',
'solubility of carbon monoxide in aqueous alkanolamine solutions',
'Amadeo Laborde Hydrogen production from the low-temperature water-gas shift reaction: kinetics and simulation of the industrial reactor',
'Ayastuy Kinetics of the low-temperature WGS reaction over a CuO/ZnO/Al2O3 catalyst',
'Koryabkina Determination of kinetic parameters for the water-gas shift reaction on copper catalysts',
'Ovesen microkinetic analysis of the water-gas shift reaction under industrial conditions',
'Smith Loganathan Shantha A review of the water gas shift reaction kinetics',
'Mendes Comparison of nanosized gold-based and copper-based catalysts for the low-temperature water-gas shift reaction',
'Weisenberger Schumpe Estimation of gas solubilities in salt solutions at temperatures from 273 K to 363 K',
'Fernandez-Prini Alvarez Harvey Henry constants and vapor-liquid distribution constants for gaseous solutes in H2O and D2O at high temperatures',
'Sander Compilation of Henry law constants version 5.0.0 for water as solvent',
'Choi Stenger Water gas shift reaction kinetics and reactor modeling for fuel cell grade hydrogen']
for q in qs:
    u='https://api.crossref.org/works?rows=3&select=DOI,title,author,container-title,volume,page,issued&query.bibliographic='+urllib.parse.quote(q)
    try:
        r=urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=25)
        print('Q:',q[:80])
        for m in json.load(r)['message']['items']:
            au=', '.join((a.get('family') or '') for a in m.get('author',[])[:4])
            print('   ',m['DOI'],'|',(m.get('title') or [''])[0][:95],'|',au,'|',(m.get('container-title') or [''])[0],m.get('volume'),m.get('page'),m.get('issued',{}).get('date-parts'))
    except Exception as e:
        print('Q ERR',q[:60],e)
