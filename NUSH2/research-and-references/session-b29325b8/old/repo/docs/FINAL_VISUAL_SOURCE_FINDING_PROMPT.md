# Final Visual Source-Finding Pass — Base Images, Templates and Edit Plan

## Purpose

This instruction defines the visual-source step that follows the post-E8 calculation/notation pass.

The user will perform the final AI image generation/editing externally.

Main Research should **not generate polished AI artwork itself**.

Instead, identify the best existing base/reference image or template for each conceptual visual, provide the source link/provenance, and specify exactly what the user's external AI tool should later edit.

Quantitative scientific plots remain deterministic Rust/TikZ/data graphics.

## General rule

Prefer:

**real/authoritative base image or existing user artwork
→ scientifically constrained edit specification
→ user external AI editing
→ upload to attachments/
→ scientific validation
→ manuscript integration.**

Do not use AI to hallucinate geography, nuclear-site approval, quantitative risk contours or exact engineering geometry.

For every external base image check:
- source;
- direct page/link;
- what it actually depicts;
- whether it is current;
- attribution/copyright/reuse considerations;
- whether it is suitable for editing or only for visual reference.

Where reuse/editing rights are unclear, use the image as a reference and ask the user to create an original derivative composition rather than copying it directly into the report.

## Visual 1 — Existing nuclear/process artwork

First inspect the user's existing repository artwork:

- attachments/figure1_conventional_smr_ccs.png
- attachments/figure2_htgr_reference_power.png
- attachments/figure3_proposed_htgr_smr_ccs.png

The user prefers these visually.

Do NOT search for a replacement nuclear image merely because another image exists online.

Determine whether Figure 3 can serve as the base for the final preferred architecture.

If Figure 3 depicts one chemical train, prepare an edit specification that preserves its visual style while modifying it to show:

**1 x 600 MWth GTHTR300C-class reactor
→ primary helium
→ IHX
→ secondary helium
→ two parallel 130 MMSCFD SMR-H2+CCS trains
→ 176.8 MWth/train
→ 353.6 MWth total process heat.**

The edited visual must not imply that this exact two-train plant is a published JAEA/INL design.

Label it:
**Project-proposed mature two-train architecture — conceptual / not to scale.**

No electricity output/revenue should be invented.

### Required output

State:
- whether existing Figure 3 is a suitable editing base;
- what must be duplicated/changed;
- what labels must remain;
- what labels must be removed/updated;
- whether any additional source/reference image is actually needed.

## Visual 2 — Jurong Island conceptual integration

This should begin from a **real Jurong Island image/map/plan/context image**, not AI-generated geography.

Search current authoritative sources first:
- JTC Jurong Island;
- Singapore Government;
- A*STAR / ISCE2 for LCT3;
- PUB/EMA/MPA where useful;
- official maps/aerial/context imagery where available and suitable.

Find 2–4 candidate base/reference images.

Prefer an image that helps the user visually communicate:
- Jurong Island industrial context;
- coastline/water;
- industrial areas;
- port/logistics;
- enough spatial context for a conceptual overlay.

Do NOT select an image because it appears to show an available nuclear parcel.

The eventual edit should conceptually overlay:
- candidate nuclear island;
- two SMR-H2+CCS trains;
- visible nuclear/chemical separation;
- cooling-water intake/outfall OR clearly labelled alternative cooling concept;
- NG connection;
- H2 product/offtake;
- CO2 conditioning/export;
- port/shipping interface where relevant;
- security/site boundary concept.

Required warning on final visual:
**Conceptual candidate-context illustration — not a proposed or approved nuclear site; not to scale.**

LCT3 may be shown as contextual low-carbon/testbed activity only if the underlying image/source supports it.

Never depict LCT3 as the reactor site.

### Required output

For each candidate Jurong base/reference provide:
- descriptive title;
- authoritative source;
- direct source-page URL;
- image URL if reliably available;
- what it shows;
- why it is useful;
- editing/reuse caution;
- recommended/not recommended.

Then select one preferred base/reference for the user.

## Visual 3 — Two-train architecture

First preference:
**edit the user's existing Figure 3.**

Only search for another base/template if Figure 3 cannot communicate the two-train architecture cleanly.

If another reference is useful, search authoritative nuclear/process-heat sources for layout inspiration:
- JAEA GTHTR300C;
- IAEA HTGR/process-heat diagrams;
- INL nuclear hydrogen/process-heat schematics.

Use these as engineering references, not as artwork to copy blindly.

The final user-edited visual should make the topology immediately understandable:

REACTOR
→ PRIMARY He
→ IHX
→ SECONDARY He HEADER
→ TRAIN A REFORMER
→ WGS
→ CO2 CAPTURE
→ PSA
→ H2

and in parallel:

SECONDARY He HEADER
→ TRAIN B REFORMER
→ WGS
→ CO2 CAPTURE
→ PSA
→ H2.

Show:
- 176.8 MWth/train;
- 353.6 MWth total;
- two x 130 MMSCFD H2;
- total 260 MMSCFD H2 if useful;
- approximately 195,892 tH2/y if useful.

Avoid overcrowding.

Quantitative labels must come from canonical E2B data.

## Visual 4 — Implementation roadmap

The user wants a visually attractive roadmap rather than a plain generated technical diagram.

Search for **clean editable roadmap/process-stage visual templates or reference layouts** that could be supplied to the user's external AI/design tool.

Search reputable template/design sources such as:
- Canva;
- Microsoft Create/PowerPoint templates;
- Adobe Express;
- other reputable template libraries where access/reuse terms are clear.

Do not choose a template because of decorative imagery alone.

The template must support approximately nine stages:

1. Current research
2. Component/integration qualification
3. FOAK process-heat demonstration
4. Two-train 353.6 MWth demonstration
5. Replication / early-commercial BOAK
6. Mature / 10-OAK cost gate
7. Singapore site + licensing + contract closure
8. Construction / commissioning
9. Commercial operation / measured verification

It should also allow STOP/REDESIGN gates or side branches without becoming unreadable.

Prefer:
- horizontal staged roadmap;
- clear milestone nodes;
- room for short sublabels;
- professional engineering/research style;
- editable visual hierarchy.

Avoid:
- calendar/year timelines, because E7 deliberately does not invent dates;
- startup/marketing graphics that cannot accommodate decision gates;
- templates with mandatory dates.

### Required output

Provide 3–5 candidate roadmap templates/reference layouts with:
- template name;
- source;
- direct link;
- screenshot/preview link if available;
- why suitable;
- limitations;
- recommended choice.

The user will provide the chosen template/image to an external AI/design tool for editing.

## Visual 5 — Integrated safety/barrier figure

First determine whether this should be:
- external AI-edited conceptual artwork; or
- deterministic vector diagram.

Quantitative event trees, risk contours, source-term chains and probabilities MUST remain deterministic technical graphics.

A conceptual safety overview may use external AI/design editing if it shows:
- reactor/core/TRISO;
- primary helium;
- IHX;
- secondary helium;
- physical separation;
- two chemical trains;
- barrier layers;
- nuclear→chemical propagation;
- chemical→nuclear propagation;
- independent safety cooling/power;
- safe-state concept.

Do not search for an image implying this exact plant has been licensed or demonstrated.

If the existing user Figure 3 can be edited into a safety overlay, prefer that for stylistic consistency.

## Visual 6 — Quantitative economics/abatement

Do NOT source web artwork.

These should remain deterministic.

Required quantitative visuals include, if manuscript space warrants:
- FOAK S$137.74/tCO2e;
- BOAK S$74.14/tCO2e;
- mature 10-OAK S$42.84/tCO2e;
- S$100/t threshold;
- mature ~1.834 MtCO2e/y;
- 0.25 Mt/y threshold;
- comparator ~0.461 Mt/y with explicit boundary qualification.

Use Rust/TikZ/data-generated graphics.

## Required durable output

Create:

`results/FINAL_VISUAL_SOURCE_PLAN.md`

For each visual include:

VISUAL:
PURPOSE:
MANUSCRIPT LOCATION:
BASE IMAGE/TEMPLATE NEEDED?:
EXISTING USER IMAGE SUFFICIENT?:
PREFERRED BASE/REFERENCE:
SOURCE:
DIRECT LINK:
REUSE/ATTRIBUTION NOTE:
WHAT USER'S AI SHOULD EDIT:
REQUIRED SCIENTIFIC LABELS:
PROHIBITED/MISLEADING CONTENT:
FINAL CAPTION INTENT:
RECOMMENDATION:

Also create a compact link table at the top so the user can quickly open all recommended sources/templates.

## Important sequencing

This source-finding pass may be performed after the post-E8 calculation/notation compliance pass.

Do NOT integrate new external artwork yet.

Do NOT begin independent review.

The user/coordinator will inspect the source plan, choose base images/templates, generate/edit visuals externally, upload them to `attachments/`, and then Main Research will receive a separate bounded integration/validation instruction.

## Report and STOP

Report:

FINAL VISUAL SOURCE-FINDING PASS:

EXISTING FIGURE 3 SUITABLE FOR TWO-TRAIN EDIT?: YES/NO

JURONG BASE/REFERENCE OPTIONS FOUND:

PREFERRED JURONG SOURCE/LINK:

ROADMAP TEMPLATE OPTIONS FOUND:

PREFERRED ROADMAP TEMPLATE/LINK:

SAFETY VISUAL BASE RECOMMENDATION:

QUANTITATIVE FIGURES REMAIN DETERMINISTIC?: YES/NO

FINAL_VISUAL_SOURCE_PLAN CREATED:

COPYRIGHT/REUSE CAUTIONS:

COMMIT SHA:

Then STOP.

Do not integrate external visuals.
Do not begin independent review.
