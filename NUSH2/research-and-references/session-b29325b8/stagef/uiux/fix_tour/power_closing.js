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
    flows: (c) => [allStreams().filter((id) => !idle(c).includes(id))],
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
