# nush2_wasm

Browser ABI of the NUS-H2 engine. A plain `cdylib` with zero imports, so the page runs `WebAssembly.instantiate(bytes, {})` in a Web Worker.

Build: `cargo build --profile wasm-release -p nush2_wasm --target wasm32-unknown-unknown`, then `python scripts/check_wasm.py <path to nush2_wasm.wasm>`. `GIT_SHA` at build time sets the commit id the module reports; without it the id is `local`.

## Memory

Exports `memory`, `alloc(n)`, `dealloc(p, n)`, `call(cmd, p, n)` and `abi_version()`. Write the UTF-8 JSON request into `alloc(n)`, call, free the request. The reply pointer holds a 4-byte little-endian length, then that many bytes of UTF-8 JSON; free it with `dealloc(r, len + 4)`. Every block is a boxed slice whose capacity equals its length. Read `memory.buffer` again after each call, since memory may grow.

## Commands

| cmd | Name | Body | Reply |
|---|---|---|---|
| 1 | `registry` | `{}` | `{schema, engine, status, params, units, kpis, fixed, dropped, levels, limits}` |
| 2 | `solve` | `{schema: 1, inputs: {id: value}, kpi_only: bool}` | result below |
| 3 | `sweep` | `{inputs, param, values}` | `{schema, engine, status, param, unit, rows: [{x, status, kpi, violations, feasible, cost_met, abatement_met}]}`, 1 to 25 values |

The registry call also fills the caches that read no live input, the H2 return closure and the M6 capture island at its design point, so the first solve after it does not pay for them. A trapped instance loses them, and the next solve fills them again.

Measured on the development machine on 2026-11-03 with `scripts/wasm_smoke.py` in headless Chrome, against the plan budget of 500 ms for a full solve: the first full solve took 30 to 42 ms over 9 page loads. The wait from the registry request to the first result, which also covers fetching and compiling the module and the cache fill, took 428 to 641 ms. The same build without the warm-up gave a first solve of 376 to 854 ms and a wait of 442 to 940 ms over 4 loads, on a machine whose load was not controlled. The warm-up moves the cache fill to page load; it does not shorten the wait to the first result, which stays above 500 ms in most loads.

A parameter record is `{id, label, group, unit, kind, default, hard_min, hard_max, model_min, model_max, step, options, class, source, note}` plus `engine_id`, `engine_scale` and `rigour`. Number inputs are JSON numbers in the record's unit; enum inputs are strings from `options[].value`. An input left out takes its default. Hard ranges and the cross rules are enforced here: a value outside them gives `status: "rejected"` with `violations`. A value outside the model range only adds a `flags` entry.

A solve reply holds `schema, engine, status (ok | not_converged | rejected), inputs, violations, flags, messages, kpi, engine_fields_missing, targets`, and unless `kpi_only` is set, `streams, units, profiles, balances, cost`. `cost` holds the selected point, the ledger lines, the L0 to L5 ladder, `grid` rows `{charged, charged_mwth, n, stage, life_y, level, s_per_t}` and `headline` rows with the L3 and L5 ledger costs, the DCF levelised abatement cost and both LCOH values.

`balances.energy` keeps the need and the rating apart. `htgr_heat_need` is the heat the process asks of the reactor (export row HTGR_HEAT). `htgr_heat_output` is the operating point, the need or the rating when the rating is lower, and `unmet_reactor_heat` is the need less the rating. Below the need the heat duties come first (`unmet_thermal`, UNMET_THERMAL), and `electricity_generation` plus `unmet_electrical` (UNMET_ELECTRICAL, J.29) meet the electricity the power branch must make; at or above the need nothing is imported. The reboiler items are the draws of the export REB_FROM_* rows. `balances.power` keeps `e_nuc` at the need and `import_at_rated` as the J.29 memo.

Where the engine has no value the field is `null` and `engine_fields_missing` names it with the reason. The page prints `n/a` for `null` and never fills the gap. Any other non-finite result sets `status` to `not_converged` and names the field in `messages`.

## Mapping onto the engine

Every solve calls `nush2_plant::run` on the Headline preset and the fast tier, then reads `Model::case_summary`, `Model::balance`, `Model::emissions`, `Model::system_adjust`, `Model::lifecycle_terms` and `cost_point` for values the run result does not carry. Columns and the gated terms stay on their anchored values in this preset. Planned controls with no response in this solve are listed in the registry under `dropped` with the reason; the unit test `every_control_moves_a_kpi` keeps every listed control live.
