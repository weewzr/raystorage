# Economics equation-first audit

## Controlling source and exact ledger mapping
Source: `model/src/e2b_maturation.rs`, `scenario(2,"10-OAK")`; the same `scenario` function calculates FOAK and BOAK with different nuclear annual burden. The candidate and baseline provide the same two-train hydrogen service, 195,892.072 t/y at 85% availability.

**Overall identity** (annual costs S$/y; avoided emissions tCO2e/y):
`AC=(C_candidate-C_baseline)/A_lifecycle`.

**Exact code terms:**
- `C_baseline = TRAIN_BASELINE_SGD_Y * 2`. This is **represented baseline NG expenditure**, not full SMR CAPEX/OPEX.
- `C_candidate = 2*(TRAIN_CANDIDATE_NG_SGD_Y+TRAIN_CCS_SGD_Y+TRAIN_TS_SGD_Y)+nuclear_annual_sgd(stage)+2*ihx_loop_annual_sgd_unscaled()`.
- `A_lifecycle = 2*TRAIN_AVOIDED_T_Y`.
- `nuclear_annual_sgd(stage) = [usd2019_to_sgd2025(heat_only_occ_usd_kwt(stage)*600000)]*CRF(0.08,25) + usd2019_to_sgd2025(10*600*7446)`.
- `heat_only_occ_usd_kwt(BOAK)=6000*(1-.09)*.4=2184 USD2019/kWt`; FOAK multiplies by 1.6; 10-OAK multiplies by `(1-.10)^log2(10)` = approximately 0.704.
- `usd2019_to_sgd2025` applies US GDP-deflator 2025/2019-quarter-average and the repository's dated SGD/USD conversion.

## Mature worked substitution (rounded S$ million/y)
| Cost component | Two-train amount | Evidence |
|---|---:|---|
| Baseline NG | 2 x 269.115246 = 538.230492 | per-train project NG model |
| Candidate NG | 2 x 174.284159 = 348.568319 | per-train project NG model |
| CCS annualized capture CAPEX | 2 x 10.682760 = 21.365520 | IEAGHG anchor with declared linear scaling |
| CO2 transport/storage | 2 x 8.135430 = 16.270860 | S$15/t captured screening tariff |
| Mature nuclear annual CAPEX/O&M | ~207.763 | INL medium 10-OAK learning scenario |
| Two IHX/secondary interfaces | ~22.846 | E1 represented interface allowance |
| Candidate total | ~616.814 | unrounded model sum |
| Incremental annual cost | ~78.583 | candidate minus baseline |
| Avoided emissions | 1.8342778 MtCO2e/y | 2 x 917138.9 t/y |
| Abatement cost | ~S$42.84/tCO2e | (S$million/y)/(Mt/y) |

Displayed three-decimal costs can differ by 0.001 million S$/y when subtracted; use unrounded Rust outputs for final division. No electricity or hypothetical co-product revenue is credited.

## FOAK / BOAK / 10-OAK comparison
FOAK = ~S$137.74/tCO2e (FAIL); BOAK = ~S$74.14/tCO2e (projected/modelled PASS); mature 10-OAK = ~S$42.84/tCO2e (projected/modelled PASS). Only the nuclear annual capital basis varies by maturity in this scenario function; represented NG, CCS, transport/storage, integration, hydrogen and avoided emissions are held fixed. This isolates learning but is not a validated real-world forecast.

## Audit findings and corrections
1. The original active economics section began with a superseded one-train cost bridge. The new leading subsection now shows the **actual** two-train baseline and candidate equations and a full numerical ledger before the historical model.
2. The ledger does **not** contain standalone conventional SMR capital/O&M or full chemical-plant capital charges. These costs must not be invented in the LaTeX decomposition or called fully priced. The newly added explanation explicitly warns of this incompleteness.
3. The capture term is annualized capital, not automatically all CCS operating expense; upstream and project-specific infrastructure costs remain uncertain.
4. The model uses one reactor annual cost for both trains and two represented interface allowances. It does not allocate reactor cost independently to each train or claim a physical 353.6 MWth single IHX.
5. The model's future learning method is an externally motivated scenario; its numerical PASS does not establish current economic feasibility.

## Dimensional and physical checks
Annual cost: S$/y. CRF: y^-1 as annualization factor. Heat-only OCC: USD/kWt multiplied by 600,000 kWt gives USD capital. OPEX: USD/MWth-h times MWth times h/y gives USD/y. Cost difference S$/y divided by tCO2e/y gives S$/tCO2e. Baseline/candidate hydrogen service and annual availability are matched by construction.

## Unresolved economic terms
Selected Singapore site and environmental costs, reactor licensing, nuclear security, waste/spent fuel, decommissioning/liability, financing and schedule risk, commercial IHX qualification, full chemical-plant CAPEX/OPEX, CCS operating costs where excluded, actual transport/storage contracts, offtake and availability. These can materially consume the mature scenario's cost margin. The manuscript now explicitly describes the represented ledger rather than a bankable total installed cost.

## Verification
Source-to-equation mapping completed; CI and PDF checks must be reported at final HEAD rather than inferred from an earlier green commit.
