"""Pre-checks for the V&V matrix. Arithmetic only, no engine code.
XF1 Table 6 pre-exponentials and E, dH from Table 5. K(T) from Y. Zhang 2021 Table 1 (K1 in bar^2 after conversion)."""
import math
R = 8.314e-3  # kJ/mol/K
A = dict(k1=4.225e15, k2=1.955e6, k3=1.020e15, KCO=8.23e-5, KH2=6.12e-9, KCH4=6.65e-4, KH2O=1.77e5)
E = dict(k1=240.1, k2=67.13, k3=243.9)
dH = dict(KCO=-70.65, KH2=-82.90, KCH4=-38.28, KH2O=88.68)
def rate_const(T, scale=1.0):
    k = {n: scale*A[n]*math.exp(-E[n]/(R*T)) for n in E}
    K = {n: A[n]*math.exp(-dH[n]/(R*T)) for n in dH}
    return k, K
def Keq(T):
    K1 = 1.198e13*math.exp(-26830/T)   # bar^2
    K2 = 1.767e-2*math.exp(4400/T)
    return K1, K2, K1*K2
def rates(T, p, scale=1.0):
    k, K = rate_const(T, scale)
    K1, K2, K3 = Keq(T)
    pc, pw, ph, pco, pco2 = p
    den = 1 + K['KCO']*pco + K['KH2']*ph + K['KCH4']*pc + K['KH2O']*pw/ph
    r1 = k['k1']/ph**2.5*(pc*pw - ph**3*pco/K1)/den**2
    r2 = k['k2']/ph*(pco*pw - ph*pco2/K2)/den**2
    r3 = k['k3']/ph**3.5*(pc*pw**2 - ph**4*pco2/K3)/den**2
    return r1, r2, r3, den
# trap T2 check
T=1000.; P=30.; y=dict(c=.15,w=.45,h=.25,co=.03)
r=rates(T,(y['c']*P,y['w']*P,y['h']*P,y['co']*P,0.12*P))
print('DEN^2 at 1000K 30 bar', round(r[3]**2,2))
# SMR integration XF1 Figs 2,3: basis 1 mol CH4; H2O/CH4=3, H2/CH4=1.25, 10 bar
def integrate_smr(T, wf, Pt=10., sw=3., h=1.25, n=4000):
    xc, xco2 = 0.0, 0.0
    dw = wf/n
    def deriv(xc, xco2):
        nc = 1-xc; nco2 = xco2; nco = xc-xco2
        nw = sw - xc - xco2; nh = h + 3*xc + xco2
        nt = nc+nco2+nco+nw+nh
        p = [Pt*v/nt for v in (nc,nw,nh,nco,nco2)]
        r1,r2,r3,_ = rates(T,p)
        return r1+r3, r2+r3
    for i in range(n):
        a=deriv(xc,xco2); b=deriv(xc+dw/2*a[0],xco2+dw/2*a[1]); c=deriv(xc+dw/2*b[0],xco2+dw/2*b[1]); d=deriv(xc+dw*c[0],xco2+dw*c[1])
        xc+=dw/6*(a[0]+2*b[0]+2*c[0]+d[0]); xco2+=dw/6*(a[1]+2*b[1]+2*c[1]+d[1])
    return xc,xco2
for T in (773,798,823,848):
    for wf in (0.10,0.20,0.36):
        print(T,wf,[round(v,4) for v in integrate_smr(T,wf)])
# reverse WGS and methanation XF1 Figs 4,5: 10 bar, H2/CO2=1, rates x 1.225
def integrate_rev(T, wf, Pt=10., h=1.0, scale=1.225, n=4000):
    xco2, xch4 = 0.0, 0.0
    dw=wf/n
    def deriv(x, m):
        # x total CO2 conversion, m conversion of CO2 to CH4; CO = x-m
        nco2=1-x; nch4=m; nco=x-m; nw=x+m; nh=h-(x-m)-4*m  # r2 reverse consumes 1 H2, CO2->CH4 consumes 4 H2
        nw = x + m
        nt=nco2+nch4+nco+nw+nh
        p=[Pt*v/nt for v in (nch4,nw,nh,nco,nco2)]
        r1,r2,r3,_=rates(T,p,scale)
        return -(r2+r3), -(r1+r3)
    x,m=1e-6,1e-7
    for i in range(n):
        a=deriv(x,m); b=deriv(x+dw/2*a[0],m+dw/2*a[1]); c=deriv(x+dw/2*b[0],m+dw/2*b[1]); d=deriv(x+dw*c[0],m+dw*c[1])
        x+=dw/6*(a[0]+2*b[0]+2*c[0]+d[0]); m+=dw/6*(a[1]+2*b[1]+2*c[1]+d[1])
    return x,m
for T in (573,598,623,648,673):
    for wf in (0.10,0.30,0.45):
        print(T,wf,[round(v,4) for v in integrate_rev(T,wf)])
# Chemistry checks
print('--- k2 MEA')
for T in (293,298,303):
    hik = 10**(10.99-2152/T)/1000
    pan = 10**(11.069-2142.34/T)/1000
    pa = {293:9.66,298:9.51,303:9.36}[T]
    bron = math.exp(pa+16.26-7188/T)
    p5b = 4.4e11*math.exp(-5400/T)/1000
    print(T,'Hikita',round(hik,3),'Pandya',round(pan,3),'Bronsted',round(bron,3),'Part5b',round(p5b,3))
print('--- W/F 0.40 SMR')
for T in (773,798,823,848):
    print(T,[round(v,4) for v in integrate_smr(T,0.40)])
