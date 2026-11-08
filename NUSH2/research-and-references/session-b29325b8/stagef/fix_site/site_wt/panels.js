// Unit panels of the interactive PFD. Each panel has seven sections in a
// fixed order: inputs, outputs, operating conditions, governing equations,
// literature sources, calculated results and model limitations.
//
// Live values come from the engine reply (result.units[id]) and validation
// rows from data/validation.json, which CI builds from results/*.csv. Static
// text comes from units.json. This file formats values for display and holds
// no model arithmetic: a value the engine does not produce prints as n/a with
// the engine's own reason.

import { pretty } from "./app.js?v=54a7ac4a0553e1b23ad16f5fc90f1bce7bb4bb16";

export const SECTIONS = [
  ["inputs", "Inputs"],
  ["outputs", "Outputs"],
  ["conditions", "Operating conditions"],
  ["equations", "Governing equations"],
  ["sources", "Literature sources"],
  ["results", "Calculated results"],
  ["limitations", "Model limitations"],
];

export const STATUS = {
  rigorous: "Rigorous",
  reduced_order: "Reduced order",
  empirical: "Empirical",
  fixed_assumption: "Fixed assumption",
  external: "External",
};

const CLASS = {
  published: "Published",
  assumed: "Assumed",
  calculated: "Calculated",
  user_decision: "Project decision",
};

const VSTATUS = { pass: "Pass", fail: "Fail", reported: "Reported", not_run: "Not run", no_data: "No data" };

// Display decimals per unit. Anything else prints to three significant figures.
const DECIMALS = {
  "°C": 0, K: 0, "t/y": 0, "kg/h": 0, "kmol/h": 0, "t/h": 0, "st/d": 0, "kg/m3": 0, "Nm3/h": 0, kPa: 0,
  MWth: 1, MW: 1, bar: 1, MPa: 1, "kg/s": 1, "S$M/y": 1, "S$/tCO2": 1, "S$/t": 1, "MMBtu/h": 1,
  MWe: 2, MMSCFD: 2, "GJ/t": 2, "mol/mol": 2, "kg/kg": 2, "S$/kg": 2,
};

export const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

/** Format a number for display. Returns {text, unit}; the unit "-" is a fraction shown as a percentage. */
export function fmtValue(v, unit) {
  const raw = unit === undefined || unit === null ? "" : String(unit);
  const u = raw === "C" ? "°C" : raw;
  if (typeof v !== "number" || !Number.isFinite(v)) return { text: "n/a", unit: u === "-" ? "%" : u };
  if (u === "-") {
    return { text: (v * 100).toLocaleString("en-GB", { minimumFractionDigits: 1, maximumFractionDigits: 1 }), unit: "%" };
  }
  if (u in DECIMALS) {
    const d = DECIMALS[u];
    return { text: v.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d }), unit: u };
  }
  if (Math.abs(v) >= 1000) return { text: v.toLocaleString("en-GB", { maximumFractionDigits: 0 }), unit: u };
  return { text: v.toLocaleString("en-GB", { maximumSignificantDigits: 3 }), unit: u };
}

// Panel prose in units.json carries no engineering number of its own. A token
// {{path|d}} names a value of the engine reply (inputs, units, kpi, ...) and
// the decimals to print. Inside an array a name that is not an index picks the
// entry with that id, as data-bind does on the drawing. A missing value prints n/a.
const TOKEN = /\{\{([A-Za-z0-9_.]+)\|(\d)\}\}/g;

function walk(result, path) {
  let v = result;
  for (const k of path.split(".")) {
    if (Array.isArray(v) && !/^\d+$/.test(k)) v = v.find((e) => e && e.id === k);
    else v = v && typeof v === "object" ? v[k] : undefined;
  }
  return v && typeof v === "object" && "value" in v ? v.value : v;
}

/** Text of units.json with each {{path|d}} token replaced by the engine value. */
export function fillText(text, result) {
  return String(text || "").replace(TOKEN, (_, path, d) => {
    const v = walk(result, path);
    if (typeof v !== "number" || !Number.isFinite(v)) return "n/a";
    return v.toLocaleString("en-GB", { minimumFractionDigits: Number(d), maximumFractionDigits: Number(d) });
  });
}

function valueHtml(v, unit) {
  const f = fmtValue(v, unit);
  return `${esc(f.text)}${f.unit ? `<span class="u">${esc(f.unit)}</span>` : ""}`;
}

function classChip(cls) {
  if (!cls) return `<span class="chip cl cl-na">No value</span>`;
  const key = String(cls);
  return `<span class="chip cl cl-${esc(key)}">${esc(CLASS[key] || key)}</span>`;
}

function itemRow(e) {
  const na = e.value === null || e.value === undefined || (typeof e.value === "number" && !Number.isFinite(e.value));
  const meta = na
    ? `${classChip(null)}<span class="src">Not produced by the engine: ${esc(e.src || "no reason given")}</span>`
    : `${classChip(e.class)}<span class="src">${esc(e.src || "")}</span>`;
  return `<div class="prow${na ? " is-na" : ""}" data-item="${esc(e.id)}">
    <span class="p-label">${esc(pretty(e.label))}</span>
    <span class="p-value">${valueHtml(na ? null : e.value, e.unit)}</span>
    <span class="p-meta">${meta}</span>
  </div>`;
}

/**
 * Facts of a stream the drawing names. A tag that is a stream id of
 * results/Integrated_Mass_Balance.csv takes its conditions and its boundary
 * flag from that file's headline flowsheet run (data/streams.json), labelled
 * as precomputed, since the live reply carries no stream table. A drawn line
 * with no row in that file says why. `flows` is a Map of stream id to row,
 * null while it loads, or false when the file is absent.
 */
export function streamFacts(id, defs, unitLabel, flows) {
  const def = (defs.streams || {})[id] || { name: id };
  const from = def.from ? unitLabel(def.from) : def.from_text || "the battery limit";
  const to = def.to ? unitLabel(def.to) : def.to_text || "the plant exit";
  const out = { id, name: def.name, from, to, engine: def.engine !== false, crosses: false, drawn: def.drawn || "", data: null };
  if (!out.engine) {
    out.data = { na: true, text: def.note || "no row in the stream table" };
    return out;
  }
  if (flows === null || flows === undefined) {
    out.data = { na: true, text: "loading the precomputed stream table" };
    return out;
  }
  const r = flows ? flows.get(id) : null;
  if (!r) {
    out.data = { na: true, text: flows === false ? "data/streams.json is not part of this build" : `no row ${id} in the stream table` };
    return out;
  }
  out.crosses = r.crosses_boundary === true;
  const t = fmtValue(r.t_c, "C");
  const p = fmtValue(r.p_bar, "bar");
  const m = fmtValue(r.total_kg_h, "kg/h");
  out.data = { na: false, text: `${t.text} °C, ${p.text} bar, ${m.text} kg/h`, flow: r.total_kg_h, basis: `precomputed, flowsheet run ${r.run_id || "FS_B_REC_F"} at the headline basis` };
  return out;
}

function streamRow(id, defs, unitLabel, flows) {
  const f = streamFacts(id, defs, unitLabel, flows);
  const meta = [];
  if (!f.data.na) meta.push(`<span class="chip">Precomputed</span>${esc(f.data.text)}, ${esc(f.data.basis)}`);
  if (f.crosses) meta.push("Crosses the dashed boundary");
  if (f.drawn) meta.push(esc(f.drawn));
  return `<div class="prow${f.data.na ? " is-na" : ""}" data-stream-ref="${esc(id)}">
    <span class="p-label"><span class="p-tag">${esc(id)}</span>${esc(f.name)}</span>
    <span class="p-value">${f.data.na ? "n/a" : valueHtml(f.data.flow, "kg/h")}</span>
    ${meta.length ? `<span class="p-meta"><span class="src">${meta.join(". ")}</span></span>` : ""}
  </div>`;
}

/** Stream rows of one section, with the reason printed once for lines that have no row. */
function streamRows(ids, defs, unitLabel, flows) {
  if (!ids || !ids.length) return [];
  const rows = ids.map((s) => streamRow(s, defs, unitLabel, flows));
  // Lines that share a reason print it once.
  const why = new Map();
  for (const s of ids) {
    const f = streamFacts(s, defs, unitLabel, flows);
    if (f.data.na) why.set(f.data.text, [...(why.get(f.data.text) || []), s]);
  }
  for (const [text, list] of why) rows.push(`<p class="empty-note">${esc(list.join(", "))}: ${esc(text)}.</p>`);
  return rows;
}

function sectionsText(list) {
  const nums = list.filter((s) => /^\d/.test(s));
  const apps = list.filter((s) => !/^\d/.test(s)).map((s) => `Appendix ${s}`);
  const parts = [];
  if (nums.length) parts.push(`${nums.length > 1 ? "sections" : "section"} ${nums.join(", ")}`);
  parts.push(...apps);
  if (parts.length <= 1) return parts.join("");
  return `${parts.slice(0, -1).join(", ")} and ${parts[parts.length - 1]}`;
}

function sourceItem(s, defs) {
  const ref = (defs.refs || {})[s.ref] || { cite: s.ref, report: [] };
  const report = Array.isArray(ref.report) ? ref.report : [];
  const traces = [];
  if (ref.unread) traces.push(`<span class="chip unverified">Not verified</span><span>${esc(ref.unread)}</span>`);
  if (report.length) traces.push(`<span>Traced to the project report, ${esc(sectionsText(report))}</span>`);
  else if (!ref.unread) traces.push(`<span class="chip">Not in the report</span><span>no section of the project report cites it</span>`);
  if (s.eq && s.eq.length) traces.push(`<span>Derivations Eq. ${esc(s.eq.join(", "))}</span>`);
  return `<li data-ref="${esc(s.ref)}">${esc(ref.cite)}${s.locator ? `. <span class="loc">${esc(s.locator)}</span>` : ""}
    <div class="trace">${traces.join("")}</div></li>`;
}

function validationRows(ids, validation) {
  if (!ids || !ids.length) return "";
  const rows = ids.map((id) => {
    if (validation === null || validation === undefined) return `<li><code>${esc(id)}</code>: loading the validation record</li>`;
    if (validation === false) return `<li><code>${esc(id)}</code>: the validation record (data/validation.json) is not part of this build</li>`;
    const r = validation.get(id);
    if (!r) return `<li><code>${esc(id)}</code>: no row with this id in results/Validation_Results.csv</li>`;
    const lit = fmtValue(typeof r.literature === "number" ? r.literature : null, r.unit);
    const sim = fmtValue(typeof r.simulated === "number" ? r.simulated : null, r.unit);
    const u = lit.unit ? ` ${esc(lit.unit)}` : "";
    const vs = typeof r.status === "string" ? r.status : "";
    const chip = `<span class="chip ${vs === "pass" ? "v-pass" : vs === "fail" ? "v-fail" : "v-other"}">${esc(VSTATUS[vs] || vs || "n/a")}</span>`;
    const litText = typeof r.literature === "number" ? `literature ${esc(lit.text)}${u}, ` : "";
    return `<li><code>${esc(id)}</code> ${esc(r.subject || "")}: ${litText}simulated ${esc(sim.text)}${u} ${chip}
      ${r.explanation ? `<span class="vnote">${esc(r.explanation)}</span>` : ""}</li>`;
  });
  return `<ul class="vrows">${rows.join("")}</ul>`;
}

function limitationItem(l, validation, result) {
  const text = fillText(l.text, result);
  const body = l.tag ? `<span class="lim-tag">${esc(l.tag)}.</span> ${esc(text)}` : esc(text);
  return `<li>${body}${validationRows(l.validation, validation)}</li>`;
}

/** The parts of one panel, as arrays of HTML strings per section. */
function parts(id, result, defs, ctx) {
  const def = (defs.units || {})[id] || {};
  const u = result && result.units ? result.units[id] : null;
  const unitLabel = ctx.unitLabel;
  const live = (k) => (u && Array.isArray(u[k]) ? u[k] : []);
  const out = {};
  out.inputs = [...live("inputs").map(itemRow), ...streamRows(def.in, defs, unitLabel, ctx.flows)];
  out.outputs = [...live("outputs").map(itemRow), ...streamRows(def.out, defs, unitLabel, ctx.flows)];
  if (def.outputs_note) out.outputs.push(`<p class="empty-note">${esc(fillText(def.outputs_note, result))}</p>`);
  out.conditions = live("conditions").map(itemRow);
  out.equations = (def.equations || []).map((e) => `<div class="eq"><div class="eq-head"><b>Eq. ${esc(e.label)}</b>${esc(e.title)}</div><div class="eq-body">${e.html}</div></div>`);
  out.sources = (def.sources || []).map((s) => sourceItem(s, defs));
  out.results = live("results").map(itemRow);
  const lim = (def.limitations || []).map((l) => limitationItem(l, ctx.validation, result));
  const flagIds = new Set(def.flags || []);
  for (const f of (result && result.flags) || []) {
    if (flagIds.has(f.param)) lim.push(`<li><span class="live">Input outside its evidence range.</span> ${esc(f.text)}</li>`);
  }
  for (const msg of (result && result.messages) || []) {
    if (msg.unit === id) lim.push(`<li><span class="live">Engine message.</span> ${esc(msg.text)}</li>`);
  }
  out.limitations = lim;
  return { def, u, out };
}

/** Count of entries in each of the seven sections, for the self-test. */
export function sectionCounts(id, result, defs, ctx) {
  const { out } = parts(id, result, defs, ctx);
  return Object.fromEntries(SECTIONS.map(([k]) => [k, out[k].length]));
}

function statusChips(id, u, regUnit) {
  const src = u || regUnit || {};
  const st = src.status;
  const chips = [`<span class="chip st-${esc(st || "na")}" title="Model status from the engine">${esc(STATUS[st] || st || "Status n/a")}</span>`];
  if (src.term) {
    const used = src.used === "anchored" ? "on its anchored value" : src.used === "modelled" ? "modelled" : "";
    const gate = src.gate && src.gate !== "none" ? `, gate ${src.gate}` : "";
    chips.push(`<span class="chip">Term ${esc(src.term)}${used ? ` ${esc(used)}` : ""}${esc(gate)}</span>`);
  }
  return { chips: chips.join(""), basis: src.basis || "" };
}

/** HTML of the panel for unit `id`. */
export function renderPanel(id, result, defs, ctx) {
  const { def, u, out } = parts(id, result, defs, ctx);
  const regUnit = ctx.registry && ctx.registry.units ? ctx.registry.units[id] : null;
  const label = (u && u.label) || (regUnit && regUnit.label) || id;
  const s = statusChips(id, u, regUnit);
  const stale = ctx.stale ? `<p class="basis">Values are the last converged result.</p>` : "";
  const noResult = !u ? `<p class="basis">The engine has not returned a result for this unit yet, so live values show n/a.</p>` : "";
  const body = SECTIONS.map(([k, title]) => {
    const items = out[k];
    let inner;
    if (!items.length) inner = `<p class="empty-note">The engine reports no entry here for this unit.</p>`;
    else if (k === "sources") inner = `<ul class="src-list">${items.join("")}</ul>`;
    else if (k === "limitations") inner = `<ul class="lim-list">${items.join("")}</ul>`;
    else inner = items.join("");
    return `<section class="psec" data-section="${k}"><h4>${title}</h4>${inner}</section>`;
  }).join("");
  return `<article class="unit-panel" data-panel="${esc(id)}" aria-labelledby="panel-title-${esc(id)}">
    <header>
      <h3 id="panel-title-${esc(id)}">${esc(label)}</h3>
      ${def.summary ? `<p class="summary">${esc(fillText(def.summary, result))}</p>` : ""}
      <div class="chips">${s.chips}</div>
      ${s.basis ? `<p class="basis">Status basis: ${esc(s.basis)}.</p>` : ""}
      ${stale}${noResult}
    </header>
    ${body}
  </article>`;
}
