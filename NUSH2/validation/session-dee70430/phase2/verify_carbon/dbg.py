import sympy as sp
R_=sp.Rational
ATOMS={"ch4":(1,4,0),"c2h6":(2,6,0),"c3h8":(3,8,0),"nc4h10":(4,10,0),"nc5h12":(5,12,0),"n2":(0,0,0),"co2":(1,0,2)}
y={"ch4":R_("0.93571"),"c2h6":R_("0.03749"),"c3h8":R_("0.0092"),"nc4h10":R_("0.0026"),"nc5h12":R_("0.00052"),"n2":R_("0.012"),"co2":R_("0.0025")}
hbar=sum(y[k]*ATOMS[k][1] for k in y); obar=sum(y[k]*ATOMS[k][2] for k in y); cb=sum(y[k]*ATOMS[k][0] for k in y)
ZN=2*cb-obar+hbar/2
n,T,Rr,P,hp=sp.symbols("n T R P hp")
f,r,xi,eta,ph,ps,om,ka,pi=sp.symbols("f r xi eta phi psi omega kappa pi")
SC=3
ch4R=f*(ps*T-P); coR=f*om*T; co2R=f*(1-eta)*ph*T
Hin=hbar*n+4*ch4R+2*SC*T
Oin=obar*n+coR+2*co2R+SC*T
H2O09=Oin-om*T-2*(ph+ka)*T
H2_09=(Hin-4*ps*T-2*H2O09)/2
# general check of H2_09 against Z formula
Zin=ZN*n+4*ch4R+coR
lhs=sp.simplify(H2_09-(Zin-4*ps*T-om*T))
print("H2_09 minus Z-balance H2:",lhs)

# solve the system with exact numbers where shares sum to 1
vals={f:R_("0.752178"),r:R_("0.92163"),xi:R_("0.88"),eta:R_("0.9"),ph:R_("0.822639"),ps:R_("0.159597"),ka:R_("0.003527"),pi:R_("0.001")/R_("0.999"),hp:R_("4458.598726")}
vals[om]=1-vals[ph]-vals[ps]-vals[ka]
eqs=[sp.Eq(T,cb*n+Rr),sp.Eq(Rr,f*(T-ka*T-eta*ph*T-P)),sp.Eq(P,pi*hp),sp.Eq(hp,xi*(H2_09/(1-r*(1-xi))))]
sol=sp.solve([e.subs(vals) for e in eqs],[T,Rr,P,n],dict=True)[0]
n_lin=sol[n]
D=1-f*(1-ka-eta*ph); chi=(1-f)*(4*ps+om)/D
xe=xi/(1-r*(1-xi))
n_dr=(hp*(1/xe+f*pi*(4-chi))/(ZN-chi*cb)).subs(vals)
print(float(n_lin),float(n_dr),float(n_lin-n_dr))

import random
sym_diff = None
omsub={om:1-ph-ps-ka}
sysq=[sp.Eq(T,cb*n+Rr),sp.Eq(Rr,f*(T-ka*T-eta*ph*T-P)),sp.Eq(P,pi*hp),sp.Eq(hp,xi*(H2_09/(1-r*(1-xi))))]
sol=sp.solve([e.subs(omsub) for e in sysq],[T,Rr,P,n],dict=True)[0]
D=1-f*(1-ka-eta*ph); chi=(1-f)*(4*ps+om)/D
xe=xi/(1-r*(1-xi))
n_dr=(hp*(1/xe+f*pi*(4-chi))/(ZN-chi*cb)).subs(omsub)
d=sol[n]-n_dr
random.seed(1)
mx=0
for _ in range(6):
    pt={s:sp.Rational(random.randint(5,95),100) for s in [f,r,xi,eta,ph,ps,ka,pi]}
    pt[hp]=sp.Rational(random.randint(1000,5000))
    pt[ph]=sp.Rational(random.randint(30,60),100); pt[ps]=sp.Rational(random.randint(10,30),100); pt[ka]=sp.Rational(1,100)
    mx=max(mx,abs(float(d.subs(pt))))
print("max |diff| at random exact points:",mx)
print("free symbols:",d.free_symbols)
print(sp.factor(sp.together(d)).is_zero, sp.count_ops(d))
