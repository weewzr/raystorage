// Parameters page. Every control comes from the engine registry: its label,
// group, unit, default, hard range, evidence range, step, class and source.
// The page blocks a value outside the hard range and states the limit; the
// engine refuses it as well, so the limit holds when the page is bypassed.
// A value outside the evidence range is accepted and carries the engine flag.

import { setInput, resetInputs, fmt, unitLabel, esc, pretty, flagText } from "./app.js?v=ef21980aa4fef40961dee7fb4425c8109770b8c1";

const WARN = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 1.5l7 13H1z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/><path d="M8 6v4M8 12v.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>';
const STICKY = [
  ["surplus_mwth", "Surplus at rating"],
  ["abatement_t_y", "Abatement"],
  ["capture_plant", "Plant capture"],
  ["cost_s_per_t", "Full cost, L5"],
  ["cost_lac_dcf_s_per_t", "Full cost, DCF"],
];

let builtFor = null;
const refs = new Map();

const finite = (v) => typeof v === "number" && Number.isFinite(v);

/** Display name of a registry unit. Fractions stay fractions in the controls. */
function unitName(u) {
  if (u === "-") return "fraction";
  return u || "";
}

/** A registry number as the registry gives it, for range text. */
function plain(v) {
  return finite(v) ? String(v) : "n/a";
}

function decimalsOf(step) {
  const s = String(step);
  const i = s.indexOf(".");
  return i < 0 ? 0 : s.length - i - 1;
}

function rangeText(p) {
  const u = unitName(p.unit);
  const hard = `Allowed ${plain(p.hard_min)} to ${plain(p.hard_max)}${u ? ` ${u}` : ""}.`;
  let model = "";
  if (finite(p.model_min) && finite(p.model_max)) {
    model = p.model_min === p.model_max
      ? ` Inside the evidence range only at ${plain(p.model_min)}${u ? ` ${u}` : ""}.`
      : ` Evidence range ${plain(p.model_min)} to ${plain(p.model_max)}${u ? ` ${u}` : ""}.`;
  }
  return hard + model;
}

/**
 * The box shows the value the engine solved: on the step grid it prints to the
 * step's decimals, and a value typed off the grid prints as typed.
 */
function boxText(v, step) {
  const d = decimalsOf(step);
  const onGrid = Math.abs(v - Number(v.toFixed(d))) < 1e-12;
  return onGrid ? v.toFixed(d) : String(v);
}

function optionText(o) {
  const tag = o.tag && !["central", "headline", "approved"].includes(o.tag) ? ` (${o.tag})` : "";
  return `${o.label}${tag}`;
}

function sourceBlock(p) {
  const bits = [];
  if (p.source) bits.push(`<p><strong>Source.</strong> ${esc(p.source)}</p>`);
  if (p.note) bits.push(`<p>${esc(p.note)}</p>`);
  if (p.engine_id) bits.push(`<p>Engine registry id <code>${esc(p.engine_id)}</code>${p.rigour ? `, rigour ${esc(p.rigour)}` : ""}.</p>`);
  if (!bits.length) return "";
  return `<details><summary>Source and note</summary>${bits.join("")}</details>`;
}

function numberControl(p) {
  const id = `p-${p.id}`;
  const u = unitName(p.unit);
  return `<div class="param" data-row="${esc(p.id)}">
    <div class="p-head">
      <label for="${id}">${esc(pretty(p.label))}</label>
      ${u ? `<span class="p-unit">${esc(u)}</span>` : ""}
      ${p.class ? `<span class="chip" data-class="${esc(p.class)}" title="${esc(p.source || "")}">${esc(String(p.class).replace(/_/g, " "))}</span>` : ""}
    </div>
    <div class="p-inputs">
      <input type="range" id="${id}-range" data-param="${esc(p.id)}" min="${p.hard_min}" max="${p.hard_max}" step="${p.step}" aria-label="${esc(pretty(p.label))} slider" aria-describedby="${id}-range-text">
      <input type="number" id="${id}" data-param="${esc(p.id)}" min="${p.hard_min}" max="${p.hard_max}" step="${p.step}" inputmode="decimal" aria-describedby="${id}-range-text ${id}-error">
    </div>
    <span class="p-range" id="${id}-range-text">${esc(rangeText(p))}</span>
    <span class="p-error" id="${id}-error" role="alert" hidden></span>
    <span class="p-flag" id="${id}-flag" hidden>${WARN}<span></span></span>
    ${sourceBlock(p)}
  </div>`;
}

function enumControl(p) {
  const id = `p-${p.id}`;
  const opts = (p.options || []).map((o) => `<option value="${esc(o.value)}">${esc(optionText(o))}</option>`).join("");
  return `<div class="param" data-row="${esc(p.id)}">
    <div class="p-head">
      <label for="${id}">${esc(pretty(p.label))}</label>
      ${p.class ? `<span class="chip" data-class="${esc(p.class)}" title="${esc(p.source || "")}">${esc(String(p.class).replace(/_/g, " "))}</span>` : ""}
    </div>
    <div class="p-inputs"><select id="${id}" data-param="${esc(p.id)}">${opts}</select></div>
    <span class="p-flag" id="${id}-flag" hidden>${WARN}<span></span></span>
    ${sourceBlock(p)}
  </div>`;
}

/** Check a typed number against the registry hard range. Returns the error text or "". */
function hardError(p, raw) {
  const u = unitName(p.unit);
  const limit = `Allowed ${plain(p.hard_min)} to ${plain(p.hard_max)}${u ? ` ${u}` : ""}.`;
  if (String(raw).trim() === "") return `Enter a number. ${limit}`;
  const v = Number(raw);
  if (!finite(v)) return `Enter a number. ${limit}`;
  if (v < p.hard_min || v > p.hard_max) return `${String(raw).trim()} is outside the hard range and was not sent to the engine. ${limit}`;
  return "";
}

function build(mount, reg) {
  refs.clear();
  const groups = [];
  const byGroup = new Map();
  for (const p of reg.params) {
    if (!byGroup.has(p.group)) { byGroup.set(p.group, []); groups.push(p.group); }
    byGroup.get(p.group).push(p);
  }
  const sticky = `<div class="param-sticky" aria-live="polite" aria-label="Key results for these inputs">${STICKY.map(([k, label]) => `<span>${esc(label)} <strong data-sticky="${k}">n/a</strong></span>`).join("")}</div>`;
  const buttons = `<div class="btn-row">
    <button type="button" class="btn primary" data-act="reset">Reset to headline basis</button>
    <button type="button" class="btn" data-act="case_a">Case A, INL reference</button>
    <button type="button" class="btn" data-act="mature">Cost scenario MATURE_60Y</button>
  </div>
  <p class="note">Reset returns every input to the headline basis, the base result. The MATURE_60Y button returns every input to the headline basis and then sets the cost stage, reactor life, size exponent, gas price and reactor charge of that scenario; the Dashboard compares it with the base result. A change to any input after that leaves the named scenario, and the Dashboard says so.</p>`;
  const fields = groups.map((g) => `<fieldset class="param-group"><legend>${esc(g)}</legend>${byGroup.get(g).map((p) => (p.kind === "number" ? numberControl(p) : enumControl(p))).join("")}</fieldset>`).join("");
  const fixed = (reg.fixed || []).map((f) => `<tr><th scope="row">${esc(pretty(f.label))}</th><td class="num">${finite(f.value) ? esc(String(f.value)) : "n/a"}</td><td>${esc(f.unit || "")}</td><td>${esc(f.source || "")}${f.note ? `. ${esc(f.note)}` : ""}</td></tr>`).join("");
  const dropped = (reg.dropped || []).map((d) => `<li><code>${esc(d.id)}</code>: ${esc(d.reason)}</li>`).join("");
  mount.innerHTML = `${sticky}${buttons}${fields}
    <section class="view-section">
      <h3>Fixed values shown without a control</h3>
      <div class="table-scroll"><table class="data"><thead><tr><th scope="col">Quantity</th><th scope="col" class="num">Value</th><th scope="col">Unit</th><th scope="col">Source</th></tr></thead><tbody>${fixed}</tbody></table></div>
    </section>
    <section class="view-section">
      <h3>Planned controls with no response in this release</h3>
      <p class="note">The Headline solve does not read these inputs, so they carry no control. Each reason comes from the engine registry.</p>
      <ul class="msg-list">${dropped}</ul>
    </section>`;

  for (const p of reg.params) {
    const row = mount.querySelector(`[data-row="${CSS.escape(p.id)}"]`);
    const r = { p, row, flag: row.querySelector(".p-flag") };
    if (p.kind === "number") {
      r.range = row.querySelector('input[type="range"]');
      r.box = row.querySelector('input[type="number"]');
      r.err = row.querySelector(".p-error");
      const d = decimalsOf(p.step);
      r.range.addEventListener("input", () => {
        const v = Number(r.range.value);
        r.box.value = v.toFixed(d);
        showError(r, "");
        setInput(p.id, v);
      });
      r.box.addEventListener("input", () => showError(r, hardError(p, r.box.value)));
      r.box.addEventListener("change", () => {
        const e = hardError(p, r.box.value);
        showError(r, e);
        if (e) return;
        const v = Number(r.box.value);
        r.range.value = String(v);
        setInput(p.id, v);
      });
    } else {
      r.select = row.querySelector("select");
      r.select.addEventListener("change", () => setInput(p.id, r.select.value));
    }
    refs.set(p.id, r);
  }
  mount.querySelector('[data-act="reset"]').addEventListener("click", () => {
    for (const r of refs.values()) if (r.err) showError(r, "");
    resetInputs();
  });
  mount.querySelector('[data-act="case_a"]').addEventListener("click", () => setInput("basis", "case_a"));
  // views.js owns the scenario, which it reads from data/reference.json.
  mount.querySelector('[data-act="mature"]').addEventListener("click", () => document.dispatchEvent(new CustomEvent("nush2:scenario", { detail: "mature" })));
}

function showError(r, text) {
  r.err.textContent = text;
  r.err.hidden = !text;
  if (text) r.box.setAttribute("aria-invalid", "true");
  else r.box.removeAttribute("aria-invalid");
  r.box.classList.toggle("invalid", Boolean(text));
}

function sync(st) {
  const flags = st.result && Array.isArray(st.result.flags) ? st.result.flags : [];
  for (const [id, r] of refs) {
    const v = st.inputs[id];
    if (r.p.kind === "number") {
      const blocked = !r.err.hidden;
      if (blocked) {
        // The shell clears field marks on every reply; a blocked entry keeps its mark.
        r.box.setAttribute("aria-invalid", "true");
        r.box.classList.add("invalid");
      }
      if (document.activeElement !== r.box && !blocked && finite(v)) r.box.value = boxText(v, r.p.step);
      if (document.activeElement !== r.range && finite(v)) r.range.value = String(v);
    } else if (r.select && document.activeElement !== r.select && v !== undefined) {
      r.select.value = String(v);
    }
    const f = flags.find((x) => x.param === id);
    r.flag.hidden = !f;
    r.flag.lastElementChild.textContent = f ? `Outside evidence range. ${flagText(f.text)}.` : "";
  }
  const kpi = st.lastOk && st.lastOk.kpi ? st.lastOk.kpi : {};
  const defs = st.registry ? st.registry.kpis : [];
  for (const [k] of STICKY) {
    const el = document.querySelector(`[data-sticky="${k}"]`);
    if (!el) continue;
    const d = defs.find((x) => x.id === k);
    const unit = d ? d.unit : "";
    const v = kpi[k];
    el.textContent = finite(v) ? `${fmt(v, unit)} ${unitLabel(unit)}` : "n/a";
  }
}

/** Route renderer for #parameters. Builds once per registry, then syncs. */
export function renderParameters(st) {
  const mount = document.querySelector('[data-mount="controls"]');
  if (!mount) return;
  if (!st.registry) {
    mount.innerHTML = '<p class="note">Loading the engine registry.</p>';
    builtFor = null;
    return;
  }
  if (builtFor !== st.registry) {
    build(mount, st.registry);
    builtFor = st.registry;
  }
  sync(st);
}
