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
// (state.lastOk) by its reply path, so it follows the controls. Where the reply
// has no field, the number comes from a results row that scripts/build_site.py
// copies into data/tour_rows.json, and the caption marks it as precomputed.
// This file formats numbers (decimals, and a fraction shown as a percentage)
// and computes no engineering quantity. Pictures and reaction lines drawn
// without engine data carry an Illustration tag.
//
// Motion is CSS animation only. Under prefers-reduced-motion the tour shows
// still steps, moves nothing and does not advance by itself.

import { esc, walk, fillText } from "./panels.js?v=ce259b048e0badda63b4d4ef655f91f63b995804";
import { flagText } from "./app.js?v=ce259b048e0badda63b4d4ef655f91f63b995804";

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
// of a control, a fixed registry value, or a precomputed results row.
const KIND = { live: "Live", input: "Input", fixed: "Fixed value", pre: "Precomputed" };

/** A number read from the live reply at `p`. */
const N = (p, u, d, k, l) => ({ p, u, d, k, l });
/** A number read from data/tour_rows.json under `row`. */
const R = (row, u, d, l) => ({ row, u, d, k: "pre", l });

// Numbers several steps share.
const ROT = N("units.htgr.conditions.ROT.value", "C", 0, "fixed", "Reactor outlet");
const T_SUP = N("units.smr.conditions.T_HE_SUPPLY.value", "C", 0, "fixed", "Helium supply");
const RECYCLED = N("units.cryogenic.results.recycled_c.value", "kmol/h", 0, "live", "Carbon recycled to the reformer");
const Q_REB = N("kpi.q_reboiler_mw", "MW", 1, "live", "Reboiler duty");
const TAIL_HEAT = N("balances.energy.tail_gas_to_reboiler.value", "MW", 1, "live", "Tail gas heat to the reboiler");

// Lines that carry nothing when the PSA reject leaves the plant or is burned
// on site, since the drawing shows the reject recycled.
const RECYCLE_LINES = ["S14", "R1", "S21", "S03", "S15", "S16"];
const rejectLeaves = (c) => c.x.tail_gas === "export" || c.x.tail_gas === "burn_on_site";

// The steps. `units` are lit, `flows` are groups of streams animated in order
// (a group's lines move together), `ctx` and `ctxUnits` are lines and units
// kept half lit, `marks` are data-tour keys of labels and frame elements to
// keep lit, and `lead` is the unit whose panel opens beside the card. `ms` is
// the dwell when playing. Text placeholders {key} name entries of `nums`.
const STEPS = [
  {
    id: "fission",
    title: "Fission heat in the HTGR core",
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
      "Inside the core, uranium nuclei in the TRISO fuel particles split when they absorb a neutron, and the graphite slows the new neutrons so the chain reaction carries on.",
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
      he: N("units.circ_primary.inputs.he_prim.value", "kg/s", 1, "live", "Primary helium flow"),
      e: N("units.circ_primary.results.e_prim.value", "MWe", 2, "live", "Primary circulator"),
      app: N("units.ihx.conditions.IHX_APPROACH_K.value", "K", 0, "input", "IHX approach"),
    },
    figs: ["ihx", "he", "e", "app"],
    text: () => [
      "Hot primary helium at {rot} passes {ihx} to a separate secondary loop in the intermediate heat exchanger (IHX), so the reactor coolant never enters the chemical plant.",
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
      q: N("kpi.q_process_mwth", "MWth", 1, "live", "Process heat"),
      he: N("units.circ_primary.inputs.he_sec.value", "kg/s", 1, "live", "Secondary helium flow"),
      e: N("units.circ_primary.results.e_sec.value", "MWe", 2, "live", "Secondary circulator"),
    },
    figs: ["sup", "ret", "q", "he"],
    text: () => [
      "The secondary helium leaves the IHX at {sup}, heats the reformer tubes and returns at {ret}, bringing {q} of process heat that a conventional reformer would raise by burning natural gas.",
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
      tube: R("tube_exit", "C", 1, "Exit reachable on the assumed tube"),
    },
    figs: ["ng", "out", "tube"],
    text: (c) => [
      `Natural gas at {ng} and process steam enter the reformer${c.q(RECYCLED.p) > 0 ? ", joined by the methane-rich recycle of step 9" : ""}. In the catalyst tubes methane and steam react to hydrogen and carbon monoxide, taking up the helium's heat.`,
      c.x.basis === "case_a"
        ? "The engine sets the exit at {out}, INL's equilibrium assumption; the tube check that tests it ran on the Case B feed."
        : "The engine sets the exit at {out}, INL's equilibrium assumption; with helium at {sup}, the tube model reaches only {tube} on the assumed tube.",
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
    figs: ["ex4", "sep", "cond"],
    text: (c) => [
      "The shifted gas is cooled in steps, and {ex4} of its heat is hot enough to help drive the MEA reboiler of step 10.",
      c.q("units.separator.results.condensate.value") !== null
        ? "In the separator at {sep}, {cond} of water condenses out before the gas goes on to the absorber."
        : "In the separator at {sep} the condensed water drains out before the gas goes on to the absorber.",
    ],
    notes: () => ["The engine models six exchangers where the drawing shows one cooler."],
  },
  {
    id: "absorber",
    title: "CO₂ capture in the MEA absorber",
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
    text: (c) => (c.x.tail_gas === "inl_selexol"
      ? ["In this scenario a physical solvent, Selexol, takes up {pp} of the CO₂ in one pass, {cap}. The drawing still shows the MEA plant, so its solvent lines stand in for the Selexol unit."]
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
    title: "PSA reject back to the cryogenic unit",
    ms: 8000,
    units: ["recycle_compressor", "cryogenic"],
    marks: (c) => (rejectLeaves(c) ? [] : ["stack", "rej", "recycle", "h2ret"]),
    flows: (c) => (rejectLeaves(c) ? [] : [
      ["S14"],
      ["R1"],
      ["S21", "S03", "S15",
        { m: "h2", d: "M1345,190 L1345,150 L1480,150 L1480,270 L1700,270 L1700,226" },
        { m: "rej", d: "M700,235 L700,330" }],
      ["S16"],
    ]),
    ctx: (c) => (rejectLeaves(c) ? [] : ["S12", "S01"]),
    lead: "cryogenic",
    ctxUnits: (c) => (rejectLeaves(c) ? ["psa"] : ["psa", "smr"]),
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
      if (c.x.tail_gas === "export") {
        return [
          "In this scenario the PSA reject, holding {h2} of hydrogen, is exported across the boundary, so the drawn recycle and stack carry nothing.",
          "The engine counts {atm} of CO₂ to the atmosphere.",
        ];
      }
      if (c.x.tail_gas === "burn_on_site") {
        return [
          "In this scenario the PSA reject, holding {h2} of hydrogen, is burned on site and gives the reboiler {tail}; the drawn recycle carries nothing.",
          "The engine counts {atm} of CO₂ to the atmosphere.",
        ];
      }
      return [
        "The PSA reject, still holding {h2} of hydrogen, crosses back into the boundary and is compressed for the cryogenic unit.",
        "The cold separation returns hydrogen to the PSA feed and a methane-rich recycle to the reformer. The nitrogen-rich cut is burned and leaves by the stack as the plant's residual {atm} of CO₂.",
      ];
    },
    notes: (c) => [c.limit("cryogenic", 1), "The burner of the nitrogen-rich cut is not drawn; the stack stands for it."],
  },
  {
    id: "stripper",
    title: "Stripper and reboiler heat",
    ms: 8000,
    units: ["rich_lean_hx", "stripper", "reboiler", "lean_cooler", "lean_pump"],
    marks: ["rich"],
    flows: (c) => (c.q(Q_REB.p) > 0 ? [["S17"], ["L3"], ["L5", "L6"], ["L4"], ["S18"]] : []),
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
    figs: ["q", "srd", "gap", "ps"],
    text: (c) => {
      if (!(c.q(Q_REB.p) > 0)) {
        return ["In this scenario Selexol takes the place of the MEA plant, and the reboiler duty is {q}."];
      }
      const tail = c.q(TAIL_HEAT.p) > 0 ? ", tail gas burned on site {tail}" : "";
      return [
        "Rich solvent is warmed in the rich/lean exchanger and stripped at {ps} by reboiler steam, which drives the CO₂ out; the lean solvent is cooled and pumped back to the absorber.",
        `The reboiler needs {q}, met in a fixed order: freed Selexol steam {sel}, export steam {exp}, syngas heat {ex4}${tail}, and reactor steam {gap} last.`,
      ];
    },
    notes: () => ["No line on the drawing brings heat into the reboiler; the order comes from the engine's energy balance."],
  },
  {
    id: "co2",
    title: "CO₂ compression, ship and offshore storage",
    ms: 8000,
    units: ["condenser", "co2_compressor", "co2_storage"],
    marks: ["co2"],
    flows: [["C1"], ["S19"], ["S20"]],
    lead: "co2_storage",
    ctxUnits: ["stripper"],
    inset: "ship",
    nums: {
      p: R("s20_p_bar", "bar", 1, "Compressed CO₂ pressure"),
      e: N("units.co2_compressor.results.e_co2.value", "MWe", 2, "live", "Compression"),
      cap: N("kpi.co2_captured_t_y", "t/y", 0, "live", "CO₂ shipped to storage"),
      ts: N("inputs.ts_s_t", "S$/t", 0, "input", "Transport and storage tariff"),
      cost: N("units.co2_storage.results.ts_cost.value", "S$M/y", 1, "live", "Transport and storage charge"),
    },
    figs: ["cap", "e", "p", "cost"],
    text: () => [
      "The condenser returns water from the stripper overhead as reflux, and the CO₂ is compressed to {p} using {e}.",
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
    flows: [["P1"], ["P4"]],
    lead: "power",
    feas: true,
    nums: {
      br: N("units.power.inputs.branch.value", "MWth", 1, "live", "Helium branch to the power block"),
      en: N("units.power.outputs.e_nuc.value", "MWe", 1, "live", "Electricity generated"),
      eta: N("inputs.eta_power", "-", 1, "input", "Conversion efficiency"),
      gap: N("units.power.results.gap.value", "MW", 1, "live", "Reactor steam to the reboiler"),
      ec: N("balances.power.e_circ.value", "MWe", 2, "live", "Helium circulators"),
      et: N("units.power.results.e_total.value", "MWe", 2, "live", "Site electricity demand"),
      imp: N("kpi.unmet_power_mwe", "MWe", 2, "live", "Grid import"),
      tg: R("tail_gas_power", "MWe", 1, "Electricity from tail gas burned on site"),
    },
    figs: ["br", "en", "et", "imp"],
    text: (c) => {
      const last = "The helium circulators draw {ec} of the {et} site demand, and the grid supplies {imp}.";
      if (c.x.tail_gas === "burn_on_site") {
        return [
          c.x.basis === "case_a"
            ? "In this scenario the helium branch carries {br}, the turbine makes {en}, and the site runs on the tail gas burned on site."
            : "In this scenario the helium branch carries {br}, the turbine makes {en}, and the site runs on the tail gas burned on site, which yields {tg} in the precomputed Case B run.",
          last,
        ];
      }
      return [
        "A branch of the hot primary helium carries {br} to the power block, whose turbine makes {en} at {eta} efficiency and sends {gap} of steam heat to the reboiler.",
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
    flows: (c) => [allStreams().filter((id) => !(rejectLeaves(c) && RECYCLE_LINES.includes(id)))],
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
      "Against the {base} an unabated reformer would emit on INL's baseline, its residual of {atm} leaves a net abatement of {ab}. Part of that is the CO₂ stored; the rest is never formed, since no natural gas is burned for reformer heat.",
      `${c.met ? "The abatement target of {tgt} is met at these inputs." : "The abatement target of {tgt} is not met at these inputs."} On Cho's and IEAGHG's baselines the net abatement is {cho} and {ie}.`,
    ],
    status: (c) => {
      const t = c.r && c.r.targets && c.r.targets.cost;
      if (!t) return "";
      return t.met ? "Cost target: met at these inputs." : `Cost target: ${t.reason || "not met at these inputs"}.`;
    },
    notes: () => ["The plant sits on the proposed Western Island site, a proposal of this study with no official approval."],
  },
];

// ------------------------------------------------------------------ state

let api = null;
const el = {};
let rows = null;           // Map of tour row key to row, once data/tour_rows.json loads
let rowsError = "";
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
    met: Boolean(r && r.targets && r.targets.abatement && r.targets.abatement.met),
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

function valueOf(spec, c) {
  if (spec.row) {
    const r = rows ? rows.get(spec.row) : null;
    return r && typeof r.value === "number" ? r.value : null;
  }
  return c.q(spec.p);
}

/** One number as HTML; the data attributes let the self test check it against its source. */
function numHtml(spec, c, inText) {
  if (!spec) return "n/a";
  const v = valueOf(spec, c);
  const text = fmtNum(v, spec.u, spec.d);
  const src = spec.row ? `data-row="${esc(spec.row)}"` : `data-path="${esc(spec.p)}"`;
  const u = unitText(spec.u);
  const unit = text === null ? "" : `<span class="tu${u === "%" ? " pct" : ""}">${esc(u)}</span>`;
  const pre = spec.row && inText && text !== null ? `<span class="tpre"> (precomputed)</span>` : "";
  return `<b class="tn" ${src} data-u="${esc(spec.u)}" data-d="${spec.d}">${esc(text === null ? "n/a" : text)}</b>${unit}${pre}`;
}

function fill(tpl, s, c) {
  return String(tpl).split(/(\{\w+\})/).map((part) => {
    const m = /^\{(\w+)\}$/.exec(part);
    return m ? numHtml(s.nums[m[1]], c, true) : esc(part);
  }).join("");
}

/** Reasons the engine gives for numbers of this step it left null. */
function missingNotes(s, c, keys) {
  const list = (c.r && Array.isArray(c.r.engine_fields_missing)) ? c.r.engine_fields_missing : [];
  const out = [];
  for (const k of keys) {
    const spec = s.nums[k];
    if (!spec || valueOf(spec, c) !== null) continue;
    if (spec.row) {
      out.push(`${spec.l}: n/a, ${rowsError || "the precomputed row is loading"}.`);
      continue;
    }
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
    label: "Illustration: a ship carries the compressed CO₂ from the plant to an offshore store, where it goes into a rock formation deep under the seabed.",
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
  for (const e of el.svg.querySelectorAll(".tour-on, .tour-ctx, .tour-move")) e.classList.remove("tour-on", "tour-ctx", "tour-move");
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
  for (const id of s.units) light(`[data-unit="${CSS.escape(id)}"]`, "tour-on");
  for (const id of pick(s.ctx, c)) light(`[data-stream="${CSS.escape(id)}"]`, "tour-ctx");
  for (const id of pick(s.ctxUnits, c)) light(`[data-unit="${CSS.escape(id)}"]`, "tour-ctx");
  for (const id of streamIds(pick(s.flows, c))) {
    light(`[data-stream="${CSS.escape(id)}"]`, "tour-on");
    if (!s.all) light(`[data-stream="${CSS.escape(id)}"]`, "tour-move");
  }
  for (const k of pick(s.marks, c)) light(`[data-tour="${CSS.escape(k)}"]`, "tour-on");
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
  el.title.textContent = s.title;
  const figs = pick(s.figs, c);
  const eqs = pick(s.eq, c);
  const stale = api.state.result && api.state.result.status === "not_converged";
  const leadStatus = c.r && c.r.units && c.r.units[s.lead] ? c.r.units[s.lead].status : null;
  const usesRow = figs.some((k) => s.nums[k] && s.nums[k].row);

  const tags = [c.r ? tagHtml("live", "Live engine values") : tagHtml("wait", "Waiting for the engine")];
  if (stale) tags.push(tagHtml("stale", "Last converged result"));
  if (s.inset || eqs.length) tags.push(tagHtml("illustration", "Includes an illustration"));
  if (usesRow) tags.push(tagHtml("pre", "Some values precomputed"));
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
      return `<div data-fig="${esc(k)}"><dt>${esc(spec.l)}</dt><dd>${numHtml(spec, c, false)}<span class="tk" data-kind="${esc(spec.k)}">${esc(KIND[spec.k] || spec.k)}</span></dd></div>`;
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
    s.inset ? insetHtml(s.inset) : "",
    mediaHtml,
    `<div class="tour-text">${text}</div>`,
    eqHtml,
    figHtml,
    status ? `<p class="tour-status">${esc(status)}</p>` : "",
    notes.length ? `<ul class="tour-notes">${notes.map((x) => `<li>${esc(x)}</li>`).join("")}</ul>` : "",
    s.all || pick(s.flows, c).length ? `<p class="tour-motion">${esc(motion)}</p>` : "",
  ].join("");
  el.card.classList.toggle("is-still", still);
  el.card.classList.toggle("is-paused", !still && !playing);
}

function announce() {
  const s = STEPS[idx];
  const text = Array.from(el.body.querySelectorAll(".tour-warn, .tour-text p")).map((p) => p.textContent.trim()).join(" ");
  el.live.textContent = `Step ${idx + 1} of ${n()}. ${s.title}. ${text}`;
}

// ------------------------------------------------------------------ controls

function updateBar() {
  const s = STEPS[idx];
  el.count.textContent = `Step ${idx + 1} of ${n()}`;
  el.track.setAttribute("aria-valuemin", "1");
  el.track.setAttribute("aria-valuemax", String(n()));
  el.track.setAttribute("aria-valuenow", String(idx + 1));
  el.track.setAttribute("aria-valuetext", `Step ${idx + 1} of ${n()}: ${s.title}`);
  el.prev.setAttribute("aria-disabled", String(idx === 0));
  el.next.setAttribute("aria-disabled", String(idx === n() - 1));
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
      go(idx + 1);
    } else {
      playing = false;
      ended = true;
      updateBar();
    }
  };
  if (!playing) stepAnim.pause();
}

function go(i) {
  if (!active) return;
  idx = Math.max(0, Math.min(n() - 1, i));
  ended = false;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  if (s.lead && api.select) api.select(s.lead);
  renderCard();
  updateBar();
  armTimer();
  announce();
  frame();
}

function pause() {
  if (!playing) return;
  playing = false;
  if (stepAnim) stepAnim.pause();
  updateBar();
}

function resume() {
  if (still) return;
  if (ended) {
    playing = true;
    go(0);
    return;
  }
  playing = true;
  if (stepAnim) stepAnim.play();
  else armTimer();
  updateBar();
}

function toggle() {
  if (playing) pause();
  else resume();
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
  go(typeof opts.step === "number" ? opts.step : 0);
  el.title.focus({ preventScroll: true });
  const top = el.bar.getBoundingClientRect().top;
  if (top < 0 || top > window.innerHeight * 0.4) el.bar.scrollIntoView({ block: "start", behavior: still ? "auto" : "smooth" });
  return true;
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

export function goTour(i) {
  go(i);
}

/** Re-render the numbers of the running step after a new reply. */
export function refreshTour() {
  if (!active) return;
  const s = STEPS[idx];
  const c = context();
  applyMarks(s, c);
  buildFlows(s, c);
  renderCard();
}

function onBar(e) {
  const b = e.target.closest("[data-act]");
  if (!b || !active || b.getAttribute("aria-disabled") === "true") return;
  const act = b.dataset.act;
  if (act === "prev") go(idx - 1);
  else if (act === "next") go(idx + 1);
  else if (act === "play") toggle();
  else if (act === "exit") exitTour(true);
}

function onKey(e) {
  if (!active || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
  const t = e.target && e.target.closest ? e.target : null;
  const inDrawing = t && t.closest("[data-unit], [data-stream], .pfd-tip");
  const formish = t && t.closest("input, select, textarea, [contenteditable=\"true\"]");
  if (inDrawing || formish) return;
  if (e.key === "ArrowRight") {
    e.preventDefault();
    go(idx + 1);
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    go(idx - 1);
  } else if (e.key === " " || e.key === "Spacebar") {
    if (t && t.closest("button, a[href], summary")) return;
    e.preventDefault();
    toggle();
  }
}

function onEscape(e) {
  if (!active || e.key !== "Escape") return;
  // An open stream tip closes first; pfd.js handles that press.
  if (api.tip && !api.tip.hidden) return;
  e.preventDefault();
  exitTour(true);
}

async function loadRows() {
  try {
    const res = await fetch(new URL("data/tour_rows.json?v=ce259b048e0badda63b4d4ef655f91f63b995804", import.meta.url));
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    rows = new Map(Object.entries(data.rows || {}));
  } catch (err) {
    rows = new Map();
    rowsError = `data/tour_rows.json did not load (${err && err.message ? err.message : err})`;
  }
  refreshTour();
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
  el.kicker = document.getElementById("tour-kicker");
  el.title = document.getElementById("tour-title");
  el.body = document.getElementById("tour-body");
  el.live = document.getElementById("tour-live");
  el.count = document.getElementById("tour-count");
  el.track = document.getElementById("tour-track");
  el.fill = document.getElementById("tour-fill");
  el.route = document.getElementById("route-pfd");
  if (!el.start || !el.bar || !el.card || !el.svg) return;
  el.prev = el.bar.querySelector('[data-act="prev"]');
  el.next = el.bar.querySelector('[data-act="next"]');
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
  document.addEventListener("keydown", onKey);
  document.addEventListener("keydown", onEscape, true);
  // Reading the drawing pauses the tour, so a panel opened by a click stays put.
  el.svg.addEventListener("click", () => { if (active && playing) pause(); }, true);
  window.addEventListener("hashchange", () => { if (active && location.hash !== "#pfd") exitTour(false); });
  startNote();
  el.start.hidden = false;
  el.note.hidden = false;
  loadRows();
}

// ------------------------------------------------------------------ self test

/** Ids, keys and reply paths the steps name, checked against the page and a reply. */
export function tourAudit(res) {
  const svg = el.svg;
  const missingUnits = [];
  const missingStreams = [];
  const missingMarks = [];
  const badPaths = [];
  const missingRows = [];
  const badKeys = [];
  const listed =new Set(((res && res.engine_fields_missing) || []).map((m) => m.field));
  const c = { x: (res && res.inputs) || {}, q: () => 1, r: res, flag: () => "", limit: () => "", feasible: true, met: true };
  for (const s of STEPS) {
    for (const id of [...s.units, ...pick(s.ctxUnits, c), s.lead]) {
      if (!svg || !svg.querySelector(`[data-unit="${CSS.escape(id)}"]`)) missingUnits.push(`${s.id}:${id}`);
    }
    for (const id of [...streamIds(pick(s.flows, c)), ...pick(s.ctx, c)]) {
      if (!svg || !svg.querySelector(`.pfd-streams > [data-stream="${CSS.escape(id)}"]`)) missingStreams.push(`${s.id}:${id}`);
    }
    for (const k of pick(s.marks, c)) if (!svg || !svg.querySelector(`[data-tour="${CSS.escape(k)}"]`)) missingMarks.push(`${s.id}:${k}`);
    for (const t of pick(s.text, c)) for (const m of String(t).matchAll(/\{(\w+)\}/g)) if (!s.nums[m[1]]) badKeys.push(`${s.id}:${m[1]}`);
    for (const k of pick(s.figs, c)) if (!s.nums[k]) badKeys.push(`${s.id}:figure ${k}`);
    for (const [k, spec] of Object.entries(s.nums)) {
      if (spec.row) {
        if (!rows || !rows.has(spec.row)) missingRows.push(`${s.id}.${k}:${spec.row}`);
        continue;
      }
      const v = res ? walk(res, spec.p) : undefined;
      const field = spec.p.replace(/\.value$/, "");
      if (typeof v === "number" && Number.isFinite(v)) continue;
      if (v === null && (listed.has(field) || listed.has(spec.p))) continue;
      badPaths.push(`${s.id}.${k}:${spec.p}`);
    }
  }
  return {
    steps: STEPS.length,
    ready: Boolean(rows),
    missing_units: missingUnits,
    missing_streams: missingStreams,
    missing_marks: missingMarks,
    bad_paths: badPaths,
    missing_rows: missingRows,
    bad_keys: badKeys,
    dwell_ms: STEPS.map((s) => s.ms),
    total_ms: STEPS.reduce((a, s) => a + s.ms, 0),
  };
}

/** Running state, for the self test. */
export function tourState() {
  const c = api ? context() : null;
  return {
    ready: Boolean(api && rows && el.start && !el.start.hidden),
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
