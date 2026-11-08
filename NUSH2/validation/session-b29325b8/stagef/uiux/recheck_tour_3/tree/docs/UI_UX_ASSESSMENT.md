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

A "Play the process tour" button sits in the toolbar above the drawing, beside the unit picker, with a line giving the step count and the play time. Pressing it opens a control bar above the drawing (Previous, Pause or Play, Next, Exit, a step counter and a segmented progress bar) and a caption card beside the drawing. On wide screens the card heads the side column; below 1200 px it sits directly under the drawing, and the long layout note under the drawing steps aside while the tour runs. The card ends with its own Previous and Next buttons and a line naming the keys the tour answers.

Each step dims the rest of the drawing, lights its units with a soft glow, keeps neighbouring units half lit, and moves dashes along its streams in the direction of flow. Streams of a step start one group after another, so the order of the flow reads on screen (for example primary helium out of the reactor, through the IHX, then back by the circulator). On narrow screens the drawing scrolls sideways to the lit part. The panel of the step's lead unit opens beside the card, and the reader's own unit comes back when the tour closes. The lead unit is marked by its glow; its selection ring stays off during the tour, because next to the power block and the cryogenic unit the ring would cross the drawing's labels. Choosing a unit or a stream on the drawing, by mouse or by Enter or Space, or from the unit list, pauses the tour, so the panel the reader opened stays until they move on.

### 5.2 Steps

| Step | Title | Lit units | Animated streams, in order | Numbers shown (reply path unless marked) |
|---|---|---|---|---|
| 1 | Fission heat in the reactor core | HTGR and its core | none; an illustration inset | reactor outlet, rating, heat output, margin |
| 2 | Primary helium loop | HTGR, IHX, circulator | P1, P2, P3 | IHX duty, primary helium flow at the need, primary circulator electricity, IHX approach |
| 3 | Secondary helium loop to the reformer | IHX, reformer | H1, H2 | helium supply and return, process heat with the feed preheat, secondary helium flow, secondary circulator electricity |
| 4 | Natural gas and steam feed the reformer | reformer | S01 with S02 | gas feed, reformer exit, helium supply; the tube limit in words |
| 5 | Water-gas shift | shift | S05 | shift inlet temperatures, shift heat release |
| 6 | Cooling and condensate | syngas cooler, separator | S09, S09B, then S10 with S11 | syngas heat to the reboiler, separator temperature, condensate |
| 7 | CO2 capture in the MEA absorber | absorber | S11 with S18, then S17 | MEA strength, absorber pressure, capture per pass, CO2 captured, solvent rate |
| 8 | Hydrogen product from the external PSA | PSA, boundary | S12, S13 | H2 into the PSA, assumed recovery, purified H2 per hour and per year |
| 9 | PSA reject back to the cryogenic unit (titled for the scenario when the reject is exported or burned on site) | recycle compressor, cryogenic unit, stack; the PSA when the reject leaves or is burned | S14, R1, then S21, S03 and S15 together with their continuations along S12 and S01, then S16; S14 then S16 when burned on site; none when exported | H2 and carbon in the reject, carbon recycled and burned, CO2 to the atmosphere |
| 10 | Stripper and reboiler heat | rich/lean exchanger, stripper, reboiler, lean cooler, pump | S17, L3, L5 with L6, L4, S18 | stripper pressure, reboiler duty, specific duty, the four heat sources in their approved order |
| 11 | CO2 compression, ship and offshore storage | condenser, CO2 compressor, storage | C1, S19, S20 (S19, S20 under Selexol) | compression electricity, CO2 shipped, tariff and charge |
| 12 | Power block and site electricity | power block, circulators, HTGR | P1, P4 (none when the branch carries nothing) | helium branch, electricity generated at the reactor output, efficiency, reactor steam to the reboiler, circulator and site demand, grid import; on an infeasible plant the branch and the turbine output at the need |
| 13 | The plant from end to end | whole plant | at once, every stream that an earlier step of the same scenario moves | purified H2, CO2 stored and released, baseline, net abatement on three baselines, target verdict and the engine's cost verdict |

The play-speed dwell is 7 or 8 s a step, 97 s for all 13. Under reduced motion there is no auto-advance.

### 5.3 Where each number comes from

Every caption number is read when the step renders, by its reply path, so it follows the controls; the shell's render hook re-renders the open step on each new reply. The tour prints no results row: where the reply holds no number for a limit, the caption states the limit in words, or quotes the unit's limitation text from `units.json`, which the page fills from the same reply. The page formats numbers (fixed decimals, and a fraction shown as a percentage) and computes nothing else. A no-break space joins each number to its unit, so the text read aloud or copied keeps them apart. Each figure tile says whether its value is live, an input or a fixed registry value.

The tour follows the plant as the engine models it, including its stated limits:

- Step 4 states that the engine sets the reformer exit at the equilibrium assumption of the Idaho National Laboratory (INL) study and that, with helium at the supply temperature, the tube model found no tube count in its search range that reaches it on the assumed tube.
- Step 8 says the PSA is external and not simulated, and its recovery is an assumption.
- Step 11 says the CO2 is compressed to INL's pipeline condition and that conditioning for the ship is not modelled, with the pipeline compression load standing in for it.
- Steps 1, 2, 3, 12 and 13 show the engine's own feasibility reason when the rating falls below the need. When heat duties sit above the rating, steps 2 and 3 give the IHX duty and the process heat as what the plant would need. On an infeasible plant step 12 gives the branch and the turbine output at the need and prints the generation at the reactor's rated output, so generation and grid import add up to the site demand.
- Step 12 splits the power branch as the engine does: the steam for the reboiler does back-pressure work in the turbine before it heats the reboiler, and the rest of the branch drives the turbine at the conversion efficiency.
- Step 13 states the abatement verdict and the cost verdict the engine gives. Net abatement at or above the target on an infeasible plant is said to be above the target and not counted as met. The CO2 never formed is credited to reactor heat replacing the reformer's fuel gas and, while the cryogenic recycle runs, to the methane it returns to the feed.
- When the PSA reject is exported, steps 9 and 13 stop animating the recycle lines and the stack. When it is burned on site, they light the reject's return and the stack, and step 9 says the burner is not drawn. Under Selexol, steps 6, 7, 10 and 11 leave out what belongs to the MEA plant, and step 9 drops the MEA limit on the cryogenic unit.
- Step 13 moves only the lines that steps 1 to 12 of the same scenario move, so a line those steps leave dark stays dark at the close: under Selexol the MEA regeneration lines L3 to L6 and the condenser line C1, and with the reject burned on site the power branch P4, which then carries nothing.
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
- Starting the tour brings the tour near the top of the window: the top of the layout when the card is in the side column, the bar when the card sits under the drawing. Focus then goes to the step title when it is fully in view, or else to the first control of the bar. Exit, or Escape, closes the tour and returns focus to the start button. Escape closes an open stream tooltip first.
- An automatic step, while the tour plays, scrolls neither the page nor the side column, wherever the reader has scrolled and wherever focus sits: it only redraws the step, with the drawing's own sideways scroll to the lit part on narrow screens, and announces it. When the card changes height above the part of the page the reader is looking at, the browser's own scroll anchoring may move the scroll position by that change so the view holds still; the tour itself scrolls neither the page nor the side column on an automatic step. Start keeps its own placement, and the steps that follow it by the timer obey the same rule.
- A step the reader asks for, with a Previous or Next button or with an arrow key, keeps the control they used whole in view with its focus ring, by the smallest scroll, and also brings the card head into view when the head and that control fit in the window together. When they do not fit, the control wins. With an arrow key the control is the focused element of the card or the bar; with focus elsewhere only the card head counts. Replay after the last step counts as such a step. The side column scrolls first when it holds the card, and then the page. The button used keeps focus.
- With focus off the drawing and off any form control, Left and Right arrows change the step; on the drawing or in a form control the arrows keep their own work. Space pauses or plays in the same places, except on a button or a link, which keeps its usual Space and Enter, so Space on the card's Previous or Next presses that button. The card's own Previous and Next buttons follow the step title in the tab order, and a line under them names these keys in the same terms.
- Each step is announced once through a visually hidden `aria-live` region; the unit panel's own live region is silenced while the tour runs.
- Pause stops every moving part, the dashes and the inset animations included.
- Under `prefers-reduced-motion: reduce` the tour is still: no dashes, no inset animation, no fades, no smooth scroll, no auto-advance and no Play button. The lit lines thicken and keep their drawn arrowheads. `pfd.js` unit focus scrolling also jumps under reduced motion.
- Text in the card, the bar and the insets uses the site tokens or fixed inset colours. Every text and background pair the tour uses was measured with the WCAG formula; the lowest is `--muted` on the figure tiles at 5.1:1, and the start button's white on the accent is 7.3:1.

### 5.6 Files

| File | Change |
|---|---|
| `frontend/tour.js` | New. Steps as data, rendering of the card, highlight and flow overlay, controls, keyboard, reduced motion, and `tourAudit` and `tourState` for the self test. |
| `frontend/tour.css` | New. Toolbar, control bar, card, insets, dimming and dash animation, reduced-motion and print rules. |
| `frontend/index.html` | Start button, control bar, caption card with its Previous and Next buttons and key line, and live region; `data-tour` keys on the core outline, the boundary, the stack and the labels the steps light. |
| `frontend/pfd.js` | Loads the tour apart from the drawing so a fault there leaves the PFD working, passes it the drawing hooks, re-renders the open step on each reply; unit focus scrolling respects reduced motion. |
| `frontend/panels.js` | Exports the existing reply path resolver. |
| `frontend/selftest.js` | Twelve tour checks (section 7). |
| `scripts/selftest.py` | `--reduced-motion` emulates the media feature, so the whole self test can run against the still tour. |
| `.github/workflows/site.yml` | Runs the browser self test a second time with `--reduced-motion`. |

## 6. Implementation plan for the rest of the brief

Each phase is a separate change with its own checks, in this order.

1. Camera and navigation on the PFD. Tween the SVG viewBox to a step's lit elements (fit to view with padding), add zoom in, zoom out and fit buttons outside the drawing, and a section list that reuses the tour's step data. Files: `tour.js`, `pfd.js`, `pfd.css`, `index.html`. Challenge: keeping stream tooltips anchored while the viewBox changes. Tests: the lit elements sit inside the visible viewBox at each step; the camera jumps with no tween under reduced motion; every new control is a button.
2. Live stream callouts. Numbers at path midpoints for the streams the reply carries values for, with precomputed rows marked. Files: `pfd.js`, `units.json`. Challenge: the live reply carries no stream table, so most stream numbers stay precomputed. Tests: each callout equals its source.
3. Tokens and the dark theme. Token pairs in `style.css`, a dark set, stream colours as tokens, contrast checked by a new self-test pass over computed styles. Files: `style.css`, `views.css`, `pfd.css`, `tour.css`, `print.css`. Challenge: the drawing's gradients are literal SVG colours and need a dark variant. Tests: every text pair at 4.5:1 in both themes, every stream at 3:1 against its page.
4. Cards, badges and the bottom summary. KPI tiles and panel headers on the card anatomy, model status as badges, a collapsible solver and balance summary on the PFD page. Files: `app.js`, `views.js`, `panels.js`, CSS. Tests: the existing panel and KPI checks, plus the summary against `messages` and `balances.closure`.
5. Background and value transitions. The static grid and glow, and KPI values that ease to the new reply value. Tests: the final displayed text equals the formatted reply value; nothing animates under reduced motion.

No phase adds a dependency, so the licence position stays as it is today. Any later wish for pinch zoom through d3-zoom, or for Tailwind, comes back for approval first because of the notices and MPL-2.0 points in section 2.

## 7. Validation

Only checks that ran are listed, all on the frontend this revision commits. The wasm module was built from the unchanged engine and the site from this branch into a scratch folder, both stamped with the parent revision 2f244e8, then served on 127.0.0.1 and driven in headless Chrome. Checks of earlier rounds that were not run again on this revision are left out.

| Check | Result |
|---|---|
| `scripts/build_wasm.py` with a new scratch `CARGO_TARGET_DIR` | finished, 1,067,548 bytes, with the same SHA-256 as the module of the previous round |
| `scripts/check_wasm.py` | PASS, 0 imports |
| `scripts/build_site.py` | built, 32 files, about 3.56 MB |
| `scripts/selftest.py --expect-commit` at 1440 by 900 | PASS: 33 checks passed and `pfd_streams_bound` skipped, as it does on the main branch, because the live reply carries no stream table; 0 browser errors, 0 requests to another host |
| The same at 375 by 812 with phone emulation | PASS, same checks |
| The same with `--reduced-motion`, at 1440 by 900 and at 375 by 812 with phone emulation | PASS, same checks, with the tour checks run on the still tour; `tour_auto_stays` forces motion at a test pace there too, since the timer is what it tests |
| The same at 1024 by 768 | PASS, same checks |
| The same at 1024 by 600 | 32 checks passed and `pfd_streams_bound` skipped; `tour_keyboard` failed. At this size Start gives focus to Play, and the check sends its Space to that button, where the tour leaves Space to the button as the key line says. The site of 2f244e8 fails the same check at this size. The CI runs the self test at 1440 by 900, where this check passes |
| `tour_steps_bound` | every unit, stream, label key, text key and reply path the 13 steps name resolves, over 16 scenario branches (each reject route with zero and non-zero values, on a feasible and an infeasible plant) |
| `tour_plays` | starts from its button with focus on the step title in view, visits all 13 steps in order with dashes moving on every step that has flows, exits by its Exit button with focus back on the start button, no error |
| `tour_numbers_live` | 100 caption numbers over 13 steps at the default inputs, each equal to its reply field |
| `tour_text_rules` | the prose of all 13 steps passes the visible text rules, with no placeholder left, no bare SMR, no em dash and no number joined to its unit |
| `tour_keyboard` | ArrowRight and ArrowLeft step, Space pauses and resumes, Escape closes an open stream tip first and then the tour, focus returns to the start button, every control in the bar and the card is a button |
| `tour_reduced_motion` | 13 still steps add no running animation and the tour does not advance by itself, with Play hidden |
| `tour_autoplay` | at a fast test pace the tour runs to the last step and stops; dwell 7 to 8 s a step, 97 s in all |
| `tour_follows_controls` | at a 100 MWth rating: step 1 shows the engine's infeasibility reason and the new rating, steps 2 and 3 read their heat at the need, step 12 prints 0.00 MWe generated against 19.78 MWe of import and demand, and step 13 says the abatement is above the target but not counted as met; at Case A: step 12 prints 6.83 MWe generated and 22.10 MWe imported against 28.94 MWe of demand, with the same abatement verdict; 51 numbers equal to the reply |
| `tour_focus_kept` | in each run, 36 step changes, by the card's own buttons to step 13 and back and then by the bar's Next to step 13; the button used kept focus on every step and lay whole, with its focus ring, inside the window and inside the scrolling side column |
| `tour_head_shown`, new | in each run, 60 step changes: the 36 above and 24 more by the card's Next and Previous with the button first placed near the top of where it shows, so that a shorter card would push it out. The card head showed whole on every step change where it fits with the button (54 at 1440 by 900, 52 on the phone); on the others the button came first and kept focus, whole in view |
| `tour_auto_stays`, new | each case runs through one automatic step. Two cases put focus on a button and scroll away from it: Play with the page scrolled to the card, and the card's Next with the page scrolled to the drawing. A further case plays on from Start with the focus Start gave and the page scrolled to the card. The side column, where it scrolls, sat off its top at 120 px or 60 px. In every run neither the page nor the side column moved, and focus stayed put. At 1440 by 900 the page sat at 471 px; on the phone at 1190 px and 665 px |
| `tour_closing_lines` | for each reject route the closing step lights exactly the lines that steps 1 to 12 move, and its dashes run on those lines only: recycled, 30 lines and none dark; exported, 24 lines with S03, S14, S15, S16, S21 and R1 dark; burned on site, 25 lines with P4 (a branch of 0.0 MWth), S03, S15, S21 and R1 dark; Selexol, 25 lines with L3 to L6 and C1 dark |
| Planted faults: the self test on a copy of the built site carrying the `tour.js` and `tour.css` of 2f244e8, at 1440 by 900, 1024 by 768, 1024 by 600 and 375 by 812 with phone emulation | FAIL as intended at every size. `tour_auto_stays` caught the automatic step that scrolls back to the focused button: at 1024 by 600 the page went from 993 px to 422 px after Start with focus on Play, and at 375 by 812 from 665 px to 1067 px with focus on the card's Next. At 1440 by 900 the side column went back from 120 px to 0. `tour_head_shown` caught the hidden head at each of the other sizes, for example at step 2 a head at -467 to -400 px with the card's Next at 8 to 46 px. `tour_focus_kept` passed there, as 2f244e8 already kept the button in view. At 1024 by 600 `tour_keyboard` failed there as well, as above |
| Before the change, on a site built from 2f244e8 | an automatic step after Start, with focus on Play, moved the page up by 571 px at 1024 by 600 and by 610 px at 375 by 667; with focus on the card's Next it moved the page down by 483 px at 1024 by 600 and by 505 px at 375 by 812. The first press of the card's Next at 1024 by 768 and 1024 by 600 left the step 2 head at -455 px with the button at 8 px, although the two fit together |
| The reader scrolls away while the tour plays, mouse wheel input, at 13 window sizes from 375 by 667 to 2560 by 1440 (phone emulation at the phone and tablet sizes) | Four ways in: Start by mouse, Start by Enter, Play pressed again after Pause, and the card's Next pressed while playing. The page was then wheeled to the card, or to the drawing after the card's Next. At the next automatic step the page moved 0 px in all 52 runs and the side column stayed where it was. The run with the card's Next, repeated with the browser's scroll anchoring off, also moved 0 px at all 13 sizes |
| A play-through to step 13 at a fast pace with the reader scrolled to the card, at the same 13 sizes, with scroll anchoring on and off | the scroll position stayed the same on all 130 automatic steps in each mode, and focus stayed put |
| The same with the reader scrolled to the unit panel under the card | with scroll anchoring off the scroll position stayed the same on all 126 automatic steps. With it on, the browser moved the scroll position on 106 of them, by up to 455 px, to hold the panel in place while the card above it changed height: the panel stayed within 1 px on screen each time |
| The same at the real dwell, from Start by mouse to step 13 | reader at the card at 1440 by 900 and 375 by 812, and at 1024 by 600 with focus on Play: page unchanged on all 12 automatic steps at each size; reader at the unit panel at 1440 by 900 and 375 by 812: the browser's scroll anchoring moved the scroll position by up to 265 px and 279 px, with the panel held in place on all 12 steps |
| The first press of the card's Next, reached by Tab from the step title or by wheeling it to the middle of the window and clicking, at the 13 sizes | the step 2 head showed with the button on all 22 presses where the two fit together; on the other 4 (812 by 375 and 375 by 667) the button came first, whole |
| The card's Next to each of steps 2 to 13 and the card's Previous to each of steps 12 to 1, with the button first wheeled to the middle of the window, at the 13 sizes | the card head showed with the button after every press where the two fit together, the head counted as the whole card header. A count with the kicker line alone as the head flagged one press at 1440 by 900 (Previous to step 1), where the kicker and the button fit with 2 px to spare but the whole header does not, so the button came first |
| Card's Next after Enter from step 10, real key events, at 8 window sizes from 375 by 667 to 1920 by 1080 | focus stayed on it and it showed whole (38 of 38 px) at each of steps 11 to 13 at every size; at 1440 by 900 step 11 puts it at 846 to 884 px |
| Every step from the keyboard, real key events, at the 13 sizes, with and without reduced motion | 60 step changes a size. Focus stayed on the button used and the button showed whole with its ring on all 60 at all 13 sizes in both modes. The card head sits above the window on some steps on short windows (36 of 60 at 812 by 375, 27 at 375 by 667) |
| The card's Next and Previous and the bar's Next and Previous, each by Enter and by mouse, at the 13 sizes, with and without reduced motion | 96 step changes a size; focus stayed on the button and the button showed whole on all of them at all 13 sizes in both modes, and no card head was hidden where it fits with the button |
| Card and focus after Start at the same 13 sizes, with and without reduced motion | 13 of 13 each time: the card head is at or below the top of the window with the focused element fully in view, and after one ArrowRight from the title the card head is still in view |
| A full keyboard tour at 1440 by 900 and at 375 by 812 with phone emulation, each with and without reduced motion: Tab to Start, Enter, steps 1 to 7 by ArrowRight, steps 7 to 13 by the card's Next | all 13 steps visited with the focused element in view on each (the card's Next at step 11 at 846 to 884 px and at 766 to 804 px), Enter on the disabled last Next stays put, Escape returns focus to the start button, no browser error; under reduced motion no animation or dash appears on any step, and the tour is still on step 1 after 9 s |
| Keyboard run with real key events at 1440 by 900 | 20 of 20 checks passed, among them Enter and Space on the bar's Next, which each press it |
| Space on the card's Next while the tour plays | moves to step 2 and the tour keeps playing, as the key line states |
| A unit chosen on the drawing by Enter or by mouse while the tour plays | the tour pauses, and the unit's panel is still open 10.2 s and 10.0 s later |
| A stream tip pinned with Enter, or a unit chosen from the unit list, while the tour plays | the tour pauses; the tip is still pinned after 10.1 s, and the chosen panel is still open after 9.7 s and 9.8 s |
| Tab from the step title at 1440 by 900 and at 375 by 812 | reaches the card's Previous and then its Next |
| Side column scrolled to 700 px, then ArrowRight with focus on the title | the side column scrolls back to its top and the card head shows at 9 to 77 px |
| Full mouse tours by the bar's Next and full automatic play at the real dwell, at 1440 by 900 and at 375 by 812 with phone emulation | 13 of 13 steps each time; the automatic play ends at 97.6 s with the button reading Replay; Exit returns focus to the start button and leaves no tour class; no browser error, and only 127.0.0.1 requested |
| The closing step at 8 input sets (each reject route, a 100 MWth rating, Case A, Case A with the reject burned, and Selexol at 100 MWth) | 8 of 8: the lit lines equal the lines that steps 1 to 12 move, with one dash path a lit line and none off them |
| Every caption at 12 input sets | each step record is identical, field for field, to the record from the site of 2f244e8; 1155 caption numbers, each with a reply path and equal to its reply value, 10 of them shown as n/a where the reply has no value |
| `scripts/check_site.py --site <site> --repo .` | PASS: site 32 files, 0 hits; repository 219 files, 0 hits |
| `scripts/check_site.py --site <site> --repo . --prove` | PASS: site proof 34 plants, 34 caught; repository proof 11 plants, 11 caught |
| `scripts/check_clean.py` | 221 files scanned, 0 hits |
| The pre-push scan kept outside the repository, on the tree with code only | 222 files, 0 hits |

### 7.1 What this stage did not change or run

- No change under `engine/` or `results/`; the engine tests were not rerun here, since no engine file changed.
- No third-party code and no new dependency, so the notices are unchanged.
- Checks of earlier rounds on parts this change leaves alone were not run again here: the screenshots, the close-ups, the read-through of every caption at seven input sets and the contrast measure. This round changes how the tour scrolls, and no visible text or colour.
- The dark theme, the camera, zoom and the other phases of section 6 are not built yet.
- Drawing questions left open: whether to redraw the reformer's helium connections as counterflow and the PSA beds with their real nozzles, whether to draw the secondary circulator, the heat lines into the reboiler and the burner, and whether the drawing's own labels (the power block's output, the reformer's process heat) should read the delivered values on an infeasible plant. The tour names each simplification on screen in the meantime.
