R=8.314462618; M=2.01588e-3  # kg/mol (H2, 2*1.00794)
for lab,T,P in [("0C,101.325kPa (NIST EOS 0.0898825)",273.15,101325.0),("15C,101.325kPa",288.15,101325.0),("20C,101.325kPa",293.15,101325.0),("25C",298.15,101325.0),("0C,100kPa",273.15,100000.0)]:
    print(lab, "ideal-gas rho = %.5f kg/m3"%(P*M/(R*T)))
# real-gas Z at 0C 1atm from NIST value
rho_nist=0.0898825
Z=101325*M/(R*273.15*rho_nist); print("Z(0C,1atm)=%.5f"%Z)
# IEAGHG implied
print("IEAGHG implied 8.994 t/h per 1e5 Nm3/h ->", 8994/1e5, "kg/Nm3; ratio to NIST =", (8994/1e5)/rho_nist)
# methane stoichiometric CO2 per GJ LHV
MC=12.011; MCO2=44.0095; MCH4=16.043
LHV_CH4=802.3 # kJ/mol (not sourced here; check) 
print("CH4 CO2/GJ LHV (kg):", MCO2/LHV_CH4*1e3/1e0 /1.0 * 1.0)
