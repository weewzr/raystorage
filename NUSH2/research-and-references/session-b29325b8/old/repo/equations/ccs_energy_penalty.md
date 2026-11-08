# CCS energy penalty and nuclear-integration implications

## Authoritative comparison

IEAGHG's comparison of the same 100,000 Nm3/h H2 plant gives:

| Case | Capture location | NG input (MWth LHV) | H2 product (MWth LHV) | Net power export (MWe) | CO2 (kg/Nm3 H2) |
|---|---|---:|---:|---:|---:|
| Base | none | 394.77 | 299.70 | 9.918 | 0.8091 |
| 1A | shifted syngas, MDEA | 407.68 | 299.70 | 1.492 | 0.3704 |
| 3 | reformer flue gas, MEA | 433.72 | 299.70 | 0.426 | 0.0888 |

Source: IEAGHG 2017-02 / IEAGHG ReCAP comparison.

## Incremental energy burden

Relative to the no-capture base case:

Case 1A:
- additional NG LHV = 12.91 MW;
- lost net electricity export = 8.426 MWe.

Case 3:
- additional NG LHV = 38.95 MW;
- lost net electricity export = 9.492 MWe.

These quantities are kept separate because 1 MWth fuel and 1 MWe electricity are not thermodynamically equivalent. A later exergy/primary-energy comparison must state its conversion convention.

## Plant-gate abatement at fixed H2 output

At 100,000 Nm3/h H2:

Case 1A avoids:

(0.8091 - 0.3704)*100000/1000
= 43.87 tCO2/h.

Case 3 avoids:

(0.8091 - 0.0888)*100000/1000
= 72.03 tCO2/h.

Therefore flue-gas capture produces substantially greater plant-gate abatement but at substantially greater NG input.

This reproduces the qualitative IEAGHG conclusion: capture location creates a real energy-vs-capture tradeoff.

## Consequence for nuclear-assisted SMR

The nuclear design has at least two conceptually distinct opportunities:

1. replace high-temperature fired reformer duty;
2. supply lower-temperature CCS regeneration/steam/electricity services.

These should not be collapsed into a single nuclear-heat number.

A direct HTGR-heated reformer may remove/reduce the conventional flue-gas source, which changes the appropriate capture topology. Conversely, if shifted-syngas capture is retained, nuclear heat may primarily displace furnace fuel and utility energy rather than changing the process-CO2 separation point.

The comparison therefore needs a temperature-resolved service vector:

Q = {Q_high-T reforming, Q_medium-T preheat/steam, Q_low-T solvent regeneration, W_electric}.

## Current falsification criterion

The proposed direct-heat architecture is not supported merely if it reduces NG fuel.

It must show, relative to a consistent comparator:
- lower lifecycle CO2e;
- sufficient annual abatement;
- a credible destination for PSA tail gas;
- no hidden replacement of exported steam/electricity;
- a physically feasible temperature approach for every heat exchanger;
- incremental cost below the assignment threshold.

## Next energy task

Recover or derive the conventional plant's major heat duties:
- reformer radiant/process duty;
- convection/feed-preheat duty;
- steam generation and superheat;
- syngas heat recovery;
- furnace/flue losses;
- PSA-tail-gas chemical energy.

Then reconcile those services with the authoritative 394.77 MW NG ledger and 9.918 MWe export.
