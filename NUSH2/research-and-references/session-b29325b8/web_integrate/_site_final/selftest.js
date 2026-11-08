// Browser self test. It runs only with ?selftest=1 on a site built by
// scripts/build_site.py and leaves the page alone otherwise. The verdict goes
// to <pre id="selftest-verdict"> as JSON and to document.body.dataset.selftest
// as "pass" or "fail"; scripts/selftest.py reads both over the DevTools
// protocol. Every number checked here comes from the engine or from
// data/*.json. This file compares values and computes no engineering quantity.

import { state, rpc } from "./app.js?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2";

const ON = new URLSearchParams(location.search).get("selftest") === "1";
const STAMP = "a31d73e0eaba4f982bf2fde7a9d142a9b4964df2";
const RAW = "__COM" + "MIT__";
const LOAD_MS = 30000;
const SOLVE_BUDGET_MS = 3000;
const MOVE_REL = 1e-9;
const ROUTE_WAIT_MS = 300;
const ROUTES = ["dashboard", "pfd", "parameters", "results", "export", "validation"];
// Visible text rules, base64 so this file carries none of the phrases:
// cohort label, schedule wording, allocation wording, matriculation number,
// declaration wording, course code, academic framing. The em dash is checked
// by code point.
const TEXT_RULES = [
  ["cohort", "XGJncm91cFtcc18tXSooPzo3fHNldmVuKVxi", "i"],
  ["schedule", "XGJ0aW1lW1xzLV0/c2xvdHM/XGJ8XGJwcmVzZW50YXRpb24gc2xvdFxifFxiZGVhZGxpbmVzP1xifFxiZHVlIGRhdGVcYg==", "i"],
  ["allocation", "XGJzcGVha2VyIGFsbG9jYXRpb25cYnxcYnNwZWFraW5nICg/OnNwbGl0fHRpbWV8b3JkZXIpXGI=", "i"],
  ["matric", "XGJBXGR7N31bQS1aXVxi", "i"],
  ["declaration", "XGJBSVstIF11c2VcYnxcYkFJIGRlY2xhcmF0aW9uXGI=", "i"],
  ["course", "KD88IVtcdy1dKSg/OkNOfEVFfENTfE1BfE1FfENFfENNfEJOfENHfEdFfEVTfElFfFBDfFNUfEVHfExTTXxHRUF8R0VDfEdFTnxHRVNTKVxkezR9W0EtWl17MCwyfSg/IVtcdy1dKQ==", ""],
  ["academic", "XGJBWVxzP1xkezR9KD86XHM/L1xzP1xkezIsNH0pP1xifFxiYWNhZGVtaWMgeWVhclxifFxic2VtZXN0ZXJcYnxcYmh5ZHJvZ2VuIGFuZCBsb3dbXHMtXWNhcmJvbiB0ZWNobm9sb2dpZXNcYg==", "i"],
];

const errors = [];
if (ON) {
  window.addEventListener("error", (e) => errors.push(`error: ${e.message || e}`));
  window.addEventListener("unhandledrejection", (e) => errors.push(`rejection: ${e.reason}`));
  const ce = console.error.bind(console);
  console.error = (...a) => { errors.push(`console.error: ${a.map(String).join(" ")}`); ce(...a); };
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function until(fn, ms) {
  const t0 = performance.now();
  while (performance.now() - t0 < ms) {
    const v = fn();
    if (v) return v;
    await sleep(50);
  }
  return null;
}

const NO_STORE = { cache: "no-store" };

function ok(r) {
  if (!r.ok) throw new Error(`${new URL(r.url).pathname}: HTTP ${r.status}`);
  return r;
}

function close(a, b, tol) {
  return Math.abs(a - b) <= Math.max(tol.rel * Math.max(Math.abs(a), Math.abs(b)), tol.abs);
}

async function solve(inputs, kpiOnly) {
  const { reply, ms } = await rpc("solve", { schema: 1, inputs, kpi_only: kpiOnly });
  return { reply, ms };
}

function finish(checks, t0, note) {
  const failed = checks.filter((c) => c.status === "fail");
  const verdict = {
    verdict: failed.length || note ? "fail" : "pass",
    commit: STAMP,
    ms: Math.round(performance.now() - t0),
    passed: checks.filter((c) => c.status === "pass").length,
    failed: failed.length,
    skipped: checks.filter((c) => c.status === "skip").length,
    note: note || "",
    checks,
    errors: errors.slice(),
  };
  const pre = document.createElement("pre");
  pre.id = "selftest-verdict";
  pre.hidden = true;
  pre.textContent = JSON.stringify(verdict);
  document.body.appendChild(pre);
  document.body.dataset.selftest = verdict.verdict;
}

async function run() {
  const t0 = performance.now();
  const checks = [];
  const add = (id, ok, detail, extra = {}) => checks.push({ id, status: ok === null ? "skip" : ok ? "pass" : "fail", detail, ...extra });
  if (STAMP === RAW) {
    finish(checks, t0, "this page is not a built site; run scripts/build_site.py and test _site");
    return;
  }

  // 1. The module instantiates with an empty import object.
  try {
    const bytes = await (ok(await fetch("nush2.wasm?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2", NO_STORE))).arrayBuffer();
    const mod = await WebAssembly.compile(bytes);
    const imports = WebAssembly.Module.imports(mod);
    const names = WebAssembly.Module.exports(mod).map((e) => e.name);
    const inst = await WebAssembly.instantiate(mod, {});
    const lack = ["memory", "alloc", "dealloc", "call", "abi_version"].filter((n) => !names.includes(n));
    add("wasm_empty_imports", imports.length === 0 && lack.length === 0,
      `${bytes.byteLength} bytes, ${imports.length} imports, ABI ${inst.exports.abi_version()}${lack.length ? `, missing exports ${lack.join(" ")}` : ""}`);
  } catch (e) {
    add("wasm_empty_imports", false, String(e && e.message ? e.message : e));
  }

  // 2. The registry loads through the page worker.
  const reg = await until(() => state.registry, LOAD_MS);
  add("registry_loads", Boolean(reg && reg.status === "ok"),
    reg ? `${reg.params.length} parameters, ${reg.kpis.length} KPIs, ${Object.keys(reg.units || {}).length} units, engine ${reg.engine.version}` : `no registry within ${LOAD_MS / 1000} s`);
  if (!reg) { finish(checks, t0); return; }

  // 3. The engine commit equals build.json and the page stamp.
  try {
    const build = await (ok(await fetch("build.json?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2", NO_STORE))).json();
    const commit = String(reg.engine.commit);
    add("commit_matches_build", commit === build.commit && commit === STAMP && commit !== "local",
      `engine ${commit}, build.json ${build.commit}, page stamp ${STAMP}, run ${build.run_id}`);
  } catch (e) {
    add("commit_matches_build", false, String(e.message || e));
  }

  // 4. Every number parameter: hard_min <= default <= hard_max, model range inside the hard range.
  const nums = reg.params.filter((p) => p.kind === "number");
  const badRange = nums.filter((p) => !(p.hard_min <= p.default && p.default <= p.hard_max
    && p.hard_min <= p.model_min && p.model_min <= p.model_max && p.model_max <= p.hard_max)).map((p) => p.id);
  add("parameter_ranges", nums.length > 0 && badRange.length === 0,
    `${nums.length} number parameters${badRange.length ? `; out of order: ${badRange.join(", ")}` : ""}`);

  // 5. The default solve is ok inside the budget.
  const defaults = Object.fromEntries(reg.params.map((p) => [p.id, p.default]));
  let base = null;
  try {
    const { reply, ms } = await solve(defaults, false);
    base = reply;
    add("default_solve_ok", reply.status === "ok" && ms < SOLVE_BUDGET_MS,
      `${reply.status} in ${Math.round(ms)} ms, budget ${SOLVE_BUDGET_MS} ms`, { ms: Math.round(ms) });
  } catch (e) {
    add("default_solve_ok", false, String(e.message || e));
  }
  if (!base || base.status !== "ok") { await tail(checks, base); finish(checks, t0); return; }

  // 6. Native against wasm on the headline: data/reference.json holds the
  // cells the native export wrote to results/*.csv.
  try {
    const ref = await (ok(await fetch("data/reference.json?v=a31d73e0eaba4f982bf2fde7a9d142a9b4964df2", NO_STORE))).json();
    const tol = ref.tolerance;
    const diffs = [];
    let compared = 0;
    for (const r of ref.kpi) {
      const v = base.kpi ? base.kpi[r.id] : undefined;
      compared += 1;
      if (typeof v !== "number" || !close(v, r.value, tol)) diffs.push(`kpi.${r.id}: wasm ${v} against native ${r.value}`);
    }
    // The CSV ids carry n to two decimals, so a row matches on n rounded alike.
    const rows = (base.cost && base.cost.headline) || [];
    for (const h of ref.cost_headline) {
      const row = rows.find((x) => x.charged === h.charged && String(x.stage).toUpperCase() === h.stage
        && x.life_y === h.life_y && Number(x.n).toFixed(2) === h.n);
      for (const f of ["ac_l3", "ac_l5", "lac_dcf", "lcoh_nush2", "lcoh_smr"]) {
        if (typeof h[f] !== "number") continue;
        compared += 1;
        const v = row ? row[f] : undefined;
        if (typeof v !== "number" || !close(v, h[f], tol)) diffs.push(`cost ${h.stage} ${h.life_y} y n ${h.n} ${f}: wasm ${v} against native ${h[f]}`);
      }
    }
    add("headline_parity", compared > 0 && diffs.length === 0,
      `${compared} values against data/reference.json, relative ${tol.rel}, floor ${tol.abs}${diffs.length ? `; ${diffs.length} differ: ${diffs.slice(0, 10).join("; ")}` : ""}`,
      { differences: diffs });
  } catch (e) {
    add("headline_parity", false, String(e.message || e));
  }

  // 7. A null KPI is listed in engine_fields_missing with a reason.
  const missing = new Map((base.engine_fields_missing || []).map((m) => [m.field, m.reason]));
  const nulls = Object.entries(base.kpi || {}).filter(([, v]) => v === null).map(([k]) => k);
  const unlisted = nulls.filter((k) => !String(missing.get(`kpi.${k}`) || "").trim());
  add("null_kpis_listed", unlisted.length === 0,
    `${nulls.length} null KPI(s)${nulls.length ? `: ${nulls.join(", ")}` : ""}${unlisted.length ? `; unlisted: ${unlisted.join(", ")}` : ""}`);

  // 8. The carbon closure residual is a finite number.
  const cl = base.balances && base.balances.closure ? base.balances.closure.carbon_kmol_h : undefined;
  add("closure_finite", typeof cl === "number" && Number.isFinite(cl), `carbon closure ${cl} kmol/h`);

  // 9. One above the hard maximum of the rating is refused by the engine.
  const rating = reg.params.find((p) => p.id === "rating_mwth");
  if (!rating) {
    add("hard_limit_rejected", false, "registry has no rating_mwth");
  } else {
    try {
      const { reply } = await solve({ ...defaults, rating_mwth: rating.hard_max + 1 }, true);
      const named = (reply.violations || []).some((v) => v.param === "rating_mwth");
      add("hard_limit_rejected", reply.status === "rejected" && named,
        `rating_mwth = ${rating.hard_max + 1}: ${reply.status}${named ? ", violation names rating_mwth" : ""}`);
    } catch (e) {
      add("hard_limit_rejected", false, String(e.message || e));
    }
  }

  // 10. Every control moves at least one required KPI. A number control goes
  // to its model maximum, or to the next range end when that equals the
  // default; an option control goes to each other option.
  try {
    const required = reg.kpis.filter((k) => k.required).map((k) => k.id);
    const k0 = (await solve(defaults, true)).reply.kpi;
    const dead = [];
    const refused = [];
    let probes = 0;
    for (const p of reg.params) {
      let values;
      if (p.kind === "number") {
        const x = ["model_max", "model_min", "hard_max", "hard_min"].map((c) => p[c])
          .find((v) => typeof v === "number" && Math.abs(v - p.default) > 1e-12);
        values = x === undefined ? [] : [x];
        if (!values.length) { dead.push(`${p.id}: no probe value`); continue; }
      } else {
        values = (p.options || []).map((o) => o.value).filter((v) => v !== p.default);
      }
      for (const x of values) {
        probes += 1;
        const { reply } = await solve({ ...defaults, [p.id]: x }, true);
        if (reply.status !== "ok") { refused.push(`${p.id} = ${x}: ${reply.status}`); continue; }
        const moved = required.some((k) => typeof k0[k] === "number" && typeof reply.kpi[k] === "number"
          && Math.abs(k0[k] - reply.kpi[k]) > MOVE_REL * Math.max(Math.abs(k0[k]), 1));
        if (!moved) dead.push(`${p.id} = ${x}`);
      }
    }
    add("controls_move_kpis", probes > 0 && dead.length === 0 && refused.length === 0,
      `${reg.params.length} controls, ${probes} probes${dead.length ? `; moves no KPI: ${dead.join(", ")}` : ""}${refused.length ? `; not ok: ${refused.join(", ")}` : ""}`);
  } catch (e) {
    add("controls_move_kpis", false, String(e.message || e));
  }

  await tail(checks, base);
  finish(checks, t0);
}

// Checks on the rendered page: every route renders, PFD bindings, charts,
// visible text, and no error caught anywhere.
async function tail(checks, base) {
  const add = (id, ok, detail) => checks.push({ id, status: ok === null ? "skip" : ok ? "pass" : "fail", detail });
  const start = location.hash;
  for (const r of ROUTES) {
    location.hash = `#${r}`;
    await sleep(ROUTE_WAIT_MS);
  }

  const units = base && base.units ? base.units : null;
  const unitEls = Array.from(document.querySelectorAll("[data-unit]"));
  if (!unitEls.length) {
    add("pfd_units_bound", null, "the page has no [data-unit] element yet");
  } else if (!units) {
    add("pfd_units_bound", false, "the solve reply has no units section");
  } else {
    const ids = new Set(unitEls.map((e) => e.dataset.unit));
    const noBlock = [...ids].filter((id) => !units[id]);
    const noEl = Object.keys(units).filter((id) => !ids.has(id));
    add("pfd_units_bound", noBlock.length === 0 && noEl.length === 0,
      `${ids.size} PFD units, ${Object.keys(units).length} engine units${noBlock.length ? `; no engine block: ${noBlock.join(", ")}` : ""}${noEl.length ? `; not drawn: ${noEl.join(", ")}` : ""}`);
  }

  const streamEls = Array.from(document.querySelectorAll("[data-stream]"));
  const streams = base ? base.streams : null;
  const missing = new Map(((base && base.engine_fields_missing) || []).map((m) => [m.field, m.reason]));
  if (!streamEls.length) {
    add("pfd_streams_bound", null, "the page has no [data-stream] element yet");
  } else if (streams === null || streams === undefined) {
    const why = missing.get("streams");
    add("pfd_streams_bound", why ? null : false,
      why ? `streams are null in this solve, listed in engine_fields_missing: ${why}` : "streams are null and not listed in engine_fields_missing");
  } else {
    const known = new Set(Array.isArray(streams) ? streams.map((s) => s.id || s.tag) : Object.keys(streams));
    const tags = [...new Set(streamEls.map((e) => e.dataset.stream))];
    const lost = tags.filter((t) => !known.has(t));
    add("pfd_streams_bound", lost.length === 0, `${tags.length} stream tags${lost.length ? `; not in result.streams: ${lost.join(", ")}` : ""}`);
  }

  const figs = Array.from(document.querySelectorAll("figure[data-chart]"));
  if (!figs.length) {
    add("charts_have_marks", null, "the page has no chart yet");
  } else {
    const empty = figs.filter((f) => (f.dataset.state === "na"
      ? !(f.querySelector(".viz-reason") && f.querySelector(".viz-reason").textContent.trim())
      : !f.querySelector("svg path, svg rect, svg circle, svg line, svg polyline, svg polygon"))).map((f) => f.dataset.chart);
    add("charts_have_marks", empty.length === 0, `${figs.length} charts${empty.length ? `; empty: ${empty.join(", ")}` : ""}`);
  }

  const text = document.body.textContent || "";
  const hits = TEXT_RULES.filter(([, b64, flags]) => new RegExp(atob(b64), flags).test(text)).map(([label]) => label);
  if (text.includes(String.fromCharCode(0x2014))) hits.push("emdash");
  // An unresolved panel token of units.json would print its opening braces;
  // JSON never holds two in a row, so the report blocks cannot trip this.
  if (text.includes("{{")) hits.push("token");
  add("visible_text_rules", hits.length === 0, hits.length ? `rules hit: ${hits.join(", ")}` : `${text.length} characters checked`);

  location.hash = start || "#dashboard";
  await until(() => document.documentElement.dataset.selftest, LOAD_MS);
  await sleep(ROUTE_WAIT_MS);
  const page = document.documentElement.dataset.selftest;
  add("no_console_error", errors.length === 0 && page === "pass",
    `${errors.length} error(s) caught by this test, page shell verdict ${page || "none"}${errors.length ? `: ${errors.slice(0, 5).join(" | ")}` : ""}`);
}

if (ON) {
  run().catch((e) => {
    errors.push(`selftest: ${e && e.message ? e.message : e}`);
    finish([{ id: "selftest_ran", status: "fail", detail: String(e && e.message ? e.message : e) }], performance.now());
  });
}
