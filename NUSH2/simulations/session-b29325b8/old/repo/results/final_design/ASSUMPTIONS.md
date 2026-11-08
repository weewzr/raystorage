# FINAL DESIGN ASSUMPTIONS

## Source-backed final process
- INL Case-6 HTGR-integrated SMR with carbon capture remains the final chemical-process basis.
- 871 C reformer outlet, S/C=3.0, 78.1% methane conversion and 88% PSA recovery.
- 925 C INL reactor outlet supplies 900 C process heat.
- 130 MMSCFD H2; 176.8 MWth process heat; 17.3 MWe process electricity.
- INL process-side helium state: approximately 900 C supply, 466 C return and 78.49 kg/s.

## Coherent JAEA/Nishihara architecture
- Selected source architecture: 600 MWth reactor, 370 MWth source heat/IHX branch, complementary 230 MWth source power branch and 88 MWe source electricity.
- Project process duty 176.8 MWth is below the 370 MWth source heat-branch capacity.
- 600 - 176.8 = 423.2 MWth is reported only as remaining reactor thermal capacity.
- No exact project gross/net/export MWe is claimed. The source does not establish an off-design turbine/internal-load mapping for the changed heat split.
- The 70.9 bn JPY / 0.57 JPY/MJ / 5.5 JPY/kWh case is an adverse doubled-loop COST sensitivity only, not a capacity claim.

## Lifecycle
- Preserved independently reproduced lifecycle avoided emissions: approximately 917,139 tCO2e/y.
- Candidate lifecycle intensity: approximately 1.95 kgCO2e/kgH2; the resolved factor-of-1000 defect must not recur.
- Upstream NG, nuclear direct-heat allocation and CCS transport remain declared screening proxies.

## Economics
- Principal mature source economics: 59.7 bn JPY plant, 0.52 JPY/MJ heat, 4.9 JPY/kWh electricity, 85% availability.
- Controlling project test uses electricity export value = S$0/MWh.
- Full selected source-product economic burden is recovered by charging the source 370 MWth heat product plus source 88 MWe generation at their published source unit costs.
- This is an economic allocation and does not claim the project produces or exports 88 MWe.
- CCS, integration/site allowance, T&S and candidate/baseline natural gas remain included.
- No S$100/150/200-MWh project cases are canonical unless a future independently verified off-design power model supports them.

## Claim strength
The result is independently verified as a CONDITIONAL MODEL RESULT; it is not demonstrated Singapore commercial feasibility.


## Stable submission-facing assumption IDs

| ID | Value | Classification | Basis / justification | Limitation |
|---|---:|---|---|---|
| A-01 | 6.3 MWe | PROJECT SCREENING ASSUMPTION / historical model anchor | retained common/baseline electricity anchor from the historical Case-2A separation model; used only so the lifecycle screen charges the 11.0 MWe increment rather than all 17.3 MWe | not established as an INL Case-6/common-output measured value |
| A-02 | 85% | SOURCE-ANCHORED DESIGN ASSUMPTION | Nishihara GTHTR300C economic/design study | design-study basis, not demonstrated Singapore commercial availability |
| A-03 | 1044 Btu/scf | PROJECT SCREENING ASSUMPTION | retained canonical NG HHV used for volumetric-to-energy conversion | actual gas HHV varies with composition |
| A-04 | 11.5 kgCO2e/GJ | AUTHORITATIVE GLOBAL PROXY | IEA global-average gas-supply lifecycle anchor | not Singapore-route-specific |
| A-05 | 5.5 kgCO2e/MWh-e | AUTHORITATIVE GLOBAL PROXY | UNECE nuclear-electricity lifecycle anchor | not direct process-heat LCA |
| A-06 | 0.504 MWh-e-equiv/MWh-th | PROJECT ALLOCATION PROXY | retained GTHTR300-class efficiency allocation used to translate A-05 to a thermal screening proxy | not measured process-heat emissions |
| A-07 | 0.025 tCO2e/tCO2 | PROJECT SCREENING ASSUMPTION | historical generic CCS transport/storage-chain sensitivity | route/storage site unresolved |
| A-08 | S$15/GJ | PROJECT SCREENING ASSUMPTION | retained natural-gas price screen | not a contracted Singapore gas tariff |
| A-13 | 8%, 25 y | SOURCE-ANCHORED ECONOMIC ASSUMPTION | IEAGHG comparator financing/life basis | screening annualisation |
| A-14 | 10% | PROJECT SCREENING ASSUMPTION | integration/site capital allowance on represented plant + CCS capital | not empirical Singapore project cost |
| A-15 | 3%, 40 y | SOURCE-ANCHORED / SCREENING ASSUMPTION | GTHTR design-study economic convention applied to the project integration allowance | not a financing offer |
| A-16 | S$15/tCO2 | PROJECT SCREENING ASSUMPTION | low T&S tariff screen; official Singapore evidence supports uncertainty, not this exact tariff | no operating contract demonstrated |
| A-17 | S$0/y | CONSERVATIVE PROJECT BOUNDARY | no validated project off-design electricity-export model | zero revenue is not a market-price claim |

Full equation/substitution provenance is in `results/final_design/MASTER_NUMBER_PROVENANCE_REGISTER.md`.
