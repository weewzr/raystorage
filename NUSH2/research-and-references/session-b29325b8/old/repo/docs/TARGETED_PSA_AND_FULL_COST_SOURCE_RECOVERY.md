# Targeted Source Recovery: PSA Carbon Closure and Complete Chemical/CCS Economics

## Purpose
Continue the unresolved scientific gates documented in:
- `results/PSA_TAIL_GAS_CARBON_CLOSURE_AUDIT.md`
- `results/FULL_CHEMICAL_AND_CCS_COST_LEDGER.md`
- `results/REVISED_CARBON_ECONOMICS_COMPARISON.md`
- `docs/MASTER_PSA_CARBON_AND_FULL_ECONOMICS_REBUILD.md`

This is an execution instruction. Actually research, implement and edit; do not stop after creating another plan.

## Step 1: Original process-source recovery
Retrieve and inspect the original INL TEV-953 and TEV-961 documents and supporting PFD/stream tables, prioritizing:
- natural gas feed composition and standard-volume conditions;
- reformer and water-gas-shift outlet streams;
- PSA inlet, hydrogen product and tail-gas species compositions/flows;
- PSA offgas recycle, combustion, purge or export destination;
- capture absorber inlet/outlet, flue gas, vent and purge boundaries;
- all carbon-bearing streams and recycle loops.

Construct a source-referenced steady-state elemental carbon balance in kmol C/h:
C_in = C_captured + C_emitted + C_products + C_other_out.
Calculate and disclose its absolute and relative residual. Do not set unknown streams to zero or infer tail-gas fate from 88% hydrogen recovery. Distinguish capture-unit efficiency, fraction of reported CO2 streams captured, plant-wide carbon capture and lifecycle emissions avoided. If source data remain insufficient, identify exact missing values and use clearly conditional bounded sensitivities, not fabricated closure.

## Step 2: Evidence-backed complete costs
Research primary IEAGHG, NETL, DOE/INL and comparable engineering studies for matched-capacity and scope conventional and nuclear-heated SMR hydrogen systems. Obtain cost evidence for:
- fired/nuclear-heated reformer, WGS, PSA, feed pretreatment and compression;
- steam, heat recovery, power, cooling, water, catalysts, consumables and maintenance;
- amine capture CAPEX and OPEX, solvent regeneration, makeup and utilities;
- CO2 drying/compression/conditioning, transport and permanent storage;
- nuclear reactor and IHX/secondary loop, shared balance of plant;
- EPC/owner costs, contingency, financing, site, licensing, security, waste and decommissioning where defensibly costed.

Normalize equipment/plant boundaries (ISBL/OSBL), capacity, operating basis, currency year, installed vs overnight cost, fixed/variable OPEX and annualization. Do not double count integrated plant estimates and equipment line items. Maintain equal hydrogen service and consistent baseline/candidate boundaries. Label missing mandatory costs as unpriced, not zero.

## Step 3: Recompute, test, and rewrite
Implement evidence-supported central cases and bounded ranges in deterministic Rust, including PSA tail-gas fate, absorber capture, chemical CAPEX/OPEX, CCS operating costs, transport/storage tariff and FOAK/BOAK/10-OAK maturity. If data do not support a central full-project cost, report defensible intervals or explicitly say not calculable.

Update active LaTeX sections, README, relevant plots/tables and conclusions to reflect verified results. Use clear reader-facing terms: 'conventional baseline', 'nuclear-assisted system' and 'overall annual cost' where correctly defined; avoid ambiguous 'candidate'. Use top-down exposition: whole equation -> components -> source inputs -> numerical substitution -> physical/economic interpretation -> limitations. Preserve prior screening outputs as historical provenance and do not force a numerical pass.

## Durable outputs
Update the three audit/result documents listed above and add source-to-stream and source-to-cost traceability files or machine-readable outputs as appropriate. Explicitly report which original INL tables were found, carbon residual, capture fraction definition, cost completeness, revised cost and avoided-emissions ranges, missing evidence and whether any screening conclusions changed.

Run Rust tests and both GitHub CI workflows, build the PDF, inspect its pages, commit all changes and STOP. Do not begin a generic manuscript polish or another independent review.
