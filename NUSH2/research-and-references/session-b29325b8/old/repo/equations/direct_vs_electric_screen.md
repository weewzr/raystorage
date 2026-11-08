# First common-service screen: direct HTGR heat vs HTGR-electric eSMR

## Purpose

This is the first apples-to-apples energy-conversion screen using the same currently bounded process-heat service.

It is not yet a techno-economic comparison and does not claim a preferred configuration.

## Common process-service basis

At the 100,000 Nm3/h H2 reference scale, the current bounded furnace-service envelope is approximately 131-151 MWth across the modelled reformer-preheat uncertainty.

For a 625 C reformer-inlet sensitivity, the current service interval is approximately:

132.5 to 149.5 MWth.

The upper side includes the deliberately conservative assumption that the entire non-syngas-WHB saturated-steam remainder is supplied by the furnace coil.

## Nuclear-electric pathway

For eSMR:

W_electric = Q_process / eta_electric-to-heat.

The 2025 eSMR review reports Joule heating as the most energy-efficient electrification route, with >90% power-to-heat efficiency at scale-up. A separate EU EReTech programme targets ~95% reactor energy efficiency.

A conservative first screen therefore uses:

eta_electric-to-heat = 0.90.

For Q_process = 132.5-149.5 MW:

W_eSMR ~= 147-166 MWe.

## HTGR power conversion

JAEA reports:
- baseline GTHTR300 net generation efficiency around 45.6-46%;
- upgraded 950 C GTHTR300 net efficiency up to 50.4%;
- approximately 50% generation efficiency as the practical high-temperature target.

Using the optimistic 50.4% benchmark:

Q_reactor,electric
= W_eSMR / 0.504
~= 292-330 MWth.

Using ~45.8% instead gives roughly:

~321-363 MWth.

These are screening values for the reformer/thermal-service electricity only. Other plant electrical loads must be added consistently later.

## Direct-heat pathway

For direct heat, the same 132.5-149.5 MWth process-service interval is supplied through:

reactor -> primary helium -> IHX -> secondary helium -> process.

The reactor thermal requirement is therefore:

Q_reactor,direct
= Q_process / eta_heat-path,

where eta_heat-path must include IHX/secondary-loop thermal losses.

That thermal efficiency has not yet been assigned a final value.

Electrical helium-circulator power is tracked separately; current pressure-drop sensitivity gives low-single-digit MWe.

## First thermodynamic inference

Because the nuclear-electric pathway converts reactor heat to electricity and then electricity back to process heat, its reactor-thermal requirement is necessarily larger than the process-heat requirement whenever:

eta_power * eta_electric-to-heat < eta_heat-path.

With eta_power ~0.46-0.50 and eta_electric-to-heat ~0.90-0.95, the combined electric pathway converts only roughly 41-48% of reactor thermal energy into delivered electric heat before other auxiliaries.

This creates a strong **reactor-thermal-utilisation advantage hypothesis** for direct heat.

It is not an overall winner claim because eSMR may offer:
- more compact/intensified reformer design;
- better controllability;
- different methane conversion/heat recovery;
- simpler separation between nuclear and chemical plant;
- different CAPEX and safety/licensing implications.

## Contradictory contemporary evidence

A 2026 International Journal of Hydrogen Energy paper directly compares HTGR-integrated helium-heated SMR and electrified SMR.

It reports:
- LCOH: $2.37/kg H2 for helium-heated SMR vs $2.99/kg for eSMR;
- net GWP: 3.5 vs 3.18 kgCO2/kgH2, respectively.

Thus the literature itself does not reduce the comparison to one universal winner: the helium-heated case is reported cheaper in that study while the electrified case has slightly lower GWP.

The paper also reports that both HTGR-integrated configurations eliminate the conventional fired furnace and recycle PSA tail gas as process feed, directly reinforcing this project's finding that tail-gas disposition must be redesigned when the furnace disappears.

Source:
Ahn & Lee, International Journal of Hydrogen Energy 247 (2026) 155890, DOI 10.1016/j.ijhydene.2026.155890.

## Next comparison layer

The next model should compare both architectures under this project's own consistent boundary:

1. same H2 production;
2. same feedstock/lifecycle methane assumptions;
3. same CCS target;
4. explicit PSA-tail-gas disposition;
5. same nuclear lifecycle assumptions;
6. direct path: IHX/loop heat loss + circulator electricity;
7. electric path: HTGR net power efficiency + electric-heater efficiency;
8. annual CO2e and abatement-cost metrics.

Only after that comparison should a preferred architecture be discussed.
