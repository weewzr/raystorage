# Reduced reformer/WGS reconstruction

## Purpose

This is an intermediate verification model connecting the validated IEAGHG PSA inlet to reforming and WGS reaction extents. It is intentionally simpler than the final natural-gas model.

The actual IEAGHG natural gas contains methane plus ethane, propane, butane, pentane and CO2. Here those carbon-bearing feed species are temporarily represented by an **equivalent methane flow containing the same number of carbon atoms**.

This preserves carbon exactly but does not preserve feed hydrogen exactly. The resulting H2 mismatch is therefore used as a diagnostic of the approximation rather than hidden by parameter fitting.

## Governing reactions

Steam methane reforming:

CH4 + H2O -> CO + 3 H2

Water-gas shift:

CO + H2O -> CO2 + H2

Let xi_R be the SMR extent and xi_W the WGS extent in kmol/h.

For equivalent methane feed F_C and PSA-inlet CH4, CO and CO2:

CH4_out = F_C - xi_R

CO_out = xi_R - xi_W

CO2_out = xi_W

H2_reaction = 3 xi_R + xi_W.

Using the previously verified source values:

F_C = 1578.56 kmol-C/h,

CH4_out ~= 199.2 kmol/h,

CO2_out ~= 1073.3 kmol/h.

Therefore:

xi_R ~= 1379.4 kmol/h

and

xi_W ~= 1073.3 kmol/h.

The predicted CO is then:

xi_R - xi_W ~= 306.1 kmol/h,

which reproduces the source PSA-inlet CO by construction-independent stoichiometry.

## Hydrogen diagnostic

The reduced model predicts:

H2 ~= 3 xi_R + xi_W ~= 5211 kmol/h,

whereas the reconstructed PSA inlet contains about 4953 kmol/h H2.

The few-percent mismatch is expected because carbon-equivalent methane substitution changes the feed H/C ratio and omits other details such as upstream CO2 and complete higher-hydrocarbon chemistry. It must not be tuned away.

The final species model will encode each hydrocarbon explicitly.

## Chemical steam consumption

For the two ideal reactions, net chemically consumed steam is:

n_H2O,chem = xi_R + xi_W.

This is not the required steam feed. Industrial SMR deliberately operates with excess steam; the final feed requirement needs the source steam-to-carbon ratio and the full water balance.

## First heat-duty lower layer

At 298.15 K with gaseous water, standard reaction enthalpies are approximately:

Delta H_R^0 = +206.1 kJ/mol

Delta H_W^0 = -41.2 kJ/mol.

The reduced reaction-only reference duty is:

Q_rxn,298 = xi_R Delta H_R^0 + xi_W Delta H_W^0,

which is of order 60-70 MW for this reference plant.

**This is not the reformer furnace duty.**

It excludes:
- heating reactants to reformer temperature;
- steam generation/vaporisation;
- excess steam;
- temperature dependence of reaction enthalpy;
- higher-hydrocarbon conversion/prereforming;
- furnace/heat-transfer losses;
- process heat recovery;
- air/flue-gas sensible heat;
- steam export.

It is retained as a thermochemical lower layer that the later full energy model must contain.

## Verification strategy

The final baseline must satisfy, without arbitrary fitting:
1. carbon closure;
2. hydrogen closure;
3. oxygen closure;
4. published PSA inlet composition;
5. approximately 90% PSA H2 recovery;
6. published NG feed/fuel rates;
7. published heat/material-balance duties within declared tolerances.

Only then will nuclear heat replace a defined set of duties.
