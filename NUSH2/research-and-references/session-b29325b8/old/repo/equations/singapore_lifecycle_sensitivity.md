# Singapore-oriented lifecycle sensitivity layer

## 1. Singapore natural-gas supply is not one pathway

EMA reports that Singapore imported 11 Mtoe of natural gas in 2024, of which 6 Mtoe was LNG. LNG therefore represented about 55% of natural-gas imports on that reported energy basis. Pipeline gas remained material.

Source: Singapore Energy Statistics, EMA, Energy Supply, 2024.

This means the model should not assign all Singapore gas either an LNG factor or a pipeline factor.

## 2. Upstream gas emissions

The IEA's 2026 LNG assessment estimates:
- global average extraction/processing/transport natural-gas supply: about 11.5 gCO2e/MJ;
- delivered LNG production-to-regasification: 18.6 gCO2e/MJ in 2025.

The IEA stresses that gas-supply emissions vary widely by geography and route, with LNG generally higher because of liquefaction and long-distance transport.

These are **global sensitivity anchors**, not Singapore-specific measured factors.

For the IEAGHG reference plant, feedstock NG energy is:

12.197 GJ / 1000 Nm3 H2
x 100,000 Nm3/h
= 1219.7 GJ/h.

At 8994 kg H2/h:

11.5 g/MJ -> approximately 1.56 kgCO2e/kgH2 upstream;
18.6 g/MJ -> approximately 2.52 kgCO2e/kgH2 upstream.

Therefore upstream gas alone is of the same order as, or larger than, the residual plant carbon in a 90% feedstock-carbon-capture case (~0.77 kgCO2/kgH2).

This is a major result: high point-source capture does not make the natural-gas pathway near-zero-carbon.

## 3. Methane uncertainty

IEA Global Methane Tracker reports roughly 1% global-average upstream methane intensity for oil/gas production in 2024, falling to 0.2% in its 2030 NZE scenario. Producer performance varies by orders of magnitude.

Therefore future scenarios should include:
- current/global-average supply;
- lower-emission/best-practice contracted gas;
- high-emission sensitivity.

Do not equate Singapore's import mix with the global average without source-route data.

## 4. Nuclear lifecycle sensitivity

UNECE reports nuclear-electricity lifecycle emissions of 5.1-6.4 gCO2e/kWh delivered electricity.

For nuclear-electric eSMR, this factor can be applied directly to the modelled electrical consumption.

For direct process heat, no equivalent authoritative gCO2e/kWh_th value has yet been identified.

A transparent proxy is therefore defined:

e_nuclear,heat-proxy [g/kWh_th]
= e_nuclear,electric [g/kWh_e] * eta_net,electric.

This allocates the same reactor lifecycle burden per unit reactor thermal throughput implied by the electricity LCA and power-cycle efficiency.

It is a **derived allocation proxy**, not a published nuclear-heat LCA result.

At 5.5 gCO2e/kWh_e and eta_e=0.504:

e_heat-proxy ~= 2.77 gCO2e/kWh_th.

For 140 MWth delivered service at 8994 kgH2/h, this is only ~0.043 kgCO2e/kgH2 before direct-loop losses.

For 140 MWth process service supplied electrically at 90% heater efficiency, the electrical load is ~155.6 MWe and the same 5.5 g/kWh_e factor gives ~0.095 kgCO2e/kgH2.

Thus nuclear lifecycle emissions are much smaller than the current upstream-NG term in this screening model.

## 5. Singapore CCS route

Singapore does not have suitable domestic geological storage and is pursuing cross-border CCS with regional partners. MTI confirms:
- Singapore-Indonesia CCS cooperation progressed from a 2024 LOI to a 2025 MOU;
- Singapore is also working with Malaysia and other regional partners;
- Singapore and Indonesia have discussed a ~2 MtCO2/y pathfinder scale.

No final storage site or transport route is assumed in this model.

## 6. CO2 transport sensitivity

IEAGHG ship-transport work shows that transport-chain emissions include:
- liquefaction electricity;
- ship fuel;
- ship/storage boil-off.

For a historical 50,000-t ship case supplied CO2 at 10 MPa, total transport emissions were about 2.5% of transported CO2 at 200 km and ~3.5% at 1000 km, increasing strongly with distance.

Those values are old and configuration-specific. They are retained only as a transport-emissions sensitivity anchor until a Singapore-to-specific-storage-site route is selected.

The Rust model therefore represents:

e_CCS,T = m_CO2,captured * f_transport

with f_transport explicit rather than hidden.

## 7. First lifecycle structure

For matched direct and electric cases:

e_LCA =
7.72(1-eta_C)
+ e_NG,upstream
+ e_nuclear
+ e_CCS,T&S
+ e_other.

At eta_C=90%:
- plant feedstock carbon ~= 0.77 kgCO2/kgH2;
- upstream NG global-average anchor ~= 1.56 kgCO2e/kgH2;
- upstream LNG anchor ~= 2.52 kgCO2e/kgH2;
- nuclear energy term is order ~0.04-0.10 kgCO2e/kgH2 under current proxy/electric assumptions.

Therefore the dominant unresolved lifecycle term is currently the **actual gas supply chain to Singapore**, not nuclear lifecycle emissions.

## 8. Falsification implication

If Singapore's future natural-gas supply has high methane/LNG-chain emissions, even very high plant-level CO2 capture may fail to produce sufficiently low lifecycle hydrogen.

This creates a direct competing hypothesis:

H_upstream:
The proposed nuclear-assisted SMR+CCS concept is lifecycle-limited by imported natural-gas emissions rather than by the reactor heat architecture.

That hypothesis is now quantitatively testable.
