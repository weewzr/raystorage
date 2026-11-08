// Dashboard, Results, Export and Validation renderers, and the print summary.
// Every engineering number shown here is read from the engine reply or from
// the CI files under data/ (JSON built from results/*.csv, and the CSV copies
// themselves). This file chooses, orders and formats values; it adds no model
// term. Where the engine has no value the page prints n/a with the engine's
// reason from engine_fields_missing.

import { state, registerRoute, render, fmt, unitLabel, esc, rpc, pretty, flagText, caseText, setInputs, defaultOf, gasText } from "./app.js?v=local";
import { frame, unavailable, barChart, stackedBars, waterfall, rangeBars, dumbbell, lineChart, hideTip } from "./charts.js?v=local";
import { renderParameters } from "./controls.js?v=local";

const $ = (s, r = document) => r.querySelector(s);
const finite = (v) => typeof v === "number" && Number.isFinite(v);
const TICK = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8.5l3 3 7-7" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
const CROSS = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
const DOT = '<svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="5" fill="currentColor"/></svg>';
const RING = '<svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="5" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
const STATUS = {
  pass: { label: "Passed", icon: TICK },
  fail: { label: "Failed", icon: CROSS },
  not_run: { label: "Not run", icon: RING },
  reported: { label: "Reported, no literature value", icon: DOT },
};
// Row status words of the results files, in the words the engine export gives
// each one as the source of its rows.
const ROW_STATUS = {
  input: "Input",
  anchored_port: "Anchored port",
  flowsheet_m7: "Level 3 flowsheet",
  flowsheet_m8: "Sweep on the Level 3 flowsheet",
  level4_m8: "Level 4 heat recovery and utility",
  model_m4: "Gas unit model",
  model_m5: "Reformer tube model",
  model_m6: "Column model",
  not_run: "Not run",
  not_converged: "Did not converge",
};
function rowStatus(s) {
  if (ROW_STATUS[s]) return ROW_STATUS[s];
  const g = /^gated_(T\d\d(?:_T\d\d)*)$/.exec(String(s));
  return g ? `Anchored port with modelled term ${g[1].replace(/_/g, " and ")}` : String(s);
}
const COST_METRICS = [
  ["ac_l5", "Full cost, L5 ledger", "S$/t"],
  ["lac_dcf", "Full cost, discounted cash flow", "S$/t"],
  ["lcoh_nush2", "LCOH of NUS-H₂", "S$/kg"],
];
const CSV_FILES = [
  "Final_Design_Basis.csv",
  "Integrated_Mass_Balance.csv",
  "Integrated_Energy_Balance.csv",
  "Emissions_Ledger.csv",
  "Economic_Ledger.csv",
  "Sensitivity_Results.csv",
  "Validation_Results.csv",
];
const HEADLINE_RUN = "B_REC_F";
const FS_RUN = "FS_B_REC_F";

// ------------------------------------------------------------ small helpers

function num(v, d) {
  return finite(v) ? v.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d }) : "n/a";
}

function sig(v, s = 6) {
  if (v === null || v === undefined || v === "") return "n/a";
  if (typeof v !== "number") return String(v);
  if (!Number.isFinite(v)) return "n/a";
  if (v !== 0 && Math.abs(v) < 1e-4) return v.toExponential(2);
  return v.toLocaleString("en-GB", { maximumSignificantDigits: s });
}

/** Engine value with its unit, as the shell formats KPIs. */
function vu(v, unit) {
  return finite(v) ? `${fmt(v, unit)} ${unitLabel(unit)}` : "n/a";
}

/** Signed display of an engine value. Formatting only. */
function signed(v, unit) {
  if (!finite(v)) return "n/a";
  return `${v > 0 ? "+" : ""}${fmt(v, unit)}`;
}

function kpiDef(st, id) {
  return st.registry ? st.registry.kpis.find((k) => k.id === id) : null;
}

function paramRec(st, id) {
  return st.registry ? st.registry.params.find((p) => p.id === id) : null;
}

function missingReason(res, field) {
  const list = res && Array.isArray(res.engine_fields_missing) ? res.engine_fields_missing : [];
  const m = list.find((x) => x.field === field);
  return m ? m.reason : "";
}

/** One item of a unit section, by unit id and item id, across the four lists. */
function unitItem(res, unit, id) {
  const u = res && res.units ? res.units[unit] : null;
  if (!u) return null;
  for (const sec of ["inputs", "outputs", "conditions", "results"]) {
    const e = (u[sec] || []).find((x) => x.id === id);
    if (e) return e;
  }
  return null;
}

function balance(res, sec, id) {
  const list = res && res.balances && Array.isArray(res.balances[sec]) ? res.balances[sec] : [];
  return list.find((x) => x.id === id) || null;
}

function commit7(st) {
  const c = st.registry ? String(st.registry.engine.commit || "local") : "local";
  return c.slice(0, 7);
}

function section(mount, id, title, tag, lede) {
  const sec = document.createElement("section");
  sec.className = "view-section";
  sec.id = id;
  sec.innerHTML = `<h3>${esc(title)}${tag ? ` <span class="tag" data-tag="${esc(tag.toLowerCase())}">${esc(tag)}</span>` : ""}</h3>${lede ? `<p class="lede">${lede}</p>` : ""}`;
  mount.appendChild(sec);
  return sec;
}

function guard(mount, where, fn) {
  try {
    fn();
  } catch (err) {
    console.error(`${where}: ${err && err.stack ? err.stack : err}`);
    const p = document.createElement("p");
    p.className = "banner";
    p.dataset.level = "bad";
    p.textContent = `${where} could not be drawn: ${err && err.message ? err.message : err}`;
    mount.appendChild(p);
  }
}

/** Open disclosure blocks survive a re-render through their data-keep name. */
function openDetails(mount) {
  return new Set(Array.from(mount.querySelectorAll("details[data-keep]")).filter((d) => d.open).map((d) => d.dataset.keep));
}

function restoreDetails(mount, keep) {
  mount.querySelectorAll("details[data-keep]").forEach((d) => { if (keep.has(d.dataset.keep)) d.open = true; });
}

function changedInputs(st) {
  if (!st.registry) return [];
  return st.registry.params.filter((p) => st.inputs[p.id] !== undefined && String(st.inputs[p.id]) !== String(p.default)).map((p) => p);
}

function inputText(st, p) {
  const v = st.inputs[p.id];
  if (p.kind === "enum") {
    const o = (p.options || []).find((x) => x.value === v);
    return o ? o.label : String(v);
  }
  return `${v}${p.unit && p.unit !== "-" ? ` ${p.unit}` : ""}`;
}

// ------------------------------------------------------------ CI data files

const files = {};
let rerenderQueued = false;

function rerender() {
  if (rerenderQueued) return;
  rerenderQueued = true;
  setTimeout(() => { rerenderQueued = false; render(); }, 0);
}

/** Load data/<name> once. Returns {status: loading|ok|missing, value, error}. */
function dataFile(name, kind = "json") {
  const key = `${kind}:${name}`;
  let f = files[key];
  if (!f) {
    f = { status: "loading", value: null, error: "" };
    files[key] = f;
    const url = kind === "csv" ? `data/csv/${name}?v=local` : `data/${name}?v=local`;
    fetch(url)
      .then(async (r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        f.value = kind === "csv" ? parseCsv(await r.text()) : await r.json();
        f.status = "ok";
      })
      .catch((e) => {
        f.status = "missing";
        f.error = `data/${kind === "csv" ? "csv/" : ""}${name} is not available here (${e && e.message ? e.message : e}). CI writes it with scripts/build_site.py`;
      })
      .finally(rerender);
  }
  return f;
}

/** RFC 4180 reader for the results CSV copies. Cells stay text. */
function parseCsv(text) {
  const rows = [];
  let row = [];
  let cell = "";
  let q = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (q) {
      if (c === '"') {
        if (text[i + 1] === '"') { cell += '"'; i += 1; } else q = false;
      } else cell += c;
    } else if (c === '"') q = true;
    else if (c === ",") { row.push(cell); cell = ""; }
    else if (c === "\n") { row.push(cell); rows.push(row); row = []; cell = ""; }
    else if (c !== "\r") cell += c;
  }
  if (cell !== "" || row.length) { row.push(cell); rows.push(row); }
  const head = rows.shift() || [];
  return rows.filter((r) => r.length === head.length).map((r) => Object.fromEntries(head.map((h, i) => [h, r[i]])));
}

/** A CSV cell read as a number for display; empty stays null. */
function cell(v) {
  if (v === undefined || v === null || v === "") return null;
  const n = Number(v);
  return Number.isFinite(n) ? n : null;
}

function waitNote(f, what) {
  return f.status === "loading" ? `<p class="note">Loading ${esc(what)}.</p>` : `<p class="note"><strong>n/a.</strong> ${esc(f.error)}.</p>`;
}

// ------------------------------------------------------------ cost tables

function nOptions(st) {
  const p = paramRec(st, "n");
  return p && p.options ? p.options : [];
}

const matchN = (n, opt) => finite(n) && Math.abs(n - Number(opt.value)) < 0.006;

function headlineRows(res, charged) {
  const list = res && res.cost && Array.isArray(res.cost.headline) ? res.cost.headline : [];
  return list.filter((r) => r.charged === charged);
}

function pickRow(rows, stage, life, opt) {
  return rows.find((r) => r.stage === stage && Number(r.life_y) === life && matchN(r.n, opt)) || null;
}

/**
 * Stage and life columns of the cost headline: the cells the native export
 * printed to results/Final_Design_Basis.csv, read from data/reference.json in
 * file order. Returns null until that file has loaded.
 */
function headlineCols() {
  const f = dataFile("reference.json");
  if (f.status !== "ok") return null;
  const seen = new Set();
  const out = [];
  for (const h of f.value.cost_headline || []) {
    const key = `${h.stage}|${h.life_y}`;
    if (!seen.has(key)) { seen.add(key); out.push([String(h.stage), Number(h.life_y)]); }
  }
  return out;
}

function costTable(st, res, charged, caption) {
  const STAGES = headlineCols();
  if (!STAGES) return waitNote(dataFile("reference.json"), "the headline cell list");
  const rows = headlineRows(res, charged);
  const opts = nOptions(st);
  const isSel = (stage, life, opt) => charged === st.inputs.charged && stage === st.inputs.stage && String(life) === String(st.inputs.life_y) && opt.value === st.inputs.n;
  let body = "";
  for (const opt of opts) {
    body += `<tr class="group"><th scope="colgroup" colspan="5">Size exponent n = ${esc(opt.label)}</th></tr>`;
    for (const [key, label, unit] of COST_METRICS) {
      body += `<tr><th scope="row">${esc(label)}, ${esc(unit)}</th>${STAGES.map(([s, l]) => {
        const r = pickRow(rows, s, l, opt);
        const sel = isSel(s, l, opt);
        return `<td class="num${sel ? " sel" : ""}"${sel ? ' title="Selected in Parameters"' : ""}>${fmt(r ? r[key] : null, unit)}</td>`;
      }).join("")}</tr>`;
    }
  }
  if (opts.length) {
    body += `<tr class="group"><th scope="colgroup" colspan="5">Unabated SMR of the same hydrogen output</th></tr><tr><th scope="row">LCOH of the unabated SMR, S$/kg</th>${STAGES.map(([s, l]) => {
      const r = pickRow(rows, s, l, opts[0]);
      return `<td class="num">${fmt(r ? r.lcoh_smr : null, "S$/kg")}</td>`;
    }).join("")}</tr>`;
  }
  return `<div class="table-scroll"><table class="data cost-table" data-charged="${esc(charged)}">
    <caption>${esc(caption)}</caption>
    <thead><tr><th scope="col">Quantity</th>${STAGES.map(([s, l]) => `<th scope="col" class="num">${s}, ${l} y</th>`).join("")}</tr></thead>
    <tbody>${body}</tbody></table></div>`;
}

/**
 * The table line reads the engine's evaluated case list, targets.cost.cases,
 * the same list the dashboard strip reads: a case counts as met only when the
 * engine says so, which needs the plant feasible at the rating. When the
 * selected case is not a cell of the table, the line gives its verdict too, so
 * the strip and the table never disagree.
 */
function targetLine(st, res) {
  const c = res.targets && res.targets.cost ? res.targets.cost : null;
  const t = c ? c.target : null;
  if (!finite(t) || !Array.isArray(c.cases)) return "";
  const feas = res.feasibility || {};
  const table = c.cases.filter((x) => x.table);
  const met = table.filter((x) => x.met);
  const sel = c.cases.find((x) => x.selected) || null;
  const selNote = sel && !sel.table
    ? ` The selected case, ${caseText(sel)}, is not a column of this table: L5 ${fmt(sel.ac_l5, "S$/t")} and DCF ${fmt(sel.lac_dcf, "S$/t")} S$/t, target ${sel.met ? "met" : "not met"}.`
    : "";
  if (!met.length) {
    const why = feas.feasible === false
      ? "The plant cannot run at the rating, as the line above the tiles states, so no case meets it."
      : "No case in this table sits at or below it on both the L5 ledger and the discounted cash flow.";
    return `<p class="target-line" data-met="false" data-cost-target="missed">${CROSS}<span>Target S$${num(t, 0)}/t: missed. ${esc(why)}${esc(selNote)}</span></p>`;
  }
  const changed = changedInputs(st).filter((p) => !["stage", "life_y", "n", "charged"].includes(p.id));
  const why = changed.length ? ` Inputs changed from the headline basis: ${changed.map((p) => `${p.label} ${inputText(st, p)}`).join("; ")}.` : "";
  return `<p class="target-line" data-met="true" data-cost-target="met">${TICK}<span>Target S$${num(t, 0)}/t: met on both methods at ${esc(met.map(caseText).join("; "))}.${esc(why)}${esc(selNote)}</span></p>`;
}

// ------------------------------------------------------------ dashboard

function balTile(o) {
  return `<div class="bal" data-state="${o.state}" data-tile="${esc(o.key)}">
    <span class="b-label">${esc(o.label)}</span>
    <span class="b-value">${esc(o.value)}${o.unit ? `<span class="b-unit">${esc(o.unit)}</span>` : ""}</span>
    ${o.stateText ? `<span class="b-state">${o.state === "ok" ? TICK : o.state === "na" ? RING : CROSS}${esc(o.stateText)}</span>` : ""}
    ${o.note ? `<span class="reason">${esc(o.note)}</span>` : ""}
  </div>`;
}

function surplusTile(key, label, v, note) {
  if (!finite(v)) return balTile({ key, label, value: "n/a", state: "na", stateText: "", note });
  if (v < 0) return balTile({ key, label: label.replace("surplus or deficit", "deficit"), value: signed(v, "MWth"), unit: "MWth", state: "deficit", stateText: "Deficit: the process needs more heat than the rating", note });
  return balTile({ key, label: label.replace("surplus or deficit", "surplus"), value: signed(v, "MWth"), unit: "MWth", state: "ok", stateText: "Surplus, held as margin with no revenue", note });
}

function unmetTile(res, key, label, v, unit, field, coverText) {
  if (!finite(v)) return balTile({ key, label, value: "n/a", state: "na", stateText: "No engine field", note: missingReason(res, field) });
  if (v > 0) return balTile({ key, label, value: fmt(v, unit), unit, state: "unmet", stateText: coverText });
  return balTile({ key, label, value: fmt(v, unit), unit, state: "ok", stateText: "None at these inputs" });
}

function dashCost(mount, st, res) {
  const rows = headlineRows(res, "rating");
  const ch = rows.length ? rows[0].charged_mwth : null;
  const sec = section(mount, "dash-cost", "Rebuilt full cost of abatement", "Live",
    `Every real plant cost is in. Two independent methods are printed side by side: the annual L5 ledger and a discounted cash flow at the discount rate set in Parameters. The reactor is charged at ${esc(vu(ch, "MWth"))}, the rating. All three size exponents appear at the same size. The <a href="#results">ladder on the Results page</a> shows what each line adds, with L3 as a named step.`);
  if (!rows.length) {
    sec.insertAdjacentHTML("beforeend", "<p class=\"note\"><strong>n/a.</strong> The reply carries no cost.headline rows.</p>");
    return;
  }
  sec.insertAdjacentHTML("beforeend", costTable(st, res, "rating", `S$ per tonne CO₂e abated, and LCOH in S$/kg, reactor charged at ${vu(ch, "MWth")}, ${gasText(res.inputs || st.inputs)}`));
  sec.insertAdjacentHTML("beforeend", targetLine(st, res));
  const req = headlineRows(res, "required");
  if (req.length) {
    sec.insertAdjacentHTML("beforeend", `<details class="view-section" data-keep="dash-required"><summary>Reactor charged at the ${esc(vu(req[0].charged_mwth, "MWth"))} the process needs</summary>${costTable(st, res, "required", `Same layout, reactor charged at ${vu(req[0].charged_mwth, "MWth")}, ${gasText(res.inputs || st.inputs)}`)}</details>`);
  }
}

function dashBalance(mount, st, res) {
  const k = res.kpi || {};
  const sec = section(mount, "dash-balance", "Heat and power at the rating", "Live",
    "Deficits and unmet demand show here as soon as the inputs produce them.");
  const tiles = [
    surplusTile("surplus", "Thermal surplus or deficit at rating", k.surplus_mwth, `Rating ${vu(k.rating_mwth, "MWth")}, process needs ${vu(k.reactor_required_mwth, "MWth")}`),
    surplusTile("surplus_ahn", "Thermal surplus or deficit, Ahn heat basis", k.surplus_ahn_mwth, "Process heat at the Ahn Case B value, helium flow following duty"),
    unmetTile(res, "unmet_power", "Unmet electrical demand at rating", k.unmet_power_mwe, "MWe", "kpi.unmet_power_mwe", "Unmet by the reactor, covered by grid import"),
    unmetTile(res, "unmet_heat", "Unmet thermal demand", k.unmet_heat_mwth, "MWth", "kpi.unmet_heat_mwth", "Heat duty above the rated reactor"),
    balTile({ key: "aux", label: "Auxiliary electricity", value: fmt(k.e_aux_mwe, "MWe"), unit: "MWe", state: finite(k.e_aux_mwe) ? "plain" : "na", note: `MEA regeneration ${vu(k.q_reboiler_mw, "MW")}, of which ${vu(k.reboiler_gap_mw, "MW")} from reactor steam` }),
    balTile({ key: "capture", label: "Capture per pass and plant capture", value: `${fmt(k.capture_per_pass, "-")} % and ${fmt(k.capture_plant, "-")} %`, state: "plain", note: `CO₂ captured ${vu(k.co2_captured_t_y, "t/y")}, to atmosphere ${vu(k.co2_atm_t_y, "t/y")}` }),
  ];
  sec.insertAdjacentHTML("beforeend", `<div class="balance-grid">${tiles.join("")}</div>`);
  const warns = (res.messages || []).filter((m) => m.level === "warn" || m.level === "error");
  if (warns.length) sec.insertAdjacentHTML("beforeend", `<ul class="flag-list">${warns.map((m) => `<li>${esc(m.unit)}: ${esc(m.text)}</li>`).join("")}</ul>`);
}

function dashAbatement(mount, st, res) {
  const k = res.kpi || {};
  const sec = section(mount, "dash-abatement", "Annual abatement on each baseline", "Live",
    "The INL baseline gives the headline. The Cho and IEAGHG baselines bound it; the system and lifecycle views are sensitivities.");
  const ids = ["abatement_t_y", "abatement_cho_t_y", "abatement_ieaghg_t_y", "abatement_system_t_y", "abatement_lifecycle_t_y"];
  const f = frame(sec, { key: "abatement", title: "Annual abatement, t CO₂e per year" });
  barChart(f, {
    unit: "t/y",
    fmt: (v) => fmt(v, "t/y"),
    rows: ids.map((id) => {
      const d = kpiDef(st, id);
      const short = d ? d.label.replace(/^Annual abatement, /, "") : id;
      return { label: short.charAt(0).toUpperCase() + short.slice(1), value: k[id] };
    }),
    ref: finite(k.target_abatement_t_y) ? { value: k.target_abatement_t_y, label: `Target ${fmt(k.target_abatement_t_y, "t/y")} t/y` } : null,
    valueName: "t/y",
    valueHead: "Abatement, t/y",
    rowHead: "Baseline or view",
    label: "Annual abatement by baseline, horizontal bars from zero with the target line",
  });
}

// Rows of the precomputed case table, by Final_Design_Basis.csv id. The two
// rating rows name the rating the native export used, read from the same file.
const MARGIN_ROW = "RES_MARGIN_210_MWTH";
const CASE_ROWS = [
  ["RES_H2_T_Y", "Purified H₂ after the external PSA"],
  ["RES_NG_MMSCFD", "Natural gas feed"],
  ["RES_PROCESS_HEAT_MW", "SMR process heat"],
  ["RES_REACTOR_NEEDED_MWTH", "Reactor heat the process needs"],
  [MARGIN_ROW, "Surplus or deficit at the rating"],
  ["RES_PLANT_CAPTURE", "Plant capture"],
  ["RES_CAPTURED_T_Y", "CO₂ captured"],
  ["RES_ABATEMENT_T_Y", "Annual abatement, INL baseline"],
  ["RES_E_TOTAL_MWE", "Auxiliary electricity"],
  ["RES_GRID_IMPORT_AT_RATED_MWE", "Grid import at the rating"],
];

function caseUnit(u) {
  return u === "MW" ? "MWth" : u;
}

function dashCases(mount, st) {
  const f = dataFile("reference.json");
  const ratingCell = f.status === "ok" ? (f.value.kpi || []).find((k) => k.id === "rating_mwth") : null;
  const rating = ratingCell ? vu(ratingCell.value, "MWth") : "n/a";
  const sec = section(mount, "dash-cases", "Case B and the Case A reference", "Precomputed",
    `From results/Final_Design_Basis.csv at this commit, with the reactor rated at ${esc(rating)} in every column. These columns do not follow the controls. Case A is the INL reference plant.`);
  if (f.status !== "ok") { sec.insertAdjacentHTML("beforeend", waitNote(f, "the reference file")); return; }
  const cols = f.value.case_columns || [];
  const body = CASE_ROWS.map(([id, label]) => {
    const r = cols.find((c) => c.id === id);
    if (!r) return `<tr><th scope="row">${esc(label)}</th><td class="num na" colspan="3">n/a, ${esc(id)} absent from the reference file</td></tr>`;
    const u = caseUnit(r.unit);
    const cellOf = (v) => {
      const deficit = id === MARGIN_ROW && finite(v) && v < 0;
      return `<td class="num">${id === MARGIN_ROW ? signed(v, u) : fmt(v, u)}${deficit ? " deficit" : ""}</td>`;
    };
    return `<tr><th scope="row">${esc(label)}, ${esc(unitLabel(u))}</th>${cellOf(r.case_b)}${cellOf(r.case_a_mea)}${cellOf(r.case_a_inl)}</tr>`;
  }).join("");
  sec.insertAdjacentHTML("beforeend", `<div class="table-scroll"><table class="data"><thead><tr><th scope="col">Quantity</th><th scope="col" class="num">Case B, headline</th><th scope="col" class="num">Case A with MEA and recycle</th><th scope="col" class="num">Case A, INL Case 6 with Selexol</th></tr></thead><tbody>${body}</tbody></table></div>`);
}

// ------------------------------------------------------------ MATURE_60Y scenario

/** The named scenario of data/reference.json, or null until it loads. */
export function scenarioDef() {
  const f = dataFile("reference.json");
  if (f.status !== "ok") return null;
  return (f.value.scenarios || []).find((s) => s.id === "MATURE_60Y") || null;
}

/** Inputs the scenario sets: its stage, life, exponent and gas price, with the reactor charged at its rating. */
export function scenarioInputs(st, sc) {
  const opt = (id, v) => {
    const p = paramRec(st, id);
    const o = p && p.options ? p.options.find((x) => Number(x.value) === Number(v) || x.value === v) : null;
    return o ? o.value : v;
  };
  return {
    stage: opt("stage", sc.inputs.stage),
    life_y: opt("life_y", sc.inputs.life_y),
    n: opt("n", sc.inputs.n),
    gas_s_gj: sc.inputs.gas_s_gj,
    charged: "rating",
    rating_mwth: sc.rating_mwth,
  };
}

const sameInput = (a, b) => (typeof b === "number" ? Number(a) === b : String(a) === String(b));

/** Whether one input sits at its registry default; an input never set reads as its default. */
function atDefault(st, id) {
  const v = st.inputs[id];
  return v === undefined || v === null || v === "" || sameInput(v, defaultOf(id));
}

/**
 * Which scenario the inputs hold: "mature", "base" or "other". A named scenario
 * holds only while every input outside its own keys (basis, PSA reject fate, heat
 * basis and every number control) sits at its default; any other change reads
 * "other", so no label lands on a result that is neither.
 */
export function scenarioState(st, sc) {
  if (!sc || !st.registry) return "other";
  const want = scenarioInputs(st, sc);
  const rest = st.registry.params.filter((p) => !(p.id in want)).every((p) => atDefault(st, p.id));
  if (!rest) return "other";
  if (Object.entries(want).every(([k, v]) => sameInput(st.inputs[k], v))) return "mature";
  if (Object.keys(want).every((k) => atDefault(st, k))) return "base";
  return "other";
}

/** Apply a scenario on the registry defaults: "mature" sets its inputs over them, "base" is the defaults. */
export function applyScenario(which) {
  const sc = scenarioDef();
  if (!sc || !state.registry) return false;
  const all = Object.fromEntries(state.registry.params.map((p) => [p.id, p.default]));
  setInputs(which === "mature" ? { ...all, ...scenarioInputs(state, sc) } : all);
  return true;
}

function optLabel(st, id, v) {
  const p = paramRec(st, id);
  const o = p && p.options ? p.options.find((x) => x.value === v) : null;
  return o ? o.label : String(v);
}

function renderScenario(st) {
  const box = $('[data-mount="scenario"]');
  if (!box) return;
  const sc = scenarioDef();
  if (!sc || !st.registry) { box.innerHTML = ""; return; }
  const mode = scenarioState(st, sc);
  const want = scenarioInputs(st, sc);
  const ref = dataFile("reference.json").value;
  const base = (id) => ((ref.kpi || []).find((k) => k.id === id) || {}).value;
  const gas0 = defaultOf("gas_s_gj");
  const baseText = `${optLabel(st, "stage", defaultOf("stage"))}, ${optLabel(st, "life_y", defaultOf("life_y"))}, gas at the central S$${num(gas0, 1)}/GJ`;
  const matText = `${optLabel(st, "stage", want.stage)}, ${optLabel(st, "life_y", want.life_y)}, gas at S$${num(want.gas_s_gj, 1)}/GJ`;
  const k = st.lastOk && st.lastOk.kpi ? st.lastOk.kpi : {};
  let note = "";
  if (mode === "mature") {
    note = `<p class="scenario-note" data-scenario-note="mature"><strong>Scenario MATURE_60Y is shown.</strong> The live engine gives L5 ${vu(k.cost_s_per_t, "S$/t")} and DCF ${vu(k.cost_lac_dcf_s_per_t, "S$/t")}; the published MATURE_60Y rows of results/Sensitivity_Results.csv at the same gas price give ${vu(sc.expected.ac_l5.value, "S$/t")} and ${vu(sc.expected.lac_dcf.value, "S$/t")}. The base result stays the reference: ${esc(baseText)}, L5 ${vu(base("cost_s_per_t"), "S$/t")} and DCF ${vu(base("cost_lac_dcf_s_per_t"), "S$/t")}. A higher gas price lowers the abatement cost here, because NUS-H₂ burns less gas than the unabated plant of equal H₂ output. The gas price is the printed one: the larger of the L5 and DCF gas solves, S$${num(sc.gas_needed.ac_l5.value, 2)} and S$${num(sc.gas_needed.lac_dcf.value, 2)}/GJ, rounded up to S$0.1/GJ.</p>`;
  } else if (mode === "other") {
    note = `<p class="scenario-note" data-scenario-note="other">The inputs differ from both named scenarios, so the result shown is neither. Choosing a scenario sets every input. The base result stays the reference: L5 ${vu(base("cost_s_per_t"), "S$/t")} and DCF ${vu(base("cost_lac_dcf_s_per_t"), "S$/t")}.</p>`;
  }
  const focused = box.contains(document.activeElement) && document.activeElement.name === "scenario";
  const steps = (sc.stepdown || []).map((s) => `<tr><th scope="row">${esc(s.label)}</th><td class="num">${num(s.gas_s_gj, 1)}</td><td class="num">${fmt(s.ac_l5, "S$/t")}</td><td class="num">${fmt(s.lac_dcf, "S$/t")}</td></tr>`).join("");
  box.innerHTML = `<fieldset class="scenario-pick">
      <legend>Cost scenario</legend>
      <label><input type="radio" name="scenario" value="base"${mode === "base" ? " checked" : ""}> <span><strong>Base result</strong>, the reference: ${esc(baseText)}</span></label>
      <label><input type="radio" name="scenario" value="mature"${mode === "mature" ? " checked" : ""}> <span><strong>MATURE_60Y</strong>: ${esc(matText)}, n = ${esc(String(want.n))}, reactor charged at ${vu(want.rating_mwth, "MWth")}</span></label>
    </fieldset>
    ${note}
    <details class="scenario-steps" data-keep="scenario-steps"${box.querySelector("details[open]") ? " open" : ""}><summary>Step-down from the base result to MATURE_60Y <span class="tag" data-tag="precomputed">Precomputed</span></summary>
      <div class="table-scroll"><table class="data"><caption>STEPDOWN rows of results/Sensitivity_Results.csv, Case B, n = 1.0, reactor charged at the rating. Every bar is an engine row.</caption>
      <thead><tr><th scope="col">Step</th><th scope="col" class="num">Gas, S$/GJ</th><th scope="col" class="num">L5, S$/t</th><th scope="col" class="num">DCF, S$/t</th></tr></thead><tbody>${steps}</tbody></table></div>
    </details>`;
  box.querySelectorAll('input[name="scenario"]').forEach((r) => r.addEventListener("change", () => { if (r.checked) applyScenario(r.value); }));
  // The fieldset is rebuilt on every render; give focus back to the radio group so
  // the next arrow key and a screen reader stay in it.
  if (focused) {
    const back = box.querySelector('input[name="scenario"]:checked') || box.querySelector('input[name="scenario"]');
    if (back) back.focus();
  }
}

/** Cost tiles of the strip carry the scenario and the base reference. */
function decorateStrip(st) {
  const sc = scenarioDef();
  if (!sc) return;
  const mode = scenarioState(st, sc);
  if (mode === "base") return;
  const ref = dataFile("reference.json").value;
  for (const id of ["cost_s_per_t", "cost_lac_dcf_s_per_t"]) {
    const tile = document.querySelector(`#kpi-strip [data-kpi="${id}"]`);
    const b = ((ref.kpi || []).find((x) => x.id === id) || {}).value;
    if (!tile || tile.querySelector(".k-scn")) continue;
    tile.insertAdjacentHTML("beforeend", `<span class="k-note k-scn">${mode === "mature" ? "Scenario MATURE_60Y. " : ""}Base result ${fmt(b, "S$/t")} S$/t</span>`);
  }
}

function renderDashboard(st) {
  const mount = $('[data-mount="dashboard"]');
  if (!mount) return;
  const keep = openDetails(mount);
  mount.replaceChildren();
  guard(mount, "The scenario control", () => { renderScenario(st); decorateStrip(st); });
  const res = st.lastOk;
  if (!st.registry || !res) {
    mount.innerHTML = `<p class="note">${st.registry ? "Waiting for the first solve." : "Loading the engine registry."}</p>`;
    return;
  }
  guard(mount, "The cost headline", () => dashCost(mount, st, res));
  guard(mount, "The heat and power tiles", () => dashBalance(mount, st, res));
  guard(mount, "The abatement chart", () => dashAbatement(mount, st, res));
  guard(mount, "The case reference table", () => dashCases(mount, st));
  restoreDetails(mount, keep);
}

// ------------------------------------------------------------ results

const TRAIN = ["S04", "S05", "S06", "S07", "S07A", "S08", "S09", "S09A", "S09B", "S09C", "S11", "S12", "S13"];
const SPECIES = [["h2", "H₂"], ["co", "CO"], ["co2", "CO₂"], ["ch4", "CH₄"], ["h2o", "H₂O"], ["n2", "N₂"]];
const SPECIES_STREAMS = ["S04", "S05", "S07", "S09", "S11", "S12", "S13", "S14"];

function itemsTable(items, res, prefix) {
  if (!items.length) return '<p class="note">No items.</p>';
  const rows = items.map((e) => {
    const na = !finite(e.value);
    const why = na ? (e.src || missingReason(res, `${prefix}.${e.id}`)) : "";
    return `<tr><th scope="row">${esc(pretty(e.label))}</th><td class="num${na ? " na" : ""}">${na ? "n/a" : esc(sig(e.value))}</td><td>${esc(e.unit || "")}</td><td>${na ? `<span class="reason">${esc(why)}</span>` : esc(e.class || "")}</td></tr>`;
  }).join("");
  return `<div class="table-scroll"><table class="data"><thead><tr><th scope="col">Item</th><th scope="col" class="num">Value</th><th scope="col">Unit</th><th scope="col">Class or reason</th></tr></thead><tbody>${rows}</tbody></table></div>`;
}

function resStreams(mount, st, res) {
  const sec = section(mount, "res-streams", "Stream table", "Precomputed",
    `The live solve runs the anchored port and returns no stream table (${esc(missingReason(res, "streams") || "no streams field")}). The Level 3 flowsheet streams below come from results/Integrated_Mass_Balance.csv, run ${FS_RUN}, at the headline basis. They do not follow the controls.`);
  const f = dataFile("Integrated_Mass_Balance.csv", "csv");
  if (f.status !== "ok") { sec.insertAdjacentHTML("beforeend", waitNote(f, "the stream table")); return null; }
  const rows = f.value.filter((r) => r.run_id === FS_RUN);
  const streams = rows.filter((r) => !r.stream_id.startsWith("CLOSURE_"));
  const closures = rows.filter((r) => r.stream_id.startsWith("CLOSURE_"));
  const body = streams.map((r) => `<tr id="stream-${esc(r.stream_id)}" data-stream-row="${esc(r.stream_id)}" tabindex="-1"><th scope="row">${esc(r.stream_id)}</th><td>${esc(pretty(r.stream_name))}</td><td>${esc(r.from_unit)} to ${esc(r.to_unit)}</td><td>${r.crosses_boundary === "true" ? "Crosses" : ""}</td><td class="num">${esc(sig(cell(r.t_c), 5))}</td><td class="num">${esc(sig(cell(r.p_bar), 5))}</td><td class="num">${esc(sig(cell(r.total_kmol_h), 6))}</td><td class="num">${esc(sig(cell(r.total_kg_h), 6))}</td><td>${esc(r.rigour)}</td><td>${esc(rowStatus(r.status))}</td></tr>`).join("");
  sec.insertAdjacentHTML("beforeend", `<div class="table-scroll"><table class="data"><thead><tr><th scope="col">Stream</th><th scope="col">Name</th><th scope="col">From and to</th><th scope="col">Boundary</th><th scope="col" class="num">T, °C</th><th scope="col" class="num">P, bar</th><th scope="col" class="num">kmol/h</th><th scope="col" class="num">kg/h</th><th scope="col">Rigour</th><th scope="col">Status</th></tr></thead><tbody>${body}</tbody></table></div>`);
  // A source cell that says more than where the stream comes from carries a
  // note on the row, such as why S13 exceeds the headline hydrogen.
  const notes = streams.filter((r) => /;/.test(r.source || "")).map((r) => `<li><strong>${esc(r.stream_id)}</strong>: ${esc(String(r.source).split(";").slice(1).join(";").trim())}</li>`).join("");
  if (notes) sec.insertAdjacentHTML("beforeend", `<p class="note">Row notes from the source column:</p><ul class="msg-list">${notes}</ul>`);
  const cl = closures.map((r) => `<li>${esc(r.stream_name)}: ${r.status === "not_run" ? "not run" : esc(sig(cell(r.total_kmol_h) ?? cell(r.carbon_kmol_h) ?? cell(r.total_kg_h)))}</li>`).join("");
  if (cl) sec.insertAdjacentHTML("beforeend", `<p class="note">Element closure of the flowsheet envelope:</p><ul class="msg-list">${cl}</ul>`);
  return streams;
}

function resComposition(mount, streams) {
  const sec = section(mount, "res-species", "Species compositions", "Precomputed",
    "Molar flow of each species along the train, from the same flowsheet run. The printed total is the stream total in the file, which also counts species not drawn.");
  if (!streams) { sec.insertAdjacentHTML("beforeend", '<p class="note">Waiting for the stream table.</p>'); return; }
  const picked = SPECIES_STREAMS.map((id) => streams.find((r) => r.stream_id === id)).filter(Boolean);
  const f = frame(sec, { key: "species", title: "Species flows by stream, kmol/h", tag: "Precomputed" });
  stackedBars(f, {
    unit: "kmol/h",
    fmt: (v) => fmt(v, "kmol/h"),
    series: SPECIES.map(([, name], i) => ({ name, slot: i })),
    rows: picked.map((r) => ({
      label: `${r.stream_id} ${r.stream_name}`,
      values: SPECIES.map(([k]) => cell(r[k])),
      totalText: `${fmt(cell(r.total_kmol_h), "kmol/h")} kmol/h`,
    })),
    rowHead: "Stream",
    totalHead: "Stream total, kmol/h",
    label: "Species molar flows by stream, stacked horizontal bars",
  });
}

/** A profile of the reply as {points, ...}, or null. */
function profile(res, key) {
  const p = res && res.profiles ? res.profiles[key] : null;
  return p && Array.isArray(p.points) && p.points.length ? p : null;
}

/** Train profile of the live Level 3 flowsheet, one point per stream in process order. */
function liveTrain(sec, p, key, title, unit, slotIx, d) {
  const src = String(p.src || "");
  const f = frame(sec, { key, title, tag: "Live", sub: `${src.charAt(0).toUpperCase()}${src.slice(1)}. It follows the controls.` });
  lineChart(f, {
    xLabels: p.points.map((q) => q.stream), xNames: p.points.map((q) => pretty(q.name)), step: "mid", zero: true,
    series: [{ name: title.split(",")[0], slot: slotIx, points: p.points.map((q, i) => ({ x: i, y: q.value })) }],
    fmtY: (v) => `${num(v, d)} ${unit}`, xHead: "Stream", yTitle: unit,
    label: `${title}, step line by stream`,
  });
}

function resProfiles(mount, st, res, streams) {
  const sec = section(mount, "res-profiles", "Temperature and pressure profiles", "Live",
    "The live solve runs the Level 3 flowsheet on the inputs and returns the train temperature and pressure by stream. A profile the engine does not return states its reason.");
  const smr = profile(res, "smr");
  if (!smr) unavailable(sec, { key: "live-smr", title: "Reformer tube profile of process gas and helium", tag: "Live", reason: missingReason(res, "profiles.smr") || "The reply carries no such profile." });
  const tT = profile(res, "train_T");
  const tP = profile(res, "train_P");
  if (tT) liveTrain(sec, tT, "train-t", "Temperature along the process train, °C", "°C", 1, 1);
  if (tP) liveTrain(sec, tP, "train-p", "Pressure along the process train, bar", "bar", 0, 2);
  if (tT && tP) return;
  // Without a live train profile, the precomputed flowsheet table is drawn instead.
  if (!streams) return;
  const train = TRAIN.map((id) => streams.find((r) => r.stream_id === id)).filter(Boolean);
  const labels = train.map((r) => r.stream_id);
  const names = train.map((r) => r.stream_name);
  const ft = frame(sec, { key: "train-t", title: "Temperature along the process train, °C", tag: "Precomputed" });
  lineChart(ft, {
    xLabels: labels, xNames: names, step: "mid", zero: true,
    series: [{ name: "Process gas temperature", slot: 1, points: train.map((r, i) => ({ x: i, y: cell(r.t_c) })) }],
    fmtY: (v) => `${num(v, 1)} °C`, xHead: "Stream", yTitle: "°C",
    label: "Process temperature by stream, step line",
  });
  const fp = frame(sec, { key: "train-p", title: "Pressure along the process train, bar", tag: "Precomputed", note: "Helium loop pressures are fixed values of the engine registry; the Interactive PFD panels list them per unit." });
  lineChart(fp, {
    xLabels: labels, xNames: names, step: "mid", zero: true,
    series: [{ name: "Process gas pressure", slot: 0, points: train.map((r, i) => ({ x: i, y: cell(r.p_bar) })) }],
    fmtY: (v) => `${num(v, 2)} bar`, xHead: "Stream", yTitle: "bar",
    label: "Process pressure by stream, step line",
  });
}

function resConversions(mount, st, res) {
  const sec = section(mount, "res-conversions", "Reaction conversions", "Live",
    "The Headline solve and the results files carry no conversion field for the reformer or the shift beds, so no conversion is printed. The species flows above give each bed inlet and outlet.");
  const smr = res.units && res.units.smr ? res.units.smr : null;
  const wgs = res.units && res.units.wgs ? res.units.wgs : null;
  if (smr) sec.insertAdjacentHTML("beforeend", `<h4>Reformer, live results</h4>${itemsTable([...(smr.results || [])], res, "units.smr.results")}`);
  if (wgs) sec.insertAdjacentHTML("beforeend", `<h4>Water gas shift, live results</h4>${itemsTable([...(wgs.results || [])], res, "units.wgs.results")}`);
}

/** Column profiles of the reply, drawn at the column model's design point. */
function columnProfile(sec, res, key) {
  const p = profile(res, key);
  const title = key === "absorber" ? "Absorber axial profiles" : "Stripper profile";
  if (!p) {
    unavailable(sec, { key: `live-${key}`, title, tag: "Live", reason: missingReason(res, `profiles.${key}`) || "The reply carries no such profile." });
    return;
  }
  const u = p.units || {};
  const xKey = key === "absorber" ? "z_m" : "stage";
  const xName = key === "absorber" ? "Height above the bottom of the packing" : "Stage from the top";
  const fx = key === "absorber" ? (v) => `${num(v, 2)} m` : (v) => `stage ${num(v, 0)}`;
  const sub = `${pretty(p.src)}. The profile reads no input, so it does not follow the controls.`;
  const temps = key === "absorber" ? [["t_gas_c", "Gas temperature", 1], ["t_liq_c", "Liquid temperature", 2]] : [["t_c", "Stage temperature", 1]];
  const ft = frame(sec, { key: `${key}-t`, title: `${title}: temperature, °C`, tag: "Design point", sub });
  lineChart(ft, {
    series: temps.map(([k, name, s]) => ({ name, slot: s, points: p.points.map((q) => ({ x: q[xKey], y: q[k] })) })),
    xName, fmtX: fx, fmtY: (v) => `${num(v, 1)} °C`, xTitle: key === "absorber" ? "Height, m" : "Stage",
    yTitle: "°C", xHead: xName, label: `${title}, temperature`,
  });
  const fl = frame(sec, { key: `${key}-l`, title: `${title}: solvent loading and gas CO₂ fraction`, tag: "Design point", sub: `Loading in ${u.loading || "mol CO2/mol MEA"}, gas fraction in ${u.y_co2 || "mol/mol"}.` });
  lineChart(fl, {
    series: [["loading", "Solvent loading, mol CO₂/mol MEA", 6], ["y_co2", "CO₂ in the gas, mol/mol", 0]].map(([k, name, s]) => ({ name, slot: s, points: p.points.map((q) => ({ x: q[xKey], y: q[k] })) })),
    xName, fmtX: fx, fmtY: (v) => num(v, 3), xTitle: key === "absorber" ? "Height, m" : "Stage",
    yTitle: "mol/mol", zero: true, xHead: xName, label: `${title}, loading and gas CO₂ fraction`,
  });
}

function resColumns(mount, st, res) {
  const sec = section(mount, "res-columns", "Absorber and stripper", "Live",
    "The column models stay anchored in the Headline solve because their validation gates failed. The profiles below come from the column model at its own design point, which the engine runs in your browser; they show the column shape and do not follow the controls.");
  columnProfile(sec, res, "absorber");
  columnProfile(sec, res, "stripper");
  const items = [];
  for (const u of ["absorber", "stripper", "reboiler"]) {
    const x = res.units ? res.units[u] : null;
    if (!x) continue;
    for (const sec2 of ["inputs", "conditions", "results"]) for (const e of x[sec2] || []) items.push({ ...e, label: `${x.label}: ${e.label}` });
  }
  if (items.length) sec.insertAdjacentHTML("beforeend", itemsTable(items, res, "units"));
  const ids = [["syngas_recovery_selexol", 0], ["syngas_recovery_export", 1], ["syngas_recovery_ex4", 2], ["reactor_steam_to_reboiler", 3]];
  const parts = ids.map(([id]) => balance(res, "energy", id));
  const total = balance(res, "energy", "mea_regeneration");
  if (parts.some((p) => !p) || !total) {
    unavailable(sec, { key: "reboiler-stack", title: "Reboiler heat sources", tag: "Live", reason: "The reply lacks one of the reboiler heat stack rows." });
    return;
  }
  // The heat order is the engine's row order; the wording is the engine's labels.
  const names = parts.map((p) => p.label.charAt(0).toLowerCase() + p.label.slice(1));
  const order = names.length > 1 ? `${names.slice(0, -1).join(", ")}, then ${names[names.length - 1]}` : names.join("");
  const f = frame(sec, { key: "reboiler-stack", title: "Reboiler heat sources, MW", tag: "Live", sub: `Heat order: ${order}.` });
  stackedBars(f, {
    unit: "MW",
    fmt: (v) => fmt(v, "MW"),
    series: parts.map((p, i) => ({ name: p.label, slot: ids[i][1] })),
    rows: [{ label: total.label, values: parts.map((p) => p.value), totalText: `${fmt(total.value, "MW")} MW` }],
    inlineLabels: true,
    rowHead: "Duty",
    totalHead: "Reboiler duty, MW",
    label: "Reboiler heat sources, one stacked bar",
  });
}

function resEnergy(mount, st, res) {
  const sec = section(mount, "res-energy", "Energy balance", "Live",
    "The thirteen energy balance items, each an engine row. A row the engine cannot fill reads n/a with the engine reason; the precomputed table below gives the native export for the headline run.");
  const energy = res.balances && Array.isArray(res.balances.energy) ? res.balances.energy : [];
  sec.insertAdjacentHTML("beforeend", itemsTable(energy, res, "balances.energy"));

  const ids = ["smr_process_heat", "recycle_slip_heat", "ihx_loss", "duct_loss", "vessel_loss"];
  const parts = ids.map((id) => balance(res, "energy", id));
  const branch = unitItem(res, "power", "branch");
  const wp = unitItem(res, "circ_primary", "w_p");
  const ws = unitItem(res, "circ_primary", "w_s");
  const need = balance(res, "energy", "htgr_heat_output");
  const k = res.kpi || {};
  if (parts.some((p) => !p) || !branch || !wp || !ws || !need) {
    unavailable(sec, { key: "reactor-heat", title: "Reactor heat budget", tag: "Live", reason: "The reply lacks the power branch or the circulator shaft work rows, so the budget cannot be drawn without a residual." });
  } else {
    const f = frame(sec, { key: "reactor-heat", title: "Reactor heat budget, MWth", tag: "Live", sub: "Duties and losses add up to the heat the process needs; circulator shaft work enters the helium and is taken off. Spare heat or a deficit then reaches the rating." });
    const steps = [
      ...parts.map((p) => ({ label: p.label, value: p.value })),
      { label: branch.label, value: branch.value },
      { label: `Less ${wp.label.toLowerCase()}`, value: wp.value, subtract: true, text: `less ${fmt(wp.value, "MW")}` },
      { label: `Less ${ws.label.toLowerCase()}`, value: ws.value, subtract: true, text: `less ${fmt(ws.value, "MW")}` },
      { label: "Reactor heat the process needs", value: need.value, kind: "total", named: true },
      { label: finite(k.surplus_mwth) && k.surplus_mwth < 0 ? "Deficit at rating" : "Spare heat at rating", value: k.surplus_mwth, text: signed(k.surplus_mwth, "MWth") },
      { label: "Reactor rating", value: k.rating_mwth, kind: "total" },
    ];
    waterfall(f, {
      unit: "MWth", fmt: (v) => fmt(v, "MWth"), steps, tolerance: 1e-6,
      upName: "Adds heat demand", downName: "Lowers heat demand", totalName: "Engine total",
      rowHead: "Line", label: "Reactor heat budget waterfall",
    });
  }

  const power = res.balances && Array.isArray(res.balances.power) ? res.balances.power : [];
  const users = ["reforming", "purification", "steam", "co2comp", "purge", "cooling", "water", "mea_pump", "mea_cooling", "e_circ"].map((id) => power.find((p) => p.id === id)).filter(Boolean);
  const tot = power.find((p) => p.id === "e_total");
  const gen = power.find((p) => p.id === "e_nuc");
  const imp = power.find((p) => p.id === "import_at_rated");
  if (users.length) {
    const f = frame(sec, { key: "electricity", title: "Electricity by consumer, MWe", tag: "Live" });
    barChart(f, {
      unit: "MWe", fmt: (v) => fmt(v, "MWe"),
      rows: users.map((p) => ({ label: p.label, value: p.value })),
      rowHead: "Consumer", valueHead: "Electricity, MWe",
      note: `Auxiliary electricity ${tot ? vu(tot.value, "MWe") : "n/a"}; generated from reactor heat at the need ${gen ? vu(gen.value, "MWe") : "n/a"}; grid import if the reactor is held at its rating ${imp ? vu(imp.value, "MWe") : "n/a"}, a memo item not added to supply. Engine totals.`,
      label: "Electricity by consumer, horizontal bars from zero",
    });
  }

  const ieb = dataFile("Integrated_Energy_Balance.csv", "csv");
  const sub = document.createElement("div");
  sub.innerHTML = `<h4>Native export for the headline run <span class="tag" data-tag="precomputed">Precomputed</span></h4>`;
  sec.appendChild(sub);
  if (ieb.status !== "ok") { sub.insertAdjacentHTML("beforeend", waitNote(ieb, "the energy balance file")); return; }
  const rows = ieb.value.filter((r) => r.run_id === HEADLINE_RUN);
  sub.insertAdjacentHTML("beforeend", `<div class="table-scroll"><table class="data"><thead><tr><th scope="col">Term</th><th scope="col">Side</th><th scope="col">Kind</th><th scope="col" class="num">MW</th><th scope="col" class="num">Low</th><th scope="col" class="num">High</th><th scope="col">Equation</th><th scope="col">Status</th></tr></thead><tbody>${rows.map((r) => `<tr><th scope="row">${esc(r.term)}</th><td>${esc(r.side)}</td><td>${esc(r.kind)}</td><td class="num${cell(r.value_mw) === null ? " na" : ""}">${cell(r.value_mw) === null ? "n/a" : esc(sig(cell(r.value_mw)))}</td><td class="num">${cell(r.low_mw) === null ? "" : esc(sig(cell(r.low_mw)))}</td><td class="num">${cell(r.high_mw) === null ? "" : esc(sig(cell(r.high_mw)))}</td><td>${esc(r.equation)}</td><td>${esc(rowStatus(r.status))}</td></tr>`).join("")}</tbody></table></div>`);
}

function resCarbon(mount, st, res) {
  const sec = section(mount, "res-carbon", "Carbon balance", "Live", "Carbon flows at the plant scale, kmol C per hour, then the annual tonnes.");
  const carbon = res.balances && Array.isArray(res.balances.carbon) ? res.balances.carbon : [];
  sec.insertAdjacentHTML("beforeend", itemsTable(carbon, res, "balances.carbon"));
  const cl = res.balances && res.balances.closure ? res.balances.closure : null;
  if (cl) sec.insertAdjacentHTML("beforeend", `<p class="note">Closure: ${esc(sig(cl.carbon_kmol_h))} kmol/h. ${esc(cl.basis || "")}.</p>`);
  const get = (id) => balance(res, "carbon", id);
  const feed = get("feed_c");
  const rec = get("recycled_c");
  const tube = get("tube_c");
  const cap = get("captured");
  const prod = get("product_c");
  const tail = get("tailgas_c");
  if (!feed || !rec || !tube || !cap || !prod || !tail || !cl) {
    unavailable(sec, { key: "carbon-waterfall", title: "Carbon waterfall", tag: "Live", reason: "The reply lacks one of the carbon rows the waterfall needs." });
    return;
  }
  const f = frame(sec, { key: "carbon-waterfall", title: "Carbon from feed to closure, kmol C/h", tag: "Live", sub: "Fresh feed plus recycled carbon gives the reformer tube carbon. Capture takes most of it out. The product carries a little, and the tail gas leaving the PSA carries the rest, which splits into recycle and the burned cut listed in the table above." });
  const less = (e) => ({ label: `Less ${e.label.toLowerCase()}`, value: e.value, subtract: true, text: `less ${fmt(e.value, "kmol/h")}` });
  waterfall(f, {
    unit: "kmol/h", fmt: (v) => fmt(v, "kmol/h"), tolerance: 1e-6,
    steps: [
      { label: feed.label, value: feed.value, kind: "total" },
      { label: `Plus ${rec.label.toLowerCase()}`, value: rec.value },
      { label: tube.label, value: tube.value, kind: "total" },
      less(cap), less(prod), less(tail),
      { label: "Closure residual", value: cl.carbon_kmol_h, kind: "total", text: sig(cl.carbon_kmol_h, 3) },
    ],
    upName: "Adds carbon", downName: "Takes carbon out", totalName: "Engine total",
    rowHead: "Line", label: "Carbon waterfall from feed to closure",
  });
}

function resCost(mount, st, res) {
  const c = res.cost;
  const sec = section(mount, "res-cost", "Cost breakdown", "Live",
    "Ledger lines in S$M per year at the selected case. Each level subtotal is the engine value with its S$ per tonne. L3 is a named step; L5 is the full cost.");
  if (!c || !Array.isArray(c.ledger) || !Array.isArray(c.ladder)) {
    unavailable(sec, { key: "cost-ladder", title: "Cost ladder", tag: "Live", reason: "The reply carries no cost section." });
    return;
  }
  const s = c.selected || {};
  sec.insertAdjacentHTML("beforeend", `<p class="note">Selected: ${esc(s.stage || "n/a")}, ${esc(sig(s.life_y))} y, n = ${esc(sig(s.n, 4))}, reactor charged at ${esc(vu(s.charged_mwth, "MWth"))}. L5 ledger ${esc(vu(s.ac_l5, "S$/t"))}; discounted cash flow ${esc(vu(s.dcf ? s.dcf.lac : null, "S$/t"))}, a relative difference of ${esc(s.dcf && finite(s.dcf.gap_rel) ? `${fmt(s.dcf.gap_rel, "-")} %` : "n/a")} between the two methods. LCOH ${esc(vu(s.dcf ? s.dcf.lcoh_nush2 : null, "S$/kg"))} for NUS-H₂ and ${esc(vu(s.dcf ? s.dcf.lcoh_smr : null, "S$/kg"))} for the unabated SMR.</p>`);
  const steps = [];
  for (const rung of c.ladder) {
    for (const line of c.ledger.filter((l) => l.level === rung.level)) steps.push({ label: line.label, value: line.value, note: `Ladder level ${line.level}` });
    const named = rung.level === "L3" || rung.level === "L5";
    const label = rung.level === "L5" ? "L5, full cost" : rung.level === "L3" ? "L3, a named step" : `${rung.level} subtotal`;
    steps.push({ label, value: rung.s_m_y, kind: "total", named, text: `${fmt(rung.s_m_y, "S$M/y")} S$M/y, ${fmt(rung.s_per_t, "S$/t")} S$/t`, extra: fmt(rung.s_per_t, "S$/t") });
  }
  const f = frame(sec, { key: "cost-ladder", title: "Cost ladder from L0 to L5, S$M/y", tag: "Live" });
  waterfall(f, {
    unit: "S$M/y", fmt: (v) => fmt(v, "S$M/y"), steps, tolerance: 1e-6,
    upName: "Adds cost", downName: "Lowers cost", totalName: "Level subtotal, engine",
    rowHead: "Line", extraHead: "Subtotal, S$/t", label: "Cost ladder waterfall from L0 to L5",
  });

  const rows = headlineRows(res, "rating");
  const opts = nOptions(st);
  const t = res.targets && res.targets.cost ? res.targets.cost.target : null;
  // The two stages and the lives of the headline cells, in file order.
  const cols = headlineCols() || [];
  const stages = [...new Set(cols.map(([s]) => s))];
  const lives = [...new Set(cols.map(([, l]) => l))];
  const [sa, sb] = stages;
  const dbRows = [];
  if (stages.length === 2) {
    for (const opt of opts) {
      for (const life of lives) {
        const b = pickRow(rows, sa, life, opt);
        const fo = pickRow(rows, sb, life, opt);
        dbRows.push({
          label: `n = ${opt.value}, ${life} y`,
          a: b ? b.ac_l5 : null,
          b: fo ? fo.ac_l5 : null,
          extra: [
            { value: b ? fmt(b.lac_dcf, "S$/t") : "n/a", label: `${sa}, discounted cash flow` },
            { value: fo ? fmt(fo.lac_dcf, "S$/t") : "n/a", label: `${sb}, discounted cash flow` },
          ],
          extraCells: [b ? fmt(b.lac_dcf, "S$/t") : "n/a", fo ? fmt(fo.lac_dcf, "S$/t") : "n/a"],
        });
      }
    }
  }
  if (!cols.length) {
    unavailable(sec, { key: "cost-dumbbell", title: "Full cost by stage against the target", tag: "Live", reason: "data/reference.json, which lists the headline cells, has not loaded." });
  } else if (dbRows.length) {
    const fd = frame(sec, { key: "cost-dumbbell", title: "Full cost by stage against the target, L5 ledger, S$/t", tag: "Live", sub: rows.length ? `Reactor charged at ${vu(rows[0].charged_mwth, "MWth")}, ${gasText(res.inputs || st.inputs)}. The table view adds the discounted cash flow values.` : "" });
    dumbbell(fd, {
      unit: "S$/t", fmt: (v) => fmt(v, "S$/t"), series: [sa, sb], rows: dbRows,
      ref: finite(t) ? { value: t, label: `Target S$${num(t, 0)}/t` } : null,
      extraCols: [`${sa}, DCF, S$/t`, `${sb}, DCF, S$/t`], rowHead: "Exponent and life",
      label: "Dumbbell of BOAK and FOAK full cost for each exponent and life, with the target line",
    });
  }

  const charged = st.inputs.charged || "rating";
  const grid = Array.isArray(c.grid) ? c.grid.filter((g) => g.charged === charged) : [];
  if (grid.length) {
    const levels = (st.registry && st.registry.levels) || ["L0", "L1", "L2", "L3", "L4", "L5"];
    const stageOpts = (paramRec(st, "stage") || {}).options || [];
    const lifeOpts = (paramRec(st, "life_y") || {}).options || [];
    let body = "";
    for (const so of stageOpts) {
      for (const lo of lifeOpts) {
        for (const opt of opts) {
          const cells = levels.map((lv) => {
            const g = grid.find((x) => x.stage === so.value && String(Number(x.life_y)) === lo.value && matchN(x.n, opt) && x.level === lv);
            return `<td class="num${lv === "L5" ? " strong" : ""}">${fmt(g ? g.s_per_t : null, "S$/t")}</td>`;
          }).join("");
          body += `<tr><th scope="row">${esc(so.label)}</th><td>${esc(lo.label)}</td><td>${esc(opt.value)}</td>${cells}</tr>`;
        }
      }
    }
    sec.insertAdjacentHTML("beforeend", `<details data-keep="cost-grid"><summary>Full cost grid in S$/t by ladder level, reactor charged at ${esc(vu(grid[0].charged_mwth, "MWth"))}</summary><div class="table-scroll"><table class="data"><thead><tr><th scope="col">Stage</th><th scope="col">Life</th><th scope="col">n</th>${levels.map((lv) => `<th scope="col" class="num">${esc(lv)}</th>`).join("")}</tr></thead><tbody>${body}</tbody></table></div></details>`);
  }
}

// ------------------------------------------------------------ sensitivity

const SWEEP_LABEL = {
  AVAILABILITY: "Availability", BASELINE: "Baseline intensity", CAPTURE: "Capture per pass", CHARGED: "Reactor size charged",
  ETA_NET_OF_PRIMARY: "Efficiency net of primary circulation", ETA_POWER: "Power conversion efficiency", GAS: "Natural gas price",
  HEAT_BASIS: "Process heat basis", HEAT_BASIS_HE_HELD: "Process heat basis, helium flow held", IHX_APPROACH: "IHX approach",
  SEC_DP: "Secondary helium pressure drop", SIZE_EXPONENT: "Size exponent", SOLVENT: "Capture solvent", SRD: "Specific reboiler duty",
  STAGE: "Cost stage", TAIL_GAS: "PSA reject fate", TS: "CO₂ transport and storage", ZLD: "ZLD steam freed",
};
const SWEEP_KPIS = ["cost_s_per_t", "cost_lac_dcf_s_per_t", "lcoh_nush2_s_per_kg", "surplus_mwth", "reactor_required_mwth", "abatement_t_y", "e_aux_mwe", "q_reboiler_mw", "h2_product_t_y"];
const live = { tornado: null, sweep: null, busy: false, cancel: false, progress: "" };

function precomputedTornado(sec, st) {
  const sens = dataFile("sensitivity.json");
  const ref = dataFile("reference.json");
  if (sens.status !== "ok" || ref.status !== "ok") {
    sec.insertAdjacentHTML("beforeend", waitNote(sens.status !== "ok" ? sens : ref, "the sensitivity files"));
    return;
  }
  const baseCell = (ref.value.kpi || []).find((k) => k.id === "cost_s_per_t");
  const base = baseCell ? baseCell.value : null;
  const groups = new Map();
  for (const sw of sens.value.sweeps || []) {
    if (sw.metric !== "AC_L5_25") continue;
    if (!groups.has(sw.sweep_id)) groups.set(sw.sweep_id, []);
    for (const p of sw.points) groups.get(sw.sweep_id).push({ ...p, run_id: sw.run_id, unit: sw.unit, parameter: p.parameter ?? sw.parameter });
  }
  const rows = [];
  for (const [id, pts] of groups) {
    const moved = pts.filter((p) => finite(p.delta) && p.delta !== 0 && finite(p.result));
    if (!moved.length) continue;
    const lo = moved.reduce((a, b) => (b.result < a.result ? b : a));
    const hi = moved.reduce((a, b) => (b.result > a.result ? b : a));
    // A sweep across runs (solvents, PSA reject fates) names each case by the
    // parameter text of its row, since a run id alone can read as the headline.
    const multiRun = new Set(pts.map((p) => p.run_id)).size > 1;
    const byText = new Set(pts.map((p) => p.parameter)).size > 1;
    const input = (p) => (byText ? pretty(p.parameter)
      : multiRun ? `run ${p.run_id}`
        : `${sig(p.test_value, 4)}${p.unit && p.unit !== "-" && p.unit !== "flag" ? ` ${p.unit}` : ""}`);
    const segs = [{ s: 0, value: lo.result, input: input(lo) }];
    if (hi !== lo) segs.push({ s: 1, value: hi.result, input: input(hi) });
    const spread = Math.max(...moved.map((p) => Math.abs(p.delta)));
    rows.push({ label: SWEEP_LABEL[id] || pts[0].parameter, segs, spread });
  }
  rows.sort((a, b) => b.spread - a.spread);
  const target = ref.value.targets ? ref.value.targets.cost_s_per_t : null;
  // Stage, life and exponent of the headline cell, from its id in Final_Design_Basis.csv.
  const cellId = baseCell && baseCell.row ? String(baseCell.row.id || "") : "";
  const m = /^AC_L5_([A-Z0-9]+)_(\d+)Y_N(\d)P(\d\d)$/.exec(cellId);
  const basis = m ? `${m[1]}, ${m[2]} y, n = ${m[3]}.${m[4]}` : "headline cell";
  const f = frame(sec, { key: "tornado-pre", title: `Tornado at the headline basis, L5 full cost, ${basis}, S$/t`, tag: "Precomputed", sub: `From results/Sensitivity_Results.csv. Bars run from the headline value ${vu(base, "S$/t")} to the lowest and highest case of each sweep.` });
  rangeBars(f, {
    unit: "S$/t", fmt: (v) => fmt(v, "S$/t"), base, baseLabel: "Headline",
    series: [{ name: "Lowest case", slot: 0 }, { name: "Highest case", slot: 1 }],
    rows, rowHead: "Sweep",
    ref: finite(target) ? { value: target, label: `Target S$${num(target, 0)}/t` } : null,
    label: "Tornado of the L5 full cost at the headline basis",
  });
}

function sweepValues(p, n) {
  const d = String(p.step).includes(".") ? String(p.step).split(".")[1].length : 0;
  const out = [];
  for (let i = 0; i < n; i += 1) out.push(Number((p.hard_min + ((p.hard_max - p.hard_min) * i) / (n - 1)).toFixed(d)));
  return [...new Set(out)];
}

async function runLiveTornado(st) {
  if (live.busy) return;
  live.busy = true;
  live.cancel = false;
  const inputs = { ...st.inputs };
  const params = st.registry.params.filter((p) => p.kind === "number");
  const base = st.lastOk && st.lastOk.kpi ? st.lastOk.kpi.cost_s_per_t : null;
  const rows = [];
  try {
    for (let i = 0; i < params.length; i += 1) {
      if (live.cancel) break;
      const p = params[i];
      live.progress = `Solving ${i + 1} of ${params.length}: ${p.label}`;
      rerender();
      const same = p.model_min === p.model_max;
      const values = same ? [p.hard_min, p.hard_max] : [p.model_min, p.model_max];
      const { reply } = await rpc("sweep", { inputs, param: p.id, values });
      // Each sweep row carries the engine verdicts; a point the rated reactor cannot
      // run is named as such beside its cost.
      const segs = (reply.rows || []).map((r, k) => ({
        s: k, value: r.status === "ok" && r.kpi ? r.kpi.cost_s_per_t : null,
        input: `${r.x}${p.unit && p.unit !== "-" ? ` ${p.unit}` : ""}${r.feasible === false ? ", infeasible at the rating" : ""}`,
      }));
      const spread = Math.max(0, ...segs.filter((g) => finite(g.value) && finite(base)).map((g) => Math.abs(g.value - base)));
      rows.push({ label: `${p.label}${same ? ", hard range" : ""}`, segs, spread });
    }
    rows.sort((a, b) => b.spread - a.spread);
    live.tornado = { rows, base, inputs: JSON.stringify(inputs), done: !live.cancel };
    live.progress = live.cancel ? "Stopped. The chart shows the parameters solved so far." : "";
  } catch (err) {
    live.progress = `The recompute failed: ${err.message}`;
  } finally {
    live.busy = false;
    rerender();
  }
}

async function runSweep(st, param, kpi) {
  if (live.busy) return;
  const p = paramRec(st, param);
  if (!p) return;
  live.busy = true;
  live.progress = `Sweeping ${p.label}`;
  rerender();
  const inputs = { ...st.inputs };
  try {
    const values = sweepValues(p, 15);
    const { reply } = await rpc("sweep", { inputs, param, values });
    if (reply.status !== "ok") throw new Error((reply.messages || []).map((m) => m.text).join(" ") || reply.status);
    live.sweep = { param, kpi, rows: reply.rows || [], inputs: JSON.stringify(inputs) };
    live.progress = "";
  } catch (err) {
    live.progress = `The sweep failed: ${err.message}`;
  } finally {
    live.busy = false;
    rerender();
  }
}

function resSensitivity(mount, st, res) {
  const sec = section(mount, "res-sensitivity", "Sensitivity", "", "The precomputed tornado comes from the published sweeps. The live tornado and the sweep line call the engine in your browser for the inputs set in Parameters.");
  precomputedTornado(sec, st);

  const curInputs = JSON.stringify(st.inputs);
  const box = document.createElement("div");
  box.innerHTML = `<div class="btn-row"><button type="button" class="btn" data-act="tornado"${live.busy ? " disabled" : ""}>Recompute the tornado for my inputs</button>${live.busy ? '<button type="button" class="btn" data-act="cancel">Stop</button>' : ""}</div><p class="progress" aria-live="polite">${esc(live.progress)}</p>`;
  sec.appendChild(box);
  box.querySelector('[data-act="tornado"]').addEventListener("click", () => runLiveTornado(state));
  const stop = box.querySelector('[data-act="cancel"]');
  if (stop) stop.addEventListener("click", () => { live.cancel = true; });
  if (live.tornado) {
    const stale = live.tornado.inputs !== curInputs;
    const f = frame(sec, { key: "tornado-live", title: "Tornado for the inputs at the time of the run, L5 full cost, S$/t", tag: "Live", sub: `Each number control moved to the ends of its evidence range, or of its hard range where the evidence range is a single value. Bars run from that run's solve, ${vu(live.tornado.base, "S$/t")}.${stale ? " The inputs have changed since this run." : ""}` });
    rangeBars(f, {
      unit: "S$/t", fmt: (v) => fmt(v, "S$/t"), base: live.tornado.base, baseLabel: "This solve",
      series: [{ name: "Low end", slot: 0 }, { name: "High end", slot: 1 }],
      rows: live.tornado.rows, rowHead: "Parameter",
      ref: res.targets && res.targets.cost && finite(res.targets.cost.target) ? { value: res.targets.cost.target, label: `Target S$${num(res.targets.cost.target, 0)}/t` } : null,
      label: "Live tornado of the L5 full cost",
    });
  }

  const nums = st.registry.params.filter((p) => p.kind === "number");
  const kpis = SWEEP_KPIS.map((id) => kpiDef(st, id)).filter(Boolean);
  const cur = live.sweep || { param: nums[0] ? nums[0].id : "", kpi: "cost_s_per_t" };
  const ctl = document.createElement("div");
  ctl.className = "sweep-controls";
  ctl.innerHTML = `<label>Parameter<select data-sweep="param">${nums.map((p) => `<option value="${esc(p.id)}"${p.id === cur.param ? " selected" : ""}>${esc(p.label)}</option>`).join("")}</select></label>
    <label>Result<select data-sweep="kpi">${kpis.map((k) => `<option value="${esc(k.id)}"${k.id === cur.kpi ? " selected" : ""}>${esc(k.label)}</option>`).join("")}</select></label>
    <button type="button" class="btn" data-act="sweep"${live.busy ? " disabled" : ""}>Run a 15-point sweep over the hard range</button>`;
  sec.appendChild(ctl);
  ctl.querySelector('[data-act="sweep"]').addEventListener("click", () => runSweep(state, ctl.querySelector('[data-sweep="param"]').value, ctl.querySelector('[data-sweep="kpi"]').value));
  ctl.querySelector('[data-sweep="kpi"]').addEventListener("change", (e) => {
    if (live.sweep) { live.sweep.kpi = e.target.value; rerender(); }
  });
  if (live.sweep) {
    const p = paramRec(st, live.sweep.param);
    const d = kpiDef(st, live.sweep.kpi);
    const unit = d ? d.unit : "";
    const stale = live.sweep.inputs !== curInputs;
    const pts = live.sweep.rows.map((r) => ({ x: Number(r.x), y: r.status === "ok" && r.kpi ? r.kpi[live.sweep.kpi] : null }));
    const refused = live.sweep.rows.filter((r) => r.status !== "ok").length;
    const infeasible = live.sweep.rows.filter((r) => r.status === "ok" && r.feasible === false).map((r) => sig(Number(r.x), 4));
    const f = frame(sec, { key: "sweep-line", title: `${d ? d.label : live.sweep.kpi} against ${p ? p.label.toLowerCase() : live.sweep.param}`, tag: "Live", sub: `${stale ? "The inputs have changed since this sweep. " : ""}${refused ? `${refused} points were refused or did not converge and are left out. ` : ""}${infeasible.length ? `The rated reactor cannot run the plant at ${infeasible.join(", ")}${p && p.unit && p.unit !== "-" ? ` ${p.unit}` : ""}, so no target is met there.` : ""}` });
    lineChart(f, {
      series: [{ name: d ? d.label : live.sweep.kpi, slot: 0, points: pts }],
      xName: p ? p.label : live.sweep.param,
      fmtX: (v) => `${sig(v, 4)}${p && p.unit && p.unit !== "-" ? ` ${p.unit}` : ""}`,
      fmtY: (v) => `${fmt(v, unit)} ${unitLabel(unit)}`,
      xTitle: p ? `${p.label}${p.unit && p.unit !== "-" ? `, ${p.unit}` : ", fraction"}` : "",
      yTitle: unitLabel(unit),
      label: "One-parameter sweep line",
    });
  }
}

function renderResults(st) {
  const mount = $('[data-mount="results"]');
  if (!mount) return;
  const y = window.scrollY;
  const keep = openDetails(mount);
  mount.replaceChildren();
  const res = st.lastOk;
  if (!st.registry || !res) {
    mount.innerHTML = `<p class="note">${st.registry ? "Waiting for the first solve." : "Loading the engine registry."}</p>`;
    return;
  }
  const toc = [["res-notes", "Engine notes"], ["res-streams", "Streams"], ["res-species", "Compositions"], ["res-profiles", "Profiles"], ["res-conversions", "Conversions"], ["res-columns", "Absorber and stripper"], ["res-energy", "Energy"], ["res-carbon", "Carbon"], ["res-cost", "Cost"], ["res-sensitivity", "Sensitivity"]];
  mount.insertAdjacentHTML("beforeend", `<nav class="results-toc" aria-label="Results sections">${toc.map(([id, t]) => `<a href="#results" data-jump="${id}">${esc(t)}</a>`).join("")}</nav>`);
  mount.querySelectorAll("[data-jump]").forEach((a) => a.addEventListener("click", (e) => {
    e.preventDefault();
    const t = document.getElementById(a.dataset.jump);
    if (t) t.scrollIntoView({ behavior: "smooth", block: "start" });
  }));
  guard(mount, "The engine notes", () => {
    const sec = section(mount, "res-notes", "Engine notes", "Live", "");
    const msgs = (res.messages || []).map((m) => `<li><strong>${esc(m.level)}</strong>, ${esc(m.unit)}: ${esc(m.text)}</li>`).join("");
    const flags = (res.flags || []).map((fl) => `<li><strong>flag</strong>, ${esc(fl.param)}: ${esc(flagText(fl.text))}</li>`).join("");
    const miss = (res.engine_fields_missing || []).map((m) => `<li><code>${esc(m.field)}</code>: ${esc(m.reason)}</li>`).join("");
    sec.insertAdjacentHTML("beforeend", `<ul class="msg-list">${msgs}${flags}</ul><details data-keep="missing-fields"><summary>Fields the engine does not produce in this solve</summary><ul class="msg-list">${miss}</ul></details>`);
  });
  let streams = null;
  guard(mount, "The stream table", () => { streams = resStreams(mount, st, res); });
  guard(mount, "The composition chart", () => resComposition(mount, streams));
  guard(mount, "The profiles", () => resProfiles(mount, st, res, streams));
  guard(mount, "The conversions", () => resConversions(mount, st, res));
  guard(mount, "The column results", () => resColumns(mount, st, res));
  guard(mount, "The energy balance", () => resEnergy(mount, st, res));
  guard(mount, "The carbon balance", () => resCarbon(mount, st, res));
  guard(mount, "The cost breakdown", () => resCost(mount, st, res));
  guard(mount, "The sensitivity charts", () => resSensitivity(mount, st, res));
  restoreDetails(mount, keep);
  window.scrollTo(0, y);
  jumpToStream();
}

// A stream link on the drawing names its row here; the jump waits until the
// stream table, which loads from data/csv, is drawn.
let pendingStream = null;
let markedStream = null;

function jumpToStream() {
  if (state.route !== "results") return;
  if (!pendingStream) {
    // A redraw keeps the mark of the last row reached, without scrolling.
    const kept = markedStream ? document.querySelector(`[data-stream-row="${CSS.escape(markedStream)}"]`) : null;
    if (kept) kept.classList.add("is-target");
    return;
  }
  const row = document.querySelector(`[data-stream-row="${CSS.escape(pendingStream)}"]`);
  if (!row) return;
  markedStream = pendingStream;
  pendingStream = null;
  document.querySelectorAll("tr.is-target").forEach((r) => r.classList.remove("is-target"));
  row.classList.add("is-target");
  row.scrollIntoView({ block: "center" });
  row.focus({ preventScroll: true });
}

document.addEventListener("nush2:stream", (e) => {
  pendingStream = e.detail && e.detail.id ? String(e.detail.id) : null;
  if (state.route === "results") jumpToStream();
});

// The Parameters page asks for a scenario by event, since this file owns it.
document.addEventListener("nush2:scenario", (e) => applyScenario(e.detail));

// ------------------------------------------------------------ export

function csvText(rows) {
  const q = (v) => {
    const s = v === null || v === undefined ? "" : String(v);
    return /[",\n\r]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
  };
  return `${rows.map((r) => r.map(q).join(",")).join("\r\n")}\r\n`;
}

function download(name, text, type) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([text], { type }));
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 4000);
}

/** The current solve as section,id,label,value,unit,class rows. Values are the engine's, unrounded. */
export function resultCsv(st) {
  const res = st.lastOk;
  const out = [["section", "id", "label", "value", "unit", "class"]];
  if (!res) return csvText(out);
  const push = (sec, id, label, v, unit, cls) => out.push([sec, id, label, finite(v) ? v : "", unit || "", finite(v) ? (cls || "calculated") : "n/a"]);
  // Each KPI carries the class the engine gives it in kpi_class.
  const kc = res.kpi_class || {};
  for (const [id, v] of Object.entries(res.kpi || {})) {
    const d = kpiDef(st, id);
    push("kpi", id, d ? d.label : id, v, d ? d.unit : "", kc[id] || "calculated");
  }
  for (const [uid, u] of Object.entries(res.units || {})) {
    for (const sec of ["inputs", "outputs", "conditions", "results"]) {
      for (const e of u[sec] || []) push(`units.${uid}.${sec}`, e.id, e.label, e.value, e.unit, e.class);
    }
  }
  for (const sec of ["energy", "power", "carbon"]) {
    for (const e of (res.balances && res.balances[sec]) || []) push(`balances.${sec}`, e.id, e.label, e.value, e.unit, e.class);
  }
  if (res.balances && res.balances.closure) push("balances.closure", "carbon_kmol_h", "Carbon closure residual", res.balances.closure.carbon_kmol_h, "kmol/h", "calculated");
  const c = res.cost || {};
  for (const l of c.ledger || []) push("cost.ledger", l.id, `${l.label}, ${l.level}`, l.value, l.unit, "calculated");
  for (const r of c.ladder || []) {
    push("cost.ladder", `${r.level}_s_m_y`, `Ladder ${r.level}`, r.s_m_y, "S$M/y", "calculated");
    push("cost.ladder", `${r.level}_s_per_t`, `Ladder ${r.level}`, r.s_per_t, "S$/t", "calculated");
  }
  for (const r of c.headline || []) {
    const tag = `${r.stage}_${Number(r.life_y)}y_n${r.n}_${r.charged}`;
    for (const [k, label, unit] of [["ac_l3", "Ladder step L3", "S$/t"], ["ac_l5", "Full cost, L5 ledger", "S$/t"], ["lac_dcf", "Full cost, discounted cash flow", "S$/t"], ["lcoh_nush2", "LCOH of NUS-H2", "S$/kg"], ["lcoh_smr", "LCOH of the unabated SMR", "S$/kg"]]) {
      push("cost.headline", `${tag}_${k}`, `${label}, ${r.stage}, ${Number(r.life_y)} y, n = ${r.n}, charged ${r.charged} at ${r.charged_mwth} MWth`, r[k], unit, "calculated");
    }
  }
  const t = res.targets || {};
  if (t.cost) {
    push("targets", "cost_target", "Cost target", t.cost.target, "S$/t", "user_decision");
    out.push(["targets", "cost_met", "Cost target met at the selected case, both methods", String(Boolean(t.cost.met)), "", "calculated"]);
  }
  if (t.abatement) {
    push("targets", "abatement_target", "Abatement target", t.abatement.target, "t/y", "user_decision");
    out.push(["targets", "abatement_met", "Abatement target met", String(Boolean(t.abatement.met)), "", "calculated"]);
  }
  return csvText(out);
}

/** One row per stream of the headline flowsheet run, every column of results/Integrated_Mass_Balance.csv. */
export function streamsCsv() {
  const f = files["csv:Integrated_Mass_Balance.csv"];
  if (!f || f.status !== "ok" || !f.value.length) return null;
  const cols = Object.keys(f.value[0]);
  const rows = f.value.filter((r) => r.run_id === FS_RUN && !String(r.stream_id).startsWith("CLOSURE_"));
  return csvText([cols, ...rows.map((r) => cols.map((c) => r[c]))]);
}

/** Inputs, flags, used parameter records and assumptions, with engine version and commit. */
export function inputsJson(st) {
  const res = st.lastOk;
  const prov = files["json:provenance.json"];
  const reg = st.registry || {};
  return `${JSON.stringify({
    schema: 1,
    title: document.title,
    engine: reg.engine || null,
    commit: reg.engine ? reg.engine.commit : null,
    exported_utc: new Date().toISOString(),
    inputs: st.inputs,
    inputs_as_solved: res ? res.inputs || null : null,
    changed_from_headline_basis: changedInputs(st).map((p) => ({ id: p.id, label: p.label, value: st.inputs[p.id], default: p.default })),
    flags: res ? res.flags || [] : [],
    messages: res ? res.messages || [] : [],
    engine_fields_missing: res ? res.engine_fields_missing || [] : [],
    parameters: reg.params || [],
    fixed_values: reg.fixed || [],
    controls_not_in_this_release: reg.dropped || [],
    assumptions: prov && prov.status === "ok" ? prov.value.assumptions : null,
    assumptions_note: prov && prov.status === "ok" ? "Input rows of results/Final_Design_Basis.csv at this commit" : "data/provenance.json was not loaded on this page",
  }, null, 2)}\n`;
}

function statusJson(st) {
  const v = files["json:validation.json"];
  const prov = files["json:provenance.json"];
  return `${JSON.stringify({
    schema: 1,
    engine: st.registry ? st.registry.engine : null,
    exported_utc: new Date().toISOString(),
    model_status: st.registry ? st.registry.units : null,
    provenance_counts: prov && prov.status === "ok" ? prov.value.counts : null,
    terms: prov && prov.status === "ok" ? prov.value.terms : null,
    validation: v && v.status === "ok" ? v.value : null,
    validation_note: v && v.status === "ok" ? "data/validation.json, built from results/Validation_Results.csv" : "data/validation.json was not loaded on this page",
  }, null, 2)}\n`;
}

function renderExport(st) {
  const mount = $('[data-mount="export"]');
  if (!mount) return;
  const prov = dataFile("provenance.json");
  const imb = dataFile("Integrated_Mass_Balance.csv", "csv");
  dataFile("validation.json");
  const c7 = commit7(st);
  const ready = Boolean(st.lastOk);
  const listing = prov.status === "ok" ? prov.value.results || [] : [];
  const csvRows = CSV_FILES.map((name) => {
    const meta = listing.find((x) => x.file === name);
    const what = meta ? `${meta.rows.toLocaleString("en-GB")} rows, ${meta.bytes.toLocaleString("en-GB")} bytes, SHA-256 ${meta.sha256.slice(0, 12)}` : (prov.status === "ok" ? "absent from the build listing" : "size and checksum load with data/provenance.json");
    return `<li><span class="e-what"><strong>${esc(name)}</strong><span>${esc(what)}</span></span><a class="btn" href="data/csv/${esc(name)}?v=local" download="${esc(name.replace(/\.csv$/, `_${c7}.csv`))}">Download</a></li>`;
  }).join("");
  mount.innerHTML = `
    <section class="view-section">
      <h3>This solve <span class="tag" data-tag="live">Live</span></h3>
      <p class="lede">Files built in your browser from the current engine reply. Values are unrounded engine numbers; a field the engine lacks is left empty with class n/a.</p>
      <ul class="export-list">
        <li><span class="e-what"><strong>Results CSV</strong><span>nush2_results_${esc(c7)}.csv: section, id, label, value, unit, class for every KPI, unit item, balance row, ledger line, ladder level and cost headline row</span></span><button type="button" class="btn" data-dl="results"${ready ? "" : " disabled"}>Download</button></li>
        <li><span class="e-what"><strong>Inputs and assumptions JSON</strong><span>nush2_inputs_${esc(c7)}.json: inputs, changes from the headline basis, flags, parameter records, fixed values, the assumption register, engine version and commit</span></span><button type="button" class="btn" data-dl="inputs"${st.registry ? "" : " disabled"}>Download</button></li>
        <li><span class="e-what"><strong>Validation and model status JSON</strong><span>nush2_status_${esc(c7)}.json: data/validation.json with the model status of every unit</span></span><button type="button" class="btn" data-dl="status"${st.registry ? "" : " disabled"}>Download</button></li>
        <li><span class="e-what"><strong>Streams CSV</strong><span>nush2_streams_${esc(c7)}.csv: one row per stream of the headline flowsheet run ${esc(FS_RUN)}, every column of Integrated_Mass_Balance.csv. Precomputed: the live solve returns no stream table.</span></span><button type="button" class="btn" data-dl="streams"${imb.status === "ok" ? "" : " disabled"}>Download</button></li>
        <li><span class="e-what"><strong>Print or save as PDF</strong><span>An engineering summary of three A4 pages: basis, KPIs, the cost table and ladder, energy and carbon balances, validation counts and the limitations, with the PFD when it is drawn</span></span><button type="button" class="btn primary" data-dl="print"${ready ? "" : " disabled"}>Print</button></li>
      </ul>
    </section>
    <section class="view-section">
      <h3>The seven published result files <span class="tag" data-tag="precomputed">Precomputed</span></h3>
      <p class="lede">Copies of results/*.csv at this commit, as written by the native engine.</p>
      <ul class="export-list">${csvRows}</ul>
    </section>
    <section class="view-section">
      <h3>Licences</h3>
      <p class="lede">The engine and this site are under the MIT licence. The WebAssembly module compiles in third-party Rust crates under their own licences; their notice lists each crate and version with its licence text.</p>
      <ul class="export-list"><li><span class="e-what"><strong>THIRD_PARTY_NOTICES.txt</strong><span>Crates compiled into nush2.wasm, the licence of each and its text</span></span><a class="btn" href="THIRD_PARTY_NOTICES.txt?v=local" data-notices>Open</a></li></ul>
    </section>`;
  mount.querySelector('[data-dl="results"]').addEventListener("click", () => download(`nush2_results_${commit7(state)}.csv`, resultCsv(state), "text/csv"));
  mount.querySelector('[data-dl="inputs"]').addEventListener("click", () => download(`nush2_inputs_${commit7(state)}.json`, inputsJson(state), "application/json"));
  mount.querySelector('[data-dl="status"]').addEventListener("click", () => download(`nush2_status_${commit7(state)}.json`, statusJson(state), "application/json"));
  mount.querySelector('[data-dl="streams"]').addEventListener("click", () => {
    const text = streamsCsv();
    if (text) download(`nush2_streams_${commit7(state)}.csv`, text, "text/csv");
  });
  mount.querySelector('[data-dl="print"]').addEventListener("click", () => { buildPrint(state); window.print(); });
}

// ------------------------------------------------------------ validation

function statusChip(s) {
  const d = STATUS[s] || { label: s, icon: RING };
  return `<span class="status-chip" data-status="${esc(s)}">${d.icon}${esc(d.label)}</span>`;
}

let valFilter = "fail";

function renderValidation(st) {
  const mount = $('[data-mount="validation"]');
  if (!mount) return;
  const vf = dataFile("validation.json");
  const prov = dataFile("provenance.json");
  if (vf.status !== "ok") { mount.innerHTML = waitNote(vf, "the validation file"); return; }
  const v = vf.value;
  const by = (v.counts && v.counts.by_status) || {};
  const order = ["pass", "fail", "not_run", "reported"];
  const extra = Object.keys(by).filter((k) => !order.includes(k));
  const commit = prov.status === "ok" ? prov.value.commit : (st.registry ? st.registry.engine.commit : "local");
  const runId = prov.status === "ok" ? String(prov.value.run_id) : "local";
  const runLink = /^\d+$/.test(runId) ? `<a href="https://github.com/weewzr/NUSH2/actions/runs/${esc(runId)}">CI run ${esc(runId)}</a>` : "a local build with no CI run";
  const commitText = /^[0-9a-f]{7,40}$/.test(String(commit)) ? `<a href="https://github.com/weewzr/NUSH2/commit/${esc(commit)}"><code>${esc(String(commit).slice(0, 7))}</code></a>` : `<code>${esc(commit)}</code>`;
  const summary = [...order, ...extra].filter((k) => k in by).map((k) => `${by[k].toLocaleString("en-GB")} ${(STATUS[k] || { label: k }).label.toLowerCase()}`).join(", ");
  const tests = v.tests ? `Engine tests in that run: ${v.tests.passed} passed, ${v.tests.failed} failed, ${v.tests.ignored} ignored.` : `Engine test counts: n/a, ${esc(v.tests_reason || "no test log")}.`;
  const rows = v.rows || [];
  const fails = rows.filter((r) => r.status === "fail");
  const gaps = rows.filter((r) => r.status === "not_run");
  const units = st.registry ? Object.entries(st.registry.units) : [];
  const terms = prov.status === "ok" ? prov.value.terms || [] : [];
  const shown = valFilter === "all" ? rows : rows.filter((r) => r.status === valFilter);
  const valRow = (r) => `<tr data-status="${esc(r.status)}"><th scope="row"><code>${esc(r.test_id)}</code></th><td>${esc(sig(r.level))}</td><td class="wrap">${esc(r.subject || "")}</td><td class="num">${esc(sig(r.literature))}</td><td class="num">${esc(sig(r.simulated))}</td><td class="num">${esc(sig(r.abs_error, 4))}</td><td class="num">${esc(sig(r.rel_error, 4))}</td><td>${esc(r.unit || "")}</td><td class="wrap">${esc(r.conditions || "")}</td><td class="wrap">${esc([r.source, r.locator].filter(Boolean).join("; "))}</td><td class="wrap">${esc(r.explanation || "")}</td><td>${esc(r.data_use || "")}</td><td>${statusChip(r.status)}</td></tr>`;
  mount.innerHTML = `
    <section class="view-section" id="val-summary">
      <p class="lede" data-validation-summary>${esc(v.counts.total.toLocaleString("en-GB"))} checks in results/Validation_Results.csv at commit ${commitText}, from ${runLink}: ${esc(summary)}. ${tests}</p>
      <div class="counts">${[...order, ...extra].filter((k) => k in by).map((k) => `<div class="count" data-status="${esc(k)}" data-count="${by[k]}"><span class="c-label">${(STATUS[k] || { icon: RING }).icon}${esc((STATUS[k] || { label: k }).label)}</span><span class="c-value">${by[k].toLocaleString("en-GB")}</span></div>`).join("")}</div>
      <p class="note">A status word comes only from the CSV the engine wrote. In the Headline preset a screening term leaves its anchored value only after its gate passes; the term table below gives each gate and the value used. The file also uses the status word reported, for a row that prints a computed value with no literature value to judge it against; such a row neither passes nor fails.</p>
    </section>
    <section class="view-section" id="val-failures">
      <h3>Failures by name</h3>
      <ol class="fail-list">${fails.map((r) => `<li data-test="${esc(r.test_id)}"><code>${esc(r.test_id)}</code>: ${esc(r.subject || "")}. Literature ${esc(sig(r.literature))}, simulated ${esc(sig(r.simulated))}${r.unit ? ` ${esc(r.unit)}` : ""}, relative error ${esc(sig(r.rel_error, 4))}, tolerance ${esc(sig(r.tolerance, 4))}. ${esc(r.explanation || "")}</li>`).join("")}</ol>
    </section>
    <section class="view-section" id="val-table">
      <h3>Validation table</h3>
      <div class="filter-row"><label>Status <select data-filter>${["all", ...order, ...extra].map((k) => `<option value="${k}"${k === valFilter ? " selected" : ""}>${k === "all" ? "All" : esc((STATUS[k] || { label: k }).label)}${k === "all" ? ` (${rows.length})` : ` (${by[k] || 0})`}</option>`).join("")}</select></label><span class="note">${shown.length} rows shown</span></div>
      <div class="table-scroll"><table class="data val-table"><thead><tr><th scope="col">Test</th><th scope="col">Level</th><th scope="col">Quantity</th><th scope="col" class="num">Literature</th><th scope="col" class="num">Simulated</th><th scope="col" class="num">Absolute error</th><th scope="col" class="num">Relative error</th><th scope="col">Units</th><th scope="col">Conditions</th><th scope="col">Source page or table</th><th scope="col">Deviation note</th><th scope="col">Tuning relation</th><th scope="col">Status</th></tr></thead><tbody>${shown.map(valRow).join("")}</tbody></table></div>
    </section>
    <section class="view-section" id="val-modules">
      <h3>Model status of each unit</h3>
      <p class="note">Status words come from the engine registry: rigorous, reduced order, empirical, fixed assumption or external.</p>
      <div class="table-scroll"><table class="data"><thead><tr><th scope="col">Unit</th><th scope="col">Status</th><th scope="col">Screening term</th><th scope="col">Basis</th></tr></thead><tbody>${units.map(([, u]) => `<tr><th scope="row">${esc(u.label)}</th><td>${esc(String(u.status).replace(/_/g, " "))}</td><td>${esc(u.term || "")}</td><td>${esc(u.basis || "")}</td></tr>`).join("")}</tbody></table></div>
    </section>
    <section class="view-section" id="val-terms">
      <h3>Screening terms and their gates <span class="tag" data-tag="precomputed">Precomputed</span></h3>
      ${terms.length ? `<div class="table-scroll"><table class="data"><thead><tr><th scope="col">Term</th><th scope="col">Term as exported</th><th scope="col" class="num">Value at Case B</th><th scope="col">Unit</th><th scope="col">Status</th></tr></thead><tbody>${terms.map((t) => `<tr><th scope="row"><code>${esc(String(t.id).replace(/^TERM_/, ""))}</code></th><td class="wrap">${esc(t.parameter || "")}</td><td class="num">${esc(sig(t.case_b))}</td><td>${esc(t.unit || "")}</td><td>${esc(String(t.status || "").replace(/_/g, " "))}</td></tr>`).join("")}</tbody></table></div>` : (prov.status === "ok" ? '<p class="note">No term rows in data/provenance.json.</p>' : waitNote(prov, "the provenance file"))}
    </section>
    <section class="view-section" id="val-gaps">
      <h3>Checks that could not run from local sources</h3>
      <ul class="msg-list">${gaps.map((r) => `<li><code>${esc(r.test_id)}</code>: ${esc(r.subject || "")}. ${esc(r.explanation || "")}</li>`).join("")}</ul>
    </section>`;
  mount.querySelector("[data-filter]").addEventListener("change", (e) => { valFilter = e.target.value; renderValidation(state); });
}

// ------------------------------------------------------------ print summary

function kv(rows) {
  return `<table class="data"><tbody>${rows.map(([k, v]) => `<tr><th scope="row">${esc(k)}</th><td class="num">${esc(v)}</td></tr>`).join("")}</tbody></table>`;
}

/** Key and value pairs in two column pairs per row, to keep the printed summary short. */
function kv2(rows) {
  const half = Math.ceil(rows.length / 2);
  const out = [];
  for (let i = 0; i < half; i += 1) {
    const b = rows[i + half];
    out.push(`<tr><th scope="row">${esc(rows[i][0])}</th><td class="num">${esc(rows[i][1])}</td>${b ? `<th scope="row">${esc(b[0])}</th><td class="num">${esc(b[1])}</td>` : "<td></td><td></td>"}</tr>`);
  }
  return `<table class="data kv2"><tbody>${out.join("")}</tbody></table>`;
}

/**
 * Copy an inline SVG for the print summary. Its ids get a prefix and every
 * url(#id) or #id reference follows, so gradients resolve inside the copy
 * while the original sits in a hidden route.
 */
const PRINT_PROPS = ["fill", "fill-opacity", "fill-rule", "stroke", "stroke-width", "stroke-opacity", "stroke-dasharray",
  "stroke-linecap", "stroke-linejoin", "opacity", "font-size", "font-weight", "font-family", "font-style", "text-anchor",
  "dominant-baseline", "letter-spacing", "display", "visibility", "marker-start", "marker-mid", "marker-end", "paint-order"];

function printableSvg(svg) {
  if (!svg) return null;
  const copy = svg.cloneNode(true);
  // The drawing is styled by selectors on its own id, which the copy cannot
  // keep, so each copied element takes the computed paint of its original.
  const src = [svg, ...svg.querySelectorAll("*")];
  const dst = [copy, ...copy.querySelectorAll("*")];
  src.forEach((n, i) => {
    const cs = getComputedStyle(n);
    const decl = PRINT_PROPS.map((p) => [p, cs.getPropertyValue(p)]).filter(([, v]) => v).map(([p, v]) => `${p}:${v}`);
    if (dst[i]) dst[i].setAttribute("style", decl.join(";"));
  });
  const ids = new Set();
  copy.querySelectorAll("[id]").forEach((n) => { ids.add(n.id); n.id = `print-${n.id}`; });
  if (copy.id) copy.id = `print-${copy.id}`;
  const fix = (v) => v.replace(/url\(\s*["']?[^"')#]*#([^"')]+?)["']?\s*\)/g, (m, id) => (ids.has(id) ? `url(#print-${id})` : m))
    .replace(/^#(.+)$/, (m, id) => (ids.has(id) ? `#print-${id}` : m));
  copy.querySelectorAll("*").forEach((n) => {
    for (const a of Array.from(n.attributes)) {
      if (a.value.includes("#")) n.setAttribute(a.name, fix(a.value));
    }
    // The copy is a picture: no focus stops, roles or unit and stream hooks.
    for (const name of ["tabindex", "role", "data-unit", "data-stream", "data-bind", "aria-label"]) n.removeAttribute(name);
  });
  copy.removeAttribute("aria-labelledby");
  copy.setAttribute("aria-hidden", "true");
  return copy;
}

/** Build the engineering summary that print.css shows in place of the page. */
function buildPrint(st) {
  let el = document.getElementById("print-summary");
  if (!el) {
    el = document.createElement("section");
    el.id = "print-summary";
    el.setAttribute("aria-hidden", "true");
    document.body.appendChild(el);
  }
  const res = st.lastOk;
  if (!st.registry || !res) { el.innerHTML = "<p>No converged solve to print.</p>"; return; }
  const reg = st.registry;
  const k = res.kpi || {};
  // L3 is printed once, as a named step of the ladder table below.
  const kpis = reg.kpis.filter((d) => d.id in k && d.id !== "cost_l3_s_per_t").map((d) => [pretty(d.label), finite(k[d.id]) ? `${fmt(k[d.id], d.unit)} ${unitLabel(d.unit)}` : `n/a, ${missingReason(res, `kpi.${d.id}`)}`]);
  const changed = changedInputs(st);
  const c = res.cost || {};
  const ladder = (c.ladder || []).map((r) => [r.level === "L5" ? "L5, full cost" : r.level === "L3" ? "L3, a named step" : r.level, `${fmt(r.s_m_y, "S$M/y")} S$M/y, ${fmt(r.s_per_t, "S$/t")} S$/t`]);
  const energy = ((res.balances && res.balances.energy) || []).map((e) => [e.label, finite(e.value) ? `${sig(e.value)} ${e.unit}` : `n/a, ${e.src || ""}`]);
  const carbon = ((res.balances && res.balances.carbon) || []).map((e) => [e.label, finite(e.value) ? `${sig(e.value)} ${e.unit}` : "n/a"]);
  const vf = files["json:validation.json"];
  const by = vf && vf.status === "ok" ? vf.value.counts.by_status : null;
  const fails = vf && vf.status === "ok" ? vf.value.rows.filter((r) => r.status === "fail").map((r) => r.test_id) : [];
  const pfd = printableSvg(document.getElementById("pfd-svg") || document.querySelector("#pfd-mount svg"));
  const t = res.targets || {};
  // Name the scenario the inputs hold, and keep the base result on the page.
  const sc = scenarioDef();
  const mode = sc ? scenarioState(st, sc) : "other";
  const ref = dataFile("reference.json");
  const baseVal = (id) => (ref.status === "ok" ? ((ref.value.kpi || []).find((x) => x.id === id) || {}).value : null);
  const baseLine = `the base result stays the reference: L5 ${vu(baseVal("cost_s_per_t"), "S$/t")} and DCF ${vu(baseVal("cost_lac_dcf_s_per_t"), "S$/t")} (BOAK, 25 y, n = 1.0, gas at the central price)`;
  const scenarioLine = mode === "mature"
    ? `Scenario MATURE_60Y is printed: tenth of a kind on INL's BOAK learning route, 60 y, n = 1.0, reactor charged at the rating, ${gasText(res.inputs || st.inputs)}; ${baseLine}.`
    : mode === "other" ? `The inputs differ from both named scenarios; ${baseLine}.` : "";
  el.innerHTML = `
    <header><h1>${esc(document.title)}</h1>
    <p>Engineering summary. Engine ${esc(reg.engine.version)}, commit ${esc(String(reg.engine.commit).slice(0, 7))}, ${esc(reg.engine.preset)} preset, ${esc(reg.engine.tier || "")} tier. Printed ${esc(new Date().toISOString().slice(0, 16).replace("T", " "))} UTC.</p>
    <p>${esc($("#basis-line") ? $("#basis-line").textContent : "")}</p>
    <p>Inputs changed from the headline basis: ${changed.length ? esc(changed.map((p) => `${p.label} ${inputText(st, p)}`).join("; ")) : "none"}.</p>
    ${scenarioLine ? `<p data-print-scenario="${mode}">${esc(scenarioLine)}</p>` : ""}</header>
    <h2>Targets</h2>
    <p>${res.feasibility && res.feasibility.feasible === false ? `${esc(res.feasibility.reason)}. ` : ""}Abatement target ${esc(vu(t.abatement ? t.abatement.target : null, "t/y"))}: ${t.abatement && t.abatement.met ? "met" : "not met"} at these inputs. Cost target ${esc(vu(t.cost ? t.cost.target : null, "S$/t"))} at ${esc(caseText(t.cost && Array.isArray(t.cost.cases) ? t.cost.cases.find((x) => x.selected) : null))}: ${t.cost && t.cost.met ? "met on both methods" : "not met"}.</p>
    <h2>Key results</h2>${kv2(kpis)}
    <h2>Rebuilt full cost</h2>${costTable(st, res, "rating", `S$ per tonne CO₂e abated, and LCOH in S$/kg, reactor charged at the rating, ${gasText(res.inputs || st.inputs)}`)}
    <div class="print-cols"><div><h2>Cost ladder at the selected case</h2>${kv(ladder)}</div><div><h2>Carbon balance</h2>${kv(carbon)}</div></div>
    ${pfd ? `<h2>Process flow diagram</h2><div class="print-pfd">${pfd.outerHTML}</div>` : ""}
    <h2>Energy balance</h2>${kv2(energy)}
    <h2>Validation</h2><p>${by ? esc(Object.entries(by).map(([s, n]) => `${n} ${(STATUS[s] || { label: s }).label.toLowerCase()}`).join(", ")) : "n/a, data/validation.json was not loaded"}.${fails.length ? ` Failed: ${esc(fails.join(", "))}.` : ""}</p>
    <h2>Limitations</h2>
    <ul>${(res.engine_fields_missing || []).map((m) => `<li>${esc(m.field)}: ${esc(m.reason)}</li>`).join("")}${(res.flags || []).map((f) => `<li>${esc(flagText(f.text))}</li>`).join("")}${(res.messages || []).filter((m) => m.code === "anchored_terms").map((m) => `<li>${esc(m.text)}</li>`).join("")}</ul>
    <p>Engineering screening tool. MIT licence.</p>`;
}

// ------------------------------------------------------------ wiring

registerRoute("dashboard", renderDashboard);
registerRoute("parameters", renderParameters);
registerRoute("results", renderResults);
registerRoute("export", renderExport);
registerRoute("validation", renderValidation);

window.addEventListener("beforeprint", () => { hideTip(); buildPrint(state); });
window.addEventListener("hashchange", hideTip);
let resizeTimer = null;
let lastWidth = window.innerWidth;
window.addEventListener("resize", () => {
  if (window.innerWidth === lastWidth) return;
  lastWidth = window.innerWidth;
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    if (state.route === "dashboard" || state.route === "results") render();
  }, 200);
});

// The cost tables and the scenario control take their data from this file,
// and the print summary its validation counts from the second, so both load
// at start on every route.
dataFile("reference.json");
dataFile("validation.json");
render();
