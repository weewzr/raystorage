import math
def K(T):
    K1=1.198e13*math.exp(-26830/T); K2=1.767e-2*math.exp(4400/T); return K1,K2
def eq(TC,P,sc,inert=0.0,c2=0.0):
    T=TC+273.15; K1,K2=K(T)
    # extents a (SMR), b (WGS), basis 1 mol CH4
    def res(a):
        # solve b from WGS by bisection
        lo,hi=-a+1e-12,min(a,sc-a)-1e-12
        lo=max(lo,0.0) if False else lo
        f=lambda b:(a-b)*(sc-a-b)*K2-(3*a+b)*b
        for _ in range(200):
            m=(lo+hi)/2
            if f(lo)*f(m)<=0: hi=m
            else: lo=m
        b=(lo+hi)/2
        n=[1-a,sc-a-b,3*a+b,a-b,b]; nt=sum(n)+inert
        p=[P*v/nt for v in n]
        return p[2]**3*p[3]/(p[0]*p[1])-K1,b,n,nt
    lo,hi=1e-6,1-1e-9
    for _ in range(200):
        m=(lo+hi)/2
        if res(m)[0]>0: hi=m
        else: lo=m
    r=res((lo+hi)/2); n=r[2]; nt=r[3]
    return (lo+hi)/2,[round(100*v/nt,2) for v in n]
print('order CH4 H2O H2 CO CO2')
for lab,a in (('TE T2.1 800C 41.9',(800,41.9,3.98)),('TE T2.1 800C 20',(800,20,3.98)),('TE T2.2 825',(825,41.9,3.98)),('TE T2.2 850',(850,41.9,3.98)),('TE T2.2 860',(860,41.9,3.98)),('INL 871C 30.3 S/C3',(871,30.27,3.0)),('HTTR-SR 830C 45 S/C3.5',(830,45,3.5)),('A-2 755C 41.4 S/C3.53',(755,41.4,3.53)),('A-3 760C 41.4 S/C3.43',(760,41.4,3.43)),('A-4 740C 41.4 S/C3.52',(740,41.4,3.52)),):
    print(lab,eq(*a))
