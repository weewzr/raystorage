// Interactive PFD: unit selection by pointer and keyboard, stream tooltips,
// the engine's model status on each unit, and the unit panel.
//
// Labels on the drawing carry data-bind paths that the page shell fills from
// the engine reply. Panel values come from the engine reply and from CI JSON.
// This file positions and formats; it holds no model arithmetic.

import { renderPanel, sectionCounts, streamFacts, STATUS, SECTIONS, esc } from "./panels.js?v=03fe6b2eb558ba5c179dbf7337548d662fb078aa";

let api = null;
let defs = null;
let defsError = null;
// Validation record: undefined until requested, null while loading, false when
// absent from this build, otherwise a Map from test_id to row.
let validation;
// Precomputed stream rows of the headline flowsheet run, data/streams.json:
// null while loading, false when absent, otherwise a Map from stream id to row.
let flows = null;
let selected = null;
let pinned = null;
const el = {};

const unitEls = () => Array.from(el.svg.querySelectorAll("[data-unit]"));

function result() {
  return api.state.lastOk || api.state.result || null;
}

function unitLabel(id) {
  const reg = api.state.registry;
  const res = result();
  if (res && res.units && res.units[id] && res.units[id].label) return res.units[id].label;
  if (reg && reg.units && reg.units[id] && reg.units[id].label) return reg.units[id].label;
  const g = el.svg.querySelector(`[data-unit="${CSS.escape(id)}"]`);
  return g ? g.getAttribute("aria-label").split(",")[0] : id;
}

function unitStatus(id) {
  const res = result();
  if (res && res.units && res.units[id]) return res.units[id].status;
  const reg = api.state.registry;
  return reg && reg.units && reg.units[id] ? reg.units[id].status : null;
}

function order() {
  if (defs && Array.isArray(defs.order)) return defs.order;
  return unitEls().map((g) => g.dataset.unit);
}

// ------------------------------------------------------------ data

async function loadDefs() {
  try {
    const res = await fetch(new URL("units.json?v=03fe6b2eb558ba5c179dbf7337548d662fb078aa", import.meta.url));
    if (!res.ok) throw new Error(`units.json could not be fetched (HTTP ${res.status})`);
    defs = await res.json();
    const cap = document.getElementById("pfd-caption");
    if (cap && defs.caption) cap.textContent = defs.caption;
  } catch (err) {
    defsError = String(err && err.message ? err.message : err);
    console.error(`PFD panel text: ${defsError}`);
  }
}

async function loadValidation() {
  validation = null;
  try {
    const res = await fetch(new URL("data/validation.json?v=03fe6b2eb558ba5c179dbf7337548d662fb078aa", import.meta.url));
    if (!res.ok) throw new Error(String(res.status));
    const data = await res.json();
    validation = new Map((data.rows || []).map((r) => [r.test_id, r]));
  } catch {
    validation = false;
  }
  if (api.state.route === "pfd") renderPanelNow();
}

async function loadFlows() {
  try {
    const res = await fetch(new URL("data/streams.json?v=03fe6b2eb558ba5c179dbf7337548d662fb078aa", import.meta.url));
    if (!res.ok) throw new Error(String(res.status));
    const data = await res.json();
    flows = new Map((data.rows || []).map((r) => [r.stream_id, r]));
  } catch {
    flows = false;
  }
  labelStreams();
  if (api.state.route === "pfd") renderPanelNow();
}

/** Accessible name of each stream line: tag, name and ends. */
function labelStreams() {
  if (!defs) return;
  for (const g of el.svg.querySelectorAll("[data-stream]")) {
    const f = streamFacts(g.dataset.stream, defs, unitLabel, flows);
    g.setAttribute("aria-label", `Stream ${f.id}, ${f.name}, from ${f.from} to ${f.to}`);
  }
}

// ------------------------------------------------------------ status on the drawing

function paintUnits() {
  for (const g of unitEls()) {
    const id = g.dataset.unit;
    const st = unitStatus(id);
    g.classList.toggle("is-external", st === "external");
    if (st) g.dataset.status = st;
    g.setAttribute("aria-label", `${unitLabel(id)}, ${STATUS[st] || "status loading"}`);
  }
  const sel = el.select;
  if (sel && sel.options.length !== order().length) {
    sel.innerHTML = order().map((id) => `<option value="${esc(id)}">${esc(unitLabel(id))}</option>`).join("");
  } else if (sel) {
    for (const o of sel.options) o.textContent = unitLabel(o.value);
  }
  if (sel && selected) sel.value = selected;
}

// ------------------------------------------------------------ selection

function select(id, opts = {}) {
  if (!id) return;
  selected = id;
  for (const g of unitEls()) g.setAttribute("aria-pressed", g.dataset.unit === id ? "true" : "false");
  if (el.select) el.select.value = id;
  if (opts.reveal) reveal(id);
  renderPanelNow();
}

/** Scroll the sideways scroller so the unit sits in view. Display geometry only. */
function reveal(id) {
  const g = el.svg.querySelector(`[data-unit="${CSS.escape(id)}"]`);
  const box = el.scroll;
  if (!g || !box || box.scrollWidth <= box.clientWidth) return;
  const r = g.getBoundingClientRect();
  const b = box.getBoundingClientRect();
  const target = box.scrollLeft + (r.left - b.left) - (b.width - r.width) / 2;
  box.scrollTo({ left: Math.max(0, target), behavior: "smooth" });
}

function move(from, step) {
  const ids = order();
  const i = ids.indexOf(from);
  const next = ids[(i + step + ids.length) % ids.length];
  const g = el.svg.querySelector(`[data-unit="${CSS.escape(next)}"]`);
  if (g) {
    g.focus();
    reveal(next);
  }
}

function bindUnits() {
  for (const g of unitEls()) {
    const id = g.dataset.unit;
    g.addEventListener("click", () => select(id));
    g.addEventListener("keydown", (e) => {
      const ids = order();
      if (e.key === "Enter" || e.key === " " || e.key === "Spacebar") {
        e.preventDefault();
        select(id);
      } else if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        e.preventDefault();
        move(id, 1);
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        e.preventDefault();
        move(id, -1);
      } else if (e.key === "Home") {
        e.preventDefault();
        move(ids[1] || id, -1);
      } else if (e.key === "End") {
        e.preventDefault();
        move(ids[ids.length - 2] || id, 1);
      }
    });
  }
}

// ------------------------------------------------------------ stream tooltip

function tipHtml(id, withLink) {
  const d = defs || { streams: {} };
  const f = streamFacts(id, d, unitLabel, flows);
  const data = f.data.na
    ? `<div class="tip-na">${f.engine ? "Stream conditions: n/a, " : "No row in the stream table. "}${esc(f.data.text)}.</div>`
    : `<div class="tip-row">${esc(f.data.text)}</div><div class="tip-na">Precomputed at the headline basis; it does not follow the controls.</div>`;
  const link = withLink && f.engine && !f.data.na;
  return `${withLink ? '<button type="button" class="tip-close" aria-label="Close">×</button>' : ""}
    <span class="tip-id">${esc(id)}</span>${esc(f.name)}
    <div class="tip-row">From ${esc(f.from)} to ${esc(f.to)}${f.crosses ? ". Crosses the dashed boundary" : ""}.${f.drawn ? ` ${esc(f.drawn)}.` : ""}</div>
    ${data}
    ${link ? `<a href="#results" data-stream-link="${esc(id)}">Row ${esc(id)} of the stream table in Results</a>` : ""}`;
}

function placeTip(clientX, clientY) {
  const tip = el.tip;
  const host = el.mount.getBoundingClientRect();
  const w = tip.offsetWidth;
  let x = clientX - host.left + 14;
  if (x + w > host.width) x = Math.max(0, clientX - host.left - w - 14);
  tip.style.left = `${x}px`;
  tip.style.top = `${clientY - host.top + 14}px`;
}

function showTip(g, e, pin) {
  const id = g.dataset.stream;
  el.tip.innerHTML = tipHtml(id, pin);
  el.tip.hidden = false;
  el.tip.classList.toggle("is-pinned", pin);
  for (const s of el.svg.querySelectorAll(".st.is-tip")) s.classList.remove("is-tip");
  g.classList.add("is-tip");
  placeTip(e.clientX, e.clientY);
  pinned = pin ? id : null;
}

function hideTip() {
  el.tip.hidden = true;
  el.tip.classList.remove("is-pinned");
  for (const s of el.svg.querySelectorAll(".st.is-tip")) s.classList.remove("is-tip");
  pinned = null;
}

/** Screen point beside a stream line, for a tooltip opened from the keyboard. */
function anchorOf(g) {
  const r = g.getBoundingClientRect();
  return { clientX: r.left + r.width / 2, clientY: r.top + r.height / 2 };
}

let tipOwner = null;
// Set while focus goes back to a stream after its tip closed, so the focus
// handler does not open the tip that was just closed.
let refocusing = false;

function focusBack(g) {
  refocusing = true;
  try { g.focus(); } finally { refocusing = false; }
}

function bindStreams() {
  for (const g of el.svg.querySelectorAll("[data-stream]")) {
    // Each stream line is a focus stop: focus shows its tooltip, Enter or
    // Space pins it with the link to its row, Escape closes it.
    g.setAttribute("tabindex", "0");
    g.setAttribute("role", "button");
    g.addEventListener("pointermove", (e) => { if (!pinned) showTip(g, e, false); });
    g.addEventListener("pointerleave", () => { if (!pinned) hideTip(); });
    g.addEventListener("click", (e) => { e.stopPropagation(); tipOwner = g; showTip(g, e, true); });
    g.addEventListener("focus", () => { if (!pinned && !refocusing) showTip(g, anchorOf(g), false); });
    g.addEventListener("blur", (e) => { if (!pinned && !el.tip.contains(e.relatedTarget)) hideTip(); });
    g.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " " || e.key === "Spacebar") {
        e.preventDefault();
        tipOwner = g;
        showTip(g, anchorOf(g), true);
        const a = el.tip.querySelector("[data-stream-link]") || el.tip.querySelector(".tip-close");
        if (a) a.focus();
      }
    });
  }
  el.tip.addEventListener("click", (e) => {
    if (e.target.closest(".tip-close")) {
      hideTip();
      if (tipOwner) focusBack(tipOwner);
      return;
    }
    const a = e.target.closest("[data-stream-link]");
    if (a) {
      // The Results route scrolls to the row once its stream table is drawn;
      // the link's own href then switches the route.
      document.dispatchEvent(new CustomEvent("nush2:stream", { detail: { id: a.dataset.streamLink } }));
      hideTip();
    }
  });
  el.svg.addEventListener("click", (e) => { if (pinned && !e.target.closest("[data-stream]")) hideTip(); });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !el.tip.hidden) {
      const back = pinned && tipOwner;
      hideTip();
      if (back) focusBack(tipOwner);
    }
  });
}

// ------------------------------------------------------------ panel

function ctx() {
  return {
    registry: api.state.registry,
    validation,
    flows,
    unitLabel,
    stale: Boolean(api.state.result && api.state.result.status === "not_converged"),
  };
}

function renderPanelNow() {
  if (!el.panel) return;
  if (!defs) {
    el.panel.innerHTML = defsError
      ? `<p class="banner" data-level="bad">The unit panel text did not load: ${esc(defsError)}</p>`
      : `<p class="empty-note">Loading the unit panels</p>`;
    return;
  }
  el.panel.innerHTML = renderPanel(selected, result(), defs, ctx());
}

function onRoute() {
  paintUnits();
  if (!selected) select(order()[0]);
  else renderPanelNow();
  if (validation === undefined) loadValidation();
}

/** Section counts of every unit panel at the current result, for the self-test. */
export function auditPanels() {
  if (!defs || !el.svg) return { ready: false, units: 0, incomplete: [] };
  const ids = unitEls().map((g) => g.dataset.unit);
  const counts = {};
  const incomplete = [];
  for (const id of ids) {
    counts[id] = sectionCounts(id, result(), defs, ctx());
    const empty = SECTIONS.map(([k]) => k).filter((k) => !counts[id][k]);
    if (empty.length) incomplete.push(`${id}: ${empty.join(", ")}`);
  }
  return { ready: true, units: ids.length, incomplete, counts };
}

/** Wire the drawing. `a` is the page shell's API: state, registerRoute. */
export async function initPfd(a) {
  api = a;
  el.svg = document.getElementById("pfd-svg");
  el.mount = document.getElementById("pfd-mount");
  el.scroll = el.mount ? el.mount.querySelector(".pfd-scroll") : null;
  el.tip = document.getElementById("pfd-tip");
  el.panel = document.getElementById("pfd-panel");
  el.select = document.getElementById("pfd-select");
  if (!el.svg || !el.mount || !el.tip || !el.panel) return;
  bindUnits();
  bindStreams();
  if (el.select) el.select.addEventListener("change", () => select(el.select.value, { reveal: true }));
  api.registerRoute("pfd", onRoute);
  paintUnits();
  await loadDefs();
  labelStreams();
  loadFlows();
  if (api.state.route === "pfd") onRoute();
}

/** Stream tags on the drawing, and the stream ids of data/streams.json, for the self-test. */
export function auditStreams() {
  if (!defs || !el.svg) return { ready: false };
  const tags = [...new Set(Array.from(el.svg.querySelectorAll("[data-stream]")).map((g) => g.dataset.stream))];
  const known = flows instanceof Map ? flows : null;
  const engine = tags.filter((t) => ((defs.streams || {})[t] || {}).engine !== false);
  const other = tags.filter((t) => !engine.includes(t));
  return {
    ready: Boolean(known),
    tags,
    engine,
    other,
    undescribed: tags.filter((t) => !(defs.streams || {})[t]),
    not_in_file: known ? engine.filter((t) => !known.has(t)) : engine,
    clash: known ? other.filter((t) => known.has(t)) : [],
    not_focusable: Array.from(el.svg.querySelectorAll("[data-stream]")).filter((g) => g.getAttribute("tabindex") !== "0").map((g) => g.dataset.stream),
    crossing: known ? engine.filter((t) => known.get(t).crosses_boundary === true) : [],
    noted: tags.filter((t) => ((defs.streams || {})[t] || {}).drawn),
    ringed: Array.from(el.svg.querySelectorAll("[data-stream]")).filter((g) => g.querySelector(".xing")).map((g) => g.dataset.stream),
  };
}
