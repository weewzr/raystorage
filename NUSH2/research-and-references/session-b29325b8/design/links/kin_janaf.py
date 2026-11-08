import urllib.request,math
def tab(c):
    t=urllib.request.urlopen(urllib.request.Request('https://janaf.nist.gov/tables/%s.txt'%c,headers={'User-Agent':'Mozilla/5.0'}),timeout=30).read().decode('latin1')
    d={}
    for l in t.splitlines():
        p=l.split('\t')
        try: d[float(p[0])]=float(p[-1])
        except: pass
    print(c,t.splitlines()[0],len(d))
    return d
co,co2,ch4,h2o=tab('C-093'),tab('C-095'),tab('C-067'),tab('H-064')
print('T K | K2 JANAF | K2 Zhang | ratio Z/J | K1 JANAF bar2 | K1 Zhang | ratio')
for T in [400,500,600,700,800,900,1000,1100,1200]:
    k2=10**(co2[T]-co[T]-h2o[T]); k1=10**(co[T]-ch4[T]-h2o[T])
    z2=1.767e-2*math.exp(4400/T); z1=1.198e13*math.exp(-26830/T)
    print(T,'%.4g %.4g %.3f | %.4g %.4g %.3f'%(k2,z2,z2/k2,k1,z1,z1/k1))
