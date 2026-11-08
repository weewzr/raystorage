"""Own calculations for the INL evidence report. Every input is a published INL value
(cited in the report) unless marked ASSUMED. Run: python inl_calcs.py"""
SCF = 379.49         # scf/lbmol at 60 F, 14.696 psia
ST = 0.90718474      # t per short ton
LB = 0.45359237
BTU_J = 1055.056
mm2mw = BTU_J * 1e6 / 3600 / 1e6  # MMBtu/h to MW (0.29307)

# Natural gas, TEV-953 A-5 / TEV-961 A-50
ng = dict(CH4=.93571, C2=.03749, C3=.0092, C4=.0026, C5=.0004, C6=.0001, CO2=.0025, N2=.0119, O2=.0001)
Cn = dict(CH4=1, C2=2, C3=3, C4=4, C5=5, C6=6, CO2=1)
C_per_mol = sum(ng[k]*Cn[k] for k in Cn)
print(f"carbon per mol NG = {C_per_mol:.5f}")

def co2eq_st(mmscfd):
    return mmscfd*1e6/SCF*C_per_mol*44.01/2000
print("\n== carbon closure, short ton/d CO2-eq vs reported captured+emitted ==")
for name, q, cap, em in [("Conv noCC", 52.5, 0, 3205), ("Conv CC", 49.2, 2143, 855), ("HTGR noCC 750", 44.5, 0, 2713),
                         ("Case2", 43.5, 2123, 525), ("Case3", 40.8, 2076, 403), ("Case4", 37.7, 2024, 267),
                         ("Case5", 34.5, 1970, 126), ("Case6", 34.0, 1927, 142)]:
    t = co2eq_st(q)
    print(f"{name:14s} NG {q:5.1f} -> {t:7.1f} ; reported {cap+em:5d} ; closure {100*(cap+em)/t:5.1f}% ; capture {100*cap/(cap+em) if cap else 0:5.1f}% ; in metric would be {t*ST:7.1f} t/d")

print("\n== short ton proof: CO2 mass flow lb/h ==")
for name, lbh, st in [("Case2", 177159, 2123), ("Case5", 164385, 1970)]:
    print(name, "lb/h*24/2000 =", round(lbh*24/2000, 1), "reported", st, "; /2204.6 =", round(lbh*24/2204.62, 1))

print("\n== conversions ==")
for n, v in [("captured Case6", 1927), ("emitted Case6", 142), ("sum", 2069), ("conv noCC", 3205)]:
    print(f"{n}: {v} st/d = {v*ST:.1f} t/d = {v*ST*365*.85/1e6:.4f} Mt/y at 85% ; {v*ST*365/1e6:.4f} at 100%")
print("H2 29000 lb/h:", round(29000*LB, 1), "kg/h ;", round(29000*LB*7446/1000, 1), "t/y @85% ;", round(29000*LB*8760/1000, 1), "t/y @100%")
print("130 MMSCFD check, 99.9%H2 0.1%CH4:", round(130e6/SCF/24*(.999*2.016+.001*16.04), 0), "lb/h")
print("Export 83.7 MMBtu/h =", round(83.7*mm2mw, 2), "MWth ;", round(83.7e6/1023), "lb/h ;", round(83.7e6/1023/29000, 2), "lb/lb H2")
print("Water 743 gpm =", round(743*0.2271, 1), "m3/h ; 354 gpm =", round(354*.2271, 1))
print("CO2 2015 psia =", round(2015*.0689476, 1), "bar ; 184 F =", round((184-32)/1.8, 1), "C")
print("389 psia =", round(389*.0689476, 1), "bar ; 439 psia =", round(439*.0689476, 1), "bar ; 460 psia =", round(460*.0689476, 1), "bar; 445 =", round(445*.0689476, 1), "bar")
print("He duty 78.49*5.193*434 =", round(78.49*5.193*434/1000, 1), "MW ; excess 78.49*5.193*5 =", round(78.49*5.193*5/1000, 2))

print("\n== reactor screening ==")
for e, eta, q, lab in [(11.6, .397, 176.8, "slide 6"), (17.3, .397, 176.8, "Case6 full"), (15.3, .39, 166.6, "Case5 full"),
                       (11.6, .475, 176.8, "11.6 at 47.5%"), (17.3, .475, 176.8, "17.3 at 47.5%")]:
    print(f"{lab}: {q}+{e}/{eta} = {q+e/eta:.1f} MWth")
print("210 MWth at 39.7%: power from (210-176.8) =", round((210-176.8)*.397, 2), "MWe ; shortfall vs 17.3 =", round(17.3-(210-176.8)*.397, 2))
print("Sum of Case 6 rows:", round(1.2+2.1+.3+6.4+5.7+.1+1.5, 1))

print("\n== recycle trend ==")
rows = [("Case2", 82083, 26099, 2244, 23855, 2174, 398.3, 769.3, 785.5, 2.0, 43.5),
        ("Case3", 76875, 26498, 8012, 18485, 7768, 1409.0, 720.5, None, 2.7, 40.8),
        ("Case4", 71091, 27310, 14743, 12568, 14310, 2542.0, 666.3, None, 3.6, 37.7),
        ("Case5", 65046, 29786, 23358, 6428, 22790, 3943.5, 609.7, 188.2, 4.7, 34.5)]
for r in rows:
    n, ng_, lt, rc1, fu, rc2, wc, wn, wa, row, q = r
    print(f"{n}: recycled {100*rc1/lt:5.1f}% of reject; recycle/NG mass {100*rc2/ng_:5.1f}%; kW per lb/h {wc/rc1:.4f}; lb/h per MMSCFD {ng_/q:.1f}; REC-COMP share of NG-reforming row {wc/1000/row*100:.0f}%")
# linear fit NG vs RECY-2, intercept at zero recycle
xs = [r[5] for r in rows]; ys = [r[1] for r in rows]
n = len(xs); mx = sum(xs)/n; my = sum(ys)/n
b = sum((x-mx)*(y-my) for x, y in zip(xs, ys))/sum((x-mx)**2 for x in xs); a = my-b*mx
lbperMM = 1886.5
print(f"fit NG = {a:.0f} {b:+.4f} * RECY-2 ; intercept {a/lbperMM:.2f} MMSCFD ; Case6 34.0 MMSCFD implies RECY-2 = {(34.0*lbperMM-a)/b:.0f} lb/h")
print("Case5->6 NG row change", 1.2-4.7, "MW; Case5 REC-COMP 3.94")
print("NG-COMP scaled to Case 6:", round(609.7*34.0/34.5, 1), "kW -> residual in 1.2 MW row:", round(1200-609.7*34.0/34.5), "kW (includes air blower and pumps)")
for mass in (23358, 26000, 29786, 33000):
    print(f"  recycle compression at {mass} lb/h with 0.165-0.169 kW/(lb/h): {mass*.165/1000:.2f}-{mass*.169/1000:.2f} MW")

print("\n== Case 5 shifted-gas reconstruction ==")
raw = 258e6/SCF/24
y = dict(H2=.488, CO=.087, CO2=.048, N2=.007, H2O=.334, CH4=.036)
mw = dict(H2=2.016, CO=28.01, CO2=44.01, N2=28.01, H2O=18.015, CH4=16.04)
print("raw lbmol/h", round(raw), "mass", round(raw*sum(y[k]*mw[k] for k in y)), "vs 349724")
n_ = {k: raw*y[k] for k in y}
dry0 = raw-n_['H2O']
x = (n_['CO']-.0033*dry0)/(1.0033)
dry = dry0+x
co = n_['CO']-x; co2 = n_['CO2']+x; h2 = n_['H2']+x
print(f"CO shifted {x:.0f}; dry {dry:.0f} lbmol/h = {dry*LB:.0f} kmol/h; CO2 {co2:.0f} ({100*co2/dry:.2f}%, reported 17.7%) = {co2*LB:.0f} kmol/h; H2 {h2:.0f} ({h2*2.016:.0f} lb/h); CH4 {n_['CH4']:.0f}; CO {co:.0f}; N2 {n_['N2']:.0f}")
Hprod = 29000*(.999*2.016)/(.999*2.016+.001*16.04)
print("H2 needed at PSA feed at 88%:", round(Hprod/.88), "lb/h; in reject:", round(Hprod/.88-Hprod), "lb/h =", round((Hprod/.88-Hprod)/2.016), "lbmol/h")
rej = dict(H2=(Hprod/.88-Hprod)/2.016, CH4=n_['CH4']-14-10, N2=n_['N2'], CO=co, CO2=co2*.01)
tot = sum(rej.values()); m = sum(rej[k]*mw[k] for k in rej)
print("reject est lbmol/h", {k: round(v) for k, v in rej.items()}, "total", round(tot), "mass", round(m), "vs 29786; mol%", {k: round(100*v/tot, 1) for k, v in rej.items()})
Cng = 34.5e6/SCF/24*C_per_mol
Crec = (rej['CH4']+rej['CO']+rej['CO2'])*23358/29786
print(f"NG carbon {Cng:.0f} lbmol/h; recycled carbon {Crec:.0f} = {100*Crec/Cng:.1f}% of NG carbon; recycled H2 {rej['H2']*23358/29786:.0f} lbmol/h = {rej['H2']*23358/29786*2.016:.0f} lb/h")
print("N2 in NG", round(34.5e6/SCF/24*.0119, 1), "lbmol/h vs N2 leaving via FUEL", round(rej['N2']*6428/29786, 1))

print("\n== Case 6 estimate (conversion ASSUMED 0.76-0.78 from Cases 2/5) ==")
Cng6 = 34.0e6/SCF/24*C_per_mol
cap = 1927*2000/24/44.01; em = 142*2000/24/44.01
print(f"NG C {Cng6:.0f}; captured {cap:.0f}; emitted {em:.0f}; shifted CO2 at 99% = {cap/.99:.0f} lbmol/h = {cap/.99*LB:.0f} kmol/h = {1927/.99:.0f} st/d")
for X in (.76, .77, .78):
    other = cap/.99*.01 + 70 + 24     # CO2 slip, CO slip, CH4 lost to products
    T = (cap+other)/X
    rejC = (1-X)*T + cap/.99*.01 + 70
    rec = T-Cng6
    print(f" X={X}: tube C {T:.0f}; reject C {rejC:.0f} ({100*rejC/Cng6:.0f}% of NG C); recycled C {rec:.0f} ({100*rec/Cng6:.0f}%); burned C {rejC-rec:.0f} (vs emitted {em:.0f})")
print("N2 in NG Case 6:", round(34.0e6/SCF/24*.0119, 1), "lbmol/h =", round(34.0*.0119, 3), "MMSCFD; CO2 emitted volume", round(142*2000/44.01*SCF/1e6, 2), "MMSCFD; so N2 mole share of N2+carbon in burned gas =", round(100*34.0*.0119/(34.0*.0119+142*2000/44.01*SCF/1e6), 1), "%")

print("\n== no-recycle consequences ==")
q0 = 44.5
print("yield no recycle", round(130/q0, 3), "scf/scf ; tube feed Case 1:", round(49.2*(1-.0902), 2), "->", round(130/(49.2*(1-.0902)), 3))
print("Case 1 noCC tube feed", round(52.5*(1-.1514), 2), "NG fired", round(52.5*.1514, 2))
print("At 34.0 no recycle: H2", round(34.0*130/q0, 1), "MMSCFD;", round(34.0/q0*29000), "lb/h;", round(34.0/q0*29000*LB*7446/1000), "t/y; change", round(100*(34.0/q0-1), 1), "%")
print("At 130 no recycle: NG", q0, "delta", round(q0-34.0, 1), round(100*(q0/34.0-1), 1), "%")
gj = (q0-34.0)*1e6*1044*BTU_J/1e9
print(f"Extra NG energy {gj:.0f} GJ/d; {gj*310.25/1e6:.3f} PJ/y; at S$15/GJ = S${gj*310.25*15/1e6:.1f}M/y")
print("NG energy saved vs 52.5: at 34.0 =", round((52.5-34.0), 1), " at 44.5 =", round(52.5-44.5, 1), " at 43.5 =", round(52.5-43.5, 1))
for nm, d in [("slide 25 43.5", 9.0), ("34.0", 18.5), ("44.5", 8.0)]:
    g = d*1e6*1044*BTU_J/1e9
    print(f"  avoided NG {nm}: {g:.0f} GJ/d, S${g*310.25*15/1e6:.1f}M/y")
print("CO2 total no recycle at 130:", round(co2eq_st(44.5)), "st/d; syngas CO2 anchored on Case 1:", round(2143/.99*44.5/44.76), "st/d; 90% capture:", round(.9*2143/.99*44.5/44.76), "st/d =", round(.9*2143/.99*44.5/44.76*ST*365*.85/1e6, 3), "Mt/y; rest emitted:", round(co2eq_st(44.5)-.9*2143/.99*44.5/44.76), "st/d")
print("CO2 total no recycle at 34.0:", round(co2eq_st(34.0)), "st/d; syngas CO2:", round(2143/.99*34.0/44.76), "; 90%:", round(.9*2143/.99*34.0/44.76), "; rest:", round(co2eq_st(34.0)-.9*2143/.99*34.0/44.76))

print("\n== 90% capture basis ==")
s = 1927/.99
print(f"Syngas CO2 Case 6 {s:.0f} st/d; 90% = {.9*s:.0f} st/d = {.9*s*ST:.0f} t/d = {.9*s*ST*365*.85:.0f} t/y; report 90% of 2069 = {.9*2069:.0f} st/d = {.9*2069*ST*365*.85:.0f} t/y; diff {(.9*2069-.9*s)*ST*365*.85:.0f} t/y")
print(f"Emitted at 90% of syngas {2069-.9*s:.0f} st/d = {(2069-.9*s)*ST*365*.85:.0f} t/y vs report 58,233; slip {.1*s:.0f} st/d vs INL 1% {.01*s:.0f}")

print("\n== heat inventory ==")
for n, v in [("Selexol Case5 65.0", 65.0), ("Selexol Case2 65.5", 65.5), ("ZLD 56.7", 56.7), ("Export C6 83.7", 83.7), ("Export C5 54.3", 54.3), ("HPS-GEN1", 236.3), ("HPS-GEN2", 45.3), ("LPS-GEN", 5.3), ("WGS-EX1", 66.7), ("WGS-EX2", 24.2), ("WGS-EX3", 65.7), ("WGS-EX4", 117.1), ("KO", 19.7), ("HW cooler", 74.1), ("HW users", 43.0), ("REFORM-1", 404.2), ("REFORM-2", 74.0)]:
    print(f"{n}: {v*mm2mw:.1f} MWth")
print("Selexol steam specific:", round(65.0e6/71874), "Btu/lb; Case2", round(65.5e6/72398), "; per t CO2 Case5:", round(65.0*BTU_J*1e6/1e9/(164385*LB/1000), 2), "GJ/t")
# split of WGS-EX4 above 130C (266F), Case 5
dryl = dry
def psat_psia(TF):
    import math
    TC = (TF-32)/1.8
    return 10**(8.07131-1730.63/(233.426+TC))/760*14.696   # Antoine, valid to 100C; for higher use below
def psat_hi(TF):
    import math
    T = (TF-32)/1.8+273.15
    return 14.696/1.01325*math.exp(11.78*(T-373.15)/(T-43))*1.01325  # Clausius-type fit
for TF in (300, 266, 250, 158):
    p = psat_hi(TF)
    yw = p/390
    nw = yw/(1-yw)*dryl
    print(f"T {TF}F psat {p:.1f} psia nw {nw:.0f} lbmol/h")
nw_in = raw*.334-x
print("water after shift", round(nw_in), "lbmol/h = ", round(nw_in*18.015), "lb/h; y", round(nw_in/(nw_in+dryl), 3), "; dew p", round(nw_in/(nw_in+dryl)*390, 1), "psia; COND reported 126553")
cp_dry = 7.2  # Btu/lbmol/F ASSUMED mean for H2-rich dry gas
def seg(T1, T2, hfg):
    n1 = psat_hi(T1)/390/(1-psat_hi(T1)/390)*dryl; n2 = psat_hi(T2)/390/(1-psat_hi(T2)/390)*dryl
    lat = (n1-n2)*18.015*hfg/1e6
    sen = dryl*cp_dry*(T1-T2)/1e6
    return lat, sen, lat+sen
print("WGS-EX4 part 300->266F:", [round(v, 1) for v in seg(300, 266, 925)], "MMBtu/h ->", round(seg(300, 266, 925)[2]*mm2mw, 1), "MWth")
print("WGS-EX4 part 300->250F:", [round(v, 1) for v in seg(300, 250, 930)], "MMBtu/h ->", round(seg(300, 250, 930)[2]*mm2mw, 1), "MWth")

print("\n== MEA balance (project 76-79 MWth, amine_spec line 55) ==")
sup = dict(selexol_freed=65.0*mm2mw, export_C6=83.7*mm2mw, wgs4_above130=seg(300, 266, 925)[2]*mm2mw)
print({k: round(v, 1) for k, v in sup.items()}, "sum", round(sum(sup.values()), 1), "shortfall", round(76-sum(sup.values()), 1), round(79-sum(sup.values()), 1), "; with ZLD steam", round(sum(sup.values())+56.7*mm2mw, 1))
print("Reject heat, Case 5 LHV est:", round((rej['H2']*2.016*51600+rej['CH4']*16.04*21500+rej['CO']*28.01*4347)/1e6, 0), "MMBtu/h =", round((rej['H2']*2.016*51600+rej['CH4']*16.04*21500+rej['CO']*28.01*4347)/1e6*mm2mw, 0), "MWth")
print("He circulator est: rho", round(65.84e5*4.0026e-3/(8.314*739), 2), "kg/m3 at 955 psia 466C; dP 60 psi =", round(60*.0689476, 2), "bar; ideal", round(78.49/(65.84e5*4.0026e-3/(8.314*739))*60*6894.76/1e6, 2), "MW; at 80% eff", round(78.49/(65.84e5*4.0026e-3/(8.314*739))*60*6894.76/1e6/.8, 2))

print("\n== added checks ==")
# Case 5 S/C closure and heat sums
print("Case5 water to tubes lbmol/h", round((248376+13513)/18.015), "tube C at S/C 3:", round((248376+13513)/18.015/3), "implied recycled C:", round((248376+13513)/18.015/3-3992))
print("Case5 process heat sum", round(10.9+5.0+14.3+36.4+36.3+61.3+404.2, 1), "vs 568.4; Case2", round(45.3+4.3+15.6+32.4+38.8+59.0+203.3, 1), "vs 398.8")
print("LPS users Case5", 65.0+56.7, "vs 121.7; Case2", 65.5+82.3, "vs 147.8; HW users Case2", round(9.1+51.7+3.5, 1), "vs 64.3")
# WGS-EX3 validation of the condensation model, 415 to 300 F, Case 5
n3 = psat_hi(300)/390/(1-psat_hi(300)/390)*dry
latent = (nw_in-n3)*18.015*890/1e6
sens_dry = dry*cp_dry*(415-300)/1e6
sens_wv = nw_in*8.3*(415-300)/1e6
print("WGS-EX3 model MMBtu/h latent", round(latent, 1), "dry sens", round(sens_dry, 1), "vapour sens", round(sens_wv, 1), "total", round(latent+sens_dry+sens_wv, 1), "vs published 65.7")
# Rough Case 6 absorber CO2 fraction, H2 at absorber between net make and PSA-feed requirement
co2_6 = cap/.99
for h2a in (14270, 16220):
    for ch4 in (1000, 1200):
        tot6 = co2_6+h2a+ch4+70+100
        print(f"H2 {h2a} CH4 {ch4}: dry total {tot6:.0f} lbmol/h, CO2 {100*co2_6/tot6:.1f}%")
print("Case 2 reject fired share", round(100*23855/26099, 1), "Case 5 fired share", round(100*6428/29786, 1))
print("Burned C share of NG C", round(100*261/3934, 1), "to", round(100*269/3934, 1))
print("Selexol share of MEA demand", round(100*19.0/76, 0), round(100*19.0/79, 0))
print("5.7 MWe at S$0.20/kWh", round(5.7*7446*200/1e6, 2), "; 4.12 MWe", round(4.12*7446*200/1e6, 2))
print("CO2 partial pressure bar: Case5", round(.177*26.8, 2), "Case2", round(.189*26.8, 2), "spec", round(.18*21, 2))
print("Case 5 absorber feed t/h", round(223172*LB/1000, 1))
print("Scaled He heat 176.8*49.2/34.0 =", round(176.8*49.2/34.0, 1), "; H2 kg/s 3.82*49.2*(3.63/130) =", round(3.82*49.2*3.63/130, 2))
