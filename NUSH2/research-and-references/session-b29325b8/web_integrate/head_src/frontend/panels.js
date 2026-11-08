// Unit panels of the interactive PFD. Each panel has seven sections in a
// fixed order: inputs, outputs, operating conditions, governing equations,
// literature sources, calculated results and model limitations.
//
// Live values come from the engine reply (result.units[id]) and validation
// rows from data/validation.json, which CI builds from results/*.csv. Static
// text comes from units.json. This file formats values for display and holds
// no model arithmetic: a value the engine does not produce prints as n/a with
// the engine's own reason.

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
    <span class="p-label">${esc(e.label)}</span>
    <span class="p-value">${valueHtml(na ? null : e.value, e.unit)}</span>
    <span class="p-meta">${meta}</span>
  </div>`;
}

function liveStream(result, id) {
  const s = result && result.streams;
  if (!Array.isArray(s)) return null;
  return s.find((x) => x && (x.id === id || x.tag === id)) || null;
}

/** Text for a stream the drawing names: live values when the engine has a stream table, else n/a with its reason. */
export function streamFacts(id, result, defs, unitLabel) {
  const def = (defs.streams || {})[id] || { name: id };
  const ends = [def.from ? unitLabel(def.from) : "the battery limit", def.to ? unitLabel(def.to) : "the plant exit"];
  const s = liveStream(result, id);
  let data;
  if (s) {
    const t = fmtValue(s.T_C, "C");
    const p = fmtValue(s.P_bar, "bar");
    const m = fmtValue(s.kg_h, "kg/h");
    data = { na: false, text: `${t.text} °C, ${p.text} bar, ${m.text} kg/h`, flow: s.kg_h, basis: s.basis || "" };
  } else {
    const missing = ((result && result.engine_fields_missing) || []).find((m) => m.field === "streams");
    data = { na: true, text: missing ? missing.reason : "no stream table in this reply" };
  }
  return { id, name: def.name, from: ends[0], to: ends[1], crosses: Boolean(def.crosses), data };
}

function streamRow(id, result, defs, unitLabel) {
  const f = streamFacts(id, result, defs, unitLabel);
  const meta = [];
  if (!f.data.na) meta.push(`${esc(f.data.text)}${f.data.basis ? `, ${esc(f.data.basis)}` : ""}`);
  if (f.crosses) meta.push("Crosses the dashed boundary");
  return `<div class="prow${f.data.na ? " is-na" : ""}" data-stream-ref="${esc(id)}">
    <span class="p-label"><span class="p-tag">${esc(id)}</span>${esc(f.name)}</span>
    <span class="p-value">${f.data.na ? "n/a" : valueHtml(f.data.flow, "kg/h")}</span>
    ${meta.length ? `<span class="p-meta"><span class="src">${meta.join(". ")}</span></span>` : ""}
  </div>`;
}

/** Stream rows of one section, with the engine's reason printed once when they are n/a. */
function streamRows(ids, result, defs, unitLabel) {
  if (!ids || !ids.length) return [];
  const rows = ids.map((s) => streamRow(s, result, defs, unitLabel));
  const na = ids.filter((s) => streamFacts(s, result, defs, unitLabel).data.na);
  if (na.length) {
    const why = streamFacts(na[0], result, defs, unitLabel).data.text;
    rows.push(`<p class="empty-note">Stream conditions of ${esc(na.join(", "))}: n/a, ${esc(why)}.</p>`);
  }
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
  else if (!ref.unread) traces.push(`<span class="chip unverified">Not verified</span><span>no section of the project report cites it</span>`);
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

function limitationItem(l, validation) {
  const m = /^(Stage \d [^:]+):\s*(.*)$/s.exec(l.text);
  const rest = m ? m[2].charAt(0).toUpperCase() + m[2].slice(1) : "";
  const body = m ? `<span class="lim-tag">${esc(m[1])}.</span> ${esc(rest)}` : esc(l.text);
  return `<li>${body}${validationRows(l.validation, validation)}</li>`;
}

/** The parts of one panel, as arrays of HTML strings per section. */
function parts(id, result, defs, ctx) {
  const def = (defs.units || {})[id] || {};
  const u = result && result.units ? result.units[id] : null;
  const unitLabel = ctx.unitLabel;
  const live = (k) => (u && Array.isArray(u[k]) ? u[k] : []);
  const out = {};
  out.inputs = [...live("inputs").map(itemRow), ...streamRows(def.in, result, defs, unitLabel)];
  out.outputs = [...live("outputs").map(itemRow), ...streamRows(def.out, result, defs, unitLabel)];
  if (def.outputs_note) out.outputs.push(`<p class="empty-note">${esc(def.outputs_note)}</p>`);
  out.conditions = live("conditions").map(itemRow);
  out.equations = (def.equations || []).map((e) => `<div class="eq"><div class="eq-head"><b>Eq. ${esc(e.label)}</b>${esc(e.title)}</div><div class="eq-body">${e.html}</div></div>`);
  out.sources = (def.sources || []).map((s) => sourceItem(s, defs));
  out.results = live("results").map(itemRow);
  const lim = (def.limitations || []).map((l) => limitationItem(l, ctx.validation));
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
      ${def.summary ? `<p class="summary">${esc(def.summary)}</p>` : ""}
      <div class="chips">${s.chips}</div>
      ${s.basis ? `<p class="basis">Status basis: ${esc(s.basis)}.</p>` : ""}
      ${stale}${noResult}
    </header>
    ${body}
  </article>`;
}
