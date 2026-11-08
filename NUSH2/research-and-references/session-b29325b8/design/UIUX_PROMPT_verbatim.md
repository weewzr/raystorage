<!-- User message 2026-11-03: "after you push, initiate this prompt:" followed by the text below, verbatim. Runs only after the deliverables push is complete. -->

# NUSH2 — Advanced Engineering Simulation UI/UX Enhancement

## 1. Project Overview

I am developing **NUSH2**, an interactive, physics-based engineering simulation platform for an integrated nuclear-assisted hydrogen production and carbon capture system.

**Primary GitHub Repository:**

https://github.com/weewzr/NUSH2

The integrated process consists of:

**High-Temperature Gas-Cooled Reactor (HTGR) → Intermediate Heat Exchanger (IHX) → Steam Methane Reforming (SMR) → Water-Gas Shift (WGS) → Monoethanolamine (MEA) Carbon Capture and Regeneration**

The simulation aims to model the interconnected physical and chemical processes involved in nuclear-assisted hydrogen production and CO₂ capture, including:

- Thermodynamic properties and phase behaviour.
- Chemical reaction kinetics and equilibrium.
- Heat and mass transfer.
- Material and energy balances.
- Equipment operating conditions.
- Process integration and thermal coupling.
- Numerical calculations and convergence.
- Hydrogen production and CO₂ capture performance.
- Engineering sensitivity analysis and scenario evaluation.

My objective is to transform NUSH2 into a **professional, visually sophisticated engineering simulation platform** comparable in presentation quality to modern industrial digital twins, advanced process simulation software and scientific computing applications.

The interface should combine rigorous engineering calculations with intuitive controls, interactive process visualisations, smooth animations and a premium industrial aesthetic.

**The primary focus of this task is improving the frontend, UI/UX, background design, interactivity and visual presentation of NUSH2 while preserving its existing simulation functionality and scientific integrity.**

---

# 2. Recommended Open-Source GitHub Repositories

I would like you to evaluate and potentially integrate components from the following three repositories.

Do not automatically install every component. First inspect NUSH2's existing architecture and identify which components provide meaningful improvements.

## 2.1 React Flow — Interactive Engineering Process Flowsheet

**GitHub Repository:**

https://github.com/xyflow/xyflow

**Documentation and Interactive Examples:**

https://reactflow.dev/examples

### Proposed Purpose

Use React Flow to create or enhance an interactive process-flow diagram representing the integrated HTGR–SMR–MEA carbon capture system.

The process flowsheet should become the central interactive workspace of NUSH2.

### A. Interactive Equipment Nodes

Represent individual process equipment as interactive nodes, including:

- High-Temperature Gas-Cooled Reactor (HTGR).
- Intermediate Heat Exchanger (IHX).
- Steam Methane Reformer (SMR).
- Water-Gas Shift Reactor (WGS).
- MEA Absorber.
- MEA Stripper / Regenerator.
- Heat exchangers.
- Compressors.
- Pumps.
- Coolers.
- Separators.
- Other equipment already implemented in NUSH2.

Each equipment node should support:

1. A professional engineering equipment icon or schematic.
2. Equipment identification and operating status.
3. Relevant operating parameters.
4. Visual indicators for warnings or convergence failures.
5. Interactive selection and highlighting.
6. Access to detailed engineering calculations and equipment properties.

Equipment symbols should resemble recognised process-engineering conventions wherever practical.

### B. Animated Material and Energy Streams

Visualise the connections between process equipment using interactive, animated stream paths.

Potential features include:

- Animated flow-direction indicators.
- Colour-coded material and energy streams.
- Stream identification numbers.
- Temperature and pressure labels.
- Mass and molar flow rates.
- Chemical compositions.
- Enthalpy and energy-transfer information.
- Stream-specific tooltips and information panels.

Use a consistent colour system to distinguish relevant process streams, such as hydrogen-rich gas, CO₂-rich gas, steam, cooling water, helium and thermal energy.

Animations must remain scientifically appropriate.

For example, particle movement may indicate flow direction, but animation speed must not imply actual fluid velocity unless explicitly linked to validated simulation data.

### C. Interactive Process Navigation

Implement:

- Zooming and panning.
- Equipment selection.
- Stream selection.
- Equipment highlighting.
- Fit-to-screen controls.
- Process-section navigation.
- Expandable equipment information.
- Optional minimap.
- Clear visual differentiation between connected process sections.

Users should be able to inspect both the complete integrated plant and individual equipment models.

### D. Simulation Integration

Connect the flowsheet to NUSH2's existing simulation engine.

When users modify operating parameters and execute a simulation:

1. The simulation engine processes the updated inputs.
2. Equipment outputs are recalculated.
3. Material and energy streams are updated.
4. The flowsheet displays the latest calculated results.
5. Relevant engineering warnings or convergence failures are highlighted.

Preserve the existing simulation calculations and data structures.

**Expected Outcome:**

A professional, interactive, DWSIM-inspired engineering flowsheet that serves as the central visualisation and navigation interface for NUSH2.

---

## 2.2 shadcn/ui — Professional Engineering Dashboard and Controls

**GitHub Repository:**

https://github.com/shadcn-ui/ui

**Documentation and Component Gallery:**

https://ui.shadcn.com

### Proposed Purpose

Use shadcn/ui to establish a consistent, modern and professional application interface.

The design should resemble advanced industrial engineering software while maintaining the usability of a contemporary scientific dashboard.

### A. Main Simulation Workspace

Propose a layout consisting of five primary interface regions.

**1. Top Navigation Bar**

Include:

- NUSH2 branding.
- Project or simulation name.
- Simulation status.
- Scenario selection.
- Run simulation button.
- Reset controls.
- Save and export options, where supported.
- Settings.

**2. Left Sidebar — Process Navigation**

Provide navigation to:

- Integrated plant overview.
- HTGR.
- Intermediate Heat Exchanger.
- Steam Methane Reformer.
- Water-Gas Shift Reactor.
- MEA Absorber.
- MEA Regenerator.
- Carbon capture performance.
- Energy integration.
- Sensitivity analysis.
- Simulation results.

Adapt the navigation to the actual modules and capabilities found in NUSH2.

**3. Central Workspace**

Display the interactive process flowsheet developed using React Flow, if compatible.

This should occupy the largest portion of the application.

**4. Right Sidebar — Equipment Properties**

When an equipment node is selected, display:

- Equipment information.
- Operating parameters.
- Adjustable inputs.
- Calculated outputs.
- Engineering constraints.
- Relevant equations.
- Assumptions.
- Model status and warnings.

**5. Bottom Panel — Simulation Results**

Provide expandable panels containing:

- Numerical solver status.
- Convergence information.
- Material balance checks.
- Energy balance checks.
- Calculation logs.
- Engineering warnings.
- Simulation errors.

### B. Interactive Engineering Controls

Use appropriate components for:

- Temperature inputs.
- Pressure inputs.
- Mass flow rates.
- Molar flow rates.
- Reactor operating conditions.
- Solvent concentrations.
- Heat exchanger parameters.
- Process operating scenarios.
- Other model-specific inputs.

Potential UI elements include:

- Numerical input fields.
- Sliders.
- Dropdown menus.
- Tabs.
- Toggles.
- Tooltips.
- Dialog boxes.
- Collapsible panels.
- Validation messages.

All controls should display the correct engineering units and enforce appropriate input constraints.

### C. Engineering Performance Dashboard

Create a polished dashboard for key simulation outputs.

Potential indicators include:

- Hydrogen production rate.
- CO₂ capture efficiency.
- Reactor thermal power.
- Heat integration performance.
- Energy consumption.
- Equipment efficiency.
- Carbon emissions.
- Other performance indicators supported by NUSH2.

Use large, clearly labelled KPI cards with appropriate units.

Where useful, include interactive charts for:

- Temperature profiles.
- Pressure profiles.
- Equipment performance.
- Energy balances.
- Sensitivity analysis.
- Scenario comparisons.
- Carbon capture performance.

Use existing charting libraries where possible.

### D. Professional Dashboard Design

Apply:

- Consistent typography.
- Dark-mode styling.
- Clear information hierarchy.
- Responsive layouts.
- Accessible controls.
- Consistent spacing.
- Professional engineering tables.
- Collapsible sidebars.
- Resizable panels where practical.

**Expected Outcome:**

A cohesive, professional engineering simulation dashboard with intuitive navigation, structured controls and clear presentation of technical results.

---

## 2.3 Aceternity UI — Futuristic Backgrounds, Lighting and Visual Effects

**GitHub Repository:**

https://github.com/aceternity/aceternity-ui

**Component Gallery:**

https://ui.aceternity.com

### Proposed Purpose

Use selected Aceternity UI components to improve the visual sophistication of NUSH2.

The objective is to create a premium industrial digital-twin aesthetic while preserving the clarity and readability expected of professional engineering software.

### A. Animated Engineering Backgrounds

Explore components such as:

- Background Grid.
- Background Beams.
- Spotlight.
- Subtle gradient backgrounds.
- Other suitable animated background components.

Potential applications include:

1. A dark navy or charcoal dashboard background.
2. Subtle technical grid patterns.
3. Soft blue and teal illumination.
4. Restrained ambient animations.
5. Background effects that respond subtly to user interaction.

Background animations should remain unobtrusive.

The central engineering flowsheet must remain clearly visible.

### B. Interactive Equipment Cards

Explore:

- Card Spotlight.
- Glowing borders.
- Hover illumination.
- Smooth highlighting effects.
- Animated selection indicators.

Potential applications include:

- Highlighting selected equipment.
- Distinguishing active process sections.
- Displaying equipment information.
- Drawing attention to important simulation outputs.
- Indicating equipment warnings.

Use effects consistently and avoid excessive visual decoration.

### C. Animated Performance Indicators

Improve KPI cards using:

- Smooth numerical transitions.
- Subtle animated borders.
- Soft illumination.
- Interactive hover effects.
- Refined loading states.

For example, after completing a simulation, hydrogen production and CO₂ capture indicators could transition smoothly from their previous displayed values to the newly calculated results.

The underlying numerical results must remain unchanged.

### D. Simulation Interaction Effects

Introduce polished visual feedback for:

- Running simulations.
- Loading results.
- Selecting equipment.
- Switching process sections.
- Opening parameter panels.
- Comparing scenarios.
- Displaying warnings.
- Completing calculations.

Animations should reflect actual application states.

Do not create artificial progress percentages or misleading solver-status messages.

### E. Overall Visual Identity

Adopt a consistent design language featuring:

- Dark navy and charcoal backgrounds.
- Teal and blue accents.
- Restrained green highlights.
- Subtle glowing borders.
- Technical grid patterns.
- Professional typography.
- Smooth interactions.
- High-contrast engineering diagrams.

**Expected Outcome:**

A visually sophisticated, modern engineering simulation platform with subtle futuristic effects and a professional industrial aesthetic.

---

# 3. NUSH2 Repository and Architecture Assessment

**Primary Repository:**

https://github.com/weewzr/NUSH2

Before implementing any modifications, thoroughly inspect the existing NUSH2 codebase.

### 3.1 Existing Architecture

Identify:

1. Frontend framework and technologies.
2. Backend architecture.
3. Numerical simulation engine.
4. Existing thermodynamic and kinetic models.
5. Existing equipment models.
6. Current process-flow visualisation implementation.
7. Charting and data visualisation libraries.
8. Current CSS and styling architecture.
9. Existing UI components.
10. Dependencies and build configuration.
11. Data structures connecting the frontend and simulation engine.
12. Deployment and hosting configuration.

Determine whether NUSH2 uses React, another frontend framework or vanilla JavaScript.

### 3.2 Library Compatibility

Evaluate the suitability of:

- React Flow.
- shadcn/ui.
- Aceternity UI.

For each repository, assess:

- Compatibility with NUSH2's existing frontend.
- Installation requirements.
- Required dependencies.
- Integration complexity.
- Performance implications.
- Maintenance requirements.
- Licensing considerations.
- Whether equivalent functionality already exists.

Prioritise solutions that integrate naturally with NUSH2.

Avoid unnecessary architectural changes or complete frontend migrations unless clearly justified.

### 3.3 Licensing Requirements

NUSH2 is being developed as an independent engineering simulation platform.

Licensing flexibility is important for academic research, open-source development and potential future commercialisation.

Before integrating any external library:

1. Verify its current licence from the actual repository.
2. Identify the licences of relevant dependencies.
3. Assess compatibility with NUSH2's existing licence and intended distribution model.
4. Identify attribution or redistribution obligations.
5. Avoid introducing restrictive copyleft dependencies without explicit approval.
6. Document the libraries and components incorporated into NUSH2.

Do not assume that every component within a repository has identical licensing terms.

---

# 4. Scientific and Engineering Integrity

**The UI/UX enhancement must preserve NUSH2's existing engineering calculations and simulation functionality.**

Do not modify the underlying physical or chemical models solely to accommodate visual enhancements.

Preserve all existing implementations of:

- Thermodynamic calculations.
- Reaction kinetics.
- Chemical equilibrium.
- Heat transfer.
- Mass transfer.
- Material balances.
- Energy balances.
- Equipment models.
- Numerical solvers.
- Convergence algorithms.
- Engineering constraints.
- Simulation assumptions.

### 4.1 Simulation Data Integrity

All displayed engineering values must originate from actual simulation calculations or clearly identified user-defined inputs.

Do not introduce:

- Fabricated engineering results.
- Placeholder values presented as calculated outputs.
- Artificial simulation performance indicators.
- Misleading animations.
- Unsupported engineering claims.
- Hardcoded results that bypass the simulation engine.

If a particular calculation or feature is not yet implemented, clearly indicate its unavailable or developmental status.

### 4.2 Frontend–Backend Separation

Maintain a clear separation between:

**Simulation Backend:** Responsible for engineering calculations, numerical models and physical accuracy.

**Frontend:** Responsible for user interaction, visualisation, parameter entry and presentation of calculated results.

Ensure UI enhancements do not interfere with the existing simulation engine.

### 4.3 Simulation Validation

After implementing frontend changes:

1. Verify existing simulation inputs remain functional.
2. Confirm numerical results remain consistent.
3. Check that displayed values match backend outputs.
4. Validate engineering units.
5. Test equipment and stream selection.
6. Test simulation execution.
7. Verify error handling.
8. Confirm that animations reflect actual application states.
9. Check performance and responsiveness.
10. Run existing automated tests and report any failures.

---

# 5. Proposed NUSH2 Visual Design

## Design Direction: Siemens × NVIDIA Industrial Digital Twin

Develop a hybrid design combining:

**Siemens-inspired engineering software**
- Clear process diagrams.
- Precise technical information.
- Professional industrial interfaces.
- Structured operating parameters.

**NVIDIA-inspired scientific visualisation**
- Sophisticated dark backgrounds.
- Interactive technical graphics.
- Subtle illumination.
- Smooth visual transitions.
- Modern digital-twin aesthetics.

**Modern SaaS Dashboard Design**
- Clean navigation.
- Well-organised information panels.
- Intuitive controls.
- Responsive layouts.
- Consistent typography.

### Suggested Colour Palette

| Interface Element | Suggested Colour |
|---|---|
| Main background | Deep navy / charcoal |
| Secondary background | Dark slate |
| Primary accent | Cyan / teal |
| Secondary accent | Electric blue |
| Success indicators | Green |
| Warning indicators | Amber |
| Error indicators | Red |
| Primary text | White / light grey |
| Secondary text | Muted grey-blue |

Maintain consistent contrast and readability.

### Design Principles

1. Scientific credibility.
2. Professional appearance.
3. Clear information hierarchy.
4. Intuitive navigation.
5. Smooth interaction.
6. Responsive performance.
7. Minimal visual clutter.
8. Consistent engineering conventions.
9. Accessibility.
10. Maintainability.

Avoid excessive animations, unnecessary 3D effects and distracting decorative elements.

---

# 6. Recommended Implementation Roadmap

## Phase 1 — Repository Audit

Inspect the NUSH2 repository and document:

- Existing architecture.
- Current UI components.
- Simulation functionality.
- Dependencies.
- Integration opportunities.
- Licensing considerations.

Identify the most appropriate components from the three recommended repositories.

## Phase 2 — Interactive Process Flowsheet

Implement or enhance the process flowsheet using React Flow if technically appropriate.

Prioritise:

- Equipment visualisation.
- Stream connections.
- Equipment selection.
- Process navigation.
- Engineering parameter display.
- Integration with actual simulation outputs.

## Phase 3 — Professional Dashboard

Integrate suitable shadcn/ui components.

Improve:

- Navigation.
- Simulation controls.
- Equipment information panels.
- Parameter inputs.
- Results dashboards.
- Engineering charts.
- Scenario comparisons.

## Phase 4 — Visual Enhancement

Introduce selected Aceternity UI components.

Improve:

- Background design.
- Lighting effects.
- KPI cards.
- Hover interactions.
- Panel transitions.
- Simulation loading states.
- Overall visual consistency.

## Phase 5 — Backend Integration

Connect all new frontend components to the existing simulation engine.

Ensure that:

- Parameter changes are correctly processed.
- Equipment results update correctly.
- Material and energy streams reflect calculated values.
- Simulation errors are displayed accurately.
- Existing numerical functionality remains intact.

## Phase 6 — Testing and Optimisation

Perform:

- Functional testing.
- Numerical regression testing.
- Frontend performance testing.
- Responsive layout testing.
- Browser compatibility testing.
- Accessibility checks.
- Dependency and licensing review.

Resolve identified issues before finalising the implementation.

---

# 7. Expected Deliverables

Please provide the following:

### Deliverable 1 — NUSH2 Architecture Assessment

A detailed review of the current repository, including its frontend, backend, simulation engine, dependencies and existing visualisation capabilities.

### Deliverable 2 — GitHub Library Compatibility Analysis

Evaluate React Flow, shadcn/ui and Aceternity UI.

Explain which components should be integrated, which should be avoided and why.

### Deliverable 3 — Proposed UI/UX Architecture

Recommend an application layout and component structure suitable for NUSH2.

Explain how the proposed interface connects to the existing simulation engine.

### Deliverable 4 — Visual Design Proposal

Develop a cohesive visual design direction, including:

- Colour palette.
- Background design.
- Navigation layout.
- Process flowsheet.
- Equipment cards.
- Engineering controls.
- Results dashboards.
- Animations and transitions.

### Deliverable 5 — Implementation Plan

Provide a phased development plan with:

- Specific files and components requiring modification.
- Dependencies.
- Integration steps.
- Potential technical challenges.
- Licensing considerations.
- Testing requirements.

### Deliverable 6 — Frontend Implementation

Where repository access and the development environment permit, implement the recommended improvements directly in NUSH2.

Preserve existing functionality and minimise unnecessary modifications.

### Deliverable 7 — Validation Report

Document:

- Changes implemented.
- Components integrated.
- Dependencies introduced.
- Licensing implications.
- Tests performed.
- Existing functionality preserved.
- Remaining limitations.
- Recommended future improvements.

---

# 8. Critical Development Instructions

Follow these requirements throughout the project:

1. **NUSH2 is the sole target repository.** Base all architectural decisions on its actual implementation.

2. **Inspect before modifying.** Do not assume the existing technology stack, frontend framework or simulation capabilities.

3. **Preserve scientific accuracy.** Maintain the existing engineering models and numerical calculations.

4. **Prioritise meaningful improvements.** Introduce components that enhance usability, visualisation or presentation quality.

5. **Avoid unnecessary rebuilding.** Reuse existing infrastructure and components whenever practical.

6. **Maintain licensing flexibility.** Verify third-party licences and avoid introducing restrictive dependencies without approval.

7. **Preserve simulation functionality.** All existing controls, calculations, outputs and workflows must remain operational.

8. **Maintain technical credibility.** Visual effects must not misrepresent engineering results or physical behaviour.

9. **Optimise performance.** Animations and background effects must not significantly degrade simulation responsiveness.

10. **Implement incrementally.** Make changes in manageable stages and validate functionality after each stage.

11. **Do not overwrite existing work unnecessarily.** Preserve existing files, data, configuration and functionality.

12. **Keep the interface extensible.** The architecture should support future equipment models, process configurations and engineering analysis capabilities.

13. **Maintain a consistent design system.** Avoid mixing unrelated visual styles across different simulation modules.

14. **Prioritise usability.** The application should remain intuitive for engineering students, researchers and technical users.

15. **Report limitations transparently.** Clearly identify any proposed feature that cannot be implemented with the existing architecture or simulation capabilities.

---

# 9. Final Objective

Transform **NUSH2** into a visually sophisticated, professional and interactive engineering simulation platform that combines:

- **React Flow** for interactive process flowsheets and equipment connectivity.
- **shadcn/ui** for professional dashboards, engineering controls and application structure.
- **Aceternity UI** for futuristic backgrounds, subtle animations and premium visual effects.

The resulting interface should resemble a modern industrial digital-twin platform while retaining the technical credibility and functionality of rigorous process simulation software.

The application should allow users to explore the integrated nuclear-assisted hydrogen production and carbon capture process, modify operating parameters, execute simulations, inspect equipment performance and interpret calculated engineering results through a polished, intuitive interface.

**The ultimate goal is to make NUSH2 visually impressive, scientifically rigorous, technically reliable and suitable for academic demonstrations, engineering research presentations, competitions and future development.**

Begin by inspecting the NUSH2 repository and evaluating the three proposed libraries. Then recommend the most appropriate integration strategy before proceeding with implementation.
