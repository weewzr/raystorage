// NUS-H2 page shell: hash routes, worker RPC, status line and banners.
// Every engineering number on the page comes from the engine reply or from CI
// JSON. This file formats numbers for display and holds no model arithmetic.

const ROUTES = ["dashboard", "pfd", "parameters", "results", "export", "validation"];
const TIMEOUT_MS = 10000;
const DEBOUNCE_MS = 200;
const STRIP = [
  "rating_mwth", "reactor_required_mwth", "surplus_mwth", "q_process_mwth",
  "h2_product_t_y", "ng_mmscfd", "co2_captured_t_y", "capture_plant",
  "q_reboiler_mw", "e_aux_mwe", "abatement_t_y", "cost_s_per_t", "cost_lac_dcf_s_per_t",
];
const DECIMALS = {
  "t/y": 0, "kg/h": 0, "kmol/h": 0, "t/h": 0, "MWth": 1, "MW": 1, "MWe": 2,
  "MMSCFD": 2, "S$/t": 1, "S$/kg": 2, "GJ/t": 2, "Nm3/h": 0,
};

export const state = {
  registry: null,
  inputs: {},
  result: null,
  lastOk: null,
  route: "dashboard",
  seq: 0,
  ms: null,
};

const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
export const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

// ------------------------------------------------------------ self-test hook

const SELFTEST = new URLSearchParams(location.search).get("selftest") === "1";
const caught = [];
if (SELFTEST) {
  window.addEventListener("error", (e) => caught.push(`error: ${e.message || e}`));
  window.addEventListener("unhandledrejection", (e) => caught.push(`rejection: ${e.reason}`));
  const ce = console.error.bind(console);
  console.error = (...a) => { caught.push(`console.error: ${a.map(String).join(" ")}`); ce(...a); };
}
let reported = false;

function selftestReport(extra) {
  if (!SELFTEST || reported) return;
  reported = true;
  const reg = state.registry;
  const res = state.result;
  const required = reg ? reg.kpis.filter((k) => k.required).map((k) => k.id) : [];
  const kpi = res && res.kpi ? res.kpi : {};
  const missingKeys = required.filter((k) => !(k in kpi));
  const verdict = {
    registry_loaded: Boolean(reg && reg.status === "ok"),
    params: reg ? reg.params.length : 0,
    engine_version: reg ? reg.engine.version : null,
    engine_commit: reg ? reg.engine.commit : null,
    engine_line: $("#engine-line").textContent,
    solve_status: res ? res.status : null,
    solve_ms: state.ms,
    missing_kpi_keys: missingKeys,
    kpi,
    errors: caught.slice(),
    ...extra,
  };
  const pass = verdict.registry_loaded && verdict.solve_status === "ok" && missingKeys.length === 0 && caught.length === 0;
  verdict.verdict = pass ? "pass" : "fail";
  const pre = document.createElement("pre");
  pre.id = "selftest-report";
  pre.className = "visually-hidden";
  pre.dataset.verdict = verdict.verdict;
  pre.textContent = JSON.stringify(verdict);
  document.body.appendChild(pre);
  document.documentElement.dataset.selftest = verdict.verdict;
}

// ------------------------------------------------------------ formatting

export function unitLabel(unit) {
  return unit === "-" ? "%" : unit;
}

export function fmt(v, unit) {
  if (v === null || v === undefined || typeof v !== "number" || !Number.isFinite(v)) return "n/a";
  if (unit === "-") return (v * 100).toLocaleString("en-GB", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  const d = unit in DECIMALS ? DECIMALS[unit] : 2;
  return v.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d });
}

function kpiDef(id) {
  return state.registry ? state.registry.kpis.find((k) => k.id === id) : null;
}

function missingReason(res, field) {
  if (!res || !Array.isArray(res.engine_fields_missing)) return "";
  const m = res.engine_fields_missing.find((x) => x.field === field);
  return m ? m.reason : "";
}

// ------------------------------------------------------------ status and banner

function setStatus(kind, text) {
  const bar = $("#statusbar");
  bar.dataset.state = kind;
  $("#status-text").textContent = text;
}

function banner(level, html) {
  const b = $("#banner");
  if (!level) { b.hidden = true; b.innerHTML = ""; return; }
  b.dataset.level = level;
  b.innerHTML = html;
  b.hidden = false;
}

function setStale(on) {
  $("#stale-tag").hidden = !on;
  $("#kpi-strip").classList.toggle("stale", on);
}

function paramDef(id) {
  return state.registry ? state.registry.params.find((p) => p.id === id) : null;
}

function violationText(v) {
  const p = paramDef(v.param);
  const label = p ? p.label : v.param;
  const unit = p && p.unit ? ` ${p.unit}` : "";
  const value = typeof v.value === "number" ? `${v.value}${unit}` : JSON.stringify(v.value);
  if (typeof v.min === "number" && typeof v.max === "number") {
    return `${label} at ${value} lies outside ${v.min} to ${v.max}${unit}. Rule: ${v.rule}.`;
  }
  return `${label} at ${value}: ${v.rule}.`;
}

function markFields(violations) {
  $$("[data-param]").forEach((el) => { el.classList.remove("invalid"); el.removeAttribute("aria-invalid"); });
  for (const v of violations || []) {
    $$(`[data-param="${CSS.escape(String(v.param))}"]`).forEach((el) => { el.classList.add("invalid"); el.setAttribute("aria-invalid", "true"); });
  }
}

// ------------------------------------------------------------ worker RPC

let worker = null;
const pending = new Map();
let rpcId = 0;

function startWorker() {
  worker = new Worker("worker.js?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2");
  worker.onmessage = (e) => {
    const { id, ok, text, error, ms, trapped } = e.data;
    const job = pending.get(id);
    if (!job) return;
    pending.delete(id);
    clearTimeout(job.timer);
    if (!ok) {
      const err = new Error(error);
      err.trapped = Boolean(trapped);
      job.rej(err);
      return;
    }
    let reply;
    try { reply = JSON.parse(text); } catch (err) { job.rej(new Error("The engine reply is not JSON")); return; }
    job.res({ reply, ms });
  };
  worker.onerror = (e) => {
    if (e.preventDefault) e.preventDefault();
    const err = new Error(e.message || "The engine worker failed");
    for (const [, job] of pending) { clearTimeout(job.timer); job.rej(err); }
    pending.clear();
    restartWorker();
  };
}

function restartWorker() {
  if (worker) worker.terminate();
  startWorker();
}

/** Post one command to the worker. Rejects after 10 s and restarts the worker. */
export function rpc(cmd, body) {
  return new Promise((res, rej) => {
    const id = ++rpcId;
    const timer = setTimeout(() => {
      pending.delete(id);
      const err = new Error("Timed out");
      err.timeout = true;
      for (const [, job] of pending) { clearTimeout(job.timer); job.rej(new Error("The engine was restarted after a timeout")); }
      pending.clear();
      restartWorker();
      rej(err);
    }, TIMEOUT_MS);
    pending.set(id, { res, rej, timer });
    worker.postMessage({ id, cmd, body: JSON.stringify(body) });
  });
}

// ------------------------------------------------------------ solve, latest wins

let running = false;
let queued = null;
let debounce = null;
const resultHooks = [];
const routeHooks = {};

export function onResult(fn) { resultHooks.push(fn); }
export function registerRoute(name, fn) { routeHooks[name] = fn; }

export function setInput(id, value) {
  state.inputs[id] = value;
  scheduleSolve();
}

export function resetInputs() {
  state.inputs = defaults();
  scheduleSolve();
}

export function scheduleSolve() {
  clearTimeout(debounce);
  debounce = setTimeout(requestSolve, DEBOUNCE_MS);
}

export function requestSolve() {
  queued = { seq: ++state.seq, inputs: { ...state.inputs } };
  pump();
}

async function pump() {
  if (running || !queued) return;
  const job = queued;
  queued = null;
  running = true;
  setStatus("busy", "Solving");
  try {
    const { reply, ms } = await rpc("solve", { schema: 1, inputs: job.inputs, kpi_only: false });
    if (job.seq === state.seq) applyReply(reply, ms);
  } catch (err) {
    if (job.seq === state.seq) applyFailure(err, job);
  } finally {
    running = false;
    if (queued) pump();
    else if (job.seq === state.seq) selftestReport({});
  }
}

function applyReply(reply, ms) {
  state.ms = Math.round(ms);
  markFields([]);
  if (reply.status === "ok") {
    state.result = reply;
    state.lastOk = reply;
    setStale(false);
    banner(null);
    setStatus("ok", `Solved in ${state.ms} ms`);
  } else if (reply.status === "rejected") {
    markFields(reply.violations);
    const lines = (reply.violations || []).map(violationText);
    const msgs = (reply.messages || []).filter((m) => m.level === "error").map((m) => esc(m.text));
    banner("warn", `<strong>Input refused.</strong> ${[...lines.map(esc), ...msgs].join(" ")} The previous result is kept.`);
    setStatus("warn", "Input refused");
  } else if (reply.status === "not_converged") {
    state.result = reply;
    setStale(true);
    const err = (reply.messages || []).find((m) => m.level === "error");
    const where = err ? `${esc(err.unit)}: ${esc(err.text)}` : "no unit named";
    banner("bad", `<strong>Not converged.</strong> ${where}. The tiles show the last converged result.`);
    setStatus("bad", "Not converged");
  } else {
    banner("bad", `<strong>Unexpected reply.</strong> ${esc(JSON.stringify(reply.messages || reply.status))}`);
    setStatus("bad", "Unexpected reply");
  }
  render();
}

function applyFailure(err, job) {
  setStale(Boolean(state.lastOk));
  if (err.timeout) {
    setStatus("bad", "Timed out");
    banner("bad", `<strong>Timed out.</strong> The engine took longer than ${TIMEOUT_MS / 1000} s and was restarted. The tiles show the last converged result.`);
  } else if (err.trapped) {
    setStatus("bad", "Engine stopped");
    banner("bad", `<strong>The engine stopped on these inputs and was restarted.</strong> <code>${esc(JSON.stringify(job.inputs))}</code> ${esc(err.message)}`);
  } else {
    setStatus("bad", "Engine failed");
    banner("bad", `<strong>Engine failed.</strong> ${esc(err.message)}`);
  }
  render();
}

// ------------------------------------------------------------ render

function defaults() {
  const out = {};
  if (state.registry) for (const p of state.registry.params) out[p.id] = p.default;
  return out;
}

function renderStrip() {
  const res = state.lastOk;
  const el = $("#kpi-strip");
  if (!state.registry) return;
  el.innerHTML = STRIP.map((id) => {
    const def = kpiDef(id);
    if (!def) return "";
    const v = res && res.kpi ? res.kpi[id] : null;
    const na = v === null || v === undefined;
    const reason = na ? missingReason(res, `kpi.${id}`) : "";
    return `<div class="kpi${na ? " is-na" : ""}" data-kpi="${esc(id)}">
      <span class="k-label">${esc(def.label)}</span>
      <span class="k-value">${fmt(v, def.unit)}<span class="k-unit">${esc(unitLabel(def.unit))}</span></span>
      ${reason ? `<span class="k-note">${esc(reason)}</span>` : ""}
    </div>`;
  }).join("");
}

const TICK = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8.5l3 3 7-7" fill="none" stroke="currentColor" stroke-width="2"/></svg>';
const CROSS = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" fill="none" stroke="currentColor" stroke-width="2"/></svg>';

function renderTargets() {
  const t = state.lastOk && state.lastOk.targets;
  const el = $("#targets");
  if (!t) { el.innerHTML = ""; return; }
  const a = t.abatement;
  const c = t.cost;
  el.innerHTML = [
    `<span class="target" data-met="${a.met}">${a.met ? TICK : CROSS}Abatement of at least ${fmt(a.target, "t/y")} t/y: ${a.met ? "Met" : "Not met at these inputs"}</span>`,
    `<span class="target" data-met="${c.met}">${c.met ? TICK : CROSS}Cost at or below S$${Number(c.target).toLocaleString("en-GB", { maximumFractionDigits: 1 })}/t on the L5 ledger and the DCF: ${c.met ? "Met at these inputs" : "Not met at these inputs"}</span>`,
  ].join("");
}

function renderFlags() {
  const res = state.result;
  const el = $("#flag-list");
  const flags = res && Array.isArray(res.flags) ? res.flags : [];
  el.innerHTML = flags.map((f) => `<li>${esc(f.text)}</li>`).join("");
}

function renderBasis() {
  const reg = state.registry;
  if (!reg) return;
  const h2 = reg.fixed.find((f) => f.id === "H2_B");
  const avail = state.inputs.availability;
  const pct = typeof avail === "number" ? `${(avail * 100).toLocaleString("en-GB", { maximumFractionDigits: 1 })}%` : "n/a";
  const text = state.inputs.basis === "case_a"
    ? `Case A: the INL reference plant at ${pct} availability, compared with a same-size unabated SMR. Hydrogen counted only in availability hours; backup firing is a sensitivity.`
    : `Case B: ${fmt(h2 ? h2.value : null, "Nm3/h")} Nm³/h H₂ at ${pct} availability, new-build at SOXAL scale, compared with a same-size unabated SMR. Proposed Western Island site, a team proposal with no official approval. Hydrogen counted only in availability hours; backup firing is a sensitivity.`;
  $("#basis-line").textContent = text;
}

// A data-bind path walks the reply. Inside an array, a name that is not an
// index picks the entry with that id, so "units.htgr.conditions.ROT.value"
// reads the ROT entry. data-decimals fixes the printed decimals.
function bound(res, path) {
  let v = res;
  for (const k of path) {
    if (Array.isArray(v) && !/^\d+$/.test(k)) v = v.find((e) => e && e.id === k);
    else v = v && typeof v === "object" ? v[k] : undefined;
  }
  return typeof v === "number" ? v : null;
}

function renderBindings() {
  const res = state.lastOk;
  $$("[data-bind]").forEach((el) => {
    const path = el.dataset.bind.split(".");
    const v = bound(res, path);
    if (el.dataset.decimals !== undefined) {
      const d = Number(el.dataset.decimals);
      el.textContent = v === null || !Number.isFinite(v) ? "n/a" : v.toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d });
      return;
    }
    const unit = path[0] === "kpi" && kpiDef(path[1]) ? kpiDef(path[1]).unit : el.dataset.unit || "";
    el.textContent = fmt(v, unit);
  });
}

export function render() {
  renderBindings();
  // The basis line follows the inputs on every route, since the print summary
  // copies it from wherever the reader prints.
  renderBasis();
  if (state.route === "dashboard") {
    renderStrip();
    renderTargets();
    renderFlags();
  }
  const hook = routeHooks[state.route];
  if (hook) hook(state);
  for (const fn of resultHooks) fn(state);
}

// ------------------------------------------------------------ routes

function showRoute() {
  const name = location.hash.replace(/^#/, "");
  state.route = ROUTES.includes(name) ? name : "dashboard";
  for (const r of ROUTES) {
    const sec = document.getElementById(`route-${r}`);
    if (sec) sec.hidden = r !== state.route;
  }
  $$(".tabs a").forEach((a) => {
    if (a.dataset.route === state.route) a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  });
  render();
}

function showEngine(engine) {
  const commit = String(engine.commit || "local");
  const short = commit.slice(0, 7);
  $("#engine-line").innerHTML = `Engine <code>${esc(engine.version)}</code>, commit <code>${esc(short)}</code>, ${esc(engine.preset)} preset, running in your browser as WebAssembly.`;
  $("#foot-version").textContent = engine.version;
  const foot = $("#foot-commit");
  if (/^[0-9a-f]{7,40}$/.test(commit)) {
    foot.innerHTML = `<a href="https://github.com/weewzr/NUSH2/commit/${esc(commit)}"><code>${esc(short)}</code></a>`;
  } else {
    foot.innerHTML = `<code>${esc(short)}</code> (local build)`;
  }
}

// The PFD module loads apart from the shell, so a fault there leaves the
// other routes working. It receives the shell's state and route hook.
function loadPfd() {
  import("./pfd.js?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2")
    .then((m) => m.initPfd({ state, registerRoute }))
    .catch((err) => {
      console.error(`The process flow diagram did not load: ${err && err.message ? err.message : err}`);
      if (state.route === "pfd") banner("bad", `<strong>The process flow diagram did not load.</strong> ${esc(String(err && err.message ? err.message : err))}`);
    });
}

async function init() {
  startWorker();
  window.addEventListener("hashchange", showRoute);
  showRoute();
  loadPfd();
  setStatus("busy", "Loading the engine registry");
  try {
    const { reply } = await rpc("registry", {});
    if (reply.status !== "ok") throw new Error(`registry status ${reply.status}`);
    state.registry = reply;
    state.inputs = defaults();
    showEngine(reply.engine);
    render();
    requestSolve();
  } catch (err) {
    setStatus("bad", "The engine did not load");
    banner("bad", `<strong>The engine did not load.</strong> ${esc(err.message)}`);
    selftestReport({ load_error: String(err.message) });
  }
}

init();
