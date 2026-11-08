# PSA tail-gas and utility boundary under furnace removal

## 1. Conventional baseline

IEAGHG 2017-02 explicitly sends base-case PSA tail gas directly to the SMR burners.

Published PSA tail gas:
- total flow: 2106.3 kmol/h;
- H2: 23.69 mol%;
- CO2: 50.95%;
- CO: 14.54%;
- CH4: 9.45%;
- balance minor species.

Thus combustible H2+CO+CH4 flow is approximately:

~1004 kmol/h.

Using standard species LHVs:
- H2: 241.826 MJ/kmol;
- CO: 282.99 MJ/kmol;
- CH4: 802.30 MJ/kmol;

gives a reconstructed tail-gas chemical inventory of approximately:

**~101.95 MW_LHV**.

This is an internal fuel inventory, not purchased energy.

It explains why:
- radiant reformer duty ~=96 MW;
- purchased supplementary NG ~=56 MW_LHV;

can coexist without violating energy conservation.

## 2. Nuclear-heated consequence

When the fired furnace disappears, the ~101.95 MW_LHV tail gas loses its conventional sink.

It must NOT be credited as avoided fuel cost because the baseline did not purchase it.

Possible destinations are competing process configurations:

A. recycle untreated tail gas toward reformer feed;
B. remove CO2 first, then recycle combustible fraction;
C. recover additional H2 and/or CO2 through an added separation process;
D. retain partial combustion for lower-grade heat;
E. purge/oxidise residuals with capture, if justified.

Each changes carbon conversion, reformer feed, capture load and economics.

## 3. Contemporary external evidence

Ahn & Lee (2026) independently eliminate the fired furnace in both HTGR h-SMR and e-SMR and recycle PSA tail gas as process feed. They report total NG-consumption reductions of:
- 18.8% for h-SMR;
- 23.3% for e-SMR
relative to gray hydrogen.

This is strong evidence that tail-gas recycle is a realistic competing architecture.

However, the paper's NG-reduction percentages must not be applied directly to the IEAGHG baseline because its flowsheet, efficiencies and process conditions differ.

A 2025 Energy Conversion and Management study instead treats tail gas with cryogenic separation + two-stage PSA, showing that additional H2 recovery and >90% CO2 removal are another technically credible pathway.

Therefore "recycle all tail gas" is not predetermined.

## 4. Power balance consequence

IEAGHG fixed-output power balance:
- Base export: 9.918 MWe;
- Case 1A export: 1.492 MWe;
- Case 3 export: 0.426 MWe.

Case 1A's CCS penalty therefore includes a loss of:

8.426 MWe

of net export relative to base.

A nuclear-heated architecture that preserves/replaces steam generation may recover some cogeneration value, but the amount is not yet known.

Economic value must therefore be represented as:

V_power = Delta W_net * h * p_electricity,

with Delta W_net derived from the final heat/steam network, not assumed to equal the base 9.918 MWe.

## 5. Steam value consequence

Base-case export steam is source-resolved at approximately:

46.053 t/h, ~395 C, ~4.23 MPa.

Furnace removal eliminates its conventional steam-superheater service but not necessarily the export itself if nuclear heat and retained syngas/WGS recovery reproduce the steam balance.

Therefore steam export is currently a **preservation constraint**, not a guaranteed savings credit.

Economic value is parameterised as:

V_steam = m_export * h * p_steam.

No Singapore steam tariff is assumed.

## 6. Immediate economic implication

The largest verified real saving so far remains purchased supplementary NG displacement.

Tail gas is a process-resource opportunity, not a direct cash saving.

Power/steam changes can be economically important, but the sign and magnitude depend on final heat integration:
- losing export is a penalty;
- preserving export is neutral relative to baseline;
- increasing export is a credit.

The model now contains explicit annual electricity- and steam-value functions so prices can be introduced without hiding the physical balance.

## 7. Next model

The highest-information next step is a carbon/energy recycle screen for the ~101.95 MW_LHV PSA tail gas.

At minimum compare:
1. untreated recycle;
2. CO2 removal + combustible recycle;
3. enhanced H2/CO2 recovery.

For each case compute:
- recycle flow/composition;
- additional reformer feed carbon;
- H2 potential;
- reformer duty change;
- captured CO2;
- compression/separation energy;
- purchased NG displacement.

Only then can the nuclear architecture receive a defensible tail-gas economic credit.
