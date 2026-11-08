# Reader-facing terminology refinement: replace "candidate"

Apply during the PSA carbon-closure and full economics rebuild.

Replace vague uses of "candidate" in the active manuscript, figure captions, tables, README and new economic/carbon explanations with context-specific plain language:
- "proposed nuclear-assisted system" or "nuclear-assisted system" for the evaluated process;
- "conventional baseline" for the comparator;
- "overall annual cost of the nuclear-assisted system" for the complete candidate-side cost ledger;
- "overall emissions of the nuclear-assisted system" where appropriate.

Use C_overall only when explicitly defined as the total annual cost of the nuclear-assisted system, never the sum of baseline and proposed-system costs. Prefer C_nuclear_assisted if there is any ambiguity. Similarly label emissions E_nuclear_assisted. Define symbols once and apply consistently across equations, nomenclature, plots, captions and source-code-to-manuscript mappings.

Do not blindly replace "candidate" in citations, source quotations, filenames, historical audit records, programmatic Rust identifiers, or genuine "candidate sites/technologies" contexts. Avoid unnecessary breaking API/schema changes. Preserve scientific meaning and LaTeX references, build/test after edits. Integrate this terminology pass with docs/PSA_CARBON_CLOSURE_AND_FULL_COST_REBUILD.md and STOP.
