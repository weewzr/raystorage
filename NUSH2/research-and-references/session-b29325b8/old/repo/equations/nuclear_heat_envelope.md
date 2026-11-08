# First temperature-resolved nuclear heat envelope

## 1. Authoritative conventional high-temperature duty

IEAGHG 2017-02 lists the base-case **Steam Reformer Furnace radiant duty** as:

82.63 million kcal/h.

Using 1 kcal = 4.184 kJ:

Q_radiant = 82.63 * 4.184 / 3.6
          = 96.04 MW.

This is the first authoritative high-temperature process-duty benchmark for the reference 100,000 Nm3/h H2 plant.

The same 82.63 MMkcal/h radiant duty is listed for IEAGHG Case 1A and Case 3, showing that those CCS variants preserve the core reformer radiant duty while changing utility/capture loads.

## 2. Why 96 MW is not the complete furnace replacement duty

IEAGHG describes the fired reformer as two linked thermal sections:

### Radiant section
Heat transferred through catalyst-filled reformer tubes to drive the strongly endothermic reforming process.

### Convective section
Hot flue gas leaving the radiant section at roughly 800-900 C supplies:
- reformer feed preheat;
- pre-reformer feed preheat;
- steam superheat;
- feed preheat;
- steam generation.

Therefore removing combustion removes both:
1. the ~96 MW radiant heat source; and
2. the flue-gas heat source used by the convective coils.

A nuclear system must replace/reconfigure both sets of services.

## 3. Syngas heat recovery survives nuclear heating

The reformer product leaves at roughly 900-950 C and is cooled to 320 C in the reformer waste-heat boiler.

IEAGHG states that about 75% of the plant's HP saturated steam is generated in this syngas waste-heat boiler. Other steam generation comes from shift heat recovery and the furnace convective-section steam generator.

Thus nuclear replacement of the fired furnace does **not** remove the syngas waste-heat source, provided comparable reformer outlet conditions are retained.

This is favorable for the proposed concept, but the remaining flue-gas-derived steam/preheat services still need replacement.

## 4. Conventional energy consistency check

At the reference scale:

- total NG LHV input = 394.77 MW;
- separately purchased furnace NG LHV = 55.94 MW;
- reformer radiant duty = 96.04 MW.

The fact that radiant duty exceeds purchased furnace-NG LHV is physically consistent with the independently verified fact that PSA tail gas is the main conventional furnace fuel.

Therefore:

Q_HTGR != purchased NG fuel LHV.

The direct high-temperature nuclear substitution has a **minimum source-backed process-duty anchor of ~96 MW at the reference plant scale**, before accounting for how convective-section services are replaced.

It is called an anchor rather than a final HTGR requirement because heat-exchanger approach temperatures, secondary-loop losses, helium pumping and retained/reconfigured heat recovery have not yet been modelled.

## 5. Reactor-side temperature constraint

IEAGHG states:
- reformer product normally leaves at about 900-950 C;
- radiant-section flue gas exits toward the convection section at about 800-900 C.

JAEA demonstrates that HTGR process heat is directly relevant to this regime:
- HTTR reactor outlet helium: 950 C;
- primary-to-secondary heat transfer occurs through an intermediate heat exchanger;
- HTTR steam-reforming design supplies 10 MW nuclear heat at 950 C to an IHX;
- secondary helium supplies a steam reformer, steam superheater and steam generator;
- a mock-up reformer used helium heated to 880 C at the reformer inlet.

This JAEA architecture is notably close to the student's original conceptual configuration.

However, it also shows why reactor outlet temperature alone is insufficient. The process receives **secondary** helium after the IHX, so temperature drop and heat-exchanger pinch must be modelled.

## 6. Immediate scale observation

The IEAGHG reformer radiant duty is approximately:

96.04 MW / (100,000 Nm3 H2/h)
= 0.960 kW per (Nm3 H2/h capacity).

JAEA's documented HTTR steam-reforming system design transfers 10 MW of nuclear heat at the IHX scale. A 96 MW radiant-duty reference plant is therefore roughly an order of magnitude larger in high-temperature heat service than that 10 MW demonstration/design heat-transfer scale.

This does **not** imply infeasibility: commercial HTGR concepts can be larger. It means the project must distinguish demonstration precedent from commercial scale.

## 7. Current heat-service vector

The nuclear comparison will use:

Q_service = {
  Q_high-T,radiant,
  Q_convective,preheat,
  Q_steam-generation,
  Q_steam-superheat,
  Q_CCS-regeneration,
  W_electric,
  W_helium-circulation
}.

Currently source-resolved:
- Q_high-T,radiant = 96.04 MW for the IEAGHG reference plant.
- syngas WHB retains a major steam-recovery role; ~75% of saturated steam generation is attributed to it by IEAGHG.

Still unresolved:
- individual convective-coil duties;
- total HP steam production/export on an unambiguous basis;
- nuclear IHX/secondary-loop temperature losses;
- helium pumping;
- CCS regeneration duty for the final capture topology.

## 8. Falsification conditions

Direct nuclear heat becomes unattractive if, under consistent assumptions:
- secondary helium cannot maintain the required process temperature approach;
- replacing the furnace destroys enough useful convective heat recovery to erase the direct-heat efficiency advantage;
- tail-gas treatment consumes comparable fuel/electricity elsewhere;
- nuclear-electric eSMR supplies the same services at lower cost/emissions/complexity;
- the required high-temperature reactor capacity becomes incompatible with the Singapore deployment scenario.

No such conclusion is yet drawn.
