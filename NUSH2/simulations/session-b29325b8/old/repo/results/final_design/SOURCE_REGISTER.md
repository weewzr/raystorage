# FINAL DESIGN SOURCE REGISTER

| Parameter | Final value | Source/classification | Final use |
|---|---:|---|---|
| Reformer outlet | 871 C | INL TEV-953/961, SOURCE-BACKED | final process |
| S/C | 3.0 | INL TEV-953, SOURCE-BACKED | final process |
| Methane conversion | 78.1% | INL TEV-953, SOURCE-BACKED MODEL RESULT | final process |
| PSA recovery | 88% | INL TEV-953, SOURCE-BACKED | final process |
| H2 product | 130 MMSCFD | INL TEV-953/961, SOURCE-BACKED | common service |
| INL ROT / process heat temperature | 925 / 900 C | INL TEV-961 Case 6, SOURCE-BACKED | final process |
| Process heat | 176.8 MWth | INL TEV-961 Case 6, SOURCE-BACKED | final duty |
| Process electricity | 17.3 MWe | INL TEV-961 Case 6, SOURCE-BACKED | chemical-process demand |
| Process He supply / return / flow | 900 C / ~466 C / 78.49 kg/s | INL TEV-961 Case 6, SOURCE-BACKED MODEL RESULT | heat-transfer anchor |
| Nishihara reactor | 600 MWth | Nishihara et al. 2007, SOURCE-BACKED | selected mature architecture |
| Nishihara heat/IHX branch | 370 MWth | Nishihara et al. 2007, SOURCE-BACKED | capacity/economic source |
| Nishihara complementary branch | 230 MWth | 600-370, SOURCE-DERIVED IDENTITY | source identity only |
| Nishihara electricity | 88 MWe | Nishihara et al. 2007, SOURCE-BACKED | source configuration/economic burden only |
| Project residual thermal capacity | 423.2 MWth | 600-176.8, PROJECT-DERIVED | capacity only; not electricity |
| Project electricity export | not claimed | no validated off-design/internal-load model | zero economic credit |
| Reference plant / heat / electricity cost | 59.7 bn JPY / 0.52 JPY/MJ / 4.9 JPY/kWh | Nishihara et al. 2007, SOURCE-BACKED DESIGN STUDY | controlling mature economic allocation |
| Doubled-loop cost case | 70.9 bn JPY / 0.57 / 5.5 | Nishihara et al. 2007, SOURCE-BACKED COST SENSITIVITY | adverse cost sensitivity only |
| Availability | 85% | Nishihara et al. 2007 | annualization |
| Upstream NG | 11.5 gCO2e/MJ | IEA, GLOBAL PROXY | lifecycle screen |
| Nuclear lifecycle | 5.5 gCO2e/kWh electricity anchor | UNECE, GLOBAL PROXY | derived direct-heat allocation |
| T&S | S$15/t captured | PROJECT SCREENING ASSUMPTION | economic screen |
| Electricity export value | S$0/MWh | PROJECT CONSERVATIVE TEST | controlling cost test |


## Numerical-audit clarifications

The submission-facing master register is `results/final_design/MASTER_NUMBER_PROVENANCE_REGISTER.md`. The following values are intentionally **not** promoted as primary-source measurements:

- 6.3 MWe common/baseline electricity anchor — historical project screening assumption (A-01), not established as an INL Case-6/common-output value;
- 1044 Btu/scf natural-gas HHV — project screening conversion basis (A-03);
- 0.504 thermal allocation — project lifecycle allocation proxy (A-06);
- 0.025 tCO2e/tCO2 T&S lifecycle factor — generic project screening assumption (A-07);
- S$15/GJ natural-gas price — project screening assumption (A-08);
- 10% integration/site allowance — project screening assumption (A-14);
- S$15/tCO2 T&S tariff — project screening assumption (A-16), not a contracted Singapore tariff;
- S$0/y project electricity revenue — conservative project boundary (A-17), not a market-price observation.

This classification prevents project assumptions from being visually confused with the INL/JAEA/IEA/UNECE source inputs.
