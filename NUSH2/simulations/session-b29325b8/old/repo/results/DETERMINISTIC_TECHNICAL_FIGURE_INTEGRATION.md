# Deterministic technical figures — integration and verification register

## Scope
Eight new two-train scientific figures are authored as deterministic TikZ macros in `paper/sections/figures_deterministic_two_train.tex` and invoked from the canonical LaTeX manuscript. No external image generation, conceptual-artwork replacement, Jurong parcel, EPZ radius, electricity output or risk contour was created. Existing Figures 1–3 are untouched.

| Figure / LaTeX label | Location | Canonical value and limitation |
|---|---|---|
| `fig:boundary-two-train` | design basis | Equal two-train 260 MMSCFD H2 service; NG baseline 105 vs candidate 68 MMSCFD; cost boundary is incomplete |
| `fig:temperature-cascade-two-train` | heat integration | INL source 925 -> 900 -> 871 C; 25/29 K are reported-location differences, not pinch |
| `fig:thermal-allocation-two-train` | energy performance | 600 MWth rating; ~370 MWth source process branch; 353.6 MWth assigned heat; residual is not cooling/decay heat |
| `fig:hydrogen-two-train` | hydrogen | 2 x 97,946 = 195,892 tH2/y; 2 x 176.8 = 353.6 MWth; 85% assumed availability |
| `fig:lifecycle-two-train` | carbon/lifecycle | 2 x per-train direct avoidance ~1.724 MtCO2e/y, upstream ~0.145, burdens ~0.035; net ~1.834; captured ~1.085 Mt/y is separate |
| `fig:two-train-economics-decomposition` | economics | mature baseline ~538.230 vs candidate ~616.814 S$m/y; candidate NG 348.568, CCS+T&S 37.637, nuclear 207.763, IHX 22.846; rounded displayed operands |
| `fig:two-train-maturity` | economics | FOAK 137.74 FAIL, BOAK 74.14 projected, 10-OAK 42.84 projected S$/tCO2e; S$100 academic screening criterion |
| `fig:two-train-uncertainty` | uncertainty | qualitative model-input dependence only; no invented probabilities or numerical tornado ranges |

## Provenance and numerical checks
Source inputs are the same per-train INL Case-6 values, JAEA 600/370 MWth architecture basis, and E2B Rust scenario outputs already cited and derived in the adjacent manuscript sections. The plots are reproducible from plain LaTeX/TikZ source and do not contain AI-generated imagery. For the lifecycle figure, two-train direct avoidance = 2(862,093.8)=1,724,187.6 tCO2e/y; upstream = 2(72,703.8)=145,407.6; new burdens = 2(3,649.2+450.5+13,559.0)=35,317.4; net ~1,834,278 tCO2e/y. For the mature cost graphic, candidate NG 348.568 + CCS/T&S 37.637 + nuclear 207.763 + interface 22.846 = 616.814 S$m/y. Values are rounded for legibility, while the Rust ledger remains controlling.

## Figure-specification compliance
These cover specification items 5–9, 11–13, and a nonduplicative thermal interpretation for item 10. Items 1–3 remain existing user art; items 4, 14 and 15 require separate conceptual/barrier/site/roadmap decisions and are not fabricated in this bounded quantitative-figure pass. The manuscript does not insert empty placeholders for those items. Figure numbering remains automatic.

## Verification boundary
LaTeX source is committed. The required Research CI and Paper/reproducibility CI must both succeed at final HEAD, and the resulting PDF must be visually inspected page by page. This register must not be used to claim CI or page QA success until checked.
