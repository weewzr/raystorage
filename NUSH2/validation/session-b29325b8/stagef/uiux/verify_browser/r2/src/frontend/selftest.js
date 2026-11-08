// Browser self test. It runs only with ?selftest=1 on a site built by
// scripts/build_site.py and leaves the page alone otherwise. The verdict goes
// to <pre id="selftest-verdict"> as JSON and to document.body.dataset.selftest
// as "pass" or "fail"; scripts/selftest.py reads both over the DevTools
// protocol. Every number checked here comes from the engine or from
// data/*.json. This file compares values and computes no engineering quantity.

import { state, rpc, setInputs, resetInputs, fmt } from "./app.js?v=__COMMIT__";
import { applyScenario, scenarioInputs } from "./views.js?v=__COMMIT__";

const ON = new URLSearchParams(location.search).get("selftest") === "1";
const STAMP = "__COMMIT__";
const RAW = "__COM" + "MIT__";
const LOAD_MS = 30000;
const SOLVE_BUDGET_MS = 3000;
const MOVE_REL = 1e-9;
const ROUTE_WAIT_MS = 300;
const ROUTES = ["dashboard", "pfd", "parameters", "results", "export", "validation"];
// Visible text rules, generic patterns only, base64 so the page text carries
// none of the phrases: cohort label, schedule wording, allocation wording,
// matriculation number, declaration wording, course code, academic framing.
// Names and local folder names are checked by a scan kept outside the
// repository. The em dash is checked by code point.
const TEXT_RULES = [
  ["cohort", "XGJncm91cFtcc18tXSpcZHsxLDJ9XGI=", "i"],
  ["schedule", "XGJ0aW1lW1xzLV0/c2xvdHM/XGJ8XGJwcmVzZW50YXRpb24gc2xvdFxifFxiZGVhZGxpbmVzP1xifFxiZHVlIGRhdGVcYg==", "i"],
  ["allocation", "XGJzcGVha2VyIGFsbG9jYXRpb25cYnxcYnNwZWFraW5nICg/OnNwbGl0fHRpbWV8b3JkZXIpXGI=", "i"],
  ["matric", "XGJBXGR7N31bQS1aXVxi", "i"],
  ["declaration", "XGJBSVstIF11c2VcYnxcYkFJIGRlY2xhcmF0aW9uXGI=", "i"],
  ["course", "KD88IVtcdy1dKSg/OkNOfEVFfENTfE1BfE1FfENFfENNfEJOfENHfEdFfEVTfElFfFBDfFNUfEVHfExTTXxHRUF8R0VDfEdFTnxHRVNTKVxkezR9W0EtWl17MCwyfSg/IVtcdy1dKQ==", ""],
  ["academic", "XGJBWVxzP1xkezR9KD86XHM/L1xzP1xkezIsNH0pP1xifFxiYWNhZGVtaWMgeWVhclxifFxic2VtZXN0ZXJcYg==", "i"],
  // Pointers into working notes that no public file defines.
  ["pointer", "XGIoPzphbWluZV9zcGVjfG1lYV9jb2x1bW5zfG1lYV90aGVybW98ZWNvbl9saXR8aHRncl9paHhcLm1kfGlubFwubWR8aW5sX2NhbGNzX291dHB1dHxmaW5hbF92MV83fGF1ZGl0X3JlY29tcHV0ZXxVU0VSX0FOU1dFUlN8c2NyZWVuaW5nXC5weXxmcmVlemUgc2VjdGlvbnxwbGFuIHNlY3Rpb258UmVzdCByb3d8b2xkIHJlcG8oPzpzaXRvcnkpPylcYg==", "i"],
];
// The first SMR on the page must be written out as steam methane reforming (SMR).
const SMR_FIRST = "c3RlYW0gbWV0aGFuZSByZWZvcm1pbmcgXChTTVJcKQ==";

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
    // The page's first full solve fills the Level 3 and M6 caches. It is recorded
    // here, not judged, so scripts/selftest.py prints it against the plan budget,
    // with the wait from the registry request to the first result beside it.
    first_solve_ms: Number.isFinite(state.firstSolveMs) ? state.firstSolveMs : null,
    first_result_ms: Number.isFinite(state.firstResultMs) ? state.firstResultMs : null,
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
    const bytes = await (ok(await fetch("nush2.wasm?v=__COMMIT__", NO_STORE))).arrayBuffer();
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
    const build = await (ok(await fetch("build.json?v=__COMMIT__", NO_STORE))).json();
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
    const ref = await (ok(await fetch("data/reference.json?v=__COMMIT__", NO_STORE))).json();
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

  // 11. Case A parity: the Case A solve against the case_a_mea column of the
  // same Final_Design_Basis.csv rows the headline check reads.
  let ref = null;
  try {
    ref = await (ok(await fetch("data/reference.json?v=__COMMIT__", NO_STORE))).json();
    const tol = ref.tolerance;
    const { reply } = await solve({ ...defaults, basis: "case_a" }, true);
    const diffs = [];
    let compared = 0;
    for (const r of ref.kpi.filter((x) => typeof x.case_a_mea === "number")) {
      const v = reply.kpi ? reply.kpi[r.id] : undefined;
      compared += 1;
      if (typeof v !== "number" || !close(v, r.case_a_mea, tol)) diffs.push(`kpi.${r.id}: wasm ${v} against case_a_mea ${r.case_a_mea}`);
    }
    add("case_a_parity", reply.status === "ok" && compared > 0 && diffs.length === 0,
      `${reply.status}, ${compared} KPIs against the case_a_mea cells, relative ${tol.rel}${diffs.length ? `; ${diffs.length} differ: ${diffs.join("; ")}` : ""}`,
      { differences: diffs });
  } catch (e) {
    add("case_a_parity", false, String(e.message || e));
  }

  // 12. The MATURE_60Y scenario solved live equals its published rows of
  // results/Sensitivity_Results.csv at display precision.
  try {
    const sc = ref ? (ref.scenarios || []).find((s) => s.id === "MATURE_60Y") : null;
    if (!sc) throw new Error("data/reference.json has no MATURE_60Y scenario");
    const inputs = { ...defaults, ...scenarioInputs(state, sc) };
    const { reply } = await solve(inputs, true);
    const pairs = [["cost_s_per_t", sc.expected.ac_l5], ["cost_lac_dcf_s_per_t", sc.expected.lac_dcf]];
    const bad = [];
    const notes = [];
    for (const [id, exp] of pairs) {
      const v = reply.kpi ? reply.kpi[id] : undefined;
      const shown = fmt(v, "S$/t");
      const want = fmt(exp.value, "S$/t");
      const rel = typeof v === "number" ? Math.abs(v - exp.value) / Math.abs(exp.value) : NaN;
      notes.push(`${id} live ${shown} against ${exp.row.run_id} ${want} (relative ${rel.toExponential(1)})`);
      if (shown !== want || !close(v, exp.value, ref.tolerance)) bad.push(id);
    }
    add("mature_scenario_parity", reply.status === "ok" && bad.length === 0,
      `${reply.status}; inputs stage ${inputs.stage}, life ${inputs.life_y} y, n ${inputs.n}, gas ${inputs.gas_s_gj} S$/GJ; ${notes.join("; ")}${bad.length ? `; differ: ${bad.join(", ")}` : ""}`);
  } catch (e) {
    add("mature_scenario_parity", false, String(e.message || e));
  }

  await tail(checks, base, ref);
  finish(checks, t0);
}

/** Solve through the page with these inputs and wait until the reply is drawn. */
async function pageSolve(values, reset) {
  const prev = state.lastOk;
  if (reset) resetInputs();
  if (values) setInputs(values);
  const got = await until(() => state.lastOk !== prev && state.lastOk, 15000);
  await sleep(150);
  return got;
}

// Checks on the rendered page: every route renders, PFD bindings, charts,
// visible text, and no error caught anywhere.
async function tail(checks, base, ref) {
  const add = (id, ok, detail) => checks.push({ id, status: ok === null ? "skip" : ok ? "pass" : "fail", detail });
  const start = location.hash;
  for (const r of ROUTES) {
    location.hash = `#${r}`;
    await sleep(ROUTE_WAIT_MS);
  }

  // Every unit panel fills its seven sections.
  try {
    const pfd = await import("./pfd.js?v=__COMMIT__");
    const a = await until(() => { const x = pfd.auditPanels(); return x.ready ? x : null; }, LOAD_MS);
    add("panels_seven_sections", Boolean(a) && a.units > 0 && a.incomplete.length === 0,
      a ? `${a.units} panels${a.incomplete.length ? `; empty sections: ${a.incomplete.join("; ")}` : ", every section filled"}` : "the panels did not load");
    // Stream tags are stream ids of data/streams.json, or lines described as
    // having no row there; every one is a focus stop; rings follow the file.
    const s = await until(() => { const x = pfd.auditStreams(); return x.ready ? x : null; }, LOAD_MS);
    if (!s) {
      add("stream_tags_engine_ids", false, "data/streams.json did not load");
    } else {
      const unringed = s.crossing.filter((t) => !s.ringed.includes(t) && !s.noted.includes(t));
      const extraRing = s.ringed.filter((t) => !s.crossing.includes(t));
      const fails = [
        s.undescribed.length ? `no description: ${s.undescribed.join(", ")}` : "",
        s.not_in_file.length ? `not in data/streams.json: ${s.not_in_file.join(", ")}` : "",
        s.clash.length ? `non-engine tag equal to a stream id: ${s.clash.join(", ")}` : "",
        s.not_focusable.length ? `not focusable: ${s.not_focusable.join(", ")}` : "",
        unringed.length ? `crossing with no ring or note: ${unringed.join(", ")}` : "",
        extraRing.length ? `ring on a stream that does not cross: ${extraRing.join(", ")}` : "",
      ].filter(Boolean);
      add("stream_tags_engine_ids", fails.length === 0,
        `${s.engine.length} engine stream tags, ${s.other.length} lines with no row (${s.other.join(", ")}); crossing ${s.crossing.join(", ")}${fails.length ? `; ${fails.join("; ")}` : ""}`);
    }
  } catch (e) {
    add("panels_seven_sections", false, String(e.message || e));
  }

  // The stream link lands on its row of the Results stream table.
  try {
    location.hash = "#pfd";
    await sleep(ROUTE_WAIT_MS);
    document.dispatchEvent(new CustomEvent("nush2:stream", { detail: { id: "S12" } }));
    location.hash = "#results";
    const row = await until(() => document.querySelector('tr.is-target[data-stream-row="S12"]'), 10000);
    const r = row ? row.getBoundingClientRect() : null;
    const inView = Boolean(r && r.bottom > 0 && r.top < window.innerHeight);
    add("stream_link_scrolls", Boolean(row) && inView && document.activeElement === row,
      row ? `row S12 marked, top ${Math.round(r.top)} px of ${window.innerHeight}, focused ${document.activeElement === row}` : "no row S12 marked");
  } catch (e) {
    add("stream_link_scrolls", false, String(e.message || e));
  }

  // The dashboard strip and the cost table line read one verdict, and an
  // infeasible plant never shows Met.
  try {
    location.hash = "#dashboard";
    await sleep(ROUTE_WAIT_MS);
    const probes = [["default", null], ["rating 100 MWth", { rating_mwth: 100 }], ["10OAK_P46 with gas at 25 S$/GJ", { stage: "10OAK_P46", gas_s_gj: 25 }], ["Case A", { basis: "case_a" }]];
    const out = [];
    const bad = [];
    for (const [name, values] of probes) {
      const res = await pageSolve(values, true);
      if (!res) { bad.push(`${name}: no solve`); continue; }
      const c = res.targets.cost;
      const sel = c.cases.find((x) => x.selected);
      const tableMet = c.cases.some((x) => x.table && x.met);
      const strip = document.querySelector('#targets [data-target="cost"]');
      const stripA = document.querySelector('#targets [data-target="abatement"]');
      const line = document.querySelector("#dash-cost .target-line");
      const feasEl = document.getElementById("feasibility");
      const fails = [];
      if (!strip || strip.dataset.met !== String(Boolean(c.met))) fails.push("strip cost differs from targets.cost.met");
      if (!sel || Boolean(sel.met) !== Boolean(c.met)) fails.push("selected case differs from targets.cost.met");
      if (!stripA || stripA.dataset.met !== String(Boolean(res.targets.abatement.met))) fails.push("strip abatement differs");
      if (!line || line.dataset.met !== String(tableMet)) fails.push("table line differs from the table cases");
      if (sel && !sel.table && line && !line.textContent.includes("The selected case")) fails.push("table line omits the off-table selected case");
      if (res.feasibility.feasible === false) {
        if (c.met || res.targets.abatement.met || tableMet) fails.push("a target reads met on an infeasible plant");
        if (!feasEl || feasEl.hidden) fails.push("no feasibility line");
      }
      out.push(`${name}: feasible ${res.feasibility.feasible}, strip cost ${strip ? strip.dataset.met : "none"}, table ${line ? line.dataset.met : "none"}`);
      if (fails.length) bad.push(`${name}: ${fails.join(", ")}`);
    }
    await pageSolve(null, true);
    add("target_verdicts_agree", bad.length === 0, `${out.join("; ")}${bad.length ? `; ${bad.join("; ")}` : ""}`);
  } catch (e) {
    add("target_verdicts_agree", false, String(e.message || e));
  }

  // The scenario control sets MATURE_60Y and the strip shows its published
  // value; the base result comes back with the other choice.
  try {
    const sc = ref ? (ref.scenarios || []).find((x) => x.id === "MATURE_60Y") : null;
    if (!sc) throw new Error("no MATURE_60Y scenario");
    let prev = state.lastOk;
    applyScenario("mature");
    await until(() => state.lastOk !== prev && state.lastOk, 15000);
    await sleep(200);
    const tile = document.querySelector('#kpi-strip [data-kpi="cost_s_per_t"] .k-value');
    const shown = tile ? tile.firstChild.textContent : "";
    const radio = document.querySelector('input[name="scenario"][value="mature"]');
    const okM = shown === fmt(sc.expected.ac_l5.value, "S$/t") && Boolean(radio && radio.checked) && Boolean(document.querySelector('[data-scenario-note="mature"]'));
    // Any other input moved off its default leaves neither scenario: no radio is
    // checked and no tile names MATURE_60Y.
    const keys = new Set(Object.keys(scenarioInputs(state, sc)));
    const other = state.registry.params.find((p) => p.kind === "number" && !keys.has(p.id) && Number.isFinite(p.model_min) && p.model_min !== p.default);
    let okO = false;
    let otherText = "no number control outside the scenario keys";
    if (other) {
      prev = state.lastOk;
      setInputs({ [other.id]: other.model_min });
      await until(() => state.lastOk !== prev && state.lastOk, 15000);
      await sleep(200);
      const checked = document.querySelector('input[name="scenario"]:checked');
      const named = Array.from(document.querySelectorAll("#kpi-strip .k-scn")).some((e) => e.textContent.includes("MATURE_60Y"));
      okO = !checked && !named && Boolean(document.querySelector('[data-scenario-note="other"]'));
      otherText = `${other.id} at ${other.model_min}: radio ${checked ? checked.value : "none"}, tile names MATURE_60Y ${named}`;
    }
    prev = state.lastOk;
    applyScenario("base");
    await until(() => state.lastOk !== prev && state.lastOk, 15000);
    await sleep(200);
    const base = (ref.kpi.find((k) => k.id === "cost_s_per_t") || {}).value;
    const tile2 = document.querySelector('#kpi-strip [data-kpi="cost_s_per_t"] .k-value');
    const shown2 = tile2 ? tile2.firstChild.textContent : "";
    const radio2 = document.querySelector('input[name="scenario"][value="base"]');
    const okB = shown2 === fmt(base, "S$/t") && Boolean(radio2 && radio2.checked);
    add("scenario_control", okM && okO && okB, `MATURE_60Y tile ${shown} against ${fmt(sc.expected.ac_l5.value, "S$/t")}, radio ${Boolean(radio && radio.checked)}; then ${otherText}; base tile ${shown2} against ${fmt(base, "S$/t")}, radio ${Boolean(radio2 && radio2.checked)}`);
  } catch (e) {
    add("scenario_control", false, String(e.message || e));
  }
  for (const r of ROUTES) {
    location.hash = `#${r}`;
    await sleep(ROUTE_WAIT_MS);
  }

  await tourChecks(add);

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
  const firstSmr = text.search(/\bSMR\b/);
  if (firstSmr >= 0 && !new RegExp(`${atob(SMR_FIRST)}$`, "i").test(text.slice(0, firstSmr + 4))) hits.push("smr_first_use");
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

// ------------------------------------------------------------ process tour

// Animations the browser is running now, and those on the tour's own elements.
const running = () => document.getAnimations().filter((a) => a.playState === "running");
const tourRunning = () => running().filter((a) => {
  const t = a.effect && a.effect.target;
  return Boolean(t && t.closest && t.closest("#pfd-svg, #tour-card, #tour-bar"));
});

/** A reply value by the data-bind path convention, resolved here apart from the page code. */
function resolvePath(res, path) {
  let v = res;
  for (const k of path.split(".")) {
    if (Array.isArray(v) && !/^\d+$/.test(k)) v = v.find((e) => e && e.id === k);
    else v = v && typeof v === "object" ? v[k] : undefined;
  }
  return v && typeof v === "object" && "value" in v ? v.value : v;
}

/** The text a value must print as: fixed decimals, a fraction as a percentage, n/a for no number. */
function shownAs(v, unit, d) {
  if (typeof v !== "number" || !Number.isFinite(v)) return "n/a";
  return (unit === "-" ? v * 100 : v).toLocaleString("en-GB", { minimumFractionDigits: d, maximumFractionDigits: d });
}

/** Every number in the caption card against its reply field or results row. */
function captionNumbers(rows) {
  const out = { count: 0, bad: [] };
  for (const b of document.querySelectorAll("#tour-card .tn")) {
    out.count += 1;
    const v = b.dataset.path ? resolvePath(state.lastOk, b.dataset.path) : (rows[b.dataset.row] || {}).value;
    const want = shownAs(v, b.dataset.u, Number(b.dataset.d));
    if (b.textContent !== want) out.bad.push(`${b.dataset.path || b.dataset.row} shows ${b.textContent}, source ${want}`);
  }
  return out;
}

const press = (key) => (document.activeElement || document.body)
  .dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

// The guided tour of the PFD page: its steps name real units, streams and
// reply fields; it starts from its button, visits every step and exits with no
// error; each caption number equals its source; the keys work; reduced motion
// runs no animation and does not advance; the numbers follow the controls.
async function tourChecks(add) {
  location.hash = "#pfd";
  await sleep(ROUTE_WAIT_MS);
  let tour;
  let rows = {};
  try {
    tour = await import("./tour.js?v=__COMMIT__");
    rows = ((await (ok(await fetch("data/tour_rows.json?v=__COMMIT__", NO_STORE))).json()).rows) || {};
  } catch (e) {
    add("tour_steps_bound", false, `the tour did not load: ${e && e.message ? e.message : e}`);
    return;
  }
  await pageSolve(null, true);
  const ready = await until(() => tour.tourState().ready, LOAD_MS);
  const a = tour.tourAudit(state.lastOk);
  const unbound = [...a.missing_units, ...a.missing_streams, ...a.missing_marks, ...a.bad_paths, ...a.missing_rows, ...a.bad_keys];
  add("tour_steps_bound", Boolean(ready) && a.steps > 0 && unbound.length === 0,
    `${a.steps} steps; every unit, stream, label key, reply path and results row they name resolves${unbound.length ? `; unbound: ${unbound.join(", ")}` : ""}`);

  const n = a.steps;
  const reducedOS = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const start = document.getElementById("tour-start");
  const bar = document.getElementById("tour-bar");
  const title = document.getElementById("tour-title");
  const act = (name) => bar.querySelector(`[data-act="${name}"]`);

  // Start from the button, step through with Next, leave with Exit.
  const texts = [];
  try {
    const errs0 = errors.length;
    start.click();
    await until(() => tour.tourState().active, 2000);
    const st0 = tour.tourState();
    const focusTitle = document.activeElement === title;
    const visited = [];
    const numBad = [];
    const motion = [];
    const unlit = [];
    let numCount = 0;
    for (let i = 0; i < n; i += 1) {
      await sleep(150);
      const st = tour.tourState();
      visited.push(st.step + 1);
      texts.push(document.getElementById("tour-card").textContent);
      const nums = captionNumbers(rows);
      numCount += nums.count;
      numBad.push(...nums.bad.map((x) => `step ${st.step + 1}: ${x}`));
      if (!document.querySelector("#pfd-svg .tour-on")) unlit.push(st.step + 1);
      const dashes = tourRunning().filter((x) => x.effect.target.classList && x.effect.target.classList.contains("tour-flow")).length;
      if (!reducedOS && st.flows > 0 && dashes === 0) motion.push(`step ${st.step + 1}: no moving dashes`);
      if (reducedOS && tourRunning().length) motion.push(`step ${st.step + 1}: animation under reduced motion`);
      if (i < n - 1) act("next").click();
    }
    const last = tour.tourState();
    const endDisabled = act("next").getAttribute("aria-disabled") === "true";
    act("exit").click();
    await sleep(100);
    const left = !tour.tourState().active && document.activeElement === start
      && !document.querySelector("#pfd-svg .tour-on, #pfd-svg .tour-flow") && document.getElementById("tour-card").hidden;
    const newErrors = errors.slice(errs0);
    const inOrder = visited.length === n && visited.every((v, k) => v === k + 1);
    const fails = [
      st0.active ? "" : "did not start",
      focusTitle ? "" : "focus did not move to the step title",
      st0.playing === !reducedOS ? "" : `playing ${st0.playing} with reduced motion ${reducedOS}`,
      inOrder ? "" : `visited ${visited.join(" ")}`,
      last.step === n - 1 && endDisabled ? "" : "the last step does not end the walk",
      unlit.length ? `nothing lit on steps ${unlit.join(", ")}` : "",
      ...motion,
      left ? "" : "Exit did not close the tour and return focus to the start button",
      newErrors.length ? `${newErrors.length} error(s): ${newErrors.slice(0, 3).join(" | ")}` : "",
    ].filter(Boolean);
    add("tour_plays", fails.length === 0,
      `started from its button, ${visited.length} of ${n} steps visited in order, ${reducedOS ? "reduced motion" : "dashes moving on every step with flows"}, exited with focus on the start button${fails.length ? `; ${fails.join("; ")}` : ""}`);
    add("tour_numbers_live", numCount > 0 && numBad.length === 0,
      `${numCount} caption numbers over ${n} steps at the default inputs, each equal to its reply field or results row${numBad.length ? `; ${numBad.length} differ: ${numBad.slice(0, 6).join("; ")}` : ""}`);
  } catch (e) {
    add("tour_plays", false, String(e && e.message ? e.message : e));
  }

  // The prose of every step against the visible text rules, with no
  // placeholder left and no bare SMR.
  const all = texts.join(" ");
  const hits = TEXT_RULES.filter(([, b64, flags]) => new RegExp(atob(b64), flags).test(all)).map(([label]) => label);
  if (all.includes(String.fromCharCode(0x2014))) hits.push("emdash");
  if (/\bSMR\b/.test(all)) hits.push("bare SMR");
  if (/\{\w*\}/.test(all)) hits.push("placeholder");
  add("tour_text_rules", texts.length === n && hits.length === 0,
    `${all.length} characters over ${texts.length} steps${hits.length ? `; rules hit: ${hits.join(", ")}` : ""}`);

  // Keys: arrows step, Space pauses, Escape closes an open stream tip first
  // and then the tour, with focus back on the start button.
  try {
    start.click();
    await until(() => tour.tourState().active, 2000);
    press("ArrowRight");
    const s1 = tour.tourState().step;
    press("ArrowLeft");
    const s0 = tour.tourState().step;
    const p0 = tour.tourState().playing;
    press(" ");
    const p1 = tour.tourState().playing;
    press(" ");
    const p2 = tour.tourState().playing;
    const spaceOk = reducedOS ? !p0 && !p1 && !p2 : p0 && !p1 && p2;
    const g = document.querySelector('#pfd-svg [data-stream="S12"]');
    const tip = document.getElementById("pfd-tip");
    g.focus();
    // A headless page without window focus moves focus but fires no focus
    // event, so the event that opens the stream tip is sent here.
    if (tip.hidden) g.dispatchEvent(new FocusEvent("focus"));
    const tipOpen = !tip.hidden;
    press("Escape");
    const tipFirst = tip.hidden && tour.tourState().active;
    press("Escape");
    const out = !tour.tourState().active && document.activeElement === start;
    const buttons = [start, ...bar.querySelectorAll("[data-act]")].every((b) => b.tagName === "BUTTON" && b.type === "button");
    add("tour_keyboard", s1 === 1 && s0 === 0 && spaceOk && tipOpen && tipFirst && out && buttons,
      `ArrowRight to step ${s1 + 1}, ArrowLeft to step ${s0 + 1}, Space ${reducedOS ? "no play under reduced motion" : `playing ${p0} then ${p1} then ${p2}`}, Escape closed the stream tip first ${tipFirst}, then the tour with focus on the start button ${out}, every control a button ${buttons}`);
  } catch (e) {
    add("tour_keyboard", false, String(e && e.message ? e.message : e));
  } finally {
    tour.exitTour(false);
  }

  // Reduced motion: still steps, nothing running, no auto-advance, no Play.
  try {
    await sleep(600);
    const before = running().length;
    tour.startTour({ reduced: true });
    const bad = [];
    for (let i = 0; i < n; i += 1) {
      tour.goTour(i);
      await sleep(60);
      const extra = running().length - before;
      if (extra > 0 || tourRunning().length) bad.push(`step ${i + 1}: ${extra} more running`);
      if (document.querySelector("#pfd-svg .tour-flow")) bad.push(`step ${i + 1}: dashes drawn`);
    }
    tour.goTour(0);
    await sleep(400);
    const st = tour.tourState();
    const stays = st.step === 0 && !st.playing && st.still;
    const noPlay = act("play").hidden;
    tour.exitTour(false);
    add("tour_reduced_motion", bad.length === 0 && stays && noPlay,
      `${n} still steps with ${before} animation(s) running before and none added, stays on step ${st.step + 1}, Play hidden ${noPlay}${bad.length ? `; ${bad.slice(0, 5).join("; ")}` : ""}`);
  } catch (e) {
    add("tour_reduced_motion", false, String(e && e.message ? e.message : e));
  }

  // Auto-advance at a fast test pace runs to the last step and stops there;
  // the play-speed dwell of every step lies between 6 and 8 s, under 2 min in all.
  try {
    const dwellOk = a.dwell_ms.every((ms) => ms >= 6000 && ms <= 8000) && a.total_ms < 120000;
    let detail;
    let pass;
    if (reducedOS) {
      start.click();
      await sleep(500);
      const st = tour.tourState();
      tour.exitTour(false);
      pass = dwellOk && st.active && st.step === 0 && !st.playing;
      detail = `reduced motion: stays on step ${st.step + 1}, playing ${st.playing}`;
    } else {
      tour.startTour({ scale: 0.02 });
      const done = await until(() => tour.tourState().ended, 10000);
      const st = tour.tourState();
      tour.exitTour(false);
      pass = Boolean(done) && dwellOk && st.step === n - 1 && !st.playing;
      detail = `ran to step ${st.step + 1} of ${n} and stopped, playing ${st.playing}`;
    }
    add("tour_autoplay", pass, `${detail}; dwell ${Math.min(...a.dwell_ms) / 1000} to ${Math.max(...a.dwell_ms) / 1000} s a step, ${a.total_ms / 1000} s in all`);
  } catch (e) {
    add("tour_autoplay", false, String(e && e.message ? e.message : e));
  }

  // The numbers follow the controls: at a rating below the need the first
  // step shows the engine's infeasibility reason and the new rating, and the
  // last step states the abatement verdict the engine gives.
  try {
    const res = await pageSolve({ rating_mwth: 100 }, true);
    tour.startTour({ reduced: true, step: 0 });
    await sleep(80);
    const warn = document.querySelector("#tour-card .tour-warn");
    const warnOk = Boolean(res && res.feasibility && res.feasibility.feasible === false && warn && warn.textContent.includes(res.feasibility.reason));
    const n1 = captionNumbers(rows);
    const rating = document.querySelector('#tour-card .tn[data-path="kpi.rating_mwth"]');
    const ratingOk = Boolean(rating && rating.textContent === shownAs(resolvePath(res, "kpi.rating_mwth"), "MWth", 0));
    tour.goTour(n - 1);
    await sleep(60);
    const n13 = captionNumbers(rows);
    const verdict = document.querySelector("#tour-card .tour-text").textContent
      .includes(res.targets.abatement.met ? "is met at these inputs" : "is not met at these inputs");
    tour.exitTour(false);
    await pageSolve(null, true);
    const bad = [...n1.bad, ...n13.bad];
    add("tour_follows_controls", warnOk && ratingOk && verdict && bad.length === 0,
      `rating 100 MWth: infeasibility reason shown ${warnOk}, rating shown ${rating ? rating.textContent : "none"}, abatement verdict stated ${verdict}, ${n1.count + n13.count} numbers equal to the reply${bad.length ? `; ${bad.slice(0, 4).join("; ")}` : ""}`);
  } catch (e) {
    add("tour_follows_controls", false, String(e && e.message ? e.message : e));
    tour.exitTour(false);
  }
}

if (ON) {
  run().catch((e) => {
    errors.push(`selftest: ${e && e.message ? e.message : e}`);
    finish([{ id: "selftest_ran", status: "fail", detail: String(e && e.message ? e.message : e) }], performance.now());
  });
}
