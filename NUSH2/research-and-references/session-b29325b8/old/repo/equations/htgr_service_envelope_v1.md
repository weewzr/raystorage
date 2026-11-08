# First uncertainty-bounded HTGR service envelope

## Status

This is the first combined thermal-service envelope assembled from the verified conventional SMR baseline.

It is **not yet a reactor rating**. It excludes the furnace-only saturated-steam-generation share, nuclear heat-exchanger/loop heat losses and several design-dependent details.

## Components

At 100,000 Nm3/h H2:

| Service | Value | Evidence status |
|---|---:|---|
| Primary-reformer radiant duty | 96.034 MW | IEAGHG equipment list, converted |
| HP steam superheat | 16.008-16.240 MW | source flows + IAPWS-consistent property bracket |
| NG + recycle-H2 feed preheat | 4.899 MW | source flows + NIST properties |
| Pre-reformer feed preheat | 7.050 MW | conservative lower-bound calculation |
| Reformer preheat @ 600 C | 6.750 MW | sensitivity, not source-reconstructed |
| Reformer preheat @ 625 C | 8.486 MW | sensitivity |
| Reformer preheat @ 650 C | 10.241 MW | sensitivity |

The pre-reformer value deliberately omits positive C2+ sensible terms above the currently encoded verified Cp range, and assumes already-superheated steam at the inlet. It is therefore a lower bound.

The reformer-preheater values use a 600-650 C contextual range from related IEAGHG work because the standalone source does not publish the exact intermediate state.

## Composite thermal service

Summing only the currently represented furnace-dependent services gives:

| Reformer inlet sensitivity | Current thermal-service envelope |
|---:|---:|
| 600 C | 130.74-130.97 MW |
| 625 C | 132.48-132.71 MW |
| 650 C | 134.23-134.46 MW |

These are calculated model outputs with CI regression tests.

They should be read as:

Q_HTGR,service >~ 131-134 MW

for the currently modelled services, depending on the reformer-preheat assumption.

They are still lower/incomplete relative to a final nuclear heat requirement because the furnace-only steam-generation contribution and nuclear-loop thermal losses remain unresolved.

## Secondary-helium electrical parasitic sensitivity

Using the existing screening loop assumptions:
- 96.034 MW radiant heat carried by secondary helium;
- 880 -> 650 C;
- cp=5.2 kJ/kg-K;
- mass flow ~80.30 kg/s;
- 5.15 MPa benchmark pressure;
- 80% circulator efficiency;
- total pressure drop = k_dp * 58 kPa;

gives:

| k_dp | DeltaP_loop | W_circulator |
|---:|---:|---:|
| 1 | 58 kPa | 2.17 MWe |
| 2 | 116 kPa | 4.33 MWe |
| 3 | 174 kPa | 6.50 MWe |

This is an electrical parasitic and is therefore kept separate from the MWth service total.

At the mid thermal-service scale (~132.5 MW), these correspond to roughly 1.6%, 3.3% and 4.9% of thermal service numerically. That ratio is not an exergy-equivalent efficiency and must not be used as one.

## Scientific interpretation

The initial idea began with an intuition that replacing purchased furnace fuel might require roughly the purchased-fuel energy scale. The verified service reconstruction now shows why that is insufficient:

- purchased make-up NG fuel: ~55.9 MW LHV;
- radiant process duty alone: ~96 MW;
- currently represented furnace-dependent services: ~131-134 MW;
- additional unresolved furnace steam-generation and nuclear-loop losses remain.

This does not falsify nuclear heating. It changes the sizing basis from fuel bookkeeping to delivered process services.

## Next missing term

The largest remaining conventional-plant ambiguity is the furnace-convection share of saturated HP steam generation. IEAGHG says ~75% is generated in the syngas WHB, with the remainder split between shift heat recovery and the furnace steam-generation coil.

The next step is to isolate or bound those latter two contributions. Once that is done, the conventional furnace-service ledger will be close enough to compare direct nuclear heat against nuclear-electric reforming on a common service basis.
