# NUS-H2 interface: assessment, library choice and the process tour

Author: Wee Zhern Ray. Licence: MIT, as the rest of the repository.

This document covers the interface work on the browser simulator. It holds the assessment of the existing site, the review of three candidate component libraries, the integration strategy that follows from it, the design of the guided process tour on the Interactive PFD page (built in this stage), a plan for the rest of the interface brief, and the record of the checks that ran.

The tour came first because it was asked for first: a button outside the drawing that walks through the plant, from fission heat in the reactor core through the helium loops, the reformer train and capture, to the hydrogen product at one end and CO2 storage at the other.

## 1. Architecture assessment

| Area | What the repository has |
|---|---|
| Frontend | Vanilla ES modules with no framework, bundler or package.json: `app.js` (shell, routes, worker RPC, status line), `views.js` (Dashboard, Parameters, Results, Export, Validation), `pfd.js` with `panels.js` (Interactive PFD and unit panels), `controls.js`, `charts.js`, `selftest.js`, and from this stage `tour.js`. Every local resource carries a `?v=` commit stamp that `scripts/build_site.py` writes and checks. |
| Backend | None. The site is static and is served by GitHub Pages. The engine runs in the reader's browser. |
| Simulation engine | A Rust workspace under `engine/`: `props` (species, Shomate data, steam, helium, equilibrium constants, transport), `mea` (MEA solvent thermodynamics and kinetics, extended UNIQUAC after Aronu), `units` (reformer equilibrium and tube model, shift, IHX, circulators, flash, coolers, absorber and stripper columns), `plant` (anchored port, Level 3 flowsheet, heat recovery, carbon and emissions ledgers, economics and DCF, sweeps), `numerics`, `vv` (validation harness), `cli` (export of the results CSVs) and `wasm` (the browser ABI). |
| Thermodynamic and kinetic models | Equilibrium reforming and shift with fitted equilibrium constants, a one-dimensional reformer tube heated by helium in counterflow with Xu and Froment rates, MEA vapour-liquid equilibrium and column models. The Headline solve keeps the gated terms on their anchored values; each unit panel states which model it uses and its status. |
| Equipment models | 20 units in every solve reply (`units.<id>` with inputs, outputs, conditions, results and a status of rigorous, reduced order, empirical, fixed assumption or external). The PSA is external and has no model. |
| Process-flow visualisation | One inline SVG in `frontend/index.html` (viewBox 2070 by 1000) after the project's Figure 2: 20 unit groups (`data-unit`, real button role, keyboard selection), 30 stream groups (`data-stream`, focusable, with tooltips), a dashed carbon-accounting boundary and labels bound to the reply by `data-bind`. `units.json` holds the panel prose, stream names and the unit order. |
| Charts | `charts.js` draws every chart as hand-written SVG from the reply or from `data/*.json`; there is no chart library. |
| Styling | Colour, spacing and font tokens on `:root` in `style.css`, with `views.css`, `pfd.css` and `print.css`. Light mode only. Stream colours are literal values in `pfd.css`. Before this stage no file handled reduced motion. |
| UI components | Hash routes as tabs, a status line with a stale tag, KPI tiles, target verdict lines, a scenario radio group, controls generated from the engine registry, unit panels with seven fixed sections, stream tooltips, a unit picker for phones. |
| Dependencies and build | Python standard library scripts (`build_wasm.py`, `build_site.py`, `check_site.py`, `check_clean.py`, `selftest.py`, `wasm_smoke.py`) and cargo. No Node toolchain locally or in CI. Third-party code exists only as Rust crates compiled into the module, with their notices in `THIRD_PARTY_NOTICES.md`. |
| Frontend and engine data | The worker instantiates `nush2.wasm` with an empty import object and answers three JSON commands: `registry`, `solve` and `sweep`. A solve reply carries `kpi`, `units`, `balances`, `flags`, `feasibility`, `targets`, `profiles`, `cost`, `messages` and `engine_fields_missing`; a null field is listed there with its reason and prints as n/a. Precomputed tables reach the page as `data/*.json`, which `build_site.py` writes from the results CSVs without arithmetic. |
| Deployment | `.github/workflows/site.yml` runs the engine tests, builds and checks the module, regenerates and compares the results, builds the site, runs both scopes of `check_site.py` and the headless browser self test, then deploys Pages from `main` only. |

The site is therefore a small, dependency-free static application whose data contract is the engine reply. Any interface change has to keep that contract, the `?v=` stamping, the content and host guards, and the self test that compares the browser solve with the native results.

## 2. Library compatibility

Licences were read from each project's own repository and published manifests on 2026-11-03. Nothing was installed.

| Library | Licence of the code | Run-time dependencies | Fit with this repository | Verdict |
|---|---|---|---|---|
| React Flow (`@xyflow/react` 12.12.1) | MIT | 17 packages, all MIT, ISC or BSD-3-Clause (React, React DOM, scheduler, zustand, d3-zoom and its d3 dependencies) | Poor. The PFD is a finished, hand-routed drawing; moving it into React Flow means rebuilding 20 units as custom nodes and 30 streams as custom edges that replay the same paths, then switching off dragging and connecting. It needs React and a compile step, and its bundles carry external URLs that the `hosts` guard rejects. Its default attribution badge links to an outside site. | Do not adopt. Take its ideas (fit to view, minimap, animated edges, focus panning). |
| shadcn/ui | MIT | Copies TSX source that needs React, Tailwind and Radix UI. Radix, `cn`, Lucide (ISC), sonner, recharts and the rest are permissive; class-variance-authority is Apache-2.0. Tailwind v4 builds with lightningcss, which is MPL-2.0. | Poor as code: there is no package to depend on and no form that works without React and Tailwind. Good as a source of conventions. | Do not adopt. Take its token pairs, card and badge anatomy, focus ring and state-from-attribute styling. Tailwind v4 would need approval because of its MPL-2.0 build tool. |
| Aceternity UI | No open licence found. The public repository named in the brief returns 404, and the only licence page forbids redistributing an item's source files. The component pages carry "All Rights Reserved". | motion, framer-motion, three and others, all MIT; tslib 0BSD | None. Copying any of its files into this public MIT repository would redistribute source under terms its author did not grant. | Copy nothing. Generic effects (a static grid, one soft glow, a gradient moving on an active path) can be written fresh. |

Two further points hold for any vendored JavaScript. MIT, ISC and BSD notices must travel with copies, and several of these notices name individual copyright holders, while the site's notices today name no person. Vendoring would also add a section to `THIRD_PARTY_NOTICES.md`, which is being edited elsewhere at the same time.

## 3. Recommended integration strategy

Port ideas into the existing SVG, CSS and ES modules, with no third-party code. The audit compared three paths:

| Path | Third-party bytes | Build and test cost |
|---|---|---|
| Adopt React with a build step | about 201 kB minified for React and React DOM alone, about 388 kB with a React Flow subset, plus 18.6 kB of CSS | Node and a bundler in CI, local builds broken on a machine without Node, PFD self-test checks rewritten, guard allowlist and notices rules changed, dozens of notices added |
| Vendor prebuilt files | React 18 browser builds plus the xyflow build, or d3-zoom and eight d3 packages for pan and zoom only | a shim for a missing React global, and a decision on notices that name individuals |
| Port ideas | none | new self-test checks only |

The third path keeps the engine binding, the panel code, the self-test audits and every guard unchanged, and leaves about 2.7 MB of room under the 6 MiB site cap (the built site is about 3.5 MB). If free pinch and wheel zoom becomes a requirement and a hand-written version fails its tests, the fallback is d3-zoom alone, after the notices question is answered.

## 4. Proposed interface architecture and visual design

The brief's five regions map onto what the site already has, so the plan extends routes instead of replacing them.

| Region in the brief | Where it lives |
|---|---|
| Top bar: name, status, scenario, run, reset, export | The existing header, status line and scenario control; the engine solves on every input change, so there is no separate run button to add. Export already has its own route. |
| Left navigation by process section | The route tabs today. A section list on the PFD page would reuse the tour's step list, so "go to the shift" lights the same units and streams as tour step 5. |
| Central flowsheet | The inline SVG, with the tour as its guided mode. A camera that tweens the SVG viewBox to a step's units, and zoom and fit buttons outside the drawing, come next (section 6). |
| Right panel: equipment properties | The unit panel, unchanged in content: inputs, outputs, conditions, equations, sources, results, limitations. |
| Bottom panel: solver status and balances | The status line, the feasibility line and the Results and Validation routes. A collapsible summary of `messages`, `flags` and the closure residuals on the PFD page is a small addition. |

How it connects to the engine: every view reads one object, the last converged solve reply (`state.lastOk`). The shell calls each route's render hook on every reply, so the tour, the panels and the drawing labels change together when a control moves. No view computes an engineering quantity; a missing field prints n/a with the engine's own reason.

Visual design proposal for the later stages:

- Colour: keep the light theme as the default and add a dark set under `prefers-color-scheme` and a `data-theme` switch. Every surface token gets a matching text token (page and ink, surface and ink, accent and accent text), and each pair is checked at 4.5:1. The dark set uses a deep navy page, a slate surface, a teal and blue accent, and keeps the success, warning and error pairs.
- Streams: move the ten stream colours of `pfd.css` into tokens, with a dark variant of each that keeps at least 3:1 against the dark page.
- Background: a static technical grid drawn with two thin linear gradients and faded at the edges, plus one soft radial glow behind the drawing. Nothing in the background moves.
- Equipment and KPI cards: the card and badge anatomy above, a token-coloured selection glow (the tour's lit units already use one), and model status as badges styled from the existing `data-status`.
- Motion: only where it reflects state. The tour's moving dashes run only on the active step's streams; a KPI value may ease to a new reply value, with the final digits always the engine's; no ambient animation, and none at all under reduced motion.

## 5. The process tour

### 5.1 What it does

A "Play the process tour" button sits in the toolbar above the drawing, beside the unit picker, with a line giving the step count and the play time. Pressing it opens a control bar above the drawing (Previous, Pause or Play, Next, Exit, a step counter and a segmented progress bar) and a caption card beside the drawing. On wide screens the card heads the side column; below 1200 px it sits directly under the drawing, and the long layout note under the drawing steps aside while the tour runs.

Each step dims the rest of the drawing, lights its units with a soft glow, keeps neighbouring units half lit, and moves dashes along its streams in the direction of flow. Streams of a step start one group after another, so the order of the flow reads on screen (for example primary helium out of the reactor, through the IHX, then back by the circulator). On narrow screens the drawing scrolls sideways to the lit part. The panel of the step's lead unit opens beside the card, and the reader's own unit comes back when the tour closes. Clicking the drawing pauses the tour.

### 5.2 Steps

| Step | Title | Lit units | Animated streams, in order | Numbers shown (reply path unless marked) |
|---|---|---|---|---|
| 1 | Fission heat in the HTGR core | HTGR and its core | none; an illustration inset | reactor outlet, rating, heat output, margin |
| 2 | Primary helium loop | HTGR, IHX, circulator | P1, P2, P3 | IHX duty, primary helium flow, primary circulator electricity, IHX approach |
| 3 | Secondary helium loop to the reformer | IHX, reformer | H1, H2 | helium supply and return, process heat, secondary helium flow, secondary circulator electricity |
| 4 | Natural gas and steam feed the reformer | reformer | S01 with S02 | gas feed, reformer exit, exit reachable on the assumed tube (results row TUBE_EXIT_C_N199) |
| 5 | Water-gas shift | shift | S05 | shift inlet temperatures, shift heat release |
| 6 | Cooling and condensate | syngas cooler, separator | S09, S09B, then S10 with S11 | syngas heat to the reboiler, separator temperature, condensate |
| 7 | CO2 capture in the MEA absorber | absorber | S11 with S18, then S17 | MEA strength, absorber pressure, capture per pass, CO2 captured, solvent rate |
| 8 | Hydrogen product from the external PSA | PSA, boundary | S12, S13 | H2 into the PSA, assumed recovery, purified H2 per hour and per year |
| 9 | PSA reject back to the cryogenic unit | recycle compressor, cryogenic unit, stack | S14, R1, then S21, S03 and S15 together with their continuations along S12 and S01, then S16 | H2 and carbon in the reject, carbon recycled and burned, CO2 to the atmosphere |
| 10 | Stripper and reboiler heat | rich/lean exchanger, stripper, reboiler, lean cooler, pump | S17, L3, L5 with L6, L4, S18 | stripper pressure, reboiler duty, specific duty, the four heat sources in their approved order |
| 11 | CO2 compression, ship and offshore storage | condenser, CO2 compressor, storage | C1, S19, S20 | compressed CO2 pressure (results row of S20), compression electricity, CO2 shipped, tariff and charge |
| 12 | Power block and site electricity | power block, circulators, HTGR | P1, P4 | helium branch, electricity generated, efficiency, steam heat to the reboiler, circulator and site demand, grid import |
| 13 | The plant from end to end | whole plant | every stream at once | purified H2, CO2 stored and released, baseline, net abatement on three baselines, target verdict and the engine's cost verdict |

The play-speed dwell is 7 or 8 s a step, 97 s for all 13. Under reduced motion there is no auto-advance.

### 5.3 Where each number comes from

Every caption number is read when the step renders, by its reply path, so it follows the controls; the shell's render hook re-renders the open step on each new reply. Where the reply has no field, the number comes from a results row named in `scripts/build_site.py` (`TOUR_ROWS`), which copies three cells into `data/tour_rows.json`: the reachable tube exit (`Final_Design_Basis.csv`, TUBE_EXIT_C_N199), the compressed CO2 pressure (`Integrated_Mass_Balance.csv`, run FS_B_REC_F, stream S20) and the electricity from tail gas burned on site (`Integrated_Energy_Balance.csv`, run B_COMB_H). Those print with a "precomputed" mark. The page formats numbers (fixed decimals, and a fraction shown as a percentage) and computes nothing else. Each figure tile says whether its value is live, an input, a fixed registry value or precomputed.

The tour follows the plant as the engine models it, including its stated limits:

- Step 4 states that the engine sets the reformer exit at INL's equilibrium assumption and that, with helium at the supply temperature, the tube model reaches only the precomputed exit on the assumed tube.
- Step 8 says the PSA is external and not simulated, and its recovery is an assumption.
- Steps 1, 2, 3, 12 and 13 show the engine's own feasibility reason when the rating falls below the need, and step 13 states the abatement verdict and the cost verdict the engine gives.
- When the PSA reject is exported or burned on site, steps 9 and 13 stop animating the recycle lines and say the drawn recycle carries nothing; under Selexol, steps 7 and 10 say so and the reboiler step shows its duty.
- Step 13 notes that the Western Island site is the author's proposal with no official approval.

Drawn without engine data, and tagged on screen as illustrations: the fission inset of step 1 (a neutron splitting a uranium nucleus inside a TRISO particle in graphite, the heat passing to helium), the ship inset of step 11, and the reaction lines of steps 4, 5 and 7. The insets carry no numerals. The caption also says where the drawing simplifies: the reformer's helium connections are schematic (the engine's tube model runs the helium in counterflow), the PSA beds are schematic, the secondary circulator and the heat lines into the reboiler are not drawn, and the burner of the nitrogen-rich cut is represented by the stack.

### 5.4 Colours

Dashes run on the stream's own coloured line, so the line colour carries the medium and the legend of the drawing applies. Each step's card lists its media with swatches. The dash colour on each line was chosen for at least 3:1 against that line:

| Medium | Line | Dashes |
|---|---|---|
| Primary helium | #e2463b | white |
| Secondary helium | #f29a2e | #5c3008 |
| Natural gas | #45a86a | #123d22 |
| Steam and condensate | #3d93d6 | #0f3557 |
| Syngas | #d6ad18 | #4a3d05 |
| MEA solvent | #9a4fc2 | white |
| H2-rich gas and H2 | #e0559f | white |
| Captured CO2 | #5d6d7d | white |
| PSA reject and recycle | #35a7b5 | #0b3a40 |
| N2-rich stack gas | #9aa3ab | #2c3640 |

Every stream moves at one fixed screen speed, and the card says the speed carries no meaning, so the motion never suggests a fluid velocity.

### 5.5 Controls and accessibility

- Every control is a real button reachable by keyboard. Previous and Next use `aria-disabled` at the ends, so focus never lands on a disabled element.
- Starting the tour moves focus to the step title; Exit, or Escape, closes it and returns focus to the start button. Escape closes an open stream tooltip first.
- Left and Right arrows step and Space pauses, except when focus is on a unit, a stream or a form control, where the drawing's own keys keep working.
- Each step is announced once through a visually hidden `aria-live` region; the unit panel's own live region is silenced while the tour runs.
- Pause stops every moving part, the dashes and the inset animations included.
- Under `prefers-reduced-motion: reduce` the tour is still: no dashes, no inset animation, no fades, no smooth scroll, no auto-advance and no Play button. The lit lines thicken and keep their drawn arrowheads. `pfd.js` unit focus scrolling also jumps under reduced motion.
- Text in the card, the bar and the insets uses the site tokens or fixed inset colours. Every text and background pair the tour uses was measured with the WCAG formula; the lowest is `--muted` on the figure tiles at 5.1:1, and the start button's white on the accent is 7.3:1.

### 5.6 Files

| File | Change |
|---|---|
| `frontend/tour.js` | New. Steps as data, rendering of the card, highlight and flow overlay, controls, keyboard, reduced motion, and `tourAudit` and `tourState` for the self test. |
| `frontend/tour.css` | New. Toolbar, control bar, card, insets, dimming and dash animation, reduced-motion and print rules. |
| `frontend/index.html` | Start button, control bar, caption card and live region; `data-tour` keys on the core outline, the boundary, the stack and the labels the steps light. |
| `frontend/pfd.js` | Loads the tour apart from the drawing so a fault there leaves the PFD working, passes it the drawing hooks, re-renders the open step on each reply; unit focus scrolling respects reduced motion. |
| `frontend/panels.js` | Exports the existing reply path resolver. |
| `frontend/selftest.js` | Eight tour checks (section 7). |
| `scripts/build_site.py` | Writes `data/tour_rows.json` from the three results rows; the build fails if a row is missing. |
| `scripts/selftest.py` | `--reduced-motion` emulates the media feature, so the whole self test can run against the still tour. |

## 6. Implementation plan for the rest of the brief

Each phase is a separate change with its own checks, in this order.

1. Camera and navigation on the PFD. Tween the SVG viewBox to a step's lit elements (fit to view with padding), add zoom in, zoom out and fit buttons outside the drawing, and a section list that reuses the tour's step data. Files: `tour.js`, `pfd.js`, `pfd.css`, `index.html`. Challenge: keeping stream tooltips anchored while the viewBox changes. Tests: the lit elements sit inside the visible viewBox at each step; the camera jumps with no tween under reduced motion; every new control is a button.
2. Live stream callouts. Numbers at path midpoints for the streams the reply carries values for, with precomputed rows marked. Files: `pfd.js`, `units.json`. Challenge: the live reply carries no stream table, so most stream numbers stay precomputed. Tests: each callout equals its source.
3. Tokens and the dark theme. Token pairs in `style.css`, a dark set, stream colours as tokens, contrast checked by a new self-test pass over computed styles. Files: `style.css`, `views.css`, `pfd.css`, `tour.css`, `print.css`. Challenge: the drawing's gradients are literal SVG colours and need a dark variant. Tests: every text pair at 4.5:1 in both themes, every stream at 3:1 against its page.
4. Cards, badges and the bottom summary. KPI tiles and panel headers on the card anatomy, model status as badges, a collapsible solver and balance summary on the PFD page. Files: `app.js`, `views.js`, `panels.js`, CSS. Tests: the existing panel and KPI checks, plus the summary against `messages` and `balances.closure`.
5. Background and value transitions. The static grid and glow, and KPI values that ease to the new reply value. Tests: the final displayed text equals the formatted reply value; nothing animates under reduced motion.

No phase adds a dependency, so the licence position stays as it is today. Any later wish for pinch zoom through d3-zoom, or for Tailwind, comes back for approval first because of the notices and MPL-2.0 points in section 2.

## 7. Validation

Only checks that ran are listed. The site was built from this branch into a scratch folder with the wasm module built from the unchanged engine, served on 127.0.0.1 and driven in headless Chrome.

| Check | Result |
|---|---|
| `scripts/build_wasm.py` with a scratch `CARGO_TARGET_DIR` | finished, 1,067,548 bytes |
| `scripts/build_site.py` | built, 33 files, about 3.53 MB, every local reference stamped and present |
| `scripts/selftest.py` at 1440 by 900 | PASS: 29 checks passed and `pfd_streams_bound` skipped, as it does on the main branch, because the live reply carries no stream table; 0 browser errors, 0 requests to another host |
| `scripts/selftest.py` at 375 by 812, phone emulation | PASS, same checks |
| `scripts/selftest.py --reduced-motion` at 1440 by 900 | PASS, same checks, with the tour checks run on the still tour |
| `tour_steps_bound` | every unit, stream, label key, reply path and results row the 13 steps name resolves |
| `tour_plays` | starts from its button, visits all 13 steps in order with dashes moving on every step that has flows, exits by its Exit button with focus back on the start button, no error |
| `tour_numbers_live` | 102 caption numbers over 13 steps at the default inputs, each equal to its reply field or results row |
| `tour_text_rules` | the prose of all 13 steps passes the visible text rules, with no placeholder left and no bare SMR |
| `tour_keyboard` | ArrowRight and ArrowLeft step, Space pauses and resumes, Escape closes an open stream tip first and then the tour, focus returns to the start button, every control is a button |
| `tour_reduced_motion` | 13 still steps add no running animation, the tour does not advance by itself, Play is hidden |
| `tour_autoplay` | at a fast test pace the tour runs to the last step and stops; dwell 7 to 8 s a step, 97 s in all |
| `tour_follows_controls` | at a 100 MWth rating step 1 shows the engine's infeasibility reason and the new rating, and step 13 states the engine's abatement verdict, with every number equal to the reply |
| Full play-through at normal speed, 1440 px | all 13 steps advanced by themselves and the tour stopped on the last step after 97.3 s, with the button reading Replay; no browser error |
| Full play-through at normal speed, 375 px phone emulation | the same, 97.4 s, no browser error |
| Screenshots of steps 1, 2, 3, 4, 7, 9, 11, 12 and 13 at both widths, in motion and under reduced motion | inspected (section 7.1) |
| Unit picker and drawing while the tour runs | picking a unit from the list opens its panel; clicking a unit during the tour pauses it and opens that unit's panel; Exit restores the reader's unit, the panel's live region and focus, and leaves no tour class or overlay on the drawing; leaving the PFD route closes the tour |
| Text contrast of every pair the tour uses, WCAG formula | lowest 5.1:1 (`--muted` on the figure tiles) |
| `scripts/check_site.py --site <site> --repo .` | PASS: site 33 files, 0 hits; repository 219 files, 0 hits |
| `scripts/check_site.py --site <site> --repo . --prove` | PASS: site proof 34 plants, 34 caught; repository proof 11 plants, 11 caught |
| `scripts/check_clean.py` | 221 files scanned, 0 hits |

### 7.1 Notes from the screenshots

The first set of screenshots showed captions too long for a 7 or 8 s step, the long layout note under the drawing pushing the caption card far below it on phones, and two labels of the fission inset running into each other. The captions were cut to two short paragraphs with the numbers in figure tiles, the layout note now steps aside while the tour runs, and the inset was laid out again. In the final set the card sits beside the drawing at 1440 px and directly under it at 375 px, where the drawing also scrolls sideways to the lit units of each step. Under reduced motion the lit lines are thicker with their drawn arrows, the fission inset shows the moment after the split as a still picture, and the bar has no Play button.

### 7.2 What this stage did not change or run

- No change under `engine/` or `results/`; the engine tests were not rerun here, since no engine file changed.
- No third-party code, no new dependency, no change to the notices.
- The dark theme, the camera, zoom and the other phases of section 6 are not built yet.
- Drawing questions left open: whether to redraw the reformer's helium connections as counterflow and the PSA beds with their real nozzles, and whether to draw the secondary circulator and the heat lines into the reboiler. The tour names each simplification on screen in the meantime.
