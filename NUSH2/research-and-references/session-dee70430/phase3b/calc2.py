nm3=100000.0; h2=8994.0
print("NG kg/kg H2", round(30563/h2,4), " feed", round(26231/h2,4), " fuel", round(4332/h2,4), " kg/Nm3", 30563/nm3)
print("Raw water L/Nm3", 59700/nm3, " kg/kg H2", round(59700/h2,3))
print("DMW makeup t/h 45.9 -> kg/kg H2", round(45900/h2,3))
print("Sea water m3/h 3435 -> L/Nm3", 3435*1000/nm3, " m3/t H2", round(3435/ (h2/1000),1))
print("Effluent+losses: waste water 15.5 t/h (BFD 15500 kg/h) kg/kg", round(15500/h2,3))
print("Flue gas kg/kg H2", round(257698/h2,2))
print("HP steam export kg/kg H2", round(46053/h2,3), " process steam", round(95301/h2,3), " steam to C: process steam 95301 kg/h vs carbon in feed? n/a")
print("Sea water pumps share of BoP load %", round(335/347*100,1), " of net export %", round(335/9918*100,2))
print("HHV basis: NG MW", round(30.563*51.473/3.6,2), " H2 MW", round(8.994*141.88/3.6,2), " HHV eff %", round(8.994*141.88/(30.563*51.473)*100,2))
print("LHV/HHV NG", round(46.502/51.473,4))
print("NG price 6 EUR/GJ LHV -> EUR/GJ HHV", round(6*46.502/51.473,4), " S$/GJ HHV at 2.322", round(6*46.502/51.473*2.322,3))
print("repo 15 S$/GJ vs", round(15/(6*46.502/51.473*2.322)-1,4))
# bridge
c_capital=2.716; c_fixed=0.928; c_fuel=8.527; c_ee=0.793; c_nonfuelvar=0.064
f=2.322; kg=0.08994
def sk(c): return c/100/kg*f   # c EUR/Nm3 -> S$/kg
print("report LCOH S$/kg", round(sk(11.441),3))
gas_up = 15/(6*46.502/51.473*2.322)-1
print("gas effect S$/kg", round(sk(c_fuel*gas_up),3))
avail = 95/85-1
print("availability effect on capital+fixed S$/kg (first order)", round(sk((c_capital+c_fixed)*avail),3))
print("EE credit if not credited S$/kg", round(sk(c_ee),3))
print("sum", round(sk(11.441)+sk(c_fuel*gas_up)+sk((c_capital+c_fixed)*avail)+sk(c_ee),3), " repo 3.618")
print("sum with EE credit kept at 0.2 S$/kWh instead of 0.18576: EE credit scales", round(sk(c_ee*0.2/0.18576),3))
# chemicals scaling impact
print("chem/cat scaling impact EUR/y", round(420000*(1-85/95)), " S$M/y", round(420000*(1-85/95)*2.322/1e6,3))
# contingency
print("contingency share of TPC %", round(28.492/170.952*100,3), " of installed %", round(28.492/142.46*100,3))
# island steam bound
export=46.053; proc=95.301; tot=export+proc
lowex=0.75*tot-proc
print("retained export lower bound t/h", round(lowex,2), " fraction", round(lowex/export,3), " gross MWe at 250.5 kWh/t", round(lowex*250.5/1000,2), " net after 1.582", round(lowex*250.5/1000-1.582,2))
print("share of export steam as share of total", round(export/tot,4))
# energy of the 24e6 kcal/h condenser share of NG input
print("PSA tail gas flow 60658 kg/h; NG fuel share of NG total %", round(4.332/30.563*100,2))
# per tonne of H2 at repository ledger: TPC S$/ (t/h)
print("TPC S$M per t/h H2", round(170.952/8.994*2.322,2), " TCR", round(222.886722/8.994*2.322,2))
print("Annual H2 t/y at 8322 h", round(8.994*8322,1), " at 85% (7446 h)", round(8.994*0.85*8760,1))
print("report O&M net EUR/kg H2 (annual O&M cost/H2 kg)", round(72427258/(8.994*1000*8322),4))
print("capital recovery factor 8%, 25 y", round(0.08/(1-1.08**-25),5), " annualised TCR M EUR/y", round(222.886722*0.08/(1-1.08**-25),3))
