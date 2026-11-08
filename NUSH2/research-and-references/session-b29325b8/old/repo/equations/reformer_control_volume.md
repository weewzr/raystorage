# Reformer control-volume resolution

## Source mapping

The authoritative IEAGHG base-case heat/material-balance table identifies:

- stream 4: **Purified Feedstock to Pre-reformer**, 500 C, 3.39 MPa, 5514.0 kmol/h;
- stream 5: **HTS Reactor Inlet**, 320 C, 2.80 MPa, 8370.3 kmol/h.

The process description states that the base case contains feed pretreatment, pre-reformer, primary reformer, syngas heat recovery, HTS and PSA. It also describes additional HP-superheated-steam / BFW conditioning between pre-reforming and primary reforming. These internal additions are not separately numbered in the summary stream table.

Primary source:
IEAGHG 2017-02, *Techno-Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS*.

## Elemental reconciliation

Using the rounded stream-4 and stream-5 compositions directly:

C4 = 1578.658 kmol-C/h
C5 = 1578.639 kmol-C/h

so

Delta C = -0.020 kmol-C/h,

which is effectively closed at the precision of the published table.

Hydrogen atoms increase by about:

Delta H = +310.34 kmol-H-atoms/h,

equivalent to:

Delta n_H2O,H = Delta H / 2 = 155.17 kmol-H2O/h.

Oxygen atoms increase by:

Delta O = +153.70 kmol-O-atoms/h,

equivalent to:

Delta n_H2O,O = 153.70 kmol-H2O/h.

The two independent estimates differ by only about 1.47 kmol/h. A least-squares reconciliation gives approximately:

n_H2O,interstage = 154.4 kmol/h.

This is a **derived reconstruction from rounded source rows plus the source-described steam/water addition**, not a directly published stream flow.

Including this water addition leaves residuals well below 0.1% of the source elemental inventories.

## Consequence

The stream-4 -> stream-5 material boundary is now sufficiently resolved for a reduced overall reformer material balance. It should not be interpreted as a single reactor: it encloses pre-reforming, interstage steam/water conditioning, primary reforming and syngas heat recovery before the HTS inlet.

The next energy model must therefore preserve at least two temperature grades:
- pre-reformer/feed-conditioning duty near 500 C;
- primary-reformer high-temperature duty, with reformer product normally around 900-950 C in the IEAGHG technical description.

## Energy benchmark already established

The IEAGHG base case reports:
- NG feedstock: 12.197 GJ/1000 Nm3 H2;
- additional NG fuel: 2.014 GJ/1000 Nm3 H2;
- total NG input: 14.212 GJ/1000 Nm3 H2;
- at 100,000 Nm3/h H2, total NG LHV input: 394.77 MW;
- H2 product LHV: 299.70 MW;
- gross cogeneration output: 11.5 MWe;
- net electricity export: 9.918 MWe.

These values constrain the future heat ledger but do not by themselves equal reformer radiant duty.

## Nuclear-integration warning

The 2.014 GJ/1000 Nm3 H2 separately supplied NG fuel corresponds to about 55.9 MW LHV at this scale. It is **not** the heat duty that an HTGR must supply, because:
- PSA tail gas supplies most reformer fuel;
- furnace efficiency is less than unity;
- the furnace convection section supplies feed preheat and steam duties;
- syngas/flue-gas heat recovery produces steam and power.

Therefore the HTGR substitution duty must be derived from the process heat ledger, not equated to purchased-fuel LHV.
