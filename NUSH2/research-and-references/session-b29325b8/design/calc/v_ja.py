import math
# JAERI A-2-b checks. All numbers from PDF p100 to p105 (printed 86 to 91).
L=[1.87,1.5,1.5,1.5]; D=[100.4,76.4,85.5,95.4]
print('sum L',sum(L),'weighted dT',sum(a*b for a,b in zip(L,D))/sum(L))
T=1018.16;Tm=745;gam=1.935
lam=2.5542e-3*T**.69+7.9378e9*Tm/(Tm**5+4.29e14)+2.0038e-4*gam+2.0554e-8/(.52+T/569.6)
mu=3.8545e-8*T**.69+5.0985e-8/(.52+T/569.6)+2.7226e-11*gam**2
print('lam kcal/m/h/C',lam,'W/m/K',lam*1.163,'mu kgf s/m2',mu,'uPa s',mu*9.807e6)
print('Petersen mu uPa s',3.674e-7*T**.7*1e6,'k W/m/K',2.682e-3*(1+1.123e-3*41.11)*T**(.71*(1-2e-4*41.11)))
b=4.5e-4+5.42/(1890+T); print('Gammah',(math.sqrt(1+4*41.11*b/0.0207723/T)-1)/2/b)
# Ergun, gas property values printed PDF p103
eps=.45;d=.005;u=.6333;rho=9.532;mu=5.82e-5;Lb=6.0
print('Ergun kPa',(150*mu*(1-eps)**2*u/(eps**3*d**2)+1.75*rho*(1-eps)*u**2/(eps**3*d))*Lb/1e3)
# X check, A-2
for n,(co,co2,ch4) in dict(A2=(6.74,11.48,14.6),A3=(7.31,11.34,14.24),A4=(6.1,11.7,16.1),A5=(6.89,11.46,15.16)).items():
    print(n,'X',round((co+co2)/(co+co2+ch4),4))
# H balance on dry gas A-2: Nm3/h H2 from carbon
