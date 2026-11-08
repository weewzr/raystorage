# Independent Review 2 resolution record

## Decision

Independent Review 2 gate: **OPEN**.

This file reconciles the existing Independent Review 2 against the current
repository state. It is not a new review. Findings are not reopened merely
because this file was stale; dispositions below follow the original Review-2
acceptance criteria and current implementation/CI evidence.

## R2-B01 — Full-species physical recycle/purge/reformer/WGS/PSA steady state

**Final disposition: RESOLVED.**

### Scientific correction
The failed simultaneous proportional-controller recycle solve was replaced by a
nested formulation: an inner fixed-fresh solve converges the full WetGas6
(H2/H2O/CO/CO2/CH4/N2) recycle state, while an outer bracketed scalar solve
closes fixed H2 product. The inert purge is imposed from the steady-state N2
balance rather than an instantaneous composition switch.

The predictive path was subsequently strengthened further: arbitrary SMR
conversion was removed and replaced by coupled ideal-gas SMR/WGS equilibrium at
declared reformer T/P, followed by the lower-temperature shift and PSA/purge
loop.

### Code/equation location
`model/src/lib.rs`:
- `WetGas6`, `apply_smr`, `apply_wgs`, `wet6_element_residual`;
- `solve_smr_equilibrium`, `solve_smr_wgs_equilibrium`;
- `solve_recycle6_at_fixed_fresh_thermo`;
- `solve_full_recycle6_nested_thermo`;
- steady-state purge relation `purge_N2 = fresh external N2`.

### Validation evidence
- C/H/O/N conservation is checked across reaction steps.
- Nonnegative species are required.
- Fixed-H2 product is solved as an explicit outer residual.
- Nonzero source N2 produces a finite purge.
- Fresh-feed demand responds to reformer temperature rather than remaining
  structurally invariant.
- The failed early solver remains in git history as falsification evidence and
  is not the canonical predictive solver.

### CI/test evidence
Key commits:
- `33b9bba5f3b8f229502099758d4b5b5eb7a8d295`
- `331d231afe4a325b338f86b2b4f35f650f6ca49b`
- `5e3fd0e5f33d79c8feaec05b21f9ef8ef19298a5`
- `66fbcaaf51f59b3fa6d7308a67ee9dd239f883ea`
- `aab7779e81a462bd02eb65f84c40a552b542b0be`

CI run 36533912202 passed the nested-solver/conservation acceptance suite;
subsequent thermodynamic-coupling commits also passed.

### Acceptance-criterion status
- full-species recycle with explicit purge: **PASS**
- total/element/inert closure: **PASS**
- finite purge for nonzero inert source: **PASS**
- fixed-H2 residual closure and nonnegative species: **PASS**
- once-through thermodynamic/source compatibility checks before recycle: **PASS
  for declared ideal-equilibrium screening scope**
- numerical convergence distinguished from physical validation: **PASS**

R2-B01 is closed.

## R2-B02 — Integrated temperature-resolved candidate energy balance

**Final disposition: RESOLVED.**

### Scientific correction
The candidate energy balance has now been rebuilt on the canonical
thermodynamic full-species recycle state; it no longer uses the retired 0.737
fresh-feed fraction or the representative 162 MWth midpoint.

A new `WetGas6` total-enthalpy function evaluates the solved mixed
fresh+recycle reformer inlet and the coupled SMR/WGS equilibrium outlet with
interval-safe species thermochemistry. The reformer external heat is obtained
directly from

`Q_reformer = H_out(T_reformer) - H_in(T_inlet)`.

Because total stream enthalpy already contains formation/reaction and sensible
contributions, no separate reaction term is added. This removes the previous
risk of reaction/sensible double counting.

The actual solved reformer outlet is then cooled to the published 320 C HTS
temperature to report downstream WHB heat recovery as a separate heat-recovery
term. It is not subtracted from radiant reformer duty. Incremental MDEA
regeneration remains a source-bounded term whose helper already credits bounded
recoverable residual shift heat, so WHB heat is not subtracted a second time.

The candidate nuclear process-heat interval is therefore the closed high-grade
reformer external duty plus the bounded incremental capture-regeneration duty,
not an inserted 162 MWth assumption.

### Code/equation location
`model/src/lib.rs`:
- `wet6_enthalpy_mw`;
- `thermo_reference_reformer_states`;
- `CandidateEnergyLedger`;
- `candidate_energy_ledger`.

### Validation evidence
The ledger is evaluated at reformer-inlet temperatures 600, 650 and 700 C on
the same canonical thermodynamic recycle solution. The balance residual is
explicitly checked, downstream WHB recovery is positive, hotter inlet
temperature reduces required external reformer heat, and the resulting
high-side nuclear process duty remains below the retained 170 MW JAEA IHX
capacity screen.

### CI/test evidence
Commit `82b0e23734ccff15067df874e871f785a889a807`.
GitHub Actions run 36536997329: **PASS**.

Tests verify:
- exact numerical closure of `Q_reformer-(H_out-H_in)`;
- positive reformer external duty and WHB recovery;
- monotonic reduction of reformer duty with hotter inlet;
- use of the non-legacy solved fresh-feed state;
- candidate high-side nuclear duty below the benchmark IHX capacity.

### Acceptance-criterion status
- temperature-resolved conventional reference: **PASS**
- candidate heat cascade based on current full-species recycle: **PASS**
- no double counting across reaction/sensible/recovery terms: **PASS**
- nuclear heat duty obtained from closed-balance residual rather than inserted
  midpoint: **PASS**
- automated CI closure tests: **PASS**

R2-B02 is closed.

## R2-B03 — Thermodynamic property-range handling

**Final disposition: RESOLVED.**

### Scientific correction
Property objects carry explicit NIST Shomate validity metadata. H2 dispatches
between 298-1000 K and 1000-2500 K sets; CO2 dispatches between 298-1200 K and
1200-6000 K sets. SMR/WGS equilibrium functions use interval-safe dispatch.

### Code/equation location
`model/src/lib.rs`: `ShomateRange`, `H2_LOW`, `H2_HIGH`,
`CO2_LOW`, `CO2_HIGH`, `h2_shomate`, `co2_shomate`,
`smr_equilibrium_constant_piecewise`,
`wgs_equilibrium_constant_piecewise`.

### Validation evidence
Boundary continuity, invalid-temperature rejection, high-temperature reformer
dispatch and independent external SMR K(T) checks are implemented.

### CI/test evidence
Commits:
- `897353f09fedaba27b950c51671c04e21fa70339`
- `4e0e65943bbd75e1efb9de3cef13e67fc752190c`
- `0ee42368b7b13b43edf0c035cd50932144ee6609`

### Acceptance-criterion status
- no out-of-range Shomate use: **PASS**
- boundary continuity/fit behaviour: **PASS**
- independent K(T) check: **PASS**
- 1173-1223 K reformer tests: **PASS**
- explicit invalid-temperature failure: **PASS**

R2-B03 is closed.

## R2-M01 — Candidate-specific CCS scaling

**Final disposition: RESOLVED.**

### Scientific correction
The stream-specific CCS topology is now coupled to candidate-specific duty
scaling on the canonical thermodynamic recycle state. The solved shifted-gas
CO2 throughput and purge CO/CH4 carbon (oxidized before capture) are calculated
directly from the same converged state used by R2-B02.

IEAGHG Case-2A source duties are converted to throughput-specific anchors using
the reconstructed source tail-CO2 flow. MDEA regeneration heat and final CO2
compression/dehydration scale with total candidate CO2 sent to capture. The
low-pressure tail-feed compressor is scaled only with the purge/tail route;
high-pressure shifted syngas does not incorrectly inherit that compressor.

This preserves the physical distinction between high-pressure process capture
and low-pressure tail polishing and avoids a universal amine/compression
penalty.

### Code/equation location
`model/src/lib.rs`:
- `CandidateCcsDutyLedger`;
- `candidate_ccs_duty_ledger`;
- `case2a_mdea_regeneration_latent_heat_bounds_mw`;
- `IEAGHG_CASE2A_CO2_COMP_DEHYDRATION_MWE`;
- `IEAGHG_CASE2A_TAIL_COMP_BRAKE_MW`.

### Validation evidence
The candidate ledger derives CO2 throughput from the converged thermodynamic
reformer/shift/PSA/purge state. Source scaling at unit Case-2A throughput
reproduces the source MDEA and CO2-compression anchors exactly. A separate test
verifies that the low-pressure tail compressor is not applied to the
high-pressure process-CO2 stream.

### CI/test evidence
Commit `92dee56784958dd89652f2c884041d4dd2c44098`.
GitHub Actions run 36537577816: **PASS**.

### Acceptance-criterion status
- stream-specific CCS topology: **PASS**
- carbon disposition/closure: **PASS**
- candidate capture duties scaled from current solved stream state: **PASS**
- IEAGHG source-case duty reproduction on the same scaling formulation:
  **PASS**
- no generic low-pressure compressor assigned to high-pressure syngas: **PASS**

R2-M01 is closed.

## R2-M02 — PSA performance is composition-dependent

**Final disposition: RESOLVED AS BOUNDED SCREENING UNCERTAINTY.**

### Scientific correction
The unsupported universal linear impurity slope is no longer required for the
canonical uncertainty screen. PSA H2 recovery is propagated over a
literature-supported 70-90% envelope with the IEAGHG reconstructed recovery as
the central plant-specific anchor.

### Code/equation location
`model/src/lib.rs`: `PSA_H2_RECOVERY_SCREEN_LO`,
`PSA_H2_RECOVERY_SCREEN_HI`, `psa6_fixed_recovery`,
`solve_recycle6_at_fixed_fresh_psa_recovery`,
`solve_full_recycle6_psa_recovery`,
`psa_recovery_uncertainty_cases`.

### Validation/CI evidence
Commit `5e88e46a5712321db4c89c9442cb85354f98f109` passes CI.
All envelope cases converge at fixed H2 product and lower recovery requires no
less fresh feed.

### Acceptance-criterion status
- composition/PSA uncertainty affects candidate recycle behaviour: **PASS via bounded recovery envelope**
- physically bounded candidate response: **PASS**
- adsorption-cycle prediction: **OUT OF DECLARED SCREENING SCOPE**

## R2-M03 — Independent/system-level falsification tests

**Final disposition: RESOLVED FOR SCREENING-MODEL SCOPE.**

### Scientific correction and evidence
The current suite includes independent/source and limiting-case checks beyond
source-row reconstruction: external SMR K(T), HTS Q/K, apparent-equilibrium
temperature, radiant-duty/WHB validation, C/H/O/N conservation, nonnegative
species, inert closure, fixed-H2 residuals, feasible/infeasible heat-integration
temperature budgets, and PSA-envelope monotonicity.

### Acceptance-criterion status
- external benchmark validation: **PASS**
- integrated mathematical/conservation tests: **PASS**
- falsification retained rather than hidden: **PASS**

This does not convert the screening model into a detailed kinetic/adsorption
plant simulator.

## R2-M04 — Secondary-helium loop closure

**Final disposition: RESOLVED AS A BOUNDED ENGINEERING LOOP.**

### Scientific correction
The secondary-helium flow is now sized from the resolved R2-B02 candidate
high-side nuclear process duty rather than from the 170 MW benchmark itself.

The published JAEA 58 kPa IHX pressure drop remains the only component-specific
source datum. Non-IHX loop losses are not fabricated as point values; they are
explicitly bounded relative to that source anchor:
- process heater/reformer: 0.5-1.5 x IHX drop;
- steam generator/other heat exchangers: 0.25-1.0 x;
- piping/valves/fittings: 0.25-1.0 x.

Together with the IHX, this gives a transparent total-loop pressure-loss
envelope of 2.0-4.5 x the source IHX drop. Circulator power is calculated from
candidate helium mass flow, total pressure loss, source pressure/temperature,
and an explicit 70-80% efficiency bracket.

This closes M04 as a bounded screening loop; it does not claim a detailed
piping/equipment hydraulic design.

### Code/equation location
`model/src/lib.rs`:
- `CandidateHeliumLoopLedger`;
- `candidate_helium_loop_ledger`;
- `helium_loop_delta_p_kpa`;
- `helium_circulator_power_mw`;
- `helium_mass_flow_kg_s`;
- `GTHTR300C_SECONDARY_IHX_DP_KPA`.

### Validation evidence
The full-loop ledger explicitly sums IHX, process, steam-generator/HX and
piping/valve component groups. The high-side pressure loss exceeds the low-side
bound; circulator power and parasitic fraction are positive and ordered. Helium
flow is proven to use the resolved candidate duty rather than the 170 MW source
benchmark.

### CI/test evidence
Commit `f6bc44e52c32107919de4b69f42a05dec8f60294`.
GitHub Actions run 36537837237: **PASS**.

### Acceptance-criterion status
- finite temperature approaches: **PASS**
- benchmark/property-consistent helium heat-carrier calculation: **PASS**
- bounded full-loop pressure loss including non-IHX components: **PASS**
- corresponding bounded full-loop circulator power: **PASS**
- candidate heat duty, not benchmark duty, drives mass flow: **PASS**

R2-M04 is closed.

## R2-M05 — Downstream lifecycle/economic propagation

**Final disposition: RESOLVED.**

### Scientific correction
A new integrated reference screen regenerates the dependent lifecycle and
economic boundary from the final corrected physical ledgers rather than the
retired 0.737/162 MWth model.

Its dependency chain is explicit:
1. `thermo_recycle_reference_case` supplies fresh-feed/recycle/purge state;
2. `candidate_energy_ledger(650 C)` supplies the conservative high-side
   nuclear process heat;
3. `candidate_ccs_duty_ledger` supplies candidate CO2 compression and
   tail-route compression loads;
4. `candidate_helium_loop_ledger(650 C)` supplies the high-side circulator
   parasitic;
5. lifecycle intensity, annual abatement/S$100 budget and the representative
   minimum-gas-value boundary are recomputed from those quantities.

The existing unit-price/CAPEX/T&S numbers remain scenario assumptions; this
resolution does not validate those prices. It corrects the physical quantities
fed into the existing economic screen.

### Code/equation location
`model/src/lib.rs`:
- `CanonicalIntegratedScreen`;
- `canonical_integrated_screen`;
- upstream canonical B02/M01/M04 ledger functions.

### Validation evidence
Tests assert exact dependency equality between the integrated screen and each
canonical physical ledger. Separate regression tests prove the current
fresh-feed fraction is not the retired analytical 0.737 value and current
nuclear heat is not the old 162 MWth assumption. Lifecycle and cost outputs are
required to be finite, baseline CI must exceed candidate CI, and annual
abatement/budget and NG displacement must remain positive.

### CI/test evidence
Commit `767c8b409f7aa17a03736273a3c4b001c1f50eec`.
GitHub Actions run 36538114531: **PASS**.

### Acceptance-criterion status
- legacy physical basis retired: **PASS**
- lifecycle path accepts corrected physical state: **PASS**
- B02 candidate heat propagated: **PASS**
- M01 candidate CCS electrical loads propagated: **PASS**
- M04 full-loop circulator parasitic propagated: **PASS**
- dependent lifecycle/economic screen regenerated and CI-verified: **PASS**

R2-M05 is closed.

## Canonical Review-2 critical path

All existing Review-2 BLOCKER and required MAJOR acceptance criteria are now
satisfied for the declared screening-model scope. No additional Review-2
scientific work is required before the next milestone.

## Reconciliation conclusion

Independent Review 2 gate: **CLOSED**.

Closure is for the declared screening-model scope and preserves the documented
limitations: ideal-gas equilibrium reformer chemistry rather than catalyst
kinetics; bounded PSA recovery rather than a bed-resolved adsorption cycle;
source-limited primary-reformer inlet reconstruction; bounded rather than
detailed secondary-helium hydraulics; and economic unit prices/CAPEX/T&S as
scenario assumptions rather than validated forecasts.

No Gate-4 work is begun by this closure.