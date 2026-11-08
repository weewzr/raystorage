# Economic parameter matrix v1

This matrix separates published values from project assumptions. No central Singapore abatement cost is claimed yet.

| Parameter | Published value / range | Basis | Role in model | Status |
|---|---:|---|---|---|
| IEAGHG SMR+CCS avoidance cost | EUR47-70/tCO2 | Q4 2014, standalone SMR | historical CCS benchmark | source |
| IEAGHG additional CCS capital | EUR40-176m | Q4 2014 | capture CAPEX range/context | source |
| GTHTR300 capital | ~JPY200,000/kWe | JAEA design study, 2006/07 | HTGR power-system anchor | source, legacy |
| GTHTR300 COE | ~JPY4.2/kWh | 80% utilisation, JAEA | nuclear-electric anchor | source, legacy |
| JAEA nuclear process heat price | JPY0.7/MJ | HTGR hydrogen economic study | direct-heat price anchor | source, legacy |
| JAEA electricity price in H2 study | JPY5.8/kWh | same study | electric-service anchor | source, legacy |
| IAEA HTGR-200+SMR NPP CAPEX | USD2065m | 4x200 MWth | large-system nuclear CAPEX anchor | source |
| IAEA HTGR-200+SMR H2-plant CAPEX | USD1013m | 440 ktH2/y case | H2 plant anchor | source |
| IAEA MHR-T+SMR NPP CAPEX | USD2748m | 4x600 MWth | alternative HTGR anchor | source |
| IAEA MHR-T+SMR H2-plant CAPEX | USD1496m | 400 ktH2/y | alternative H2 plant anchor | source |
| IAEA HTGR-200+SMR H2 cost | USD1.28/kgH2 | source study assumptions | external benchmark | source |
| IAEA MHR-T+SMR H2 cost | USD0.96/kgH2 | source study assumptions | external benchmark | source |
| Ahn & Lee helium-SMR LCOH | USD2.37/kgH2 | 2026 study | modern validation benchmark | source |
| Ahn & Lee eSMR LCOH | USD2.99/kgH2 | 2026 study | modern validation benchmark | source |
| Singapore USEP | ~S$100-200/MWh in 2025 | EMA | opportunity/grid electricity context | source, not nuclear cost |
| Singapore gas-system charge PNG | S$0.23/MMBtu FY26 | EMA indicative schedule | network-charge context only | source |
| Singapore gas-system charge LNG | S$0.04/MMBtu FY26 | EMA indicative schedule | network-charge context only | source |
| Singapore CCS T&S | not established | MTI Apr 2025 | sensitivity | unresolved |
| Discount rate | TBD | project finance assumption | CRF | unresolved |
| Plant life | TBD | technology/project assumption | CRF | unresolved |
| Capacity factor | TBD | project assumption | annual output | unresolved |
| NG commodity price | TBD | Singapore contract data unavailable publicly | feed/fuel OPEX | unresolved |

## Interpretation rules

1. Legacy JAEA values are not inflated or FX-converted silently.
2. IAEA cases are large multi-module systems and cannot be scaled linearly to Singapore without an explicit scaling law.
3. Singapore USEP is an electricity-market/opportunity-cost benchmark, not the cost of nuclear electricity.
4. EMA gas-system charges are network/system charges, not the commodity natural-gas price.
5. MTI has not published a complete Singapore cross-border CCS cost; T&S remains a sensitivity.
6. All eventual S$ values must use a declared common price year and FX source.

## First useful break-even framing

The economic question should be solved as:

Delta C_max = 100 S$/tCO2e * annual avoided CO2e.

At 0.25 Mt/y:
Delta C_max = S$25m/y.

At 0.50 Mt/y:
Delta C_max = S$50m/y.

At 0.60 Mt/y:
Delta C_max = S$60m/y.

The model should ask whether annualised incremental nuclear + integration + CCS costs, net of avoided conventional costs and coproduct credits, fit inside this budget.
