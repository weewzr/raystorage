# Focused final figure integration QA

## Scope and starting evidence
At the starting HEAD, Research CI and Paper/reproducibility CI both passed. Three conceptual PNGs are present under attachments and included from active LaTeX sections. The GitHub file API exposed their filenames but did not return image pixels; the compiled PDF artifact could not be materialized into the available container for direct page-by-page inspection during this pass. **Do not treat this as a verified visual inspection.**

## Corrections made
1. **Roadmap:** the uploaded conceptual roadmap is known to have an extra numbered stage. Removed its inclusion from the active manuscript and replaced it with an exact nine-stage reproducible TikZ panel based on E7. This avoids displaying a known incorrect visual. Removed the competing superseded eight-step Singapore roadmap, replacing it with a cross-reference to the authoritative nine stages.
2. **Safety:** added a local explanatory paragraph identifying the reported 925/900/871 C source temperatures, primary helium -> IHX -> secondary helium -> reformer interfaces, and the difference between source states and qualified component margins. The PNG remains conceptual; no claim is made that embedded standards or numeric labels have been visually verified. Constrained inclusion to width <= textwidth and height <= 0.73 textheight, keeping aspect ratio.
3. **Jurong:** added a local warning that the map is hypothetical and cannot identify an available, proposed or approved nuclear site. Constrained the graphic to textwidth and 0.73 textheight with preserved aspect ratio.
4. **Surrounding Singapore text:** corrected multiple controlling-architecture mismatches: 423.2 MWth old one-train residual -> preferred conditional 246.4 MWth; 542 kt/y old captured throughput -> 1.085 Mt/y two-train throughput; 34 MMSCFD -> 68 MMSCFD two-train NG; S$3.725/t historical differential no longer controlling; FOAK S$137.74/t FAIL and future BOAK/10-OAK projections reinstated.
5. **Redundancy:** removed duplicated alternate roadmap enumeration and cross-referenced E7.

## Scientific qualification
The 925/900/871 C labels are source-model locations, not independently validated reactor/heat-exchanger/reformer metal temperatures. IHX separation is physical, not perfectly radiological; tritium and structural qualification remain unresolved. Any standards referenced inside the uploaded safety bitmap are **not certified** by this pass because its pixels were not available for inspection. The Jurong conceptual map cannot confer site approval. No nuclear EPZ radius, electricity export or risk contour was created.

## Remaining acceptance checks
- Obtain the final paper CI PDF artifact in a viewable form and inspect **every page**, especially safety, Jurong and roadmap pages, for readable labels, float placement, clipping and whitespace.
- Visually inspect the safety bitmap's exact temperatures, interface labels and any standards; correct the source artwork if needed rather than relying on a caption alone.
- Visually inspect Jurong map's exact parcel/approval wording; revise bitmap if it implies siting approval.
- Verify both final-HEAD CI workflows complete successfully.

This register records confirmed source-level corrections, not a completed PDF QA sign-off.
