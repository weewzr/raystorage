# Equipment to Literature Map

For each equipment item in the proposed HTGR, IHX, helium-heated SMR, WGS and MEA capture train, this map lists the papers whose actual contents support that use. Reference IDs are defined in `Literature_Inventory.csv`, and each category folder has an `INDEX.md` with the full citation, DOI or official link and page references. PDF page numbers are PDF pages; printed page numbers are marked as printed. Where no source has been verified the entry reads Not yet identified. Papers are never assigned on the strength of a filename or title alone.

## 1. HTGR reactor

### Reference design

- REF-HTGR-001 (IAEA 2003, TECDOC-1382): HTTR and HTR-10 benchmark problem descriptions (Ch. 2 and Ch. 3; the HTR-10 core physics chapter is not in the file)
- REF-IHX-001 (Tochio and Nakagawa 2011): Table 1, HTTR specifications (PDF p. 3)
- REF-INT-003 (IAEA 2017, NP-T-4.3): concept design parameters for GTHTR300C, HTR-MODUL, HTR-10, GT-MHR and MHR-100SMR (Tables 16 to 20, PDF pp. 38-43)

### Governing equations

- Not yet identified. REF-HTGR-001 reports the participants' calculation methods, not one consolidated equation set.

### Numerical parameter values

- REF-HTGR-001 (IAEA 2003, TECDOC-1382): benchmark parameters (Ch. 2 and Ch. 3)
- REF-IHX-001 (Tochio and Nakagawa 2011): Table 1, HTTR specifications (PDF p. 3)
- REF-INT-003 (IAEA 2017, NP-T-4.3): thermal power, helium temperatures and pressure for those concepts; for example MHR-100SMR 215 MW(th), 450/950 C, 5.0 MPa (Table 20, PDF p. 43)

### Experimental validation

- REF-HTGR-001 (IAEA 2003, TECDOC-1382): measurements from initial HTTR and HTR-10 testing compared with participants' calculations
- REF-IHX-001 (Tochio and Nakagawa 2011): HTTR 50-day high-temperature continuous operation data

### Supporting references

- REF-INT-002 (Zhang et al. 2021): HTR-PM based hydrogen production system analysis
- REF-SMR-004 (Hoseinzade and Adams 2017): Tables 5-6, design parameters for an SMR/HTGR at industrial scale (PDF pp. 10-11)

Note: No source for the proposed 210 MWth rating was found: a text search of the searchable papers in the library returned no 210 MWth figure. The closest rating found is the 215 MW(th) MHR-100SMR concept in REF-INT-003 (Table 20, PDF p. 43).

## 2. Primary helium circulator

### Reference design

- Not yet identified

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- REF-HTGR-001 (IAEA 2003, TECDOC-1382): coast-down of primary and secondary helium circulators in the loss-of-power benchmark (PDF p. 158); a scenario description, not circulator design
- REF-IHX-001 (Tochio and Nakagawa 2011): primary gas circulator named in the system notation (PDF p. 4)

## 3. Intermediate heat exchanger

### Reference design

- REF-IHX-001 (Tochio and Nakagawa 2011): Table 2, major specifications of the IHX (PDF p. 5)

### Governing equations

- REF-INT-002 (Zhang et al. 2021): energy balance equation for the IHX in the primary helium loop (PDF pp. 4-5)
- REF-IHX-001 evaluates heat exchange performance; whether it states the equations used has not been verified.

### Numerical parameter values

- REF-IHX-001 (Tochio and Nakagawa 2011): Table 2, major specifications of the IHX (PDF p. 5)
- REF-INT-003 (IAEA 2017, NP-T-4.3): primary and secondary helium temperatures at the IHX for the GTHTR300C hydrogen cogeneration concept (Table 16, PDF p. 38); a typical helium temperature drop of about 50 C across the IHX is stated on PDF p. 26

### Experimental validation

- REF-IHX-001 (Tochio and Nakagawa 2011): 50-day continuous operation, heat exchange performance compared with design; Table 3 measurement equipment and accuracy (PDF p. 6)

### Supporting references

- REF-SMR-003 (Yin et al. 2007): heat flow from the IHX outlet to the reformer inlet (PDF p. 6)
- REF-SMR-004 (Hoseinzade and Adams 2017): states that an IHX is present in both designs studied (PDF p. 8)

## 4. Secondary helium loop

### Reference design

- REF-IHX-001 (Tochio and Nakagawa 2011): secondary helium side of the IHX (Table 2); secondary-side entries not yet checked line by line

### Governing equations

- Not yet identified

### Numerical parameter values

- REF-IHX-001 (Tochio and Nakagawa 2011): IHX specifications (Table 2); same caveat

### Experimental validation

- REF-IHX-001 (Tochio and Nakagawa 2011): continuous-operation data; secondary helium side not page-verified

### Supporting references

- REF-HTGR-001 (IAEA 2003, TECDOC-1382): coast-down of secondary helium circulators (PDF p. 158)
- REF-SMR-005 (JAERI 2005, JAERI-Tech 2005-031): helium gas cooler among the mock-up facility heat exchangers (abstract only)

## 5. Helium-heated SMR

### Reference design

- REF-SMR-003 (Yin et al. 2007): Table 1, main parameters of the HTR-10 steam reformer (PDF p. 4)
- REF-SMR-005 (JAERI 2005, JAERI-Tech 2005-031): specifications and structures of the mock-up steam reformer and other heat exchangers
- REF-INT-001 (Sato et al. 2006): Table 1, design specifications of the mock-up test facility (PDF p. 2)
- REF-SMR-004 (Hoseinzade and Adams 2017): Tables 1-2 and 5-6, design specifications and parameters (PDF pp. 8-11)
- REF-INT-003 (IAEA 2017, NP-T-4.3): MHR-100SMR steam reformer built from three heat exchangers HX1 to HX3 (Table 20, PDF p. 43); AVR-II reformer (Table 15, PDF p. 37)

### Governing equations

- REF-SMR-001 (Xu and Froment 1989 Part 1): reforming, water-gas shift and methanation rate equations
- REF-SMR-002 (Xu and Froment 1989 Part 2): pellet diffusion and reactor simulation equations
- REF-SMR-003 (Yin et al. 2007): steady-state pseudo-homogeneous one-dimensional reformer model
- REF-SMR-004 (Hoseinzade and Adams 2017): dynamic two-dimensional model (mass, momentum and energy)
- REF-INT-001 (Sato et al. 2006): Table 2, chemical equations and reaction rate equations used (PDF p. 4)
- REF-INT-002 (Zhang et al. 2021): thermodynamic equilibrium model

### Numerical parameter values

- REF-SMR-001 (Xu and Froment 1989 Part 1): Tables 4-5 and 7, parameter estimates and activation energies (PDF pp. 6-8)
- REF-SMR-002 (Xu and Froment 1989 Part 2): Tables 1-2, pore size distribution and simulation input data (PDF pp. 2 and 4)
- REF-SMR-004 (Hoseinzade and Adams 2017): Table 3, estimated parameters (PDF p. 9)
- REF-INT-002 (Zhang et al. 2021): Table 1, reaction equilibrium constants (PDF p. 5)
- REF-INT-003 (IAEA 2017, NP-T-4.3): reformer thermal power, helium and steam-methane flow rates and temperatures (Tables 15 and 20, PDF pp. 37 and 43)

### Experimental validation

- REF-SMR-003 (Yin et al. 2007): agreement with experimental data
- REF-SMR-005 (JAERI 2005, JAERI-Tech 2005-031): measured overall heat transfer coefficients compared with calculated values
- REF-SMR-006 (Ishii et al. 2023, not in library): validation against HTTR mock-up tests; NOT in the library, obtain through the DOI
- REF-SMR-001 (Xu and Froment 1989 Part 1): the authors' own experiments (Table 1 conditions, PDF p. 2)

### Supporting references

- REF-INT-001 (Sato et al. 2006): system-level dynamic behaviour of the HTTR-coupled reforming system
- REF-SUP-002 (IEAGHG 2022, 2022-07): process data for a conventional fired SMR (Tables 19 and 20, PDF p. 114)

## 6. WGS reactor

### Reference design

- Not yet identified. No source on WGS reactor sizing or configuration.

### Governing equations

- REF-WGS-002 (Kraussler and Hofbauer 2017): kinetic model for Fe/Cr catalyst
- REF-WGS-001 (Chinchen et al. 1984): mass transport and pore diffusion analysis

### Numerical parameter values

- REF-WGS-002 (Kraussler and Hofbauer 2017): Table 3, parameters of the improved kinetic model (PDF p. 8)
- REF-WGS-001 (Chinchen et al. 1984): activation energy about 121 kJ/mol (abstract); Tables 1-5 (PDF pp. 6-13)

### Experimental validation

- REF-WGS-002 (Kraussler and Hofbauer 2017): Tables 4-5, measured and calculated outlet composition (PDF pp. 9-10)
- REF-WGS-001 (Chinchen et al. 1984): microreactor and semi-technical plant data at pressures up to 30 bar

### Supporting references

- REF-SMR-001 (Xu and Froment 1989 Part 1): water-gas shift rate within the reforming network; this is on a Ni catalyst, so it is not a shift-catalyst source
- REF-SUP-002 (IEAGHG 2022, 2022-07): syngas composition before the WGS reactor and cooling of the product gas to about 340 to 350 C ahead of it (PDF pp. 18, 32 and 58); a process description, not reactor design

## 7. Syngas cooler

### Reference design

- Not yet identified

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- REF-SUP-002 (IEAGHG 2022, 2022-07): states only that the product gas is cooled to about 340 to 350 C before the WGS reactor (PDF p. 58); no cooler design

Note: No document in the library describes a syngas cooler design.

## 8. Condensate separator

### Reference design

- Not yet identified

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- Not yet identified

Note: No document in the library describes a process condensate separator.

## 9. MEA absorber

### Reference design

- REF-MEA-006 (Sønderby et al. 2013): pilot packed absorber column (Tables 1-6, PDF pp. 4-8)
- REF-MEA-004 (Morgan et al. 2018): NCCC absorber, Table 1 (PDF p. 4)
- REF-MEA-005 (Morgan et al. 2018, supporting information): Table S1, NCCC absorber variables
- REF-MEA-002 (Plaza 2012): pilot plant studies (Ch. 4, printed p. 92)
- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): plant-scale PCC absorber design basis (Vol. II, printed pp. 53-73)
- REF-MEA-008 (Tan et al. 2015): pilot packed column, inner diameter 0.145 m, 1.0 m of FLEXIPAC 1Y packing (PDF p. 3)

### Governing equations

- REF-MEA-002 (Plaza 2012): absorber model development (Ch. 3, printed p. 23)
- REF-MEA-003 (Cownden and Lucquiaud 2026): improved mass transfer and thermodynamic/chemistry models
- REF-MEA-004 (Morgan et al. 2018): rigorous modelling framework, steady-state validation
- REF-MEA-006 (Sønderby et al. 2013): rate-based packed-column model
- REF-MEA-009 (Wong et al. 2016): CO2 loading calculation and equilibrium relations (eqns (2) to (7)) with the Deshmukh-Mather activity coefficient model (PDF pp. 3-4)

### Numerical parameter values

- REF-MEA-003 (Cownden and Lucquiaud 2026): regressed model parameters (Tables 1-3, PDF pp. 3, 5, 10)
- REF-MEA-001 (Dugas 2009): rate and CO2 partial pressure results and modelling (Ch. 4-5, printed pp. 60-94)
- REF-MEA-002 (Plaza 2012): Ch. 3 model parameters (printed p. 23 onward)
- REF-MEA-009 (Wong et al. 2016): Table 2, equilibrium constant parameters, and Table 3, coefficients for selected ions or molecules (PDF pp. 3-4)

### Experimental validation

- REF-MEA-004 (Morgan et al. 2018): NCCC steady-state data (Table 1, PDF p. 4)
- REF-MEA-005 (Morgan et al. 2018, supporting information): Tables S1 and S3, absorber data and model comparison
- REF-MEA-006 (Sønderby et al. 2013): pilot absorber measurements
- REF-MEA-002 (Plaza 2012): pilot plant studies (Ch. 4)
- REF-MEA-003 (Cownden and Lucquiaud 2026): 23 test cases from 3 facilities (abstract)
- REF-MEA-007 (Kwak et al. 2012): 2 ton-CO2/day pilot data
- REF-MEA-008 (Tan et al. 2015): CO2 removal along the column at 0.1, 1 and 5 MPa for 20 wt% MEA (Figs. 3-9, PDF pp. 3-5)
- REF-MEA-009 (Wong et al. 2016): CO2 loading in 10 to 30 mass% MEA at 1 to 50 bar and 303 to 323 K (Fig. 2, Table 4 and Fig. 4, PDF pp. 4-5)

### Supporting references

- REF-MEA-001 (Dugas 2009): wetted-wall mass transfer data (Ch. 4)
- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): plant-scale MEA capture process description
- REF-SUP-002 (IEAGHG 2022, 2022-07): MEA capture on the SMR reformer flue gas at 90% capture (PDF pp. 58-59)

Note: REF-MEA-006 and REF-MEA-003 describe near-atmospheric flue gas absorption. REF-MEA-008 gives packed-column absorption data up to 5 MPa, but for 20 wt% MEA and CO2 in natural gas. REF-MEA-009 gives equilibrium data at 1 to 50 bar for 10 to 30 mass% MEA at 303 to 323 K. No column test at the provisional 25 bar absolute condition with 30 wt% MEA and a syngas feed has been identified.

## 10. Rich amine flash drum

### Reference design

- Not yet identified

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- Not yet identified

Note: No document in the library describes a rich amine flash drum.

## 11. Rich/lean amine exchanger

### Reference design

- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): rich/lean amine heat exchanger in the PCC process description (PDF p. 185)
- REF-MEA-002 (Plaza 2012): plate-and-frame cross heat exchanger in the pilot plant description (PDF p. 131)

### Governing equations

- Not yet identified. No exchanger equation set has been verified.

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- REF-MEA-003 (Cownden and Lucquiaud 2026): the lean-rich cross heat exchanger was modelled for each test case (PDF p. 5); equations not verified
- REF-MEA-004 (Morgan et al. 2018): discusses a rigorous lean-rich heat exchanger model (PDF p. 8); context not verified
- REF-MEA-007 (Kwak et al. 2012): pilot equipment includes a lean/rich amine exchanger (PDF p. 2)

## 12. MEA stripper

### Reference design

- REF-MEA-007 (Kwak et al. 2012): pilot stripper (equipment description, PDF p. 2)
- REF-MEA-005 (Morgan et al. 2018, supporting information): Table S2, NCCC stripper variables
- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): plant-scale PCC stripper (Vol. II design basis)

### Governing equations

- REF-MEA-004 (Morgan et al. 2018): rigorous modelling framework, steady-state validation
- REF-MEA-003 (Cownden and Lucquiaud 2026): regenerator within the process model

### Numerical parameter values

- REF-MEA-007 (Kwak et al. 2012): regeneration energy 3.92 GJ/ton CO2 at 90% removal, flue gas 40 C, stripper pressure 0.5 kgf/cm2, L/G 3.7 kg/Sm3 (abstract)

### Experimental validation

- REF-MEA-007 (Kwak et al. 2012): effect of stripper pressure and temperature on regeneration energy; lean and rich loadings and temperature profiles
- REF-MEA-005 (Morgan et al. 2018, supporting information): Tables S2-S4, stripper data and model comparison
- REF-MEA-004 (Morgan et al. 2018): NCCC steady-state validation

### Supporting references

- REF-MEA-002 (Plaza 2012): absorption and stripping process background (Ch. 2, printed p. 6)

## 13. Reboiler

### Reference design

- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): regenerator reboiler and its LP steam supply (PDF pp. 17 and 66)
- REF-MEA-007 (Kwak et al. 2012): pilot reboiler; regeneration energy defined (PDF p. 2)

### Governing equations

- Not yet identified

### Numerical parameter values

- REF-MEA-007 (Kwak et al. 2012): regeneration energy values (abstract)

### Experimental validation

- REF-MEA-007 (Kwak et al. 2012): regeneration energy versus flue gas temperature, stripper pressure and absorbent injection height
- REF-MEA-004 (Morgan et al. 2018): reboiler duty among the NCCC variables (PDF pp. 3 and 12)
- REF-MEA-003 (Cownden and Lucquiaud 2026): specific reboiler duty comparison (PDF p. 5)

### Supporting references

- Not yet identified

## 14. Condenser and reflux drum

### Reference design

- Not yet identified. REF-SUP-001 and REF-MEA-007 list a condenser among their equipment, but no design details have been verified.

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): condenser in the PCC equipment list (PDF p. 17)
- REF-MEA-007 (Kwak et al. 2012): condenser in the pilot equipment list (PDF p. 2)
- REF-MEA-003 (Cownden and Lucquiaud 2026): condenser reflux inlet temperature used as test case data (PDF p. 5)

## 15. Lean amine pump and cooler

### Reference design

- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): lean amine pump (PDF p. 197); lean amine cooler E-102 in the equipment list (PDF p. 215)

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- Not yet identified

## 16. Solvent recycle

### Reference design

- Not yet identified

### Governing equations

- Not yet identified

### Numerical parameter values

- Not yet identified

### Experimental validation

- Not yet identified

### Supporting references

- REF-MEA-006 (Sønderby et al. 2013): lean solvent recycled to the absorber (PDF p. 1)
- REF-MEA-007 (Kwak et al. 2012): solvent recycled back to the absorber (PDF p. 2)
- REF-SUP-001 (Nexant and Bechtel 2020, DICE-CRCC): PCC process description

Note: The recycle loop is described qualitatively only; no source with solvent inventory or recycle loop data has been identified.
