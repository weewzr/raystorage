// Guided process tour of the Interactive PFD.
//
// A button above the drawing starts a walk through the plant in steps: fission
// heat in the reactor core, the primary and secondary helium loops, the
// reformer train, capture, the external PSA and the hydrogen product, the
// reject recycle, CO2 compression and storage, and the power block. Each step
// lights its units, moves dashes along its streams in the direction of flow
// and fills a caption card beside the drawing.
//
// Every number in a caption is read at render time from the live engine reply
// (state.lastOk) by its reply path, so it follows the controls. A limit the
// reply holds no number for is stated in words, or through the unit's
// limitation text of units.json, which the page fills from the same reply.
// This file formats numbers (decimals, and a fraction shown as a percentage)
// and computes no engineering quantity. Pictures and reaction lines drawn
// without engine data carry an Illustration tag.
//
// Motion is CSS animation only. Under prefers-reduced-motion the tour shows
// still steps, moves nothing and does not advance by itself.

import { esc, walk, fillText } from "./panels.js?v=ef21980aa4fef40961dee7fb4425c8109770b8c1";
import { flagText } from "./app.js?v=ef21980aa4fef40961dee7fb4425c8109770b8c1";

const MOTION = "(prefers-reduced-motion: reduce)";
const STAGGER_MS = 550;

// Medium of a stream line, from its st-* class on the drawing. The names
// follow the legend of the drawing.
const MEDIA = {
  ph: "Primary helium",
  sh: "Secondary helium",
  ng: "Natural gas",
  stm: "Steam and condensate",
  syn: "Syngas",
  mea: "MEA solvent",
  h2: "H₂-rich gas and H₂",
  co2: "Captured CO₂",
  rej: "PSA reject and recycle",
  vent: "N₂-rich stack gas",
};

// What a number is: read from the reply and moving with the controls, an echo
// of a control, or a fixed registry value.
const KIND = { live: "Live", input: "Input", fixed: "Fixed value" };

/** A number read from the live reply at `p`. */
const N = (p, u, d, k, l) => ({ p, u, d, k, l });

// Numbers several steps share.
const ROT = N("units.htgr.conditions.ROT.value", "C", 0, "fixed", "Reactor outlet");
const T_SUP = N("units.smr.conditions.T_HE_SUPPLY.value", "C", 0, "fixed", "Helium supply");
const RECYCLED = N("units.cryogenic.results.recycled_c.value", "kmol/h", 0, "live", "Carbon recycled to the reformer");
const Q_REB = N("kpi.q_reboiler_mw", "MW", 1, "live", "Reboiler duty");
const TAIL_HEAT = N("balances.energy.tail_gas_to_reboiler.value", "MW", 1, "live", "Tail gas heat to the reboiler");
const BRANCH = "units.power.inputs.branch.value";
const STEAM_GAP = "balances.energy.reactor_steam_to_reboiler.value";
const UNMET_HEAT = "balances.energy.unmet_thermal.value";

// What happens to the PSA reject. The drawing shows it recycled; when it is
// exported, the recycle lines and the stack carry nothing, and when it is
// burned on site, the reject reaches the burner, which the stack stands for,
// while the cryogenic lines carry nothing.
const exported = (c) => c.x.tail_gas === "export";
const burned = (c) => c.x.tail_gas === "burn_on_site";
const rejectLeaves = (c) => exported(c) || burned(c);
const selexol = (c) => c.x.tail_gas === "inl_selexol";
const recycles = (c) => c.q(RECYCLED.p) > 0;
const hasReboiler = (c) => c.q(Q_REB.p) > 0;

/** Lines of the closing step: every drawn line that an earlier step of the
 * same scenario moves, in drawing order. A line the steps leave dark in this
 * scenario stays dark at the close too, such as the recycle lines when the
 * reject is exported, or the MEA regeneration lines under Selexol. */
function closingLines(c) {
  const moved = new Set(STEPS.filter((s) => !s.all).flatMap((s) => streamIds(pick(s.flows, c))));
  return allStreams().filter((id) => moved.has(id));
}

// Heat duties above the rated reactor: the IHX and reformer figures are then
// what the plant would need, and the warning gives the engine's reason. With no
// field to read, an infeasible plant is treated the same way.
const heatShort = (c) => {
  const v = c.q(UNMET_HEAT);
  return v === null ? !c.feasible : v > 0;
};

// The steps. `units` are lit, `flows` are groups of streams animated in order
// (a group's lines move together), `ctx` and `ctxUnits` are lines and units
// kept half lit, `marks` are data-tour keys of labels and frame elements to
// keep lit, and `lead` is the unit whose panel opens beside the card. `ms` is
// the dwell when playing. Text placeholders {key} name entries of `nums`.
const STEPS = [
  {
    id: "fission",
    title: "Fission heat in the reactor core",
    ms: 8000,
    units: ["htgr"],
    marks: ["core"],
    flows: [],
    lead: "htgr",
    inset: "fission",
    feas: true,
    nums: {
      rot: ROT,
      rating: N("kpi.rating_mwth", "MWth", 0, "input", "Reactor rating"),
      output: N("balances.energy.htgr_heat_output.value", "MWth", 1, "live", "Reactor heat output"),
      need: N("kpi.reactor_required_mwth", "MWth", 1, "live", "Heat the process needs"),
      surplus: N("kpi.surplus_mwth", "MWth", 1, "live", "Margin at the rating"),
    },
    figs: ["rot", "rating", "output", "surplus"],
    text: (c) => [
      "Inside the core of the high-temperature gas-cooled reactor (HTGR), uranium nuclei in the tristructural isotropic (TRISO) fuel particles split when they absorb a neutron, and the graphite moderator slows the new neutrons so the chain reaction carries on.",
      c.feasible
        ? "The fission heat passes to helium flowing through the core, which leaves at {rot}. The reactor runs at {output} of its {rating} rating."
        : "The fission heat passes to helium flowing through the core, which leaves at {rot}. The plant needs {need} and the reactor is rated at {rating}.",
    ],
    notes: (c) => [c.limit("htgr", 0)],
  },
  {
    id: "primary",
    title: "Primary helium loop",
    ms: 7000,
    units: ["htgr", "ihx", "circ_primary"],
    marks: ["ph"],
    flows: [["P1"], ["P2"], ["P3"]],
    lead: "ihx",
    feas: true,
    nums: {
      rot: ROT,
      ihx: N("balances.energy.ihx_duty.value", "MW", 1, "live", "IHX duty"),
      he: N("units.circ_primary.inputs.he_prim.value", "kg/s", 1, "live", "Primary helium flow at the need"),
      e: N("units.circ_primary.results.e_prim.value", "MWe", 2, "live", "Primary circulator"),
      app: N("units.ihx.conditions.IHX_APPROACH_K.value", "K", 0, "input", "IHX approach"),
    },
    figs: ["ihx", "he", "e", "app"],
    text: (c) => [
      heatShort(c)
        ? "At the need, hot primary helium at {rot} would pass {ihx} to a separate secondary loop in the intermediate heat exchanger (IHX), which keeps the reactor coolant out of the chemical plant. The rated reactor cannot supply all of the plant's heat, as the warning above says."
        : "Hot primary helium at {rot} passes {ihx} to a separate secondary loop in the intermediate heat exchanger (IHX), so the reactor coolant never enters the chemical plant.",
      "The circulator sends the cooled helium back to the core.",
    ],
    notes: (c) => ["A branch of the hot helium also runs to the power block, which step 12 follows.", c.flag("ihx_approach_k")],
  },
  {
    id: "secondary",
    title: "Secondary helium loop to the reformer",
    ms: 7000,
    units: ["ihx", "smr"],
    marks: ["sh"],
    flows: [["H1"], ["H2"]],
    lead: "smr",
    feas: true,
    nums: {
      sup: T_SUP,
      ret: N("units.smr.conditions.T_HE_RETURN.value", "C", 0, "fixed", "Helium return"),
      q: N("kpi.q_process_mwth", "MWth", 1, "live", "Process heat, with feed preheat"),
      he: N("units.circ_primary.inputs.he_sec.value", "kg/s", 1, "live", "Secondary helium flow"),
      e: N("units.circ_primary.results.e_sec.value", "MWe", 2, "live", "Secondary circulator"),
    },
    figs: ["sup", "ret", "q", "he"],
    text: (c) => [
      heatShort(c)
        ? "The secondary helium leaves the IHX at {sup}, heats the reformer and its feed preheat, and returns at {ret}. At the need it would bring {q} of process heat, which a conventional reformer raises by burning natural gas."
        : "The secondary helium leaves the IHX at {sup}, heats the reformer and its feed preheat, and returns at {ret}. It brings {q} of process heat that a conventional reformer would raise by burning natural gas.",
      "A second circulator on the return leg draws {e}; the drawing does not show it.",
    ],
    notes: (c) => [
      "The helium connections on the reformer are drawn schematically. In the engine's tube model the hot helium enters at the process gas outlet end and flows against the process gas.",
      c.flag("T_HE_SUPPLY"),
    ],
  },
  {
    id: "feed",
    title: "Natural gas and steam feed the reformer",
    ms: 8000,
    units: ["smr"],
    marks: ["ng", "steam"],
    flows: [["S01", "S02"]],
    lead: "smr",
    eq: ["CH₄ + H₂O ⇌ CO + 3 H₂"],
    nums: {
      ng: N("kpi.ng_mmscfd", "MMSCFD", 1, "live", "Natural gas feed"),
      out: N("units.smr.conditions.SYNGAS_CHAIN_C5_HPS_GEN1_T_IN_C.value", "C", 0, "fixed", "Reformer exit, INL assumption"),
      sup: T_SUP,
    },
    figs: ["ng", "out", "sup"],
    text: (c) => [
      `Natural gas at {ng} and process steam enter the reformer${recycles(c) ? ", joined by the methane-rich recycle of step 9" : ""}. In the catalyst tubes methane and steam react to hydrogen and carbon monoxide, taking up the helium's heat.`,
      c.x.basis === "case_a"
        ? "The engine sets the exit at {out}, the equilibrium assumption of the Idaho National Laboratory (INL) study; the tube check that tests it ran on the Case B feed."
        : "The engine sets the exit at {out}, the equilibrium assumption of the Idaho National Laboratory (INL) study. With helium supplied at {sup}, the engine's tube model found no tube count in its search range that reaches it on the assumed tube, so the exit stays an assumption.",
    ],
  },
  {
    id: "shift",
    title: "Water-gas shift",
    ms: 7000,
    units: ["wgs"],
    marks: ["syn"],
    flows: [["S05"]],
    lead: "wgs",
    ctxUnits: ["smr"],
    eq: ["CO + H₂O ⇌ CO₂ + H₂"],
    nums: {
      hts: N("units.wgs.conditions.SYNGAS_CHAIN_C5_HPS_GEN1_T_OUT_C.value", "C", 0, "fixed", "High-temperature shift inlet"),
      lts: N("units.wgs.conditions.SYNGAS_CHAIN_C5_WGS_EX2_T_OUT_C.value", "C", 0, "fixed", "Low-temperature shift inlet"),
      q: N("units.wgs.results.wgs_heat_release.value", "MW", 1, "live", "Shift heat release"),
    },
    figs: ["hts", "lts", "q"],
    text: (c) => [
      "The syngas raises steam as it leaves the reformer and enters the high-temperature shift bed at {hts}.",
      c.q("units.wgs.results.wgs_heat_release.value") !== null
        ? "Over the shift catalyst, carbon monoxide and steam form more hydrogen and carbon dioxide, and the beds release {q} of heat."
        : "Over the shift catalyst, carbon monoxide and steam form more hydrogen and carbon dioxide.",
    ],
    notes: () => ["The drawing shows the two beds, and the coolers between them, as one vessel."],
  },
  {
    id: "cooling",
    title: "Cooling and condensate",
    ms: 7000,
    units: ["syngas_cooler", "separator"],
    marks: ["cond"],
    flows: [["S09"], ["S09B"], ["S10", "S11"]],
    lead: "separator",
    ctxUnits: ["wgs", "absorber"],
    nums: {
      ex4: N("units.syngas_cooler.results.ex4.value", "MW", 1, "live", "Syngas heat to the reboiler"),
      sep: N("units.separator.conditions.SYNGAS_CHAIN_C5_KO_DRM_1_T_OUT_C.value", "C", 0, "fixed", "Separator temperature"),
      cond: N("units.separator.results.condensate.value", "t/h", 0, "live", "Condensate"),
    },
    figs: (c) => (hasReboiler(c) ? ["ex4", "sep", "cond"] : ["sep", "cond"]),
    text: (c) => [
      hasReboiler(c)
        ? "The shifted gas is cooled in steps, and {ex4} of its heat is hot enough to help drive the reboiler of the monoethanolamine (MEA) capture plant in step 10."
        : "The shifted gas is cooled in steps on its way to capture.",
      c.q("units.separator.results.condensate.value") !== null
        ? "In the separator at {sep}, {cond} of water condenses out before the gas goes on to the absorber."
        : "In the separator at {sep} the condensed water drains out before the gas goes on to the absorber.",
    ],
    notes: () => ["The engine models six exchangers where the drawing shows one cooler."],
  },
  {
    id: "absorber",
    title: (c) => (selexol(c) ? "CO₂ capture by Selexol" : "CO₂ capture in the MEA absorber"),
    ms: 7000,
    units: ["absorber"],
    marks: ["lean", "rich"],
    flows: [["S11", "S18"], ["S17"]],
    lead: "absorber",
    ctxUnits: ["separator"],
    eq: (c) => (c.x.tail_gas === "inl_selexol" ? [] : ["CO₂ + 2 RNH₂ ⇌ RNHCOO⁻ + RNH₃⁺"]),
    nums: {
      w: N("inputs.w_mea", "wt%", 0, "input", "MEA strength"),
      p: N("units.absorber.conditions.P_ABSORBER.value", "bar", 1, "input", "Absorber pressure"),
      pp: N("kpi.capture_per_pass", "-", 1, "live", "Capture per pass"),
      cap: N("units.absorber.results.cap.value", "kmol/h", 0, "live", "CO₂ captured"),
      sol: N("kpi.solvent_t_h", "t/h", 0, "live", "Lean solvent"),
    },
    figs: (c) => (c.x.tail_gas === "inl_selexol" ? ["pp", "cap"] : ["pp", "cap", "sol", "p"]),
    text: (c) => (selexol(c)
      ? ["In this scenario a physical solvent, Selexol, takes up {pp} of the CO₂ in one pass, {cap}. The drawing still shows the monoethanolamine (MEA) plant, so its solvent lines stand in for the Selexol unit."]
      : [
        "In the packed absorber the gas rises against falling {w} aqueous monoethanolamine (MEA) at {p}.",
        "The amine takes up {pp} of the CO₂ in one pass and leaves at the bottom as rich solvent; the hydrogen-rich gas leaves at the top.",
      ]),
    notes: (c) => ["w_mea", "p_abs_bar", "lean_loading", "capture_target"].map(c.flag),
  },
  {
    id: "psa",
    title: "Hydrogen product from the external PSA",
    ms: 7000,
    units: ["psa"],
    marks: ["boundary", "h2rich", "h2prod"],
    flows: [["S12"], ["S13"]],
    lead: "psa",
    ctxUnits: ["absorber"],
    nums: {
      feed: N("kpi.h2_rich_kg_h", "kg/h", 0, "live", "H₂ into the PSA"),
      rec: N("units.psa.conditions.PSA_H2_RECOVERY.value", "-", 1, "fixed", "PSA recovery, assumed"),
      prod: N("units.psa.results.h2_product.value", "kg/h", 0, "live", "Purified H₂"),
      ty: N("kpi.h2_product_t_y", "t/y", 0, "live", "Purified H₂ per year"),
      av: N("inputs.availability", "-", 1, "input", "Availability"),
    },
    figs: ["feed", "prod", "ty", "rec"],
    text: () => [
      "The hydrogen-rich gas, with {feed} of hydrogen, crosses the dashed boundary to pressure swing adsorption (PSA), an external block the engine does not simulate.",
      "At an assumed {rec} recovery it delivers {prod} of purified hydrogen, {ty} at {av} availability.",
    ],
    notes: () => ["The two PSA beds are drawn schematically, since the block is external and has no model."],
  },
  {
    id: "reject",
    title: (c) => (exported(c) ? "PSA reject exported" : burned(c) ? "PSA reject burned on site" : "PSA reject back to the cryogenic unit"),
    ms: 8000,
    units: (c) => (rejectLeaves(c) ? ["psa"] : ["recycle_compressor", "cryogenic"]),
    marks: (c) => (exported(c) ? ["boundary"] : burned(c) ? ["stack", "rej"] : ["stack", "rej", "recycle", "h2ret"]),
    flows: (c) => (exported(c) ? [] : burned(c) ? [["S14"], ["S16"]] : [
      ["S14"],
      ["R1"],
      ["S21", "S03", "S15",
        { m: "h2", d: "M1345,190 L1345,150 L1480,150 L1480,270 L1700,270 L1700,226" },
        { m: "rej", d: "M700,235 L700,330" }],
      ["S16"],
    ]),
    ctx: (c) => (rejectLeaves(c) ? [] : ["S12", "S01"]),
    lead: (c) => (rejectLeaves(c) ? "psa" : "cryogenic"),
    ctxUnits: (c) => (rejectLeaves(c) ? [] : ["psa", "smr"]),
    nums: {
      h2: N("units.psa.results.h2_reject.value", "kg/h", 0, "live", "H₂ in the reject"),
      tc: N("units.cryogenic.inputs.tailgas_c.value", "kmol/h", 0, "live", "Carbon in the reject"),
      rc: RECYCLED,
      bc: N("units.cryogenic.results.burned_c.value", "kmol/h", 0, "live", "Carbon burned"),
      atm: N("kpi.co2_atm_t_y", "t/y", 0, "live", "CO₂ to the atmosphere"),
      tail: TAIL_HEAT,
    },
    figs: ["tc", "rc", "bc", "atm"],
    text: (c) => {
      if (exported(c)) {
        return [
          "In this scenario the PSA reject, holding {h2} of hydrogen, is exported across the boundary, so the drawn recycle and stack carry nothing.",
          "The engine counts {atm} of CO₂ to the atmosphere.",
        ];
      }
      if (burned(c)) {
        return [
          "In this scenario the PSA reject, holding {h2} of hydrogen, crosses back into the boundary and is burned on site, which gives the reboiler {tail}; the drawn cryogenic unit and recycle carry nothing.",
          "The flue gas leaves by the stack, and the engine counts {atm} of CO₂ to the atmosphere.",
        ];
      }
      return [
        "The PSA reject, still holding {h2} of hydrogen, crosses back into the boundary and is compressed for the cryogenic unit.",
        "The cold separation returns hydrogen to the PSA feed and a methane-rich recycle to the reformer. The nitrogen-rich cut is burned and leaves by the stack as the plant's residual {atm} of CO₂.",
      ];
    },
    notes: (c) => {
      if (exported(c)) return ["No line on the drawing shows the export."];
      if (burned(c)) return ["The burner is not drawn. Here the reject goes to it in place of the recycle compressor and the cryogenic unit, and the stack stands for its flue gas."];
      return [
        // INL drew the unit for the small CO₂ slip of Selexol, so this limit is MEA's.
        selexol(c) ? "" : c.limit("cryogenic", 1),
        "The burner of the nitrogen-rich cut is not drawn; the stack stands for it.",
      ];
    },
  },
  {
    id: "stripper",
    title: "Stripper and reboiler heat",
    ms: 8000,
    units: ["rich_lean_hx", "stripper", "reboiler", "lean_cooler", "lean_pump"],
    marks: ["rich"],
    flows: (c) => (hasReboiler(c) ? [["S17"], ["L3"], ["L5", "L6"], ["L4"], ["S18"]] : []),
    lead: "reboiler",
    ctxUnits: ["absorber"],
    nums: {
      ps: N("units.stripper.conditions.P_STRIPPER.value", "bar", 1, "input", "Stripper pressure"),
      q: Q_REB,
      srd: N("units.reboiler.conditions.srd.value", "GJ/t", 1, "input", "Specific reboiler duty"),
      sel: N("balances.energy.syngas_recovery_selexol.value", "MW", 1, "live", "Freed Selexol steam"),
      exp: N("balances.energy.syngas_recovery_export.value", "MW", 1, "live", "Export steam"),
      ex4: N("balances.energy.syngas_recovery_ex4.value", "MW", 1, "live", "Syngas heat"),
      tail: TAIL_HEAT,
      gap: N("balances.energy.reactor_steam_to_reboiler.value", "MW", 1, "live", "Reactor steam to the reboiler"),
    },
    figs: (c) => (hasReboiler(c) ? ["q", "srd", "gap", "ps"] : ["q"]),
    text: (c) => {
      if (!hasReboiler(c)) {
        return ["In this scenario Selexol takes the place of the MEA plant, and the reboiler duty is {q}."];
      }
      const tail = c.q(TAIL_HEAT.p) > 0 ? ", tail gas burned on site {tail}" : "";
      return [
        "Rich solvent is warmed in the rich/lean exchanger and stripped at {ps} by reboiler steam, which drives the CO₂ out; the lean solvent is cooled and pumped back to the absorber.",
        `The reboiler needs {q}, met in a fixed order: freed Selexol steam {sel}, export steam {exp}, syngas heat {ex4}${tail}, and reactor steam {gap} last.`,
      ];
    },
    notes: (c) => (hasReboiler(c) ? ["No line on the drawing brings heat into the reboiler; the order comes from the engine's energy balance."] : []),
  },
  {
    id: "co2",
    title: "CO₂ compression, ship and offshore storage",
    ms: 8000,
    units: (c) => (selexol(c) ? ["co2_compressor", "co2_storage"] : ["condenser", "co2_compressor", "co2_storage"]),
    marks: ["co2"],
    flows: (c) => (selexol(c) ? [["S19"], ["S20"]] : [["C1"], ["S19"], ["S20"]]),
    lead: "co2_storage",
    ctxUnits: (c) => (selexol(c) ? [] : ["stripper"]),
    inset: "ship",
    nums: {
      e: N("units.co2_compressor.results.e_co2.value", "MWe", 2, "live", "Compression"),
      cap: N("kpi.co2_captured_t_y", "t/y", 0, "live", "CO₂ shipped to storage"),
      ts: N("inputs.ts_s_t", "S$/t", 0, "input", "Transport and storage tariff"),
      cost: N("units.co2_storage.results.ts_cost.value", "S$M/y", 1, "live", "Transport and storage charge"),
    },
    figs: ["cap", "e", "ts", "cost"],
    text: (c) => [
      selexol(c)
        ? "The CO₂ from the Selexol unit, drawn as the line from the condenser, is compressed to INL's pipeline condition using {e}. The model does not include conditioning the CO₂ for the ship; the pipeline compression load stands in for it."
        : "The condenser returns water from the stripper overhead as reflux, and the CO₂ is compressed to INL's pipeline condition using {e}. The model does not include conditioning the CO₂ for the ship; the pipeline compression load stands in for it.",
      "Then {cap} of CO₂ leaves by ship for storage offshore abroad, which the model treats as a tariff of {ts} with no simulation.",
    ],
    notes: (c) => [c.limit("co2_storage", 1)],
  },
  {
    id: "power",
    title: "Power block and site electricity",
    ms: 7000,
    units: ["power", "circ_primary", "htgr"],
    marks: [],
    flows: (c) => (c.q(BRANCH) > 0 ? [["P1"], ["P4"]] : []),
    lead: "power",
    feas: true,
    nums: {
      br: N(BRANCH, "MWth", 1, "live", "Helium branch to the power block"),
      brn: N(BRANCH, "MWth", 1, "live", "Helium branch at the need"),
      en: N("units.power.outputs.e_nuc.value", "MWe", 2, "live", "Turbine output at the need"),
      gen: N("balances.energy.electricity_generation.value", "MWe", 2, "live", "Electricity generated"),
      eta: N("inputs.eta_power", "-", 1, "input", "Conversion efficiency"),
      gap: N("balances.energy.reactor_steam_to_reboiler.value", "MW", 1, "live", "Reactor steam to the reboiler"),
      ec: N("balances.power.e_circ.value", "MWe", 2, "live", "Helium circulators"),
      et: N("units.power.results.e_total.value", "MWe", 2, "live", "Site electricity demand"),
      imp: N("kpi.unmet_power_mwe", "MWe", 2, "live", "Grid import"),
    },
    figs: (c) => (c.feasible ? ["br", "gen", "et", "imp"] : ["brn", "gen", "et", "imp"]),
    text: (c) => {
      const last = "The helium circulators draw {ec} of the {et} site demand, and the grid supplies {imp}.";
      if (burned(c)) {
        return [
          "In this scenario electricity from the reject burned on site takes the place of the power block, so the helium branch carries {br} and the turbine makes {gen}.",
          last,
        ];
      }
      if (!c.feasible) {
        // The branch and the turbine output of the reply are at the need; the
        // generation at the rated reactor output is a field of its own.
        return [
          "At the need, a branch of the hot primary helium would carry {brn} to the power block and its turbine would make {en}. At the reactor's rated output the turbine makes {gen}.",
          last,
        ];
      }
      if (c.q(STEAM_GAP) > 0) {
        return [
          "A branch of the hot primary helium carries {br} to the power block. The steam for the reboiler first does back-pressure work in the turbine and then gives the reboiler {gap}; the rest of the branch drives the turbine at {eta} efficiency, and the two together make {gen}.",
          last,
        ];
      }
      return [
        "A branch of the hot primary helium carries {br} to the power block, whose turbine makes {gen} at {eta} efficiency.",
        last,
      ];
    },
    notes: (c) => [c.limit("circ_primary", 1)],
  },
  {
    id: "closing",
    title: "The plant from end to end",
    ms: 8000,
    all: true,
    units: [],
    marks: [],
    flows: (c) => [closingLines(c)],
    lead: "htgr",
    feas: true,
    nums: {
      h2: N("kpi.h2_product_t_y", "t/y", 0, "live", "Purified H₂"),
      base: N("balances.carbon.baseline_t_y.value", "t/y", 0, "live", "Unabated reformer baseline"),
      cap: N("kpi.co2_captured_t_y", "t/y", 0, "live", "CO₂ to storage"),
      atm: N("kpi.co2_atm_t_y", "t/y", 0, "live", "CO₂ to the atmosphere"),
      ab: N("kpi.abatement_t_y", "t/y", 0, "live", "Net abatement"),
      tgt: N("kpi.target_abatement_t_y", "t/y", 0, "fixed", "Abatement target"),
      cho: N("kpi.abatement_cho_t_y", "t/y", 0, "live", "Net abatement, Cho baseline"),
      ie: N("kpi.abatement_ieaghg_t_y", "t/y", 0, "live", "Net abatement, IEAGHG baseline"),
    },
    figs: ["h2", "cap", "atm", "ab"],
    text: (c) => [
      "End to end, the plant makes {h2} of purified hydrogen and sends {cap} of CO₂ to storage.",
      `Against the {base} an unabated reformer would emit on INL's baseline, its residual of {atm} leaves a net abatement of {ab}. Part of that is the CO₂ stored; the rest is never formed, because ${recycles(c)
        ? "reactor heat replaces the reformer's fuel gas and the cryogenic recycle returns methane to the feed"
        : "reactor heat replaces the reformer's fuel gas"}.`,
      `${abatementVerdict(c)} On Cho's and IEAGHG's baselines the net abatement is {cho} and {ie}.`,
    ],
    status: (c) => {
      const t = c.r && c.r.targets && c.r.targets.cost;
      if (!t) return "";
      return t.met ? "Cost target: met at these inputs." : `Cost target: ${t.reason || "not met at these inputs"}.`;
    },
    notes: () => ["The plant sits on the proposed Western Island site, a proposal of this study with no official approval."],
  },
];

/** The engine's abatement verdict in words. Net abatement at or above the
 * target on an infeasible plant does not count as met. */
function abatementVerdict(c) {
  const a = c.r && c.r.targets ? c.r.targets.abatement : null;
  if (a && a.met) return "The abatement target of {tgt} is met at these inputs.";
  if (a && a.at_or_above_target && a.feasible === false) {
    return "The net abatement is at or above the target of {tgt}, but the engine does not count the target as met, since the plant is not feasible at these inputs.";
  }
  return "The abatement target of {tgt} is not met at these inputs.";
}

// ------------------------------------------------------------------ state

let api = null;
const el = {};
let mq = null;
let active = false;
let idx = 0;
let playing = false;
let ended = false;
let still = false;
let scale = 1;
let stepAnim = null;
let flowSig = "";
let prevSelected = null;
let prevLive = null;

const n = () => STEPS.length;
const pick = (v, c) => (typeof v === "function" ? v(c) : v || []);
const titleOf = (s, c) => (typeof s.title === "function" ? s.title(c) : s.title);
const leadOf = (s, c) => (typeof s.lead === "function" ? s.lead(c) : s.lead);

function reply() {
  return api ? api.state.lastOk || null : null;
}

function allStreams() {
  return el.svg ? Array.from(el.svg.querySelectorAll(".pfd-streams > [data-stream]")).map((g) => g.dataset.stream) : [];
}

/** Context of one render: the reply, the inputs it was solved at and helpers. */
function context() {
  const r = reply();
  const x = (r && r.inputs) || (api && api.state.inputs) || {};
  const flags = (r && Array.isArray(r.flags)) ? r.flags : [];
  return {
    r,
    x,
    q: (p) => {
      const v = r ? walk(r, p) : undefined;
      return typeof v === "number" && Number.isFinite(v) ? v : null;
    },
    feasible: !(r && r.feasibility && r.feasibility.feasible === false),
    flag: (param) => {
      const f = flags.find((e) => e.param === param);
      return f ? `Input outside its evidence range: ${flagText(f.text)}.` : "";
    },
    limit: (unit, i) => {
      const d = api.defs();
      const l = d && d.units && d.units[unit] && Array.isArray(d.units[unit].limitations) ? d.units[unit].limitations[i] : null;
      return l ? fillText(l.text, r) : "";
    },
  };
}

// ------------------------------------------------------------------ numbers

function unitText(u) {
  return u === "C" ? "°C" : u === "-" ? "%" : u;
}

/** Display text of a value: fixed decimals, a fraction as a percentage. */
function fmtNum(v, u, d) {
  if (typeof v !== "number" || !Number.isFinite(v)) return null;
  const x = u === "-" ? v * 100 : v;
  return x.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d });
}

/** One number as HTML; the data attributes let the self test check it against
 * its reply path. A no-break space keeps the unit with its number in the text
 * that is read aloud or copied; a percentage takes no space. */
function numHtml(spec, c) {
  if (!spec) return "n/a";
  const text = fmtNum(c.q(spec.p), spec.u, spec.d);
  const u = unitText(spec.u);
  const unit = text === null ? "" : `${u === "%" ? "" : "\u00a0"}<span class="tu">${esc(u)}</span>`;
  return `<b class="tn" data-path="${esc(spec.p)}" data-u="${esc(spec.u)}" data-d="${spec.d}">${esc(text === null ? "n/a" : text)}</b>${unit}`;
}

function fill(tpl, s, c) {
  return String(tpl).split(/(\{\w+\})/).map((part) => {
    const m = /^\{(\w+)\}$/.exec(part);
    return m ? numHtml(s.nums[m[1]], c) : esc(part);
  }).join("");
}

/** Reasons the engine gives for numbers of this step it left null. */
function missingNotes(s, c, keys) {
  const list = (c.r && Array.isArray(c.r.engine_fields_missing)) ? c.r.engine_fields_missing : [];
  const out = [];
  for (const k of keys) {
    const spec = s.nums[k];
    if (!spec || c.q(spec.p) !== null) continue;
    const field = spec.p.replace(/\.value$/, "");
    const m = list.find((e) => e.field === field || e.field === spec.p);
    out.push(`${spec.l}: n/a${m ? `, ${m.reason}` : c.r ? "" : ", waiting for the engine"}.`);
  }
  return out;
}

// ------------------------------------------------------------------ pictures

// Illustrations drawn without engine data. They carry no numerals.
const INSETS = {
  fission: {
    caption: "Illustration. The engine models the reactor as a heat source and holds no fuel, neutron or core data.",
    label: "Illustration: inside a TRISO fuel particle in graphite, a neutron splits a uranium nucleus into two fragments and new neutrons, and the heat passes to the helium flowing past.",
    svg: `<svg viewBox="0 0 340 196" class="tf" aria-hidden="true" focusable="false">
<defs>
<pattern id="tf-gr" width="14" height="14" patternUnits="userSpaceOnUse"><circle cx="3" cy="4" r="1.2" fill="#3c444d"/><circle cx="10" cy="11" r="1.5" fill="#363d45"/></pattern>
<pattern id="tf-kr" width="9" height="9" patternUnits="userSpaceOnUse"><circle cx="2" cy="2" r="1" fill="#82461a"/><circle cx="6.5" cy="6.5" r="1" fill="#7a4016"/></pattern>
<clipPath id="tf-clip"><circle cx="232" cy="72" r="56"/></clipPath>
</defs>
<rect width="340" height="196" fill="#262c33"/>
<rect width="340" height="150" fill="url(#tf-gr)"/>
<text class="tf-t" x="8" y="16">Graphite</text>
<circle cx="62" cy="72" r="44" fill="#4a5058"/>
<circle cx="62" cy="72" r="39" fill="#c3cbd2"/>
<circle cx="62" cy="72" r="34" fill="#4a5058"/>
<circle cx="62" cy="72" r="29" fill="#2f3439"/>
<circle cx="62" cy="72" r="22" fill="#cf7a2c"/>
<path d="M62,50 L232,16 M62,94 L232,128" stroke="#8b939b" stroke-width="1" stroke-dasharray="3 3" fill="none"/>
<text class="tf-t" x="62" y="142" text-anchor="middle">TRISO fuel particle</text>
<g clip-path="url(#tf-clip)">
<rect x="170" y="12" width="130" height="122" fill="#6e3a12"/>
<rect x="170" y="12" width="130" height="122" fill="url(#tf-kr)"/>
<circle class="tf-ghost" cx="232" cy="72" r="14" fill="none" stroke="#f2c14e" stroke-width="1.5" stroke-dasharray="3 3"/>
<circle class="tf-nuc" cx="232" cy="72" r="14" fill="#f2c14e"/>
<g class="tf-n0"><path d="M186,72 L206,72" stroke="#9fd3ff" stroke-width="1.5" stroke-dasharray="2 3"/><circle cx="211" cy="72" r="4.5" fill="#9fd3ff"/></g>
<g class="tf-f1"><circle cx="254" cy="52" r="9" fill="#f29a2e"/></g>
<g class="tf-f2"><circle cx="210" cy="93" r="9" fill="#f29a2e"/></g>
<g class="tf-n1"><circle cx="278" cy="90" r="4" fill="#9fd3ff"/></g>
<g class="tf-n2"><circle cx="205" cy="44" r="4" fill="#9fd3ff"/></g>
<g class="tf-heat" fill="none" stroke="#ffd29a" stroke-width="1.6" stroke-linecap="round">
<path d="M266,38 q4,-4 8,0 t8,0"/><path d="M188,106 q4,-4 8,0 t8,0"/><path d="M244,110 q4,-4 8,0 t8,0"/>
</g>
</g>
<circle cx="232" cy="72" r="56" fill="none" stroke="#c3cbd2" stroke-width="1.5"/>
<text class="tf-t" x="203" y="60" text-anchor="middle">neutron</text>
<text class="tf-t" x="232" y="142" text-anchor="middle">a neutron splits a uranium nucleus</text>
<rect y="150" width="340" height="46" fill="#3d1f1c"/>
<g class="tf-heatup" fill="none" stroke="#ffb38a" stroke-width="1.6" stroke-linecap="round">
<path d="M40,150 q-4,4 0,7 t0,7"/><path d="M126,150 q-4,4 0,7 t0,7"/><path d="M300,150 q-4,4 0,7 t0,7"/>
</g>
<path class="tf-he" d="M6,172 L334,172" stroke="#e2463b" stroke-width="5" fill="none"/>
<path class="tf-he-flow" d="M6,172 L334,172" stroke="#ffffff" stroke-width="2.5" stroke-dasharray="8 22" stroke-linecap="round" fill="none"/>
<path d="M334,172 L324,166 L324,178 Z" fill="#e2463b"/>
<text class="tf-t" x="170" y="190" text-anchor="middle">helium carries the heat out of the core</text>
</svg>`,
  },
  ship: {
    caption: "Illustration. The engine models the ship and the store as a tariff only.",
    label: "Illustration: a ship carries the CO₂ from the plant to an offshore store, where it goes into a rock formation deep under the seabed.",
    svg: `<svg viewBox="0 0 340 160" class="ts" aria-hidden="true" focusable="false">
<rect width="340" height="160" fill="#dfeaf3"/>
<rect y="84" width="340" height="34" fill="#3f7fb0"/>
<path class="ts-wave" d="M0,90 q10,-4 20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0 t20,0" fill="none" stroke="#a9d0ee" stroke-width="1.4"/>
<rect y="118" width="340" height="42" fill="#76644f"/>
<rect y="136" width="340" height="16" fill="#4c3e30"/>
<text class="ts-tl" x="150" y="148">storage formation</text>
<path d="M0,72 L64,72 L74,84 L0,84 Z" fill="#8b9a7c"/>
<rect x="14" y="44" width="26" height="30" rx="4" fill="#9aa5b0" stroke="#4a545e" stroke-width="1.5"/>
<text class="ts-t" x="6" y="36">from the plant</text>
<g class="ts-ship">
<path d="M84,80 L146,80 L138,92 L92,92 Z" fill="#2f363e"/>
<rect x="124" y="70" width="14" height="10" fill="#e8ecef" stroke="#2f363e" stroke-width="1"/>
<rect x="96" y="72" width="22" height="8" rx="3" fill="#9aa5b0"/>
<text class="ts-t" x="100" y="64">ship</text>
</g>
<path class="ts-route" d="M150,100 L250,100" stroke="#ffffff" stroke-width="1.5" stroke-dasharray="4 4" fill="none"/>
<path d="M256,100 L249,96 L249,104 Z" fill="#ffffff"/>
<rect x="268" y="58" width="44" height="8" fill="#4a545e"/>
<path d="M274,66 L274,118 M306,66 L306,118" stroke="#4a545e" stroke-width="3"/>
<path d="M290,66 L290,144" stroke="#1d2329" stroke-width="2.5" stroke-dasharray="5 3"/>
<path d="M290,148 L285,140 L295,140 Z" fill="#1d2329"/>
<text class="ts-t" x="256" y="46">offshore store</text>
</svg>`,
  },
};

function insetHtml(key) {
  const p = INSETS[key];
  if (!p) return "";
  return `<figure class="tour-inset" data-inset="${esc(key)}" role="img" aria-label="${esc(p.label)}">${p.svg}<figcaption>${esc(p.caption)}</figcaption></figure>`;
}

// ------------------------------------------------------------------ the drawing

function clearMarks() {
  for (const e of el.svg.querySelectorAll(".tour-on, .tour-ctx, .tour-move, .tour-lead")) e.classList.remove("tour-on", "tour-ctx", "tour-move", "tour-lead");
}

function light(sel, cls) {
  for (const e of el.svg.querySelectorAll(sel)) e.classList.add(cls);
}

function streamIds(groups) {
  return groups.flat().filter((x) => typeof x === "string");
}

function applyMarks(s, c) {
  clearMarks();
  const svg = el.svg;
  svg.classList.add("is-touring");
  svg.classList.toggle("is-still", still);
  svg.classList.toggle("tour-all", Boolean(s.all));
  svg.dataset.tourStep = s.id;
  for (const id of pick(s.units, c)) light(`[data-unit="${CSS.escape(id)}"]`, "tour-on");
  for (const id of pick(s.ctx, c)) light(`[data-stream="${CSS.escape(id)}"]`, "tour-ctx");
  for (const id of pick(s.ctxUnits, c)) light(`[data-unit="${CSS.escape(id)}"]`, "tour-ctx");
  for (const id of streamIds(pick(s.flows, c))) {
    light(`[data-stream="${CSS.escape(id)}"]`, "tour-on");
    if (!s.all) light(`[data-stream="${CSS.escape(id)}"]`, "tour-move");
  }
  for (const k of pick(s.marks, c)) light(`[data-tour="${CSS.escape(k)}"]`, "tour-on");
  const lead = leadOf(s, c);
  if (lead) light(`[data-unit="${CSS.escape(lead)}"]`, "tour-lead");
}

function mediumOf(g) {
  const m = Array.from(g.classList).map((k) => /^st-(\w+)$/.exec(k)).find(Boolean);
  return m ? m[1] : "rej";
}

function segmentsOf(item) {
  if (typeof item !== "string") return [item];
  const g = el.svg.querySelector(`.pfd-streams > [data-stream="${CSS.escape(item)}"]`);
  if (!g) return [];
  const p = g.querySelector("path.core") || g.querySelector("path.vent");
  return p ? [{ m: mediumOf(g), d: p.getAttribute("d") }] : [];
}

function clearFlows() {
  if (el.flows) el.flows.replaceChildren();
}

/** Dashes moving along the step's streams, one group after another. None when still. */
function buildFlows(s, c) {
  const groups = pick(s.flows, c);
  const sig = `${s.id}|${still}|${JSON.stringify(groups)}`;
  if (sig === flowSig) return;
  flowSig = sig;
  clearFlows();
  if (still) return;
  const ns = "http://www.w3.org/2000/svg";
  groups.forEach((grp, gi) => {
    for (const item of grp) {
      for (const seg of segmentsOf(item)) {
        const p = document.createElementNS(ns, "path");
        p.setAttribute("class", "tour-flow");
        p.setAttribute("d", seg.d);
        p.dataset.m = seg.m;
        p.style.setProperty("--tour-delay", `${gi * STAGGER_MS}ms`);
        el.flows.appendChild(p);
      }
    }
  });
}

/** Scroll the sideways scroller so the lit part of the drawing sits in view. */
function frame() {
  const box = api.scroll;
  if (!box || box.scrollWidth <= box.clientWidth + 1) return;
  const lit = el.svg.querySelectorAll(".pfd-units .tour-on, .pfd-streams .tour-on");
  if (!lit.length) return;
  let left = Infinity;
  let right = -Infinity;
  for (const e of lit) {
    const b = e.getBoundingClientRect();
    left = Math.min(left, b.left);
    right = Math.max(right, b.right);
  }
  const host = box.getBoundingClientRect();
  const span = right - left;
  const target = span > box.clientWidth
    ? box.scrollLeft + (left - host.left) - 12
    : box.scrollLeft + (left - host.left) - (box.clientWidth - span) / 2;
  box.scrollTo({ left: Math.max(0, target), behavior: still ? "auto" : "smooth" });
}

// ------------------------------------------------------------------ the card

function tagHtml(tag, text) {
  return `<span class="tour-tag" data-tag="${esc(tag)}">${esc(text)}</span>`;
}

function scenarioText(c) {
  const reg = api.state.registry;
  if (!reg || !c.r) return "";
  const off = [];
  for (const id of ["basis", "tail_gas", "heat_basis"]) {
    const p = reg.params.find((e) => e.id === id);
    if (!p || c.x[id] === undefined || c.x[id] === p.default) continue;
    const o = (p.options || []).find((e) => e.value === c.x[id]);
    off.push(`${p.label}: ${o ? o.label : c.x[id]}`);
  }
  return off.length
    ? `Scenario: ${off.join("; ")}. The drawing shows the headline plant, Case B with the reject recycled, so some drawn lines may carry nothing here.`
    : "";
}

function renderCard() {
  const s = STEPS[idx];
  const c = context();
  el.kicker.textContent = `Step ${idx + 1} of ${n()}`;
  el.title.textContent = titleOf(s, c);
  const figs = pick(s.figs, c);
  const eqs = pick(s.eq, c);
  const stale = api.state.result && api.state.result.status === "not_converged";
  const lead = leadOf(s, c);
  const leadStatus = c.r && c.r.units && c.r.units[lead] ? c.r.units[lead].status : null;

  const tags = [c.r ? tagHtml("live", "Live engine values") : tagHtml("wait", "Waiting for the engine")];
  if (stale) tags.push(tagHtml("stale", "Last converged result"));
  if (s.inset || eqs.length) tags.push(tagHtml("illustration", "Includes an illustration"));
  if (leadStatus === "external") tags.push(tagHtml("external", "External block, not simulated"));

  const media = [];
  for (const item of pick(s.flows, c).flat()) {
    const segs = segmentsOf(item);
    for (const seg of segs) if (!media.includes(seg.m)) media.push(seg.m);
  }
  const mediaHtml = media.length && !s.all
    ? `<ul class="tour-media" aria-label="Flows in this step">${media.map((m) => `<li><span class="sw" data-m="${esc(m)}" aria-hidden="true"></span>${esc(MEDIA[m] || m)}</li>`).join("")}</ul>`
    : "";

  const warn = [];
  if (s.feas && !c.feasible && c.r && c.r.feasibility) {
    warn.push(`<p class="tour-warn"><strong>Not feasible at these inputs.</strong> ${esc(c.r.feasibility.reason)}.</p>`);
  }
  const scen = scenarioText(c);

  const text = pick(s.text, c).map((t) => `<p>${fill(t, s, c)}</p>`).join("");
  const eqHtml = eqs.map((e) => `<p class="tour-eq">${tagHtml("illustration", "Reaction, for illustration")}<span class="eq-line">${esc(e)}</span></p>`).join("");
  const status = s.status ? s.status(c) : "";
  const figHtml = figs.length
    ? `<dl class="tour-figs">${figs.map((k) => {
      const spec = s.nums[k];
      return `<div data-fig="${esc(k)}"><dt>${esc(spec.l)}</dt><dd>${numHtml(spec, c)}<span class="tk" data-kind="${esc(spec.k)}">${esc(KIND[spec.k] || spec.k)}</span></dd></div>`;
    }).join("")}</dl>`
    : "";
  const textKeys = [];
  for (const t of pick(s.text, c)) for (const m of String(t).matchAll(/\{(\w+)\}/g)) textKeys.push(m[1]);
  const notes = [
    ...missingNotes(s, c, [...new Set([...textKeys, ...figs])]),
    ...pick(s.notes, c),
  ].filter(Boolean);
  const motion = still
    ? "The lit lines keep the drawn arrows that mark the direction of flow."
    : "The moving dashes mark the direction of flow. Their speed is fixed and carries no meaning.";

  el.body.innerHTML = [
    `<div class="tour-tags">${tags.join("")}</div>`,
    warn.join(""),
    scen ? `<p class="tour-scen">${esc(scen)}</p>` : "",
    mediaHtml,
    `<div class="tour-text">${text}</div>`,
    eqHtml,
    s.inset ? insetHtml(s.inset) : "",
    figHtml,
    status ? `<p class="tour-status">${esc(status)}</p>` : "",
    notes.length ? `<ul class="tour-notes">${notes.map((x) => `<li>${esc(x)}</li>`).join("")}</ul>` : "",
    s.all || pick(s.flows, c).length ? `<p class="tour-motion">${esc(motion)}</p>` : "",
  ].join("");
  el.card.classList.toggle("is-still", still);
  el.card.classList.toggle("is-paused", !still && !playing);
}

function announce() {
  const text = Array.from(el.body.querySelectorAll(".tour-warn, .tour-text p")).map((p) => p.textContent.trim()).join(" ");
  el.live.textContent = `Step ${idx + 1} of ${n()}. ${el.title.textContent}. ${text}`;
}

/** The key line under the card, which names only the keys that act now. */
function keyHint() {
  if (!el.keys) return;
  el.keys.textContent = still
    ? "With focus off the drawing, the Left and Right arrow keys change the step. Escape leaves the tour."
    : "With focus off the drawing, the Left and Right arrow keys change the step, and Space pauses or plays except on a button or a link, where it keeps its usual action. Escape leaves the tour.";
}

// ------------------------------------------------------------------ controls

function updateBar() {
  el.count.textContent = `Step ${idx + 1} of ${n()}`;
  el.track.setAttribute("aria-valuemin", "1");
  el.track.setAttribute("aria-valuemax", String(n()));
  el.track.setAttribute("aria-valuenow", String(idx + 1));
  el.track.setAttribute("aria-valuetext", `Step ${idx + 1} of ${n()}: ${el.title.textContent}`);
  for (const b of el.prevs) b.setAttribute("aria-disabled", String(idx === 0));
  for (const b of el.nexts) b.setAttribute("aria-disabled", String(idx === n() - 1));
  el.play.hidden = still;
  const word = playing ? "Pause" : ended ? "Replay" : "Play";
  el.play.querySelector("span").textContent = word;
  el.play.setAttribute("aria-label", `${word} the tour`);
  el.play.dataset.state = playing ? "playing" : "paused";
  el.svg.classList.toggle("is-paused", !still && !playing);
  el.card.classList.toggle("is-paused", !still && !playing);
}

function dwell(i) {
  return Math.max(50, STEPS[i].ms * scale);
}

/** The progress fill of the step; when it finishes, the tour moves on. */
function armTimer() {
  if (stepAnim) {
    stepAnim.onfinish = null;
    stepAnim.cancel();
    stepAnim = null;
  }
  const from = (idx / n()) * 100;
  const to = ((idx + 1) / n()) * 100;
  if (still || typeof el.fill.animate !== "function") {
    el.fill.style.width = `${to}%`;
    return;
  }
  el.fill.style.width = `${from}%`;
  stepAnim = el.fill.animate([{ width: `${from}%` }, { width: `${to}%` }], { duration: dwell(idx), easing: "linear", fill: "forwards" });
  stepAnim.onfinish = () => {
    if (!active) return;
    if (idx < n() - 1) {
      // An automatic step: the view stays where the reader left it.
      go(idx + 1);
    } else {
      playing = false;
      ended = true;
      updateBar();
    }
  };
  if (!playing) stepAnim.pause();
}

// Room kept around the control and the card head when the tour scrolls them
// into view, so a focus ring shows as well.
const RING_PX = 8;

/** The side column when it scrolls by itself, from 1200 px wide. */
function sideScroller() {
  const side = el.card.parentElement;
  if (!side) return null;
  const o = getComputedStyle(side).overflowY;
  return o === "auto" || o === "scroll" ? side : null;
}

/** The focused element when it sits in the card or the bar, else null. */
function tourFocus() {
  const a = document.activeElement;
  return a && a !== document.body && (el.card.contains(a) || el.bar.contains(a)) ? a : null;
}

/** How far a scroller that shows top..bottom moves its content up (down when
 * negative), by the smallest scroll, so that the control shows whole with
 * RING_PX of room, and the card head as well when the two fit there
 * together. The head keeps the same room when it can and may otherwise touch
 * the edge. Boxes are {top, bottom} in window coordinates; ctl is null when
 * the reader used no control, and head is null when this scroller cannot
 * bring it into view. */
function shift(top, bottom, ctl, head) {
  // The shifts that keep each box inside its range, as [least, most].
  const keep = (b, room) => [b.bottom - (bottom - room), b.top - (top + room)];
  const meet = (p, q) => [Math.max(p[0], q[0]), Math.min(p[1], q[1])];
  const nearest = (r) => (r[0] > r[1] ? null : Math.min(Math.max(0, r[0]), r[1]));
  const own = ctl ? keep(ctl, RING_PX) : [-Infinity, Infinity];
  const solo = ctl ? (nearest(own) ?? own[1]) : 0;
  if (!head) return solo;
  const both = nearest(meet(own, keep(head, RING_PX))) ?? nearest(meet(own, keep(head, 0)));
  if (both !== null) return both;
  // They do not fit, so the control wins; with no control the head's top shows.
  return ctl ? solo : head.top - top;
}

/** After a step change the reader asked for: keep the control they used in
 * view with the smallest scroll, and bring the card head into view too when
 * the head and that control fit in the window together. When they do not fit,
 * the control wins. The side column, which holds the card from 1200 px wide,
 * scrolls first, and then the page. */
function reveal(ctl) {
  const behavior = still ? "auto" : "smooth";
  const box = (e) => {
    const r = e.getBoundingClientRect();
    return { top: r.top, bottom: r.bottom };
  };
  const moved = (b, dy) => ({ top: b.top - dy, bottom: b.bottom - dy });
  let c = ctl && ctl.isConnected ? box(ctl) : null;
  let h = box(el.head || el.card);
  const side = sideScroller();
  if (side && side.contains(el.card)) {
    const s = side.getBoundingClientRect();
    const inSide = Boolean(c && side.contains(ctl));
    const want = shift(s.top, s.bottom, inSide ? c : null, h);
    const dy = Math.round(Math.max(-side.scrollTop, Math.min(side.scrollHeight - side.clientHeight - side.scrollTop, want)));
    if (dy) {
      side.scrollBy({ top: dy, behavior });
      if (inSide) c = moved(c, dy);
      h = moved(h, dy);
    }
    // A head the side column keeps hidden, because it does not fit there with
    // the control, cannot be shown by scrolling the page.
    if (h.top < s.top - 0.5 || h.bottom > s.bottom + 0.5) h = null;
  }
  const max = document.documentElement.scrollHeight - window.innerHeight;
  const want = shift(0, window.innerHeight, c, h);
  const dy = Math.round(Math.max(-window.scrollY, Math.min(max - window.scrollY, want)));
  if (dy) window.scrollBy({ top: dy, behavior });
}

/** Show step i. A step the reader asks for passes reader true and the control
 * they used, if any, and the view follows it through reveal(). Every other
 * step change, by the timer or at Start, leaves the page and the side column
 * where they are: it only redraws the step and announces it. */
function go(i, reader = false, ctl = null) {
  if (!active) return;
  idx = Math.max(0, Math.min(n() - 1, i));
  ended = false;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  const lead = leadOf(s, c);
  if (lead && api.select) api.select(lead);
  renderCard();
  updateBar();
  armTimer();
  announce();
  frame();
  if (reader) reveal(ctl);
}

function pause() {
  if (!playing) return;
  playing = false;
  if (stepAnim) stepAnim.pause();
  updateBar();
}

/** Play on; after the last step, Replay goes back to step 1 as a step the
 * reader asked for, with ctl the control they used. */
function resume(ctl) {
  if (still) return;
  if (ended) {
    playing = true;
    go(0, true, ctl);
    return;
  }
  playing = true;
  if (stepAnim) stepAnim.play();
  else armTimer();
  updateBar();
}

function toggle(ctl = null) {
  if (playing) pause();
  else resume(ctl);
}

/** Start the tour. Options for the self test: reduced (force still steps),
 * scale (dwell factor) and step (first step index). */
export function startTour(opts = {}) {
  if (!api) return false;
  if (active) exitTour(false);
  still = opts.reduced === undefined ? Boolean(mq && mq.matches) : Boolean(opts.reduced);
  scale = typeof opts.scale === "number" && opts.scale > 0 ? opts.scale : 1;
  active = true;
  playing = !still;
  ended = false;
  flowSig = "";
  prevSelected = api.selected ? api.selected() : null;
  prevLive = el.panel.getAttribute("aria-live");
  el.panel.setAttribute("aria-live", "off");
  el.start.hidden = true;
  el.note.hidden = true;
  el.bar.hidden = false;
  el.card.hidden = false;
  if (el.route) el.route.classList.add("is-touring");
  el.bar.classList.toggle("is-still", still);
  el.track.style.setProperty("--n", String(n()));
  keyHint();
  go(typeof opts.step === "number" ? opts.step : 0);
  placeStart();
  return true;
}

/** After Start: bring the tour near the top of the window and move focus.
 * With the card in the side column the anchor is the top of the layout, which
 * the card shares; with the card under the drawing it is the bar. The page
 * scrolls when the anchor is above the window or low in it, or when that
 * brings an unseen step title into view. Focus goes to the step title when it
 * will be fully in view, or else to the first control of the bar. */
function placeStart() {
  const bar = el.bar.getBoundingClientRect();
  const side = el.card.getBoundingClientRect().top < bar.top;
  const anchor = side ? el.bar.closest(".pfd-layout") || el.bar : el.bar;
  const top = anchor.getBoundingClientRect().top;
  const t = el.title.getBoundingClientRect();
  const fits = (dy) => t.top - dy >= 0 && t.bottom - dy <= window.innerHeight;
  const max = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
  const lift = Math.max(0, Math.min(max, window.scrollY + top - 8)) - window.scrollY;
  const dy = top < 0 || top > window.innerHeight * 0.4 || (!fits(0) && fits(lift)) ? lift : 0;
  if (dy) window.scrollTo({ top: window.scrollY + dy, behavior: still ? "auto" : "smooth" });
  const target = fits(dy) ? el.title : still ? el.nexts[0] : el.play;
  target.focus({ preventScroll: true });
}

/** Leave the tour; with focusStart, focus goes back to the start button. */
export function exitTour(focusStart) {
  if (!active) return;
  active = false;
  playing = false;
  ended = false;
  if (stepAnim) {
    stepAnim.onfinish = null;
    stepAnim.cancel();
    stepAnim = null;
  }
  flowSig = "";
  clearFlows();
  clearMarks();
  el.svg.classList.remove("is-touring", "is-still", "is-paused", "tour-all");
  delete el.svg.dataset.tourStep;
  el.bar.hidden = true;
  el.card.hidden = true;
  if (el.route) el.route.classList.remove("is-touring");
  el.start.hidden = false;
  el.note.hidden = false;
  // The panel goes back to the reader's unit before it speaks again.
  if (prevSelected && api.select) api.select(prevSelected);
  if (prevLive === null) el.panel.removeAttribute("aria-live");
  else el.panel.setAttribute("aria-live", prevLive);
  el.live.textContent = "The process tour is closed.";
  if (focusStart) el.start.focus();
}

/** Go to step i as the reader would, with the focused tour element as the control. */
export function goTour(i) {
  go(i, true, tourFocus());
}

/** Re-render the numbers of the running step after a new reply. */
export function refreshTour() {
  if (!active) return;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  renderCard();
  updateBar();
}

function onBar(e) {
  const b = e.target.closest("[data-act]");
  if (!b || !active || b.getAttribute("aria-disabled") === "true") return;
  const act = b.dataset.act;
  if (act === "prev") go(idx - 1, true, b);
  else if (act === "next") go(idx + 1, true, b);
  else if (act === "play") toggle(b);
  else if (act === "exit") exitTour(true);
}

function onKey(e) {
  if (!active || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
  const t = e.target && e.target.closest ? e.target : null;
  const inDrawing = t && t.closest("[data-unit], [data-stream], .pfd-tip");
  const formish = t && t.closest("input, select, textarea, [contenteditable=\"true\"]");
  if (inDrawing || formish) return;
  // With an arrow key the control is the focused element of the card or the
  // bar, if focus is there.
  if (e.key === "ArrowRight") {
    e.preventDefault();
    go(idx + 1, true, tourFocus());
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    go(idx - 1, true, tourFocus());
  } else if (e.key === " " || e.key === "Spacebar") {
    // A focused button or link keeps its own Space, as the key line says.
    if (t && t.closest("button, a[href], summary")) return;
    e.preventDefault();
    toggle(tourFocus());
  }
}

function onEscape(e) {
  if (!active || e.key !== "Escape") return;
  // An open stream tip closes first; pfd.js handles that press.
  if (api.tip && !api.tip.hidden) return;
  e.preventDefault();
  exitTour(true);
}

function startNote() {
  const total = STEPS.reduce((a, s) => a + s.ms, 0);
  const minutes = Math.round(total / 30000) / 2;
  el.note.textContent = mq && mq.matches
    ? `${n()} steps, from fission heat to the hydrogen and CO₂ ends; step through at your own pace.`
    : `${n()} steps, from fission heat to the hydrogen and CO₂ ends, about ${minutes.toLocaleString("en-GB")} minutes.`;
}

/** Wire the tour to the drawing. `a` holds the shell state, the drawing and pfd.js hooks. */
export function initTour(a) {
  api = a;
  el.svg = a.svg;
  el.panel = a.panel;
  el.start = document.getElementById("tour-start");
  el.note = document.getElementById("tour-start-note");
  el.bar = document.getElementById("tour-bar");
  el.card = document.getElementById("tour-card");
  el.head = el.card ? el.card.querySelector(".tour-head") : null;
  el.kicker = document.getElementById("tour-kicker");
  el.title = document.getElementById("tour-title");
  el.body = document.getElementById("tour-body");
  el.live = document.getElementById("tour-live");
  el.count = document.getElementById("tour-count");
  el.track = document.getElementById("tour-track");
  el.fill = document.getElementById("tour-fill");
  el.route = document.getElementById("route-pfd");
  el.keys = document.getElementById("tour-keys");
  el.pick = document.getElementById("pfd-select");
  if (!el.start || !el.bar || !el.card || !el.svg) return;
  // Previous and Next sit in the bar above the drawing and again under the
  // card, so a keyboard reader on the card need not cross the drawing.
  el.prevs = Array.from(document.querySelectorAll('#tour-bar [data-act="prev"], #tour-card [data-act="prev"]'));
  el.nexts = Array.from(document.querySelectorAll('#tour-bar [data-act="next"], #tour-card [data-act="next"]'));
  el.play = el.bar.querySelector('[data-act="play"]');
  el.flows = document.createElementNS("http://www.w3.org/2000/svg", "g");
  el.flows.setAttribute("class", "tour-flows");
  el.flows.setAttribute("aria-hidden", "true");
  el.svg.appendChild(el.flows);
  mq = window.matchMedia ? window.matchMedia(MOTION) : null;
  if (mq && mq.addEventListener) {
    mq.addEventListener("change", () => {
      startNote();
      if (active) startTour({ step: idx });
    });
  }
  el.start.addEventListener("click", () => startTour());
  el.bar.addEventListener("click", onBar);
  el.card.addEventListener("click", onBar);
  document.addEventListener("keydown", onKey);
  document.addEventListener("keydown", onEscape, true);
  // Reading the drawing pauses the tour, so a panel the reader opens stays put
  // until they move on: a click, Enter or Space on a unit or stream, or a
  // choice from the unit list.
  el.svg.addEventListener("click", () => { if (active && playing) pause(); }, true);
  el.svg.addEventListener("keydown", (e) => {
    if (!active || !playing || !(e.key === "Enter" || e.key === " " || e.key === "Spacebar")) return;
    if (e.target && e.target.closest && e.target.closest("[data-unit], [data-stream]")) pause();
  }, true);
  if (el.pick) el.pick.addEventListener("change", () => { if (active && playing) pause(); });
  window.addEventListener("hashchange", () => { if (active && location.hash !== "#pfd") exitTour(false); });
  startNote();
  el.start.hidden = false;
  el.note.hidden = false;
}

// ------------------------------------------------------------------ self test

/** Ids, keys and reply paths the steps name, checked against the page and a reply. */
export function tourAudit(res) {
  const svg = el.svg;
  const miss = { units: new Set(), streams: new Set(), marks: new Set(), keys: new Set() };
  const badPaths = [];
  const listed = new Set(((res && res.engine_fields_missing) || []).map((m) => m.field));
  // Every branch of the steps: each reject route, a zero and a non-zero value
  // for every field a branch reads, and a feasible and an infeasible plant.
  const contexts = [];
  for (const tail of ["recycle", "export", "burn_on_site", "inl_selexol"]) {
    for (const v of [0, 1]) {
      for (const feasible of [true, false]) {
        contexts.push({ x: { ...((res && res.inputs) || {}), tail_gas: tail }, q: () => v, r: res, flag: () => "", limit: () => "", feasible });
      }
    }
  }
  for (const s of STEPS) {
    for (const c of contexts) {
      for (const id of [...pick(s.units, c), ...pick(s.ctxUnits, c), leadOf(s, c)]) {
        if (!svg || !svg.querySelector(`[data-unit="${CSS.escape(id)}"]`)) miss.units.add(`${s.id}:${id}`);
      }
      for (const id of [...streamIds(pick(s.flows, c)), ...pick(s.ctx, c)]) {
        if (!svg || !svg.querySelector(`.pfd-streams > [data-stream="${CSS.escape(id)}"]`)) miss.streams.add(`${s.id}:${id}`);
      }
      for (const k of pick(s.marks, c)) if (!svg || !svg.querySelector(`[data-tour="${CSS.escape(k)}"]`)) miss.marks.add(`${s.id}:${k}`);
      for (const t of pick(s.text, c)) for (const m of String(t).matchAll(/\{(\w+)\}/g)) if (!s.nums[m[1]]) miss.keys.add(`${s.id}:${m[1]}`);
      for (const k of pick(s.figs, c)) if (!s.nums[k]) miss.keys.add(`${s.id}:figure ${k}`);
      if (!titleOf(s, c)) miss.keys.add(`${s.id}:title`);
    }
    for (const [k, spec] of Object.entries(s.nums)) {
      const v = res ? walk(res, spec.p) : undefined;
      const field = spec.p.replace(/\.value$/, "");
      if (typeof v === "number" && Number.isFinite(v)) continue;
      if (v === null && (listed.has(field) || listed.has(spec.p))) continue;
      badPaths.push(`${s.id}.${k}:${spec.p}`);
    }
  }
  return {
    steps: STEPS.length,
    ids: STEPS.map((s) => s.id),
    branches: contexts.length,
    missing_units: [...miss.units],
    missing_streams: [...miss.streams],
    missing_marks: [...miss.marks],
    bad_paths: badPaths,
    bad_keys: [...miss.keys],
    dwell_ms: STEPS.map((s) => s.ms),
    total_ms: STEPS.reduce((a, s) => a + s.ms, 0),
  };
}

/** Running state, for the self test. */
export function tourState() {
  const c = api ? context() : null;
  return {
    ready: Boolean(api && el.start && !el.start.hidden),
    active,
    step: idx,
    id: STEPS[idx].id,
    playing,
    ended,
    still,
    steps: STEPS.length,
    flows: c ? pick(STEPS[idx].flows, c).length : 0,
  };
}
