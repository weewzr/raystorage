// Interactive PFD: unit selection by pointer and keyboard, stream tooltips,
// the engine's model status on each unit, and the unit panel.
//
// Labels on the drawing carry data-bind paths that the page shell fills from
// the engine reply. Panel values come from the engine reply and from CI JSON.
// This file positions and formats; it holds no model arithmetic.

import { renderPanel, sectionCounts, streamFacts, STATUS, SECTIONS, esc } from "./panels.js?v=__COMMIT__";

let api = null;
let defs = null;
let defsError = null;
// Validation record: undefined until requested, null while loading, false when
// absent from this build, otherwise a Map from test_id to row.
let validation;
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
    const res = await fetch(new URL("units.json?v=__COMMIT__", import.meta.url));
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
    const res = await fetch(new URL("data/validation.json?v=__COMMIT__", import.meta.url));
    if (!res.ok) throw new Error(String(res.status));
    const data = await res.json();
    validation = new Map((data.rows || []).map((r) => [r.test_id, r]));
  } catch {
    validation = false;
  }
  if (api.state.route === "pfd") renderPanelNow();
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
  const res = result();
  const d = defs || { streams: {} };
  const f = streamFacts(id, res, d, unitLabel);
  const data = f.data.na ? `<div class="tip-na">Stream conditions: n/a, ${esc(f.data.text)}.</div>` : `<div class="tip-row">${esc(f.data.text)}</div>`;
  return `${withLink ? '<button type="button" class="tip-close" aria-label="Close">×</button>' : ""}
    <span class="tip-id">${esc(id)}</span>${esc(f.name)}
    <div class="tip-row">From ${esc(f.from)} to ${esc(f.to)}${f.crosses ? ". Crosses the dashed boundary" : ""}.</div>
    ${data}
    ${withLink ? `<a href="#results" data-stream-link="${esc(id)}">Stream table in Results</a>` : ""}`;
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

function bindStreams() {
  for (const g of el.svg.querySelectorAll("[data-stream]")) {
    g.addEventListener("pointermove", (e) => { if (!pinned) showTip(g, e, false); });
    g.addEventListener("pointerleave", () => { if (!pinned) hideTip(); });
    g.addEventListener("click", (e) => { e.stopPropagation(); showTip(g, e, true); });
  }
  el.tip.addEventListener("click", (e) => {
    if (e.target.closest(".tip-close")) { hideTip(); return; }
    const a = e.target.closest("[data-stream-link]");
    if (a) {
      const id = a.dataset.streamLink;
      hideTip();
      document.dispatchEvent(new CustomEvent("nush2:stream", { detail: { id } }));
      setTimeout(() => {
        const row = document.querySelector(`[data-stream-row="${CSS.escape(id)}"]`);
        if (row) row.scrollIntoView({ block: "center" });
      }, 350);
    }
  });
  el.svg.addEventListener("click", (e) => { if (pinned && !e.target.closest("[data-stream]")) hideTip(); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !el.tip.hidden) hideTip(); });
}

// ------------------------------------------------------------ panel

function ctx() {
  return {
    registry: api.state.registry,
    validation,
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
  if (api.state.route === "pfd") onRoute();
}
