# Priority scientific correction: PSA tail-gas carbon closure and full chemical/CCS costs

## Mandate
Execute actual scientific/model/manuscript changes. The user requests a credible assessment of the asserted ~90% capture of remaining CO2, with PSA tail gas recycle/combustion/purge explicitly accounted for, and a much more complete chemical plant plus CCS techno-economic model. These changes take priority over final visual polish. Do not assume the existing FOAK/BOAK/10-OAK economic pass survives.

## Phase A: audit the carbon basis before changing anything
Read original INL TEV-953 and TEV-961 source materials, source register, stream tables, active paper/sections/07_carbon_ccs.tex, 07_lifecycle_formulation.tex, 02_background.tex, model/src/final_design.rs and e2b_maturation.rs, E2B/other results, and the originating source files under literature/data/attachments.

Determine exactly what the 90% capture claim applies to (absorber inlet, process CO2, total generated CO2, direct plant emissions, or lifecycle emissions), and whether the source actually supports it. Trace methane and steam feed, reforming/WGS conversions, PSA feed and offgas composition, tail-gas recycle/combustion/export/purge, residual methane/CO/CO2, and all relevant flue-gas streams. Do not assert PSA tail gas is recycled without source evidence. Identify carbon counted twice or omitted. Reconcile INL 1927 short tons/day captured, 142 short tons/day emitted and 3205 short tons/day baseline to the same process boundary, explicitly noting that these three numbers are not necessarily one closed total-carbon balance.

Implement a streamwise elemental carbon balance with C_in = C_captured + C_emitted + C_products + C_other + accumulation (steady state accumulation zero); include all inlet and outlet carbon-bearing species and gas-phase carbon transformed on combustion. State exact basis and residual in kmol C/h and percent. If source stream compositions or flows are unavailable, use an explicit bounded sensitivity and report 'NOT VERIFIED' for plant-wide 90% capture rather than inventing values. Separate capture fraction at a specific inlet from overall plant capture and lifecycle avoided emissions. Address 85% annual availability, storage credit and any CO2 losses consistently.

## Phase B: full baseline/candidate chemical plant and CCS economics
Build a transparent annualized ledger with separate categories, and evidence-backed scaling, for BOTH equal-H2-service cases:
- reformer and fired furnace or nuclear-heated reformer;
- WGS reactors, steam system, heat recovery, PSA, feed treatment and gas compression;
- natural-gas feed/fuel, steam, electricity, cooling, water, consumables, catalyst and maintenance;
- solvent capture absorbers/strippers, regeneration heat, solvent makeup, capture OPEX and CAPEX;
- CO2 drying/compression/liquefaction as relevant, transport and geological storage;
- nuclear plant CAPEX/OPEX, IHX/secondary helium, shared BOP, engineering, owner/EPC and contingency as evidence permits;
- site, financing/IDC, insurance, licensing, security, waste/decommissioning and Singapore-specific allowances when defensible.

Do not sum equipment CAPEX with plant-level estimates that already include it. Label ISBL/OSBL, total installed cost, working capital, fixed/variable OPEX, annualization and currency/year. Normalize published plant costs by capacity, scope, location and date; cite primary engineering studies/IEAGHG/NETL/DOE where available. No arbitrary zero values for known required items. Missing costs should be ranges or explicitly unpriced and should block claims of fully costed economics. Make clear what can be costed credibly without vendor quotations.

Implement deterministic Rust model/tests and CSV output, including sensitivity for PSA tail-gas fate, process-capture efficiency, CO2 transport/storage tariff, chemical plant CAPEX/OPEX and nuclear maturity. Recalculate annual cost, emissions and AC for FOAK/BOAK/10-OAK. If evidence cannot support a central number, present a bounded interval/conditional scenarios instead of a false point estimate.

## Phase C: manuscript rewrite and QA
Update the actual active LaTeX sections for carbon, economics, assumptions, sensitivity, discussion and conclusion; update README and status. Use top-down high-school-readable exposition: overall mass/carbon/cost identities -> definitions -> submodels -> numeric substitution -> interpretation -> limitations. Provide an exact crosswalk from old screening results to new revised results; preserve historical records in appendices, but remove superseded numbers from the controlling narrative. Clearly distinguish '90% of capture-unit inlet' from '90% of remaining emissions' and '90% lifecycle abatement'. Make all results consistent with any changed assumptions.

Required durable outputs:
- results/PSA_TAIL_GAS_CARBON_CLOSURE_AUDIT.md
- results/FULL_CHEMICAL_AND_CCS_COST_LEDGER.md
- results/REVISED_CARBON_ECONOMICS_COMPARISON.md
- deterministic machine-readable outputs and tests.

Build PDF, check citations/units/references, visually inspect pages, run both CI workflows, commit all changes. If unavailable source data block quantitative closure, report them honestly with targeted evidence requests and conditional calculations. STOP; do not claim complete validation without source evidence.
